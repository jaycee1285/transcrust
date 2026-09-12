//! A second front end onto the tray's mode-switch contract.
//!
//! `fuzzel --dmenu` reads newline-separated items on stdin, draws a
//! Wayland-native layer-shell picker, and writes the chosen line — or, with
//! `--index`, its 0-based index — to stdout, exiting non-zero on cancel. That
//! is the whole mechanism: the picker never builds a model and never touches
//! the state machine. It resolves a label to an index and sends that index on
//! the same `mpsc::UnboundedSender<usize>` the tray radio uses, so every rule
//! the main loop already enforces — refused unless Idle, the `watch` channel
//! echoing where we actually landed — applies unchanged.
//!
//! `--index` is preferred because it removes string matching from the path
//! entirely, which is what lets the active row carry a marker. Older fuzzels
//! lack it, so the fallback is an exact-string match against the label list and
//! in that case the labels are fed through unprettified.

use std::process::Stdio;

use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const BINARY: &str = "fuzzel";

/// Marker on the row that is currently loaded. dmenu has no notion of a
/// "current" entry, so the list carries it. Only used on the `--index` path,
/// where nothing has to be stripped back off.
const ACTIVE_MARKER: &str = "● ";

/// Whether a picker binary is on `PATH`. `--doctor` reports this; a daemon
/// without it still dictates, it just cannot be picked at.
pub fn is_available() -> bool {
    std::env::var_os("PATH")
        .map(|path| {
            std::env::split_paths(&path).any(|dir| dir.join(BINARY).is_file())
        })
        .unwrap_or(false)
}

/// Does this fuzzel understand `--index`?
///
/// Asked once per pick rather than cached: it costs one `--help` and the answer
/// changes under you on a rolling system.
async fn supports_index() -> bool {
    match Command::new(BINARY).arg("--help").output().await {
        Ok(output) => {
            let help = String::from_utf8_lossy(&output.stdout);
            help.contains("--index")
        }
        Err(_) => false,
    }
}

/// Show the picker and return the chosen mode index.
///
/// `Ok(None)` is a cancel — a non-zero exit from fuzzel — and is a no-op, not
/// an error state.
pub async fn pick(
    labels: &[(String, crate::mode::Capture)],
    active: usize,
) -> Result<Option<usize>, String> {
    if !is_available() {
        return Err(format!("{BINARY} is not on PATH"));
    }

    let indexed = supports_index().await;
    // Annotation is only safe on the indexed path. Without `--index` fuzzel
    // returns the chosen *string*, so a decorated row would match no label and
    // the pick would be dropped in silence.
    let menu: String = labels
        .iter()
        .enumerate()
        .map(|(i, (label, capture))| {
            if !indexed {
                return label.clone();
            }
            let marker = if i == active { ACTIVE_MARKER } else { "  " };
            let capture = match capture {
                crate::mode::Capture::Hold => "hold",
                crate::mode::Capture::Toggle => "toggle",
            };
            format!("{marker}{label}  [{capture}]")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut command = Command::new(BINARY);
    command.arg("--dmenu").arg("--prompt").arg("mode> ");
    if indexed {
        command.arg("--index");
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to spawn {BINARY}: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(menu.as_bytes())
            .await
            .map_err(|e| format!("failed to feed {BINARY}: {e}"))?;
        // fuzzel does not draw until stdin closes.
        drop(stdin);
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("failed to wait on {BINARY}: {e}"))?;

    if !output.status.success() {
        return Ok(None);
    }

    let chosen = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if chosen.is_empty() {
        return Ok(None);
    }

    Ok(resolve(&chosen, labels, indexed))
}

/// Turn fuzzel's stdout into an index into `labels`.
fn resolve(
    chosen: &str,
    labels: &[(String, crate::mode::Capture)],
    indexed: bool,
) -> Option<usize> {
    if indexed {
        return chosen
            .parse::<usize>()
            .ok()
            .filter(|index| *index < labels.len());
    }
    // Exact-string fallback. The marker is never applied on this path, so a
    // miss here means the label list moved under the picker, not that the
    // prefix ate the match.
    labels.iter().position(|(label, _)| label == chosen)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels() -> Vec<(String, crate::mode::Capture)> {
        use crate::mode::Capture;
        vec![
            ("Parakeet TDT 0.6B v3 (int4)".to_string(), Capture::Hold),
            ("Nemotron Speech Streaming EN (int8)".to_string(), Capture::Toggle),
            ("Granite Speech 5 470m TurboCTC".to_string(), Capture::Hold),
            ("Granite Speech 5 470m TurboCTC — Long".to_string(), Capture::Hold),
        ]
    }

    #[test]
    fn an_index_resolves_to_a_mode() {
        assert_eq!(resolve("3", &labels(), true), Some(3));
    }

    #[test]
    fn an_out_of_range_index_is_refused_rather_than_panicking() {
        assert_eq!(resolve("9", &labels(), true), None);
        assert_eq!(resolve("not-a-number", &labels(), true), None);
    }

    #[test]
    fn the_fallback_matches_the_label_exactly() {
        // The em-dashed label is the one most likely to be mangled by any
        // prettifying, which is why the fallback path prettifies nothing.
        assert_eq!(
            resolve("Granite Speech 5 470m TurboCTC — Long", &labels(), false),
            Some(3)
        );
        assert_eq!(resolve("Granite Speech 5", &labels(), false), None);
    }

    #[test]
    fn the_active_marker_never_reaches_the_string_fallback() {
        // If a marked row were ever fed to a fuzzel without --index, the
        // returned string would not match any label and the pick would be
        // silently dropped. The marker is applied only on the indexed path.
        let marked = format!("{ACTIVE_MARKER}{}", labels()[0].0);
        assert_eq!(resolve(&marked, &labels(), false), None);
    }
}
