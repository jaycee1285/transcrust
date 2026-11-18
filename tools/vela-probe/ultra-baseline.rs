//! Replay exactly the Vela probe's recorded states through the current native Ultra model.
use decision_tensor::Error;
use serde_json::{json, Value};
use std::{fs, path::Path, time::Instant};
#[allow(dead_code)]
use transcrust_decision::decision_tensor;
#[allow(dead_code)]
use transcrust_decision::ultra;
fn main() -> Result<(), Error> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("ultra-text-baseline VELA_RESULTS.json OUT.json".into());
    }
    let source: Value = serde_json::from_slice(&fs::read(&args[0])?)?;
    let records = source["rows"].as_array().ok_or("missing Vela rows")?;
    if source["scope"] != "stripped" {
        return Err("baseline expects the stripped comparison".into());
    }
    let dir = transcrust_decision::model_path(transcrust_decision::Backend::Ultra);
    let start = Instant::now();
    let model = ultra::Ultra::load(&dir)?;
    let load = start.elapsed().as_secs_f64();
    let mut rows = Vec::new();
    let mut batches = Vec::new();
    for chunk in records.chunks(8) {
        let requests = chunk
            .iter()
            .map(|r| {
                (
                    r["state"].as_str().unwrap().to_string(),
                    r["question"]["instructions"].as_str().unwrap().to_string(),
                    ["keep", "name", "abstain"]
                        .iter()
                        .map(|key| r["question"]["criteria"][key].as_str().unwrap().to_string())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        let now = Instant::now();
        let probabilities = model.batch(&requests)?;
        let seconds = now.elapsed().as_secs_f64();
        eprintln!("Ultra baseline: {} rows, {seconds:.3}s", chunk.len());
        batches.push(json!({"rows":chunk.len(),"seconds":seconds}));
        for (r, p) in chunk.iter().zip(probabilities) {
            let i = decision_tensor::best(&p);
            let choice = ["keep", "name", "abstain"][i];
            rows.push(json!({"case":r["case"],"state":r["state"],"question":r["question"],"choice":choice,"probabilities":p}));
        }
    }
    let mut metrics = Vec::new();
    for split in ["dev", "eval", "diagnostic"] {
        for arm in ["single", "two"] {
            let r = rows
                .iter()
                .filter(|r| r["case"]["split"] == split && r["case"]["arm"] == arm)
                .collect::<Vec<_>>();
            metrics.push(json!({"split":split,"arm":arm,"n":r.len(),"repair_total":r.iter().filter(|r|r["case"]["want"]=="name").count(),"repairs":r.iter().filter(|r|r["case"]["want"]=="name"&&r["choice"]=="name").count(),"preserve_total":r.iter().filter(|r|r["case"]["want"]!="name").count(),"unwanted_edits":r.iter().filter(|r|r["case"]["want"]!="name"&&r["choice"]=="name").count(),"abstentions":r.iter().filter(|r|r["choice"]=="abstain").count()}));
        }
    }
    let out = json!({"model":"current UltraBERT INT4 stripped","input_sha256":decision_tensor::file_hash(Path::new(&args[0]))?,"weights_sha256":decision_tensor::file_hash(&dir.join("weights.q4"))?,"load_seconds":load,"wall_seconds":start.elapsed().as_secs_f64(),"batches":batches,"rows":rows,"metrics":metrics});
    fs::write(&args[1], serde_json::to_vec_pretty(&out)?)?;
    Ok(())
}
