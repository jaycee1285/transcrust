//! s1-mini: a 0.6 B text normaliser, driven through `ort`.
//!
//! Takes a raw ASR transcript and returns written English — punctuation, casing,
//! contractions, spoken numbers rendered as digits, false starts resolved to
//! whatever the speaker landed on. Text in, text out; it never touches audio.
//!
//! **Why it is worth a second model.** Granite's CTC head emits bare lowercase
//! with no punctuation, so a long `--wav` transcript arrives as one run-on per
//! window. `mode::Profile::Long` repairs the mechanical half deterministically
//! — 13.71% to 10.11% WER, measured — and then stops at the possessive, because
//! `today is` to `today's` is not safely reversible. That is the seam this
//! crosses.
//!
//! **Order matters, and it was settled by measurement.** Fed *raw* Granite,
//! s1-mini invents: it turned `but what if you are would not it be nice` into a
//! clause that was never spoken. Fed the same text after `Profile::Long`, it
//! left the awkward stretch verbatim and invented nothing. So the deterministic
//! pass is a prerequisite rather than something this supersedes — it repairs the
//! damage so the model is not guessing. This runs *after* the rules, which is
//! the opposite of where Harper sat when it was removed for starving the
//! phonetic dictionary.
//!
//! ONNX rather than the GGUF build because `ort` is already here, and because
//! `voxlin` runs `onnxruntime-android` — this is the format that ports to the
//! phone.

use std::path::Path;

use ndarray::{Array0, Array2, Array4};
use ort::session::Session;
use ort::value::TensorRef;
use tokenizers::Tokenizer;

/// From `config.json` / the graph's `past_key_values` shapes.
const LAYERS: usize = 28;
const KV_HEADS: usize = 8;
const KV_DIM: usize = 128;

/// `generation_config.json` lists both; either ends generation.
const EOS: [i64; 2] = [151_645, 151_643];

/// A ceiling on generation, since an autoregressive model handed odd input can
/// loop. Sized for a `--wav` window or a long dictation, not a book.
const MAX_NEW_TOKENS: usize = 2048;

/// How much rewriting to do. `Styling` is the contraction decision — the axis
/// `Profile::Long` cannot reach — and `Structure` is advisory in practice:
/// measured on a real transcript, `prose` still produced lists where the content
/// was clearly enumerable.
#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub styling: &'static str,
    pub structure: &'static str,
    pub context: &'static str,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            styling: "semi-formal",
            structure: "prose",
            context: "general",
        }
    }
}

pub struct Normaliser {
    session: Session,
    tokenizer: Tokenizer,
}

impl Normaliser {
    /// Expects the onnx-community layout: `onnx/model*.onnx` with its
    /// `*.onnx_data` sidecar beside it, and `tokenizer.json` at the root.
    pub fn load(dir: &Path) -> Result<Self, String> {
        let onnx = dir.join("onnx");
        let model = ["model_q4.onnx", "model_quantized.onnx", "model.onnx"]
            .into_iter()
            .map(|name| onnx.join(name))
            .find(|path| path.is_file())
            .ok_or_else(|| format!("no model*.onnx under {}", onnx.display()))?;

        // Must be `commit_from_file`: the weights live in a sidecar that ORT
        // resolves relative to the model path, so an in-memory load cannot find
        // them.
        let session = Session::builder()
            .map_err(|e| format!("failed to create ORT session builder: {e}"))?
            .commit_from_file(&model)
            .map_err(|e| format!("failed to open {}: {e}", model.display()))?;

        let tokenizer_path = dir.join("tokenizer.json");
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| format!("failed to load {}: {e}", tokenizer_path.display()))?;

        Ok(Self { session, tokenizer })
    }

    pub fn normalise(&mut self, transcript: &str, style: Style) -> Result<String, String> {
        if transcript.trim().is_empty() {
            return Ok(String::new());
        }
        let prompt = build_prompt(transcript, style);
        let encoded = self
            .tokenizer
            .encode(prompt, false)
            .map_err(|e| format!("s1-mini tokenise failed: {e}"))?;
        let mut ids: Vec<i64> = encoded.get_ids().iter().map(|id| *id as i64).collect();

        // Keep the batch axis on the cache so no per-step temporary is needed;
        // a borrowed view has to outlive the input list it is pushed into.
        let mut past: Vec<Array4<f32>> = (0..LAYERS * 2)
            .map(|_| Array4::<f32>::zeros((1, KV_HEADS, 0, KV_DIM)))
            .collect();
        let mut generated: Vec<u32> = Vec::new();
        let mut step_ids = ids.clone();
        let mut total = ids.len();

        for _ in 0..MAX_NEW_TOKENS {
            let step_len = step_ids.len();
            let input = Array2::from_shape_vec((1, step_len), step_ids.clone())
                .map_err(|e| format!("failed to shape input ids: {e}"))?;
            // The mask covers the whole sequence so far, not just this step.
            let mask = Array2::<i64>::ones((1, total));
            // Only the final position's logits are needed, and the vocabulary is
            // 151936 wide — asking for all of them on a long prompt is the
            // difference between a fast first token and a slow one.
            // Declared `[]` in the graph: a genuine scalar, not a length-1
            // vector. A 1-D value makes the lm_head Slice compute a malformed
            // `starts` and fail with "Starts must be a 1-D array".
            let keep = Array0::from_elem((), 1i64);

            let mut inputs = ort::inputs![
                "input_ids" => TensorRef::from_array_view(&input)
                    .map_err(|e| format!("failed to build id tensor: {e}"))?,
                "attention_mask" => TensorRef::from_array_view(&mask)
                    .map_err(|e| format!("failed to build mask tensor: {e}"))?,
                "num_logits_to_keep" => TensorRef::from_array_view(&keep)
                    .map_err(|e| format!("failed to build logit-count tensor: {e}"))?,
            ];
            for (index, value) in past.iter().enumerate() {
                inputs.push((
                    kv_name("past_key_values", index).into(),
                    TensorRef::from_array_view(value)
                        .map_err(|e| format!("failed to build past-kv tensor: {e}"))?
                        .into(),
                ));
            }

            let outputs = self
                .session
                .run(inputs)
                .map_err(|e| format!("s1-mini inference failed: {e}"))?;

            let (shape, logits) = outputs["logits"]
                .try_extract_tensor::<f32>()
                .map_err(|e| format!("s1-mini logits were not a float tensor: {e}"))?;
            let vocab = *shape.last().unwrap_or(&0) as usize;
            if vocab == 0 {
                return Err("s1-mini returned an empty vocabulary axis".into());
            }
            let next = logits[logits.len() - vocab..]
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .map(|(index, _)| index as i64)
                .unwrap_or(EOS[0]);

            if EOS.contains(&next) {
                break;
            }
            generated.push(next as u32);

            let mut fresh = Vec::with_capacity(past.len());
            for index in 0..past.len() {
                let (shape, data) = outputs[kv_name("present", index).as_str()]
                    .try_extract_tensor::<f32>()
                    .map_err(|e| format!("s1-mini present-kv was not a float tensor: {e}"))?;
                fresh.push(
                    Array4::from_shape_vec(
                        (
                            shape[0] as usize,
                            shape[1] as usize,
                            shape[2] as usize,
                            shape[3] as usize,
                        ),
                        data.to_vec(),
                    )
                    .map_err(|e| format!("failed to shape present kv: {e}"))?,
                );
            }
            past = fresh;

            total += 1;
            step_ids = vec![next];
            ids.push(next);
        }

        self.tokenizer
            .decode(&generated, true)
            .map_err(|e| format!("s1-mini detokenise failed: {e}"))
    }
}

/// The chat template, written out rather than rendered.
///
/// The README specifies this exact string, including the empty `<think>` block —
/// that is what `enable_thinking: false` produces, and it is the prefix the
/// model saw in training. Writing it literally avoids carrying a Jinja engine to
/// render four lines.
fn build_prompt(transcript: &str, style: Style) -> String {
    format!(
        "<|im_start|>system\nYou are a text normalizer for speech-to-text \
         transcripts. The input begins with a control line specifying the \
         styling, structure, and context settings; clean the transcript to match \
         those settings and output only the cleaned text.<|im_end|>\n\
         <|im_start|>user\n[Styling: {}] [Structure: {}] [Context: {}]\n\
         {transcript}<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n",
        style.styling, style.structure, style.context
    )
}

/// `past_key_values.{layer}.{key|value}`, key before value per layer.
fn kv_name(prefix: &str, index: usize) -> String {
    let layer = index / 2;
    let part = if index % 2 == 0 { "key" } else { "value" };
    format!("{prefix}.{layer}.{part}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_matches_the_documented_template() {
        let prompt = build_prompt("hello there", Style::default());
        assert!(prompt.starts_with("<|im_start|>system\nYou are a text normalizer"));
        assert!(prompt.contains("[Styling: semi-formal] [Structure: prose] [Context: general]\nhello there<|im_end|>"));
        // The empty think block is what `enable_thinking: false` emits, and the
        // model was trained with it present.
        assert!(prompt.ends_with("<|im_start|>assistant\n<think>\n\n</think>\n\n"));
    }

    #[test]
    fn kv_names_alternate_key_then_value() {
        assert_eq!(kv_name("past_key_values", 0), "past_key_values.0.key");
        assert_eq!(kv_name("past_key_values", 1), "past_key_values.0.value");
        assert_eq!(kv_name("present", 2), "present.1.key");
        assert_eq!(kv_name("present", 55), "present.27.value");
    }
}
