//! Corpus capture: bank the audio the engine actually heard, plus what it said.
//!
//! The point is that this rides the **live path**. Audio recorded by a separate
//! tool would come off the device at a different rate, through a different
//! downmix, and would not exercise `audio.rs`'s resampler — which on this
//! machine is a 44100→16000 linear interpolation with no anti-aliasing filter,
//! and is the leading suspect for the proper-noun errors worth measuring.
//!
//! Each dictation writes a pair:
//!
//! ```text
//! 2026-09-06T01-14-22.wav    mono f32 at the device rate, exactly what the
//!                            engine was handed before resampling
//! 2026-09-06T01-14-22.json   mode, raw text, injected text, timings, and a
//!                            null `reference` for you to fill in
//! ```
//!
//! Fill `reference` only for the entries that came out wrong. A corpus biased
//! toward failures is more useful per minute of your time than a balanced one,
//! and it is the only kind you will actually finish.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Serialize;

/// Written next to each clip. `reference` stays `null` until a human fills it;
/// `--bench` treats a filled reference as ground truth and ignores the rest.
#[derive(Debug, Serialize)]
pub struct Entry {
    /// UTC stamp, also the file stem.
    pub recorded: String,
    pub mode: String,
    pub engine: String,
    pub device_sample_rate: u32,
    pub duration_secs: f32,
    /// Straight off the engine, before the mode profile.
    pub raw: String,
    /// After the mode profile and `postprocess::fix_transcription` — what was
    /// actually typed.
    pub injected: String,
    pub transcribe_secs: f32,
    /// Hand-corrected truth. `None` until someone writes it.
    pub reference: Option<String>,
}

pub fn dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from(".local/share"))
        .join("transcrust")
        .join("corpus")
}

/// A filesystem- and sort-friendly **UTC** timestamp: `2026-09-06T17-19-20Z`.
///
/// Built from `SystemTime` rather than a date crate — this is the only place in
/// the tree that needs a clock, and local-time conversion is the only part that
/// would need one. The `Z` is there because the first test run produced a
/// 17:19 stem on a file the shell listed at 13:19, and an unlabelled stamp that
/// disagrees with `ls` is a trap. UTC also sorts correctly across a DST change,
/// which local time does not.
pub fn stamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (days, rem) = (secs / 86_400, secs % 86_400);
    let (hour, minute, second) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let (year, month, day) = civil_from_days(days as i64);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}-{minute:02}-{second:02}Z")
}

/// Days since the Unix epoch to a civil date. Howard Hinnant's `civil_from_days`,
/// which is exact for the whole range and needs no table.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Write one clip and its sidecar. Returns the WAV path.
///
/// Samples are stored as 32-bit float at the capture rate, not 16-bit PCM, so
/// the file is bit-identical to what the engine consumed. Quantising here would
/// mean a corpus that cannot reproduce a bug caused by the audio path.
pub fn save(samples: &[f32], sample_rate: u32, entry: &Entry) -> Result<PathBuf, String> {
    let dir = dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create {}: {e}", dir.display()))?;

    let wav_path = dir.join(format!("{}.wav", entry.recorded));
    write_wav(&wav_path, samples, sample_rate)?;

    let json_path = dir.join(format!("{}.json", entry.recorded));
    let json = serde_json::to_string_pretty(entry)
        .map_err(|e| format!("failed to serialise corpus entry: {e}"))?;
    std::fs::write(&json_path, json)
        .map_err(|e| format!("failed to write {}: {e}", json_path.display()))?;

    Ok(wav_path)
}

pub fn write_wav(path: &Path, samples: &[f32], sample_rate: u32) -> Result<(), String> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|e| format!("failed to create {}: {e}", path.display()))?;
    for sample in samples {
        writer
            .write_sample(*sample)
            .map_err(|e| format!("failed writing {}: {e}", path.display()))?;
    }
    writer
        .finalize()
        .map_err(|e| format!("failed to finalise {}: {e}", path.display()))
}

pub fn duration_of(samples: usize, sample_rate: u32) -> Duration {
    Duration::from_secs_f64(samples as f64 / sample_rate.max(1) as f64)
}

/// `MM:SS`, for a status line that updates once a second.
pub fn clock(elapsed: Duration) -> String {
    let total = elapsed.as_secs();
    format!("{:02}:{:02}", total / 60, total % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamps_sort_lexicographically_in_time_order() {
        let (y, m, d) = civil_from_days(20_337); // 2025-09-06
        assert_eq!((y, m, d), (2025, 9, 6));
        // The epoch itself, as a fixed point.
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(59), (1970, 3, 1));
    }

    #[test]
    fn stamp_is_filesystem_safe_and_fixed_width() {
        let s = stamp();
        assert_eq!(s.len(), 20, "{s}");
        assert!(!s.contains(':'), "colons break FAT and confuse scp: {s}");
        assert!(s.contains('T'));
        // The zone marker is load-bearing: without it the stem silently
        // disagrees with what `ls` prints.
        assert!(s.ends_with('Z'), "{s}");
    }

    #[test]
    fn leap_day_round_trips() {
        // 2024-02-29 is day 19782 since the epoch.
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
    }

    #[test]
    fn clock_counts_past_a_minute() {
        assert_eq!(clock(Duration::from_secs(0)), "00:00");
        assert_eq!(clock(Duration::from_secs(9)), "00:09");
        assert_eq!(clock(Duration::from_secs(75)), "01:15");
        assert_eq!(clock(Duration::from_secs(600)), "10:00");
    }

    #[test]
    fn duration_tracks_the_device_rate() {
        assert_eq!(duration_of(44_100, 44_100).as_secs_f32(), 1.0);
        assert_eq!(duration_of(22_050, 44_100).as_secs_f32(), 0.5);
        // A zero rate must not divide by zero.
        assert_eq!(duration_of(100, 0).as_secs_f32(), 100.0);
    }

    #[test]
    fn wav_round_trips_through_the_reader_the_engines_use() {
        let dir = std::env::temp_dir().join(format!("transcrust-corpus-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("probe.wav");
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 / 1000.0) - 0.5).collect();
        write_wav(&path, &samples, 44_100).unwrap();

        let (read, rate) = crate::audio::read_wav_mono(&path).unwrap();
        assert_eq!(rate, 44_100);
        assert_eq!(read.len(), samples.len());
        // f32 storage means bit-exact, not approximately equal.
        assert_eq!(read, samples);
        std::fs::remove_dir_all(&dir).ok();
    }
}
