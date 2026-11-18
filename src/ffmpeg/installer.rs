use crate::ffmpeg::discovery::{configure_background_command, discover_local_ffmpeg};
use crate::models::media::WorkerEvent;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;

pub const WINGET_FFMPEG_PACKAGE_ID: &str = "Gyan.FFmpeg";

/// Returns strict constant CLI arguments passed directly to `winget.exe` (no shell).
pub fn winget_install_args() -> &'static [&'static str] {
    &[
        "install",
        "--id",
        WINGET_FFMPEG_PACKAGE_ID,
        "--exact",
        "--source",
        "winget",
        "--accept-package-agreements",
        "--accept-source-agreements",
        "--disable-interactivity",
    ]
}

/// Locates `winget.exe` safely using absolute WindowsApps/PATH directories (avoiding CWD hijacking).
pub fn find_winget_exe() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
            let alias = PathBuf::from(local_appdata)
                .join("Microsoft")
                .join("WindowsApps")
                .join("winget.exe");
            if alias.is_file() {
                return Some(alias);
            }
        }

        if let Some(paths) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&paths) {
                if !dir.is_absolute() {
                    continue;
                }
                let candidate = dir.join("winget.exe");
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        None
    }

    #[cfg(not(windows))]
    {
        None
    }
}

/// Spawns a non-blocking FFmpeg re-discovery pass off the UI thread.
pub fn spawn_ffmpeg_rescan(tx: Sender<WorkerEvent>, ctx: eframe::egui::Context) {
    thread::spawn(move || {
        let info = discover_local_ffmpeg();
        let _ = tx.send(WorkerEvent::FfmpegRediscovered(info));
        ctx.request_repaint();
    });
}

/// Installs `Gyan.FFmpeg` non-interactively via `winget.exe` and triggers FFmpeg re-discovery on completion.
pub fn spawn_winget_install(tx: Sender<WorkerEvent>, ctx: eframe::egui::Context) {
    thread::spawn(move || {
        let Some(winget_path) = find_winget_exe() else {
            let _ = tx.send(WorkerEvent::FfmpegInstallFinished(Err(
                "winget.exe was not found on this Windows installation.".to_string(),
            )));
            ctx.request_repaint();
            return;
        };

        let _ = tx.send(WorkerEvent::FfmpegInstallProgress(
            "Running: winget install --id Gyan.FFmpeg --exact ...".to_string(),
        ));
        ctx.request_repaint();

        let mut cmd = Command::new(&winget_path);
        cmd.args(winget_install_args());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        configure_background_command(&mut cmd);

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(WorkerEvent::FfmpegInstallFinished(Err(format!(
                    "Failed to launch winget: {e}"
                ))));
                ctx.request_repaint();
                return;
            }
        };

        let stdout_handle = child.stdout.take().map(|reader| {
            let tx_out = tx.clone();
            let ctx_out = ctx.clone();
            thread::spawn(move || read_winget_stream(reader, tx_out, ctx_out))
        });

        let stderr_handle = child.stderr.take().map(|reader| {
            let tx_err = tx.clone();
            let ctx_err = ctx.clone();
            thread::spawn(move || read_winget_stream(reader, tx_err, ctx_err))
        });

        let status_res = child.wait();
        let out_last = stdout_handle.and_then(|h| h.join().ok()).flatten();
        let err_last = stderr_handle.and_then(|h| h.join().ok()).flatten();

        match status_res {
            Ok(status) if status.success() => {
                let _ = tx.send(WorkerEvent::FfmpegInstallFinished(Ok(())));
                let discovered = discover_local_ffmpeg();
                let _ = tx.send(WorkerEvent::FfmpegRediscovered(discovered));
            }
            Ok(status) => {
                let detail = err_last
                    .or(out_last)
                    .unwrap_or_else(|| format!("winget exited with status {status}"));
                let _ = tx.send(WorkerEvent::FfmpegInstallFinished(Err(format!(
                    "{detail} (exit: {status})"
                ))));
                let discovered = discover_local_ffmpeg();
                if discovered.is_ready() {
                    let _ = tx.send(WorkerEvent::FfmpegRediscovered(discovered));
                }
            }
            Err(e) => {
                let _ = tx.send(WorkerEvent::FfmpegInstallFinished(Err(format!(
                    "Failed waiting for winget process: {e}"
                ))));
            }
        }

        ctx.request_repaint();
    });
}

fn read_winget_stream<R: Read>(
    mut reader: R,
    tx: Sender<WorkerEvent>,
    ctx: eframe::egui::Context,
) -> Option<String> {
    let mut buf = [0u8; 1024];
    let mut pending = Vec::with_capacity(256);
    let mut last_meaningful: Option<String> = None;

    while let Ok(n) = reader.read(&mut buf) {
        if n == 0 {
            break;
        }
        for &b in &buf[..n] {
            if b == b'\r' || b == b'\n' {
                if !pending.is_empty() {
                    let text = String::from_utf8_lossy(&pending);
                    if let Some(clean) = sanitize_console_line(&text)
                        && last_meaningful.as_deref() != Some(clean.as_str())
                    {
                        last_meaningful = Some(clean.clone());
                        let _ = tx.send(WorkerEvent::FfmpegInstallProgress(clean));
                        ctx.request_repaint();
                    }
                    pending.clear();
                }
            } else {
                pending.push(b);
            }
        }
    }

    if !pending.is_empty() {
        let text = String::from_utf8_lossy(&pending);
        if let Some(clean) = sanitize_console_line(&text)
            && last_meaningful.as_deref() != Some(clean.as_str())
        {
            last_meaningful = Some(clean.clone());
            let _ = tx.send(WorkerEvent::FfmpegInstallProgress(clean));
            ctx.request_repaint();
        }
    }

    last_meaningful
}

/// Strips ANSI escapes and noisy spinner/progress bar fragments from console output lines.
pub fn sanitize_console_line(raw: &str) -> Option<String> {
    let stripped = strip_ansi_sequences(raw);
    let cleaned: String = stripped
        .chars()
        .filter(|c| !c.is_control())
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        return None;
    }
    if matches!(trimmed, "-" | "\\" | "|" | "/") {
        return None;
    }
    let has_alnum = trimmed.chars().any(|c| c.is_alphanumeric());
    if !has_alnum {
        return None;
    }
    Some(trimmed.to_string())
}

fn strip_ansi_sequences(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.peek().copied() {
                Some('[') => {
                    chars.next();
                    for seq_ch in chars.by_ref() {
                        if ('@'..='~').contains(&seq_ch) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next();
                    while let Some(seq_ch) = chars.next() {
                        if seq_ch == '\x07' {
                            break;
                        }
                        if seq_ch == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn winget_args_target_exact_official_package_non_interactively() {
        let args = winget_install_args();
        assert!(args.contains(&"Gyan.FFmpeg"));
        assert!(args.contains(&"--exact"));
        assert!(args.contains(&"--accept-package-agreements"));
        assert!(args.contains(&"--accept-source-agreements"));
        assert!(args.contains(&"--disable-interactivity"));
    }

    #[test]
    fn sanitizes_winget_ansi_and_spinners() {
        assert_eq!(sanitize_console_line("\x1b[32m-\x1b[0m"), None);
        assert_eq!(sanitize_console_line("   \\   "), None);
        assert_eq!(sanitize_console_line("  ██████  "), None);
        assert_eq!(
            sanitize_console_line("\x1b[1KDownloading https://www.gyan.dev/ffmpeg.7z"),
            Some("Downloading https://www.gyan.dev/ffmpeg.7z".to_string())
        );
    }
}
