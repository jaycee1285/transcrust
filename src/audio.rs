use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};

pub struct AudioCapture {
    stream: cpal::Stream,
    chunk_tx: Arc<Mutex<Option<mpsc::Sender<Vec<f32>>>>>,
    sample_rate: u32,
    sample_count: Arc<AtomicUsize>,
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
        let sample_count = Arc::new(AtomicUsize::new(0));
        let count_clone = sample_count.clone();
        let ch = channels as usize;

        let config = cpal::StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = match supported.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mono: Vec<f32> = if ch > 1 {
                        data.chunks(ch).map(|frame| frame[0]).collect()
                    } else {
                        data.to_vec()
                    };
                    if let Some(tx) = tx_clone.lock().unwrap().as_ref() {
                        count_clone.fetch_add(mono.len(), Ordering::Relaxed);
                        let _ = tx.send(mono);
                    }
                },
                |err| eprintln!("Audio stream error: {err}"),
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
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
                        count_clone.fetch_add(mono.len(), Ordering::Relaxed);
                        let _ = tx.send(mono);
                    }
                },
                |err| eprintln!("Audio stream error: {err}"),
                None,
            ),
            fmt => return Err(format!("Unsupported sample format: {fmt:?}")),
        }
        .map_err(|e| format!("Failed to build input stream: {e}"))?;

        Ok(Self {
            stream,
            chunk_tx,
            sample_rate,
            sample_count,
        })
    }

    /// Start recording. Returns a receiver that yields mono f32 audio chunks.
    /// The channel closes when `stop_recording()` is called.
    pub fn start_recording(&self) -> mpsc::Receiver<Vec<f32>> {
        let (tx, rx) = mpsc::channel();
        self.sample_count.store(0, Ordering::Relaxed);
        *self.chunk_tx.lock().unwrap() = Some(tx);
        self.stream.play().unwrap();
        rx
    }

    /// Stop recording. Drops the sender, closing the channel.
    pub fn stop_recording(&self) {
        self.stream.pause().unwrap();
        *self.chunk_tx.lock().unwrap() = None;
    }

    pub fn captured_secs(&self) -> f32 {
        self.sample_count.load(Ordering::Relaxed) as f32 / self.sample_rate as f32
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

pub use transcrust_core::audio::*;
