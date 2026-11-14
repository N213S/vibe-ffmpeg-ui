use crate::models::config::{
    AudioCodec, ContainerFormat, ConvertMode, ConversionSettings, PixelFormat, ResolutionOption,
    VideoCodec, VideoRateControl,
};
use crate::models::media::MediaInfo;
use std::ffi::OsString;
use std::path::Path;

/// Fully constructed, injection-safe FFmpeg invocation arguments.
#[derive(Debug, Clone)]
pub struct CommandPlan {
    pub args: Vec<OsString>,
    pub notes: Vec<String>,
}

impl CommandPlan {
    /// Human-readable command preview string (for UI display only; never executed via a shell).
    pub fn display_preview(&self) -> String {
        let mut out = String::from("ffmpeg");
        for arg in &self.args {
            let s = arg.to_string_lossy();
            out.push(' ');
            if s.contains(' ') || s.contains('&') || s.contains('(') || s.contains(',') || s.is_empty() {
                out.push('"');
                out.push_str(&s);
                out.push('"');
            } else {
                out.push_str(&s);
            }
        }
        out
    }
}

/// Builds a complete, corruption-safe FFmpeg argument list for `input_path -> temp_output_path`.
pub fn build_command_plan(
    settings: &ConversionSettings,
    media: Option<&MediaInfo>,
    input_path: &Path,
    temp_output_path: &Path,
) -> Result<CommandPlan, String> {
    let mut normalized = settings.clone();
    normalized.enforce_compatibility();

    let has_video = media.map(|m| m.has_video).unwrap_or(true);
    let has_audio = media.map(|m| m.has_audio).unwrap_or(true);
    let has_text_subs = media.map(|m| m.has_text_subtitles).unwrap_or(false);

    if normalized.mode == ConvertMode::ImageGif && !has_video {
        return Err("Cannot create an image or GIF from an audio-only file.".to_string());
    }
    if normalized.mode == ConvertMode::AudioOnly && !has_audio {
        return Err("Input file has no audio stream for Audio Only extraction.".to_string());
    }

    let mut args: Vec<OsString> = Vec::with_capacity(48);
    let mut notes = Vec::new();
    let push = |args: &mut Vec<OsString>, s: &str| args.push(OsString::from(s));

    push(&mut args, "-hide_banner");
    push(&mut args, "-nostdin");
    push(&mut args, "-y");

    if normalized.safety.repair_timestamps && normalized.mode != ConvertMode::ImageGif {
        push(&mut args, "-fflags");
        push(&mut args, "+genpts");
    }

    push(&mut args, "-i");
    args.push(input_path.as_os_str().to_os_string());

    match normalized.mode {
        ConvertMode::Video => build_video_mode_args(
            &mut args,
            &mut notes,
            &normalized,
            media,
            has_video,
            has_audio,
            has_text_subs,
        ),
        ConvertMode::AudioOnly => build_audio_only_args(&mut args, &normalized),
        ConvertMode::ImageGif => build_image_gif_args(&mut args, &normalized, media),
        ConvertMode::Remux => build_remux_args(
            &mut args,
            &mut notes,
            &normalized,
            media,
            has_video,
            has_audio,
            has_text_subs,
        ),
    }

    if normalized.safety.preserve_metadata && !matches!(normalized.container, ContainerFormat::Gif) {
        push(&mut args, "-map_metadata");
        push(&mut args, "0");
    } else if !normalized.safety.preserve_metadata {
        push(&mut args, "-map_metadata");
        push(&mut args, "-1");
    }

    if normalized.safety.repair_timestamps
        && matches!(normalized.mode, ConvertMode::Video | ConvertMode::AudioOnly | ConvertMode::Remux)
    {
        push(&mut args, "-avoid_negative_ts");
        push(&mut args, "make_zero");
        push(&mut args, "-max_muxing_queue_size");
        push(&mut args, "4096");
    }

    if normalized.container.supports_faststart() && normalized.safety.faststart {
        push(&mut args, "-movflags");
        push(&mut args, "+faststart");
    }

    push(&mut args, "-f");
    push(&mut args, normalized.container.muxer_name());

    push(&mut args, "-progress");
    push(&mut args, "pipe:1");
    push(&mut args, "-stats_period");
    push(&mut args, "0.25");

    args.push(temp_output_path.as_os_str().to_os_string());

    Ok(CommandPlan { args, notes })
}

fn build_audio_only_args(args: &mut Vec<OsString>, settings: &ConversionSettings) {
    args.push("-map".into());
    args.push("0:a:0".into());
    args.push("-vn".into());
    args.push("-sn".into());
    push_audio_codec_args(args, settings);
}

fn build_image_gif_args(
    args: &mut Vec<OsString>,
    settings: &ConversionSettings,
    media: Option<&MediaInfo>,
) {
    let ig = &settings.image_gif;
    let max_w = ((ig.max_width.clamp(64, 3840)) / 2) * 2;
    let fps = ig.fps.clamp(1, 30);

    args.push("-map".into());
    args.push("0:v:0".into());
    args.push("-an".into());
    args.push("-sn".into());

    match settings.container {
        ContainerFormat::Gif => {
            let scale_expr = if ig.limit_max_width {
                format!("scale='min(iw\\,{max_w})':-2:flags=lanczos")
            } else {
                "scale=trunc(iw/2)*2:trunc(ih/2)*2:flags=lanczos".to_string()
            };
            let vf = format!(
                "fps={fps},{scale_expr},split[s0][s1];[s0]palettegen=stats_mode=diff[p];[s1][p]paletteuse=dither=bayer:bayer_scale=5"
            );
            args.push("-vf".into());
            args.push(vf.into());
            args.push("-loop".into());
            args.push("0".into());
        }
        ContainerFormat::Webp => {
            let mut filters = Vec::new();
            if ig.webp_animated {
                filters.push(format!("fps={fps}"));
            }
            if ig.limit_max_width {
                filters.push(format!("scale='min(iw\\,{max_w})':-2:flags=lanczos"));
            }
            if !filters.is_empty() {
                args.push("-vf".into());
                args.push(filters.join(",").into());
            }
            if ig.webp_animated {
                args.push("-c:v".into());
                args.push("libwebp_anim".into());
                args.push("-loop".into());
                args.push("0".into());
            } else {
                args.push("-frames:v".into());
                args.push("1".into());
                args.push("-c:v".into());
                args.push("libwebp".into());
            }
            args.push("-quality".into());
            args.push(ig.quality.clamp(1, 100).to_string().into());
        }
        ContainerFormat::Png => {
            if ig.limit_max_width {
                args.push("-vf".into());
                args.push(format!("scale='min(iw\\,{max_w})':-1:flags=lanczos").into());
            }
            args.push("-frames:v".into());
            args.push("1".into());
            args.push("-update".into());
            args.push("1".into());
            args.push("-c:v".into());
            args.push("png".into());
            args.push("-compression_level".into());
            args.push(ig.png_compression.clamp(0, 9).to_string().into());
        }
        ContainerFormat::Jpg => {
            if ig.limit_max_width {
                args.push("-vf".into());
                args.push(format!("scale='min(iw\\,{max_w})':-1:flags=lanczos").into());
            }
            // Map 1..=100 quality to mjpeg -q:v 31..=2 (lower q:v = higher visual quality).
            let q = ig.quality.clamp(1, 100);
            let qv = 31 - ((q * 29) / 100);
            args.push("-frames:v".into());
            args.push("1".into());
            args.push("-update".into());
            args.push("1".into());
            args.push("-c:v".into());
            args.push("mjpeg".into());
            args.push("-q:v".into());
            args.push(qv.clamp(2, 31).to_string().into());
        }
        ContainerFormat::Avif => {
            let needs_even = match media.and_then(|m| m.width.zip(m.height)) {
                Some((w, h)) => w % 2 != 0 || h % 2 != 0,
                None => true,
            };
            if ig.limit_max_width {
                args.push("-vf".into());
                args.push(format!("scale='min(iw\\,{max_w})':-2:flags=lanczos").into());
            } else if needs_even {
                args.push("-vf".into());
                args.push("scale=trunc(iw/2)*2:trunc(ih/2)*2:flags=lanczos".into());
            }
            args.push("-frames:v".into());
            args.push("1".into());
            args.push("-c:v".into());
            args.push("libaom-av1".into());
            args.push("-still-picture".into());
            args.push("1".into());
            args.push("-crf".into());
            args.push(ig.avif_crf.clamp(10, 55).to_string().into());
            args.push("-cpu-used".into());
            args.push("6".into());
            args.push("-pix_fmt".into());
            args.push("yuv420p".into());
        }
        _ => {}
    }
}

fn build_remux_args(
    args: &mut Vec<OsString>,
    notes: &mut Vec<String>,
    settings: &ConversionSettings,
    media: Option<&MediaInfo>,
    has_video: bool,
    has_audio: bool,
    has_text_subs: bool,
) {
    if has_video {
        args.push("-map".into());
        args.push("0:v:0".into());
    }
    if has_audio {
        args.push("-map".into());
        args.push("0:a:0?".into());
    }
    let allow_subtitles = settings.safety.keep_subtitles
        && has_text_subs
        && matches!(
            settings.container,
            ContainerFormat::Mp4 | ContainerFormat::Mov | ContainerFormat::Mkv
        );
    if allow_subtitles {
        args.push("-map".into());
        args.push("0:s?".into());
    } else {
        args.push("-sn".into());
    }

    args.push("-c".into());
    args.push("copy".into());

    if allow_subtitles && matches!(settings.container, ContainerFormat::Mp4 | ContainerFormat::Mov) {
        args.push("-c:s".into());
        args.push("mov_text".into());
        notes.push("Text subtitles converted to mov_text for MP4/MOV compatibility.".to_string());
    }

    if matches!(settings.container, ContainerFormat::Mp4 | ContainerFormat::Mov)
        && media
            .and_then(|m| m.video_codec.as_deref())
            .is_some_and(|c| c == "hevc" || c == "h265")
    {
        args.push("-tag:v".into());
        args.push("hvc1".into());
    }
}

fn build_video_mode_args(
    args: &mut Vec<OsString>,
    notes: &mut Vec<String>,
    settings: &ConversionSettings,
    media: Option<&MediaInfo>,
    has_video: bool,
    has_audio: bool,
    has_text_subs: bool,
) {
    if has_video {
        args.push("-map".into());
        args.push("0:v:0".into());
    }
    if has_audio && settings.audio_codec != AudioCodec::None {
        args.push("-map".into());
        args.push("0:a:0?".into());
    }

    let allow_subtitles = settings.safety.keep_subtitles
        && has_text_subs
        && matches!(
            settings.container,
            ContainerFormat::Mp4 | ContainerFormat::Mov | ContainerFormat::Mkv
        );
    if allow_subtitles {
        args.push("-map".into());
        args.push("0:s?".into());
    } else {
        args.push("-sn".into());
    }

    if has_video {
        push_video_codec_args(args, notes, settings, media);
    } else {
        args.push("-vn".into());
    }

    if has_audio && settings.audio_codec != AudioCodec::None {
        push_audio_codec_args(args, settings);
    } else {
        args.push("-an".into());
    }

    if allow_subtitles {
        args.push("-c:s".into());
        match settings.container {
            ContainerFormat::Mp4 | ContainerFormat::Mov => args.push("mov_text".into()),
            _ => args.push("copy".into()),
        }
    }
}

fn push_video_codec_args(
    args: &mut Vec<OsString>,
    notes: &mut Vec<String>,
    settings: &ConversionSettings,
    media: Option<&MediaInfo>,
) {
    if settings.video_codec == VideoCodec::Copy {
        args.push("-c:v".into());
        args.push("copy".into());
        if settings.resolution != ResolutionOption::Original {
            notes.push("Resolution scaling skipped during Stream Copy.".to_string());
        }
        if matches!(settings.container, ContainerFormat::Mp4 | ContainerFormat::Mov)
            && media
                .and_then(|m| m.video_codec.as_deref())
                .is_some_and(|c| c == "hevc" || c == "h265")
        {
            args.push("-tag:v".into());
            args.push("hvc1".into());
        }
        return;
    }

    args.push("-c:v".into());
    args.push(settings.video_codec.ffmpeg_name().into());

    if let Some(vf) = build_video_filter(settings, media) {
        args.push("-vf".into());
        args.push(vf.into());
    }

    let effective_pix_fmt = if matches!(
        settings.video_codec,
        VideoCodec::H264Nvenc | VideoCodec::H264Amf | VideoCodec::H264Qsv
    ) && settings.pixel_format == PixelFormat::Yuv420p10le
    {
        notes.push("Hardware H.264 uses 8-bit yuv420p for compatibility.".to_string());
        PixelFormat::Yuv420p
    } else {
        settings.pixel_format
    };

    if let Some(pix) = effective_pix_fmt.ffmpeg_value() {
        args.push("-pix_fmt".into());
        args.push(pix.into());
    }

    if let Some(fps_str) = settings.fps.ffmpeg_value() {
        args.push("-r".into());
        args.push(fps_str.into());
    }

    push_encoder_tuning_args(args, settings, effective_pix_fmt);
}

fn build_video_filter(settings: &ConversionSettings, media: Option<&MediaInfo>) -> Option<String> {
    let mut filters = Vec::new();
    match settings.resolution {
        ResolutionOption::Original => {
            if settings.safety.even_dimensions_guard
                && settings.pixel_format != PixelFormat::KeepOriginal
            {
                let needs_even = match media.and_then(|m| m.width.zip(m.height)) {
                    Some((w, h)) => w % 2 != 0 || h % 2 != 0,
                    None => true,
                };
                if needs_even {
                    filters.push("scale=trunc(iw/2)*2:trunc(ih/2)*2".to_string());
                }
            }
        }
        ResolutionOption::Uhd2160p => filters.push("scale=-2:2160:flags=lanczos".to_string()),
        ResolutionOption::Fhd1080p => filters.push("scale=-2:1080:flags=lanczos".to_string()),
        ResolutionOption::Hd720p => filters.push("scale=-2:720:flags=lanczos".to_string()),
        ResolutionOption::Sd480p => filters.push("scale=-2:480:flags=lanczos".to_string()),
        ResolutionOption::Custom { width, height } => {
            let w = width.max(16);
            let h = height.max(16);
            let (ew, eh) = if settings.safety.even_dimensions_guard {
                ((w / 2) * 2, (h / 2) * 2)
            } else {
                (w, h)
            };
            filters.push(format!("scale={ew}:{eh}:flags=lanczos"));
        }
    }

    if filters.is_empty() {
        None
    } else {
        Some(filters.join(","))
    }
}

fn push_encoder_tuning_args(
    args: &mut Vec<OsString>,
    settings: &ConversionSettings,
    pix_fmt: PixelFormat,
) {
    let push_rate = |args: &mut Vec<OsString>, crf_flag: &str| match settings.rate_control {
        VideoRateControl::Quality(q) => {
            args.push(crf_flag.into());
            args.push(q.to_string().into());
        }
        VideoRateControl::BitrateKbps(kbps) => {
            args.push("-b:v".into());
            args.push(format!("{kbps}k").into());
        }
    };

    match settings.video_codec {
        VideoCodec::Libx264 => {
            args.push("-preset".into());
            args.push(settings.speed.x264_preset().into());
            push_rate(args, "-crf");
            if pix_fmt == PixelFormat::Yuv420p {
                args.push("-profile:v".into());
                args.push("high".into());
                args.push("-level:v".into());
                args.push("4.1".into());
            }
        }
        VideoCodec::Libx265 => {
            args.push("-preset".into());
            args.push(settings.speed.x264_preset().into());
            push_rate(args, "-crf");
        }
        VideoCodec::LibSvtAv1 => {
            args.push("-preset".into());
            args.push(settings.speed.svt_av1_preset().into());
            push_rate(args, "-crf");
            args.push("-svtav1-params".into());
            args.push("tune=0".into());
        }
        VideoCodec::LibVpxVp9 => {
            args.push("-deadline".into());
            args.push("good".into());
            args.push("-cpu-used".into());
            args.push(settings.speed.vp9_cpu_used().into());
            args.push("-row-mt".into());
            args.push("1".into());
            match settings.rate_control {
                VideoRateControl::Quality(q) => {
                    args.push("-crf".into());
                    args.push(q.to_string().into());
                    args.push("-b:v".into());
                    args.push("0".into());
                }
                VideoRateControl::BitrateKbps(kbps) => {
                    args.push("-b:v".into());
                    args.push(format!("{kbps}k").into());
                }
            }
        }
        VideoCodec::H264Nvenc | VideoCodec::HevcNvenc | VideoCodec::Av1Nvenc => {
            args.push("-preset".into());
            args.push(settings.speed.nvenc_preset().into());
            match settings.rate_control {
                VideoRateControl::Quality(q) => {
                    args.push("-rc".into());
                    args.push("vbr".into());
                    args.push("-cq".into());
                    args.push(q.to_string().into());
                    args.push("-b:v".into());
                    args.push("0".into());
                }
                VideoRateControl::BitrateKbps(kbps) => {
                    args.push("-rc".into());
                    args.push("vbr".into());
                    args.push("-b:v".into());
                    args.push(format!("{kbps}k").into());
                }
            }
        }
        VideoCodec::H264Amf | VideoCodec::HevcAmf => {
            args.push("-quality".into());
            args.push(settings.speed.amf_quality().into());
            match settings.rate_control {
                VideoRateControl::Quality(q) => {
                    args.push("-rc".into());
                    args.push("cqp".into());
                    args.push("-qp_i".into());
                    args.push(q.to_string().into());
                    args.push("-qp_p".into());
                    args.push(q.to_string().into());
                }
                VideoRateControl::BitrateKbps(kbps) => {
                    args.push("-rc".into());
                    args.push("vbr_latency".into());
                    args.push("-b:v".into());
                    args.push(format!("{kbps}k").into());
                }
            }
        }
        VideoCodec::H264Qsv | VideoCodec::HevcQsv => {
            args.push("-preset".into());
            args.push(settings.speed.x264_preset().into());
            push_rate(args, "-global_quality");
        }
        VideoCodec::Copy => {}
    }

    if settings.video_codec.is_hevc()
        && matches!(settings.container, ContainerFormat::Mp4 | ContainerFormat::Mov)
    {
        args.push("-tag:v".into());
        args.push("hvc1".into());
    }
}

fn push_audio_codec_args(args: &mut Vec<OsString>, settings: &ConversionSettings) {
    if settings.audio_codec == AudioCodec::Copy {
        args.push("-c:a".into());
        args.push("copy".into());
        return;
    }

    args.push("-c:a".into());
    args.push(settings.audio_codec.ffmpeg_name().into());

    if !settings.audio_codec.is_lossless() {
        let kbps = settings.audio_bitrate_kbps.clamp(32, 512);
        args.push("-b:a".into());
        args.push(format!("{kbps}k").into());
    }

    if let Some(sr) = settings.audio_sample_rate.ffmpeg_value() {
        args.push("-ar".into());
        args.push(sr.into());
    }

    if let Some(ch) = settings.audio_channels.ffmpeg_value() {
        args.push("-ac".into());
        args.push(ch.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::PresetProfile;

    fn arg_list(plan: &CommandPlan) -> Vec<String> {
        plan.args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn universal_mp4_includes_required_integrity_flags() {
        let settings = ConversionSettings::from_preset(PresetProfile::Mp4UniversalH264);
        let media = MediaInfo {
            has_video: true,
            has_audio: true,
            width: Some(1921),
            height: Some(1081),
            ..Default::default()
        };
        let plan = build_command_plan(
            &settings,
            Some(&media),
            Path::new("input & test.mkv"),
            Path::new("out.tmp.mp4"),
        )
        .unwrap();
        let args = arg_list(&plan);

        assert!(args.windows(2).any(|w| w == ["-pix_fmt", "yuv420p"]));
        assert!(args.windows(2).any(|w| w == ["-movflags", "+faststart"]));
        assert!(args.windows(2).any(|w| w == ["-fflags", "+genpts"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2"]));
        assert!(args.windows(2).any(|w| w == ["-profile:v", "high"]));
        assert!(args.windows(2).any(|w| w == ["-c:a", "aac"]));
        assert!(args.windows(2).any(|w| w == ["-f", "mp4"]));
    }

    #[test]
    fn audio_only_mode_passes_vn_and_explicit_muxer() {
        let settings = ConversionSettings::from_preset(PresetProfile::ExtractMp3320);
        let media = MediaInfo {
            has_video: true,
            has_audio: true,
            ..Default::default()
        };
        let plan = build_command_plan(
            &settings,
            Some(&media),
            Path::new("clip.mp4"),
            Path::new("clip.tmp.mp3"),
        )
        .unwrap();
        let args = arg_list(&plan);
        assert!(args.contains(&"-vn".to_string()));
        assert!(args.windows(2).any(|w| w == ["-c:a", "libmp3lame"]));
        assert!(args.windows(2).any(|w| w == ["-b:a", "320k"]));
        assert!(args.windows(2).any(|w| w == ["-f", "mp3"]));
    }

    #[test]
    fn gif_mode_uses_palettegen_and_paletteuse() {
        let settings = ConversionSettings::from_preset(PresetProfile::SmoothGif15fps);
        let plan = build_command_plan(
            &settings,
            None,
            Path::new("clip.mp4"),
            Path::new("clip.tmp.gif"),
        )
        .unwrap();
        let args = arg_list(&plan);
        let vf_idx = args.iter().position(|a| a == "-vf").unwrap();
        let vf = &args[vf_idx + 1];
        assert!(vf.contains("palettegen"));
        assert!(vf.contains("paletteuse"));
        assert!(args.windows(2).any(|w| w == ["-f", "gif"]));
    }

    #[test]
    fn image_static_png_and_avif_use_expected_flags() {
        let png_settings = ConversionSettings::from_preset(PresetProfile::StaticPngLossless);
        let png_plan = build_command_plan(
            &png_settings,
            None,
            Path::new("clip.mp4"),
            Path::new("clip.tmp.png"),
        )
        .unwrap();
        let png_args = arg_list(&png_plan);
        assert!(png_args.windows(2).any(|w| w == ["-update", "1"]));
        assert!(png_args.windows(2).any(|w| w == ["-f", "image2"]));

        let avif_settings = ConversionSettings::from_preset(PresetProfile::StaticAvifCompact);
        let avif_plan = build_command_plan(
            &avif_settings,
            None,
            Path::new("photo.png"),
            Path::new("photo.tmp.avif"),
        )
        .unwrap();
        let avif_args = arg_list(&avif_plan);
        assert!(avif_args.windows(2).any(|w| w == ["-still-picture", "1"]));
        assert!(avif_args.windows(2).any(|w| w == ["-f", "avif"]));
    }

    #[test]
    fn remux_mode_uses_c_copy_without_reencoding() {
        let settings = ConversionSettings::from_preset(PresetProfile::InstantRemuxMp4);
        let plan = build_command_plan(
            &settings,
            None,
            Path::new("movie.mkv"),
            Path::new("movie.tmp.mp4"),
        )
        .unwrap();
        let args = arg_list(&plan);
        assert!(args.windows(2).any(|w| w == ["-c", "copy"]));
        assert!(!args.contains(&"-crf".to_string()));
        assert!(!args.contains(&"-vf".to_string()));
    }

    #[test]
    fn hevc_mp4_includes_hvc1_tag() {
        let settings = ConversionSettings::from_preset(PresetProfile::Mp4HevcCompact);
        let plan = build_command_plan(
            &settings,
            None,
            Path::new("in.mp4"),
            Path::new("in.tmp.mp4"),
        )
        .unwrap();
        let args = arg_list(&plan);
        assert!(args.windows(2).any(|w| w == ["-c:v", "libx265"]));
        assert!(args.windows(2).any(|w| w == ["-tag:v", "hvc1"]));
    }

    #[test]
    fn webm_vp9_crf_includes_bv_zero_and_row_mt() {
        let settings = ConversionSettings::from_preset(PresetProfile::WebmStreamingVp9);
        let plan = build_command_plan(
            &settings,
            None,
            Path::new("in.mp4"),
            Path::new("in.tmp.webm"),
        )
        .unwrap();
        let args = arg_list(&plan);
        assert!(args.windows(2).any(|w| w == ["-c:v", "libvpx-vp9"]));
        assert!(args.windows(2).any(|w| w == ["-b:v", "0"]));
        assert!(args.windows(2).any(|w| w == ["-row-mt", "1"]));
        assert!(args.windows(2).any(|w| w == ["-c:a", "libopus"]));
    }

    #[test]
    fn rejects_gif_from_audio_only_input() {
        let settings = ConversionSettings::from_preset(PresetProfile::SmoothGif15fps);
        let media = MediaInfo {
            has_video: false,
            has_audio: true,
            ..Default::default()
        };
        let err = build_command_plan(
            &settings,
            Some(&media),
            Path::new("song.mp3"),
            Path::new("song.tmp.gif"),
        )
        .unwrap_err();
        assert!(err.contains("audio-only"));
    }
}
