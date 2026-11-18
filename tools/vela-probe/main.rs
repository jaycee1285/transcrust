//! Native translation of vela_onnx.py's decisions-only formatter/runner.
use ort::{session::Session, value::Tensor};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path, time::Instant};
use tokenizers::Tokenizer;
type Error = Box<dyn std::error::Error>;
const MODEL_HASH: &str = "a40ddd8c8662e05d9432f44d8f34ec11b0078375e1b709618914f251d6d33643";
const TOKENIZER_HASH: &str = "38734bb17ef19f84c67b5eb5692bea19801ffd02cb5c5ffc11429e76b0242e08";
fn read(p: impl AsRef<Path>) -> Result<Value, Error> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
fn save(p: &str, v: &Value) -> Result<(), Error> {
    fs::write(p, serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn hash(p: &Path) -> Result<String, Error> {
    let mut f = fs::File::open(p)?;
    let mut h = Sha256::new();
    let mut b = vec![0; 1024 * 1024];
    loop {
        let n = f.read(&mut b)?;
        if n == 0 {
            break;
        }
        h.update(&b[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
fn signature(dir: &Path) -> Result<Value, Error> {
    let model = hash(&dir.join("model.onnx"))?;
    let tok = hash(&dir.join("tokenizer.json"))?;
    if model != MODEL_HASH || tok != TOKENIZER_HASH {
        return Err("Vela artifact checksum mismatch".into());
    }
    let mut h = Sha256::new();
    h.update(include_str!("main.rs"));
    Ok(json!({"model":model,"tokenizer":tok,"source":format!("{:x}",h.finalize())}))
}
fn best(p: &[f64]) -> usize {
    (0..p.len())
        .max_by(|a, b| p[*a].total_cmp(&p[*b]).then_with(|| b.cmp(a)))
        .unwrap()
}
fn probs(z: &[f32]) -> Result<Vec<f64>, Error> {
    if z.is_empty() || z.iter().any(|z| !z.is_finite()) {
        return Err("invalid Vela logits".into());
    }
    let m = z.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
    let p: Vec<_> = z.iter().map(|z| (*z as f64 - m).exp()).collect();
    let sum: f64 = p.iter().sum();
    Ok(p.into_iter().map(|p| p / sum).collect())
}
#[derive(Clone)]
struct Row {
    ids: Vec<i64>,
    positions: Vec<i64>,
    state_start: i64,
}
struct Model {
    session: Session,
    tok: Tokenizer,
}
impl Model {
    fn load(dir: &Path, mode: &str) -> Result<Self, Error> {
        let mut builder = Session::builder()?
            .with_intra_threads(4)?
            .with_inter_threads(1)?;
        builder = match mode {
            "fast" => builder.with_config_entry("session.disable_quant_qdq", "1")?,
            "low" => builder.with_config_entry("session.qdq_matmulnbits_accuracy_level", "0")?,
            _ => return Err("mode must be fast or low".into()),
        };
        let session = builder.commit_from_file(dir.join("model.onnx"))?;
        let mut tok =
            Tokenizer::from_file(dir.join("tokenizer.json")).map_err(|e| e.to_string())?;
        tok.with_truncation(None).map_err(|e| e.to_string())?;
        tok.with_padding(None);
        Ok(Self { session, tok })
    }
    fn encode(
        &self,
        state: &str,
        instructions: &str,
        opts: &[(String, String)],
    ) -> Result<Row, Error> {
        let tokens = |s: &str| -> Result<Vec<i64>, Error> {
            if s.is_empty() {
                return Ok(Vec::new());
            }
            Ok(self
                .tok
                .encode(s, false)
                .map_err(|e| e.to_string())?
                .get_ids()
                .iter()
                .map(|n| *n as i64)
                .collect())
        };
        let mut ids = vec![0, 49152];
        ids.extend(tokens(instructions)?);
        let mut positions = Vec::new();
        for (key, desc) in opts {
            positions.push(ids.len() as i64);
            ids.push(49153);
            ids.extend(tokens(&if desc.is_empty() {
                key.clone()
            } else {
                format!("{key}: {desc}")
            })?);
        }
        ids.push(49155);
        let state_start = ids.len() as i64;
        ids.push(49156);
        ids.extend(tokens(state)?);
        ids.push(0);
        if ids.len() > 32768 {
            return Err("Vela truncation refused".into());
        }
        Ok(Row {
            ids,
            positions,
            state_start,
        })
    }
    fn packed(&mut self, rows: &[Row]) -> Result<Vec<Vec<f64>>, Error> {
        if rows.len() > 1 && rows.iter().any(|r| r.ids.len() > 129) {
            return Err("Vela graph cannot pack prompts above 129 tokens".into());
        }
        let mut ids = Vec::new();
        let mut pos = Vec::new();
        let mut seg = Vec::new();
        let mut options = Vec::new();
        let mut questions = Vec::new();
        let mut starts = Vec::new();
        let mut ends = Vec::new();
        for (i, row) in rows.iter().enumerate() {
            let offset = ids.len() as i64;
            let n = row.ids.len();
            ids.extend_from_slice(&row.ids);
            pos.extend((0..n).map(|n| n as i64));
            seg.extend(std::iter::repeat_n(i as i64, n));
            for p in &row.positions {
                options.push(offset + p);
                questions.push(offset + 1);
                starts.push(offset + row.state_start);
                ends.push(offset + n as i64 - 1);
            }
        }
        let seq = |v: Vec<i64>| Tensor::from_array(([1, v.len()], v));
        let vec = |v: Vec<i64>| Tensor::from_array(([v.len()], v));
        let outputs=self.session.run(ort::inputs!["input_ids"=>seq(ids)?,"position_ids"=>seq(pos)?,"segment_ids"=>seq(seg)?,"positions"=>vec(options)?,"questions"=>vec(questions)?,"state_starts"=>vec(starts)?,"state_ends"=>vec(ends)?])?;
        let (_, logits) = outputs["logits"].try_extract_tensor::<f32>()?;
        let mut at = 0;
        let mut out = Vec::new();
        for row in rows {
            let end = at + row.positions.len();
            out.push(probs(
                logits.get(at..end).ok_or("Vela logit count mismatch")?,
            )?);
            at = end;
        }
        if at != logits.len() {
            return Err("unexpected extra Vela logits".into());
        }
        Ok(out)
    }
    fn infer(&mut self, rows: &[Row]) -> Result<(Vec<Vec<f64>>, usize), Error> {
        // Exact upstream packing policy: <=129 tokens per prompt, ~128 per pass.
        let mut groups = Vec::new();
        let mut cur = Vec::new();
        let mut size = 0;
        for (i, r) in rows.iter().enumerate() {
            let n = r.ids.len();
            if n > 129 {
                groups.push(vec![i]);
                continue;
            }
            if !cur.is_empty() && size + n > 128 {
                groups.push(std::mem::take(&mut cur));
                size = 0;
            }
            cur.push(i);
            size += n;
        }
        if !cur.is_empty() {
            groups.push(cur);
        }
        let mut out = vec![Vec::new(); rows.len()];
        for group in &groups {
            let batch = group.iter().map(|i| rows[*i].clone()).collect::<Vec<_>>();
            for (i, p) in group.iter().zip(self.packed(&batch)?) {
                out[*i] = p;
            }
        }
        Ok((out, groups.len()))
    }
}
fn public(dir: &Path, mode: &str, out: &str) -> Result<(), Error> {
    let sig = signature(dir)?;
    let start = Instant::now();
    let mut model = Model::load(dir, mode)?;
    let load = start.elapsed().as_secs_f64();
    eprintln!("Vela {mode} loaded in {load:.2}s");
    let cases = read("tools/vela-probe/fixtures/public.json")?;
    let mut rows = Vec::new();
    let mut passed = true;
    for c in cases.as_array().unwrap() {
        let opts: Vec<(String, String)> = serde_json::from_value(c["options"].clone())?;
        let row = model.encode(
            c["state"].as_str().unwrap(),
            c["instructions"].as_str().unwrap(),
            &opts,
        )?;
        let p = model.packed(&[row.clone()])?.remove(0);
        let expected: Vec<f64> = serde_json::from_value(c["expected"].clone())?;
        let delta = p
            .iter()
            .zip(&expected)
            .map(|(a, b)| (a - b).abs())
            .fold(0f64, f64::max);
        let ok = delta < 0.02 && best(&p) == best(&expected);
        passed &= ok;
        eprintln!("public {}: {p:?}, delta {delta:.6}", c["id"]);
        rows.push(json!({"id":c["id"],"probabilities":p,"expected_rounded":expected,"passed":ok,"token_ids":row.ids,"positions":row.positions,"state_start":row.state_start}));
    }
    let small = model.encode(
        "I was charged twice.",
        "Which team?",
        &[
            ("billing".into(), "Payments and refunds".into()),
            ("tech".into(), "Software bugs".into()),
        ],
    )?;
    let other = model.encode(
        "My application crashes on login.",
        "Which team?",
        &[
            ("billing".into(), "Payments and refunds".into()),
            ("tech".into(), "Software bugs".into()),
        ],
    )?;
    let mut single = model.packed(&[small.clone()])?.remove(0);
    single.extend(model.packed(&[other.clone()])?.remove(0));
    let packed = model.packed(&[small, other])?;
    let delta = packed
        .iter()
        .flatten()
        .zip(single.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0f64, f64::max);
    passed &= delta < 1e-5;
    save(
        out,
        &json!({"signature":sig,"mode":mode,"passed":passed,"validation":"Two rounded published example distributions, not a full upstream parity suite; distinct billing/technical short prompts checked packed versus isolated","rows":rows,"packed_single_max_delta":delta,"load_seconds":load,"wall_seconds":start.elapsed().as_secs_f64()}),
    )?;
    if !passed {
        return Err("Vela public checks failed".into());
    }
    Ok(())
}
fn question(c: &Value, synthetic: &Value) -> Result<Value, Error> {
    if c["candidate"]["question"].is_object() {
        return Ok(c["candidate"]["question"].clone());
    }
    synthetic["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["case"]["id"] == c["id"] && r["arm"] == c["arm"])
        .map(|r| r["questions"]["decision"].clone())
        .ok_or_else(|| "missing synthetic question".into())
}
fn evaluate(
    dir: &Path,
    mode: &str,
    gate: &str,
    scope: &str,
    surface: &str,
    out: &str,
) -> Result<(), Error> {
    let gate = read(gate)?;
    if gate["passed"] != true || gate["mode"] != mode || gate["signature"] != signature(dir)? {
        return Err("matching public/source gate required".into());
    }
    let start = Instant::now();
    let mut model = Model::load(dir, mode)?;
    let load = start.elapsed().as_secs_f64();
    let cases = read("tools/pair-router-probe/cross-encoder-q8/cases.json")?;
    let synthetic = read("tools/pair-router-probe/bluey-q8-expanded-glossary/results.json")?;
    let mut groups = Vec::new();
    for source in ["parakeet", "phonon"] {
        for arm in ["single", "two"] {
            groups.push(
                cases
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|c| c["source"] == source && c["arm"] == arm)
                    .cloned()
                    .collect::<Vec<_>>(),
            );
        }
    }
    if surface == "all" {
        let s = cases
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["source"] == "synthetic")
            .cloned()
            .collect::<Vec<_>>();
        groups.extend(s.chunks(8).map(|c| c.to_vec()));
    } else if surface != "phone" {
        return Err("surface must be phone or all".into());
    }
    let mut results = Vec::new();
    let mut batches = Vec::new();
    for group in groups {
        let mut rows = Vec::new();
        let mut states = Vec::new();
        let mut questions = Vec::new();
        for c in &group {
            let q = question(c, &synthetic)?;
            let state = match scope {
                "full" => c["state"].as_str().unwrap(),
                "stripped" => {
                    c["state"]
                        .as_str()
                        .unwrap()
                        .split_once("\n\nDictation:\n")
                        .ok_or("missing glossary boundary")?
                        .1
                }
                _ => return Err("scope must be full or stripped".into()),
            };
            let opts = ["keep", "name", "abstain"]
                .iter()
                .map(|k| {
                    (
                        k.to_string(),
                        q["criteria"][k].as_str().unwrap().to_string(),
                    )
                })
                .collect::<Vec<_>>();
            rows.push(model.encode(state, q["instructions"].as_str().unwrap(), &opts)?);
            states.push(state.to_string());
            questions.push(q);
        }
        let now = Instant::now();
        let (scores, calls) = model.infer(&rows)?;
        let seconds = now.elapsed().as_secs_f64();
        eprintln!(
            "Vela {mode}/{scope} {} {}: {} candidates, {calls} graph calls, {seconds:.3}s",
            group[0]["source"],
            group[0]["arm"],
            group.len()
        );
        batches.push(json!({"source":group[0]["source"],"arm":group[0]["arm"],"candidates":group.len(),"calls":calls,"seconds":seconds,"token_counts":rows.iter().map(|r|r.ids.len()).collect::<Vec<_>>()}));
        for ((((c, row), p), state), q) in group
            .into_iter()
            .zip(rows)
            .zip(scores)
            .zip(states)
            .zip(questions)
        {
            let choice = ["keep", "name", "abstain"][best(&p)];
            results.push(json!({"case":c,"state":state,"question":q,"choice":choice,"probabilities":p,"token_ids":row.ids,"positions":row.positions,"state_start":row.state_start}));
        }
    }
    for source in ["parakeet", "phonon"] {
        for arm in ["single", "two"] {
            let evidence = read(format!(
                "tools/pair-router-probe/phone-household-q8/evidence/{source}-decision.json"
            ))?;
            let mut text = evidence["input"].as_str().unwrap().to_string();
            let mut edits = results
                .iter()
                .filter(|r| {
                    r["case"]["source"] == source
                        && r["case"]["arm"] == arm
                        && r["choice"] == "name"
                })
                .collect::<Vec<_>>();
            edits.sort_by_key(|r| {
                std::cmp::Reverse(r["case"]["candidate"]["start"].as_u64().unwrap())
            });
            for r in edits {
                let c = &r["case"]["candidate"];
                let a = c["start"].as_u64().unwrap() as usize;
                let b = c["end"].as_u64().unwrap() as usize;
                if text.get(a..b) != c["heard"].as_str() {
                    return Err("occurrence span mismatch".into());
                }
                text.replace_range(a..b, c["canonical"].as_str().unwrap());
            }
            fs::write(format!("{out}.{source}-{arm}.txt"), text)?;
        }
    }
    let mut metrics = Vec::new();
    for split in ["dev", "eval", "diagnostic"] {
        for arm in ["single", "two"] {
            let rows = results
                .iter()
                .filter(|r| r["case"]["split"] == split && r["case"]["arm"] == arm)
                .collect::<Vec<_>>();
            metrics.push(json!({"split":split,"arm":arm,"n":rows.len(),"repair_total":rows.iter().filter(|r|r["case"]["want"]=="name").count(),"repairs":rows.iter().filter(|r|r["case"]["want"]=="name"&&r["choice"]=="name").count(),"preserve_total":rows.iter().filter(|r|r["case"]["want"]!="name").count(),"unwanted_edits":rows.iter().filter(|r|r["case"]["want"]!="name"&&r["choice"]=="name").count(),"abstentions":rows.iter().filter(|r|r["choice"]=="abstain").count()}));
        }
    }
    save(
        out,
        &json!({"signature":gate["signature"],"mode":mode,"scope":scope,"surface":surface,"load_seconds":load,"wall_seconds":start.elapsed().as_secs_f64(),"batches":batches,"rows":results,"metrics":metrics}),
    )?;
    Ok(())
}
fn main() -> Result<(), Error> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    match a.first().map(String::as_str){Some("public")if a.len()==4=>public(Path::new(&a[1]),&a[2],&a[3]),Some("evaluate")if a.len()==7=>evaluate(Path::new(&a[1]),&a[2],&a[3],&a[4],&a[5],&a[6]),_=>Err("vela-probe public MODEL fast|low OUT; vela-probe evaluate MODEL fast|low GATE full|stripped phone|all OUT".into())}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_readout() {
        assert_eq!(best(&[0.5, 0.5, 0.]), 0);
        assert!(probs(&[f32::NAN]).is_err());
        assert!(probs(&[f32::INFINITY]).is_err());
    }
}
