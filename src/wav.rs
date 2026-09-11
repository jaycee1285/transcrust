//! `--wav`: run a file through the live engine seam and write a Markdown sidecar.
//!
//! This is the offline twin of the hotkey path. It reuses the one seam every
//! engine already speaks — `TranscriptionService::transcribe(observer, rx,
//! rate)` — and the one post-transcription pipeline (`mode::apply_profile`
//! then `postprocess::fix_transcription`), so a file transcript is the same
//! text the app would have injected. Nothing engine-specific lives here.
//!
//! The only thing this path adds is **windowing**, and it adds it because a
//! dictation clip is seconds long and a file is not. A 36-minute recording is
//! ~2.2M samples; handing that to a conformer encoder in one call is a
//! quadratic-attention allocation, not a transcription. So the file is cut into
//! windows first, and each window goes through the seam untouched.

use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::observe::Observer;
use crate::transcription::TranscriptionService;
use crate::{audio, corpus, mode, postprocess};

/// Windowing constants.
///
/// One constant serves both engines, for two unrelated reasons.
///
/// Parakeet is a transducer: it carries a decode loop, its RTF *worsens* with
/// window length (45s → 0.18, 90s → 0.20, 180s → 0.26, 300s → 0.32), and past
/// roughly five minutes its encoder does not slow down, it throws — ORT dies
/// broadcasting `2501 by 7501` in `self_attn`. Short windows are its safety.
///
/// Granite is CTC: pure forward pass, no decode loop, so its RTF is flat at
/// 0.06-0.07 from 45s to a 1200s single call. Length costs it nothing in time
/// and everything in memory — roughly 19 MB of peak RSS per extra second of
/// window. That, not speed, is what caps the window.
///
/// 60s is where those meet on a 16 GB laptop: ~1.8 GB peak, Parakeet still
/// near its best RTF and far under its cliff, and half the seams a 30s split
/// would cut.
const TARGET_SECS: f32 = 60.0;
/// How far either side of the target boundary we hunt for a pause.
const SEARCH_SECS: f32 = 8.0;
/// Energy is measured over frames this long, which is about a phoneme.
const FRAME_MS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    pub start: usize,
    pub end: usize,
}

/// Cut the clip at the quietest frame near each target boundary.
///
/// The alternative — fixed windows with overlap, stitched at the text level —
/// needs a dedup heuristic on every seam, and a wrong guess there silently
/// deletes real words. Cutting inside a pause costs nothing and is unambiguous:
/// the engine sees a whole utterance either side, and the join is a space.
///
/// The last window is allowed to run up to `TARGET + SEARCH` so a file never
/// ends with a two-second orphan.
pub fn plan_windows(sample_count: usize, rate: u32, energy: impl Fn(usize, usize) -> f32) -> Vec<Window> {
    let frame = (rate as usize * FRAME_MS) / 1000;
    let target = (TARGET_SECS * rate as f32) as usize;
    let search = (SEARCH_SECS * rate as f32) as usize;
    let mut windows = Vec::new();
    let mut start = 0usize;

    while start < sample_count {
        if sample_count - start <= target + search || frame == 0 {
            windows.push(Window { start, end: sample_count });
            break;
        }
        let lo = start + target - search;
        let hi = (start + target + search).min(sample_count);
        let mut best = lo;
        let mut best_energy = f32::MAX;
        let mut pos = lo;
        while pos + frame <= hi {
            let e = energy(pos, pos + frame);
            if e < best_energy {
                best_energy = e;
                best = pos;
            }
            pos += frame;
        }
        // Cut mid-pause rather than at its leading edge, so neither side gets a
        // clipped onset.
        let cut = (best + frame / 2).max(start + 1);
        windows.push(Window { start, end: cut });
        start = cut;
    }
    windows
}

fn rms_energy(samples: &[f32]) -> impl Fn(usize, usize) -> f32 + '_ {
    move |from, to| samples[from..to].iter().map(|s| s * s).sum::<f32>()
}

/// Transcribe each path and write `<stem>.md` beside it.
pub async fn run(paths: &[PathBuf], mode_filter: Option<&str>, config: &Config) {
    let observer = match Observer::new(config.observe.sample_chars, false, false) {
        Ok(observer) => observer,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    let modes = mode::discover_modes(config.model.path.as_deref());
    if modes.is_empty() {
        eprintln!("No supported ASR model found. Run --doctor to see the search path.");
        std::process::exit(1);
    }
    let chosen = match mode_filter {
        Some(needle) => modes
            .iter()
            .find(|m| m.label.to_lowercase().contains(&needle.to_lowercase())),
        None => modes.first(),
    };
    let Some(chosen) = chosen else {
        eprintln!("No installed mode matches {:?}. Installed:", mode_filter.unwrap_or(""));
        for m in &modes {
            eprintln!("  {}", m.label);
        }
        std::process::exit(1);
    };

    let service = match TranscriptionService::new(
        chosen.model.path.to_string_lossy().into_owned(),
        chosen.model.kind,
        // Nothing here is idle: the windows land back to back. A long timeout
        // just stops the worker unloading between files.
        config.observe.idle_timeout_secs.max(300),
    ) {
        Ok(service) => service,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    let mut failed = false;
    for path in paths {
        match transcribe_file(&service, &observer, chosen, path).await {
            Ok(out) => println!("{}", out.display()),
            Err(error) => {
                eprintln!("{}: {error}", path.display());
                failed = true;
            }
        }
    }
    if failed {
        std::process::exit(1);
    }
}

async fn transcribe_file(
    service: &TranscriptionService,
    observer: &Observer,
    chosen: &mode::Mode,
    path: &Path,
) -> Result<PathBuf, String> {
    let (samples, rate) = audio::read_wav_mono(path)?;
    let audio_secs = samples.len() as f64 / rate as f64;
    let windows = plan_windows(samples.len(), rate, rms_energy(&samples));

    eprintln!(
        "{} — {} of audio, {} window(s), {}",
        path.display(),
        corpus::clock(std::time::Duration::from_secs_f64(audio_secs)),
        windows.len(),
        chosen.label
    );

    let started = std::time::Instant::now();
    let mut paragraphs: Vec<String> = Vec::new();

    for (index, window) in windows.iter().enumerate() {
        let slice = &samples[window.start..window.end];
        let (tx, rx) = std::sync::mpsc::channel();
        // The live path streams chunks and the worker drains until the sender
        // drops. One chunk then drop is the same contract, and is what --bench
        // already does.
        tx.send(slice.to_vec())
            .map_err(|_| "engine receiver closed".to_string())?;
        drop(tx);

        let raw = service.transcribe(observer.clone(), rx, rate).await?;
        // Profile first, shared pipeline second — the post-processing contract.
        let shaped = mode::apply_profile(chosen.profile, &raw);
        let fixed = postprocess::fix_transcription(&shaped);
        if !fixed.trim().is_empty() {
            paragraphs.push(fixed.trim().to_string());
        }

        let done_secs = window.end as f64 / rate as f64;
        let elapsed = started.elapsed().as_secs_f64();
        eprint!(
            "\r  {:>3}/{:<3}  {} / {}  wall {}  RTF {:.2}   ",
            index + 1,
            windows.len(),
            corpus::clock(std::time::Duration::from_secs_f64(done_secs)),
            corpus::clock(std::time::Duration::from_secs_f64(audio_secs)),
            corpus::clock(started.elapsed()),
            elapsed / done_secs.max(0.001)
        );
        use std::io::Write;
        let _ = std::io::stderr().flush();
    }
    eprintln!();

    let wall = started.elapsed().as_secs_f64();
    let out = path.with_extension("md");
    let title = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "transcript".into());

    let mut md = String::new();
    md.push_str("---\n");
    md.push_str(&format!("source: {}\n", path.display()));
    md.push_str(&format!("engine: {}\n", chosen.label));
    md.push_str(&format!("audio_secs: {audio_secs:.1}\n"));
    md.push_str(&format!("transcribe_secs: {wall:.1}\n"));
    md.push_str(&format!("rtf: {:.3}\n", wall / audio_secs.max(0.001)));
    md.push_str(&format!("windows: {}\n", windows.len()));
    md.push_str(&format!("generated: {}\n", corpus::stamp()));
    md.push_str("---\n\n");
    md.push_str(&format!("# {title}\n\n"));
    md.push_str(&paragraphs.join("\n\n"));
    md.push('\n');

    std::fs::write(&out, md).map_err(|e| format!("failed to write {}: {e}", out.display()))?;
    eprintln!(
        "  {} of audio in {} (RTF {:.2})",
        corpus::clock(std::time::Duration::from_secs_f64(audio_secs)),
        corpus::clock(started.elapsed()),
        wall / audio_secs.max(0.001)
    );
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A clip shorter than one window is never cut.
    #[test]
    fn short_clip_is_a_single_window() {
        let rate = 16_000;
        let samples = vec![0.5f32; rate * 10];
        let windows = plan_windows(samples.len(), rate as u32, rms_energy(&samples));
        assert_eq!(windows, vec![Window { start: 0, end: samples.len() }]);
    }

    /// The cut lands in the silence, not on the target boundary.
    #[test]
    fn cuts_in_the_quietest_place_near_the_target() {
        let rate = 16_000u32;
        // Loud audio with a half-second hole right on the target boundary.
        // Derived from the constants rather than hardcoded, so retuning the
        // window moves the fixture with it instead of failing the test.
        let total = (TARGET_SECS * 2.0) as usize * rate as usize;
        let mut samples = vec![0.5f32; total];
        let hole = (TARGET_SECS * rate as f32) as usize;
        for sample in &mut samples[hole..hole + rate as usize / 2] {
            *sample = 0.0;
        }
        let windows = plan_windows(samples.len(), rate, rms_energy(&samples));
        assert_eq!(windows.len(), 2);
        let cut = windows[0].end;
        assert!(
            cut >= hole && cut <= hole + rate as usize / 2,
            "cut at {cut} is outside the silence at {hole}"
        );
        assert_eq!(windows[1], Window { start: cut, end: samples.len() });
    }

    /// Windows tile the clip exactly: no sample is dropped or heard twice.
    #[test]
    fn windows_tile_the_clip_without_gaps_or_overlap() {
        let rate = 16_000u32;
        let samples: Vec<f32> = (0..rate as usize * 600)
            .map(|i| ((i % 977) as f32 / 977.0) - 0.5)
            .collect();
        let windows = plan_windows(samples.len(), rate, rms_energy(&samples));
        assert!(windows.len() > 10);
        assert_eq!(windows[0].start, 0);
        assert_eq!(windows.last().unwrap().end, samples.len());
        for pair in windows.windows(2) {
            assert_eq!(pair[0].end, pair[1].start);
            assert!(pair[0].end > pair[0].start);
        }
    }
}
