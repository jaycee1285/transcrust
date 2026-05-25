use wl_clipboard_rs::copy::{MimeType, Options, Source};

use crate::config::OutputConfig;

pub async fn inject_text(text: &str, config: &OutputConfig) -> Result<(), String> {
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
            Err(e) => {
                match type_with_dotool(text).await {
                    Ok(()) => successes += 1,
                    Err(e2) => errors.push(format!("typing: wtype={e}, dotool={e2}")),
                }
            }
        }
    }

    if successes > 0 {
        Ok(())
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
