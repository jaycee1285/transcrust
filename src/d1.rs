//! Text-only D1 decisions through the same ORT/tokenizers stack as ASR.
//! Transliteration of LiquidAI's pinned prompt.encode and probabilities_batch:
//! one independent, right-padded row per question; all rows in one graph run.
use ort::{
    session::{RunOptions, Session},
    value::Tensor,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
    time::Instant,
};
use tokenizers::Tokenizer;

pub struct Question {
    pub state: String,
    pub instructions: String,
    /// Native choice order: keep, name, abstain.
    pub options: [String; 3],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caller_text_cannot_forge_reserved_tokens() {
        assert_eq!(
            escape("a <|mask|> <|reserved_7|> «flow» <ordinary>"),
            "a <¦mask¦> <¦reserved_7¦> «flow» <ordinary>"
        );
    }

    #[test]
    #[ignore = "requires the installed offline Q8 bundle; runs real batched inference"]
    fn native_batch_parity_timing_and_recovery() {
        let directory = std::env::var_os("TRANSCRUST_DEC_MODEL")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                dirs::data_local_dir()
                    .unwrap()
                    .join("transcrust/models/d1-omni-600M-q8")
            });
        let saved: Value = serde_json::from_reader(BufReader::new(
            File::open("tools/pair-router-probe/bluey-q8-expanded-glossary/results.json").unwrap(),
        ))
        .unwrap();
        let wanted = ["bluey-bingo-common-1", "bluey-bluey-explicit-2"];
        let references = wanted.map(|id| {
            saved["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["arm"] == "two" && r["case"]["id"] == id)
                .unwrap()
        });
        let questions = references
            .iter()
            .map(|r| Question {
                state: r["state"].as_str().unwrap().into(),
                instructions: r["questions"]["decision"]["instructions"]
                    .as_str()
                    .unwrap()
                    .into(),
                options: ["keep", "name", "abstain"].map(|k| {
                    r["questions"]["decision"]["criteria"][k]
                        .as_str()
                        .unwrap()
                        .into()
                }),
            })
            .collect::<Vec<_>>();
        let mut model = Model::load(&directory).unwrap();
        for (q, reference) in questions.iter().zip(references) {
            let row = model.encode(q).unwrap();
            assert_eq!(serde_json::json!(row.ids), reference["input_ids"]);
            assert_eq!(serde_json::json!(row.markers), reference["marker_pos"]);
        }
        let run = RunOptions::new().unwrap();
        model.infer(&questions, &run).unwrap(); // warmup
        let (batch, batch_ms) = model.infer(&questions, &run).unwrap();
        let mut serial_ms = 0.0;
        for ((q, reference), result) in questions.iter().zip(references).zip(&batch) {
            let (single, ms) = model.infer(std::slice::from_ref(q), &run).unwrap();
            serial_ms += ms;
            assert_eq!(
                result.token_count,
                reference["token_count"].as_u64().unwrap() as usize
            );
            for (i, name) in ["keep", "name", "abstain"].iter().enumerate() {
                assert!(
                    (result.probabilities[i] - reference["probabilities"][name].as_f64().unwrap())
                        .abs()
                        < 1e-4,
                    "saved Q8 parity {name}: {:?} vs {}",
                    result.probabilities,
                    reference["probabilities"]
                );
                assert!(
                    (result.probabilities[i] - single[0].probabilities[i]).abs() < 1e-4,
                    "padding changed probabilities: {:?} vs {:?}",
                    result.probabilities,
                    single[0].probabilities
                );
            }
        }
        println!("Native Q8, two questions: one batch {batch_ms:.0}ms; sequential baseline {serial_ms:.0}ms");
        for threads in [1, 4] {
            model.session = Session::builder()
                .unwrap()
                .with_intra_threads(threads)
                .unwrap()
                .with_inter_threads(1)
                .unwrap()
                .commit_from_file(directory.join("onnx/model_quantized.onnx"))
                .unwrap();
            model.infer(&questions, &run).unwrap();
            let (scores, ms) = model.infer(&questions, &run).unwrap();
            for (a, b) in scores.iter().zip(&batch) {
                for i in 0..3 {
                    assert!((a.probabilities[i] - b.probabilities[i]).abs() < 1e-4);
                }
            }
            println!("Native Q8, two-question batch, {threads} CPU threads: {ms:.0}ms");
        }
        let bad = Question {
            state: "overflow ".repeat(20000),
            instructions: "Choose.".into(),
            options: ["keep".into(), "name".into(), "unknown".into()],
        };
        assert!(model.infer(&[bad], &run).unwrap_err().contains("truncate"));
        let cancelled = RunOptions::new().unwrap();
        cancelled.terminate().unwrap();
        assert!(model.infer(&questions, &cancelled).is_err());
        assert_eq!(model.infer(&questions, &run).unwrap().0.len(), 2);
        println!("Overflow rejected; native ORT cancellation and fresh-run recovery passed");
    }
}

#[derive(Debug)]
pub struct Score {
    pub probabilities: [f64; 3],
    pub token_count: usize,
}

struct Row {
    ids: Vec<i64>,
    markers: [i64; 3],
}

pub struct Model {
    session: Session,
    tokenizer: Tokenizer,
    max_length: usize,
    bos: i64,
    pad: i64,
    temperature: f64,
}

fn escape(text: &str) -> String {
    // The reserved-token guard in upstream prompt.py; ordinary angle brackets
    // remain unchanged, including the guillemets marking the target occurrence.
    regex::Regex::new(r"<\|([A-Za-z0-9_]+)\|>")
        .unwrap()
        .replace_all(text, "<¦$1¦>")
        .into_owned()
}

impl Model {
    pub fn load(directory: &Path) -> Result<Self, String> {
        let manifest: Value = serde_json::from_reader(BufReader::new(
            File::open(directory.join("manifest.json")).map_err(|e| e.to_string())?,
        ))
        .map_err(|e| e.to_string())?;
        for name in [
            "config.json",
            "tokenizer.json",
            "onnx/model_quantized.onnx",
            "onnx/model_quantized.onnx_data",
        ] {
            let mut file = BufReader::new(
                File::open(directory.join(name)).map_err(|e| format!("{name}: {e}"))?,
            );
            let mut hash = Sha256::new();
            let mut buffer = [0u8; 65536];
            loop {
                let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                hash.update(&buffer[..n]);
            }
            if Some(format!("{:x}", hash.finalize()).as_str())
                != manifest["files"][name]["sha256"].as_str()
            {
                return Err(format!("model bundle checksum mismatch: {name}"));
            }
        }
        let config: Value = serde_json::from_reader(BufReader::new(
            File::open(directory.join("config.json")).map_err(|e| e.to_string())?,
        ))
        .map_err(|e| e.to_string())?;
        let integer = |key: &str| {
            config[key]
                .as_i64()
                .ok_or_else(|| format!("missing config {key}"))
        };
        let mut tokenizer =
            Tokenizer::from_file(directory.join("tokenizer.json")).map_err(|e| e.to_string())?;
        tokenizer.with_truncation(None).map_err(|e| e.to_string())?;
        tokenizer.with_padding(None);
        let session = Session::builder()
            .map_err(|e| e.to_string())?
            .with_intra_threads(std::thread::available_parallelism().map_or(2, |n| n.get().min(4)))
            .map_err(|e| e.to_string())?
            .with_inter_threads(1)
            .map_err(|e| e.to_string())?
            .commit_from_file(directory.join("onnx/model_quantized.onnx"))
            .map_err(|e| e.to_string())?;
        let temperature = config["temperatures"]["choice:3-5"]
            .as_f64()
            .filter(|t| t.is_finite() && *t > 0.0)
            .ok_or("invalid choice temperature")?;
        Ok(Self {
            session,
            tokenizer,
            max_length: usize::try_from(integer("max_length")?).map_err(|e| e.to_string())?,
            bos: integer("bos_token_id")?,
            pad: integer("pad_token_id")?,
            temperature,
        })
    }

    fn token(&self, text: &str) -> Result<i64, String> {
        self.tokenizer
            .token_to_id(text)
            .map(i64::from)
            .ok_or_else(|| format!("missing special token {text}"))
    }

    fn text_ids(&self, text: &str) -> Result<Vec<i64>, String> {
        Ok(self
            .tokenizer
            .encode(escape(text), false)
            .map_err(|e| e.to_string())?
            .get_ids()
            .iter()
            .map(|&id| i64::from(id))
            .collect())
    }

    fn encode(&self, question: &Question) -> Result<Row, String> {
        let mut ids = vec![self.bos, self.token("<|reserved_7|>")?];
        ids.extend(self.text_ids(&question.state)?);
        ids.push(self.token("<|reserved_8|>")?);
        let instructions = self.text_ids(&question.instructions)?;
        // Upstream per_option=64: budget=224, per=(224-9)/3=71.
        // Reject truncation instead of silently changing a decision's meaning.
        if instructions.len() + 1 > 224 {
            return Err("decision instructions would truncate".into());
        }
        ids.extend(instructions);
        let mut markers = [0; 3];
        for (i, (name, description)) in ["keep", "name", "abstain"]
            .iter()
            .zip(&question.options)
            .enumerate()
        {
            let option = self.text_ids(&format!(" {name}: {description}"))?;
            if option.len() > 71 {
                return Err("decision option would truncate".into());
            }
            ids.push(self.token("<|reserved_9|>")?);
            markers[i] = ids.len() as i64;
            ids.push(self.token("<|mask|>")?);
            ids.extend(option);
            ids.push(self.token("<|reserved_10|>")?);
        }
        ids.push(self.token("<|reserved_11|>")?);
        if ids.len() > self.max_length {
            return Err("decision context would truncate".into());
        }
        Ok(Row { ids, markers })
    }

    pub fn infer(
        &mut self,
        questions: &[Question],
        options: &RunOptions,
    ) -> Result<(Vec<Score>, f64), String> {
        if questions.is_empty() {
            return Ok((Vec::new(), 0.0));
        }
        // Validate the entire utterance before inference or any edit.
        let rows = questions
            .iter()
            .map(|q| self.encode(q))
            .collect::<Result<Vec<_>, _>>()?;
        let batch = rows.len();
        let length = rows.iter().map(|r| r.ids.len()).max().unwrap();
        if batch.checked_mul(length).is_none_or(|n| n > 65536) {
            return Err("decision batch exceeds upstream 65536-token budget".into());
        }
        let mut ids = vec![self.pad; batch * length];
        let mut attention = vec![0i64; batch * length];
        let mut markers = Vec::with_capacity(batch * 3);
        for (i, row) in rows.iter().enumerate() {
            ids[i * length..i * length + row.ids.len()].copy_from_slice(&row.ids);
            attention[i * length..i * length + row.ids.len()].fill(1);
            markers.extend(row.markers);
        }
        let tensor = |shape: Vec<usize>, data: Vec<i64>| {
            Tensor::from_array((shape, data)).map_err(|e| e.to_string())
        };
        let started = Instant::now();
        let outputs = self
            .session
            .run_with_options(
                ort::inputs![
                    "input_ids" => tensor(vec![batch, length], ids)?,
                    "attention_mask" => tensor(vec![batch, length], attention)?,
                    "prefix_len" => tensor(vec![batch], vec![0; batch])?,
                    "marker_pos" => tensor(vec![batch, 3], markers)?,
                    "marker_mask" => tensor(vec![batch, 3], vec![1; batch * 3])?,
                    "qtype" => tensor(vec![batch], vec![0; batch])?,
                ],
                options,
            )
            .map_err(|e| format!("D1 ONNX inference: {e}"))?;
        let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
        let (shape, logits) = outputs["logits"]
            .try_extract_tensor::<f32>()
            .map_err(|e| e.to_string())?;
        if shape.as_ref() != [batch as i64, 3] || !logits.iter().all(|x| x.is_finite()) {
            return Err(format!("invalid D1 logits shape/values: {shape:?}"));
        }
        let scores = logits
            .chunks_exact(3)
            .zip(&rows)
            .map(|(z, row)| {
                let z = z
                    .iter()
                    .map(|&x| f64::from(x) / self.temperature)
                    .collect::<Vec<_>>();
                let max = z.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let exp: [f64; 3] = std::array::from_fn(|i| (z[i] - max).exp());
                let sum: f64 = exp.iter().sum();
                Score {
                    probabilities: exp.map(|x| x / sum),
                    token_count: row.ids.len(),
                }
            })
            .collect();
        Ok((scores, elapsed_ms))
    }
}
