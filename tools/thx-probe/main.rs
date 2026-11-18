//! Offline THX-01 choice-only research probe. No daemon/backend changes.
//! Native transliteration of pinned thx01/common.py's formatter and decision head,
//! and Transformers ModernBertModel's encoder. CPU F32 from the published F16 file.
use candle_core::{safetensors::MmapedSafetensors, DType, Device, IndexOp, Result, Tensor, D};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs, io::Read, path::Path, time::Instant};
use tokenizers::Tokenizer;

const REV: &str = "b323afcef149f9288a9d0daef4badd67b9b87c61";
const NAMES: [&str; 3] = ["keep", "name", "abstain"];

struct Model {
    weights: HashMap<String, Tensor>,
    tok: Tokenizer,
    cls: u32,
    sep: u32,
    mask: u32,
    pad: u32,
    temperature: f64,
}

impl Model {
    fn load(dir: &Path) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        for (file, expected) in [
            (
                "model.safetensors",
                "52ce834b2a4d265fbc351338655977023ddb61b0363f5f6a4fd096e6c70154b0",
            ),
            (
                "tokenizer/tokenizer.json",
                "609d8f4c067cd3950f88594c5a802616cea245823836ef5848ee4fc40aab5b6f",
            ),
            (
                "tokenizer/tokenizer_config.json",
                "6c6b2d8e3c84ce0e671c129cd6b374b235d6f9863042a5836358d00a89bbb5a1",
            ),
            (
                "encoder-config.json",
                "83f6916d13ef0f556ac461f28308dc2bffa7ebeadee8ec9e2db5812020ea5bb4",
            ),
            (
                "rl_agent_config.json",
                "1bb5c62b14c0029fea4891fd6f82ae882f81dae2ba5786a813a8d8de2c02f930",
            ),
        ] {
            let mut input = fs::File::open(dir.join(file))?;
            let mut hash = Sha256::new();
            let mut buffer = vec![0; 1024 * 1024];
            loop {
                let n = input.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                hash.update(&buffer[..n]);
            }
            assert_eq!(
                format!("{:x}", hash.finalize()),
                expected,
                "hash mismatch: {file}"
            );
        }
        let cfg: Value = serde_json::from_slice(&fs::read(dir.join("encoder-config.json"))?)?;
        // Fail closed rather than quietly approximate another encoder/checkpoint.
        for (key, expected) in [
            ("hidden_size", 768),
            ("num_hidden_layers", 22),
            ("num_attention_heads", 12),
            ("intermediate_size", 1152),
            ("local_attention", 128),
            ("global_attn_every_n_layers", 3),
        ] {
            assert_eq!(cfg[key].as_u64(), Some(expected), "unsupported {key}");
        }
        assert_eq!(
            cfg["rope_parameters"]["full_attention"]["rope_theta"],
            160000
        );
        assert_eq!(
            cfg["rope_parameters"]["sliding_attention"]["rope_theta"],
            160000
        );
        assert_eq!(cfg["layer_norm_eps"], 0.00001);
        let decision: Value = serde_json::from_slice(&fs::read(dir.join("rl_agent_config.json"))?)?;
        assert_eq!(decision["head_layers"], 2);
        assert_eq!(decision["max_len"], 1024);
        assert_eq!(decision["head_max_len"], 256);
        let temperature = decision["temperature_by_options"]["choice:3-5"]
            .as_f64()
            .unwrap();
        assert!(temperature.is_finite() && temperature > 0.0);
        let tok = Tokenizer::from_file(dir.join("tokenizer/tokenizer.json"))
            .map_err(|e| e.to_string())?;
        let special: Value =
            serde_json::from_slice(&fs::read(dir.join("tokenizer/tokenizer_config.json"))?)?;
        let special_id = |key: &str| tok.token_to_id(special[key].as_str().unwrap()).unwrap();
        // Tokenizer CLS is <bos>=2, despite encoder config's cls_token_id=1.
        let (cls, sep, mask, pad) = (
            special_id("cls_token"),
            special_id("sep_token"),
            special_id("mask_token"),
            special_id("pad_token"),
        );
        let mut weights = HashMap::new();
        // SAFETY: the downloaded, pinned file must remain unmodified while mapped.
        let file = unsafe { MmapedSafetensors::new(dir.join("model.safetensors"))? };
        for (key, _) in file.tensors() {
            // act_head is ancillary action confidence, not keep/name/abstain logits.
            if key.starts_with("act_head.") || key == "temperature" {
                continue;
            }
            weights.insert(
                key.clone(),
                file.load(&key, &Device::Cpu)?.to_dtype(DType::F32)?,
            );
        }
        Ok(Self {
            weights,
            tok,
            cls,
            sep,
            mask,
            pad,
            temperature,
        })
    }

    fn w(&self, name: &str) -> &Tensor {
        &self.weights[name]
    }
    fn tokens(&self, text: &str) -> Vec<u32> {
        self.tok
            .encode(text.replace("<mask>", " "), false)
            .unwrap()
            .get_ids()
            .to_vec()
    }

    fn encode(&self, state: &str, question: &Value) -> (Vec<u32>, Vec<usize>) {
        let mut head = self.tokens(&format!(
            "choice question: {}",
            question["instructions"].as_str().unwrap()
        ));
        let mut options: Vec<_> = NAMES
            .iter()
            .map(|key| {
                let description = question["criteria"][key].as_str().unwrap();
                let mut ids = self.tokens(&format!(" {key}: {description}"));
                assert!(ids.len() <= 48, "option truncation refused");
                ids.insert(0, self.mask);
                ids
            })
            .collect();
        let budget = 256 - options.iter().map(Vec::len).sum::<usize>();
        assert!(
            budget >= 16 && head.len() <= budget,
            "head truncation refused"
        );
        let mut ids = vec![self.cls];
        ids.append(&mut head);
        ids.push(self.sep);
        let mut markers = Vec::new();
        for option in &mut options {
            markers.push(ids.len());
            ids.append(option);
        }
        ids.push(self.sep);
        ids.extend(self.tokens(state));
        ids.push(self.sep);
        assert!(ids.len() <= 1024, "state truncation refused");
        (ids, markers)
    }

    fn linear_named(&self, x: &Tensor, weight: &str, bias: Option<&str>) -> Result<Tensor> {
        let (b, s, d) = x.dims3()?;
        let w = self.w(weight);
        let y = x
            .reshape((b * s, d))?
            .matmul(&w.t()?)?
            .reshape((b, s, w.dim(0)?))?;
        match bias {
            Some(key) => y.broadcast_add(self.w(key)),
            None => Ok(y),
        }
    }
    fn linear(&self, x: &Tensor, prefix: &str) -> Result<Tensor> {
        let bias = format!("{prefix}.bias");
        self.linear_named(
            x,
            &format!("{prefix}.weight"),
            self.weights.contains_key(&bias).then_some(bias.as_str()),
        )
    }
    fn norm(&self, x: &Tensor, prefix: &str) -> Result<Tensor> {
        let centered = x.broadcast_sub(&x.mean_keepdim(D::Minus1)?)?;
        let variance = (centered.sqr()?.mean_keepdim(D::Minus1)? + 1e-5)?;
        let y = centered
            .broadcast_div(&variance.sqrt()?)?
            .broadcast_mul(self.w(&format!("{prefix}.weight")))?;
        match self.weights.get(&format!("{prefix}.bias")) {
            Some(bias) => y.broadcast_add(bias),
            None => Ok(y),
        }
    }

    fn attention(
        &self,
        packed: &Tensor,
        mask: &Tensor,
        rotary: Option<(&Tensor, &Tensor)>,
    ) -> Result<Tensor> {
        let (b, s, _) = packed.dims3()?;
        let qkv = packed
            .reshape((b, s, 3, 12, 64))?
            .permute((2, 0, 3, 1, 4))?;
        let mut q = qkv.i(0)?.contiguous()?;
        let mut k = qkv.i(1)?.contiguous()?;
        if let Some((cos, sin)) = rotary {
            let rotate = |x: &Tensor| -> Result<Tensor> {
                let half = Tensor::cat(&[&x.narrow(3, 32, 32)?.neg()?, &x.narrow(3, 0, 32)?], 3)?;
                x.broadcast_mul(cos)? + half.broadcast_mul(sin)?
            };
            q = rotate(&q)?;
            k = rotate(&k)?;
        }
        let scores = ((q / 8.0)?.matmul(&k.transpose(2, 3)?.contiguous()?)?).broadcast_add(mask)?;
        let exp = scores
            .broadcast_sub(&scores.max_keepdim(D::Minus1)?)?
            .exp()?;
        let probabilities = exp.broadcast_div(&exp.sum_keepdim(D::Minus1)?)?;
        probabilities
            .matmul(&qkv.i(2)?.contiguous()?)?
            .transpose(1, 2)?
            .contiguous()?
            .reshape((b, s, 768))
    }

    fn infer(&self, rows: &[(Vec<u32>, Vec<usize>)]) -> Result<Vec<[f32; 3]>> {
        let b = rows.len();
        let s = rows.iter().map(|r| r.0.len()).max().unwrap();
        let ids: Vec<_> = rows
            .iter()
            .flat_map(|r| (0..s).map(|i| r.0.get(i).copied().unwrap_or(self.pad)))
            .collect();
        let ids = Tensor::from_vec(ids, b * s, &Device::Cpu)?;
        let mut h = self
            .w("encoder.embeddings.tok_embeddings.weight")
            .index_select(&ids, 0)?
            .reshape((b, s, 768))?;
        h = self.norm(&h, "encoder.embeddings.norm")?;
        let padding: Vec<_> = rows
            .iter()
            .flat_map(|r| {
                (0..s).map(|i| {
                    if i < r.0.len() {
                        0f32
                    } else {
                        f32::NEG_INFINITY
                    }
                })
            })
            .collect();
        let global = Tensor::from_vec(padding, (b, 1, 1, s), &Device::Cpu)?;
        let local: Vec<_> = (0..s)
            .flat_map(|i| {
                (0..s).map(move |j| {
                    if i.abs_diff(j) <= 64 {
                        0f32
                    } else {
                        f32::NEG_INFINITY
                    }
                })
            })
            .collect();
        let local = global.broadcast_add(&Tensor::from_vec(local, (1, 1, s, s), &Device::Cpu)?)?;
        let angles: Vec<f32> = (0..s)
            .flat_map(|position| {
                (0..64).map(move |i| position as f32 / 160000f32.powf((2 * (i % 32)) as f32 / 64.0))
            })
            .collect();
        let angles = Tensor::from_vec(angles, (1, 1, s, 64), &Device::Cpu)?;
        let (cos, sin) = (angles.cos()?, angles.sin()?);
        for layer in 0..22 {
            let p = format!("encoder.layers.{layer}");
            let x = if layer == 0 {
                h.clone()
            } else {
                self.norm(&h, &format!("{p}.attn_norm"))?
            };
            let qkv = self.linear(&x, &format!("{p}.attn.Wqkv"))?;
            let a = self.attention(
                &qkv,
                if layer % 3 == 0 { &global } else { &local },
                Some((&cos, &sin)),
            )?;
            h = (&h + self.linear(&a, &format!("{p}.attn.Wo"))?)?;
            let mlp = self.linear(
                &self.norm(&h, &format!("{p}.mlp_norm"))?,
                &format!("{p}.mlp.Wi"),
            )?;
            let gated = (mlp.narrow(2, 0, 1152)?.gelu_erf()? * mlp.narrow(2, 1152, 1152)?)?;
            h = (h + self.linear(&gated, &format!("{p}.mlp.Wo"))?)?;
        }
        h = self
            .norm(&h, "encoder.final_norm")?
            .broadcast_add(&self.w("type_emb.weight").i(0)?)?;
        for layer in 0..2 {
            let p = format!("head.layers.{layer}");
            let x = self.norm(&h, &format!("{p}.norm1"))?;
            let qkv = self.linear_named(
                &x,
                &format!("{p}.self_attn.in_proj_weight"),
                Some(&format!("{p}.self_attn.in_proj_bias")),
            )?;
            let a = self.attention(&qkv, &global, None)?;
            h = (&h + self.linear(&a, &format!("{p}.self_attn.out_proj"))?)?;
            let x = self
                .linear(
                    &self.norm(&h, &format!("{p}.norm2"))?,
                    &format!("{p}.linear1"),
                )?
                .relu()?;
            h = (h + self.linear(&x, &format!("{p}.linear2"))?)?;
        }
        let mut results = Vec::new();
        for (i, (_, markers)) in rows.iter().enumerate() {
            let positions = Tensor::new(
                markers
                    .iter()
                    .map(|p| *p as u32)
                    .collect::<Vec<_>>()
                    .as_slice(),
                &Device::Cpu,
            )?;
            let selected = h.i(i)?.index_select(&positions, 0)?.unsqueeze(0)?;
            let logits = self
                .linear(
                    &self
                        .linear(&self.norm(&selected, "scorer.0")?, "scorer.1")?
                        .gelu_erf()?,
                    "scorer.3",
                )?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(logits.iter().all(|x| x.is_finite()));
            let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let mut p = [0f32; 3];
            for j in 0..3 {
                p[j] = ((logits[j] - max) / self.temperature as f32).exp();
            }
            let sum: f32 = p.iter().sum();
            for value in &mut p {
                *value /= sum;
            }
            results.push(p);
        }
        Ok(results)
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        return Err("usage: thx-probe MODEL_DIR OUTPUT.json INPUT.json... (D1 audit or saved suite results)".into());
    }
    let started = Instant::now();
    eprintln!("THX: loading native F32 weights");
    let model = Model::load(Path::new(&args[0]))?;
    let load_secs = started.elapsed().as_secs_f64();
    eprintln!("THX: loaded in {load_secs:.2}s");
    let mut sources = Vec::new();
    for path in &args[2..] {
        let input: Value = serde_json::from_slice(&fs::read(path)?)?;
        let cases: Vec<_> = if let Some(candidates) = input["candidates"].as_array() {
            candidates.iter().enumerate().map(|(i,c)| json!({"id":i,"state":c["state"],"question":c["question"],"candidate":c,"d1":input["results"][i]})).collect()
        } else {
            input["cases"].as_array().ok_or("expected audit or saved suite")?.iter().map(|c| json!({"id":c["case"]["id"],"arm":c["arm"],"case":c["case"],"state":c["state"],"question":c["questions"]["decision"],"d1_choice":c["choice"]})).collect()
        };
        let mut output = Vec::new();
        let mut parity = Value::Null;
        let mut delivered = input["input"].as_str().map(str::to_owned);
        // Frozen production audits retain one call per utterance. Larger synthetic
        // suites use bounded batches of 4 to avoid enormous attention allocations.
        let batch_size = if input["candidates"].is_array() {
            cases.len().max(1)
        } else {
            4
        };
        for chunk in cases.chunks(batch_size) {
            let rows: Vec<_> = chunk
                .iter()
                .map(|c| model.encode(c["state"].as_str().unwrap(), &c["question"]))
                .collect();
            let single = if sources.is_empty() && output.is_empty() {
                eprintln!("THX: warming and checking first row against padded batch");
                Some(model.infer(&rows[..1])?[0])
            } else {
                None
            };
            let started = Instant::now();
            let scores = model.infer(&rows)?;
            let ms = started.elapsed().as_secs_f64() * 1000.0;
            if let Some(single) = single {
                let delta = (0..3)
                    .map(|i| (single[i] - scores[0][i]).abs())
                    .fold(0f32, f32::max);
                assert!(delta < 2e-4, "batch/single probability drift: {delta}");
                parity = json!({"max_probability_delta":delta,"tolerance":0.0002});
            }
            eprintln!(
                "THX: {} rows, {ms:.0}ms ({}/{})",
                rows.len(),
                output.len() + rows.len(),
                cases.len()
            );
            for ((case, row), p) in chunk.iter().zip(rows).zip(scores) {
                let mut best = 0;
                for i in 1..3 {
                    if p[i] > p[best] {
                        best = i;
                    }
                }
                let mut record = case.clone();
                record["choice"] = json!(NAMES[best]);
                record["probabilities"] = json!({"keep":p[0],"name":p[1],"abstain":p[2]});
                record["input_ids"] = json!(row.0);
                record["marker_pos"] = json!(row.1);
                record["batch_ms"] = json!(ms);
                output.push(record);
            }
        }
        if let Some(text) = &mut delivered {
            for record in output.iter().rev().filter(|r| r["choice"] == "name") {
                let c = &record["candidate"];
                text.replace_range(
                    c["start"].as_u64().unwrap() as usize..c["end"].as_u64().unwrap() as usize,
                    c["canonical"].as_str().unwrap(),
                );
            }
        }
        sources.push(json!({"source":path,"raw":input["raw"],"input":input["input"],"delivered":delivered,"d1_delivered":input["delivered"],"first_row_batch_check":parity,"cases":output}));
        // Persist each completed source so a long synthetic run is inspectable.
        fs::write(
            &args[1],
            serde_json::to_vec_pretty(
                &json!({"model":"doofz/THX-01","revision":REV,"runtime":"native Rust candle-core CPU F32","temperature":model.temperature,"load_secs":load_secs,"sources":sources}),
            )?,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_attention_layout_and_padding_preserve_independent_rows() -> Result<()> {
        let model = Model {
            weights: HashMap::new(),
            tok: Tokenizer::new(tokenizers::models::bpe::BPE::default()),
            cls: 2,
            sep: 1,
            mask: 4,
            pad: 0,
            temperature: 2.55,
        };
        let mut data = vec![0f32; 2 * 2 * 2304];
        for (row, values) in [[1f32, 10f32], [3f32, 9f32]].iter().enumerate() {
            for (position, value) in values.iter().enumerate() {
                data[(row * 2 + position) * 2304 + 1536..(row * 2 + position + 1) * 2304]
                    .fill(*value);
            }
        }
        let packed = Tensor::from_vec(data, (2, 2, 2304), &Device::Cpu)?;
        let mask = Tensor::from_vec(
            vec![0f32, f32::NEG_INFINITY, 0f32, 0f32],
            (2, 1, 1, 2),
            &Device::Cpu,
        )?;
        let out = model.attention(&packed, &mask, None)?.to_vec3::<f32>()?;
        for position in 0..2 {
            assert!(out[0][position].iter().all(|v| (*v - 1.0).abs() < 1e-6));
            assert!(out[1][position].iter().all(|v| (*v - 6.0).abs() < 1e-6));
        }
        Ok(())
    }
}
