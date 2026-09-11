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
    /// labwc-style chord, e.g. `"A-space"`, `"W-A-p"`, `"Print"`.
    ///
    /// Overrides `key`/`modifiers` when set, so a binding can be written the way
    /// it is written in `rc.xml` instead of as a key plus an array. Prefixes are
    /// labwc's: `W-` super, `A-` alt, `C-` ctrl, `S-` shift.
    pub chord: Option<String>,
    /// Take exclusive control of the keyboard (`EVIOCGRAB`) while the trigger is
    /// held, so a printable trigger cannot also reach the focused window.
    ///
    /// Off by default, and an incomplete fix by construction — see
    /// `hotkey.rs`'s grab handling for exactly what it does and does not stop.
    #[serde(default)]
    pub grab: bool,
    /// Key that cycles to the next mode. Unset by default: a keyboard-driven
    /// desktop already has a full keymap and this should not claim a chord
    /// without being asked.
    pub mode_key: Option<String>,
    #[serde(default)]
    pub mode_modifiers: Vec<String>,
}

/// Push-to-talk trigger. **Modifier-only on purpose.**
///
/// `hotkey.rs` reads evdev passively and never calls `EVIOCGRAB`, so the
/// compositor and the focused window see every press and every auto-repeat no
/// matter what transcrust does with them. A printable trigger therefore types
/// into whatever has focus for the entire hold — the previous default,
/// `Space + LeftAlt`, cycled Firefox tabs and left a trail of spaces behind
/// while dictating. Both alts emit no character, so there is nothing to leak.
///
/// `--doctor` warns if this is set to something that produces a character.
fn default_key() -> String {
    "RightAlt".to_string()
}

fn default_modifiers() -> Vec<String> {
    vec!["LeftAlt".to_string()]
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            key: default_key(),
            chord: None,
            grab: false,
            modifiers: default_modifiers(),
            device: None,
            mode_key: None,
            mode_modifiers: Vec::new(),
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
    /// Bank every dictation to `~/.local/share/transcrust/corpus/` as a WAV
    /// plus a JSON sidecar. Off by default — it writes ~10 MB/minute and
    /// records your speech to disk.
    #[serde(default)]
    pub corpus: bool,
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
            corpus: false,
        }
    }
}

pub fn load() -> Config {
    let path = config_path();
    let mut config: Config = match std::fs::read_to_string(&path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
            eprintln!("Config parse error: {e}, using defaults");
            toml::from_str("").unwrap()
        }),
        Err(_) => {
            toml::from_str("").unwrap()
        }
    };
    if let Ok(model_path) = std::env::var("TRANSCRUST_MODEL_PATH") {
        config.model.path = Some(model_path);
    }
    config
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(".config"))
        .join("transcrust")
        .join("config.toml")
}
