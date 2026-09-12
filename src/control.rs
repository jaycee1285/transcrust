use std::path::PathBuf;

const PID_FILE_NAME: &str = "transcrust.pid";

fn control_dir() -> PathBuf {
    let candidates = [
        dirs::runtime_dir().map(|p| p.join("transcrust")),
        Some(std::env::temp_dir().join("transcrust")),
    ];

    for candidate in candidates.into_iter().flatten() {
        if std::fs::create_dir_all(&candidate).is_ok() {
            return candidate;
        }
    }

    PathBuf::from("/tmp/transcrust")
}

pub fn pid_file_path() -> PathBuf {
    control_dir().join(PID_FILE_NAME)
}

pub struct PidFileGuard {
    path: PathBuf,
}

impl Drop for PidFileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub fn write_pid_file() -> Result<PidFileGuard, String> {
    let path = pid_file_path();
    let pid = std::process::id().to_string();
    std::fs::write(&path, pid).map_err(|e| format!("failed to write pid file: {e}"))?;
    Ok(PidFileGuard { path })
}

/// Ask a running daemon to start or stop recording.
///
/// Same rails as [`request_quit`] — the pid file plus a signal — because the
/// daemon already owns that file and a second IPC mechanism would be a second
/// thing to go stale. `SIGUSR1` rather than `SIGTERM`, and the daemon only
/// listens for it when started with `--long`.
///
/// **This is the escape from evdev.** The default hold-to-talk path reads the
/// keyboard passively and therefore cannot stop a printable key reaching the
/// focused window. A compositor binding that runs `transcrust --toggle` has
/// none of that problem: labwc does the key handling, which is its job.
pub fn request_toggle() -> Result<(), String> {
    signal_daemon("USR1", "toggle")
}

/// Ask a running daemon to show its mode picker.
///
/// `SIGUSR2`, because `SIGUSR1` is taken by [`request_toggle`]. The daemon
/// registers a handler for it unconditionally: the default disposition is
/// *terminate*, so an unhandled `transcrust --pick` would kill the daemon
/// silently — the same trap SIGUSR1 already documents at `main.rs`.
///
/// The daemon spawns the picker rather than the CLI doing it, because the
/// daemon is the only process that knows the current mode list and is already
/// inside the user's Wayland session.
pub fn request_pick() -> Result<(), String> {
    signal_daemon("USR2", "pick")
}

pub fn request_quit() -> Result<(), String> {
    signal_daemon("TERM", "quit")
}

fn signal_daemon(signal: &str, what: &str) -> Result<(), String> {
    let path = pid_file_path();
    let pid_raw = std::fs::read_to_string(&path).map_err(|e| {
        format!(
            "failed to read pid file at {} ({e}). Is transcrust running?",
            path.display()
        )
    })?;
    let pid = pid_raw.trim();
    if pid.is_empty() {
        return Err("pid file is empty".into());
    }

    let status = std::process::Command::new("kill")
        .args([&format!("-{signal}"), pid])
        .status()
        .map_err(|e| format!("failed to execute kill: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{what} signal failed: kill exited with status {status}"))
    }
}
