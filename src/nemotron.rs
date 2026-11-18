//! Desktop worker adapter over the shared Nemotron driver.
use crate::observe::Observer;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::oneshot;
pub use transcrust_core::nemotron::*;

const WORKER_IDLE_CHECK_INTERVAL: Duration = Duration::from_secs(1);
const WORKER_STARTUP_TIMEOUT: Duration = Duration::from_secs(20);

pub struct NemotronService {
    inner: Arc<ServiceInner>,
}

struct ServiceInner {
    model_dir: String,
    tx: Mutex<Option<mpsc::Sender<Job>>>,
    idle_timeout_secs: u64,
}

struct Job {
    observer: Observer,
    audio_rx: mpsc::Receiver<Vec<f32>>,
    source_sample_rate: u32,
    reply_tx: oneshot::Sender<Result<String, String>>,
}

impl Clone for NemotronService {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl NemotronService {
    pub fn new(model_dir: impl Into<String>, idle_timeout_secs: u64) -> Result<Self, String> {
        let model_dir = model_dir.into();
        // Fail at construction with a named directory rather than 20 s later on
        // the worker-startup timeout, same as the other two services.
        if !crate::model::has_nemotron_model(std::path::Path::new(&model_dir)) {
            return Err(format!("incomplete Nemotron model directory: {model_dir}"));
        }
        Ok(Self {
            inner: Arc::new(ServiceInner {
                model_dir,
                tx: Mutex::new(None),
                idle_timeout_secs,
            }),
        })
    }

    /// Release the model now instead of waiting out `idle_timeout_secs`.
    pub fn shutdown(&self) {
        if let Ok(mut guard) = self.inner.tx.lock() {
            *guard = None;
        }
    }

    pub async fn transcribe(
        &self,
        observer: Observer,
        audio_rx: mpsc::Receiver<Vec<f32>>,
        source_sample_rate: u32,
    ) -> Result<String, String> {
        let mut audio_rx = Some(audio_rx);
        for attempt in 0..2 {
            let tx = self.ensure_worker(&observer)?;
            let (reply_tx, reply_rx) = oneshot::channel();
            let rx = audio_rx.take().expect("audio_rx should be available");
            match tx.send(Job {
                observer: observer.clone(),
                audio_rx: rx,
                source_sample_rate,
                reply_tx,
            }) {
                Ok(()) => {
                    return reply_rx
                        .await
                        .map_err(|e| format!("nemotron worker reply failed: {e}"))?
                }
                Err(mpsc::SendError(job)) => {
                    // Worker exited on its idle timeout: recover the receiver
                    // and retry against a fresh one.
                    audio_rx = Some(job.audio_rx);
                    if attempt == 0 {
                        observer.phase("worker.retry", "worker unloaded, respawning");
                    }
                }
            }
        }
        Err("failed to send transcription job after retries".into())
    }

    fn ensure_worker(&self, observer: &Observer) -> Result<mpsc::Sender<Job>, String> {
        let mut guard = self
            .inner
            .tx
            .lock()
            .map_err(|_| "nemotron worker state lock poisoned".to_string())?;
        if let Some(tx) = guard.as_ref() {
            return Ok(tx.clone());
        }

        observer.phase(
            "worker.start",
            "starting Nemotron worker on first transcription",
        );
        let (tx, rx) = mpsc::channel::<Job>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();
        let inner = Arc::clone(&self.inner);
        let startup_observer = observer.clone();

        std::thread::Builder::new()
            .name("transcrust-nemotron".into())
            .spawn(move || worker_main(startup_observer, inner, rx, ready_tx))
            .map_err(|e| format!("failed to spawn nemotron worker: {e}"))?;

        match ready_rx.recv_timeout(WORKER_STARTUP_TIMEOUT) {
            Ok(Ok(())) => {
                *guard = Some(tx.clone());
                Ok(tx)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err("timed out waiting for nemotron worker startup".into()),
        }
    }
}

fn worker_main(
    observer: Observer,
    inner: Arc<ServiceInner>,
    rx: mpsc::Receiver<Job>,
    ready_tx: mpsc::Sender<Result<(), String>>,
) {
    let idle_timeout = Duration::from_secs(inner.idle_timeout_secs);
    observer.phase("worker.start", "loading Nemotron model on demand");

    let model_dir = std::path::Path::new(&inner.model_dir);
    let mut model = match Nemotron::load(model_dir, Tuning::for_export(model_dir)) {
        Ok(model) => {
            observer.phase(
                "worker.start",
                &format!(
                    "Nemotron model loaded, {} ms chunks",
                    model.chunk_frames() * 10
                ),
            );
            observer.notify(
                "Transcrust ready",
                "Nemotron model loaded and worker is ready.",
            );
            let _ = ready_tx.send(Ok(()));
            model
        }
        Err(e) => {
            observer.error("worker.start", &e);
            let _ = ready_tx.send(Err(e));
            return;
        }
    };

    let mut last_activity = Instant::now();

    loop {
        match rx.recv_timeout(WORKER_IDLE_CHECK_INTERVAL) {
            Ok(job) => {
                let result = transcribe_job(
                    &job.observer,
                    &mut model,
                    job.audio_rx,
                    job.source_sample_rate,
                );
                last_activity = Instant::now();
                let _ = job.reply_tx.send(result);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if last_activity.elapsed() > idle_timeout {
                    if let Ok(mut guard) = inner.tx.lock() {
                        *guard = None;
                    }
                    drop(rx);
                    drop(model);
                    observer.phase(
                        "worker.idle",
                        &format!(
                            "model unloaded after {}s inactivity",
                            inner.idle_timeout_secs
                        ),
                    );
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                drop(model);
                observer.phase("worker.unload", "model dropped on switch");
                observer.notify("Transcrust", "Previous engine unloaded");
                return;
            }
        }
    }

    observer.phase("worker.stop", "worker channel closed");
}

fn transcribe_job(
    observer: &Observer,
    model: &mut Nemotron,
    audio_rx: mpsc::Receiver<Vec<f32>>,
    source_sample_rate: u32,
) -> Result<String, String> {
    // Encode while the user is still talking.
    //
    // The batch engines drain the capture and then work; a cache-aware
    // streaming encoder is built to do the work *during* the capture, so the
    // wait after the toggle is one chunk plus a tail decode rather than the
    // whole capture's compute. Both the resampler and the frontend carry state
    // across chunk boundaries, and both are pinned by tests against their batch
    // equivalents — without that this path would quietly disagree with `--wav`.
    observer.phase("transcription.stream", "encoding while recording");
    let mut resampler = crate::audio::StreamingResampler::new(source_sample_rate, 16_000);
    let mut frontend = model.streaming_frontend()?;
    let mut state = model.stream_begin();
    let started = Instant::now();
    let mut samples_in = 0usize;
    let mut chunk_count = 0usize;
    let mut busy = std::time::Duration::ZERO;

    while let Ok(chunk) = audio_rx.recv() {
        chunk_count += 1;
        samples_in += chunk.len();
        let work = Instant::now();
        let audio_16k = resampler.push(&chunk);
        if let Some(mel) = frontend.push(&audio_16k)? {
            model.stream_push(&mut state, &mel)?;
        }
        busy += work.elapsed();
    }

    if samples_in < (source_sample_rate as usize / 10) {
        return Ok(String::new());
    }

    // The tail: whatever the filters were still holding, then the last partial
    // chunk. This is the only part the user actually waits for.
    let tail = Instant::now();
    let audio_16k = resampler.finish();
    if let Some(mel) = frontend.push(&audio_16k)? {
        model.stream_push(&mut state, &mel)?;
    }
    if let Some(mel) = frontend.finish()? {
        model.stream_push(&mut state, &mel)?;
    }
    let (text, chunks) = model.stream_finish(&mut state)?;
    let tail = tail.elapsed();

    observer.phase(
        "transcription.stream",
        &format!(
            "{chunk_count} audio chunks, {chunks} encoder chunks, {:.2}s encoding hidden under {:.2}s of speech, {:.2}s tail",
            busy.as_secs_f64(),
            started.elapsed().as_secs_f64() - tail.as_secs_f64(),
            tail.as_secs_f64()
        ),
    );
    Ok(text.trim().to_string())
}
