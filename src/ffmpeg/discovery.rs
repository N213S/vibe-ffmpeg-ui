use crate::models::media::LocalFfmpegInfo;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Configures a child command so Windows does not flash console windows.
pub fn configure_background_command(cmd: &mut Command) {
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.stdin(Stdio::null());
}

/// Locates `ffmpeg` and `ffprobe`, preferring binaries beside our executable, then PATH.
pub fn discover_local_ffmpeg() -> LocalFfmpegInfo {
    let ffmpeg_path = find_binary("ffmpeg");
    let ffprobe_path = find_binary("ffprobe");

    let mut info = LocalFfmpegInfo {
        ffmpeg_path: ffmpeg_path.clone(),
        ffprobe_path,
        raw_version_line: String::new(),
        semantic_version: None,
        available_encoders: Vec::new(),
    };

    if let Some(ref ff) = ffmpeg_path {
        if let Some((raw, semver)) = read_ffmpeg_version(ff) {
            info.raw_version_line = raw;
            info.semantic_version = semver;
        }
        info.available_encoders = read_available_encoders(ff);
    }

    info
}

fn find_binary(name: &str) -> Option<PathBuf> {
    let exe_name = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };

    if let Ok(current_exe) = std::env::current_exe()
        && let Some(dir) = current_exe.parent()
    {
        let sibling = dir.join(&exe_name);
        if sibling.is_file() {
            return Some(sibling);
        }
    }

    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join(&exe_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

fn read_ffmpeg_version(ffmpeg_path: &PathBuf) -> Option<(String, Option<String>)> {
    let mut cmd = Command::new(ffmpeg_path);
    cmd.arg("-version");
    configure_background_command(&mut cmd);
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let first_line = stdout.lines().next()?.trim().to_string();
    let semver = extract_version_token(&first_line);
    Some((first_line, semver))
}

/// Extracts a clean version token like `8.0.1` or `2026-11-11-git-...` from `ffmpeg version ...`.
pub fn extract_version_token(first_line: &str) -> Option<String> {
    let rest = first_line.strip_prefix("ffmpeg version ")?;
    let token = rest.split_whitespace().next()?;
    let base = token.split('-').next().unwrap_or(token);
    if base.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        Some(base.to_string())
    } else {
        Some(token.to_string())
    }
}

fn read_available_encoders(ffmpeg_path: &PathBuf) -> Vec<String> {
    let mut cmd = Command::new(ffmpeg_path);
    cmd.args(["-hide_banner", "-encoders"]);
    configure_background_command(&mut cmd);
    let Ok(out) = cmd.output() else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut encoders = Vec::new();
    let mut past_header = false;
    for line in stdout.lines() {
        if line.trim_start().starts_with("------") {
            past_header = true;
            continue;
        }
        if !past_header {
            continue;
        }
        let mut parts = line.split_whitespace();
        let _flags = parts.next();
        if let Some(name) = parts.next() {
            encoders.push(name.to_string());
        }
    }
    encoders
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gyan_ffmpeg_version_line() {
        let line = "ffmpeg version 8.0.1-full_build-www.gyan.dev Copyright (c) 2000-2025 the FFmpeg developers";
        assert_eq!(extract_version_token(line), Some("8.0.1".to_string()));
    }
}
