use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub hotkey: HotkeyConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub model: ModelConfig,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub observe: ObserveConfig,
}

#[derive(Debug, Deserialize)]
pub struct HotkeyConfig {
    #[serde(default = "default_key")]
    pub key: String,
    #[serde(default = "default_modifiers")]
    pub modifiers: Vec<String>,
    pub device: Option<String>,
}

fn default_key() -> String {
    "Space".to_string()
}

fn default_modifiers() -> Vec<String> {
    vec!["LeftAlt".to_string()]
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            key: default_key(),
            modifiers: default_modifiers(),
            device: None,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct AudioConfig {
    pub device: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ModelConfig {
    pub path: Option<String>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self { path: None }
    }
}

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_true")]
    pub wtype: bool,
    #[serde(default = "default_true")]
    pub clipboard: bool,
}

fn default_true() -> bool {
    true
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            wtype: true,
            clipboard: true,
        }
    }
}

impl Clone for OutputConfig {
    fn clone(&self) -> Self {
        Self {
            wtype: self.wtype,
            clipboard: self.clipboard,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ObserveConfig {
    #[serde(default = "default_true")]
    pub desktop_notifications: bool,
    #[serde(default = "default_sample_chars")]
    pub sample_chars: usize,
    /// Seconds of inactivity before the model is unloaded from memory (default: 60).
    #[serde(default = "default_idle_timeout_secs")]
    pub idle_timeout_secs: u64,
}

fn default_sample_chars() -> usize {
    120
}

fn default_idle_timeout_secs() -> u64 {
    60
}

impl Default for ObserveConfig {
    fn default() -> Self {
        Self {
            desktop_notifications: true,
            sample_chars: default_sample_chars(),
            idle_timeout_secs: default_idle_timeout_secs(),
        }
    }
}

pub fn load() -> Config {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
            eprintln!("Config parse error: {e}, using defaults");
            toml::from_str("").unwrap()
        }),
        Err(_) => {
            toml::from_str("").unwrap()
        }
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(".config"))
        .join("transcrust")
        .join("config.toml")
}
