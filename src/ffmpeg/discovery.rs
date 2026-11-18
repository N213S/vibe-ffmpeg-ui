use crate::models::media::LocalFfmpegInfo;
use std::path::{Path, PathBuf};
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

/// Returns the directory containing the running executable, if available.
pub fn app_executable_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
}

fn find_binary(name: &str) -> Option<PathBuf> {
    let exe_name = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };

    if let Some(dir) = app_executable_dir() {
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

    #[cfg(windows)]
    {
        if let Some(found) = find_windows_fallback_binary(&exe_name) {
            return Some(found);
        }
    }

    None
}

#[cfg(windows)]
fn find_windows_fallback_binary(exe_name: &str) -> Option<PathBuf> {
    if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
        let winget_root = PathBuf::from(local_appdata)
            .join("Microsoft")
            .join("WinGet");

        let link_candidate = winget_root.join("Links").join(exe_name);
        if link_candidate.is_file() {
            return Some(link_candidate);
        }

        let packages_dir = winget_root.join("Packages");
        if let Ok(pkg_entries) = std::fs::read_dir(&packages_dir) {
            for pkg in pkg_entries.flatten() {
                let pkg_name = pkg.file_name().to_string_lossy().to_ascii_lowercase();
                if !pkg_name.starts_with("gyan.ffmpeg") {
                    continue;
                }
                let direct_bin = pkg.path().join("bin").join(exe_name);
                if direct_bin.is_file() {
                    return Some(direct_bin);
                }
                if let Ok(sub_entries) = std::fs::read_dir(pkg.path()) {
                    for sub in sub_entries.flatten() {
                        let nested_bin = sub.path().join("bin").join(exe_name);
                        if nested_bin.is_file() {
                            return Some(nested_bin);
                        }
                    }
                }
            }
        }
    }

    for key in [
        "HKCU\\Environment",
        "HKLM\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment",
    ] {
        if let Some(raw_path) = query_windows_registry_path(key) {
            let expanded = expand_windows_env_vars(&raw_path, |var| std::env::var(var).ok());
            for dir in std::env::split_paths(&expanded) {
                if !dir.is_absolute() {
                    continue;
                }
                let candidate = dir.join(exe_name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

#[cfg(windows)]
fn query_windows_registry_path(subkey: &str) -> Option<String> {
    let sys_root = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("C:\\Windows"));
    let reg_exe = sys_root.join("System32").join("reg.exe");
    let mut cmd = Command::new(reg_exe);
    cmd.args(["query", subkey, "/v", "Path"]);
    configure_background_command(&mut cmd);
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    for line in stdout.lines() {
        if let Some(idx) = line.find("REG_EXPAND_SZ").or_else(|| line.find("REG_SZ")) {
            let marker_len = if line[idx..].starts_with("REG_EXPAND_SZ") {
                "REG_EXPAND_SZ".len()
            } else {
                "REG_SZ".len()
            };
            let value = line[idx + marker_len..].trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Expands `%VAR%` placeholders commonly found in Windows Registry `Path` values.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn expand_windows_env_vars<F>(input: &str, mut lookup: F) -> String
where
    F: FnMut(&str) -> Option<String>,
{
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after_first = &rest[start + 1..];
        if let Some(end) = after_first.find('%') {
            let var_name = &after_first[..end];
            if !var_name.is_empty()
                && let Some(val) = lookup(var_name)
            {
                out.push_str(&val);
            } else {
                out.push('%');
                out.push_str(var_name);
                out.push('%');
            }
            rest = &after_first[end + 1..];
        } else {
            out.push('%');
            out.push_str(after_first);
            return out;
        }
    }
    out.push_str(rest);
    out
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
    use std::collections::HashMap;

    #[test]
    fn parses_gyan_ffmpeg_version_line() {
        let line = "ffmpeg version 8.0.1-full_build-www.gyan.dev Copyright (c) 2000-2025 the FFmpeg developers";
        assert_eq!(extract_version_token(line), Some("8.0.1".to_string()));
    }

    #[test]
    fn expands_windows_registry_env_vars() {
        let vars = HashMap::from([
            ("LOCALAPPDATA", "C:\\Users\\Alice\\AppData\\Local"),
            ("SystemRoot", "C:\\Windows"),
        ]);
        let expanded = expand_windows_env_vars(
            "%SystemRoot%\\system32;%LOCALAPPDATA%\\Microsoft\\WinGet\\Links;%UNKNOWN%\\bin",
            |k| vars.get(k).map(|v| (*v).to_string()),
        );
        assert_eq!(
            expanded,
            "C:\\Windows\\system32;C:\\Users\\Alice\\AppData\\Local\\Microsoft\\WinGet\\Links;%UNKNOWN%\\bin"
        );
    }
}
