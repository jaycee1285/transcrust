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
                if modifier_keys.contains(&key) {
                    match value {
                        1 => { mods_held.insert(key); }
                        0 => { mods_held.remove(&key); }
                        _ => {}
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
