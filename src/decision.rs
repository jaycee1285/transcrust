//! Desktop delivery audit; inference lives in the shared decision crate.
use crate::observe::Observer;
use serde_json::{json, Value};
pub use transcrust_decision::{Backend, Decision};

pub fn log_delivery(
    observer: &Observer,
    raw: &str,
    delivered: &str,
    asr_secs: f32,
    wait_secs: f32,
    mut audit: Value,
) {
    use std::io::Write;
    audit["raw"] = json!(raw);
    audit["delivered"] = json!(delivered);
    audit["asr_secs"] = json!(asr_secs);
    audit["wait_secs"] = json!(wait_secs);
    audit["timestamp"] = json!(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs());
    let path = observer.log_path().with_file_name("decisions.jsonl");
    if let Err(e) = (|| -> Result<(), String> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        writeln!(file, "{audit}").map_err(|e| e.to_string())
    })() {
        observer.error("dec.log", &e);
    }
    observer.phase(
        "timing",
        &format!("ASR {asr_secs:.2}s; total wait to typing {wait_secs:.2}s"),
    );
}
