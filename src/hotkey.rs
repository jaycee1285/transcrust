use evdev::{Device, InputEventKind, Key};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use crate::config::HotkeyConfig;

#[derive(Debug)]
pub enum HotkeyEvent {
    Pressed,
    Released,
    /// Advance to the next mode. Fires on key-down only; the main loop
    /// declines it unless idle, exactly as the tray radio does.
    CycleMode,
}

fn parse_key(name: &str) -> Option<Key> {
    match name.to_uppercase().as_str() {
        "SCROLLLOCK" => Some(Key::KEY_SCROLLLOCK),
        "PAUSE" => Some(Key::KEY_PAUSE),
        "F13" => Some(Key::KEY_F13),
        "F14" => Some(Key::KEY_F14),
        "F15" => Some(Key::KEY_F15),
        "F16" => Some(Key::KEY_F16),
        "F17" => Some(Key::KEY_F17),
        "F18" => Some(Key::KEY_F18),
        "F19" => Some(Key::KEY_F19),
        "F20" => Some(Key::KEY_F20),
        "LEFTCTRL" | "LCTRL" => Some(Key::KEY_LEFTCTRL),
        "RIGHTCTRL" | "RCTRL" => Some(Key::KEY_RIGHTCTRL),
        "LEFTSHIFT" | "LSHIFT" => Some(Key::KEY_LEFTSHIFT),
        "RIGHTSHIFT" | "RSHIFT" => Some(Key::KEY_RIGHTSHIFT),
        "LEFTALT" | "LALT" => Some(Key::KEY_LEFTALT),
        "RIGHTALT" | "RALT" => Some(Key::KEY_RIGHTALT),
        "LEFTMETA" | "SUPER" | "LMETA" => Some(Key::KEY_LEFTMETA),
        "RIGHTMETA" | "RMETA" => Some(Key::KEY_RIGHTMETA),
        "CAPSLOCK" => Some(Key::KEY_CAPSLOCK),
        // Letter keys (for modifier combos like Super+V)
        "A" => Some(Key::KEY_A), "B" => Some(Key::KEY_B), "C" => Some(Key::KEY_C),
        "D" => Some(Key::KEY_D), "E" => Some(Key::KEY_E), "F" => Some(Key::KEY_F),
        "G" => Some(Key::KEY_G), "H" => Some(Key::KEY_H), "I" => Some(Key::KEY_I),
        "J" => Some(Key::KEY_J), "K" => Some(Key::KEY_K), "L" => Some(Key::KEY_L),
        "M" => Some(Key::KEY_M), "N" => Some(Key::KEY_N), "O" => Some(Key::KEY_O),
        "P" => Some(Key::KEY_P), "Q" => Some(Key::KEY_Q), "R" => Some(Key::KEY_R),
        "S" => Some(Key::KEY_S), "T" => Some(Key::KEY_T), "U" => Some(Key::KEY_U),
        "V" => Some(Key::KEY_V), "W" => Some(Key::KEY_W), "X" => Some(Key::KEY_X),
        "Y" => Some(Key::KEY_Y), "Z" => Some(Key::KEY_Z),
        "SPACE" => Some(Key::KEY_SPACE),
        _ => None,
    }
}

fn find_keyboard_device(config_device: Option<&str>) -> Result<Device, String> {
    if let Some(path) = config_device {
        return Device::open(path).map_err(|e| format!("Failed to open {path}: {e}"));
    }

    // Auto-detect: find first keyboard-like device
    let input_dir = PathBuf::from("/dev/input");
    let mut entries: Vec<_> = std::fs::read_dir(&input_dir)
        .map_err(|e| format!("Cannot read /dev/input: {e}"))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|n| n.starts_with("event"))
                .unwrap_or(false)
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if let Ok(device) = Device::open(&path) {
            if let Some(keys) = device.supported_keys() {
                if keys.contains(Key::KEY_A) && keys.contains(Key::KEY_Z) {
                    return Ok(device);
                }
            }
        }
    }

    Err("No keyboard device found. Check input group membership or set hotkey.device in config".into())
}

pub async fn listen(config: &HotkeyConfig) -> mpsc::Receiver<HotkeyEvent> {
    let (tx, rx) = mpsc::channel(16);

    let trigger_key = parse_key(&config.key)
        .unwrap_or_else(|| panic!("Unknown key name: {}. Use ScrollLock, F13-F20, Pause, etc.", config.key));

    let modifier_keys: Vec<Key> = config
        .modifiers
        .iter()
        .filter_map(|m| {
            let k = parse_key(m);
            if k.is_none() {
                eprintln!("Unknown modifier key: {m}");
            }
            k
        })
        .collect();

    // The mode toggle rides the *same* evdev stream: a device can only be
    // turned into one event stream, and opening it twice would race.
    // Unbound by default — a keyboard-driven desktop has a curated keymap and
    // this should not stomp it. Set hotkey.mode_key to enable.
    let mode_key = config.mode_key.as_deref().and_then(|name| {
        let key = parse_key(name);
        if key.is_none() {
            eprintln!("Unknown mode_key: {name}");
        }
        key
    });
    let mode_modifier_keys: Vec<Key> = config
        .mode_modifiers
        .iter()
        .filter_map(|m| {
            let k = parse_key(m);
            if k.is_none() {
                eprintln!("Unknown mode modifier key: {m}");
            }
            k
        })
        .collect();
    // Every key whose held state has to be tracked, from either binding.
    let tracked: Vec<Key> = modifier_keys
        .iter()
        .chain(mode_modifier_keys.iter())
        .copied()
        .collect();

    let device = find_keyboard_device(config.device.as_deref())
        .expect("Failed to find keyboard device");

    let mut stream = device
        .into_event_stream()
        .expect("Failed to create event stream");

    tokio::spawn(async move {
        let mut mods_held: HashSet<Key> = HashSet::new();

        while let Some(Ok(event)) = stream.next().await {
            if let InputEventKind::Key(key) = event.kind() {
                let value = event.value(); // 0=release, 1=press, 2=repeat

                // Track modifier state
                if tracked.contains(&key) {
                    match value {
                        1 => { mods_held.insert(key); }
                        0 => { mods_held.remove(&key); }
                        _ => {}
                    }
                }

                // Mode toggle, checked before the trigger so the two can share
                // a key with different modifiers if the user wants that.
                if let Some(mode_key) = mode_key {
                    if key == mode_key && value == 1 {
                        let all_mods = mode_modifier_keys.iter().all(|m| mods_held.contains(m));
                        if all_mods {
                            let _ = tx.send(HotkeyEvent::CycleMode).await;
                            continue;
                        }
                    }
                }

                // Check trigger key
                if key == trigger_key {
                    let all_mods = modifier_keys.iter().all(|m| mods_held.contains(m));
                    match value {
                        1 if all_mods => {
                            let _ = tx.send(HotkeyEvent::Pressed).await;
                        }
                        0 => {
                            let _ = tx.send(HotkeyEvent::Released).await;
                        }
                        _ => {}
                    }
                }
            }
        }
        eprintln!("Keyboard event stream ended");
    });

    rx
}

/// Does this trigger emit a character into whatever has focus?
///
/// transcrust reads evdev **passively** — no `EVIOCGRAB` — so the compositor and
/// the focused window see every press and every auto-repeat regardless of what
/// transcrust does with them. A printable trigger therefore types into your
/// document for the whole hold: `Space + LeftAlt` cycled Firefox tabs and left
/// spaces behind. Modifiers, the F13-F20 block, Pause and Scroll Lock emit
/// nothing, so they are the only safe push-to-talk triggers without a grab.
pub fn is_silent_key(name: &str) -> bool {
    matches!(
        name.to_uppercase().as_str(),
        "LEFTCTRL" | "LCTRL" | "RIGHTCTRL" | "RCTRL"
            | "LEFTSHIFT" | "LSHIFT" | "RIGHTSHIFT" | "RSHIFT"
            | "LEFTALT" | "LALT" | "RIGHTALT" | "RALT"
            | "LEFTMETA" | "LMETA" | "SUPER" | "RIGHTMETA" | "RMETA"
            | "PAUSE" | "SCROLLLOCK"
            | "F13" | "F14" | "F15" | "F16" | "F17" | "F18" | "F19" | "F20"
    )
}

pub fn list_devices() {
    let input_dir = PathBuf::from("/dev/input");
    let mut entries: Vec<_> = match std::fs::read_dir(&input_dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(e) => {
            eprintln!("Cannot read /dev/input: {e}");
            eprintln!("Are you in the 'input' group?");
            return;
        }
    };
    entries.sort_by_key(|e| e.file_name());

    println!("Keyboard devices:");
    for entry in entries {
        let path = entry.path();
        if let Ok(device) = Device::open(&path) {
            if let Some(keys) = device.supported_keys() {
                if keys.contains(Key::KEY_A) && keys.contains(Key::KEY_Z) {
                    println!("  {} - {}", path.display(), device.name().unwrap_or("unknown"));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every name `is_silent_key` claims is safe must actually parse, or
    /// `--doctor` will bless a binding that never fires.
    #[test]
    fn every_silent_key_parses() {
        for name in [
            "LEFTCTRL", "RIGHTCTRL", "LEFTSHIFT", "RIGHTSHIFT", "LEFTALT", "RIGHTALT",
            "LEFTMETA", "RIGHTMETA", "SUPER", "PAUSE", "SCROLLLOCK", "F13", "F20",
        ] {
            assert!(parse_key(name).is_some(), "{name} is called silent but does not parse");
            assert!(is_silent_key(name), "{name} should be silent");
        }
    }

    /// The binding that caused the leak, and the one that replaced it.
    #[test]
    fn character_keys_are_not_silent() {
        for name in ["SPACE", "A", "Z", "CAPSLOCK"] {
            assert!(!is_silent_key(name), "{name} emits something and must warn");
        }
    }

    /// Config is written in mixed case; the check must not depend on it.
    #[test]
    fn silence_check_ignores_case() {
        assert!(is_silent_key("RightAlt"));
        assert!(is_silent_key("rightalt"));
        assert!(!is_silent_key("Space"));
    }
}
