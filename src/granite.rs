use std::f32::consts::PI;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use ndarray::{Array2, Array3};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::TensorRef;
use realfft::{RealFftPlanner, RealToComplex};
use tokenizers::Tokenizer;
use tokio::sync::oneshot;

use crate::observe::Observer;

const SAMPLE_RATE: usize = 16_000;
const N_FFT: usize = 512;
const WIN_LENGTH: usize = 400;
const HOP_LENGTH: usize = 160;
const N_MELS: usize = 80;
const FEATURE_WIDTH: usize = 320;
const ATTENTION_MULTIPLE: usize = 512;
const VOCAB_SIZE: usize = 16_384;
const WORKER_IDLE_CHECK_INTERVAL: Duration = Duration::from_secs(1);

pub struct GraniteService {
    inner: Arc<ServiceInner>,
}

struct ServiceInner {
    model_dir: PathBuf,
    tx: Mutex<Option<mpsc::Sender<Job>>>,
    idle_timeout_secs: u64,
}

struct Job {
    observer: Observer,
    audio_rx: mpsc::Receiver<Vec<f32>>,
    source_sample_rate: u32,
    reply_tx: oneshot::Sender<Result<String, String>>,
}

impl Clone for GraniteService {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl GraniteService {
    pub fn new(model_dir: impl Into<PathBuf>, idle_timeout_secs: u64) -> Result<Self, String> {
        let model_dir = model_dir.into();
        if !crate::model::has_granite_model(&model_dir) {
            return Err(format!(
                "incomplete Granite model directory: {}",
                model_dir.display()
            ));
        }
        Ok(Self {
            inner: Arc::new(ServiceInner {
                model_dir,
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
                        .map_err(|e| format!("Granite worker reply failed: {e}"))?
                }
                Err(mpsc::SendError(job)) => {
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
            .map_err(|_| "Granite worker state lock poisoned".to_string())?;
        if let Some(tx) = guard.as_ref() {
            return Ok(tx.clone());
        }

        observer.phase(
            "worker.start",
            "starting Granite worker on first transcription",
        );
        let (tx, rx) = mpsc::channel::<Job>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();
        let inner = Arc::clone(&self.inner);
        let startup_observer = observer.clone();
        std::thread::Builder::new()
            .name("transcrust-granite".into())
            .spawn(move || worker_main(startup_observer, inner, rx, ready_tx))
            .map_err(|e| format!("failed to spawn Granite worker: {e}"))?;

        match ready_rx.recv_timeout(Duration::from_secs(20)) {
            Ok(Ok(())) => {
                *guard = Some(tx.clone());
                Ok(tx)
            }
            Ok(Err(error)) => Err(error),
            Err(_) => Err("timed out waiting for Granite worker startup".into()),
        }
    }
}

struct LoadedGranite {
    session: Session,
    tokenizer: Tokenizer,
    frontend: GraniteFrontend,
}

impl LoadedGranite {
    fn load(model_dir: &Path) -> Result<Self, String> {
        let model_path = crate::model::granite_onnx_path(model_dir).ok_or_else(|| {
            format!("no Granite ONNX graph in {}", model_dir.display())
        })?;
        let session = Session::builder()
            .map_err(|e| format!("failed to create ONNX session builder: {e}"))?
            .with_optimization_level(GraphOptimizationLevel::Level1)
            .map_err(|e| format!("failed to set ONNX optimization level: {e}"))?
            .with_intra_threads(1)
            .map_err(|e| format!("failed to set ONNX intra-op threads: {e}"))?
            .with_inter_threads(1)
            .map_err(|e| format!("failed to set ONNX inter-op threads: {e}"))?
            // Mirrors `parakeet::build_execution_config` so both engines run
            // under the same ORT settings and stay comparable in a smoke run.
            .with_memory_pattern(true)
            .map_err(|e| format!("failed to set ONNX memory pattern: {e}"))?
            .commit_from_file(&model_path)
            .map_err(|e| format!("failed to load {}: {e}", model_path.display()))?;
        let tokenizer_path = model_dir.join("tokenizer.json");
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| format!("failed to load {}: {e}", tokenizer_path.display()))?;
        Ok(Self {
            session,
            tokenizer,
            frontend: GraniteFrontend::new(),
        })
    }

    fn transcribe(&mut self, audio: &[f32]) -> Result<String, String> {
        let (features, attention_mask, real_feature_frames) = self.frontend.extract(audio)?;
        let features_input = TensorRef::from_array_view(&features)
            .map_err(|e| format!("failed to build Granite feature tensor: {e}"))?;
        let mask_input = TensorRef::from_array_view(&attention_mask)
            .map_err(|e| format!("failed to build Granite mask tensor: {e}"))?;
        let outputs = self
            .session
            .run(ort::inputs![
                "input_features" => features_input,
                "attention_mask" => mask_input,
            ])
            .map_err(|e| format!("Granite ONNX inference failed: {e}"))?;
        let (shape, logits) = outputs["logits"]
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Granite logits were not a float tensor: {e}"))?;
        if shape.len() != 3 || shape[0] != 1 || shape[2] != VOCAB_SIZE as i64 {
            return Err(format!("unexpected Granite logits shape: {shape:?}"));
        }
        let real_logit_frames = real_feature_frames / 4;
        if shape[1] < real_logit_frames as i64 {
            return Err(format!(
                "Granite returned {} frames for {real_logit_frames} required frames",
                shape[1]
            ));
        }
        let ids = greedy_ctc_ids(logits, real_logit_frames, VOCAB_SIZE);
        self.tokenizer
            .decode(&ids, true)
            .map(|text| text.trim().to_string())
            .map_err(|e| format!("Granite tokenizer decode failed: {e}"))
    }
}

pub fn run_model_smoke(model_dir: &Path) -> Result<String, String> {
    let mut model = LoadedGranite::load(model_dir)?;
    // Twelve seconds produces 600 real stacked frames and exercises the
    // export's dynamic 1024-frame path, not only its 512-frame example shape.
    let audio: Vec<f32> = (0..SAMPLE_RATE * 12)
        .map(|index| (2.0 * PI * 440.0 * index as f32 / SAMPLE_RATE as f32).sin() * 0.1)
        .collect();
    model.transcribe(&audio)
}

fn worker_main(
    observer: Observer,
    inner: Arc<ServiceInner>,
    rx: mpsc::Receiver<Job>,
    ready_tx: mpsc::Sender<Result<(), String>>,
) {
    observer.phase(
        "worker.start",
        &format!(
            "loading Granite model: {}",
            crate::model::granite_onnx_path(&inner.model_dir)
                .unwrap_or_else(|| inner.model_dir.clone())
                .display()
        ),
    );
    let mut model = match LoadedGranite::load(&inner.model_dir) {
        Ok(model) => {
            observer.phase("worker.start", "Granite model loaded");
            observer.notify(
                "Transcrust ready",
                "Granite model loaded and worker is ready.",
            );
            let _ = ready_tx.send(Ok(()));
            model
        }
        Err(error) => {
            observer.error("worker.start", &error);
            let _ = ready_tx.send(Err(error));
            return;
        }
    };
    let idle_timeout = Duration::from_secs(inner.idle_timeout_secs);
    let mut last_activity = Instant::now();

    loop {
        match rx.recv_timeout(WORKER_IDLE_CHECK_INTERVAL) {
            Ok(job) => {
                let result = transcribe_with_loaded_model(
                    &job.observer,
                    &mut model,
                    job.audio_rx,
                    job.source_sample_rate,
                );
                // Inactivity begins after the transcription loop completes,
                // not when the job first arrives.
                last_activity = Instant::now();
                let _ = job.reply_tx.send(result);
            }
            Err(mpsc::RecvTimeoutError::Timeout) if last_activity.elapsed() > idle_timeout => {
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
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    observer.phase("worker.stop", "worker channel closed");
}

fn transcribe_with_loaded_model(
    observer: &Observer,
    model: &mut LoadedGranite,
    audio_rx: mpsc::Receiver<Vec<f32>>,
    source_sample_rate: u32,
) -> Result<String, String> {
    observer.phase("transcription.collect", "draining recorded audio");
    let mut audio_buf = Vec::new();
    let mut chunk_count = 0usize;
    while let Ok(chunk) = audio_rx.recv() {
        chunk_count += 1;
        audio_buf.extend_from_slice(&chunk);
    }
    observer.phase(
        "transcription.collect",
        &format!("captured {chunk_count} chunks, {} samples", audio_buf.len()),
    );
    if audio_buf.len() < source_sample_rate as usize / 10 {
        return Ok(String::new());
    }

    observer.phase("transcription.resample", "resampling to 16kHz mono");
    let audio_16k = crate::audio::resample_to_16k(&audio_buf, source_sample_rate);
    // Same phase names and order as the Parakeet worker, so a `--smoke` log of
    // one engine lines up against the other.
    observer.phase(
        "transcription.resample",
        &format!("resampled to {} samples", audio_16k.len()),
    );
    observer.phase(
        "transcription.frontend",
        "extracting Granite log-mel features",
    );
    observer.phase("transcription.infer", "starting ONNX inference");
    let text = model.transcribe(&audio_16k)?;
    observer.phase("transcription.infer", "ONNX inference finished");
    Ok(text)
}

struct GraniteFrontend {
    fft: Arc<dyn RealToComplex<f32>>,
    window: Vec<f32>,
    mel_filters: Vec<f32>,
}

impl GraniteFrontend {
    fn new() -> Self {
        let mut planner = RealFftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(N_FFT);
        let window = (0..WIN_LENGTH)
            .map(|index| 0.5 - 0.5 * (2.0 * PI * index as f32 / WIN_LENGTH as f32).cos())
            .collect();
        Self {
            fft,
            window,
            mel_filters: htk_mel_filterbank(),
        }
    }

    fn extract(&self, audio: &[f32]) -> Result<(Array3<f32>, Array2<bool>, usize), String> {
        if audio.len() <= N_FFT / 2 {
            return Err("Granite frontend needs more than 256 audio samples".into());
        }
        let mel_frames = audio.len() / HOP_LENGTH;
        let feature_frames = ((mel_frames + 1) / 2) * 2;
        if feature_frames == 0 {
            return Err("Granite frontend produced no frames".into());
        }

        let samples_needed = (feature_frames - 1) * HOP_LENGTH + 1;
        let mut waveform = audio.to_vec();
        waveform.resize(waveform.len().max(samples_needed), 0.0);
        let centered = reflect_pad(&waveform, N_FFT / 2);
        let mut input = self.fft.make_input_vec();
        let mut output = self.fft.make_output_vec();
        let mut scratch = self.fft.make_scratch_vec();
        let mut mel = vec![0.0f32; feature_frames * N_MELS];

        for frame in 0..feature_frames {
            input.fill(0.0);
            let start = frame * HOP_LENGTH + (N_FFT - WIN_LENGTH) / 2;
            for index in 0..WIN_LENGTH {
                input[(N_FFT - WIN_LENGTH) / 2 + index] =
                    centered[start + index] * self.window[index];
            }
            self.fft
                .process_with_scratch(&mut input, &mut output, &mut scratch)
                .map_err(|e| format!("Granite FFT failed: {e}"))?;
            for mel_index in 0..N_MELS {
                let mut power = 0.0f32;
                for (bin, value) in output.iter().enumerate() {
                    power += value.norm_sqr() * self.mel_filters[mel_index * (N_FFT / 2 + 1) + bin];
                }
                mel[frame * N_MELS + mel_index] = power.max(1e-10).log10();
            }
        }

        let maximum = mel.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        for value in &mut mel {
            *value = value.max(maximum - 8.0) / 4.0 + 1.0;
        }
        let deltas = deltas(&mel, feature_frames);
        let real_stacked_frames = feature_frames / 2;
        let mut stacked = vec![0.0f32; real_stacked_frames * FEATURE_WIDTH];
        for pair in 0..feature_frames / 2 {
            for half in 0..2 {
                let source_frame = pair * 2 + half;
                let target_base = pair * FEATURE_WIDTH + half * (N_MELS * 2);
                let source_base = source_frame * N_MELS;
                stacked[target_base..target_base + N_MELS]
                    .copy_from_slice(&mel[source_base..source_base + N_MELS]);
                stacked[target_base + N_MELS..target_base + N_MELS * 2]
                    .copy_from_slice(&deltas[source_base..source_base + N_MELS]);
            }
        }
        // `feature_frames` above counts pre-stack mel frames. The ONNX input's
        // time axis counts pairs, so pad that axis to 512 rather than padding
        // the mel axis before stacking.
        let padded_stacked_frames =
            real_stacked_frames.div_ceil(ATTENTION_MULTIPLE) * ATTENTION_MULTIPLE;
        stacked.resize(padded_stacked_frames * FEATURE_WIDTH, 0.0);
        let features = Array3::from_shape_vec((1, padded_stacked_frames, FEATURE_WIDTH), stacked)
            .map_err(|e| format!("failed to shape Granite features: {e}"))?;
        let attention_mask = Array2::from_shape_fn((1, padded_stacked_frames), |(_, frame)| {
            frame < real_stacked_frames
        });
        Ok((features, attention_mask, real_stacked_frames))
    }
}

fn reflect_pad(audio: &[f32], amount: usize) -> Vec<f32> {
    let mut padded = Vec::with_capacity(audio.len() + amount * 2);
    for index in (1..=amount).rev() {
        padded.push(audio[index]);
    }
    padded.extend_from_slice(audio);
    for index in 1..=amount {
        padded.push(audio[audio.len() - 1 - index]);
    }
    padded
}

fn htk_mel_filterbank() -> Vec<f32> {
    let freq_bins = N_FFT / 2 + 1;
    let mel_max = hz_to_mel_htk(SAMPLE_RATE as f32 / 2.0);
    let points: Vec<f32> = (0..N_MELS + 2)
        .map(|index| mel_to_hz_htk(mel_max * index as f32 / (N_MELS + 1) as f32))
        .collect();
    let mut filters = vec![0.0f32; N_MELS * freq_bins];
    for mel in 0..N_MELS {
        let lower_width = points[mel + 1] - points[mel];
        let upper_width = points[mel + 2] - points[mel + 1];
        for bin in 0..freq_bins {
            let frequency = bin as f32 * SAMPLE_RATE as f32 / N_FFT as f32;
            let lower = (frequency - points[mel]) / lower_width;
            let upper = (points[mel + 2] - frequency) / upper_width;
            filters[mel * freq_bins + bin] = lower.min(upper).max(0.0);
        }
    }
    filters
}

fn hz_to_mel_htk(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

fn mel_to_hz_htk(mel: f32) -> f32 {
    700.0 * (10.0f32.powf(mel / 2595.0) - 1.0)
}

fn deltas(features: &[f32], frames: usize) -> Vec<f32> {
    let mut output = vec![0.0f32; features.len()];
    for frame in 0..frames {
        let previous = frame.saturating_sub(1);
        let next = (frame + 1).min(frames - 1);
        for mel in 0..N_MELS {
            output[frame * N_MELS + mel] =
                (features[next * N_MELS + mel] - features[previous * N_MELS + mel]) / 2.0;
        }
    }
    output
}

fn greedy_ctc_ids(logits: &[f32], frames: usize, vocab_size: usize) -> Vec<u32> {
    let mut ids = Vec::new();
    let mut previous = None;
    for frame in logits.chunks_exact(vocab_size).take(frames) {
        let (id, _) = frame
            .iter()
            .copied()
            .enumerate()
            .max_by(|left, right| left.1.total_cmp(&right.1))
            .expect("vocabulary cannot be empty");
        if previous != Some(id) && id != 0 {
            ids.push(id as u32);
        }
        previous = Some(id);
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctc_collapses_before_removing_blank() {
        let mut logits = vec![0.0; 7 * 4];
        for (frame, id) in [0, 2, 2, 0, 2, 3, 3].into_iter().enumerate() {
            logits[frame * 4 + id] = 1.0;
        }
        assert_eq!(greedy_ctc_ids(&logits, 7, 4), vec![2, 2, 3]);
    }

    #[test]
    fn frontend_emits_onnx_contract_shape() {
        let audio: Vec<f32> = (0..SAMPLE_RATE)
            .map(|index| (2.0 * PI * 440.0 * index as f32 / SAMPLE_RATE as f32).sin())
            .collect();
        let (features, mask, real_frames) = GraniteFrontend::new().extract(&audio).unwrap();
        assert_eq!(real_frames, 50);
        assert_eq!(features.shape(), &[1, 512, FEATURE_WIDTH]);
        assert_eq!(mask.iter().filter(|value| **value).count(), 50);
        assert!(features.iter().all(|value| value.is_finite()));
    }

    #[test]
    fn frontend_matches_transformers_reference() {
        let audio: Vec<f32> = (0..SAMPLE_RATE)
            .map(|index| (2.0 * PI * 440.0 * index as f32 / SAMPLE_RATE as f32).sin())
            .collect();
        let (features, _, _) = GraniteFrontend::new().extract(&audio).unwrap();
        let expected = [
            ((0, 0), 1.4784813),
            ((0, 1), 1.4888508),
            ((0, 79), 0.7002542),
            ((0, 80), -0.25796065),
            ((0, 159), -0.25503838),
            ((0, 160), 0.96256),
            ((0, 239), 0.19017744),
            ((0, 240), -0.74722266),
            ((0, 319), -0.35810912),
            ((1, 0), -0.015964031),
            ((10, 17), 1.4980546),
            ((49, 319), 0.1063613),
        ];
        for ((frame, column), reference) in expected {
            let actual = features[[0, frame, column]];
            assert!(
                (actual - reference).abs() < 2e-3,
                "feature[{frame}, {column}] was {actual}, expected {reference}"
            );
        }
        let reference_sum = 2009.210_4f32;
        let actual_sum: f32 = features
            .slice(ndarray::s![0, 0..50, ..])
            .iter()
            .copied()
            .sum();
        assert!((actual_sum - reference_sum).abs() < 0.1);
    }
}
