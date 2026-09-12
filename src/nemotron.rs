//! Nemotron Speech Streaming EN 0.6B — cache-aware streaming driver.
//!
//! Route 4 of `design-long-form-routes.md`: toggled capture at `Profile::Raw`,
//! wired in as a `TranscriptionService` arm. Gates 3, 5 and 7 were answered on
//! `Record-2.wav` before it was wired; `--wav --mode nemotron` reproduces them
//! through the live seam.
//!
//! The loop is the one `config.json` in the danielbodart export describes, and
//! it differs from `parakeet_ort.rs` in exactly two ways:
//!
//! 1. **The encoder carries cache.** Three tensors in, three out, fed back
//!    chunk to chunk. Parakeet's encoder sees the whole utterance at once.
//! 2. **There is no duration head.** Nemotron is plain RNN-T, so the decode
//!    advances one encoder frame at a time and never frame-skips. That is the
//!    "same size is not the same speed" finding in `Parakeet-v3.md` §2, made
//!    executable.
//!
//! **Mel is computed incrementally.** `StreamingFrontend` carries preemphasis
//! history, a sliding signal window and the frame counter across chunks, and
//! is pinned against the batch reference so a streamed capture and a `--wav`
//! run produce the same frames.

use std::path::Path;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::oneshot;

use crate::observe::Observer;

use ndarray::{Array1, Array2, Array3, Array4, Axis};
use ort::session::Session;
use ort::value::TensorRef;

/// From the export's own `config.json`, not carried across from Parakeet.
const LAYERS: usize = 24;
const HIDDEN: usize = 1024;
const MELS: usize = 128;
const CHANNEL_CACHE: usize = 70;
const TIME_CACHE: usize = 8;
const PRED_HIDDEN: usize = 640;
const BLANK_ID: u32 = 1024;
const MAX_SYMBOLS_PER_FRAME: usize = 10;
/// 560 ms at the 10 ms hop, plus the pre-encode cache the encoder expects.
const CHUNK_MEL_FRAMES: usize = 56;
const PRE_ENCODE_CACHE_FRAMES: usize = 9;
/// Frontend constants, from the export's own `preprocessor.config`.
const N_FFT: usize = 512;
const HOP_LENGTH: usize = 160;
const WIN_LENGTH: usize = 400;
const PREEMPH: f32 = 0.97;
/// NeMo's `log_zero_guard_value` on the natural-log magnitude path.
const LOG_GUARD: f32 = 5.960464477539063e-8;

type DecoderState = (Array3<f32>, Array3<f32>);

struct Vocab {
    pieces: Vec<String>,
}

impl Vocab {
    /// `tokens.txt` is `piece<space>id` per line, SentencePiece style: `▁`
    /// marks a word boundary. Ids run 0..=1023; 1024 is blank and has no line,
    /// which is why the blank id comes from config rather than from the file.
    fn load(path: &Path) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let mut pieces = vec![String::new(); BLANK_ID as usize];
        for line in raw.lines() {
            let Some((piece, id)) = line.rsplit_once(' ') else {
                continue;
            };
            let Ok(id) = id.trim().parse::<usize>() else {
                continue;
            };
            if id < pieces.len() {
                pieces[id] = piece.to_string();
            }
        }
        Ok(Self { pieces })
    }

    fn detokenize(&self, ids: &[u32]) -> String {
        let mut out = String::new();
        for id in ids {
            let Some(piece) = self.pieces.get(*id as usize) else {
                continue;
            };
            if let Some(rest) = piece.strip_prefix('▁') {
                if !out.is_empty() {
                    out.push(' ');
                }
                out.push_str(rest);
            } else {
                out.push_str(piece);
            }
        }
        out.trim().to_string()
    }
}

/// Encoder cache, carried chunk to chunk. Zeroed once at the start of a
/// capture and never reset, which is what makes the attention memory
/// continuous across the whole dictation.
struct Cache {
    channel: Array4<f32>,
    time: Array4<f32>,
    len: Array1<i64>,
}

impl Cache {
    fn zeroed(layout: &Layout) -> Self {
        Self {
            channel: Array4::zeros(layout.channel_cache_dims),
            time: Array4::zeros(layout.time_cache_dims),
            len: Array1::from_vec(vec![0i64]),
        }
    }
}

/// What an export calls its tensors, and which way round it stacks the cache.
///
/// The two exports tested disagree on both, and neither is wrong — potgieterdl
/// takes `processed_signal` with a layers-first cache `[24,1,70,1024]`,
/// danielbodart takes `audio_signal` with a batch-first `[1,24,70,1024]`, and
/// they even spell the cache-length *output* differently
/// (`cache_last_channel_len_next` vs `cache_last_channel_next_len`). Reading all
/// of it off the graph is what lets one driver serve both.
struct Layout {
    mel_input: String,
    length_input: String,
    channel_cache_dims: (usize, usize, usize, usize),
    time_cache_dims: (usize, usize, usize, usize),
    encoded: String,
    encoded_len: String,
    channel_next: String,
    time_next: String,
    len_next: String,
}

impl Layout {
    fn detect(encoder: &Session) -> Result<Self, String> {
        let mut mel_input = None;
        let mut length_input = None;
        let mut channel_cache_dims = (1, LAYERS, CHANNEL_CACHE, HIDDEN);
        let mut time_cache_dims = (1, LAYERS, HIDDEN, TIME_CACHE);
        for input in encoder.inputs() {
            let name = input.name().to_string();
            let ort::value::ValueType::Tensor { shape, .. } = input.dtype() else {
                continue;
            };
            let lower = name.to_lowercase();
            if lower.contains("cache_last_channel") && !lower.contains("len") {
                channel_cache_dims = fill4(shape, channel_cache_dims);
            } else if lower.contains("cache_last_time") {
                time_cache_dims = fill4(shape, time_cache_dims);
            } else if shape.len() == 3 && shape[1] == MELS as i64 {
                mel_input = Some(name);
            } else if shape.len() == 1 && !lower.contains("cache") {
                length_input = Some(name);
            }
        }
        let mut encoded = None;
        let mut encoded_len = None;
        let mut channel_next = None;
        let mut time_next = None;
        let mut len_next = None;
        for output in encoder.outputs() {
            let name = output.name().to_string();
            let lower = name.to_lowercase();
            if lower.contains("cache_last_channel") && lower.contains("len") {
                len_next = Some(name);
            } else if lower.contains("cache_last_channel") {
                channel_next = Some(name);
            } else if lower.contains("cache_last_time") {
                time_next = Some(name);
            } else if lower.contains("len") {
                encoded_len = Some(name);
            } else {
                encoded = Some(name);
            }
        }
        Ok(Self {
            mel_input: mel_input.ok_or("encoder has no 128-mel input")?,
            length_input: length_input.ok_or("encoder has no length input")?,
            channel_cache_dims,
            time_cache_dims,
            encoded: encoded.ok_or("encoder has no encodings output")?,
            encoded_len: encoded_len.ok_or("encoder has no encoded-length output")?,
            channel_next: channel_next.ok_or("encoder has no channel-cache output")?,
            time_next: time_next.ok_or("encoder has no time-cache output")?,
            len_next: len_next.ok_or("encoder has no cache-length output")?,
        })
    }
}

/// Take fixed axes from the graph; keep the canonical value where it says -1.
fn fill4(shape: &[i64], fallback: (usize, usize, usize, usize)) -> (usize, usize, usize, usize) {
    let canonical = [fallback.0, fallback.1, fallback.2, fallback.3];
    let mut dims = canonical;
    for (axis, declared) in shape.iter().enumerate().take(4) {
        if *declared > 0 {
            dims[axis] = *declared as usize;
        } else {
            // A dynamic axis: the layers axis is the one that is always 24, so
            // find where this export put it and keep the rest in order.
            dims[axis] = canonical[axis];
        }
    }
    (dims[0], dims[1], dims[2], dims[3])
}

pub struct Nemotron {
    encoder: Session,
    decoder: Session,
    vocab: Vocab,
    layout: Layout,
    tuning: Tuning,
    /// The export's own frontend, never Parakeet's. See gate 3.
    frontend: Frontend,
}

impl Nemotron {
    pub fn load(dir: &Path, tuning: Tuning) -> Result<Self, String> {
        let encoder_path = ["encoder_model.onnx", "encoder.onnx"]
            .iter()
            .map(|name| dir.join(name))
            .find(|path| path.exists())
            .unwrap_or_else(|| dir.join("encoder_model.onnx"));
        // Two naming conventions for the same fused decoder+joint graph.
        let decoder_path = ["decoder_model.onnx", "decoder_joint.onnx"]
            .iter()
            .map(|name| dir.join(name))
            .find(|path| path.exists())
            .unwrap_or_else(|| dir.join("decoder_model.onnx"));
        let tokens_path = dir.join("tokens.txt");
        for path in [&encoder_path, &decoder_path, &tokens_path] {
            if !path.exists() {
                return Err(format!("missing {}", path.display()));
            }
        }
        let encoder = tuning
            .builder()?
            .commit_from_file(&encoder_path)
            .map_err(|e| format!("failed to open Nemotron encoder: {e}"))?;
        let decoder = tuning
            .builder()?
            .commit_from_file(&decoder_path)
            .map_err(|e| format!("failed to open Nemotron decoder: {e}"))?;
        Ok(Self {
            layout: Layout::detect(&encoder)?,
            encoder,
            decoder,
            vocab: Vocab::load(&tokens_path)?,
            tuning,
            frontend: Frontend::new(&dir.join("filterbank.bin"))?,
        })
    }

    /// Run one chunk of mel through the encoder, advancing the cache.
    ///
    /// Returns encodings as `[time, 1024]`. The graph emits `[1, 1024, time]`,
    /// same layout Parakeet's does, so the transpose is the same one
    /// `parakeet_ort::encode` performs.
    fn encode_chunk(
        &mut self,
        mel: &Array3<f32>,
        cache: &mut Cache,
    ) -> Result<Array2<f32>, String> {
        let frames = mel.shape()[2] as i64;
        let lengths = Array1::from_vec(vec![frames]);
        let names = (
            self.layout.mel_input.clone(),
            self.layout.length_input.clone(),
        );
        let outputs = self
            .encoder
            .run(ort::inputs![
                names.0.as_str() => TensorRef::from_array_view(mel)
                    .map_err(|e| format!("failed to build Nemotron mel tensor: {e}"))?,
                names.1.as_str() => TensorRef::from_array_view(&lengths)
                    .map_err(|e| format!("failed to build Nemotron length tensor: {e}"))?,
                "cache_last_channel" => TensorRef::from_array_view(&cache.channel)
                    .map_err(|e| format!("failed to build channel cache: {e}"))?,
                "cache_last_time" => TensorRef::from_array_view(&cache.time)
                    .map_err(|e| format!("failed to build time cache: {e}"))?,
                "cache_last_channel_len" => TensorRef::from_array_view(&cache.len)
                    .map_err(|e| format!("failed to build cache length: {e}"))?,
            ])
            .map_err(|e| format!("Nemotron encoder failed: {e}"))?;

        let valid = {
            let (_, lens) = outputs[self.layout.encoded_len.as_str()]
                .try_extract_tensor::<i64>()
                .map_err(|e| format!("Nemotron encoded_lengths were not int64: {e}"))?;
            lens.first().copied().unwrap_or(0).max(0) as usize
        };
        let (shape, data) = outputs[self.layout.encoded.as_str()]
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Nemotron encodings were not float: {e}"))?;
        if shape.len() != 3 || shape[1] != HIDDEN as i64 {
            return Err(format!("unexpected Nemotron encoding shape: {shape:?}"));
        }
        let time = shape[2] as usize;
        let mut encodings = Array2::<f32>::zeros((time.min(valid.max(0)), HIDDEN));
        let kept = encodings.shape()[0];
        for channel in 0..HIDDEN {
            for step in 0..kept {
                encodings[[step, channel]] = data[channel * time + step];
            }
        }

        // Feed the cache forward. Doing this after the reads above keeps the
        // borrow of `outputs` alive only as long as it has to be.
        cache.channel = extract4(&outputs, self.layout.channel_next.as_str(), cache.channel.dim())?;
        cache.time = extract4(&outputs, self.layout.time_next.as_str(), cache.time.dim())?;
        if let Ok((_, lens)) = outputs[self.layout.len_next.as_str()].try_extract_tensor::<i64>() {
            cache.len = Array1::from_vec(vec![lens.first().copied().unwrap_or(0)]);
        }
        Ok(encodings)
    }

    /// One joint step. Same five-in/four-out shape as Parakeet's fused
    /// decoder+joint, minus the duration logits.
    fn joint_step(
        &mut self,
        previous: u32,
        state: &DecoderState,
        encoding: &Array3<f32>,
    ) -> Result<(Vec<f32>, DecoderState), String> {
        let targets = Array2::from_shape_vec((1, 1), vec![previous as i32])
            .map_err(|e| format!("failed to shape Nemotron target: {e}"))?;
        let target_length = Array1::from_vec(vec![1i32]);
        let outputs = self
            .decoder
            .run(ort::inputs![
                "encoder_outputs" => TensorRef::from_array_view(encoding)
                    .map_err(|e| format!("failed to build joint encoding: {e}"))?,
                "targets" => TensorRef::from_array_view(&targets)
                    .map_err(|e| format!("failed to build joint target: {e}"))?,
                "target_length" => TensorRef::from_array_view(&target_length)
                    .map_err(|e| format!("failed to build joint length: {e}"))?,
                "input_states_1" => TensorRef::from_array_view(&state.0)
                    .map_err(|e| format!("failed to build joint state 1: {e}"))?,
                "input_states_2" => TensorRef::from_array_view(&state.1)
                    .map_err(|e| format!("failed to build joint state 2: {e}"))?,
            ])
            .map_err(|e| format!("Nemotron joint failed: {e}"))?;

        let logits = {
            let (_, data) = outputs["outputs"]
                .try_extract_tensor::<f32>()
                .map_err(|e| format!("Nemotron joint logits were not float: {e}"))?;
            data.to_vec()
        };
        let next = (
            extract3(&outputs, "output_states_1")?,
            extract3(&outputs, "output_states_2")?,
        );
        Ok((logits, next))
    }

    /// RNN-T greedy over one chunk's encodings, carrying decoder state in.
    ///
    /// No duration head means no frame skip: every encoder frame gets a joint
    /// step, and `max_symbols_per_frame` is the only thing standing between a
    /// confident model and an infinite emission loop.
    fn decode_chunk(
        &mut self,
        encodings: &Array2<f32>,
        state: &mut DecoderState,
        previous: &mut u32,
        out: &mut Vec<u32>,
    ) -> Result<(), String> {
        for step in 0..encodings.shape()[0] {
            let frame = encodings
                .index_axis(Axis(0), step)
                .to_owned()
                .insert_axis(Axis(0))
                .insert_axis(Axis(2));
            let mut emitted = 0usize;
            while emitted < MAX_SYMBOLS_PER_FRAME {
                let (logits, next_state) = self.joint_step(*previous, state, &frame)?;
                let token = argmax(&logits) as u32;
                if token == BLANK_ID {
                    break;
                }
                out.push(token);
                *previous = token;
                *state = next_state;
                emitted += 1;
            }
        }
        Ok(())
    }

    /// A fresh incremental frontend over the same filterbank this model loaded.
    /// Clones the loaded frontend rather than re-reading `filterbank.bin`, so a
    /// capture cannot start against a different filterbank than the one the
    /// model was validated with.
    pub fn streaming_frontend(&self) -> Result<StreamingFrontend, String> {
        Ok(StreamingFrontend::from_frontend(self.frontend.clone()))
    }
}

fn argmax(values: &[f32]) -> usize {
    let mut best = 0usize;
    let mut best_value = f32::NEG_INFINITY;
    for (index, value) in values.iter().enumerate() {
        if *value > best_value {
            best_value = *value;
            best = index;
        }
    }
    best
}

fn extract3(
    outputs: &ort::session::SessionOutputs,
    name: &str,
) -> Result<Array3<f32>, String> {
    let (shape, data) = outputs[name]
        .try_extract_tensor::<f32>()
        .map_err(|e| format!("{name} was not float: {e}"))?;
    let dims = (shape[0] as usize, shape[1] as usize, shape[2] as usize);
    Array3::from_shape_vec(dims, data.to_vec())
        .map_err(|e| format!("failed to shape {name}: {e}"))
}

fn extract4(
    outputs: &ort::session::SessionOutputs,
    name: &str,
    dims: (usize, usize, usize, usize),
) -> Result<Array4<f32>, String> {
    let (_, data) = outputs[name]
        .try_extract_tensor::<f32>()
        .map_err(|e| format!("{name} was not float: {e}"))?;
    Array4::from_shape_vec(dims, data.to_vec())
        .map_err(|e| format!("failed to shape {name}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentencepiece_pieces_join_into_words() {
        let vocab = Vocab {
            pieces: {
                let mut pieces = vec![String::new(); BLANK_ID as usize];
                pieces[1] = "▁Hello".into();
                pieces[2] = "▁world".into();
                pieces[3] = "s".into();
                pieces
            },
        };
        // `▁` opens a word, a bare piece continues the previous one.
        assert_eq!(vocab.detokenize(&[1, 2, 3]), "Hello worlds");
    }

    #[test]
    fn blank_has_no_piece_and_is_never_emitted() {
        let vocab = Vocab {
            pieces: vec![String::new(); BLANK_ID as usize],
        };
        // Blank is 1024 and the table is 1024 long, so it cannot resolve even
        // if a decode bug let it through.
        assert_eq!(vocab.detokenize(&[BLANK_ID]), "");
    }
}

/// Nemotron's own frontend, built from the export's `preprocessor.config`.
///
/// **Gate 3 failed with `nemo128.onnx`.** `Parakeet-v3.md` §7 says the two
/// families declare identical extractors and that is true of every parameter
/// *except the one that matters*: Parakeet's graph applies NeMo's
/// `normalize: per_feature`, and Nemotron's config says `normalize: null`.
/// Feeding per-feature-normalised mel to Nemotron does not error — it produces
/// fluent, confident, wrong English, which is the expensive failure the gate
/// exists to catch. Measured: "I can feel some of this. I used to abstract two
/// key habits." against a recording that says nothing of the kind.
///
/// So this reimplements the frontend the export asks for: preemphasis 0.97,
/// 512-point Hann STFT at hop 160 / win 400, the shipped Slaney filterbank,
/// natural log, and **no normalisation at all**.
#[derive(Clone)]
struct Frontend {
    fft: std::sync::Arc<dyn realfft::RealToComplex<f32>>,
    window: Vec<f32>,
    /// `[128, 257]` row-major, straight from `filterbank.bin`.
    filters: Vec<f32>,
}

impl Frontend {
    fn new(filterbank: &Path) -> Result<Self, String> {
        let raw = std::fs::read(filterbank)
            .map_err(|e| format!("failed to read {}: {e}", filterbank.display()))?;
        let expected = MELS * (N_FFT / 2 + 1) * 4;
        if raw.len() != expected {
            return Err(format!(
                "filterbank is {} bytes, expected {expected}",
                raw.len()
            ));
        }
        let filters = raw
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        let mut planner = realfft::RealFftPlanner::<f32>::new();
        // Periodic Hann, matching librosa/NeMo — divisor is the window length,
        // not length - 1. A symmetric window here is a quiet half-bin error.
        let window = (0..WIN_LENGTH)
            .map(|i| 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / WIN_LENGTH as f32).cos())
            .collect();
        Ok(Self {
            fft: planner.plan_fft_forward(N_FFT),
            window,
            filters,
        })
    }

    /// One frame's power spectrum, `n_fft/2 + 1` bins.
    ///
    /// Shared by the batch and streaming paths on purpose: two copies of an
    /// STFT is two chances to drift, and a mel that drifts does not error, it
    /// invents words. Allocating the FFT buffers per call keeps this `&self`
    /// so both callers can hold the frontend immutably; it is a few hundred
    /// nanoseconds against ~60 ms of encoder per chunk.
    fn spectrum(&self, window: &[f32]) -> Result<Vec<f32>, String> {
        let mut input = self.fft.make_input_vec();
        let mut output = self.fft.make_output_vec();
        let mut scratch = self.fft.make_scratch_vec();
        input.copy_from_slice(window);
        self.fft
            .process_with_scratch(&mut input, &mut output, &mut scratch)
            .map_err(|e| format!("Nemotron FFT failed: {e}"))?;
        Ok(output.iter().map(|value| value.norm_sqr()).collect())
    }

    /// Reference implementation, kept for the test that pins the incremental
    /// frontend against it. Production uses [`StreamingFrontend`]; if these two
    /// ever disagree the streamed transcript and a `--wav` transcript stop being
    /// comparable, which is the whole reason this stays.
    #[cfg(test)]
    fn extract(&self, audio: &[f32]) -> Result<Array3<f32>, String> {
        // Preemphasis first, on the raw signal, exactly as NeMo orders it.
        let mut signal = Vec::with_capacity(audio.len());
        signal.push(audio.first().copied().unwrap_or(0.0));
        for i in 1..audio.len() {
            signal.push(audio[i] - PREEMPH * audio[i - 1]);
        }

        // NeMo centres the STFT, so the first frame is centred on sample 0.
        let padded = reflect_pad(&signal, N_FFT / 2);
        let frames = signal.len() / HOP_LENGTH + 1;
        let mut mel = Array3::<f32>::zeros((1, MELS, frames));
        let offset = (N_FFT - WIN_LENGTH) / 2;

        for frame in 0..frames {
            let mut window = vec![0.0f32; N_FFT];
            let start = frame * HOP_LENGTH;
            for i in 0..WIN_LENGTH {
                let Some(sample) = padded.get(start + i) else {
                    break;
                };
                window[offset + i] = sample * self.window[i];
            }
            let spectrum = self.spectrum(&window)?;
            for band in 0..MELS {
                let row = band * (N_FFT / 2 + 1);
                let mut power = 0.0f32;
                for (bin, value) in spectrum.iter().enumerate() {
                    power += value * self.filters[row + bin];
                }
                // Natural log with NeMo's guard. No per-feature normalisation:
                // that is the whole point of this function existing.
                mel[[0, band, frame]] = (power + LOG_GUARD).ln();
            }
        }
        Ok(mel)
    }
}

/// Mirror the signal about its edges, the `pad_mode="reflect"` librosa default.
///
/// Only the reference [`Frontend::extract`] needs this; the streaming path
/// resolves the same mirror index-wise in [`reflect_index`].
#[cfg(test)]
fn reflect_pad(signal: &[f32], pad: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(signal.len() + pad * 2);
    for i in (1..=pad).rev() {
        out.push(signal.get(i).copied().unwrap_or(0.0));
    }
    out.extend_from_slice(signal);
    for i in 1..=pad {
        out.push(
            signal
                .len()
                .checked_sub(1 + i)
                .and_then(|index| signal.get(index))
                .copied()
                .unwrap_or(0.0),
        );
    }
    out
}

/// Session and chunk knobs, so the RAM and speed claims get measured rather
/// than argued.
///
/// The three session knobs all target the same thing: peak RSS is dominated not
/// by the 799 MB of encoder weights on disk but by what ORT does with them at
/// init and at first run.
///
/// - **prepacking** rearranges quantised MatMul weights into a kernel-friendly
///   layout — a large transient *and* a retained second copy at this size.
/// - **device-allocated initializers** bypass the BFC arena for weights, so
///   the arena never has to grow to hold a copy of them.
/// - **memory pattern** pre-plans activations from the first run's shapes.
///   Good for fixed shapes, wasted when the final chunk is short.
///
/// `chunk` is the fifth item on the list: 1120 ms halves the number of encoder
/// invocations and with it the redundant 9-frame pre-encode context each one
/// re-encodes.
#[derive(Clone, Copy)]
pub struct Tuning {
    pub no_prepack: bool,
    pub device_initializers: bool,
    pub no_mem_pattern: bool,
    pub chunk_frames: usize,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            no_prepack: false,
            device_initializers: false,
            no_mem_pattern: false,
            chunk_frames: CHUNK_MEL_FRAMES,
        }
    }
}

impl Tuning {
    fn builder(&self) -> Result<ort::session::builder::SessionBuilder, String> {
        let mut builder =
            Session::builder().map_err(|e| format!("failed to create session builder: {e}"))?;
        if self.no_prepack {
            builder = builder
                .with_prepacking(false)
                .map_err(|e| format!("failed to disable prepacking: {e}"))?;
        }
        if self.device_initializers {
            builder = builder
                .with_device_allocated_initializers()
                .map_err(|e| format!("failed to set device initializers: {e}"))?;
        }
        if self.no_mem_pattern {
            builder = builder
                .with_memory_pattern(false)
                .map_err(|e| format!("failed to disable memory pattern: {e}"))?;
        }
        Ok(builder)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Service: route 4 wired into the live path.
//
// Same shape as `ParakeetService` and `GraniteService` deliberately — one
// worker thread holding one loaded model, a job channel the main loop owns, an
// idle timeout, and a `shutdown` that frees the model on a mode switch. A
// fourth shape here would be a fourth thing to keep in step.
//
// **It encodes while you are still speaking.** The worker resamples, computes
// mel, encodes and decodes inside the drain loop, so the wait after the toggle
// is one chunk plus a tail rather than the whole capture's compute. That rests
// on `audio::StreamingResampler` and `StreamingFrontend`, each pinned by a test
// against its batch equivalent: splitting the whole-buffer polyphase decimator
// naively puts a discontinuity at every seam, the class of defect A.1 removed.
// ─────────────────────────────────────────────────────────────────────────────

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

    let mut model = match Nemotron::load(std::path::Path::new(&inner.model_dir), Tuning::default())
    {
        Ok(model) => {
            observer.phase("worker.start", "Nemotron model loaded");
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
    let mut resampler =
        crate::audio::StreamingResampler::new(source_sample_rate, 16_000);
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

/// [`Frontend`] that can be fed in pieces.
///
/// The batch `extract` preemphasises the whole signal, reflect-pads both ends,
/// and then walks every frame. Streaming has to produce *identical* frames from
/// a growing buffer, so three pieces of state carry across calls:
///
/// 1. **One sample of preemphasis history.** `y[n] = x[n] - 0.97·x[n-1]`, so a
///    chunk boundary without the previous raw sample puts a step in the signal.
/// 2. **A sliding window of preemphasised signal.** Frame `f` reads
///    `signal[f·160-256 .. f·160+143]` once the centring pad is accounted for,
///    so the buffer keeps everything still reachable and drops the rest.
/// 3. **The frame counter**, because the reflect pad belongs to the *start of
///    the capture*, not to the start of each chunk.
///
/// A frame is emitted only when its whole window is real data. The final frames,
/// which need the trailing reflect pad, come out of [`finish`] — which is what
/// makes a streamed capture agree with a batched one frame for frame.
pub struct StreamingFrontend {
    inner: Frontend,
    /// Preemphasised signal still reachable by an unemitted frame.
    signal: Vec<f32>,
    /// Absolute signal index of `signal[0]`.
    signal_start: usize,
    /// Last *raw* sample seen, for preemphasis across a seam.
    last_raw: Option<f32>,
    /// Next frame index to emit.
    next_frame: usize,
    /// Total preemphasised samples ever produced.
    total: usize,
}

impl StreamingFrontend {
    /// Tests build one straight from a filterbank; the service clones the
    /// model's loaded frontend instead, via `Nemotron::streaming_frontend`.
    #[cfg(test)]
    pub fn new(filterbank: &Path) -> Result<Self, String> {
        Ok(Self::from_frontend(Frontend::new(filterbank)?))
    }

    fn from_frontend(inner: Frontend) -> Self {
        Self {
            inner,
            signal: Vec::new(),
            signal_start: 0,
            last_raw: None,
            next_frame: 0,
            total: 0,
        }
    }

    /// Feed 16 kHz mono audio; get back `[1, 128, n]` for every frame now
    /// fully covered, or `None` when no frame completed.
    pub fn push(&mut self, audio: &[f32]) -> Result<Option<Array3<f32>>, String> {
        for &sample in audio {
            let previous = self.last_raw;
            self.signal.push(match previous {
                Some(prev) => sample - PREEMPH * prev,
                // The batch filter leaves the very first sample untouched.
                None => sample,
            });
            self.last_raw = Some(sample);
        }
        self.total += audio.len();

        // Two separate requirements, and missing the second one silently zeroed
        // the first frames of every capture:
        //
        // - the window's right edge needs `f*HOP + WIN_LENGTH - N_FFT/2`;
        // - the *left* edge of an early frame is reflect-padded, and the mirror
        //   reads *forward* into the signal — frame 0 reads signal[256]. So a
        //   frame near the start needs more audio than its own span, not less.
        let reach = WIN_LENGTH - N_FFT / 2;
        let pad = N_FFT / 2;
        let mut frames = 0usize;
        loop {
            let start = self.next_frame * HOP_LENGTH;
            let mirror_depth = pad.saturating_sub(start);
            let needed = (start + reach).max(if mirror_depth > 0 { mirror_depth + 1 } else { 0 });
            if needed > self.total {
                break;
            }
            frames += 1;
            self.next_frame += 1;
        }
        if frames == 0 {
            return Ok(None);
        }
        let first = self.next_frame - frames;
        let mel = self.render(first, frames, None)?;
        self.trim();
        Ok(Some(mel))
    }

    /// Emit the frames that need the trailing reflect pad.
    pub fn finish(&mut self) -> Result<Option<Array3<f32>>, String> {
        let total_frames = self.total / HOP_LENGTH + 1;
        if self.next_frame >= total_frames {
            return Ok(None);
        }
        let frames = total_frames - self.next_frame;
        let first = self.next_frame;
        self.next_frame = total_frames;
        let mel = self.render(first, frames, Some(self.total))?;
        Ok(Some(mel))
    }

    /// Frames `[first, first + count)`, reading the retained signal directly.
    ///
    /// `reflect_end` is the absolute signal length once the capture is known to
    /// be complete; until then a frame that would read past the end is not
    /// emitted at all, so the pad is never fabricated mid-capture.
    fn render(
        &self,
        first: usize,
        count: usize,
        reflect_end: Option<usize>,
    ) -> Result<Array3<f32>, String> {
        let mut mel = Array3::<f32>::zeros((1, MELS, count));
        let pad = N_FFT / 2;
        for slot in 0..count {
            let frame = first + slot;
            let mut window = vec![0.0f32; N_FFT];
            let offset = (N_FFT - WIN_LENGTH) / 2;
            for i in 0..WIN_LENGTH {
                // Padded coordinate `frame*HOP + i` maps to signal index minus
                // the centring pad; outside the signal the batch path mirrors
                // about the edges.
                let padded = (frame * HOP_LENGTH + i) as isize - pad as isize;
                let index = match reflect_index(padded, self.total, reflect_end) {
                    Some(index) => index,
                    None => continue,
                };
                let local = index as isize - self.signal_start as isize;
                if local < 0 || local as usize >= self.signal.len() {
                    continue;
                }
                window[offset + i] = self.signal[local as usize] * self.inner.window[i];
            }
            let spectrum = self.inner.spectrum(&window)?;
            for band in 0..MELS {
                let row = band * (N_FFT / 2 + 1);
                let mut power = 0.0f32;
                for (bin, value) in spectrum.iter().enumerate() {
                    power += value * self.inner.filters[row + bin];
                }
                mel[[0, band, slot]] = (power + LOG_GUARD).ln();
            }
        }
        Ok(mel)
    }

    fn trim(&mut self) {
        let oldest = (self.next_frame * HOP_LENGTH).saturating_sub(N_FFT / 2);
        if oldest > self.signal_start {
            let drop = (oldest - self.signal_start).min(self.signal.len());
            self.signal.drain(..drop);
            self.signal_start += drop;
        }
    }
}

/// Mirror an out-of-range index back inside, matching `reflect_pad`.
///
/// Returns `None` for an index past the end while the capture is still running,
/// which is the signal to hold the frame back rather than invent data for it.
fn reflect_index(index: isize, total: usize, reflect_end: Option<usize>) -> Option<usize> {
    if index >= 0 && (index as usize) < total {
        return Some(index as usize);
    }
    if index < 0 {
        // reflect_pad emits signal[pad], …, signal[1] before the signal, i.e. a
        // mirror about sample 0 that does not repeat it.
        let mirrored = (-index) as usize;
        return if mirrored < total { Some(mirrored) } else { None };
    }
    let end = reflect_end?;
    // Mirror about the last sample, likewise without repeating it.
    let over = index as usize - end + 1;
    end.checked_sub(1 + over)
}

#[cfg(test)]
mod streaming_frontend_tests {
    use super::*;
    use ndarray::Axis;

    /// The filterbank ships with the export, so this test needs a real one.
    /// Skips rather than fails when no Nemotron is installed, because the unit
    /// suite has to pass on a machine that has never downloaded a model.
    fn filterbank() -> Option<std::path::PathBuf> {
        crate::model::discover_models(None)
            .into_iter()
            .find(|model| model.kind == crate::model::ModelKind::Nemotron)
            .map(|model| model.path.join("filterbank.bin"))
            .filter(|path| path.is_file())
    }

    fn tone(samples: usize) -> Vec<f32> {
        (0..samples)
            .map(|i| {
                let t = i as f32 / 16000.0;
                0.4 * (2.0 * std::f32::consts::PI * 220.0 * t).sin()
                    + 0.2 * (2.0 * std::f32::consts::PI * 1750.0 * t).sin()
            })
            .collect()
    }

    /// The property the whole streaming path rests on: fed in arbitrary pieces,
    /// the incremental frontend must produce the same mel as one batch call.
    /// If it does not, the streamed transcript and the `--wav` transcript are
    /// not comparable, and every accuracy number taken through one of them
    /// says nothing about the other.
    #[test]
    fn streamed_mel_matches_batch_mel() {
        let Some(filterbank) = filterbank() else {
            eprintln!("no Nemotron install; skipping");
            return;
        };
        let audio = tone(16000);
        let batch = Frontend::new(&filterbank)
            .expect("filterbank should load")
            .extract(&audio)
            .expect("batch mel");

        for chunk in [160usize, 1024, 8960, 17920] {
            let mut streaming =
                StreamingFrontend::new(&filterbank).expect("filterbank should load");
            let mut frames: Vec<Array3<f32>> = Vec::new();
            for piece in audio.chunks(chunk) {
                if let Some(mel) = streaming.push(piece).expect("push") {
                    frames.push(mel);
                }
            }
            if let Some(mel) = streaming.finish().expect("finish") {
                frames.push(mel);
            }
            let streamed: usize = frames.iter().map(|mel| mel.shape()[2]).sum();
            assert_eq!(
                streamed,
                batch.shape()[2],
                "frame count differs at chunk {chunk}"
            );

            let mut column = 0usize;
            for mel in &frames {
                for slot in 0..mel.shape()[2] {
                    for band in 0..MELS {
                        let a = mel[[0, band, slot]];
                        let b = batch[[0, band, column]];
                        assert!(
                            (a - b).abs() < 1e-3,
                            "chunk {chunk}, frame {column}, band {band}: {a} vs {b}"
                        );
                    }
                    column += 1;
                }
            }
            assert_eq!(column, batch.shape()[2]);
            let _ = batch.index_axis(Axis(0), 0);
        }
    }

    #[test]
    fn preemphasis_carries_across_a_seam() {
        // y[n] = x[n] - 0.97·x[n-1]. Split without the previous raw sample and
        // the first sample of every chunk is wrong by 0.97·x[n-1], which is a
        // step, not a rounding error.
        let Some(filterbank) = filterbank() else {
            eprintln!("no Nemotron install; skipping");
            return;
        };
        let mut streaming = StreamingFrontend::new(&filterbank).expect("load");
        // Under the 257 samples the first frame needs, so nothing is emitted
        // and nothing is trimmed — the buffer still starts at absolute 0.
        let _ = streaming.push(&[1.0f32; 100]).expect("push");
        assert_eq!(streaming.last_raw, Some(1.0));
        // A constant signal preemphasises to 1 - 0.97 = 0.03 everywhere after
        // the untouched first sample.
        assert!((streaming.signal[1] - 0.03).abs() < 1e-6);
        assert!((streaming.signal[0] - 1.0).abs() < 1e-6);
    }
}

/// Everything one capture carries between chunks.
///
/// The encoder cache is the whole point: it is what makes attention continuous
/// across a capture instead of restarting at every chunk boundary. Zeroed once
/// here and then fed forward, never reset — which is also why `wav.rs` must not
/// window this engine.
pub struct StreamState {
    cache: Cache,
    decoder: DecoderState,
    previous: u32,
    tokens: Vec<u32>,
    /// Mel frames not yet part of a complete chunk, each 128 values.
    pending: Vec<Vec<f32>>,
    /// The trailing frames of the last chunk, re-fed as pre-encode context.
    context: Vec<Vec<f32>>,
    chunks: usize,
}

impl Nemotron {
    pub fn stream_begin(&self) -> StreamState {
        StreamState {
            cache: Cache::zeroed(&self.layout),
            decoder: (
                Array3::zeros((2, 1, PRED_HIDDEN)),
                Array3::zeros((2, 1, PRED_HIDDEN)),
            ),
            previous: BLANK_ID,
            tokens: Vec::new(),
            pending: Vec::new(),
            context: Vec::new(),
            chunks: 0,
        }
    }

    /// Feed newly available mel frames, encoding and decoding every chunk that
    /// completes. Returns the text emitted so far.
    ///
    /// This is the call that makes route 4 worth having: it runs while the user
    /// is still talking, so the wait after the toggle is one chunk plus a tail
    /// decode rather than the whole capture's worth of compute.
    pub fn stream_push(
        &mut self,
        state: &mut StreamState,
        mel: &Array3<f32>,
    ) -> Result<String, String> {
        for slot in 0..mel.shape()[2] {
            state
                .pending
                .push((0..MELS).map(|band| mel[[0, band, slot]]).collect());
        }
        let chunk = self.tuning.chunk_frames;
        while state.pending.len() >= chunk {
            let frames: Vec<Vec<f32>> = state.pending.drain(..chunk).collect();
            self.stream_chunk(state, &frames)?;
        }
        Ok(self.vocab.detokenize(&state.tokens))
    }

    /// Flush whatever is left, however short, and return the final transcript.
    pub fn stream_finish(&mut self, state: &mut StreamState) -> Result<(String, usize), String> {
        if !state.pending.is_empty() {
            let frames: Vec<Vec<f32>> = std::mem::take(&mut state.pending);
            self.stream_chunk(state, &frames)?;
        }
        Ok((self.vocab.detokenize(&state.tokens), state.chunks))
    }

    fn stream_chunk(
        &mut self,
        state: &mut StreamState,
        frames: &[Vec<f32>],
    ) -> Result<(), String> {
        // The encoder expects the chunk preceded by `PRE_ENCODE_CACHE_FRAMES`
        // of context. The first chunk has none, so it is zero-padded — same as
        // the offline path, which is what keeps the two agreeing.
        let missing = PRE_ENCODE_CACHE_FRAMES - state.context.len();
        let width = PRE_ENCODE_CACHE_FRAMES + frames.len();
        let mut window = Array3::<f32>::zeros((1, MELS, width));
        for (slot, frame) in state.context.iter().enumerate() {
            for band in 0..MELS {
                window[[0, band, missing + slot]] = frame[band];
            }
        }
        for (slot, frame) in frames.iter().enumerate() {
            for band in 0..MELS {
                window[[0, band, PRE_ENCODE_CACHE_FRAMES + slot]] = frame[band];
            }
        }

        let encodings = self.encode_chunk(&window, &mut state.cache)?;
        let mut decoder = std::mem::replace(
            &mut state.decoder,
            (
                Array3::zeros((2, 1, PRED_HIDDEN)),
                Array3::zeros((2, 1, PRED_HIDDEN)),
            ),
        );
        let mut previous = state.previous;
        self.decode_chunk(
            &encodings,
            &mut decoder,
            &mut previous,
            &mut state.tokens,
        )?;
        state.decoder = decoder;
        state.previous = previous;

        // Carry the tail forward as the next chunk's context.
        let keep = frames.len().min(PRE_ENCODE_CACHE_FRAMES);
        state.context = frames[frames.len() - keep..].to_vec();
        state.chunks += 1;
        Ok(())
    }
}
