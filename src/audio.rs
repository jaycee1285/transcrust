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

/// Resample f32 audio to 16kHz, returning f32 for batch transcription backends.
pub fn resample_to_16k(input: &[f32], from_rate: u32) -> Vec<f32> {
    if from_rate == 16000 {
        input.to_vec()
    } else {
        resample(input, from_rate, 16000)
    }
}

fn resample(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    let ratio = to_rate as f64 / from_rate as f64;
    let output_len = (input.len() as f64 * ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_pos = i as f64 / ratio;
        let idx = src_pos as usize;
        let frac = src_pos - idx as f64;

        let sample = if idx + 1 < input.len() {
            input[idx] * (1.0 - frac as f32) + input[idx + 1] * frac as f32
        } else if idx < input.len() {
            input[idx]
        } else {
            0.0
        };
        output.push(sample);
    }

    output
}
