//! Desktop lifecycle adapter. Graph, weights and segmentation are native Rust.
use crate::observe::Observer;
use std::{
    path::Path,
    sync::{mpsc, Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::oneshot;
use transcrust_phonon::{segment::Segmenter, Phonon};

#[derive(Clone)]
pub struct PhononService {
    inner: Arc<Inner>,
}
struct Inner {
    dir: String,
    idle: u64,
    window_seconds: u32,
    tx: Mutex<Option<mpsc::Sender<Job>>>,
}
struct Job {
    observer: Observer,
    rx: mpsc::Receiver<Vec<f32>>,
    rate: u32,
    window_seconds: u32,
    reply: oneshot::Sender<Result<String, String>>,
}
impl PhononService {
    /// Start consuming microphone audio now; delivery still waits for release.
    pub fn start_capture(
        &self,
        observer: Observer,
        audio: mpsc::Receiver<Vec<f32>>,
        rate: u32,
        bank: bool,
    ) -> (
        tokio::task::JoinHandle<Result<String, String>>,
        mpsc::Receiver<Vec<f32>>,
    ) {
        let (engine_audio, banked_audio) = tee_capture(audio, bank);
        let service = self.clone();
        let result =
            tokio::spawn(async move { service.transcribe(observer, engine_audio, rate).await });
        (result, banked_audio)
    }

    pub fn new(dir: String, idle: u64) -> Result<Self, String> {
        if !crate::model::has_phonon_model(Path::new(&dir)) {
            return Err(format!("missing native Phonon files in {dir}"));
        }
        let window_seconds = std::env::var("TRANSCRUST_PHONON_WINDOW_SECS")
            .map(|s| {
                s.parse::<u32>()
                    .map_err(|_| "invalid Phonon window seconds".to_string())
            })
            .unwrap_or(Ok(6))?;
        Segmenter::with_window_seconds(window_seconds)?;
        Ok(Self {
            inner: Arc::new(Inner {
                dir,
                idle,
                window_seconds,
                tx: Mutex::new(None),
            }),
        })
    }
    pub fn shutdown(&self) {
        if let Ok(mut tx) = self.inner.tx.lock() {
            *tx = None;
        }
    }
    fn worker(&self, observer: &Observer) -> Result<mpsc::Sender<Job>, String> {
        let mut slot = self.inner.tx.lock().map_err(|e| e.to_string())?;
        if let Some(tx) = &*slot {
            return Ok(tx.clone());
        }
        let (tx, rx) = mpsc::channel();
        let (ready, wait) = mpsc::channel();
        let inner = self.inner.clone();
        let observer = observer.clone();
        std::thread::Builder::new()
            .name("transcrust-phonon".into())
            .spawn(move || {
                let start = Instant::now();
                observer.phase("worker.start", "loading native Rust Phonon 2.5 English");
                let model = match Phonon::load(Path::new(&inner.dir)) {
                    Ok(m) => m,
                    Err(e) => {
                        let _ = ready.send(Err(e.to_string()));
                        return;
                    }
                };
                observer.phase(
                    "worker.start",
                    &format!(
                        "Phonon ready in {:.2}s; resident packed weights; {}-second window cap",
                        start.elapsed().as_secs_f64(),
                        inner.window_seconds
                    ),
                );
                let _ = ready.send(Ok(()));
                let mut last = Instant::now();
                loop {
                    match rx.recv_timeout(Duration::from_secs(1)) {
                        Ok(job) => {
                            let result = transcribe_job(&model, &job);
                            let _ = job.reply.send(result);
                            last = Instant::now();
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            if last.elapsed() > Duration::from_secs(inner.idle) {
                                if let Ok(mut slot) = inner.tx.lock() {
                                    *slot = None;
                                }
                                observer.phase("worker.idle", "Phonon model unloaded");
                                break;
                            }
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            observer.phase("worker.unload", "Phonon model dropped on switch");
                            break;
                        }
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        wait.recv_timeout(Duration::from_secs(30))
            .map_err(|e| format!("Phonon startup: {e}"))??;
        *slot = Some(tx.clone());
        Ok(tx)
    }
    pub async fn transcribe(
        &self,
        observer: Observer,
        rx: mpsc::Receiver<Vec<f32>>,
        rate: u32,
    ) -> Result<String, String> {
        let mut audio = Some(rx);
        for _ in 0..2 {
            let tx = self.worker(&observer)?;
            let (reply, wait) = oneshot::channel();
            let job = Job {
                observer: observer.clone(),
                rx: audio.take().unwrap(),
                rate,
                window_seconds: self.inner.window_seconds,
                reply,
            };
            match tx.send(job) {
                Ok(()) => return wait.await.map_err(|e| e.to_string())?,
                Err(mpsc::SendError(job)) => {
                    audio = Some(job.rx);
                }
            }
        }
        Err("Phonon worker exited before capture".into())
    }
}

/// Forward each original chunk immediately, retaining an exact copy only when
/// corpus recording is enabled. An engine failure must not discard the take.
pub(crate) fn tee_capture(
    audio: mpsc::Receiver<Vec<f32>>,
    bank: bool,
) -> (mpsc::Receiver<Vec<f32>>, mpsc::Receiver<Vec<f32>>) {
    let (engine_tx, engine_rx) = mpsc::channel();
    let (bank_tx, bank_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut samples = Vec::new();
        for chunk in audio {
            if bank {
                samples.extend_from_slice(&chunk);
            }
            let _ = engine_tx.send(chunk);
        }
        drop(engine_tx);
        if bank {
            let _ = bank_tx.send(samples);
        }
    });
    (engine_rx, bank_rx)
}

fn transcribe_job(model: &Phonon, job: &Job) -> Result<String, String> {
    let mut resampler = crate::audio::StreamingResampler::new(job.rate, 16000);
    let mut segment = Segmenter::with_window_seconds(job.window_seconds)?;
    let mut frame = Vec::with_capacity(480);
    let mut parts = Vec::new();
    let mut work = Duration::ZERO;
    let mut push = |audio: &[f32]| -> Result<(), String> {
        for &sample in audio {
            frame.push(sample);
            if frame.len() == 480 {
                if let Some(piece) = segment.push(&frame) {
                    let start = Instant::now();
                    let result = model.transcribe(&piece, None).map_err(|e| e.to_string())?;
                    let elapsed = start.elapsed();
                    work += elapsed;
                    job.observer.phase(
                        "phonon.window",
                        &format!(
                            "{:.2}s audio; {:.2}s processing",
                            piece.len() as f64 / 16000.,
                            elapsed.as_secs_f64()
                        ),
                    );
                    if !result.text.is_empty() {
                        parts.push(result.text);
                    }
                }
                frame.clear();
            }
        }
        Ok(())
    };
    for chunk in job.rx.iter() {
        push(&resampler.push(&chunk))?;
    }
    let tail = Instant::now();
    push(&resampler.finish())?;
    drop(push);
    if !frame.is_empty() {
        if let Some(piece) = segment.push(&frame) {
            let result = model.transcribe(&piece, None).map_err(|e| e.to_string())?;
            if !result.text.is_empty() {
                parts.push(result.text);
            }
        }
    }
    let rest = segment.finish();
    let result = model.transcribe(&rest, None).map_err(|e| e.to_string())?;
    if !result.text.is_empty() {
        parts.push(result.text);
    }
    job.observer.phase(
        "phonon.stream",
        &format!(
            "{} text windows; {:.2}s completed-window work; {:.2}s drain/final tail",
            parts.len(),
            work.as_secs_f64(),
            tail.elapsed().as_secs_f64()
        ),
    );
    Ok(parts.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "local model and banked take; replays at microphone pace without injection"]
    async fn paced_capture_preserves_transcript_and_audio() {
        let wav = std::env::var("TRANSCRUST_PHONON_REPLAY").expect("set replay WAV path");
        let model = std::env::var("TRANSCRUST_PHONON_MODEL").expect("set native model directory");
        let (samples, rate) = crate::audio::read_wav_mono(Path::new(&wav)).unwrap();
        let expected: serde_json::Value =
            serde_json::from_slice(&std::fs::read(Path::new(&wav).with_extension("json")).unwrap())
                .unwrap();
        // Separate probe log: never truncate the running daemon's latest.log.
        let observer = Observer::dec_probe().unwrap();
        let service = PhononService::new(model, 60).unwrap();
        let (tx, rx) = mpsc::channel();
        let (pending, bank) = service.start_capture(observer.clone(), rx, rate, true);
        let original = samples.clone();
        let feed = tokio::task::spawn_blocking(move || {
            let start = Instant::now();
            let mut sent = 0usize;
            for chunk in samples.chunks(rate as usize * 30 / 1000) {
                let deadline = start + Duration::from_secs_f64(sent as f64 / rate as f64);
                std::thread::sleep(deadline.saturating_duration_since(Instant::now()));
                tx.send(chunk.to_vec()).unwrap();
                sent += chunk.len();
            }
            std::thread::sleep(
                (start + Duration::from_secs_f64(sent as f64 / rate as f64))
                    .saturating_duration_since(Instant::now()),
            );
            observer.phase("test.release", "microphone sender closing");
            let released = Instant::now();
            drop(tx);
            released
        });
        let released = feed.await.unwrap();
        let text = pending.await.unwrap().unwrap();
        let tail = released.elapsed().as_secs_f64();
        assert_eq!(text, expected["raw"].as_str().unwrap());
        assert_eq!(bank.recv().unwrap(), original);
        eprintln!(
            "paced Phonon replay: exact transcript and corpus audio; {tail:.3}s after release"
        );
    }

    #[test]
    fn capture_reaches_engine_before_stop_and_banks_exact_samples() {
        let (tx, rx) = mpsc::channel();
        let (engine, bank) = tee_capture(rx, true);
        tx.send(vec![0.125, -0.25]).unwrap();
        // The capture sender is still open: inference cannot wait for release.
        assert_eq!(
            engine.recv_timeout(Duration::from_secs(1)).unwrap(),
            vec![0.125, -0.25]
        );
        assert!(matches!(bank.try_recv(), Err(mpsc::TryRecvError::Empty)));
        tx.send(vec![0.5]).unwrap();
        drop(tx);
        assert_eq!(engine.into_iter().flatten().collect::<Vec<_>>(), vec![0.5]);
        assert_eq!(
            bank.recv_timeout(Duration::from_secs(1)).unwrap(),
            vec![0.125, -0.25, 0.5]
        );
    }
    #[test]
    fn engine_failure_preserves_audio_and_disabled_banking_keeps_no_copy() {
        let (tx, rx) = mpsc::channel();
        let (engine, bank) = tee_capture(rx, true);
        drop(engine);
        tx.send(vec![0.25]).unwrap();
        drop(tx);
        assert_eq!(
            bank.recv_timeout(Duration::from_secs(1)).unwrap(),
            vec![0.25]
        );
        let (tx, rx) = mpsc::channel();
        let (engine, bank) = tee_capture(rx, false);
        tx.send(vec![0.5]).unwrap();
        drop(tx);
        assert_eq!(engine.into_iter().flatten().collect::<Vec<_>>(), vec![0.5]);
        assert!(bank.recv_timeout(Duration::from_secs(1)).is_err());
    }
}
