use std::fs;
use std::path::{Path, PathBuf};

const MAX_COLLISION_ATTEMPTS: u32 = 10_000;
const TRUSTED_HTTPS_PREFIXES: &[&str] = &[
    "https://www.gyan.dev/ffmpeg/",
    "https://ffmpeg.org/",
];

pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "mov", "avi", "webm", "flv", "ts", "m4v", "wmv", "mpg", "mpeg", "3gp",
];
pub const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "wav", "flac", "m4a", "aac", "ogg", "opus", "wma",
];
pub const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "bmp", "tiff", "tif", "gif", "avif",
];

/// Checks whether a file path has a recognized video, audio, or image extension.
pub fn is_supported_media_extension(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    let lower = ext.to_ascii_lowercase();
    VIDEO_EXTENSIONS.contains(&lower.as_str())
        || AUDIO_EXTENSIONS.contains(&lower.as_str())
        || IMAGE_EXTENSIONS.contains(&lower.as_str())
}

/// Validates that `input` is an existing regular file without dangerous path artifacts.
pub fn validate_input_file(input: &Path) -> Result<PathBuf, String> {
    let raw = input.to_string_lossy();
    if raw.is_empty() || raw.contains('\0') {
        return Err("Invalid empty or null-containing input path.".to_string());
    }
    if !input.exists() {
        return Err(format!("Input file does not exist: {}", input.display()));
    }
    if !input.is_file() {
        return Err(format!("Input path is not a regular file: {}", input.display()));
    }
    fs::canonicalize(input)
        .map_err(|e| format!("Failed to resolve input path '{}': {e}", input.display()))
}

/// Strips Windows-forbidden filename characters and control chars while keeping Unicode/Thai intact.
pub fn sanitize_file_stem(stem: &str) -> String {
    let cleaned: String = stem
        .chars()
        .filter(|&c| {
            !c.is_control()
                && !matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
        })
        .collect();
    let trimmed = cleaned.trim().trim_end_matches('.');
    if trimmed.is_empty() {
        "converted_media".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Sanitizes an optional prefix or suffix fragment without injecting fallback text when empty.
pub fn sanitize_affix(affix: &str) -> String {
    affix
        .chars()
        .filter(|&c| {
            !c.is_control()
                && !matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
        })
        .collect::<String>()
        .replace("..", "_")
}

fn is_windows_reserved_name(stem: &str) -> bool {
    let upper = stem.to_ascii_uppercase();
    matches!(
        upper.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

/// Combines custom stem, prefix, suffix, and auto-synced extension into a safe filename.
pub fn compose_output_filename(
    custom_stem: &str,
    prefix: &str,
    suffix: &str,
    ext: &str,
) -> String {
    let clean_ext = ext.trim_start_matches('.');
    let mut stem = sanitize_file_stem(custom_stem);
    let suffix_dot = format!(".{clean_ext}");
    if stem.len() > suffix_dot.len()
        && stem[stem.len() - suffix_dot.len()..].eq_ignore_ascii_case(&suffix_dot)
    {
        stem.truncate(stem.len() - suffix_dot.len());
        stem = sanitize_file_stem(&stem);
    }

    let clean_prefix = sanitize_affix(prefix.trim_start());
    let clean_suffix = sanitize_affix(suffix.trim_end());
    let mut combined = format!("{clean_prefix}{stem}{clean_suffix}");
    combined = sanitize_file_stem(&combined);
    if is_windows_reserved_name(&combined) {
        combined.push_str("_media");
    }
    format!("{combined}.{clean_ext}")
}

/// Checks whether two paths refer to the same location (case-insensitive on Windows).
pub fn paths_refer_to_same_file(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| -> String {
        let resolved = fs::canonicalize(p).unwrap_or_else(|_| {
            if let (Some(parent), Some(name)) = (p.parent(), p.file_name()) {
                let canon_parent =
                    fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
                canon_parent.join(name)
            } else {
                p.to_path_buf()
            }
        });
        let s = resolved.to_string_lossy().into_owned();
        if cfg!(windows) {
            s.to_lowercase()
        } else {
            s
        }
    };
    norm(a) == norm(b)
}

/// Resolves a collision-safe, non-overwriting output path using custom stem + global prefix/suffix.
pub fn build_safe_output_path(
    input: &Path,
    custom_stem: Option<&str>,
    prefix: &str,
    suffix: &str,
    output_dir: Option<&Path>,
    ext: &str,
    overwrite_existing: bool,
) -> Result<PathBuf, String> {
    let canonical_input = validate_input_file(input)?;

    let target_dir = match output_dir {
        Some(dir) => {
            if !dir.exists() || !dir.is_dir() {
                return Err(format!(
                    "Selected output directory does not exist: {}",
                    dir.display()
                ));
            }
            fs::canonicalize(dir)
                .map_err(|e| format!("Failed to resolve output directory: {e}"))?
        }
        None => canonical_input
            .parent()
            .ok_or_else(|| "Input path has no parent directory.".to_string())?
            .to_path_buf(),
    };

    let clean_ext = ext.trim_start_matches('.');
    if clean_ext.is_empty() || !clean_ext.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(format!("Invalid output extension: {ext}"));
    }

    let fallback_stem = input
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_string());
    let chosen_stem = custom_stem
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&fallback_stem);

    let file_name = compose_output_filename(chosen_stem, prefix, suffix, clean_ext);
    let first_candidate = target_dir.join(&file_name);

    // Never overwrite or truncate the active input file in place.
    let candidate = if paths_refer_to_same_file(&canonical_input, &first_candidate) {
        let base_stem = first_candidate
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "output".to_string());
        target_dir.join(format!("{base_stem}_out.{clean_ext}"))
    } else {
        first_candidate
    };

    if overwrite_existing {
        if paths_refer_to_same_file(&canonical_input, &candidate) {
            return Err("Refusing to overwrite input file with output.".to_string());
        }
        return Ok(candidate);
    }

    if !candidate.exists() {
        return Ok(candidate);
    }

    let base_stem = candidate
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_string());
    for idx in 1..=MAX_COLLISION_ATTEMPTS {
        let numbered = target_dir.join(format!("{base_stem} ({idx}).{clean_ext}"));
        if !numbered.exists() && !paths_refer_to_same_file(&canonical_input, &numbered) {
            return Ok(numbered);
        }
    }

    Err("Too many filename collisions in output directory.".to_string())
}

/// Creates a sibling temporary file retaining the target extension after `.tmp` (e.g. `name.ffui_123_1.tmp.mp4`).
pub fn build_temp_staging_path(final_output: &Path, job_id: u64) -> Result<PathBuf, String> {
    let parent = final_output
        .parent()
        .ok_or_else(|| "Output path has no parent directory.".to_string())?;
    let ext = final_output
        .extension()
        .and_then(|e| e.to_str())
        .filter(|e| !e.is_empty() && e.chars().all(|c| c.is_ascii_alphanumeric()))
        .ok_or_else(|| "Output path is missing a valid extension.".to_string())?;
    let stem = final_output
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output")
        .replace('%', "_");
    let pid = std::process::id();
    Ok(parent.join(format!(".{stem}.{pid}_{job_id}.tmp.{ext}")))
}

/// Atomically promotes a validated temporary output file to its final location.
pub fn commit_temp_file(
    temp_path: &Path,
    final_output: &Path,
    input_path: &Path,
    overwrite_existing: bool,
) -> Result<(), String> {
    if paths_refer_to_same_file(input_path, final_output) {
        return Err("Blocked commit: destination matches input file.".to_string());
    }
    let meta = fs::metadata(temp_path)
        .map_err(|e| format!("Temporary output missing before commit: {e}"))?;
    if meta.len() == 0 {
        return Err("Temporary output is empty (0 bytes); aborting commit.".to_string());
    }
    if !overwrite_existing && final_output.exists() {
        return Err(format!(
            "Destination already exists and overwrite is disabled: {}",
            final_output.display()
        ));
    }

    fs::rename(temp_path, final_output)
        .map_err(|e| format!("Failed to move verified output to destination: {e}"))
}

/// Validates that an external URL uses HTTPS and belongs to an allowlisted official origin.
pub fn validate_trusted_https_url(url: &str) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("Only HTTPS URLs are permitted.".to_string());
    }
    if TRUSTED_HTTPS_PREFIXES.iter().any(|p| url.starts_with(p)) {
        Ok(())
    } else {
        Err(format!("Untrusted URL host rejected: {url}"))
    }
}

/// Strictly sanitizes version strings fetched from remote plaintext endpoints.
pub fn sanitize_remote_version(raw: &str) -> Option<String> {
    let first_line = raw.lines().next()?.trim();
    if first_line.is_empty() || first_line.len() > 64 {
        return None;
    }
    if first_line
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
    {
        Some(first_line.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_test_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ffui_test_{tag}_{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn never_overwrites_input_file_in_same_folder() {
        let dir = make_temp_test_dir("same_folder");
        let input = dir.join("clip.mp4");
        fs::write(&input, b"dummy").unwrap();

        let out = build_safe_output_path(&input, Some("clip"), "", "", None, "mp4", true).unwrap();
        assert!(!paths_refer_to_same_file(&input, &out));
        assert_eq!(out.file_name().unwrap().to_string_lossy(), "clip_out.mp4");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn applies_custom_prefix_suffix_and_auto_extension() {
        let dir = make_temp_test_dir("affix");
        let input = dir.join("video.mkv");
        fs::write(&input, b"dummy").unwrap();

        let out = build_safe_output_path(
            &input,
            Some("my_edit.mp4"),
            "final_",
            "_web",
            None,
            "mp4",
            false,
        )
        .unwrap();
        assert_eq!(
            out.file_name().unwrap().to_string_lossy(),
            "final_my_edit_web.mp4"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn staging_path_retains_tmp_and_target_extension() {
        let p = PathBuf::from("C:\\videos\\sample%20.mp4");
        let staged = build_temp_staging_path(&p, 7).unwrap();
        let name = staged.file_name().unwrap().to_string_lossy();
        assert!(name.contains(".tmp.mp4"));
        assert!(!name.contains('%'));
    }

    #[test]
    fn preserves_thai_unicode_and_strips_illegal_chars() {
        let cleaned = sanitize_file_stem("วิดีโอ ทดสอบ:ตอน/ที่*1?");
        assert_eq!(cleaned, "วิดีโอ ทดสอบตอนที่1");
    }

    #[test]
    fn increments_suffix_when_overwrite_disabled() {
        let dir = make_temp_test_dir("collision");
        let input = dir.join("source.mkv");
        let existing = dir.join("source.mp4");
        fs::write(&input, b"in").unwrap();
        fs::write(&existing, b"out").unwrap();

        let out = build_safe_output_path(&input, None, "", "", None, "mp4", false).unwrap();
        assert_eq!(out.file_name().unwrap().to_string_lossy(), "source (1).mp4");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn sanitizes_remote_version_strings() {
        assert_eq!(sanitize_remote_version("9.0.2\n"), Some("9.0.2".to_string()));
        assert_eq!(
            sanitize_remote_version("2026-11-11-git-23c4540053"),
            Some("2026-11-11-git-23c4540053".to_string())
        );
        assert_eq!(sanitize_remote_version("<script>alert(1)</script>"), None);
        assert_eq!(sanitize_remote_version(""), None);
    }

    #[test]
    fn enforces_trusted_https_allowlist() {
        assert!(validate_trusted_https_url("https://www.gyan.dev/ffmpeg/builds/").is_ok());
        assert!(validate_trusted_https_url("http://www.gyan.dev/ffmpeg/builds/").is_err());
        assert!(validate_trusted_https_url("https://evil.example/payload").is_err());
    }
}
