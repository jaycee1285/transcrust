//! Shared dictation mechanics. No capture device, UI, tray, paste or downloads.
pub mod audio;
pub mod correction;
pub mod lexicon;
pub mod model;
#[cfg(feature = "asr")]
pub mod nemotron;
#[cfg(feature = "asr")]
pub mod parakeet_ort;
pub mod postprocess;
pub mod vocabulary;

/// The embedding application owns logging and any platform notifications.
pub trait Events: Send + Sync {
    fn phase(&self, phase: &str, message: &str);
    fn error(&self, phase: &str, message: &str);
}

pub struct SilentEvents;
impl Events for SilentEvents {
    fn phase(&self, _: &str, _: &str) {}
    fn error(&self, _: &str, _: &str) {}
}

/// The caller must arrange the target's ORT library/framework. No download.
#[cfg(feature = "asr")]
pub fn init_ort_default() {
    let _ = ort::init()
        .with_name("transcrust")
        .with_telemetry(false)
        .commit();
}
