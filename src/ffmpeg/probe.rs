use crate::ffmpeg::discovery::configure_background_command;
use crate::models::media::MediaInfo;
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    format: Option<FfprobeFormat>,
    #[serde(default)]
    streams: Vec<FfprobeStream>,
    error: Option<FfprobeError>,
}

#[derive(Debug, Deserialize)]
struct FfprobeError {
    string: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    format_name: Option<String>,
    duration: Option<String>,
    size: Option<String>,
    bit_rate: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    pix_fmt: Option<String>,
    r_frame_rate: Option<String>,
    avg_frame_rate: Option<String>,
    sample_rate: Option<String>,
    channels: Option<u32>,
}

/// Probes a media file using `ffprobe` and returns parsed stream/format metadata.
pub fn probe_media(ffprobe_path: &Path, input_path: &Path) -> Result<MediaInfo, String> {
    let mut cmd = Command::new(ffprobe_path);
    cmd.args([
        "-hide_banner",
        "-v",
        "quiet",
        "-print_format",
        "json",
        "-show_format",
        "-show_streams",
        "-show_error",
    ]);
    cmd.arg(input_path);
    configure_background_command(&mut cmd);

    let out = cmd
        .output()
        .map_err(|e| format!("Failed to execute ffprobe: {e}"))?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    if stdout.trim().is_empty() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!(
            "ffprobe returned no data (exit {}): {}",
            out.status,
            stderr.trim()
        ));
    }

    parse_ffprobe_json(&stdout, out.status.success())
}

/// Parses `ffprobe` JSON output into `MediaInfo`.
pub fn parse_ffprobe_json(json_str: &str, exit_ok: bool) -> Result<MediaInfo, String> {
    let parsed: FfprobeOutput =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid ffprobe JSON: {e}"))?;

    if let Some(err) = parsed.error {
        return Err(err
            .string
            .unwrap_or_else(|| "ffprobe reported media error".to_string()));
    }
    if !exit_ok {
        return Err("ffprobe exited with non-zero status".to_string());
    }

    let mut info = MediaInfo::default();

    if let Some(fmt) = parsed.format {
        info.format_name = fmt.format_name.unwrap_or_default();
        info.duration_secs = fmt.duration.and_then(|s| s.parse::<f64>().ok());
        info.file_size_bytes = fmt
            .size
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        info.bit_rate = fmt.bit_rate.and_then(|s| s.parse::<u64>().ok());
    }

    for stream in parsed.streams {
        let kind = stream.codec_type.as_deref().unwrap_or("");
        match kind {
            "video" if !info.has_video => {
                // Skip attached album art images? No, keep track unless audio-only container drops it.
                info.has_video = true;
                info.video_codec = stream.codec_name;
                info.width = stream.width;
                info.height = stream.height;
                info.pixel_format = stream.pix_fmt;
                info.fps = parse_fraction(
                    stream
                        .avg_frame_rate
                        .as_deref()
                        .filter(|r| *r != "0/0")
                        .or(stream.r_frame_rate.as_deref()),
                );
            }
            "audio" if !info.has_audio => {
                info.has_audio = true;
                info.audio_codec = stream.codec_name;
                info.audio_sample_rate = stream.sample_rate.and_then(|s| s.parse::<u32>().ok());
                info.audio_channels = stream.channels;
            }
            "subtitle" => {
                let codec = stream.codec_name.as_deref().unwrap_or("");
                if matches!(
                    codec,
                    "subrip" | "srt" | "ass" | "ssa" | "webvtt" | "mov_text" | "text"
                ) {
                    info.has_text_subtitles = true;
                }
            }
            _ => {}
        }
    }

    if !info.has_video && !info.has_audio {
        return Err("No playable video or audio streams found in file.".to_string());
    }

    Ok(info)
}

fn parse_fraction(raw: Option<&str>) -> Option<f64> {
    let s = raw?.trim();
    if let Some((num, den)) = s.split_once('/') {
        let n: f64 = num.parse().ok()?;
        let d: f64 = den.parse().ok()?;
        if d > 0.0 && n > 0.0 {
            Some(n / d)
        } else {
            None
        }
    } else {
        s.parse::<f64>().ok().filter(|v| *v > 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sample_ffprobe_json() {
        let sample = r#"{
            "format": {
                "format_name": "mov,mp4,m4a,3gp,3g2,mj2",
                "duration": "12.500000",
                "size": "1048576",
                "bit_rate": "671088"
            },
            "streams": [
                {
                    "codec_type": "video",
                    "codec_name": "h264",
                    "width": 1920,
                    "height": 1080,
                    "pix_fmt": "yuv420p",
                    "avg_frame_rate": "30000/1001"
                },
                {
                    "codec_type": "audio",
                    "codec_name": "aac",
                    "sample_rate": "48000",
                    "channels": 2
                }
            ]
        }"#;
        let info = parse_ffprobe_json(sample, true).unwrap();
        assert!(info.has_video);
        assert!(info.has_audio);
        assert_eq!(info.width, Some(1920));
        assert_eq!(info.height, Some(1080));
        assert!((info.fps.unwrap() - 29.97).abs() < 0.02);
        assert_eq!(info.audio_sample_rate, Some(48000));
    }
}
