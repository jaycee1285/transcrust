use super::common::*;
use candle_core::{Device, Tensor, D};
use std::path::Path;
use tokenizers::Tokenizer;
pub struct Eg2 {
    pub w: Weights,
    tok: Tokenizer,
}
impl Eg2 {
    pub fn load(dir: &Path) -> Result<Self, Error> {
        let mut tok =
            Tokenizer::from_file(dir.join("tokenizer.json")).map_err(|e| e.to_string())?;
        tok.with_truncation(None).map_err(|e| e.to_string())?;
        tok.with_padding(None);
        Ok(Self {
            w: Weights::packed(dir)?,
            tok,
        })
    }
    pub fn tokens(&self, text: &str) -> Result<Vec<u32>, Error> {
        let mut ids = vec![2];
        ids.extend_from_slice(
            self.tok
                .encode(format!("task: classification | query: {text}"), false)
                .map_err(|e| e.to_string())?
                .get_ids(),
        );
        ids.push(1);
        if ids.len() > 8192 {
            return Err("EmbeddingGemma truncation refused".into());
        }
        Ok(ids)
    }
    pub fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Error> {
        let mut ids = Vec::new();
        let mut spans = Vec::new();
        for text in texts {
            let row = self.tokens(text)?;
            let start = ids.len();
            ids.extend(row);
            spans.push(start..ids.len());
        }
        if ids.len() > 8192 {
            return Err("EmbeddingGemma packed token budget exceeded".into());
        }
        let w = &self.w;
        let mut x = w.embed_packed(&ids)?;
        let original = x.clone();
        let mut norm = w.rms(&x, "layers.0.input_layernorm")?;
        for layer in 0..24 {
            let p = format!("layers.{layer}.");
            let full = layer % 6 == 5;
            let d = if full { 512 } else { 256 };
            let kv = if full { 1 } else { 2 };
            let qkv = w.linear(&norm, &format!("{p}qkv"), None)?;
            let q = qkv.narrow(1, 0, 4 * d)?.reshape((ids.len(), 4, d))?;
            let k = qkv.narrow(1, 4 * d, kv * d)?.reshape((ids.len(), kv, d))?;
            let v = qkv
                .narrow(1, (4 + kv) * d, kv * d)?
                .reshape((ids.len(), kv, d))?;
            let q = w
                .rms(&q, &format!("{p}q_norm"))?
                .reshape((ids.len(), 4 * d))?;
            let k = w
                .rms(&k, &format!("{p}k_norm"))?
                .reshape((ids.len(), kv * d))?;
            let v = unit_rms(&v)?.reshape((ids.len(), kv * d))?;
            let a = attention(
                &q,
                &k,
                &v,
                &spans,
                4,
                kv,
                d,
                if full { 1000000. } else { 10000. },
                if full { None } else { Some(512) },
                1.,
            )?;
            x = (&x
                + w.rms(
                    &w.linear(&a, &format!("{p}o"), None)?,
                    &format!("{p}post_attention_layernorm"),
                )?)?;
            norm = w.rms(&x, &format!("{p}pre_feedforward_layernorm"))?;
            let gu = w.linear(&norm, &format!("{p}gu"), None)?;
            let mix = (gelu(&gu.narrow(1, 0, 2048)?)? * gu.narrow(1, 2048, 2048)?)?;
            x = (&x
                + w.rms(
                    &w.linear(&mix, &format!("{p}down"), None)?,
                    &format!("{p}post_feedforward_layernorm"),
                )?)?;
            let gate = gelu(&w.linear(&x, &format!("{p}ple_gate"), None)?)?;
            let e = w.rms(
                &w.linear(&original, &format!("{p}ple_in"), None)?,
                "ple.norm",
            )?;
            x =
                ((&x + w.rms(
                    &w.linear(&(gate * e)?, &format!("{p}ple_proj"), None)?,
                    &format!("{p}ple_norm"),
                )?)? * w.manifest["config"]["layer_scalars"][layer]
                    .as_f64()
                    .unwrap())?;
            norm = w.rms(
                &x,
                &if layer == 23 {
                    "final_norm".into()
                } else {
                    format!("layers.{}.input_layernorm", layer + 1)
                },
            )?;
        }
        let mut result = Vec::new();
        for span in spans {
            let mean = norm.narrow(0, span.start, span.len())?.mean_keepdim(0)?;
            let embedding = l2(&w.linear(&mean, "projection", None)?)?;
            result.push(embedding.flatten_all()?.to_vec1::<f32>()?);
        }
        Ok(result)
    }
    pub fn logits(
        &self,
        s: &[f32],
        q: &[f32],
        options: &[Vec<f32>],
        kind: &str,
    ) -> Result<Vec<f64>, Error> {
        let w = &self.w;
        let cfg = &w.manifest["config"];
        let t = |v: &[f32]| Tensor::from_slice(v, (1, v.len()), &Device::Cpu);
        let s0 = t(s)?;
        let q0 = t(q)?;
        let o = Tensor::from_vec(
            options.iter().flatten().copied().collect::<Vec<_>>(),
            (options.len(), 768),
            &Device::Cpu,
        )?;
        let s = w.affine(&s0, "head.state_proj")?;
        let q = w.affine(&q0, "head.question_proj")?;
        let o_proj = w.affine(&o, "head.option_proj")?;
        let field = Tensor::cat(&[s.clone(), q.clone(), (&s * &q)?], 1)?;
        let field = w.affine(
            &w.affine(&field, "head.field.0")?.gelu_erf()?,
            "head.field.3",
        )?;
        let type_id = match kind {
            "choice" => 0,
            "score" => 1,
            _ => 2,
        };
        let field = w.ln(
            &field.broadcast_add(&w.dense["head.type_emb.weight"].narrow(0, type_id, 1)?)?,
            "head.field_norm",
        )?;
        let on = w.ln(&o_proj, "head.option_norm")?;
        let f = field.broadcast_as(on.shape())?;
        let joint = (l2(&f)? * l2(&on)?)?.sum(D::Minus1)?;
        let pair = Tensor::cat(&[f.clone(), on.clone(), (&f * &on)?, (&f - &on)?.abs()?], 1)?;
        let residual = w.linear(
            &w.affine(&pair, "head.residual.0")?.gelu_erf()?,
            "head.residual.3.weight",
            None,
        )?;
        let anchor = l2(&(&s0 + &q0)?)?;
        let prior = l2(&o)?.matmul(&anchor.t()?)?.flatten_all()?;
        let prior = prior.to_vec1::<f32>()?;
        let joint = joint.to_vec1::<f32>()?;
        let residual = residual.flatten_all()?.to_vec1::<f32>()?;
        let h = &cfg["head"];
        let raw: Vec<_> = (0..options.len())
            .map(|i| {
                (h["prior_scale"].as_f64().unwrap() * prior[i] as f64
                    + h["gate"].as_f64().unwrap()
                        * (h["joint_scale"].as_f64().unwrap() * joint[i] as f64
                            + residual[i] as f64
                            + h["residual_bias"].as_f64().unwrap())) as f32
            })
            .collect();
        if !raw.iter().all(|x| x.is_finite()) {
            return Err("non-finite EG2 head logits".into());
        }
        Ok(softmax(&raw, cfg["temperatures"][kind].as_f64().unwrap()))
    }
    pub fn render(state: &str, instructions: &str, meanings: &[String]) -> Vec<String> {
        let mut t = vec![
            format!("workflow:  | state: {state}"),
            format!("choice question: {instructions}"),
        ];
        for (k, v) in ["keep", "name", "abstain"].iter().zip(meanings) {
            t.push(format!("question: {instructions} | answer: {k}: {v}"));
        }
        t
    }
}
