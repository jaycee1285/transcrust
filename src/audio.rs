use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex, mpsc};

pub struct AudioCapture {
    stream: cpal::Stream,
    chunk_tx: Arc<Mutex<Option<mpsc::Sender<Vec<f32>>>>>,
    sample_rate: u32,
}

impl AudioCapture {
    pub fn new(device_name: Option<&str>) -> Result<Self, String> {
        let host = cpal::default_host();

        let device = if let Some(name) = device_name {
            host.input_devices()
                .map_err(|e| format!("Cannot enumerate audio devices: {e}"))?
                .find(|d| d.name().ok().as_deref() == Some(name))
                .ok_or_else(|| format!("Audio device not found: {name}"))?
        } else {
            host.default_input_device()
                .ok_or("No default audio input device")?
        };

        let supported = device
            .default_input_config()
            .map_err(|e| format!("No supported input config: {e}"))?;

        let sample_rate = supported.sample_rate().0;
        let channels = supported.channels();

        let chunk_tx: Arc<Mutex<Option<mpsc::Sender<Vec<f32>>>>> = Arc::new(Mutex::new(None));
        let tx_clone = chunk_tx.clone();
        let ch = channels as usize;

        let config = cpal::StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = match supported.sample_format() {
            cpal::SampleFormat::F32 => {
                device.build_input_stream(
                    &config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mono: Vec<f32> = if ch > 1 {
                            data.chunks(ch).map(|frame| frame[0]).collect()
                        } else {
                            data.to_vec()
                        };
                        if let Some(tx) = tx_clone.lock().unwrap().as_ref() {
                            let _ = tx.send(mono);
                        }
                    },
                    |err| eprintln!("Audio stream error: {err}"),
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                device.build_input_stream(
                    &config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let mono: Vec<f32> = if ch > 1 {
                            data.chunks(ch)
                                .map(|frame| frame[0] as f32 / i16::MAX as f32)
                                .collect()
                        } else {
                            data.iter().map(|&s| s as f32 / i16::MAX as f32).collect()
                        };
                        if let Some(tx) = tx_clone.lock().unwrap().as_ref() {
                            let _ = tx.send(mono);
                        }
                    },
                    |err| eprintln!("Audio stream error: {err}"),
                    None,
                )
            }
            fmt => return Err(format!("Unsupported sample format: {fmt:?}")),
        }
        .map_err(|e| format!("Failed to build input stream: {e}"))?;

        Ok(Self {
            stream,
            chunk_tx,
            sample_rate,
        })
    }

    /// Start recording. Returns a receiver that yields mono f32 audio chunks.
    /// The channel closes when `stop_recording()` is called.
    pub fn start_recording(&self) -> mpsc::Receiver<Vec<f32>> {
        let (tx, rx) = mpsc::channel();
        *self.chunk_tx.lock().unwrap() = Some(tx);
        self.stream.play().unwrap();
        rx
    }

    /// Stop recording. Drops the sender, closing the channel.
    pub fn stop_recording(&self) {
        self.stream.pause().unwrap();
        *self.chunk_tx.lock().unwrap() = None;
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn list_devices() {
        let host = cpal::default_host();
        println!("Audio input devices:");
        if let Ok(devices) = host.input_devices() {
            for device in devices {
                let name = device.name().unwrap_or_else(|_| "unknown".into());
                let config = device
                    .default_input_config()
                    .map(|c| format!("{}Hz {}ch", c.sample_rate().0, c.channels()))
                    .unwrap_or_else(|_| "no config".into());
                println!("  {name} ({config})");
            }
        }
    }
}

/// Name, sample rate and channel count of the device dictation would use.
///
/// Same selection `AudioCapture::new` performs, so `--doctor` reports the device
/// that will actually be recorded from rather than the first one enumerated.
pub fn default_input_summary(device_name: Option<&str>) -> Result<(String, u32, u16), String> {
    let host = cpal::default_host();
    let device = match device_name {
        Some(name) => host
            .input_devices()
            .map_err(|e| format!("cannot enumerate audio devices: {e}"))?
            .find(|d| d.name().ok().as_deref() == Some(name))
            .ok_or_else(|| format!("audio device not found: {name}"))?,
        None => host
            .default_input_device()
            .ok_or("no default audio input device")?,
    };
    let name = device.name().unwrap_or_else(|_| "unknown".into());
    let config = device
        .default_input_config()
        .map_err(|e| format!("no supported input config: {e}"))?;
    Ok((name, config.sample_rate().0, config.channels()))
}

/// Measure the resampler's actual attenuation at frequencies that would alias.
///
/// Reported rather than asserted, because the defect this exists to catch was
/// invisible for months: linear interpolation passed 12 kHz at −2.1 dB and
/// folded it onto 4 kHz, mid speech band. A number in `--doctor` is how that
/// gets noticed the next time.
///
/// Returns `(input_hz, attenuation_db, folds_onto_hz)` for each probe above the
/// output Nyquist. An empty result means no probe frequency aliases, which is
/// the case when the device already runs at the target rate.
pub fn resampler_response(from_rate: u32, to_rate: u32) -> Vec<(f64, f64, f64)> {
    if from_rate == to_rate {
        return Vec::new();
    }
    let nyquist = to_rate as f64 / 2.0;
    let mut out = Vec::new();
    for probe in [10_000.0f64, 12_000.0, 14_000.0] {
        if probe <= nyquist || probe >= from_rate as f64 / 2.0 {
            continue;
        }
        let input: Vec<f32> = (0..from_rate as usize)
            .map(|i| {
                (2.0 * std::f64::consts::PI * probe * i as f64 / from_rate as f64).sin() as f32
            })
            .collect();
        let resampled = resample(&input, from_rate, to_rate);
        let ratio = rms_body(&resampled) / rms_body(&input).max(f64::EPSILON);
        // Where the tone lands once the rate is decimated: reflect it about
        // Nyquist until it falls inside the band.
        let mut folded = probe % to_rate as f64;
        if folded > nyquist {
            folded = to_rate as f64 - folded;
        }
        out.push((probe, 20.0 * ratio.max(1e-12).log10(), folded));
    }
    out
}

/// RMS with the kernel-length edges skipped, where a half-covered window rolls
/// the amplitude off legitimately.
fn rms_body(x: &[f32]) -> f64 {
    if x.is_empty() {
        return 0.0;
    }
    let skip = (x.len() / 10).min(2000);
    if x.len() <= skip * 2 {
        return 0.0;
    }
    let body = &x[skip..x.len() - skip];
    (body.iter().map(|&v| (v as f64) * (v as f64)).sum::<f64>() / body.len() as f64).sqrt()
}

/// Resample f32 audio to 16kHz, returning f32 for batch transcription backends.
pub fn resample_to_16k(input: &[f32], from_rate: u32) -> Vec<f32> {
    if from_rate == 16000 {
        input.to_vec()
    } else {
        resample(input, from_rate, 16000)
    }
}

/// Decode a WAV to mono `f32` at its own sample rate.
///
/// Only the offline paths use this — `--bench <WAV>` and
/// `--parakeet-direct <DIR> <WAV>`. The rate comes back untouched so each engine resamples it
/// with the same function the live capture path uses, which is the whole point
/// of benchmarking through a file: everything downstream of here is identical
/// to a real dictation.
pub fn read_wav_mono(path: &std::path::Path) -> Result<(Vec<f32>, u32), String> {
    let mut reader = hound::WavReader::open(path)
        .map_err(|e| format!("failed to open {}: {e}", path.display()))?;
    let spec = reader.spec();
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?,
        hound::SampleFormat::Int => {
            let scale = 1.0 / (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|sample| sample.map(|value| value as f32 * scale))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("failed to read {}: {e}", path.display()))?
        }
    };
    let mono: Vec<f32> = if spec.channels > 1 {
        let channels = spec.channels as usize;
        samples
            .chunks(channels)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        samples
    };
    Ok((mono, spec.sample_rate))
}

/// Zero crossings of the sinc kernel kept either side of centre. Sets the
/// transition width and stopband depth; 16 with a Blackman window puts the
/// stopband near -70 dB, far below anything the mel frontend can see.
const SINC_ZERO_CROSSINGS: f64 = 16.0;

/// Cutoff as a fraction of the *lower* Nyquist. 0.45 leaves a transition band
/// between 7.2 kHz and 8 kHz when decimating to 16 kHz — above the speech
/// energy that matters and below the fold point.
const CUTOFF_FRACTION: f64 = 0.45;

/// Band-limited resampling by windowed-sinc interpolation.
///
/// **The lowpass is the point, not the interpolation.** The previous
/// implementation was linear interpolation with no anti-aliasing filter, which
/// on this machine's 44100 Hz capture attenuated 12 kHz by only 2.2 dB and
/// folded it onto 4 kHz — the middle of the speech band, exactly where sibilant
/// and plosive-burst cues live. Linear interpolation *is* a filter, just a
/// terrible one: a two-tap average whose first null sits at the input rate.
///
/// One pass does both jobs. Each output sample is a sum of input samples
/// weighted by a sinc kernel centred on the fractional source position and cut
/// off below the output Nyquist, so the signal is band-limited before it is
/// ever decimated. Upsampling keeps the input's own Nyquist as the cutoff,
/// which is why the fraction is taken against whichever rate is lower.
fn resample(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate || input.is_empty() {
        return input.to_vec();
    }

    let from = from_rate as f64;
    let to = to_rate as f64;
    let step = from / to;
    let output_len = (input.len() as f64 / step) as usize;

    // Cutoff in cycles per *input* sample.
    let cutoff = CUTOFF_FRACTION * from.min(to) / from;
    // Kernel half-width in input samples: enough to hold the requested number
    // of zero crossings, which get wider as the cutoff drops.
    let half_width = (SINC_ZERO_CROSSINGS / (2.0 * cutoff)).ceil() as isize;

    // Polyphase: the fractional part of the source position cycles through a
    // fixed set of phases, so every kernel the loop will ever need can be built
    // once. Without this the inner loop calls `sin` per tap — ~1.7 billion of
    // them for 18 minutes of audio, which measured 38 s of pure filter time and
    // would put ~0.36 s of latency on a ten-second dictation.
    let phases = (to_rate / gcd(from_rate, to_rate)) as usize;
    let taps = (2 * half_width) as usize;
    let mut bank = vec![0.0f64; phases * taps];
    let mut gains = vec![0.0f64; phases];
    for (p, gain) in gains.iter_mut().enumerate() {
        let frac = p as f64 / phases as f64;
        let mut sum = 0.0;
        for k in 0..taps {
            // Tap k sits at input offset (k - half_width + 1) from the floor of
            // the source position, so its distance from the true centre is that
            // offset minus the fractional part.
            let t = (k as isize - half_width + 1) as f64 - frac;
            let w = sinc(2.0 * cutoff * t) * blackman(t, half_width as f64);
            bank[p * taps + k] = w;
            sum += w;
        }
        *gain = sum;
    }

    let mut output = Vec::with_capacity(output_len);
    for i in 0..output_len {
        let centre = i as f64 * step;
        let base = centre.floor() as isize;
        // Recovering the phase from the position keeps this exact for any rate
        // pair, including ones where `step` is irrational in binary.
        let p = (((centre - base as f64) * phases as f64).round() as usize) % phases;
        let kernel = &bank[p * taps..(p + 1) * taps];

        let mut acc = 0.0f64;
        for (k, &w) in kernel.iter().enumerate() {
            let j = base - half_width + 1 + k as isize;
            // Outside the clip is silence rather than a clamped edge sample,
            // which would smear a DC step across the first and last few ms.
            if j >= 0 && (j as usize) < input.len() {
                acc += input[j as usize] as f64 * w;
            }
        }
        // Normalising by the window sum holds unity gain wherever the
        // fractional centre lands between taps.
        let gain = gains[p];
        output.push(if gain.abs() > f64::EPSILON {
            (acc / gain) as f32
        } else {
            0.0
        });
    }

    output
}

/// Greatest common divisor, for reducing a rate pair to its phase count.
fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Normalised sinc, `sin(pi x) / (pi x)`, defined as 1 at the origin.
fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-9 {
        1.0
    } else {
        let pix = std::f64::consts::PI * x;
        pix.sin() / pix
    }
}

/// Blackman window over `[-half_width, half_width]`, zero outside.
///
/// Blackman rather than Hann because the extra stopband depth is nearly free
/// here and aliasing that folds into the speech band is the whole defect being
/// fixed.
fn blackman(t: f64, half_width: f64) -> f64 {
    if t.abs() > half_width {
        return 0.0;
    }
    let x = std::f64::consts::PI * (t + half_width) / half_width;
    0.42 - 0.5 * (x).cos() + 0.08 * (2.0 * x).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(freq_hz: f64, rate: u32, secs: f64) -> Vec<f32> {
        let n = (rate as f64 * secs) as usize;
        (0..n)
            .map(|i| {
                let t = i as f64 / rate as f64;
                (2.0 * std::f64::consts::PI * freq_hz * t).sin() as f32
            })
            .collect()
    }

    fn rms(x: &[f32]) -> f64 {
        if x.is_empty() {
            return 0.0;
        }
        // Skip the kernel-length edges, where the half-covered window rolls the
        // amplitude off legitimately and would drag the average down.
        let skip = (x.len() / 10).min(2000);
        let body = &x[skip..x.len() - skip];
        (body.iter().map(|&v| (v as f64) * (v as f64)).sum::<f64>() / body.len() as f64).sqrt()
    }

    /// Speech-band content must come through at full level.
    #[test]
    fn passband_tone_survives_decimation() {
        let input = tone(1000.0, 44_100, 1.0);
        let out = resample(&input, 44_100, 16_000);
        let ratio = rms(&out) / rms(&input);
        assert!(
            (0.95..=1.05).contains(&ratio),
            "1 kHz should pass at unity, got {ratio:.3}"
        );
    }

    /// The defect this replaced. 12 kHz at 44100 folds onto 4 kHz when
    /// decimated to 16 kHz — the middle of the speech band. Linear
    /// interpolation attenuated it by 2.2 dB, so the alias arrived at roughly
    /// three quarters amplitude. It has to be gone, not merely reduced.
    #[test]
    fn alias_band_is_rejected() {
        for freq in [10_000.0, 12_000.0, 14_000.0] {
            let input = tone(freq, 44_100, 1.0);
            let out = resample(&input, 44_100, 16_000);
            let ratio = rms(&out) / rms(&input);
            assert!(
                ratio < 0.01,
                "{freq} Hz must be at least 40 dB down after decimation, got {ratio:.4}"
            );
        }
    }

    /// A rate that needs no conversion must not be filtered at all.
    #[test]
    fn identity_when_rates_match() {
        let input = tone(1000.0, 16_000, 0.1);
        assert_eq!(resample(&input, 16_000, 16_000), input);
        assert_eq!(resample_to_16k(&input, 16_000), input);
    }

    #[test]
    fn output_length_tracks_the_ratio() {
        let input = tone(440.0, 44_100, 1.0);
        let out = resample(&input, 44_100, 16_000);
        let expected = (44_100.0f64 / (44_100.0 / 16_000.0)) as usize;
        assert!(
            out.len().abs_diff(expected) <= 1,
            "expected ~{expected} samples, got {}",
            out.len()
        );
    }

    /// Upsampling takes its cutoff from the input's Nyquist, so a tone well
    /// inside the source band must survive going the other way too.
    #[test]
    fn upsampling_preserves_the_passband() {
        let input = tone(1000.0, 16_000, 1.0);
        let out = resample(&input, 16_000, 44_100);
        let ratio = rms(&out) / rms(&input);
        assert!(
            (0.95..=1.05).contains(&ratio),
            "1 kHz should survive upsampling at unity, got {ratio:.3}"
        );
    }

    #[test]
    fn empty_input_is_empty_output() {
        assert!(resample(&[], 44_100, 16_000).is_empty());
    }

    /// Not an assertion about speed so much as a guard on the shape of the
    /// cost: this runs on the dictation path, so a ten-second clip must
    /// resample in single-digit milliseconds, not hundreds.
    #[test]
    fn resampling_a_dictation_clip_is_cheap() {
        let input = tone(1000.0, 44_100, 10.0);
        // Best of three, not a single run. This asserts a wall-clock bound, and
        // a single sample fails whenever the machine is busy — it did, once,
        // while an LLM was generating in another process. The best run still
        // catches the regression this guards against: the pre-polyphase form
        // called `sin` once per tap and was an order of magnitude slower.
        let mut best = std::time::Duration::MAX;
        let mut out_len = 0;
        for _ in 0..3 {
            let start = std::time::Instant::now();
            let out = resample(&input, 44_100, 16_000);
            best = best.min(start.elapsed());
            out_len = out.len();
        }
        assert_eq!(out_len, 160_000);
        println!("10s of 44.1kHz -> 16kHz took {best:?} (best of 3)");
        assert!(
            best < std::time::Duration::from_millis(150),
            "resampling 10s took {best:?}; the polyphase bank should keep this in single-digit ms"
        );
    }

    /// `--doctor` reports these numbers, so the fold arithmetic has to be right
    /// or the report is confidently wrong about where the energy lands.
    #[test]
    fn resampler_response_reports_fold_targets_and_rejection() {
        let response = resampler_response(44_100, 16_000);
        assert_eq!(response.len(), 3, "expected probes at 10, 12 and 14 kHz");
        let folds: Vec<f64> = response.iter().map(|(_, _, f)| f.round()).collect();
        assert_eq!(folds, vec![6000.0, 4000.0, 2000.0]);
        for (probe, db, _) in &response {
            assert!(
                *db <= -40.0,
                "{probe} Hz should be rejected, reported {db:.1} dB"
            );
        }
    }

    /// A device already at the target rate is not resampled, so there is
    /// nothing to report and `--doctor` must not invent a row.
    #[test]
    fn resampler_response_is_empty_when_no_conversion_happens() {
        assert!(resampler_response(16_000, 16_000).is_empty());
    }
}
