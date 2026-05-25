use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct Observer {
    inner: Arc<Inner>,
}

struct Inner {
    file: Mutex<File>,
    latest_path: PathBuf,
    sample_chars: usize,
    desktop_notifications: bool,
    echo_stderr: bool,
}

impl Observer {
    pub fn new(
        sample_chars: usize,
        desktop_notifications: bool,
        echo_stderr: bool,
    ) -> Result<Self, String> {
        let log_dir = std::env::current_dir()
            .map_err(|e| format!("failed to read cwd: {e}"))?
            .join("logs");
        fs::create_dir_all(&log_dir).map_err(|e| format!("failed to create log dir: {e}"))?;

        let latest_path = log_dir.join("latest.log");
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&latest_path)
            .map_err(|e| format!("failed to open log file: {e}"))?;

        Ok(Self {
            inner: Arc::new(Inner {
                file: Mutex::new(file),
                latest_path,
                sample_chars,
                desktop_notifications,
                echo_stderr,
            }),
        })
    }

    pub fn log_path(&self) -> &PathBuf {
        &self.inner.latest_path
    }

    pub fn phase(&self, phase: &str, message: &str) {
        self.write("INFO", phase, message);
    }

    pub fn sample(&self, phase: &str, text: &str) {
        let sample = truncate(text, self.inner.sample_chars);
        self.write("SAMPLE", phase, &sample);
    }

    pub fn error(&self, phase: &str, message: &str) {
        self.write("ERROR", phase, message);
        self.notify("Transcrust error", &truncate(message, 180));
    }

    pub fn notify(&self, title: &str, body: &str) {
        if !self.inner.desktop_notifications {
            return;
        }

        let _ = std::process::Command::new("notify-send")
            .arg(title)
            .arg(body)
            .status();
    }

    fn write(&self, level: &str, phase: &str, message: &str) {
        let timestamp = timestamp_secs();
        let line = format!("[{timestamp}] {level} {phase}: {message}\n");
        if let Ok(mut file) = self.inner.file.lock() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
        if self.inner.echo_stderr {
            eprint!("{line}");
        }
    }
}

fn timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn truncate(text: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (idx, ch) in text.chars().enumerate() {
        if idx >= max_chars {
            out.push_str("...");
            return out;
        }
        out.push(ch);
    }
    out
}
