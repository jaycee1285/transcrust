use wl_clipboard_rs::copy::{MimeType, Options, Source};

use crate::config::OutputConfig;

/// Put the transcript where the user asked for it.
///
/// Returns the failures of any method that was **requested and did not work**,
/// even when another one succeeded. Previously those were collected and then
/// dropped on the floor whenever `successes > 0`, and since `clipboard` defaults
/// to `true` and is an in-process library call, it essentially always succeeds —
/// so a missing `wtype`/`dotool` looked like a clean run while nothing was ever
/// typed. The transcript went to the clipboard, the log said nothing, and the
/// symptom was "it transcribes but does not do anything".
///
/// `Err` still means nothing worked at all.
pub async fn inject_text(text: &str, config: &OutputConfig) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    let mut successes = 0usize;

    if config.clipboard {
        match copy_to_clipboard(text) {
            Ok(()) => successes += 1,
            Err(e) => errors.push(format!("clipboard: {e}")),
        }
    }

    if config.wtype {
        match type_with_wtype(text).await {
            Ok(()) => successes += 1,
            Err(e) => match type_with_dotool(text).await {
                Ok(()) => successes += 1,
                Err(e2) => errors.push(format!(
                    "typing failed, transcript not typed: wtype={e}; dotool={e2}. \
                     Both live in the nix devShell — run through `nix develop -c` \
                     or use the wrapped install"
                )),
            },
        }
    }

    if successes > 0 {
        Ok(errors)
    } else {
        Err(if errors.is_empty() {
            "no output methods enabled".into()
        } else {
            errors.join("; ")
        })
    }
}

fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let opts = Options::new();
    opts.copy(
        Source::Bytes(text.to_string().into_bytes().into()),
        MimeType::Text,
    )
    .map_err(|e| format!("wl-copy failed: {e}"))
}

async fn type_with_wtype(text: &str) -> Result<(), String> {
    let status = tokio::process::Command::new("wtype")
        .arg("--")
        .arg(text)
        .status()
        .await
        .map_err(|e| format!("wtype not found or failed to execute: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("wtype exited with status {status}"))
    }
}

async fn type_with_dotool(text: &str) -> Result<(), String> {
    let input = format!("type {text}");
    let mut child = tokio::process::Command::new("dotool")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("dotool not found: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin
            .write_all(input.as_bytes())
            .await
            .map_err(|e| format!("dotool stdin write failed: {e}"))?;
    }

    let status = child
        .wait()
        .await
        .map_err(|e| format!("dotool wait failed: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("dotool exited with status {status}"))
    }
}

pub fn copy_correction(text: &str) -> Result<(), String> {
    copy_to_clipboard(text)
}

/// Linux contract: the review has closed, the editor regains focus, and its
/// caret is immediately after `original`. Selecting first avoids deleting the
/// span before the replacement is available. Read the selection back against
/// saved destination text; extend only a verified short suffix. Never blindly
/// repeat selection or typing after a partial edit.
pub async fn replace_last(
    original: &str,
    corrected: &str,
    observer: &crate::observe::Observer,
) -> Result<(), String> {
    use unicode_segmentation::UnicodeSegmentation;
    let count = original.graphemes(true).count();
    if count == 0 || count > 20_000 {
        return Err("Replacement span is empty or too large; use Copy".into());
    }
    // Put the recovery text on the clipboard before attempting replacement.
    copy_to_clipboard(corrected)?;
    let selection = select_saved_span(original, count, observer).await;
    // Selection verification temporarily uses the clipboard. Always restore
    // the edited recovery text, including when validation refuses replacement.
    copy_to_clipboard(corrected)?;
    selection?;
    if corrected.is_empty() {
        let status = tokio::process::Command::new("wtype")
            .args(["-k", "BackSpace"])
            .status()
            .await
            .map_err(|e| e.to_string())?;
        return if status.success() {
            Ok(())
        } else {
            Err(format!("Could not remove verified dictation ({status})"))
        };
    }
    type_with_wtype(corrected)
        .await
        .map_err(|e| format!("Replacement typing failed: {e}; use the corrected clipboard text"))
}

async fn select_saved_span(
    original: &str,
    count: usize,
    observer: &crate::observe::Observer,
) -> Result<(), String> {
    select_backward(count, true).await?;
    for attempt in 0..3 {
        let selected = copy_selected(original.len()).await?;
        let prefix = crate::correction::unselected_prefix(original, &selected)?;
        if prefix.is_empty() {
            observer.phase(
                "correct.selection",
                &format!("verified {count} graphemes against saved destination text"),
            );
            return Ok(());
        }
        // Require a grapheme boundary, not merely a UTF-8 boundary, before
        // extending a selection through arbitrary Unicode text.
        use unicode_segmentation::UnicodeSegmentation;
        if !original
            .grapheme_indices(true)
            .any(|(i, _)| i == prefix.len())
        {
            return Err("Editor selection splits a grapheme; use Copy".into());
        }
        if attempt == 2 {
            break;
        }
        let missing = prefix.graphemes(true).count();
        observer.phase(
            "correct.selection",
            &format!(
                "read-back is short by {missing} graphemes; extending selection without deleting"
            ),
        );
        select_backward(missing, false).await?;
    }
    Err("Could not verify the whole saved dictation; nothing replaced. Use Copy".into())
}

async fn select_backward(count: usize, restore_focus: bool) -> Result<(), String> {
    let mut command = tokio::process::Command::new("wtype");
    // The review surface has just closed. Its process exiting does not mean
    // the compositor has already restored keyboard focus to the editor. Give
    // it a frame budget before the first selection key; otherwise a prefix
    // can remain when the first few Left events miss the editor.
    if restore_focus {
        command.args(["-s", "150"]);
    }
    command.args(["-M", "shift"]);
    for _ in 0..count {
        command.args(["-k", "Left"]);
    }
    command.args(["-m", "shift"]);
    let status = command
        .status()
        .await
        .map_err(|e| format!("Could not replace with wtype: {e}"))?;
    if !status.success() {
        return Err(format!(
            "Selection failed ({status}); use the corrected clipboard text"
        ));
    }
    Ok(())
}

async fn copy_selected(expected_bytes: usize) -> Result<String, String> {
    let marker = format!(
        "transcrust-selection-{}-{:?}",
        std::process::id(),
        std::time::SystemTime::now()
    );
    copy_to_clipboard(&marker)?;
    let status = tokio::process::Command::new("wtype")
        .args(["-M", "ctrl", "-k", "c", "-m", "ctrl"])
        .status()
        .await
        .map_err(|e| format!("Could not copy editor selection: {e}"))?;
    if !status.success() {
        return Err(format!(
            "Could not copy editor selection ({status}); nothing replaced"
        ));
    }
    // Clipboard ownership is asynchronous. A sentinel prevents treating our
    // previous clipboard contents as proof that the editor copied anything.
    let max_bytes = expected_bytes.max(marker.len()) + 1;
    for _ in 0..10 {
        let read = tokio::task::spawn_blocking(move || {
            use std::io::Read;
            use wl_clipboard_rs::paste::{get_contents, ClipboardType, MimeType, Seat};
            let (reader, _) =
                get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text)
                    .map_err(|e| e.to_string())?;
            let mut text = String::new();
            reader
                .take(max_bytes as u64)
                .read_to_string(&mut text)
                .map_err(|e| e.to_string())?;
            Ok::<_, String>(text)
        });
        let text = tokio::time::timeout(std::time::Duration::from_millis(500), read)
            .await
            .map_err(|_| "Editor selection read timed out; nothing replaced".to_string())?
            .map_err(|e| e.to_string())??;
        if text != marker {
            return Ok(text);
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    Err("The editor did not copy its selection; nothing replaced. Use Copy".into())
}
