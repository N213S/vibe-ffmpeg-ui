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
            if s.contains(' ')
                || s.contains('&')
                || s.contains('(')
                || s.contains(',')
                || s.contains('[')
                || s.contains(';')
                || s.is_empty()
            {
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
    let has_subtitle = media.map(|m| m.has_subtitle).unwrap_or(true);
    let has_text_subs = media.map(|m| m.has_text_subtitles).unwrap_or(false);
    let is_still_image = media.map(|m| m.is_still_image).unwrap_or(false);

    match normalized.mode {
        ConvertMode::AudioOnly if !has_audio => {
            return Err("Input file has no audio stream for Audio Only extraction.".to_string());
        }
        ConvertMode::Subtitle if !has_subtitle => {
            return Err("Input file has no subtitle stream for Subtitle extraction.".to_string());
        }
        ConvertMode::ImageGif if !has_video && !is_still_image && !has_audio => {
            return Err("Input file has no video, image, or audio stream.".to_string());
        }
        ConvertMode::Video if !has_video && !is_still_image && !has_audio => {
            return Err("Input file has no video, image, or audio stream.".to_string());
        }
        _ => {}
    }

    let mut args: Vec<OsString> = Vec::with_capacity(48);
    let mut notes = Vec::new();
    let push = |args: &mut Vec<OsString>, s: &str| args.push(OsString::from(s));

    push(&mut args, "-hide_banner");
    push(&mut args, "-nostdin");
    push(&mut args, "-y");

    let loop_still_image =
        normalized.mode == ConvertMode::Video && media.is_some_and(|m| m.is_still_image && !m.has_audio);

    if normalized.safety.repair_timestamps
        && !matches!(normalized.mode, ConvertMode::ImageGif | ConvertMode::Subtitle)
        && !loop_still_image
    {
        push(&mut args, "-fflags");
        push(&mut args, "+genpts");
    }

    if loop_still_image {
        push(&mut args, "-loop");
        push(&mut args, "1");
    }

    push(&mut args, "-i");
    args.push(input_path.as_os_str().to_os_string());

    if loop_still_image {
        let secs = normalized.image_to_video_duration_secs.clamp(1, 600);
        push(&mut args, "-t");
        args.push(secs.to_string().into());
    }

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
        ConvertMode::ImageGif => build_image_gif_args(&mut args, &normalized, media, has_video, has_audio),
        ConvertMode::Subtitle => build_subtitle_args(&mut args, &normalized),
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

    if !matches!(normalized.mode, ConvertMode::Subtitle) {
        if normalized.safety.preserve_metadata && !matches!(normalized.container, ContainerFormat::Gif) {
            push(&mut args, "-map_metadata");
            push(&mut args, "0");
        } else if !normalized.safety.preserve_metadata {
            push(&mut args, "-map_metadata");
            push(&mut args, "-1");
        }
    }

    if normalized.safety.repair_timestamps
        && matches!(
            normalized.mode,
            ConvertMode::Video | ConvertMode::AudioOnly | ConvertMode::Remux
        )
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

fn build_subtitle_args(args: &mut Vec<OsString>, settings: &ConversionSettings) {
    args.push("-map".into());
    args.push("0:s:0".into());
    args.push("-vn".into());
    args.push("-an".into());
    args.push("-c:s".into());
    args.push(settings.container.subtitle_codec_name().into());
}

fn build_image_gif_args(
    args: &mut Vec<OsString>,
    settings: &ConversionSettings,
    media: Option<&MediaInfo>,
    has_video: bool,
    has_audio: bool,
) {
    let ig = &settings.image_gif;
    let max_w = ((ig.max_width.clamp(64, 3840)) / 2) * 2;
    let fps = ig.fps.clamp(1, 30);

    // Audio-only source without cover art: render waveform image via showwavespic.
    if !has_video && has_audio {
        let wave_size = if settings.container == ContainerFormat::Ico {
            "256x256"
        } else {
            "1280x400"
        };
        args.push("-filter_complex".into());
        args.push(format!("[0:a]showwavespic=s={wave_size}:colors=white[v]").into());
        args.push("-map".into());
        args.push("[v]".into());
        args.push("-an".into());
        args.push("-sn".into());
        push_still_image_encoder_args(args, settings, true);
        return;
    }

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
        ContainerFormat::Apng => {
            let mut filters = vec![format!("fps={fps}")];
            if ig.limit_max_width {
                filters.push(format!("scale='min(iw\\,{max_w})':-2:flags=lanczos"));
            }
            args.push("-vf".into());
            args.push(filters.join(",").into());
            args.push("-c:v".into());
            args.push("apng".into());
            args.push("-plays".into());
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
        ContainerFormat::Ico => {
            let cap = if ig.limit_max_width { max_w.min(256) } else { 256 };
            args.push("-vf".into());
            args.push(
                format!("scale='min({cap},iw)':'min({cap},ih)':force_original_aspect_ratio=decrease")
                    .into(),
            );
            args.push("-frames:v".into());
            args.push("1".into());
            args.push("-c:v".into());
            args.push("png".into());
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
            push_still_image_encoder_args(args, settings, false);
        }
        _ => {
            if ig.limit_max_width {
                args.push("-vf".into());
                args.push(format!("scale='min(iw\\,{max_w})':-1:flags=lanczos").into());
            }
            push_still_image_encoder_args(args, settings, false);
        }
    }
}

fn push_still_image_encoder_args(
    args: &mut Vec<OsString>,
    settings: &ConversionSettings,
    waveform_only: bool,
) {
    let ig = &settings.image_gif;
    match settings.container {
        ContainerFormat::Png => {
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
        ContainerFormat::Webp => {
            args.push("-frames:v".into());
            args.push("1".into());
            args.push("-c:v".into());
            args.push(if waveform_only { "libwebp" } else if ig.webp_animated { "libwebp_anim" } else { "libwebp" }.into());
            args.push("-quality".into());
            args.push(ig.quality.clamp(1, 100).to_string().into());
        }
        ContainerFormat::Gif => {
            args.push("-frames:v".into());
            args.push("1".into());
        }
        ContainerFormat::Apng => {
            args.push("-frames:v".into());
            args.push("1".into());
            args.push("-c:v".into());
            args.push("apng".into());
        }
        ContainerFormat::Ico => {
            args.push("-frames:v".into());
            args.push("1".into());
            args.push("-c:v".into());
            args.push("png".into());
        }
        ContainerFormat::Avif => {
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
        ContainerFormat::Bmp
        | ContainerFormat::Tiff
        | ContainerFormat::Tga
        | ContainerFormat::Jxl
        | ContainerFormat::Exr
        | ContainerFormat::Hdr => {
            let codec = match settings.container {
                ContainerFormat::Bmp => "bmp",
                ContainerFormat::Tiff => "tiff",
                ContainerFormat::Tga => "targa",
                ContainerFormat::Jxl => "libjxl",
                ContainerFormat::Exr => "exr",
                ContainerFormat::Hdr => "hdr",
                _ => "png",
            };
            args.push("-frames:v".into());
            args.push("1".into());
            args.push("-update".into());
            args.push("1".into());
            args.push("-c:v".into());
            args.push(codec.into());
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
            ContainerFormat::Mp4 | ContainerFormat::Mov | ContainerFormat::Mkv | ContainerFormat::M4v
        );
    if allow_subtitles {
        args.push("-map".into());
        args.push("0:s?".into());
    } else {
        args.push("-sn".into());
    }

    args.push("-c".into());
    args.push("copy".into());

    if allow_subtitles && matches!(settings.container, ContainerFormat::Mp4 | ContainerFormat::Mov | ContainerFormat::M4v) {
        args.push("-c:s".into());
        args.push("mov_text".into());
        notes.push("Text subtitles converted to mov_text for MP4/MOV compatibility.".to_string());
    }

    if matches!(settings.container, ContainerFormat::Mp4 | ContainerFormat::Mov | ContainerFormat::M4v)
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
    // Audio-only source -> Video container: synthesize clean waveform visualization video stream.
    if !has_video && has_audio {
        args.push("-filter_complex".into());
        args.push("[0:a]showwaves=s=1280x720:mode=line:rate=30:colors=white[v]".into());
        args.push("-map".into());
        args.push("[v]".into());
        args.push("-map".into());
        args.push("0:a:0".into());
        args.push("-sn".into());

        let mut synth_settings = settings.clone();
        if synth_settings.video_codec == VideoCodec::Copy {
            synth_settings.video_codec = VideoCodec::Libx264;
            synth_settings.enforce_compatibility();
        }
        if synth_settings.pixel_format == PixelFormat::KeepOriginal {
            synth_settings.pixel_format = PixelFormat::Yuv420p;
        }
        push_video_codec_args(args, notes, &synth_settings, None, true);

        if synth_settings.audio_codec != AudioCodec::None {
            push_audio_codec_args(args, &synth_settings);
        } else {
            args.push("-an".into());
        }
        args.push("-shortest".into());
        notes.push("Synthesizing 720p waveform video from audio-only input.".to_string());
        return;
    }

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
            ContainerFormat::Mp4 | ContainerFormat::Mov | ContainerFormat::Mkv | ContainerFormat::M4v
        );
    if allow_subtitles {
        args.push("-map".into());
        args.push("0:s?".into());
    } else {
        args.push("-sn".into());
    }

    if has_video {
        push_video_codec_args(args, notes, settings, media, false);
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
            ContainerFormat::Mp4 | ContainerFormat::Mov | ContainerFormat::M4v => {
                args.push("mov_text".into())
            }
            _ => args.push("copy".into()),
        }
    }
}

fn push_video_codec_args(
    args: &mut Vec<OsString>,
    notes: &mut Vec<String>,
    settings: &ConversionSettings,
    media: Option<&MediaInfo>,
    skip_vf: bool,
) {
    if settings.video_codec == VideoCodec::Copy {
        args.push("-c:v".into());
        args.push("copy".into());
        if settings.resolution != ResolutionOption::Original {
            notes.push("Resolution scaling skipped during Stream Copy.".to_string());
        }
        if matches!(settings.container, ContainerFormat::Mp4 | ContainerFormat::Mov | ContainerFormat::M4v)
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

    if !skip_vf && let Some(vf) = build_video_filter(settings, media) {
        args.push("-vf".into());
        args.push(vf.into());
    }

    let is_image_or_gif_input = media.is_some_and(|m| {
        m.is_still_image
            || matches!(
                m.video_codec.as_deref(),
                Some("gif" | "webp" | "apng" | "png" | "mjpeg" | "bmp")
            )
    });

    let effective_pix_fmt = if matches!(
        settings.video_codec,
        VideoCodec::H264Nvenc | VideoCodec::H264Amf | VideoCodec::H264Qsv
    ) && settings.pixel_format == PixelFormat::Yuv420p10le
    {
        notes.push("Hardware H.264 uses 8-bit yuv420p for compatibility.".to_string());
        PixelFormat::Yuv420p
    } else if (is_image_or_gif_input || settings.container == ContainerFormat::Dv)
        && settings.pixel_format == PixelFormat::KeepOriginal
    {
        PixelFormat::Yuv420p
    } else {
        settings.pixel_format
    };

    if let Some(pix) = effective_pix_fmt.ffmpeg_value() {
        args.push("-pix_fmt".into());
        args.push(pix.into());
    }

    if settings.container == ContainerFormat::Dv {
        args.push("-r".into());
        args.push("25".into());
    } else if let Some(fps_str) = settings.fps.ffmpeg_value() {
        args.push("-r".into());
        args.push(fps_str.into());
    }

    push_encoder_tuning_args(args, settings, effective_pix_fmt);
}

fn build_video_filter(settings: &ConversionSettings, media: Option<&MediaInfo>) -> Option<String> {
    if settings.container == ContainerFormat::Dv {
        return Some("scale=720:576:flags=lanczos".to_string());
    }

    let is_image_or_gif_input = media.is_some_and(|m| {
        m.is_still_image
            || matches!(
                m.video_codec.as_deref(),
                Some("gif" | "webp" | "apng" | "png" | "mjpeg" | "bmp")
            )
    });

    let mut filters = Vec::new();
    match settings.resolution {
        ResolutionOption::Original => {
            let needs_even = is_image_or_gif_input
                || (settings.safety.even_dimensions_guard
                    && settings.pixel_format != PixelFormat::KeepOriginal
                    && match media.and_then(|m| m.width.zip(m.height)) {
                        Some((w, h)) => w % 2 != 0 || h % 2 != 0,
                        None => true,
                    });
            if needs_even {
                filters.push("scale=trunc(iw/2)*2:trunc(ih/2)*2".to_string());
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
        VideoCodec::Mpeg2Video | VideoCodec::Wmv2 => match settings.rate_control {
            VideoRateControl::Quality(q) => {
                let qv = (q / 2).clamp(2, 16);
                args.push("-q:v".into());
                args.push(qv.to_string().into());
            }
            VideoRateControl::BitrateKbps(kbps) => {
                args.push("-b:v".into());
                args.push(format!("{kbps}k").into());
            }
        },
        VideoCodec::LibTheora => match settings.rate_control {
            VideoRateControl::Quality(q) => {
                let tq = 10u8.saturating_sub((q.clamp(12, 40) - 12) / 3).clamp(1, 10);
                args.push("-q:v".into());
                args.push(tq.to_string().into());
            }
            VideoRateControl::BitrateKbps(kbps) => {
                args.push("-b:v".into());
                args.push(format!("{kbps}k").into());
            }
        },
        VideoCodec::DvVideo => {}
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
        && matches!(
            settings.container,
            ContainerFormat::Mp4 | ContainerFormat::Mov | ContainerFormat::M4v
        )
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

    if settings.audio_codec == AudioCodec::Dts {
        args.push("-strict".into());
        args.push("-2".into());
    }

    if settings.audio_codec == AudioCodec::AmrNb {
        args.push("-b:a".into());
        args.push("12.2k".into());
        args.push("-ar".into());
        args.push("8000".into());
        args.push("-ac".into());
        args.push("1".into());
        return;
    }

    if settings.container == ContainerFormat::Dv {
        args.push("-ar".into());
        args.push("48000".into());
        args.push("-ac".into());
        args.push("2".into());
        return;
    }

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
    fn gif_to_mp4_applies_even_scale_and_yuv420p() {
        let settings = ConversionSettings::from_preset(PresetProfile::Mp4UniversalH264);
        let gif_media = MediaInfo {
            has_video: true,
            has_audio: false,
            is_still_image: false,
            video_codec: Some("gif".to_string()),
            width: Some(321),
            height: Some(241),
            ..Default::default()
        };
        let plan = build_command_plan(
            &settings,
            Some(&gif_media),
            Path::new("anim.gif"),
            Path::new("anim.tmp.mp4"),
        )
        .unwrap();
        let args = arg_list(&plan);
        assert!(!args.contains(&"-loop".to_string()));
        assert!(args
            .windows(2)
            .any(|w| w == ["-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2"]));
        assert!(args.windows(2).any(|w| w == ["-pix_fmt", "yuv420p"]));
        assert!(args.windows(2).any(|w| w == ["-f", "mp4"]));
    }

    #[test]
    fn static_image_to_mp4_uses_loop_and_duration() {
        let mut settings = ConversionSettings::from_preset(PresetProfile::Mp4UniversalH264);
        settings.image_to_video_duration_secs = 8;
        let img_media = MediaInfo {
            has_video: true,
            has_audio: false,
            is_still_image: true,
            video_codec: Some("png".to_string()),
            width: Some(501),
            height: Some(303),
            ..Default::default()
        };
        let plan = build_command_plan(
            &settings,
            Some(&img_media),
            Path::new("poster.png"),
            Path::new("poster.tmp.mp4"),
        )
        .unwrap();
        let args = arg_list(&plan);
        assert!(args.windows(2).any(|w| w == ["-loop", "1"]));
        assert!(args.windows(2).any(|w| w == ["-t", "8"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2"]));
    }

    #[test]
    fn audio_to_mp4_generates_showwaves_filter_complex() {
        let settings = ConversionSettings::from_preset(PresetProfile::Mp4UniversalH264);
        let audio_media = MediaInfo {
            has_video: false,
            has_audio: true,
            ..Default::default()
        };
        let plan = build_command_plan(
            &settings,
            Some(&audio_media),
            Path::new("song.mp3"),
            Path::new("song.tmp.mp4"),
        )
        .unwrap();
        let args = arg_list(&plan);
        let fc_idx = args.iter().position(|a| a == "-filter_complex").unwrap();
        assert!(args[fc_idx + 1].contains("showwaves=s=1280x720"));
        assert!(args.windows(2).any(|w| w == ["-map", "[v]"]));
        assert!(args.windows(2).any(|w| w == ["-map", "0:a:0"]));
        assert!(args.contains(&"-shortest".to_string()));
        assert!(!args.contains(&"-vf".to_string()));
    }

    #[test]
    fn audio_to_png_uses_cover_art_or_showwavespic() {
        let png_settings = ConversionSettings::from_preset(PresetProfile::StaticPngLossless);

        // No embedded cover art -> showwavespic
        let audio_only = MediaInfo {
            has_video: false,
            has_audio: true,
            ..Default::default()
        };
        let wave_plan = build_command_plan(
            &png_settings,
            Some(&audio_only),
            Path::new("track.wav"),
            Path::new("track.tmp.png"),
        )
        .unwrap();
        let wave_args = arg_list(&wave_plan);
        let fc_idx = wave_args.iter().position(|a| a == "-filter_complex").unwrap();
        assert!(wave_args[fc_idx + 1].contains("showwavespic"));
        assert!(wave_args.windows(2).any(|w| w == ["-frames:v", "1"]));

        // With embedded cover art -> extracts 0:v:0
        let with_cover = MediaInfo {
            has_video: true,
            has_audio: true,
            is_still_image: true,
            ..Default::default()
        };
        let cover_plan = build_command_plan(
            &png_settings,
            Some(&with_cover),
            Path::new("album.mp3"),
            Path::new("album.tmp.png"),
        )
        .unwrap();
        let cover_args = arg_list(&cover_plan);
        assert!(cover_args.windows(2).any(|w| w == ["-map", "0:v:0"]));
        assert!(!cover_args.contains(&"-filter_complex".to_string()));
    }

    #[test]
    fn mkv_to_srt_extracts_subtitle_stream() {
        let mut settings = ConversionSettings::default();
        settings.set_mode(ConvertMode::Subtitle);
        settings.container = ContainerFormat::Srt;
        settings.enforce_compatibility();

        let media = MediaInfo {
            has_video: true,
            has_audio: true,
            has_subtitle: true,
            has_text_subtitles: true,
            ..Default::default()
        };
        let plan = build_command_plan(
            &settings,
            Some(&media),
            Path::new("movie.mkv"),
            Path::new("movie.tmp.srt"),
        )
        .unwrap();
        let args = arg_list(&plan);
        assert!(args.windows(2).any(|w| w == ["-map", "0:s:0"]));
        assert!(args.contains(&"-vn".to_string()));
        assert!(args.contains(&"-an".to_string()));
        assert!(args.windows(2).any(|w| w == ["-c:s", "srt"]));
        assert!(args.windows(2).any(|w| w == ["-f", "srt"]));
    }

    #[test]
    fn image_to_ico_scales_within_256x256() {
        let mut settings = ConversionSettings::default();
        settings.set_mode(ConvertMode::ImageGif);
        settings.container = ContainerFormat::Ico;
        settings.image_gif.limit_max_width = false;
        settings.enforce_compatibility();

        let media = MediaInfo {
            has_video: true,
            is_still_image: true,
            width: Some(1024),
            height: Some(1024),
            ..Default::default()
        };
        let plan = build_command_plan(
            &settings,
            Some(&media),
            Path::new("logo.png"),
            Path::new("logo.tmp.ico"),
        )
        .unwrap();
        let args = arg_list(&plan);
        let vf_idx = args.iter().position(|a| a == "-vf").unwrap();
        assert!(args[vf_idx + 1].contains("min(256,iw)"));
        assert!(args[vf_idx + 1].contains("min(256,ih)"));
        assert!(args.windows(2).any(|w| w == ["-f", "ico"]));
    }

    #[test]
    fn rejects_invalid_mode_stream_combinations() {
        let audio_only = MediaInfo {
            has_video: false,
            has_audio: true,
            ..Default::default()
        };
        let mut sub_settings = ConversionSettings::default();
        sub_settings.set_mode(ConvertMode::Subtitle);
        assert!(build_command_plan(
            &sub_settings,
            Some(&audio_only),
            Path::new("song.mp3"),
            Path::new("song.tmp.srt")
        )
        .is_err());

        let video_only = MediaInfo {
            has_video: true,
            has_audio: false,
            ..Default::default()
        };
        let mp3_settings = ConversionSettings::from_preset(PresetProfile::ExtractMp3320);
        assert!(build_command_plan(
            &mp3_settings,
            Some(&video_only),
            Path::new("silent.mp4"),
            Path::new("silent.tmp.mp3")
        )
        .is_err());
    }
}
