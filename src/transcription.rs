use std::sync::mpsc;

use crate::model::ModelKind;
use crate::observe::Observer;

#[derive(Clone)]
pub enum TranscriptionService {
    Parakeet(crate::parakeet::ParakeetService),
    Granite(crate::granite::GraniteService),
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
        }
    }

    pub fn name(&self) -> &'static str {
        self.kind().display_name()
    }

    pub fn kind(&self) -> ModelKind {
        match self {
            Self::Parakeet(_) => ModelKind::Parakeet,
            Self::Granite(_) => ModelKind::Granite,
        }
    }
}
