//! Moonshine: a small English encoder-decoder ASR, driven through `ort`.
//!
//! Added to measure, not yet to ship. The question it answers is cold **load**
//! time, which is what a tap-to-talk surface actually pays: a Quick Settings
//! tile is cold every time, so a model that loads in 150 ms and decodes slowly
//! beats one that loads in 1.2 s and decodes fast.
//!
//! Three properties make it worth the measurement:
//!
//! * **It pads to a multiple of 80 samples — 5 ms.** Granite pads to 10.24 s
//!   blocks, so a 1.5 s command there wastes 8.7 s of encoder. That quantum is
//!   what disqualified Granite for commands; this is the same axis, two
//!   thousand times finer.
//! * **No mel frontend.** `feature_size: 1`, `do_normalize: false` — raw 16 kHz
//!   waveform straight in, convolutional frontend inside the encoder. Neither
//!   `granite.rs`'s hand-written mel nor Parakeet's `nemo128.onnx` is needed.
//! * **English-only, 71 MB encoder** against Parakeet's 390 MB int4 encoder.
//!
//! What it gives up is the thing `Parakeet-v3.md` §3 is about: it is
//! autoregressive, so unlike a transducer it *can* hallucinate into silence.
//! Hence [`MAX_NEW_TOKENS`].

use std::path::Path;

use ndarray::{Array2, Array3, Array4};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::TensorRef;
use tokenizers::Tokenizer;

/// The encoder's conv frontend reshapes by 80, so a ragged tail would change
/// the frame count. The preprocessor config states this as `pad_to_multiple_of`.
const PAD_MULTIPLE: usize = 80;
/// From `config.json`: decoder_start_token_id.
const DECODER_START: i64 = 1;
/// From `generation_config.json`: eos_token_id.
const EOS: i64 = 2;
/// Encoder hidden width, per the graph's declared output shape.
const HIDDEN: usize = 620;
/// KV head count and per-head width, from the `past_key_values` shapes.
const KV_HEADS: usize = 8;
const KV_DIM: usize = 64;
/// Decoder layers, from the 40 present/past tensors (4 per layer).
const LAYERS: usize = 10;
/// A hard ceiling on generation.
///
/// An autoregressive decoder handed silence will loop happily, which a
/// transducer structurally cannot do. Moonshine's own card says to bound the
/// output for exactly this reason. Sized for commands and short dictation, not
/// long-form.
const MAX_NEW_TOKENS: usize = 256;

pub struct LoadedMoonshine {
    encoder: Session,
    decoder: Session,
    decoder_with_past: Session,
    tokenizer: Tokenizer,
    /// Exports disagree: Mazino0 declares `attention_mask` on the encoder,
    /// Workmind does not. Asking the graph is cheaper than a config flag.
    encoder_wants_mask: bool,
}

impl LoadedMoonshine {
    /// Expects the Workmind ONNX layout: `onnx/encoder_model*.onnx`,
    /// `onnx/decoder_model_merged*.onnx`, and `tokenizer.json` at the root.
    pub fn load(dir: &Path) -> Result<Self, String> {
        let onnx = dir.join("onnx");
        let encoder_path = pick(&onnx, "encoder_model")?;
        let decoder_path = pick(&onnx, "decoder_model")?;
        let with_past_path = pick(&onnx, "decoder_with_past_model")?;
        let tokenizer_path = dir.join("tokenizer.json");

        let encoder = open(&encoder_path)?;
        let decoder = open(&decoder_path)?;
        let decoder_with_past = open(&with_past_path)?;
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| format!("failed to load {}: {e}", tokenizer_path.display()))?;

        Ok(Self {
            encoder_wants_mask: encoder
                .inputs()
                .iter()
                .any(|input| input.name() == "attention_mask"),
            encoder,
            decoder,
            decoder_with_past,
            tokenizer,
        })
    }

    /// Greedy decode. `audio` must already be 16 kHz mono.
    pub fn transcribe(&mut self, audio: &[f32]) -> Result<String, String> {
        let encoded = self.encode(audio)?;
        let tokens = self.decode_greedy(&encoded)?;
        self.tokenizer
            .decode(&tokens, true)
            .map_err(|e| format!("Moonshine detokenise failed: {e}"))
    }

    fn encode(&mut self, audio: &[f32]) -> Result<Array3<f32>, String> {
        // Zero-pad rather than truncate: the frontend reshapes by 80 and a
        // ragged tail silently changes the frame count.
        let mut padded = audio.to_vec();
        let remainder = padded.len() % PAD_MULTIPLE;
        if remainder != 0 {
            padded.resize(padded.len() + (PAD_MULTIPLE - remainder), 0.0);
        }
        let samples = padded.len();
        let values = Array2::from_shape_vec((1, samples), padded)
            .map_err(|e| format!("failed to shape Moonshine waveform: {e}"))?;

        let mask = Array2::<i64>::ones((1, samples));
        let mut inputs = ort::inputs![
            "input_values" => TensorRef::from_array_view(&values)
                .map_err(|e| format!("failed to build Moonshine input tensor: {e}"))?,
        ];
        if self.encoder_wants_mask {
            inputs.push((
                "attention_mask".into(),
                TensorRef::from_array_view(&mask)
                    .map_err(|e| format!("failed to build attention mask: {e}"))?
                    .into(),
            ));
        }
        let outputs = self
            .encoder
            .run(inputs)
            .map_err(|e| format!("Moonshine encoder failed: {e}"))?;

        // Workmind names it `last_hidden_state`, Mazino0 `encoder_hidden_states`.
        let hidden = ["last_hidden_state", "encoder_hidden_states"]
            .into_iter()
            .find(|name| outputs.get(*name).is_some())
            .ok_or("Moonshine encoder produced no recognised hidden-state output")?;
        let (shape, data) = outputs[hidden]
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Moonshine encodings were not a float tensor: {e}"))?;
        if shape.len() != 3 || shape[2] as usize != HIDDEN {
            return Err(format!("unexpected Moonshine encoder shape: {shape:?}"));
        }
        Array3::from_shape_vec((1, shape[1] as usize, HIDDEN), data.to_vec())
            .map_err(|e| format!("failed to shape Moonshine encodings: {e}"))
    }

    /// Greedy decode across the split decoder pair.
    ///
    /// Two graphs rather than one merged graph with an `If` node, and that is
    /// the whole reason this export is used: the merged variant fuses across
    /// its own branch under ORT and dies on the first token with
    /// "right operand cannot broadcast on dim 0". Its author validated it in
    /// transformers.js, which takes a different path through the same file.
    ///
    /// Cross-attention KV is computed once by the first graph and handed back
    /// unchanged on every later step; only the self-attention cache grows.
    fn decode_greedy(&mut self, encoded: &Array3<f32>) -> Result<Vec<u32>, String> {
        let mut tokens: Vec<u32> = Vec::new();
        let mut next = DECODER_START;

        let ids = Array2::from_shape_vec((1, 1), vec![next])
            .map_err(|e| format!("failed to shape Moonshine input ids: {e}"))?;
        let first = self
            .decoder
            .run(ort::inputs![
                "decoder_input_ids" => TensorRef::from_array_view(&ids)
                    .map_err(|e| format!("failed to build id tensor: {e}"))?,
                "encoder_hidden_states" => TensorRef::from_array_view(encoded)
                    .map_err(|e| format!("failed to build encoder-state tensor: {e}"))?,
            ])
            .map_err(|e| format!("Moonshine first decode step failed: {e}"))?;

        next = argmax(&first, "logits")?;
        if next != EOS {
            tokens.push(next as u32);
        }
        let mut self_kv = collect(&first, "present_self_", "")?;
        let cross_kv = collect(&first, "present_cross_", "")?;

        while next != EOS && tokens.len() < MAX_NEW_TOKENS {
            let ids = Array2::from_shape_vec((1, 1), vec![next])
                .map_err(|e| format!("failed to shape Moonshine input ids: {e}"))?;
            let mut inputs = ort::inputs![
                "decoder_input_ids" => TensorRef::from_array_view(&ids)
                    .map_err(|e| format!("failed to build id tensor: {e}"))?,
                "encoder_hidden_states" => TensorRef::from_array_view(encoded)
                    .map_err(|e| format!("failed to build encoder-state tensor: {e}"))?,
            ];
            for (index, value) in self_kv.iter().enumerate() {
                inputs.push((
                    kv_name("past_self_", index, "").into(),
                    TensorRef::from_array_view(value)
                        .map_err(|e| format!("failed to build past-self tensor: {e}"))?
                        .into(),
                ));
            }
            for (index, value) in cross_kv.iter().enumerate() {
                inputs.push((
                    kv_name("present_cross_", index, "_orig").into(),
                    TensorRef::from_array_view(value)
                        .map_err(|e| format!("failed to build cross tensor: {e}"))?
                        .into(),
                ));
            }

            let step = self
                .decoder_with_past
                .run(inputs)
                .map_err(|e| format!("Moonshine decode step failed: {e}"))?;

            next = argmax(&step, "logits")?;
            if next == EOS {
                break;
            }
            tokens.push(next as u32);
            self_kv = collect(&step, "present_self_", "")?;
        }

        Ok(tokens)
    }
}

/// Highest-scoring token id at the final position.
fn argmax(outputs: &ort::session::SessionOutputs, name: &str) -> Result<i64, String> {
    let (shape, logits) = outputs[name]
        .try_extract_tensor::<f32>()
        .map_err(|e| format!("Moonshine {name} were not a float tensor: {e}"))?;
    let vocab = *shape.last().unwrap_or(&0) as usize;
    if vocab == 0 {
        return Err("Moonshine returned an empty vocabulary axis".into());
    }
    Ok(logits[logits.len() - vocab..]
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(index, _)| index as i64)
        .unwrap_or(EOS))
}

/// Gather the 20 key/value tensors for a prefix, key before value per layer.
fn collect(
    outputs: &ort::session::SessionOutputs,
    prefix: &str,
    suffix: &str,
) -> Result<Vec<Array4<f32>>, String> {
    let mut out = Vec::with_capacity(LAYERS * 2);
    for index in 0..LAYERS * 2 {
        let name = kv_name(prefix, index, suffix);
        let (shape, data) = outputs[name.as_str()]
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Moonshine {name} was not a float tensor: {e}"))?;
        out.push(
            Array4::from_shape_vec(
                (
                    shape[0] as usize,
                    shape[1] as usize,
                    shape[2] as usize,
                    shape[3] as usize,
                ),
                data.to_vec(),
            )
            .map_err(|e| format!("failed to shape {name}: {e}"))?,
        );
    }
    Ok(out)
}

/// `<prefix>{key|value}_{layer}{suffix}` — the naming this export uses, with
/// key and value alternating per layer.
fn kv_name(prefix: &str, index: usize, suffix: &str) -> String {
    let layer = index / 2;
    let part = if index % 2 == 0 { "key" } else { "value" };
    format!("{prefix}{part}_{layer}{suffix}")
}

/// Prefer the quantised graph, then q4, then fp32 — smallest first, because
/// load time is the property being measured.
fn pick(dir: &Path, stem: &str) -> Result<std::path::PathBuf, String> {
    // Quantised first, because load time is the property being measured.
    // Exports disagree on the spelling: Workmind writes `_quantized`, Mazino0
    // writes `_int8`.
    for suffix in ["_quantized.onnx", "_int8.onnx", "_q4.onnx", ".onnx"] {
        let candidate = dir.join(format!("{stem}{suffix}"));
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(format!("no {stem}*.onnx under {}", dir.display()))
}

fn open(path: &Path) -> Result<Session, String> {
    Session::builder()
        .map_err(|e| format!("failed to create ORT session builder: {e}"))?
        .commit_from_file(path)
        .map_err(|e| format!("failed to open {}: {e}", path.display()))
}

/// The merged decoder has to be opened with fusion off.
///
/// At the default optimisation level ORT fuses across the `optimum::if` node —
/// the failing kernel names itself
/// `encoder_attn/MatMul/MatMulScaleFusion//MatmulTransposeFusion/` — and the
/// fused matmul then runs against the branch that was not taken, producing
/// "right operand cannot broadcast on dim 0" on the very first token. The
/// encoder has no `If` and is unaffected, so only this graph pays the cost.
fn open_unfused(path: &Path) -> Result<Session, String> {
    Session::builder()
        .map_err(|e| format!("failed to create ORT session builder: {e}"))?
        .with_optimization_level(GraphOptimizationLevel::Disable)
        .map_err(|e| format!("failed to set optimisation level: {e}"))?
        .commit_from_file(path)
        .map_err(|e| format!("failed to open {}: {e}", path.display()))
}
