use std::{path::Path, time::Instant};
use transcrust_phonon::{Phonon, Result};
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(2..=3).contains(&args.len()) {
        return Err("phonon-native MODEL_DIR WAV [TRACE_DIR]".into());
    }
    let (audio, rate) = transcrust_core::audio::read_wav_mono(Path::new(&args[1]))?;
    let audio = transcrust_core::audio::resample_to_16k(&audio, rate);
    let start = Instant::now();
    let engine = Phonon::load(Path::new(&args[0]))?;
    eprintln!(
        "native Rust Phonon 2.5 English loaded in {:.3}s",
        start.elapsed().as_secs_f64()
    );
    for pass in 1..=2 {
        let trace = if pass == 1 {
            args.get(2).map(Path::new)
        } else {
            None
        };
        let start = Instant::now();
        let result = engine.transcribe(&audio, trace)?;
        println!(
            "{}",
            serde_json::json!({"pass":pass,"audio_seconds":audio.len() as f64/16000.,"seconds":start.elapsed().as_secs_f64(),"text":result.text,"tokens":result.tokens.iter().map(|t|serde_json::json!({"id":t.id,"frame":t.frame,"duration":t.duration})).collect::<Vec<_>>()})
        );
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status
                .lines()
                .filter(|s| s.starts_with("VmRSS:") || s.starts_with("VmHWM:"))
            {
                eprintln!("{line}");
            }
        }
    }
    Ok(())
}
