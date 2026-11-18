//! Local-only replay: identical float WAVs through Rust and an external C oracle.
use std::{path::Path, process::Command, time::Instant};
use transcrust_phonon::{segment::Segmenter, Phonon, Result};

fn wav(path: &Path, audio: &[f32]) -> Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut out = hound::WavWriter::create(path, spec)?;
    for &s in audio {
        out.write_sample(s)?;
    }
    out.finalize()?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(5..=6).contains(&args.len()) {
        return Err(
            "replay NATIVE_MODEL WAV OUTPUT_DIR C_BINARY C_MODEL [live|twelve|twenty]".into(),
        );
    }
    if args
        .get(5)
        .is_some_and(|p| !["live", "twelve", "twenty"].contains(&p.as_str()))
    {
        return Err("unknown replay window plan".into());
    }
    let (input, rate) = transcrust_core::audio::read_wav_mono(Path::new(&args[1]))?;
    let mut resampler = transcrust_core::audio::StreamingResampler::new(rate, 16000);
    let mut audio = resampler.push(&input);
    audio.extend(resampler.finish());
    let out = Path::new(&args[2]);
    std::fs::create_dir_all(out)?;
    wav(&out.join("full-16k.wav"), &audio)?;
    let engine = Phonon::load(Path::new(&args[0]))?;
    let mut segmenter = Segmenter::default();
    let mut pieces = Vec::new();
    for frame in audio.chunks(480) {
        if let Some(piece) = segmenter.push(frame) {
            pieces.push(piece);
        }
    }
    pieces.push(segmenter.finish());
    let mut twelve = Segmenter::with_window_seconds(12)?;
    let mut twelve_pieces = Vec::new();
    for frame in audio.chunks(480) {
        if let Some(piece) = twelve.push(frame) {
            twelve_pieces.push(piece);
        }
    }
    twelve_pieces.push(twelve.finish());
    let mut rows = Vec::new();
    for (plan, windows) in [
        ("live", pieces),
        ("twelve", twelve_pieces),
        (
            "twenty",
            audio.chunks(20 * 16000).map(<[f32]>::to_vec).collect(),
        ),
    ] {
        if args.get(5).is_some_and(|p| p != plan) {
            continue;
        }
        let mut offset = 0usize;
        for (i, piece) in windows.iter().enumerate() {
            let path = out.join(format!("{plan}-{i}.wav"));
            wav(&path, piece)?;
            let start = Instant::now();
            let rust = engine.transcribe(piece, None)?;
            let seconds = start.elapsed().as_secs_f64();
            let c = Command::new(&args[3])
                .args(["--quiet", "--exact", "--threads", "4"])
                .arg(&args[4])
                .arg(&path)
                .env("P2_CACHE_CODES", "0")
                .env("P2_SEGMENT", "0")
                .output()?;
            std::fs::write(out.join(format!("{plan}-{i}-c.log")), &c.stderr)?;
            if !c.status.success() {
                return Err(format!("C failed: {}", c.status).into());
            }
            let c_text = String::from_utf8(c.stdout)?.trim().to_string();
            let row = serde_json::json!({"plan":plan,"window":i,
                "start":offset as f64/16000.,"end":(offset+piece.len()) as f64/16000.,
                "rust":rust.text,"c":c_text,"equal":rust.text == c_text,"rust_seconds":seconds});
            println!("{row}");
            rows.push(row);
            offset += piece.len();
        }
    }
    std::fs::write(
        out.join("comparison.json"),
        serde_json::to_vec_pretty(&rows)?,
    )?;
    Ok(())
}
