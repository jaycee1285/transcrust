use std::sync::mpsc;

use crate::model::ModelKind;
use crate::observe::Observer;

#[derive(Clone)]
pub enum TranscriptionService {
    Parakeet(crate::parakeet::ParakeetService),
    Granite(crate::granite::GraniteService),
    Nemotron(crate::nemotron::NemotronService),
}

impl TranscriptionService {
    pub fn new(model_dir: String, kind: ModelKind, idle_timeout_secs: u64) -> Result<Self, String> {
        match kind {
            ModelKind::Parakeet => Ok(Self::Parakeet(crate::parakeet::ParakeetService::new(
                model_dir,
                idle_timeout_secs,
            )?)),
            ModelKind::Granite => Ok(Self::Granite(crate::granite::GraniteService::new(
                model_dir,
                idle_timeout_secs,
            )?)),
            ModelKind::Nemotron => Ok(Self::Nemotron(crate::nemotron::NemotronService::new(
                model_dir,
                idle_timeout_secs,
            )?)),
        }
    }

    pub async fn transcribe(
        &self,
        observer: Observer,
        audio_rx: mpsc::Receiver<Vec<f32>>,
        source_sample_rate: u32,
    ) -> Result<String, String> {
        match self {
            Self::Parakeet(service) => {
                service
                    .transcribe(observer, audio_rx, source_sample_rate)
                    .await
            }
            Self::Granite(service) => {
                service
                    .transcribe(observer, audio_rx, source_sample_rate)
                    .await
            }
            Self::Nemotron(service) => {
                service
                    .transcribe(observer, audio_rx, source_sample_rate)
                    .await
            }
        }
    }

    /// Release this engine's model immediately.
    ///
    /// `runtime-stack.md` recorded the old behaviour: the outgoing worker held
    /// its model until `observe.idle_timeout_secs` (default 60), so both
    /// engines stayed resident for up to a minute after every switch.
    /// Measured, that is Parakeet's 1252 MB plus Nemotron's 1036 MB — 2.3 GB
    /// concurrent on a 16 GB machine, and a larger saving than any session
    /// tuning flag produced.
    ///
    /// Only safe to call on a service the main loop has just replaced, and
    /// only when the incoming mode uses a *different* model directory — two
    /// modes over one model share a service, so shutting it down would unload
    /// the engine that was just switched to.
    pub fn shutdown(&self) {
        match self {
            Self::Parakeet(service) => service.shutdown(),
            Self::Granite(service) => service.shutdown(),
            Self::Nemotron(service) => service.shutdown(),
        }
    }

    /// Does a long toggled capture want cutting into windows before this engine
    /// sees it?
    ///
    /// Yes for the two batch engines, and for opposite reasons: Parakeet throws
    /// past ~300 s on a positional table sized for ~2500 frames, and Granite
    /// costs ~19 MB of RSS per extra second of window (one 1200 s call peaked at
    /// 10.5 GB).
    ///
    /// No for Nemotron, and windowing it is actively harmful twice over:
    /// every seam resets the encoder cache and the decoder LSTM — measured on
    /// `Record-2.wav` as `Tarakeate` and `paint Dominion` where one continuous
    /// pass gives `parakeet` and `paid Dominion` — and the windowing branch
    /// drains the whole capture before the engine is called, which bypasses
    /// encoding-while-speaking entirely. Its own footprint is a fixed 7.7 MB of
    /// streaming state at any length, so it needs neither protection.
    pub fn windows_long_captures(&self) -> bool {
        match self {
            Self::Parakeet(_) | Self::Granite(_) => true,
            Self::Nemotron(_) => false,
        }
    }

    /// How long `main` waits before giving up on a job.
    ///
    /// Both current engines answer well inside this: measured worst case is
    /// Parakeet at 11.7 s for 60 s of audio. It stays per-engine rather than a
    /// global constant because a mode that adds a learned normalisation pass
    /// will need its own ceiling, and that is the seam to widen.
    pub fn timeout(&self) -> std::time::Duration {
        match self {
            Self::Parakeet(_) | Self::Granite(_) => std::time::Duration::from_secs(45),
            // No duration head means no frame skip: every encoder frame gets a
            // joint step, so a long toggled capture is slower per second than
            // Parakeet even though its RTF is comfortable. Measured 16 s for
            // 2:23 at 1120 ms chunks, so 45 s would clip a five-minute capture.
            Self::Nemotron(_) => std::time::Duration::from_secs(180),
        }
    }

    pub fn name(&self) -> &'static str {
        self.kind().display_name()
    }

    pub fn kind(&self) -> ModelKind {
        match self {
            Self::Parakeet(_) => ModelKind::Parakeet,
            Self::Granite(_) => ModelKind::Granite,
            Self::Nemotron(_) => ModelKind::Nemotron,
        }
    }
}
