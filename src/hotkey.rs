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
        // Emits no character, but most desktops bind it to a screenshot tool, so
        // it is offered rather than recommended.
        "PRTSCR" | "PRINT" | "PRINTSCREEN" | "SYSRQ" => Some(Key::KEY_SYSRQ),
        "MENU" | "COMPOSE" => Some(Key::KEY_COMPOSE),
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

    let (bound_modifiers, bound_key) = resolve_binding(config)
        .unwrap_or_else(|error| panic!("{error}. Use ScrollLock, F13-F20, Pause, PrtScr, etc."));

    let trigger_key = parse_key(&bound_key)
        .unwrap_or_else(|| panic!("Unknown key name: {bound_key}. Use ScrollLock, F13-F20, Pause, etc."));

    let modifier_keys: Vec<Key> = bound_modifiers
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

    let grab_while_held = config.grab;

    let device = find_keyboard_device(config.device.as_deref())
        .expect("Failed to find keyboard device");

    let mut stream = device
        .into_event_stream()
        .expect("Failed to create event stream");

    tokio::spawn(async move {
        let mut mods_held: HashSet<Key> = HashSet::new();
        let mut grabbed = false;

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
                            // EVIOCGRAB for the duration of the hold. This stops
                            // the auto-repeat storm — the bulk of the damage from
                            // a printable trigger — but **not the first press**,
                            // which has already been delivered to the compositor
                            // by the time we see it. Grabbing earlier, on the
                            // modifier, would mean owning the whole keyboard
                            // whenever Alt is down and would break every other
                            // Alt binding on the desktop.
                            //
                            // Safe to fail: a grab is tied to the open file
                            // description, so the kernel drops it if the process
                            // dies. The hazard is a hang, not a crash.
                            if grab_while_held {
                                if let Err(error) = stream.device_mut().grab() {
                                    eprintln!("hotkey grab failed, continuing ungrabbed: {error}");
                                } else {
                                    grabbed = true;
                                }
                            }
                            let _ = tx.send(HotkeyEvent::Pressed).await;
                        }
                        0 => {
                            if grabbed {
                                if let Err(error) = stream.device_mut().ungrab() {
                                    eprintln!("hotkey ungrab failed: {error}");
                                }
                                grabbed = false;
                            }
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

/// Split a labwc-style chord into `(modifiers, key)`.
///
/// `rc.xml` writes bindings as `A-space`, `W-b`, `C-A-t`, so accepting the same
/// spelling means a binding can be moved between the two files without
/// translation. Prefixes are labwc's, and each maps to the **left** variant,
/// which is what labwc itself matches:
///
/// | prefix | key |
/// |---|---|
/// | `W-` | LeftMeta / Super |
/// | `A-` | LeftAlt |
/// | `C-` | LeftCtrl |
/// | `S-` | LeftShift |
///
/// A chord with no prefix is a bare trigger: `"Print"` is exactly `key = "Print"`
/// with no modifiers. Returns `None` if any segment is not a key transcrust can
/// bind, so a typo fails loudly at startup rather than silently never firing.
pub fn parse_chord(chord: &str) -> Option<(Vec<String>, String)> {
    let mut modifiers = Vec::new();
    let mut rest = chord.trim();

    loop {
        let (prefix, tail) = match rest.split_once('-') {
            // A trailing `-` is the key itself (the minus key), not a prefix.
            Some((p, t)) if !t.is_empty() => (p, t),
            _ => break,
        };
        let named = match prefix.to_uppercase().as_str() {
            "W" => "LeftMeta",
            "A" => "LeftAlt",
            "C" => "LeftCtrl",
            "S" => "LeftShift",
            _ => break,
        };
        modifiers.push(named.to_string());
        rest = tail;
    }

    if rest.is_empty() || parse_key(rest).is_none() {
        return None;
    }
    Some((modifiers, rest.to_string()))
}

/// The binding actually in force, after `chord` has had its say.
///
/// `chord` wins when set, because a config carrying both should not depend on
/// which one the reader noticed first.
pub fn resolve_binding(config: &HotkeyConfig) -> Result<(Vec<String>, String), String> {
    match config.chord.as_deref() {
        Some(chord) => parse_chord(chord)
            .ok_or_else(|| format!("hotkey.chord \"{chord}\" is not a chord transcrust can bind")),
        None => Ok((config.modifiers.clone(), config.key.clone())),
    }
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
            | "PAUSE" | "SCROLLLOCK" | "PRTSCR" | "PRINT" | "PRINTSCREEN" | "SYSRQ"
            | "MENU" | "COMPOSE"
            | "F13" | "F14" | "F15" | "F16" | "F17" | "F18" | "F19" | "F20"
    )
}

/// Every name `parse_key` accepts, canonical spelling first.
///
/// Exists so `--keys` can name a key it just saw. `parse_key` is a one-way
/// match, and a probe that can only say "some key" is no better than guessing.
const BINDABLE: &[&str] = &[
    "LeftCtrl", "RightCtrl", "LeftShift", "RightShift", "LeftAlt", "RightAlt",
    "LeftMeta", "RightMeta", "Pause", "ScrollLock", "PrtScr", "Menu", "CapsLock",
    "F13", "F14", "F15", "F16", "F17", "F18", "F19", "F20", "Space",
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
    "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
];

fn name_for(key: Key) -> Option<&'static str> {
    BINDABLE.iter().copied().find(|n| parse_key(n) == Some(key))
}

/// Print keyboard events as transcrust sees them, with the name to put in
/// config and whether it can leak into the focused window.
///
/// The reason this exists: choosing a push-to-talk chord was guesswork. The
/// daemon reads one evdev device, a chord only fires if every modifier is
/// already held when the trigger goes down, and nothing reported either fact.
/// Watching the actual stream answers both in seconds.
pub async fn probe_keys(device_name: Option<&str>) {
    let device = match find_keyboard_device(device_name) {
        Ok(device) => device,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("Try --list-devices; are you in the 'input' group?");
            std::process::exit(1);
        }
    };
    println!(
        "Watching {}. Hold your candidate chord; Ctrl+C to stop.",
        device.name().unwrap_or("keyboard")
    );
    println!("A chord fires only if every modifier is already down when the trigger goes down.\n");

    let mut stream = match device.into_event_stream() {
        Ok(stream) => stream,
        Err(error) => {
            eprintln!("Failed to read events: {error}");
            std::process::exit(1);
        }
    };
    let mut held: Vec<&'static str> = Vec::new();

    loop {
        let event = match stream.next().await {
            Some(Ok(event)) => event,
            _ => break,
        };
        let InputEventKind::Key(key) = event.kind() else { continue };
        let value = event.value();
        if value == 2 {
            continue; // auto-repeat: noise here, and the thing that leaks
        }
        let Some(name) = name_for(key) else {
            if value == 1 {
                println!("  press    {:<12} — not bindable by transcrust", format!("{key:?}"));
            }
            continue;
        };
        if value == 1 {
            if !held.contains(&name) {
                held.push(name);
            }
        } else {
            held.retain(|h| *h != name);
        }
        let leaks = if is_silent_key(name) { "silent" } else { "LEAKS a character" };
        let action = if value == 1 { "press  " } else { "release" };
        println!("  {action}  {name:<12} {leaks}");
        if value == 1 && held.len() > 1 {
            let (trigger, mods) = held.split_last().expect("held is non-empty");
            let quoted: Vec<String> = mods.iter().map(|m| format!("\"{m}\"")).collect();
            println!(
                "           chord: key = \"{trigger}\", modifiers = [{}]",
                quoted.join(", ")
            );
        }
    }
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

    /// The spelling used in labwc's `rc.xml`, so a binding can move between the
    /// two files without translation.
    #[test]
    fn parses_labwc_chords() {
        assert_eq!(
            parse_chord("A-space"),
            Some((vec!["LeftAlt".to_string()], "space".to_string()))
        );
        assert_eq!(
            parse_chord("C-A-t"),
            Some((vec!["LeftCtrl".to_string(), "LeftAlt".to_string()], "t".to_string()))
        );
        assert_eq!(parse_chord("Print"), Some((vec![], "Print".to_string())));
    }

    /// A typo must fail at startup, not fire never and say nothing.
    #[test]
    fn rejects_chords_it_cannot_bind() {
        assert_eq!(parse_chord("A-nope"), None);
        assert_eq!(parse_chord("X-space"), None, "unknown prefix is not a modifier");
        assert_eq!(parse_chord(""), None);
        assert_eq!(parse_chord("A-"), None, "a dangling prefix binds nothing");
    }

    /// `chord` wins when both are present, so the binding does not depend on
    /// which field the reader happened to notice first.
    #[test]
    fn chord_overrides_key_and_modifiers() {
        let mut config = HotkeyConfig::default();
        config.key = "Pause".into();
        config.modifiers = vec!["LeftCtrl".into()];
        assert_eq!(
            resolve_binding(&config).unwrap(),
            (vec!["LeftCtrl".to_string()], "Pause".to_string())
        );
        config.chord = Some("A-space".into());
        assert_eq!(
            resolve_binding(&config).unwrap(),
            (vec!["LeftAlt".to_string()], "space".to_string())
        );
        config.chord = Some("A-nope".into());
        assert!(resolve_binding(&config).is_err());
    }
}
