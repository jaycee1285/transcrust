//! Context ablation using the unchanged production D1 encoder and model runner.
use ort::session::RunOptions;
use serde_json::{json, Value};
use std::{fs, path::PathBuf, time::Instant};
use transcrust_decision::d1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: d1-context-probe CASES.json RESULTS.json".into());
    }
    let cases: Vec<Value> = serde_json::from_slice(&fs::read(&args[0])?)?;
    let baseline: Value = serde_json::from_slice(&fs::read(
        "tools/pair-router-probe/bluey-q8-expanded-glossary/results.json",
    )?)?;
    let directory = std::env::var_os("TRANSCRUST_DEC_MODEL")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::data_local_dir()
                .unwrap()
                .join("transcrust/models/d1-omni-600M-q8")
        });
    let started = Instant::now();
    let mut model = d1::Model::load(&directory)?;
    let load_secs = started.elapsed().as_secs_f64();
    let mut groups: Vec<Vec<Value>> = Vec::new();
    for source in ["parakeet", "phonon"] {
        for arm in ["single", "two"] {
            groups.push(
                cases
                    .iter()
                    .filter(|c| c["source"] == source && c["arm"] == arm)
                    .cloned()
                    .collect(),
            );
        }
    }
    let synthetic: Vec<_> = cases
        .into_iter()
        .filter(|c| c["source"] == "synthetic")
        .collect();
    groups.extend(synthetic.chunks(8).map(|chunk| chunk.to_vec()));
    let mut records = Vec::new();
    let mut batches = Vec::new();
    let run = RunOptions::new()?;
    let mut repeat_delta = 0.0f64;
    for (batch, group) in groups.iter().enumerate() {
        let questions: Vec<_> = group
            .iter()
            .map(|c| {
                let q = if c["candidate"].is_null() {
                    &baseline["cases"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|r| r["case"]["id"] == c["id"] && r["arm"] == c["arm"])
                        .unwrap()["questions"]["decision"]
                } else {
                    &c["candidate"]["question"]
                };
                d1::Question {
                    state: c["state"].as_str().unwrap().into(),
                    instructions: q["instructions"].as_str().unwrap().into(),
                    options: ["keep", "name", "abstain"]
                        .map(|k| q["criteria"][k].as_str().unwrap().into()),
                }
            })
            .collect();
        if batch == 0 {
            model.infer(&questions[..1], &run)?;
        }
        let (scores, ms) = model.infer(&questions, &run)?;
        if batch == 0 {
            let (again, _) = model.infer(&questions, &run)?;
            for (a, b) in scores.iter().zip(again) {
                for i in 0..3 {
                    repeat_delta =
                        repeat_delta.max((a.probabilities[i] - b.probabilities[i]).abs());
                }
            }
            assert!(repeat_delta < 1e-6, "identical batch repeat drift");
        }
        eprintln!("D1: {} rows, {ms:.0}ms, batch {}", group.len(), batch + 1);
        batches.push(json!({"batch":batch,"source":group[0]["source"],"arm":group[0]["arm"],"rows":group.len(),"graph_ms":ms}));
        for ((case, question), score) in group.iter().zip(questions).zip(scores) {
            let mut best = 0;
            for i in 1..3 {
                if score.probabilities[i] > score.probabilities[best] {
                    best = i;
                }
            }
            records.push(json!({"case":case,"question":{"instructions":question.instructions,"options":question.options},"choice":(["keep","name","abstain"][best]),"probabilities":score.probabilities,"token_count":score.token_count,"batch":batch}));
        }
    }
    let mut metrics = Vec::new();
    for split in ["dev", "eval", "diagnostic"] {
        for arm in ["single", "two"] {
            let rows: Vec<_> = records
                .iter()
                .filter(|r| r["case"]["split"] == split && r["case"]["arm"] == arm)
                .collect();
            metrics.push(json!({"split":split,"arm":arm,"n":rows.len(),
            "repair_total":rows.iter().filter(|r|r["case"]["want"]=="name").count(),
            "repairs":rows.iter().filter(|r|r["case"]["want"]=="name"&&r["choice"]=="name").count(),
            "preserve_total":rows.iter().filter(|r|r["case"]["want"]!="name").count(),
            "unwanted_edits":rows.iter().filter(|r|r["case"]["want"]!="name"&&r["choice"]=="name").count(),
            "abstentions":rows.iter().filter(|r|r["choice"]=="abstain").count()}));
        }
    }
    let mut household = Vec::new();
    for source in ["parakeet", "phonon"] {
        for arm in ["single", "two"] {
            let original: Value = serde_json::from_slice(&fs::read(format!(
                "tools/pair-router-probe/phone-household-q8/evidence/{source}-decision.json"
            ))?)?;
            let mut text = original["input"].as_str().unwrap().to_owned();
            let mut edits: Vec<_> = records
                .iter()
                .filter(|r| {
                    r["case"]["source"] == source
                        && r["case"]["arm"] == arm
                        && r["choice"] == "name"
                })
                .collect();
            edits.sort_by_key(|r| {
                std::cmp::Reverse(r["case"]["candidate"]["start"].as_u64().unwrap())
            });
            for r in &edits {
                let c = &r["case"]["candidate"];
                text.replace_range(
                    c["start"].as_u64().unwrap() as usize..c["end"].as_u64().unwrap() as usize,
                    c["canonical"].as_str().unwrap(),
                );
            }
            household
                .push(json!({"source":source,"arm":arm,"delivered":text,"changes":edits.len()}));
        }
    }
    fs::write(
        &args[1],
        serde_json::to_vec_pretty(
            &json!({"model":"native production D1 Q8","load_secs":load_secs,"repeat_max_probability_delta":repeat_delta,"batches":batches,"metrics":metrics,"household":household,"records":records}),
        )?,
    )?;
    println!("Results: {}", args[1]);
    Ok(())
}
