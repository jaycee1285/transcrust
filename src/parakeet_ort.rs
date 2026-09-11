//! Parakeet TDT driven directly through `ort`, instead of through `parakeet-rs`.
//!
//! Why this exists: the crate runs the decode internally and hands back a
//! `String`, discarding the joint network's vocab logits. That made it look
//! like Parakeet exposes no confidence signal — `design-dictation-as-control.md`
//! recorded exactly that, and it sent the command-channel plan toward Granite.
//! The logits were always in `decoder_joint-model.onnx`. Driving the graphs
//! ourselves — the same pattern `granite.rs` already uses —
//! yields per-token probability, word confidence and frame timestamps from the
//! files already on disk.
//!
//! Three graphs, all NVIDIA/istupakov exports:
//!
//! ```text
//! nemo128.onnx              waveforms [1, N] @16k   -> features [1, 128, T]
//! encoder-model.onnx        audio_signal [1,128,T]  -> outputs [1, 1024, T/8]
//! decoder_joint-model.onnx  one encoder frame + LSTM state
//!                                                   -> 8198 logits, next state
//! ```
//!
//! The joint's 8198 outputs are 8193 vocabulary logits (blank = 8192) followed
//! by 5 duration logits — the TDT head that says how many encoder frames to
//! skip. Reference for the decode: `murmure/src-tauri/src/engine/engine.rs`.

use std::path::Path;

use ndarray::{Array1, Array2, Array3, ArrayD, Axis};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::TensorRef;

/// Blank is the last vocabulary entry; the 5 trailing logits are the duration
/// head. Both are read from the graph at load time rather than hardcoded —
/// these are only the expected values, asserted once.
const EXPECTED_JOINT_WIDTH: usize = 8198;
/// Upstream's cap on tokens emitted at a single encoder frame before the
/// decoder is forced to advance. Matches NeMo and `parakeet-rs`.
const MAX_SYMBOLS_PER_STEP: usize = 10;
const SAMPLE_RATE: u32 = 16_000;

/// LSTM prediction-network state: two `[2, batch, 640]` tensors.
type DecoderState = (Array3<f32>, Array3<f32>);

/// One emitted token with the evidence the crate used to throw away.
#[derive(Clone, Debug, PartialEq)]
pub struct TimedToken {
    pub id: u32,
    /// Encoder frame the token was emitted at. At subsampling 8 and hop 160,
    /// one frame is 80 ms.
    pub frame: usize,
    /// Softmax probability of this token over the raw vocabulary logits.
    pub probability: f32,
}

/// A decoded utterance plus its per-token evidence.
#[derive(Clone, Debug, Default)]
pub struct Decoded {
    pub text: String,
    pub tokens: Vec<TimedToken>,
}

impl Decoded {
    /// Per-word confidence: the minimum token probability across the word — the
    /// weakest link, not the average, because one badly-heard sub-token is
    /// enough to make the whole word wrong.
    ///
    /// Parakeet's tokenizer is SentencePiece, so a word starts at the boundary
    /// marker `▁` (already rendered as a leading space by `Vocabulary`), not
    /// at a ByteLevel `Ġ`.
    ///
    /// Punctuation-only pieces are appended to the word but **excluded from the
    /// minimum**. Deciding where a period goes is genuinely uncertain and
    /// scores low; letting that dominate would make every sentence-final word
    /// look unconfident, and a confidence gate would then happily
    /// fuzzy-correct the last word of every sentence. Measured on a real clip
    /// before this exclusion: `phenomenal.` 0.589, `home.` 0.623, `you.` 0.605
    /// — all of them words the model actually heard perfectly.
    pub fn word_confidences(&self, vocab: &Vocabulary) -> Vec<(String, f32)> {
        let mut words: Vec<(String, f32)> = Vec::new();
        for token in &self.tokens {
            let piece = vocab.piece(token.id);
            let starts_word = piece.starts_with(' ') || words.is_empty();
            let scores = !is_punctuation_only(piece);
            if starts_word {
                let confidence = if scores { token.probability } else { f32::INFINITY };
                words.push((piece.trim_start().to_string(), confidence));
            } else if let Some(last) = words.last_mut() {
                last.0.push_str(piece);
                if scores {
                    last.1 = last.1.min(token.probability);
                }
            }
        }
        words.retain(|(word, _)| !word.trim().is_empty());
        // A word made only of punctuation never scored; report it as certain
        // rather than as +inf.
        for (_, confidence) in &mut words {
            if !confidence.is_finite() {
                *confidence = 1.0;
            }
        }
        words
    }
}

/// A piece carrying no letters or digits — a period, comma, or the space
/// before one. These are excluded from word confidence; see
/// [`Decoded::word_confidences`].
fn is_punctuation_only(piece: &str) -> bool {
    !piece.is_empty() && !piece.chars().any(char::is_alphanumeric)
}

/// `vocab.txt` as shipped: `<token> <id>` per line, SentencePiece `▁` rendered
/// as a leading space so pieces concatenate into text directly.
pub struct Vocabulary {
    pieces: Vec<String>,
    blank_id: u32,
}

impl Vocabulary {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let mut entries: Vec<(String, usize)> = Vec::new();
        let mut blank_id = None;
        let mut max_id = 0usize;
        for line in text.lines() {
            // Split from the right: the token itself may contain a space.
            let Some((token, id)) = line.trim_end().rsplit_once(' ') else {
                continue;
            };
            let Ok(id) = id.parse::<usize>() else { continue };
            if token == "<blk>" {
                blank_id = Some(id as u32);
            }
            max_id = max_id.max(id);
            entries.push((token.to_string(), id));
        }
        let blank_id =
            blank_id.ok_or_else(|| format!("no <blk> token in {}", path.display()))?;
        let mut pieces = vec![String::new(); max_id + 1];
        for (token, id) in entries {
            pieces[id] = token.replace('\u{2581}', " ");
        }
        Ok(Self { pieces, blank_id })
    }

    pub fn piece(&self, id: u32) -> &str {
        self.pieces.get(id as usize).map_or("", String::as_str)
    }

    pub fn blank_id(&self) -> u32 {
        self.blank_id
    }

    pub fn len(&self) -> usize {
        self.pieces.len()
    }

    pub fn decode(&self, tokens: &[TimedToken]) -> String {
        let mut text = String::new();
        for token in tokens {
            text.push_str(self.piece(token.id));
        }
        text.trim().to_string()
    }
}

pub struct LoadedParakeet {
    preprocessor: Session,
    encoder: Session,
    joint: Session,
    vocab: Vocabulary,
    /// Width of the vocabulary slice of the joint output; the remainder is the
    /// duration head.
    vocab_width: usize,
}

fn build_session(path: &Path) -> Result<Session, String> {
    Session::builder()
        .map_err(|e| format!("failed to create ONNX session builder: {e}"))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| format!("failed to set ONNX optimization level: {e}"))?
        .with_inter_threads(1)
        .map_err(|e| format!("failed to set ONNX inter-op threads: {e}"))?
        .commit_from_file(path)
        .map_err(|e| format!("failed to load {}: {e}", path.display()))
}

impl LoadedParakeet {
    pub fn load(model_dir: &Path) -> Result<Self, String> {
        let graphs = crate::model::parakeet_direct_graphs(model_dir).ok_or_else(|| {
            format!(
                "no direct-drive Parakeet graph set in {} (needs nemo128*.onnx)",
                model_dir.display()
            )
        })?;
        let preprocessor = build_session(&graphs.preprocessor)?;
        let encoder = build_session(&graphs.encoder)?;
        let joint = build_session(&graphs.decoder_joint)?;
        let vocab = Vocabulary::load(&model_dir.join("vocab.txt"))?;

        // The joint emits vocabulary logits followed by the duration head. Read
        // the split from the graph rather than trusting a constant.
        let joint_width = joint
            .outputs()
            .iter()
            .find(|output| output.name() == "outputs")
            .and_then(|output| output.dtype().tensor_shape().map(|shape| shape.to_vec()))
            .and_then(|shape| shape.last().copied())
            .filter(|width| *width > 0)
            .map(|width| width as usize)
            .unwrap_or(EXPECTED_JOINT_WIDTH);
        if joint_width <= vocab.len() {
            return Err(format!(
                "joint emits {joint_width} logits but the vocabulary has {}; \
                 this export has no duration head and is not TDT",
                vocab.len()
            ));
        }

        Ok(Self {
            preprocessor,
            encoder,
            joint,
            vocab_width: vocab.len(),
            vocab,
        })
    }

    pub fn vocabulary(&self) -> &Vocabulary {
        &self.vocab
    }

    /// `audio` must already be 16 kHz mono.
    pub fn transcribe(&mut self, audio: &[f32]) -> Result<Decoded, String> {
        let features = self.preprocess(audio)?;
        let (encodings, frames) = self.encode(&features)?;
        let tokens = self.decode_greedy(&encodings, frames)?;
        Ok(Decoded {
            text: self.vocab.decode(&tokens),
            tokens,
        })
    }

    /// Raw waveform to 128-bin log-mel, via NeMo's own preprocessor exported as
    /// a graph. Deliberately not reimplemented in Rust: matching NeMo's
    /// preemphasis, window, filterbank, log guard and per-feature Bessel-
    /// corrected normalisation by hand is the exact class of silent-drift bug
    /// that is expensive to find and free to avoid.
    fn preprocess(&mut self, audio: &[f32]) -> Result<Array3<f32>, String> {
        let samples = audio.len();
        let waveforms = Array2::from_shape_vec((1, samples), audio.to_vec())
            .map_err(|e| format!("failed to shape Parakeet waveform: {e}"))?;
        let lengths = Array1::from_vec(vec![samples as i64]);
        let outputs = self
            .preprocessor
            .run(ort::inputs![
                "waveforms" => TensorRef::from_array_view(&waveforms)
                    .map_err(|e| format!("failed to build waveform tensor: {e}"))?,
                "waveforms_lens" => TensorRef::from_array_view(&lengths)
                    .map_err(|e| format!("failed to build waveform length tensor: {e}"))?,
            ])
            .map_err(|e| format!("Parakeet preprocessor failed: {e}"))?;
        let (shape, data) = outputs["features"]
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Parakeet features were not a float tensor: {e}"))?;
        if shape.len() != 3 || shape[1] != 128 {
            return Err(format!("unexpected Parakeet feature shape: {shape:?}"));
        }
        Array3::from_shape_vec((1, 128, shape[2] as usize), data.to_vec())
            .map_err(|e| format!("failed to shape Parakeet features: {e}"))
    }

    /// Returns encodings as `[time, 1024]` plus the valid frame count.
    fn encode(&mut self, features: &Array3<f32>) -> Result<(Array2<f32>, usize), String> {
        let lengths = Array1::from_vec(vec![features.shape()[2] as i64]);
        let outputs = self
            .encoder
            .run(ort::inputs![
                "audio_signal" => TensorRef::from_array_view(features)
                    .map_err(|e| format!("failed to build encoder tensor: {e}"))?,
                "length" => TensorRef::from_array_view(&lengths)
                    .map_err(|e| format!("failed to build encoder length tensor: {e}"))?,
            ])
            .map_err(|e| format!("Parakeet encoder failed: {e}"))?;

        let frames = {
            let (_, lens) = outputs["encoded_lengths"]
                .try_extract_tensor::<i64>()
                .map_err(|e| format!("Parakeet encoded_lengths were not int64: {e}"))?;
            lens.first().copied().unwrap_or(0).max(0) as usize
        };
        let (shape, data) = outputs["outputs"]
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Parakeet encodings were not a float tensor: {e}"))?;
        if shape.len() != 3 || shape[1] != 1024 {
            return Err(format!("unexpected Parakeet encoding shape: {shape:?}"));
        }
        // The graph emits [1, 1024, time]; the decode wants a frame at a time.
        let time = shape[2] as usize;
        let mut encodings = Array2::<f32>::zeros((time, 1024));
        for channel in 0..1024 {
            for step in 0..time {
                encodings[[step, channel]] = data[channel * time + step];
            }
        }
        Ok((encodings, frames.min(time)))
    }

    fn zero_state(&self) -> DecoderState {
        (Array3::zeros((2, 1, 640)), Array3::zeros((2, 1, 640)))
    }

    /// One joint step at encoder frame `encoding`, conditioned on the last
    /// emitted token and the LSTM state.
    fn joint_step(
        &mut self,
        previous: Option<u32>,
        state: &DecoderState,
        encoding: &Array3<f32>,
    ) -> Result<(ArrayD<f32>, DecoderState), String> {
        let target = previous.unwrap_or(self.vocab.blank_id()) as i32;
        let targets = Array2::from_shape_vec((1, 1), vec![target])
            .map_err(|e| format!("failed to shape Parakeet target: {e}"))?;
        let target_length = Array1::from_vec(vec![1i32]);

        let outputs = self
            .joint
            .run(ort::inputs![
                "encoder_outputs" => TensorRef::from_array_view(encoding)
                    .map_err(|e| format!("failed to build joint encoding tensor: {e}"))?,
                "targets" => TensorRef::from_array_view(&targets)
                    .map_err(|e| format!("failed to build joint target tensor: {e}"))?,
                "target_length" => TensorRef::from_array_view(&target_length)
                    .map_err(|e| format!("failed to build joint length tensor: {e}"))?,
                "input_states_1" => TensorRef::from_array_view(&state.0)
                    .map_err(|e| format!("failed to build joint state 1: {e}"))?,
                "input_states_2" => TensorRef::from_array_view(&state.1)
                    .map_err(|e| format!("failed to build joint state 2: {e}"))?,
            ])
            .map_err(|e| format!("Parakeet joint failed: {e}"))?;

        let logits = {
            let (shape, data) = outputs["outputs"]
                .try_extract_tensor::<f32>()
                .map_err(|e| format!("Parakeet joint logits were not float: {e}"))?;
            let width = *shape.last().unwrap_or(&0) as usize;
            ArrayD::from_shape_vec(ndarray::IxDyn(&[width]), data.to_vec())
                .map_err(|e| format!("failed to shape Parakeet joint logits: {e}"))?
        };
        let next = (
            extract_state(&outputs, "output_states_1")?,
            extract_state(&outputs, "output_states_2")?,
        );
        Ok((logits, next))
    }

    /// TDT greedy decode.
    ///
    /// Two things distinguish this from an RNN-T loop and both matter:
    /// the duration head chooses how many encoder frames to skip after each
    /// step (which is why Parakeet's cost is sublinear in tokens), and the
    /// LSTM state only advances on a non-blank emission.
    fn decode_greedy(
        &mut self,
        encodings: &Array2<f32>,
        frames: usize,
    ) -> Result<Vec<TimedToken>, String> {
        let blank = self.vocab.blank_id();
        let mut state = self.zero_state();
        let mut tokens: Vec<TimedToken> = Vec::new();
        let mut previous: Option<u32> = None;
        let mut step = 0usize;
        let mut emitted_here = 0usize;

        while step < frames {
            // The joint wants [1, 1024, 1]: one batch, 1024 channels, one frame.
            let frame = encodings
                .index_axis(Axis(0), step)
                .to_owned()
                .insert_axis(Axis(0))
                .insert_axis(Axis(2));
            let (logits, next_state) = self.joint_step(previous, &state, &frame)?;
            let slice = logits
                .as_slice()
                .ok_or_else(|| "Parakeet joint logits were not contiguous".to_string())?;
            let (vocab_logits, duration_logits) = slice.split_at(self.vocab_width);

            let token = argmax(vocab_logits) as u32;
            if token != blank {
                tokens.push(TimedToken {
                    id: token,
                    frame: step,
                    // Probability from the raw logits — the value the crate
                    // computes and discards.
                    probability: softmax_at(vocab_logits, token as usize),
                });
                previous = Some(token);
                state = next_state;
                emitted_here += 1;
            }

            let duration = argmax(duration_logits);
            if duration > 0 {
                step += duration;
                emitted_here = 0;
            } else if token == blank || emitted_here >= MAX_SYMBOLS_PER_STEP {
                // Duration 0 with a real token means "same frame, keep going";
                // only a blank or the symbol cap forces the frame forward.
                step += 1;
                emitted_here = 0;
            }
        }
        Ok(tokens)
    }
}

fn extract_state(
    outputs: &ort::session::SessionOutputs,
    name: &str,
) -> Result<Array3<f32>, String> {
    let (shape, data) = outputs[name]
        .try_extract_tensor::<f32>()
        .map_err(|e| format!("Parakeet {name} was not a float tensor: {e}"))?;
    if shape.len() != 3 {
        return Err(format!("unexpected Parakeet {name} shape: {shape:?}"));
    }
    Array3::from_shape_vec(
        (shape[0] as usize, shape[1] as usize, shape[2] as usize),
        data.to_vec(),
    )
    .map_err(|e| format!("failed to shape Parakeet {name}: {e}"))
}

/// First-wins argmax, matching `numpy.argmax` and therefore NeMo.
///
/// Not `max_by`: that returns the *last* maximum on a tie, which would silently
/// disagree with the Python the export was validated against. Exact ties in
/// float logits are vanishingly rare, but the duration head is only 5 wide and
/// a disagreement there changes how many frames get skipped.
fn argmax(values: &[f32]) -> usize {
    let mut best = 0usize;
    let mut best_value = f32::NEG_INFINITY;
    for (index, value) in values.iter().copied().enumerate() {
        if value > best_value {
            best_value = value;
            best = index;
        }
    }
    best
}

/// Softmax probability of one index, computed with the max subtracted so a
/// large logit cannot overflow `exp`.
fn softmax_at(logits: &[f32], index: usize) -> f32 {
    let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    if !max.is_finite() {
        return 0.0;
    }
    let total: f32 = logits.iter().map(|value| (value - max).exp()).sum();
    if total <= 0.0 {
        return 0.0;
    }
    logits.get(index).map_or(0.0, |value| (value - max).exp() / total)
}

/// Frame index to seconds. Subsampling factor 8 at hop 160 on 16 kHz audio, so
/// one encoder frame is 80 ms.
pub fn frame_to_seconds(frame: usize) -> f32 {
    frame as f32 * 8.0 * 160.0 / SAMPLE_RATE as f32
}

pub fn run_model_smoke(model_dir: &Path, wav: Option<&Path>) -> Result<Decoded, String> {
    let mut model = LoadedParakeet::load(model_dir)?;
    let audio = match wav {
        Some(path) => {
            let (samples, rate) = crate::audio::read_wav_mono(path)?;
            crate::audio::resample_to_16k(&samples, rate)
        }
        None => (0..SAMPLE_RATE as usize * 4)
            .map(|index| {
                (2.0 * std::f32::consts::PI * 440.0 * index as f32 / SAMPLE_RATE as f32).sin() * 0.1
            })
            .collect(),
    };
    model.transcribe(&audio)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vocab_from(pairs: &[(&str, usize)], blank: usize) -> Vocabulary {
        let max = pairs.iter().map(|(_, id)| *id).max().unwrap_or(0).max(blank);
        let mut pieces = vec![String::new(); max + 1];
        for (token, id) in pairs {
            pieces[*id] = token.replace('\u{2581}', " ");
        }
        Vocabulary {
            pieces,
            blank_id: blank as u32,
        }
    }

    #[test]
    fn softmax_is_stable_against_large_logits() {
        let logits = [1000.0f32, 1000.0, 1000.0];
        let probability = softmax_at(&logits, 0);
        assert!((probability - 1.0 / 3.0).abs() < 1e-6, "got {probability}");
    }

    #[test]
    fn softmax_picks_out_the_dominant_logit() {
        let logits = [0.0f32, 10.0, 0.0];
        assert!(softmax_at(&logits, 1) > 0.99);
        assert!(softmax_at(&logits, 0) < 0.01);
    }

    #[test]
    fn argmax_breaks_ties_toward_the_first() {
        assert_eq!(argmax(&[1.0, 5.0, 5.0, 2.0]), 1);
        assert_eq!(argmax(&[]), 0);
    }

    /// One encoder frame is 80 ms: subsampling 8, hop 160, 16 kHz.
    #[test]
    fn frames_convert_to_eighty_millisecond_steps() {
        assert!((frame_to_seconds(0) - 0.0).abs() < 1e-6);
        assert!((frame_to_seconds(1) - 0.08).abs() < 1e-6);
        assert!((frame_to_seconds(25) - 2.0).abs() < 1e-6);
    }

    /// SentencePiece marks a word start with `▁`, rendered as a leading space.
    #[test]
    fn word_confidence_takes_the_weakest_sub_token() {
        let vocab = vocab_from(&[("\u{2581}hel", 1), ("lo", 2), ("\u{2581}there", 3)], 0);
        let decoded = Decoded {
            text: String::new(),
            tokens: vec![
                TimedToken { id: 1, frame: 0, probability: 0.9 },
                TimedToken { id: 2, frame: 1, probability: 0.4 },
                TimedToken { id: 3, frame: 2, probability: 0.8 },
            ],
        };
        let words = decoded.word_confidences(&vocab);
        assert_eq!(words.len(), 2);
        assert_eq!(words[0].0, "hello");
        // 0.4, not the 0.9 of the confident first piece and not the mean.
        assert!((words[0].1 - 0.4).abs() < 1e-6);
        assert_eq!(words[1].0, "there");
    }

    /// A sentence-final period is genuinely uncertain and scores low. Letting
    /// it set the word's confidence would mark the last word of every sentence
    /// as doubtful, which is exactly the word a naive gate would then corrupt.
    #[test]
    fn trailing_punctuation_does_not_drag_down_word_confidence() {
        let vocab = vocab_from(&[("\u{2581}phenomenal", 1), (".", 2)], 0);
        let decoded = Decoded {
            text: String::new(),
            tokens: vec![
                TimedToken { id: 1, frame: 0, probability: 0.97 },
                TimedToken { id: 2, frame: 1, probability: 0.31 },
            ],
        };
        let words = decoded.word_confidences(&vocab);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0].0, "phenomenal.");
        assert!((words[0].1 - 0.97).abs() < 1e-6, "got {}", words[0].1);
    }

    #[test]
    fn a_punctuation_only_word_is_reported_as_certain() {
        let vocab = vocab_from(&[("\u{2581}...", 1)], 0);
        let decoded = Decoded {
            text: String::new(),
            tokens: vec![TimedToken { id: 1, frame: 0, probability: 0.2 }],
        };
        let words = decoded.word_confidences(&vocab);
        assert_eq!(words, vec![("...".to_string(), 1.0)]);
    }

    #[test]
    fn punctuation_detection_covers_the_real_pieces() {
        assert!(is_punctuation_only("."));
        assert!(is_punctuation_only("?"));
        assert!(is_punctuation_only(" ,"));
        assert!(!is_punctuation_only("home"));
        assert!(!is_punctuation_only("2nd"));
        assert!(!is_punctuation_only(""));
    }

    #[test]
    fn pieces_join_into_text_with_sentencepiece_spacing() {
        let vocab = vocab_from(&[("\u{2581}go", 1), ("od", 2), ("\u{2581}day", 3)], 0);
        let tokens: Vec<TimedToken> = [1u32, 2, 3]
            .iter()
            .enumerate()
            .map(|(frame, id)| TimedToken { id: *id, frame, probability: 1.0 })
            .collect();
        assert_eq!(vocab.decode(&tokens), "good day");
    }

    /// `vocab.txt` splits on the *last* space: a token may itself be a space.
    #[test]
    fn vocab_lines_split_from_the_right() {
        let dir = std::env::temp_dir().join(format!("transcrust-vocab-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("vocab.txt");
        std::fs::write(&path, "\u{2581}a 0\n<blk> 1\n").unwrap();
        let vocab = Vocabulary::load(&path).unwrap();
        assert_eq!(vocab.blank_id(), 1);
        assert_eq!(vocab.piece(0), " a");
        std::fs::remove_dir_all(&dir).ok();
    }
}
