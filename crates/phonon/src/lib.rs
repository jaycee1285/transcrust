//! Native Rust Phonon 2.5 English. Actual packed Fermion artifact and TDT graph.
//! C runtime is only a numerical/performance reference; it is not linked here.
//! See NOTICE for reference implementations and model attribution.
mod decoder;
mod encoder;
mod frontend;
mod kernel;
pub mod segment;
mod weights;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
pub use decoder::Token;
use std::path::Path;
pub use weights::Weights;

pub struct Phonon {
    weights: Weights,
    decoder: decoder::Decoder,
    labels: Vec<String>,
    pool: rayon::ThreadPool,
}
pub struct Transcript {
    pub text: String,
    pub tokens: Vec<Token>,
}
impl Phonon {
    pub fn load(dir: &Path) -> Result<Self> {
        use sha2::{Digest, Sha256};
        let config_bytes = std::fs::read(dir.join("config.json"))?;
        if format!("{:x}", Sha256::digest(&config_bytes))
            != "9e5dfcc4e49a6c169d53540f07f4115e591007759b835132c849166b235dac95"
        {
            return Err("Phonon 2.5 archive config checksum mismatch".into());
        }
        let config: serde_json::Value = serde_json::from_slice(&config_bytes)?;
        if config["sample_rate"] != 16000
            || config["encoder"]["n_layers"] != 24
            || config["encoder"]["d_model"] != 1024
        {
            return Err("unexpected Phonon 2.5 configuration".into());
        }
        let labels = config["labels"]
            .as_array()
            .ok_or("missing labels")?
            .iter()
            .map(|v| v.as_str().map(str::to_string).ok_or("invalid label"))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if labels.len() != 8192 {
            return Err("unexpected vocabulary length".into());
        }
        let mut weights = Weights::load(dir)?;
        let decoder = decoder::Decoder::load(&mut weights)?;
        Ok(Self {
            weights,
            decoder,
            labels,
            pool: rayon::ThreadPoolBuilder::new().num_threads(4).build()?,
        })
    }
    /// One independent window of 16 kHz mono. The session adapter owns windowing.
    pub fn transcribe(&self, audio: &[f32], trace: Option<&Path>) -> Result<Transcript> {
        if audio.len() < 1600 {
            return Ok(Transcript {
                text: String::new(),
                tokens: Vec::new(),
            });
        }
        if audio.len() > 30 * 16000 {
            return Err("Phonon window exceeds 30 seconds; feed the bounded session adapter".into());
        }
        self.pool.install(|| {
            let (mel, frames) = frontend::mel(audio)?;
            if let Some(dir) = trace {
                std::fs::create_dir_all(dir)?;
                std::fs::write(
                    dir.join("mel.f32"),
                    mel.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>(),
                )?;
            }
            let enc = encoder::encode(&self.weights, mel, frames, trace)?;
            let tokens = self.decoder.decode(&enc)?;
            let text = tokens
                .iter()
                .map(|t| self.labels[t.id].replace('▁', " "))
                .collect::<String>()
                .trim()
                .to_string();
            Ok(Transcript { text, tokens })
        })
    }
}
