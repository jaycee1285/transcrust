use decision_tensor as common;
use transcrust_decision::decision_tensor;
mod eg2;
mod quant;
use common::{best, Error};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs, io::Read, path::Path, time::Instant};
use transcrust_decision::ultra;
fn read(p: impl AsRef<Path>) -> Result<Value, Error> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
fn save(p: &str, v: &Value) -> Result<(), Error> {
    fs::write(p, serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn signature(dir: &Path) -> Result<Value, Error> {
    let mut result = serde_json::Map::new();
    for name in [
        "model.safetensors",
        "weights.bin",
        "manifest.json",
        "tokenizer.json",
        "reference.json",
        "weights.q4",
        "ultra-q4.json",
    ] {
        let path = dir.join(name);
        if !path.exists() {
            continue;
        }
        let mut f = fs::File::open(path)?;
        let mut h = Sha256::new();
        let mut b = vec![0; 1024 * 1024];
        loop {
            let n = f.read(&mut b)?;
            if n == 0 {
                break;
            }
            h.update(&b[..n]);
        }
        let hash = format!("{:x}", h.finalize());
        let expected = match name {
            "model.safetensors" => {
                Some("67ac518c172934138f5d0bb06aa34523c9d9dfb6261f5dac3c860ab3f3542e91")
            }
            "weights.bin" => {
                Some("4ab373232624a889707669cfe3163b81e98ec762affff02505a72286af810792")
            }
            _ => None,
        };
        if expected.is_some_and(|e| e != hash) {
            return Err(format!("artifact hash mismatch: {name}").into());
        }
        result.insert(name.into(), json!(hash));
    }
    let mut h = Sha256::new();
    for s in [
        include_str!("main.rs"),
        include_str!("../../crates/decision/src/decision_tensor.rs"),
        include_str!("eg2.rs"),
        include_str!("../../crates/decision/src/ultra.rs"),
        include_str!("quant.rs"),
    ] {
        h.update(s);
    }
    result.insert("native_source".into(), json!(format!("{:x}", h.finalize())));
    Ok(Value::Object(result))
}
fn embeddings(m: &eg2::Eg2, texts: &[String]) -> Result<HashMap<String, Vec<f32>>, Error> {
    let mut unique = Vec::new();
    for t in texts {
        if !unique.contains(t) {
            unique.push(t.clone());
        }
    }
    let mut cache = HashMap::new();
    for chunk in unique.chunks(8) {
        let now = Instant::now();
        let v = m.embed(chunk)?;
        eprintln!(
            "EG2: {} texts in {:.2}s",
            chunk.len(),
            now.elapsed().as_secs_f64()
        );
        for (t, e) in chunk.iter().zip(v) {
            cache.insert(t.clone(), e);
        }
    }
    Ok(cache)
}
fn eg_parity(dir: &Path, fixtures: &Path, out: &str) -> Result<(), Error> {
    let sig = signature(dir)?;
    let start = Instant::now();
    let m = eg2::Eg2::load(dir)?;
    let load = start.elapsed().as_secs_f64();
    let gold = read(dir.join("reference.json"))?;
    let fixture = read(fixtures.join("eg2-prompts.json"))?;
    let requests = read(fixtures.join("requests.json"))?;
    let texts: Vec<String> = serde_json::from_value(gold["texts"].clone())?;
    let got = m.embed(&texts)?;
    let expected: Vec<Vec<f32>> = serde_json::from_value(gold["embeddings"].clone())?;
    let mut enc = Vec::new();
    for (a, b) in got.iter().zip(&expected) {
        let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let bn: f32 = b.iter().map(|v| v * v).sum::<f32>().sqrt();
        let delta = a
            .iter()
            .zip(b)
            .map(|(x, y)| (x - y).abs())
            .fold(0f32, f32::max);
        enc.push(json!({"cosine":dot/bn,"max_abs":delta}));
    }
    eprintln!("encoder checks: {enc:?}");
    if enc.iter().any(|e| e["cosine"].as_f64().unwrap() < 0.999) {
        save(out, &json!({"signature":sig,"passed":false,"encoder":enc}))?;
        return Err("EG2 encoder reference mismatch".into());
    }
    let mut texts = Vec::new();
    for state in fixture["states"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["workflow"] == "" && s["text"].is_string())
    {
        let text = state["text"].as_str().unwrap();
        assert_eq!(json!(m.tokens(text)?), state["ids"], "state tokens differ");
        texts.push(text.into());
    }
    for q in fixture["questions"].as_array().unwrap() {
        for (t, ids) in q["texts"]
            .as_array()
            .unwrap()
            .iter()
            .zip(q["ids"].as_array().unwrap())
        {
            let text = t.as_str().unwrap();
            assert_eq!(json!(m.tokens(text)?), *ids, "question tokens differ");
            texts.push(text.into());
        }
    }
    let cache = embeddings(&m, &texts)?;
    let mut rows = Vec::new();
    let mut agrees = 0;
    let mut delta = 0f64;
    for s in fixture["states"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["workflow"] == "")
    {
        let si = s["state"].as_u64().unwrap() as usize;
        for q in fixture["questions"].as_array().unwrap() {
            let qi = q["question"].as_u64().unwrap() as usize;
            if gold["quantized"][si][qi].is_null() {
                continue;
            }
            let text = q["texts"].as_array().unwrap();
            let opts: Vec<_> = text[1..]
                .iter()
                .map(|v| cache[v.as_str().unwrap()].clone())
                .collect();
            let kind = requests["questions"][qi]["type"].as_str().unwrap();
            let p = m.logits(
                &cache[s["text"].as_str().unwrap()],
                &cache[text[0].as_str().unwrap()],
                &opts,
                kind,
            )?;
            let mut e: Vec<f64> = serde_json::from_value(gold["quantized"][si][qi].clone())?;
            if kind == "noul" {
                e.reverse();
            }
            let d = p
                .iter()
                .zip(&e)
                .map(|(a, b)| (a - b).abs())
                .fold(0f64, f64::max);
            let same = best(&p) == best(&e);
            agrees += usize::from(same);
            delta = delta.max(d);
            rows.push(json!({"state":si,"question":qi,"native":p,"reference":e,"same":same,"max_delta":d}));
        }
    }
    let passed = rows.len() == 54 && agrees == 54 && delta < 0.02;
    save(
        out,
        &json!({"signature":sig,"passed":passed,"encoder":enc,"agrees":agrees,"count":rows.len(),"max_delta":delta,"load_seconds":load,"wall_seconds":start.elapsed().as_secs_f64(),"rows":rows}),
    )?;
    eprintln!("EG2: {agrees}/{}; max delta {delta}", rows.len());
    if !passed {
        return Err("EG2 reference parity gate failed".into());
    }
    Ok(())
}
fn ultra_parity(
    dir: &Path,
    fixture: &Path,
    out: &str,
    baseline: Option<&Path>,
) -> Result<(), Error> {
    let sig = signature(dir)?;
    let quantized = dir.join("ultra-q4.json").exists();
    if quantized {
        let b = read(baseline.ok_or("Q4 public checks require the current FP32 public gate")?)?;
        if b["passed"] != true
            || b["signature"]["native_source"] != sig["native_source"]
            || b["signature"]["model.safetensors"] != quant::SOURCE_HASH
            || b["signature"]["tokenizer.json"] != sig["tokenizer.json"]
        {
            return Err("FP32 reference/source gate mismatch".into());
        }
    }
    let start = Instant::now();
    let m = ultra::Ultra::load(dir)?;
    let load = start.elapsed().as_secs_f64();
    let checks = read(fixture)?;
    let mut rows = Vec::new();
    let mut agrees = 0;
    for c in checks.as_array().unwrap() {
        let t = Instant::now();
        let labels: Vec<String> = serde_json::from_value(c["labels"].clone())?;
        let p = m.score(
            c["state"].as_str().unwrap(),
            c["instructions"].as_str().unwrap(),
            &labels,
        )?;
        let i = best(&p);
        let same = labels[i] == c["expected_label"].as_str().unwrap();
        agrees += usize::from(same);
        let delta = (p[i] - c["expected_p"].as_f64().unwrap()).abs();
        eprintln!(
            "Ultra {}: {} {:.5}, delta {:.5}, {:.2}s",
            c["id"],
            labels[i],
            p[i],
            delta,
            t.elapsed().as_secs_f64()
        );
        rows.push(json!({"id":c["id"],"native":p,"labels":labels,"expected_label":c["expected_label"],"expected_p":c["expected_p"],"same":same,"top_probability_delta":delta}));
    }
    let public_quality_pass = agrees == rows.len()
        && rows.len() >= 4
        && rows
            .iter()
            .all(|r| r["top_probability_delta"].as_f64().unwrap() < 0.02);
    save(
        out,
        &json!({"signature":sig,"passed":if quantized{rows.len()==9}else{public_quality_pass},"public_quality_pass":public_quality_pass,"quantized":quantized,"fp32_gate":baseline,"agrees":agrees,"count":rows.len(),"load_seconds":load,"wall_seconds":start.elapsed().as_secs_f64(),"rows":rows}),
    )?;
    if !quantized && !public_quality_pass {
        return Err("Ultra public output gate failed".into());
    }
    Ok(())
}
fn evaluate(model: &str, dir: &Path, gate: &Path, cases: &Path, out: &str) -> Result<(), Error> {
    let parity = read(gate)?;
    if parity["passed"] != true || parity["signature"] != signature(dir)? {
        return Err("matching successful independent check required".into());
    }
    let cases = read(cases)?;
    let start = Instant::now();
    let u = if model == "ultra" {
        Some(ultra::Ultra::load(dir)?)
    } else {
        None
    };
    let e = if model == "eg2" {
        Some(eg2::Eg2::load(dir)?)
    } else {
        None
    };
    let load = start.elapsed().as_secs_f64();
    let mut rows = Vec::new();
    let mut batches = Vec::new();
    let mut cache = HashMap::new();
    for source in ["parakeet", "phonon"] {
        for arm in ["single", "two"] {
            let group: Vec<_> = cases
                .as_array()
                .unwrap()
                .iter()
                .filter(|c| c["source"] == source && c["arm"] == arm)
                .collect();
            let now = Instant::now();
            let mut reqs = Vec::new();
            let mut rendered = Vec::new();
            for c in &group {
                let q = &c["candidate"]["question"];
                let ins = q["instructions"].as_str().unwrap();
                let meanings: Vec<_> = ["keep", "name", "abstain"]
                    .iter()
                    .map(|k| q["criteria"][k].as_str().unwrap().to_string())
                    .collect();
                let state = c["state"].as_str().unwrap();
                reqs.push((state.to_string(), ins.to_string(), meanings.clone()));
                rendered.push(eg2::Eg2::render(state, ins, &meanings));
            }
            let p = if let Some(m) = &u {
                m.batch(&reqs)?
            } else {
                let m = e.as_ref().unwrap();
                let missing: Vec<_> = rendered
                    .iter()
                    .flatten()
                    .filter(|t| !cache.contains_key(t.as_str()))
                    .cloned()
                    .collect();
                cache.extend(embeddings(m, &missing)?);
                let mut p = Vec::new();
                for r in &rendered {
                    let opts = r[2..].iter().map(|t| cache[t].clone()).collect::<Vec<_>>();
                    p.push(m.logits(&cache[&r[0]], &cache[&r[1]], &opts, "choice")?);
                }
                p
            };
            let seconds = now.elapsed().as_secs_f64();
            eprintln!(
                "{model} {source}/{arm}: {} decisions {seconds:.2}s",
                group.len()
            );
            batches.push(json!({"source":source,"arm":arm,"seconds":seconds,"candidates":group.len(),"cached_texts":cache.len()}));
            let evidence = read(format!(
                "tools/pair-router-probe/phone-household-q8/evidence/{source}-decision.json"
            ))?;
            let mut delivered = evidence["input"]
                .as_str()
                .ok_or("missing evidence input")?
                .to_string();
            let mut edits = Vec::new();
            for ((c, p), r) in group.iter().zip(p).zip(rendered) {
                let decision = ["keep", "name", "abstain"][best(&p)];
                if decision == "name" {
                    edits.push((
                        c["candidate"]["start"].as_u64().unwrap() as usize,
                        c["candidate"]["end"].as_u64().unwrap() as usize,
                        c["candidate"]["canonical"].as_str().unwrap().to_string(),
                        c["heard"].as_str().unwrap().to_string(),
                    ));
                }
                rows.push(json!({"case":c,"choice":decision,"probabilities":p,"rendered_eg2":if model=="eg2"{json!(r)}else{Value::Null}}));
            }
            edits.sort_by_key(|e| e.0);
            for (a, b, term, heard) in edits.into_iter().rev() {
                if delivered.get(a..b) != Some(heard.as_str()) {
                    return Err("frozen occurrence span mismatch".into());
                }
                delivered.replace_range(a..b, &term);
            }
            fs::write(format!("{out}.{source}-{arm}.txt"), delivered)?;
        }
    }
    save(
        out,
        &json!({"model":model,"signature":parity["signature"],"load_seconds":load,"wall_seconds":start.elapsed().as_secs_f64(),"batches":batches,"rows":rows}),
    )?;
    Ok(())
}
fn main() -> Result<(), Error> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    if a.len() == 3 && a[0] == "quantize-ultra" {
        return quant::export(Path::new(&a[1]), Path::new(&a[2]));
    }
    if a.len() < 5 {
        return Err("schema-probe eg2|ultra parity MODEL_DIR FIXTURES OUTPUT.json; schema-probe eg2|ultra evaluate MODEL_DIR PARITY.json CASES.json OUTPUT.json".into());
    }
    match (a[0].as_str(), a[1].as_str()) {
        ("eg2", "parity") => eg_parity(Path::new(&a[2]), Path::new(&a[3]), &a[4]),
        ("ultra", "parity") => ultra_parity(
            Path::new(&a[2]),
            Path::new(&a[3]),
            &a[4],
            a.get(5).map(Path::new),
        ),
        (m, "evaluate") if a.len() == 6 => evaluate(
            m,
            Path::new(&a[2]),
            Path::new(&a[3]),
            Path::new(&a[4]),
            &a[5],
        ),
        _ => Err("unknown probe mode".into()),
    }
}
