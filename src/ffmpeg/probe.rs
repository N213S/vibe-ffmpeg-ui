use crate::ffmpeg::discovery::configure_background_command;
use crate::models::media::{MediaInfo, StreamDetail};
use serde::Deserialize;
use std::collections::BTreeMap;
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
    format_long_name: Option<String>,
    start_time: Option<String>,
    duration: Option<String>,
    size: Option<String>,
    bit_rate: Option<String>,
    probe_score: Option<i32>,
    #[serde(default)]
    tags: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    index: Option<usize>,
    codec_type: Option<String>,
    codec_name: Option<String>,
    codec_long_name: Option<String>,
    profile: Option<String>,
    codec_tag_string: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    pix_fmt: Option<String>,
    color_space: Option<String>,
    color_range: Option<String>,
    color_transfer: Option<String>,
    color_primaries: Option<String>,
    r_frame_rate: Option<String>,
    avg_frame_rate: Option<String>,
    nb_frames: Option<String>,
    duration: Option<String>,
    bit_rate: Option<String>,
    disposition: Option<FfprobeDisposition>,
    sample_rate: Option<String>,
    channels: Option<u32>,
    channel_layout: Option<String>,
    #[serde(default)]
    tags: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize, Default)]
struct FfprobeDisposition {
    #[serde(default)]
    default: i32,
    #[serde(default)]
    attached_pic: i32,
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

    let mut info = MediaInfo {
        raw_json: json_str.trim().to_string(),
        ..Default::default()
    };

    if let Some(fmt) = parsed.format {
        info.format_name = fmt.format_name.unwrap_or_default();
        info.format_long_name = fmt.format_long_name;
        info.start_time_secs = fmt.start_time.and_then(|s| s.parse::<f64>().ok());
        info.duration_secs = fmt.duration.and_then(|s| s.parse::<f64>().ok());
        info.file_size_bytes = fmt
            .size
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        info.bit_rate = fmt.bit_rate.and_then(|s| s.parse::<u64>().ok());
        info.probe_score = fmt.probe_score;
        info.format_tags = fmt.tags.into_iter().collect();
    }

    let fmt_lower = info.format_name.to_ascii_lowercase();
    let is_image_demuxer = fmt_lower.contains("image2")
        || fmt_lower.contains("_pipe")
        || fmt_lower == "ico"
        || fmt_lower.contains(",ico");

    for (idx, stream) in parsed.streams.into_iter().enumerate() {
        let kind = stream.codec_type.as_deref().unwrap_or("");
        let stream_fps = parse_fraction(
            stream
                .avg_frame_rate
                .as_deref()
                .filter(|r| *r != "0/0")
                .or(stream.r_frame_rate.as_deref()),
        );
        let is_default = stream.disposition.as_ref().is_some_and(|d| d.default == 1);
        let attached_pic = stream
            .disposition
            .as_ref()
            .is_some_and(|d| d.attached_pic == 1);

        info.streams.push(StreamDetail {
            index: stream.index.unwrap_or(idx),
            codec_type: kind.to_string(),
            codec_name: stream
                .codec_name
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            codec_long_name: stream.codec_long_name.clone(),
            profile: stream.profile.clone(),
            codec_tag_string: stream.codec_tag_string.clone(),
            width: stream.width,
            height: stream.height,
            pixel_format: stream.pix_fmt.clone(),
            color_space: stream.color_space.clone(),
            color_range: stream.color_range.clone(),
            color_transfer: stream.color_transfer.clone(),
            color_primaries: stream.color_primaries.clone(),
            fps: stream_fps,
            avg_frame_rate: stream.avg_frame_rate.clone(),
            r_frame_rate: stream.r_frame_rate.clone(),
            nb_frames: stream.nb_frames.as_deref().and_then(|s| s.parse::<u64>().ok()),
            duration_secs: stream.duration.as_deref().and_then(|s| s.parse::<f64>().ok()),
            bit_rate: stream.bit_rate.as_deref().and_then(|s| s.parse::<u64>().ok()),
            sample_rate: stream.sample_rate.as_deref().and_then(|s| s.parse::<u32>().ok()),
            channels: stream.channels,
            channel_layout: stream.channel_layout.clone(),
            language: stream.tags.get("language").cloned(),
            title: stream.tags.get("title").cloned(),
            is_default,
            is_attached_pic: attached_pic,
            tags: stream.tags.into_iter().collect(),
        });

        match kind {
            "video" if !info.has_video => {
                info.has_video = true;
                let codec = stream.codec_name.clone().unwrap_or_default();
                let is_animated_img_codec = matches!(codec.as_str(), "gif" | "webp" | "apng");
                let is_static_img_codec = matches!(
                    codec.as_str(),
                    "png" | "mjpeg" | "bmp" | "tiff" | "targa" | "ico" | "jpegxl" | "exr" | "hdr"
                );
                let single_frame = stream.nb_frames.as_deref() == Some("1");
                let no_motion_dur = info.duration_secs.unwrap_or(0.0) <= 0.0;

                if attached_pic
                    || (!is_animated_img_codec
                        && (is_image_demuxer || (is_static_img_codec && (no_motion_dur || single_frame))))
                {
                    info.is_still_image = true;
                }

                info.video_codec = stream.codec_name;
                info.width = stream.width;
                info.height = stream.height;
                info.pixel_format = stream.pix_fmt;
                info.fps = stream_fps;
            }
            "audio" if !info.has_audio => {
                info.has_audio = true;
                info.audio_codec = stream.codec_name;
                info.audio_sample_rate = stream.sample_rate.and_then(|s| s.parse::<u32>().ok());
                info.audio_channels = stream.channels;
            }
            "subtitle" => {
                info.has_subtitle = true;
                let codec = stream.codec_name.as_deref().unwrap_or("");
                if info.subtitle_codec.is_none() {
                    info.subtitle_codec = stream.codec_name.clone();
                }
                if matches!(
                    codec,
                    "subrip" | "srt" | "ass" | "ssa" | "webvtt" | "mov_text" | "text" | "lrc"
                ) {
                    info.has_text_subtitles = true;
                }
            }
            _ => {}
        }
    }

    if ["srt", "ass", "webvtt", "lrc", "subviewer", "microdvd"]
        .iter()
        .any(|tag| fmt_lower.contains(tag))
    {
        info.has_subtitle = true;
        info.has_text_subtitles = true;
    }

    if !info.has_video && !info.has_audio && !info.has_subtitle {
        return Err("No playable video, audio, or subtitle streams found in file.".to_string());
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
        assert_eq!(info.streams.len(), 2);
        assert_eq!(info.streams[0].codec_type, "video");
        assert_eq!(info.streams[1].codec_type, "audio");
        let report = info.formatted_report(Path::new("sample.mp4"));
        assert!(report.contains("mov,mp4,m4a,3gp,3g2,mj2"));
        assert!(report.contains("1920x1080"));
    }
}
