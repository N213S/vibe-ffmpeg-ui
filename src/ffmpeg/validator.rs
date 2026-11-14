use crate::ffmpeg::discovery::configure_background_command;
use crate::ffmpeg::probe::probe_media;
use crate::models::config::{ContainerFormat, ConvertMode, ConversionSettings};
use crate::models::media::{MediaInfo, ValidationReport};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;

/// Validates a finished temporary output file according to `ConvertMode` before atomic commit.
pub fn validate_output_file(
    ffmpeg_path: &Path,
    ffprobe_path: &Path,
    temp_output: &Path,
    settings: &ConversionSettings,
    source_media: Option<&MediaInfo>,
) -> Result<ValidationReport, String> {
    let meta = fs::metadata(temp_output)
        .map_err(|e| format!("Output file missing after conversion: {e}"))?;
    let size_bytes = meta.len();
    if size_bytes == 0 {
        return Err("Output file is 0 bytes (corrupted/empty).".to_string());
    }

    let info = probe_media(ffprobe_path, temp_output)
        .map_err(|e| format!("Output ffprobe validation failed: {e}"))?;

    let stream_count = usize::from(info.has_video) + usize::from(info.has_audio);
    validate_streams_for_mode(&info, settings, source_media)?;

    let mut deep_decode_passed = false;
    let mut verification_note = String::from("Header OK");

    if settings.safety.deep_decode_verify {
        if settings.mode == ConvertMode::ImageGif
            && settings.container == ContainerFormat::Webp
            && settings.image_gif.webp_animated
        {
            verify_animated_webp_container(temp_output)?;
            deep_decode_passed = true;
            verification_note = String::from("RIFF/ANIM Verified");
        } else {
            run_deep_decode_check(ffmpeg_path, temp_output)?;
            deep_decode_passed = true;
            verification_note = String::from("Decode Clean");
        }
    }

    Ok(ValidationReport {
        size_bytes,
        duration_secs: info.duration_secs,
        stream_count,
        deep_decode_passed,
        verification_note,
    })
}

/// Checks expected streams and duration rules for each `ConvertMode`.
pub fn validate_streams_for_mode(
    info: &MediaInfo,
    settings: &ConversionSettings,
    source_media: Option<&MediaInfo>,
) -> Result<(), String> {
    match settings.mode {
        ConvertMode::Video | ConvertMode::Remux => {
            if !info.has_video && !info.has_audio {
                return Err("Output contains no valid video or audio streams.".to_string());
            }
            if settings.mode == ConvertMode::Video
                && source_media.is_some_and(|m| m.has_video)
                && !info.has_video
            {
                return Err("Expected video stream is missing from converted output.".to_string());
            }
            match info.duration_secs {
                Some(dur) if dur.is_finite() && dur > 0.0 => {}
                other => {
                    return Err(format!(
                        "Video/Remux output reported invalid duration: {other:?}"
                    ));
                }
            }
        }
        ConvertMode::AudioOnly => {
            if !info.has_audio {
                return Err("Audio Only output contains no valid audio stream.".to_string());
            }
            match info.duration_secs {
                Some(dur) if dur.is_finite() && dur > 0.0 => {}
                other => {
                    return Err(format!(
                        "Audio Only output reported invalid duration: {other:?}"
                    ));
                }
            }
        }
        ConvertMode::ImageGif => {
            if !info.has_video {
                return Err("Image / GIF output contains no valid image/video stream.".to_string());
            }
            // Static formats report explicit pixel dimensions; animated WebP reports 0x0 in ffprobe.
            let is_animated_webp = settings.container == ContainerFormat::Webp
                && settings.image_gif.webp_animated;
            if !is_animated_webp
                && (info.width.unwrap_or(0) == 0 || info.height.unwrap_or(0) == 0)
            {
                return Err("Image output reported zero frame dimensions.".to_string());
            }
        }
    }
    Ok(())
}

/// Verifies RIFF/WEBP header magic bytes for animated WebP (FFmpeg's webp decoder does not decode ANIM frames).
fn verify_animated_webp_container(path: &Path) -> Result<(), String> {
    let mut file = fs::File::open(path)
        .map_err(|e| format!("Failed to open animated WebP for verification: {e}"))?;
    let mut header = [0u8; 64];
    let n = file
        .read(&mut header)
        .map_err(|e| format!("Failed reading WebP header: {e}"))?;
    if n < 16 || &header[0..4] != b"RIFF" || &header[8..12] != b"WEBP" {
        return Err("Animated WebP failed RIFF/WEBP header signature check.".to_string());
    }
    Ok(())
}

/// Decodes every packet of the generated file into `-f null -` with `-xerror`
/// so truncated frames or bitstream corruption are caught immediately.
fn run_deep_decode_check(ffmpeg_path: &Path, media_path: &Path) -> Result<(), String> {
    let mut cmd = Command::new(ffmpeg_path);
    cmd.args(["-hide_banner", "-nostdin", "-v", "error", "-xerror", "-i"]);
    cmd.arg(media_path);
    cmd.args(["-f", "null", "-"]);
    configure_background_command(&mut cmd);

    let out = cmd
        .output()
        .map_err(|e| format!("Failed to launch deep integrity verification: {e}"))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let snippet: String = stderr.lines().take(5).collect::<Vec<_>>().join(" | ");
        return Err(format!(
            "Deep decode verification detected stream errors (exit {}): {}",
            out.status,
            if snippet.is_empty() {
                "unknown decode error"
            } else {
                &snippet
            }
        ));
    }

    let stderr = String::from_utf8_lossy(&out.stderr);
    if !stderr.trim().is_empty() {
        let snippet: String = stderr.lines().take(5).collect::<Vec<_>>().join(" | ");
        return Err(format!(
            "Deep decode verification reported errors: {snippet}"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffmpeg::command_builder::build_command_plan;
    use crate::ffmpeg::discovery::discover_local_ffmpeg;
    use crate::models::config::PresetProfile;
    use crate::security::path_guard::{
        build_safe_output_path, build_temp_staging_path, commit_temp_file,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(tag: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ffui_e2e_{tag}_{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn mode_aware_verification_checks_rules_accurately() {
        let audio_settings = ConversionSettings::from_preset(PresetProfile::ExtractMp3320);
        let audio_only_info = MediaInfo {
            has_video: false,
            has_audio: true,
            duration_secs: Some(12.3),
            ..Default::default()
        };
        assert!(validate_streams_for_mode(&audio_only_info, &audio_settings, None).is_ok());

        let image_settings = ConversionSettings::from_preset(PresetProfile::StaticPngLossless);
        let static_img_info = MediaInfo {
            has_video: true,
            has_audio: false,
            width: Some(640),
            height: Some(360),
            duration_secs: None,
            ..Default::default()
        };
        assert!(validate_streams_for_mode(&static_img_info, &image_settings, None).is_ok());

        let video_settings = ConversionSettings::from_preset(PresetProfile::Mp4UniversalH264);
        assert!(validate_streams_for_mode(&static_img_info, &video_settings, None).is_err());
    }

    #[test]
    fn converts_odd_dimensions_and_passes_deep_integrity_verify() {
        let info = discover_local_ffmpeg();
        let ffmpeg = info.ffmpeg_path.expect("ffmpeg must be on PATH");
        let ffprobe = info.ffprobe_path.expect("ffprobe must be on PATH");
        let dir = make_temp_dir("video");

        let input_path = dir.join("odd & source.mkv");
        let mut gen_cmd = Command::new(&ffmpeg);
        gen_cmd.args([
            "-hide_banner",
            "-nostdin",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc=duration=1:size=321x241:rate=24",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=1",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv444p",
            "-c:a",
            "aac",
        ]);
        gen_cmd.arg(&input_path);
        configure_background_command(&mut gen_cmd);
        assert!(gen_cmd.status().unwrap().success());

        let media = probe_media(&ffprobe, &input_path).unwrap();
        let settings = ConversionSettings::from_preset(PresetProfile::Mp4UniversalH264);
        let final_out = build_safe_output_path(&input_path, None, "", "", None, "mp4", false).unwrap();
        let temp_out = build_temp_staging_path(&final_out, 1).unwrap();
        let plan = build_command_plan(&settings, Some(&media), &input_path, &temp_out).unwrap();

        let mut conv_cmd = Command::new(&ffmpeg);
        conv_cmd.args(&plan.args);
        configure_background_command(&mut conv_cmd);
        assert!(conv_cmd.status().unwrap().success());

        let report = validate_output_file(&ffmpeg, &ffprobe, &temp_out, &settings, Some(&media))
            .expect("output failed deep decode validation");
        assert!(report.deep_decode_passed);
        assert_eq!(report.stream_count, 2);

        commit_temp_file(&temp_out, &final_out, &input_path, false).unwrap();
        let verified_media = probe_media(&ffprobe, &final_out).unwrap();
        assert_eq!(verified_media.width, Some(320));
        assert_eq!(verified_media.height, Some(240));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn extracts_audio_only_and_static_png_with_mode_aware_verify() {
        let info = discover_local_ffmpeg();
        let ffmpeg = info.ffmpeg_path.expect("ffmpeg must be on PATH");
        let ffprobe = info.ffprobe_path.expect("ffprobe must be on PATH");
        let dir = make_temp_dir("multimode");

        let input_path = dir.join("clip.mp4");
        let mut gen_cmd = Command::new(&ffmpeg);
        gen_cmd.args([
            "-hide_banner",
            "-nostdin",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc=duration=1:size=128x72:rate=15",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=1",
            "-c:v",
            "libx264",
            "-c:a",
            "aac",
        ]);
        gen_cmd.arg(&input_path);
        configure_background_command(&mut gen_cmd);
        assert!(gen_cmd.status().unwrap().success());
        let media = probe_media(&ffprobe, &input_path).unwrap();

        // 1. Audio Only (MP3)
        let mp3_settings = ConversionSettings::from_preset(PresetProfile::ExtractMp3320);
        let mp3_final = build_safe_output_path(&input_path, None, "", "", None, "mp3", false).unwrap();
        let mp3_temp = build_temp_staging_path(&mp3_final, 2).unwrap();
        let mp3_plan = build_command_plan(&mp3_settings, Some(&media), &input_path, &mp3_temp).unwrap();
        let mut mp3_cmd = Command::new(&ffmpeg);
        mp3_cmd.args(&mp3_plan.args);
        configure_background_command(&mut mp3_cmd);
        assert!(mp3_cmd.status().unwrap().success());
        let mp3_rep = validate_output_file(&ffmpeg, &ffprobe, &mp3_temp, &mp3_settings, Some(&media)).unwrap();
        assert_eq!(mp3_rep.stream_count, 1);
        assert!(mp3_rep.deep_decode_passed);

        // 2. Image / PNG (zero duration)
        let png_settings = ConversionSettings::from_preset(PresetProfile::StaticPngLossless);
        let png_final = build_safe_output_path(&input_path, None, "", "", None, "png", false).unwrap();
        let png_temp = build_temp_staging_path(&png_final, 3).unwrap();
        let png_plan = build_command_plan(&png_settings, Some(&media), &input_path, &png_temp).unwrap();
        let mut png_cmd = Command::new(&ffmpeg);
        png_cmd.args(&png_plan.args);
        configure_background_command(&mut png_cmd);
        assert!(png_cmd.status().unwrap().success());
        let png_rep = validate_output_file(&ffmpeg, &ffprobe, &png_temp, &png_settings, Some(&media)).unwrap();
        assert_eq!(png_rep.stream_count, 1);
        assert!(png_rep.deep_decode_passed);

        let _ = fs::remove_dir_all(&dir);
    }
}
