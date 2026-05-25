use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use ort::execution_providers::CPUExecutionProvider;
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use parakeet_rs::{ExecutionConfig, ParakeetTDT, TimestampMode, Transcriber};
use tokio::sync::oneshot;

use crate::observe::Observer;

pub struct ParakeetService {
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

impl Clone for ParakeetService {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl ParakeetService {
    pub fn new(model_dir: impl Into<String>, idle_timeout_secs: u64) -> Result<Self, String> {
        Ok(Self {
            inner: Arc::new(ServiceInner {
                model_dir: model_dir.into(),
                tx: Mutex::new(None),
                idle_timeout_secs,
            }),
        })
    }

    pub async fn transcribe(
        &self,
        observer: Observer,
        audio_rx: mpsc::Receiver<Vec<f32>>,
        source_sample_rate: u32,
    ) -> Result<String, String> {
        // Worker may have unloaded due to idle timeout; recover audio_rx on send failure.
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
                        .map_err(|e| format!("parakeet worker reply failed: {e}"))?
                }
                Err(mpsc::SendError(job)) => {
                    // Worker exited (idle timeout). Recover audio_rx and retry.
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
            .map_err(|_| "parakeet worker state lock poisoned".to_string())?;

        if let Some(tx) = guard.as_ref() {
            return Ok(tx.clone());
        }

        observer.phase("worker.start", "starting Parakeet worker on first transcription");

        let (tx, rx) = mpsc::channel::<Job>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();
        let inner = Arc::clone(&self.inner);
        let startup_observer = observer.clone();

        std::thread::Builder::new()
            .name("transcrust-parakeet".into())
            .spawn(move || worker_main(startup_observer, inner, rx, ready_tx))
            .map_err(|e| format!("failed to spawn parakeet worker: {e}"))?;

        match ready_rx.recv_timeout(Duration::from_secs(20)) {
            Ok(Ok(())) => {
                *guard = Some(tx.clone());
                Ok(tx)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err("timed out waiting for parakeet worker startup".into()),
        }
    }
}

#[derive(Clone, Copy)]
struct ProbeConfig {
    name: &'static str,
    load_mode: ProbeLoadMode,
    optimization: Option<ProbeOptimization>,
    cpu_provider: bool,
    parallel_execution: Option<bool>,
    memory_pattern: Option<bool>,
    intra_threads: Option<usize>,
    inter_threads: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProbeLoadMode {
    File,
    Memory,
}

#[derive(Clone, Copy)]
enum ProbeOptimization {
    Level1,
    Level3,
}

impl ProbeConfig {
    fn defaults_file() -> Self {
        Self {
            name: "defaults-file",
            load_mode: ProbeLoadMode::File,
            optimization: None,
            cpu_provider: false,
            parallel_execution: None,
            memory_pattern: None,
            intra_threads: None,
            inter_threads: None,
        }
    }

    fn defaults_memory() -> Self {
        Self {
            name: "defaults-memory",
            load_mode: ProbeLoadMode::Memory,
            optimization: None,
            cpu_provider: false,
            parallel_execution: None,
            memory_pattern: None,
            intra_threads: None,
            inter_threads: None,
        }
    }

    fn conservative() -> Self {
        Self {
            name: "conservative-file",
            load_mode: ProbeLoadMode::File,
            optimization: Some(ProbeOptimization::Level1),
            cpu_provider: false,
            parallel_execution: None,
            memory_pattern: Some(false),
            intra_threads: Some(1),
            inter_threads: Some(1),
        }
    }

    fn conservative_memory() -> Self {
        Self {
            name: "conservative-memory",
            load_mode: ProbeLoadMode::Memory,
            optimization: Some(ProbeOptimization::Level1),
            cpu_provider: false,
            parallel_execution: None,
            memory_pattern: Some(false),
            intra_threads: Some(1),
            inter_threads: Some(1),
        }
    }

    fn silentkeys_like() -> Self {
        let threads = available_probe_threads();
        Self {
            name: "silentkeys-file",
            load_mode: ProbeLoadMode::File,
            optimization: Some(ProbeOptimization::Level3),
            cpu_provider: true,
            parallel_execution: Some(true),
            memory_pattern: None,
            intra_threads: Some(threads),
            inter_threads: Some(threads),
        }
    }

    fn silentkeys_memory() -> Self {
        let threads = available_probe_threads();
        Self {
            name: "silentkeys-memory",
            load_mode: ProbeLoadMode::Memory,
            optimization: Some(ProbeOptimization::Level3),
            cpu_provider: true,
            parallel_execution: Some(true),
            memory_pattern: None,
            intra_threads: Some(threads),
            inter_threads: Some(threads),
        }
    }
}

fn available_probe_threads() -> usize {
    std::env::var("ORT_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|threads| *threads > 0)
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|threads| threads.get())
                .unwrap_or(1)
        })
}

fn load_single_session(path: &std::path::Path, config: &ProbeConfig) -> Result<(), String> {
    let mut builder = Session::builder().map_err(|e| format!("builder failed: {e}"))?;
    if let Some(level) = config.optimization {
        let level = match level {
            ProbeOptimization::Level1 => GraphOptimizationLevel::Level1,
            ProbeOptimization::Level3 => GraphOptimizationLevel::Level3,
        };
        builder = builder
            .with_optimization_level(level)
            .map_err(|e| format!("set optimization failed: {e}"))?;
    }
    if config.cpu_provider {
        builder = builder
            .with_execution_providers(vec![CPUExecutionProvider::default().build()])
            .map_err(|e| format!("set execution provider failed: {e}"))?;
    }
    if let Some(parallel) = config.parallel_execution {
        builder = builder
            .with_parallel_execution(parallel)
            .map_err(|e| format!("set parallel execution failed: {e}"))?;
    }
    if let Some(enabled) = config.memory_pattern {
        builder = builder
            .with_memory_pattern(enabled)
            .map_err(|e| format!("set memory pattern failed: {e}"))?;
    }
    if let Some(threads) = config.intra_threads {
        builder = builder
            .with_intra_threads(threads)
            .map_err(|e| format!("set intra threads failed: {e}"))?;
    }
    if let Some(threads) = config.inter_threads {
        builder = builder
            .with_inter_threads(threads)
            .map_err(|e| format!("set inter threads failed: {e}"))?;
    }
    let _session = match config.load_mode {
        ProbeLoadMode::File => builder
            .commit_from_file(path)
            .map_err(|e| format!("commit_from_file failed: {e}"))?,
        ProbeLoadMode::Memory => {
            let bytes = std::fs::read(path)
                .map_err(|e| format!("read for memory load failed: {e}"))?;
            builder
                .commit_from_memory(&bytes)
                .map_err(|e| format!("commit_from_memory failed: {e}"))?
        }
    };
    Ok(())
}

#[allow(dead_code)]
pub fn probe_onnx_file(path: &std::path::Path) -> Result<std::time::Duration, String> {
    run_probe_case(path, &ProbeConfig::conservative(), Duration::from_secs(15))
}

pub fn run_probe_suite(
    observer: &Observer,
    paths: &[std::path::PathBuf],
    timeout: Duration,
) -> Result<(), String> {
    if paths.is_empty() {
        return Err("probe suite requires at least one ONNX path".into());
    }

    let configs = [
        ProbeConfig::defaults_file(),
        ProbeConfig::defaults_memory(),
        ProbeConfig::conservative(),
        ProbeConfig::conservative_memory(),
        ProbeConfig::silentkeys_like(),
        ProbeConfig::silentkeys_memory(),
    ];

    let mut failures = 0usize;
    for path in paths {
        observer.phase("probe.file", &format!("testing {}", path.display()));
        for config in configs {
            if config.load_mode == ProbeLoadMode::Memory && !supports_memory_probe(path) {
                observer.phase(
                    "probe.skip",
                    &format!("skipping {} for {}", config.name, path.display()),
                );
                continue;
            }
            let phase = format!("probe.{}", config.name);
            match run_probe_with_timeout(observer, &phase, path, &config, timeout) {
                Ok(elapsed) => observer.phase(
                    "probe.result",
                    &format!(
                        "{} succeeded for {} in {:?}",
                        config.name,
                        path.display(),
                        elapsed
                    ),
                ),
                Err(message) => {
                    failures += 1;
                    observer.error(
                        "probe.result",
                        &format!("{} failed for {}: {}", config.name, path.display(), message),
                    );
                }
            }
        }
    }

    if failures > 0 {
        return Err(format!("probe suite finished with {failures} failed cases"));
    }

    observer.phase("probe.result", "all probe cases succeeded");
    Ok(())
}

fn supports_memory_probe(path: &std::path::Path) -> bool {
    if path.extension().and_then(|ext| ext.to_str()) != Some("onnx") {
        return false;
    }

    let external_data = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("{name}.data"))
        .map(|data_name| path.with_file_name(data_name))
        .is_some_and(|data_path| data_path.exists());
    if external_data {
        return false;
    }

    path.metadata()
        .map(|meta| meta.len() <= 256 * 1024 * 1024)
        .unwrap_or(false)
}

fn run_probe_with_timeout(
    observer: &Observer,
    phase: &str,
    path: &std::path::Path,
    config: &ProbeConfig,
    timeout: Duration,
) -> Result<Duration, String> {
    observer.phase(
        phase,
        &format!(
            "starting ORT session load with {} for {}",
            config.name,
            path.display()
        ),
    );

    match run_probe_case(path, config, timeout) {
        Ok(elapsed) => {
            observer.phase(
                phase,
                &format!("ORT session loaded with {} in {:?}", config.name, elapsed),
            );
            Ok(elapsed)
        }
        Err(message) => {
            observer.error(phase, &message);
            Err(message)
        }
    }
}

fn run_probe_case(
    path: &std::path::Path,
    config: &ProbeConfig,
    timeout: Duration,
) -> Result<Duration, String> {
    let (tx, rx) = mpsc::channel();
    let path = path.to_path_buf();
    let load_path = path.clone();
    let config = *config;

    std::thread::Builder::new()
        .name(format!(
            "probe-{}",
            path.file_name().and_then(|s| s.to_str()).unwrap_or("onnx")
        ))
        .spawn(move || {
            let start = Instant::now();
            let result = load_single_session(&load_path, &config).map(|_| start.elapsed());
            let _ = tx.send(result);
        })
        .map_err(|e| format!("failed to spawn ORT probe: {e}"))?;

    match rx.recv_timeout(timeout) {
        Ok(Ok(elapsed)) => Ok(elapsed),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(format!(
            "timed out after {:?} loading {} with {}",
            timeout,
            path.display(),
            config.name
        )),
    }
}

const WORKER_IDLE_CHECK_INTERVAL: Duration = Duration::from_secs(1);

pub fn build_execution_config() -> ExecutionConfig {
    ExecutionConfig::new()
        .with_intra_threads(1)
        .with_inter_threads(1)
        .with_custom_configure(|builder| {
            Ok(builder
                .with_optimization_level(GraphOptimizationLevel::Level1)?
                .with_memory_pattern(true)?)
        })
}

fn worker_main(
    observer: Observer,
    inner: Arc<ServiceInner>,
    rx: mpsc::Receiver<Job>,
    ready_tx: mpsc::Sender<Result<(), String>>,
) {
    let idle_timeout = Duration::from_secs(inner.idle_timeout_secs);
    observer.phase("worker.start", "loading Parakeet model on demand");
    let exec = build_execution_config();

    let mut model = match ParakeetTDT::from_pretrained(&inner.model_dir, Some(exec)) {
        Ok(model) => {
            observer.phase("worker.start", "Parakeet model loaded");
            observer.notify("Transcrust ready", "Parakeet model loaded and worker is ready.");
            let _ = ready_tx.send(Ok(()));
            model
        }
        Err(e) => {
            let message = format!("failed to load parakeet model: {e}");
            observer.error("worker.start", &message);
            let _ = ready_tx.send(Err(message));
            return;
        }
    };

    let mut last_activity = Instant::now();

    loop {
        match rx.recv_timeout(WORKER_IDLE_CHECK_INTERVAL) {
            Ok(job) => {
                last_activity = Instant::now();
                let result = transcribe_with_loaded_model(
                    &job.observer,
                    &mut model,
                    job.audio_rx,
                    job.source_sample_rate,
                );
                let _ = job.reply_tx.send(result);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if last_activity.elapsed() > idle_timeout {
                    if let Ok(mut guard) = inner.tx.lock() {
                        *guard = None;
                    }
                    // Close the receiver before dropping the model so a racing sender gets a
                    // SendError and retries against a fresh worker instead of queueing into an
                    // exiting thread.
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
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    observer.phase("worker.stop", "worker channel closed");
}

fn transcribe_with_loaded_model(
    observer: &Observer,
    model: &mut ParakeetTDT,
    audio_rx: mpsc::Receiver<Vec<f32>>,
    source_sample_rate: u32,
) -> Result<String, String> {
    observer.phase("transcription.collect", "draining recorded audio");
    let mut audio_buf: Vec<f32> = Vec::new();
    let mut chunk_count = 0usize;
    while let Ok(chunk) = audio_rx.recv() {
        chunk_count += 1;
        audio_buf.extend_from_slice(&chunk);
    }
    observer.phase(
        "transcription.collect",
        &format!("captured {chunk_count} chunks, {} samples", audio_buf.len()),
    );

    if audio_buf.len() < (source_sample_rate as usize / 10) {
        return Ok(String::new());
    }

    observer.phase("transcription.resample", "resampling to 16kHz mono");
    let audio_16k = crate::audio::resample_to_16k(&audio_buf, source_sample_rate);
    observer.phase(
        "transcription.resample",
        &format!("resampled to {} samples", audio_16k.len()),
    );

    observer.phase("transcription.infer", "starting ONNX inference");
    let result = model
        .transcribe_samples(audio_16k, 16000, 1, Some(TimestampMode::Words))
        .map_err(|e| format!("Parakeet transcription failed: {e}"))?;
    observer.phase("transcription.infer", "ONNX inference finished");

    Ok(result.text.trim().to_string())
}
