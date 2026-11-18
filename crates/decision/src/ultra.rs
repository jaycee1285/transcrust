//! Native UltraBERT/GLiClass; inference shared with the research harness.
//! Reference attribution: tools/schema-probe/NOTICE and LICENSE (Apache-2.0).
use super::decision_tensor::*;
use candle_core::{Device, Tensor};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use tokenizers::Tokenizer;
pub struct Ultra {
    w: Weights,
    tok: Tokenizer,
    pool: rayon::ThreadPool,
}
impl Ultra {
    pub fn load(dir: &Path) -> Result<Self, Error> {
        let mut tok =
            Tokenizer::from_file(dir.join("tokenizer.json")).map_err(|e| e.to_string())?;
        tok.with_truncation(None).map_err(|e| e.to_string())?;
        tok.with_padding(None);
        Ok(Self {
            w: if dir.join("ultra-q4.json").exists() {
                Weights::ultra_q4(dir)?
            } else {
                Weights::safetensors(dir)?
            },
            tok,
            pool: rayon::ThreadPoolBuilder::new()
                .num_threads(std::thread::available_parallelism().map_or(2, |n| n.get().min(4)))
                .build()?,
        })
    }
    pub fn score(
        &self,
        state: &str,
        instructions: &str,
        labels: &[String],
    ) -> Result<Vec<f64>, Error> {
        Ok(self
            .batch(&[(state.to_string(), instructions.to_string(), labels.to_vec())])?
            .remove(0))
    }
    pub fn batch(
        &self,
        requests: &[(String, String, Vec<String>)],
    ) -> Result<Vec<Vec<f64>>, Error> {
        self.batch_cancellable(requests, &AtomicBool::new(false))
    }
    pub fn batch_cancellable(
        &self,
        requests: &[(String, String, Vec<String>)],
        cancel: &AtomicBool,
    ) -> Result<Vec<Vec<f64>>, Error> {
        self.pool.install(|| self.batch_inner(requests, cancel))
    }
    pub fn tokens(&self, text: &str, prompt: &str, labels: &[String]) -> Result<Vec<u32>, Error> {
        if ["<<LABEL>>", "<<SEP>>", "<<EXAMPLE>>"]
            .iter()
            .any(|marker| {
                text.contains(marker)
                    || prompt.contains(marker)
                    || labels.iter().any(|s| s.contains(marker))
            })
        {
            return Err("reserved UltraBERT marker in input".into());
        }
        let input = format!(
            "{}<<SEP>>{prompt}{text}",
            labels
                .iter()
                .map(|s| format!("<<LABEL>>{s}"))
                .collect::<String>()
        );
        let row = self.tok.encode(input, true).map_err(|e| e.to_string())?;
        if row.len() > 4096 {
            return Err("UltraBERT truncation refused".into());
        }
        Ok(row.get_ids().to_vec())
    }
    fn batch_inner(
        &self,
        requests: &[(String, String, Vec<String>)],
        cancel: &AtomicBool,
    ) -> Result<Vec<Vec<f64>>, Error> {
        let check = || -> Result<(), Error> {
            if cancel.load(Ordering::Relaxed) {
                Err("UltraBERT inference cancelled".into())
            } else {
                Ok(())
            }
        };
        check()?;
        if requests.is_empty() {
            return Ok(Vec::new());
        }
        let mut ids = Vec::new();
        let mut spans = Vec::new();
        let mut markers = Vec::new();
        for (text, prompt, labels) in requests {
            let row = self.tokens(text, prompt, labels)?;
            let start = ids.len();
            markers.push(
                row.iter()
                    .enumerate()
                    .filter_map(|(i, id)| (*id == 49152).then_some(start + i))
                    .collect::<Vec<_>>(),
            );
            assert_eq!(markers.last().unwrap().len(), labels.len());
            ids.extend_from_slice(&row);
            spans.push(start..ids.len());
        }
        if ids.len() > 8192 {
            return Err("UltraBERT batch exceeds 8192-token budget".into());
        }
        let mut x = self.w.ultra_embed(&ids)?;
        for layer in 0..24 {
            check()?;
            let p = format!("model.encoder_model.layers.{layer}");
            let norm = self.w.rms(&x, &format!("{p}.input_layernorm.weight"))?;
            let q = self
                .w
                .linear(&norm, &format!("{p}.self_attn.q_proj.weight"), None)?;
            let k = self
                .w
                .linear(&norm, &format!("{p}.self_attn.k_proj.weight"), None)?;
            let v = self
                .w
                .linear(&norm, &format!("{p}.self_attn.v_proj.weight"), None)?;
            let full = layer % 3 == 2;
            let a = attention(
                &q,
                &k,
                &v,
                &spans,
                8,
                8,
                128,
                if full { 2560000. } else { 10000. },
                if full { None } else { Some(128) },
                1. / 128f64.sqrt(),
            )?;
            x = (x + self
                .w
                .linear(&a, &format!("{p}.self_attn.o_proj.weight"), None)?)?;
            let n = self
                .w
                .rms(&x, &format!("{p}.post_attention_layernorm.weight"))?;
            let gate = self
                .w
                .linear(&n, &format!("{p}.mlp.gate_proj.weight"), None)?;
            let up = self
                .w
                .linear(&n, &format!("{p}.mlp.up_proj.weight"), None)?;
            let silu = gate.broadcast_div(&(gate.neg()?.exp()? + 1.)?)?;
            x = (x + self
                .w
                .linear(&(silu * up)?, &format!("{p}.mlp.down_proj.weight"), None)?)?;
        }
        check()?;
        x = self.w.rms(&x, "model.encoder_model.norm.weight")?;
        let mut outputs = Vec::new();
        for (span, positions) in spans.iter().zip(markers) {
            let state = x.narrow(0, span.start, 1)?;
            let classes = x.index_select(
                &Tensor::new(
                    positions
                        .iter()
                        .map(|i| *i as u32)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &Device::Cpu,
                )?,
                0,
            )?;
            let project = |t: &Tensor, p: &str| -> Result<Tensor, Error> {
                self.w.affine(
                    &self.w.affine(t, &format!("{p}.linear_1"))?.gelu_erf()?,
                    &format!("{p}.linear_2"),
                )
            };
            let s =
                project(&state, "model.text_projector")?.broadcast_as((positions.len(), 1024))?;
            let c = project(&classes, "model.classes_projector")?;
            let pair = Tensor::cat(&[s, c], 1)?;
            let h = self.w.affine(&pair, "model.scorer.mlp.0")?.relu()?;
            let h = self.w.affine(&h, "model.scorer.mlp.2")?.relu()?;
            let logits = self
                .w
                .affine(&h, "model.scorer.mlp.4")?
                .flatten_all()?
                .to_vec1::<f32>()?;
            if !logits.iter().all(|x| x.is_finite()) {
                return Err("non-finite UltraBERT logits".into());
            }
            outputs.push(softmax(&logits, 1.));
        }
        Ok(outputs)
    }
}
