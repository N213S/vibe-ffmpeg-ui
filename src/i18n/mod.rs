mod de;
mod en;
mod es;
mod fr;
mod id;
mod it;
mod ja;
mod ko;
mod pt_br;
mod ru;
mod th;
mod vi;
mod zh_cn;
mod zh_tw;

use crate::models::config::{
    AudioChannels, AudioCodec, AudioSampleRate, ContainerFormat, ConvertMode, EncodeSpeed,
    FpsOption, PixelFormat, PresetProfile, ResolutionOption, VideoCodec,
};
use crate::models::media::{MediaInfo, format_duration};
use serde::{Deserialize, Serialize};

/// Supported UI display languages (14 LTR languages).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    #[default]
    En,
    Th,
    Ja,
    ZhCn,
    ZhTw,
    Ko,
    Vi,
    Id,
    Es,
    PtBr,
    Fr,
    De,
    It,
    Ru,
}

impl Language {
    pub const ALL: &'static [Self] = &[
        Self::En,
        Self::Th,
        Self::Ja,
        Self::ZhCn,
        Self::ZhTw,
        Self::Ko,
        Self::Vi,
        Self::Id,
        Self::Es,
        Self::PtBr,
        Self::Fr,
        Self::De,
        Self::It,
        Self::Ru,
    ];

    pub const fn short_code(self) -> &'static str {
        match self {
            Self::En => "EN",
            Self::Th => "TH",
            Self::Ja => "JA",
            Self::ZhCn => "ZH-CN",
            Self::ZhTw => "ZH-TW",
            Self::Ko => "KO",
            Self::Vi => "VI",
            Self::Id => "ID",
            Self::Es => "ES",
            Self::PtBr => "PT-BR",
            Self::Fr => "FR",
            Self::De => "DE",
            Self::It => "IT",
            Self::Ru => "RU",
        }
    }

    pub const fn native_name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Th => "ไทย (Thai)",
            Self::Ja => "日本語 (Japanese)",
            Self::ZhCn => "简体中文 (Simplified Chinese)",
            Self::ZhTw => "繁體中文 (Traditional Chinese)",
            Self::Ko => "한국어 (Korean)",
            Self::Vi => "Tiếng Việt (Vietnamese)",
            Self::Id => "Bahasa Indonesia (Indonesian)",
            Self::Es => "Español (Spanish)",
            Self::PtBr => "Português (Brasil)",
            Self::Fr => "Français (French)",
            Self::De => "Deutsch (German)",
            Self::It => "Italiano (Italian)",
            Self::Ru => "Русский (Russian)",
        }
    }

    /// ASCII/international keywords to help users filter languages quickly.
    pub const fn search_aliases(self) -> &'static str {
        match self {
            Self::En => "en english usa uk",
            Self::Th => "th thai thailand ไทย",
            Self::Ja => "ja jp japanese nihongo 日本語",
            Self::ZhCn => "zh cn zh-cn simplified chinese mandarin 简体中文",
            Self::ZhTw => "zh tw zh-tw traditional chinese taiwan 繁體中文",
            Self::Ko => "ko kr korean hangul 한국어",
            Self::Vi => "vi vn vietnam vietnamese tieng viet",
            Self::Id => "id indonesian bahasa indonesia",
            Self::Es => "es spanish espanol",
            Self::PtBr => "pt br pt-br portuguese brazil brasil portugues",
            Self::Fr => "fr french francais",
            Self::De => "de german deutsch",
            Self::It => "it italian italiano",
            Self::Ru => "ru russian russkiy русский",
        }
    }

    /// Returns whether this language matches a user search filter (case-insensitive).
    pub fn matches_filter(self, query: &str) -> bool {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return true;
        }
        self.short_code().to_lowercase().contains(&q)
            || self.native_name().to_lowercase().contains(&q)
            || self.search_aliases().contains(&q)
    }
}

macro_rules! define_translations {
    ($($field:ident),+ $(,)?) => {
        /// Per-language static translation table with optional entries so missing strings fall back cleanly to English.
        #[derive(Debug, Clone, Copy)]
        pub struct RawTranslations {
            $(pub $field: Option<&'static str>,)+
        }

        impl RawTranslations {
            #[allow(dead_code)]
            pub const EMPTY: Self = Self {
                $($field: None,)+
            };

            #[cfg(test)]
            pub fn missing_or_empty_keys(&self) -> Vec<&'static str> {
                let mut missing = Vec::new();
                $(
                    match self.$field {
                        Some(s) if !s.trim().is_empty() => {}
                        _ => missing.push(stringify!($field)),
                    }
                )+
                missing
            }
        }

        /// Zero-allocation localization accessor with automatic English (`EN`) fallback.
        #[derive(Debug, Clone, Copy)]
        pub struct LocaleText {
            raw: &'static RawTranslations,
        }

        impl LocaleText {
            pub const fn from_raw(raw: &'static RawTranslations) -> Self {
                Self { raw }
            }

            $(
                #[inline]
                pub const fn $field(self) -> &'static str {
                    match self.raw.$field {
                        Some(s) if !s.is_empty() => s,
                        _ => match en::RAW.$field {
                            Some(fallback) => fallback,
                            None => "",
                        },
                    }
                }
            )+

            #[cfg(test)]
            pub fn all_values(self) -> Vec<(&'static str, &'static str)> {
                vec![
                    $((stringify!($field), self.$field()),)+
                ]
            }
        }
    };
}

define_translations!(
    // Language switcher & AI disclaimer
    language_button,
    language_window_title,
    language_select_prompt,
    language_search_hint,
    language_no_match,
    ai_translation_notice,
    close_btn,
    // Top bar
    app_title,
    ffmpeg_detected,
    ffmpeg_missing,
    check_update,
    checking,
    official_source,
    update_available,
    official_builds,
    up_to_date,
    update_failed,
    add_media_files,
    clear_finished,
    clear_all,
    stop_batch,
    convert_verify_all,
    // Queue panel & file picker filters
    file_filter_all_media,
    file_filter_video,
    file_filter_audio,
    file_filter_image,
    file_filter_all_files,
    drop_hover_hint,
    drop_empty_title,
    drop_empty_subtitle,
    browse_media_files,
    inspecting_streams,
    probe_error_prefix,
    ffprobe_not_found,
    remove_btn,
    output_label,
    output_hint,
    verifying_progress,
    saved_label,
    stream_singular,
    stream_plural,
    reveal_in_folder,
    status_probing,
    status_ready,
    status_converting,
    status_verifying,
    status_verified_ok,
    status_failed,
    status_cancelled,
    metric_speed,
    metric_fps,
    metric_bitrate,
    media_video_prefix,
    media_audio_prefix,
    media_ch_suffix,
    media_unprobed,
    verify_header_ok,
    verify_anim_webp_ok,
    verify_decode_clean,
    // Settings sections
    mode_section,
    mode_video,
    mode_audio,
    mode_image_gif,
    mode_remux,
    quick_presets,
    choose_preset_placeholder,
    format_output_section,
    target_format,
    global_affix_label,
    prefix_hint,
    suffix_hint,
    output_folder,
    same_folder_as_source,
    choose_folder,
    reset_to_source,
    video_params_section,
    video_codec,
    unavailable_tag,
    stream_copy_video_note,
    encoder_speed,
    rate_crf,
    rate_bitrate,
    crf_slider_label,
    kbps_unit,
    pixel_format,
    resolution,
    custom_wxh,
    width_short,
    height_short,
    frame_rate,
    image_gif_section,
    animated_webp_toggle,
    fps_label,
    limit_max_width,
    max_width_px,
    gif_palette_note,
    quality_label,
    png_compression_label,
    avif_crf_label,
    remux_section,
    remux_description,
    audio_params_section,
    audio_strip_note,
    audio_codec,
    bitrate_kbps,
    sample_rate,
    channels,
    safety_section,
    guard_faststart,
    guard_even_dims,
    guard_repair_ts,
    guard_deep_verify,
    guard_preserve_meta,
    guard_keep_subs,
    guard_overwrite,
    command_preview_section,
    note_prefix,
    // Container formats
    fmt_mp4,
    fmt_mkv,
    fmt_mov,
    fmt_webm,
    fmt_mp3,
    fmt_flac,
    fmt_wav,
    fmt_m4a,
    fmt_opus,
    fmt_ogg,
    fmt_gif,
    fmt_webp,
    fmt_png,
    fmt_jpg,
    fmt_avif,
    // Codec & encoder option labels
    vcodec_copy,
    vcodec_libx264,
    vcodec_libx265,
    vcodec_libsvtav1,
    vcodec_libvpx_vp9,
    vcodec_h264_nvenc,
    vcodec_hevc_nvenc,
    vcodec_av1_nvenc,
    vcodec_h264_amf,
    vcodec_hevc_amf,
    vcodec_h264_qsv,
    vcodec_hevc_qsv,
    acodec_aac,
    acodec_opus,
    acodec_mp3,
    acodec_vorbis,
    acodec_flac,
    acodec_pcm,
    acodec_copy,
    acodec_mute,
    speed_ultrafast,
    speed_veryfast,
    speed_fast,
    speed_medium,
    speed_slow,
    speed_slower,
    pix_yuv420p,
    pix_yuv420p10le,
    pix_keep_original,
    res_original,
    res_2160p,
    res_1080p,
    res_720p,
    res_480p,
    fps_original,
    ch_original,
    ch_stereo,
    ch_mono,
    sr_original,
    // Curated preset labels
    preset_mp4_h264,
    preset_mp4_hevc,
    preset_mkv_h265,
    preset_webm_vp9,
    preset_extract_mp3,
    preset_lossless_flac,
    preset_wav_pcm,
    preset_m4a_aac,
    preset_opus_160,
    preset_ogg_vorbis,
    preset_smooth_gif,
    preset_animated_webp,
    preset_static_png,
    preset_static_jpg,
    preset_static_avif,
    preset_remux_mp4,
    preset_remux_mkv,
    // Diagnostic log & runtime notifications
    diagnostic_log_title,
    clear_log,
    no_logs_yet,
    log_init_missing,
    log_init_encoders,
    log_skipped_ext,
    log_skipped_prefix,
    log_batch_cancel,
    log_update_ok,
    log_update_err,
    log_unknown_version,
);

const fn raw_table(lang: Language) -> &'static RawTranslations {
    match lang {
        Language::En => &en::RAW,
        Language::Th => &th::RAW,
        Language::Ja => &ja::RAW,
        Language::ZhCn => &zh_cn::RAW,
        Language::ZhTw => &zh_tw::RAW,
        Language::Ko => &ko::RAW,
        Language::Vi => &vi::RAW,
        Language::Id => &id::RAW,
        Language::Es => &es::RAW,
        Language::PtBr => &pt_br::RAW,
        Language::Fr => &fr::RAW,
        Language::De => &de::RAW,
        Language::It => &it::RAW,
        Language::Ru => &ru::RAW,
    }
}

/// Returns the localization accessor for `lang` with safe fallback to English.
pub const fn tr(lang: Language) -> LocaleText {
    LocaleText::from_raw(raw_table(lang))
}

pub const fn mode_label(lang: Language, mode: ConvertMode) -> &'static str {
    let t = tr(lang);
    match mode {
        ConvertMode::Video => t.mode_video(),
        ConvertMode::AudioOnly => t.mode_audio(),
        ConvertMode::ImageGif => t.mode_image_gif(),
        ConvertMode::Remux => t.mode_remux(),
    }
}

pub const fn container_label(lang: Language, c: ContainerFormat) -> &'static str {
    let t = tr(lang);
    match c {
        ContainerFormat::Mp4 => t.fmt_mp4(),
        ContainerFormat::Mkv => t.fmt_mkv(),
        ContainerFormat::Mov => t.fmt_mov(),
        ContainerFormat::WebM => t.fmt_webm(),
        ContainerFormat::Mp3 => t.fmt_mp3(),
        ContainerFormat::Flac => t.fmt_flac(),
        ContainerFormat::Wav => t.fmt_wav(),
        ContainerFormat::M4a => t.fmt_m4a(),
        ContainerFormat::Opus => t.fmt_opus(),
        ContainerFormat::Ogg => t.fmt_ogg(),
        ContainerFormat::Gif => t.fmt_gif(),
        ContainerFormat::Webp => t.fmt_webp(),
        ContainerFormat::Png => t.fmt_png(),
        ContainerFormat::Jpg => t.fmt_jpg(),
        ContainerFormat::Avif => t.fmt_avif(),
    }
}

pub const fn video_codec_label(lang: Language, vc: VideoCodec) -> &'static str {
    let t = tr(lang);
    match vc {
        VideoCodec::Copy => t.vcodec_copy(),
        VideoCodec::Libx264 => t.vcodec_libx264(),
        VideoCodec::Libx265 => t.vcodec_libx265(),
        VideoCodec::LibSvtAv1 => t.vcodec_libsvtav1(),
        VideoCodec::LibVpxVp9 => t.vcodec_libvpx_vp9(),
        VideoCodec::H264Nvenc => t.vcodec_h264_nvenc(),
        VideoCodec::HevcNvenc => t.vcodec_hevc_nvenc(),
        VideoCodec::Av1Nvenc => t.vcodec_av1_nvenc(),
        VideoCodec::H264Amf => t.vcodec_h264_amf(),
        VideoCodec::HevcAmf => t.vcodec_hevc_amf(),
        VideoCodec::H264Qsv => t.vcodec_h264_qsv(),
        VideoCodec::HevcQsv => t.vcodec_hevc_qsv(),
    }
}

pub const fn audio_codec_label(lang: Language, ac: AudioCodec) -> &'static str {
    let t = tr(lang);
    match ac {
        AudioCodec::Aac => t.acodec_aac(),
        AudioCodec::LibOpus => t.acodec_opus(),
        AudioCodec::LibMp3Lame => t.acodec_mp3(),
        AudioCodec::LibVorbis => t.acodec_vorbis(),
        AudioCodec::Flac => t.acodec_flac(),
        AudioCodec::PcmS16le => t.acodec_pcm(),
        AudioCodec::Copy => t.acodec_copy(),
        AudioCodec::None => t.acodec_mute(),
    }
}

pub const fn encode_speed_label(lang: Language, speed: EncodeSpeed) -> &'static str {
    let t = tr(lang);
    match speed {
        EncodeSpeed::Ultrafast => t.speed_ultrafast(),
        EncodeSpeed::Veryfast => t.speed_veryfast(),
        EncodeSpeed::Fast => t.speed_fast(),
        EncodeSpeed::Medium => t.speed_medium(),
        EncodeSpeed::Slow => t.speed_slow(),
        EncodeSpeed::Slower => t.speed_slower(),
    }
}

pub const fn pixel_format_label(lang: Language, pf: PixelFormat) -> &'static str {
    let t = tr(lang);
    match pf {
        PixelFormat::Yuv420p => t.pix_yuv420p(),
        PixelFormat::Yuv420p10le => t.pix_yuv420p10le(),
        PixelFormat::KeepOriginal => t.pix_keep_original(),
    }
}

pub const fn resolution_label(lang: Language, res: ResolutionOption) -> &'static str {
    let t = tr(lang);
    match res {
        ResolutionOption::Original => t.res_original(),
        ResolutionOption::Uhd2160p => t.res_2160p(),
        ResolutionOption::Fhd1080p => t.res_1080p(),
        ResolutionOption::Hd720p => t.res_720p(),
        ResolutionOption::Sd480p => t.res_480p(),
        ResolutionOption::Custom { .. } => t.custom_wxh(),
    }
}

pub const fn fps_label(lang: Language, fps: FpsOption) -> &'static str {
    let t = tr(lang);
    match fps {
        FpsOption::Original => t.fps_original(),
        FpsOption::Fps24 => "24 fps",
        FpsOption::Fps30 => "30 fps",
        FpsOption::Fps60 => "60 fps",
    }
}

pub const fn audio_channels_label(lang: Language, ch: AudioChannels) -> &'static str {
    let t = tr(lang);
    match ch {
        AudioChannels::KeepOriginal => t.ch_original(),
        AudioChannels::Stereo => t.ch_stereo(),
        AudioChannels::Mono => t.ch_mono(),
    }
}

pub const fn audio_sample_rate_label(lang: Language, sr: AudioSampleRate) -> &'static str {
    let t = tr(lang);
    match sr {
        AudioSampleRate::KeepOriginal => t.sr_original(),
        AudioSampleRate::Hz44100 => "44.1 kHz",
        AudioSampleRate::Hz48000 => "48 kHz",
    }
}

pub const fn preset_label(lang: Language, preset: PresetProfile) -> &'static str {
    let t = tr(lang);
    match preset {
        PresetProfile::Mp4UniversalH264 => t.preset_mp4_h264(),
        PresetProfile::Mp4HevcCompact => t.preset_mp4_hevc(),
        PresetProfile::MkvArchiveH265 => t.preset_mkv_h265(),
        PresetProfile::WebmStreamingVp9 => t.preset_webm_vp9(),
        PresetProfile::ExtractMp3320 => t.preset_extract_mp3(),
        PresetProfile::LosslessFlac => t.preset_lossless_flac(),
        PresetProfile::UncompressedWav => t.preset_wav_pcm(),
        PresetProfile::AudioM4aAac => t.preset_m4a_aac(),
        PresetProfile::AudioOpus160 => t.preset_opus_160(),
        PresetProfile::AudioOggVorbis => t.preset_ogg_vorbis(),
        PresetProfile::SmoothGif15fps => t.preset_smooth_gif(),
        PresetProfile::AnimatedWebp15fps => t.preset_animated_webp(),
        PresetProfile::StaticPngLossless => t.preset_static_png(),
        PresetProfile::StaticJpgHigh => t.preset_static_jpg(),
        PresetProfile::StaticAvifCompact => t.preset_static_avif(),
        PresetProfile::InstantRemuxMp4 => t.preset_remux_mp4(),
        PresetProfile::InstantRemuxMkv => t.preset_remux_mkv(),
    }
}

/// Formats probed stream summary cleanly in the selected UI language.
pub fn media_summary_localized(lang: Language, media: &MediaInfo) -> String {
    let t = tr(lang);
    let mut parts = Vec::new();
    if let (Some(w), Some(h)) = (media.width, media.height) {
        let vcodec = media.video_codec.as_deref().unwrap_or("video");
        if let Some(fps) = media.fps {
            parts.push(format!("{w}x{h} @ {fps:.2}fps ({vcodec})"));
        } else {
            parts.push(format!("{w}x{h} ({vcodec})"));
        }
    } else if let Some(ref vcodec) = media.video_codec {
        parts.push(format!("{} {vcodec}", t.media_video_prefix()));
    }

    if let Some(ref acodec) = media.audio_codec {
        let ch = media
            .audio_channels
            .map(|c| format!(" {c}{}", t.media_ch_suffix()))
            .unwrap_or_default();
        parts.push(format!("{} {acodec}{ch}", t.media_audio_prefix()));
    }

    if let Some(d) = media.duration_secs {
        parts.push(format_duration(d));
    }

    if parts.is_empty() {
        t.media_unprobed().to_string()
    } else {
        parts.join("  |  ")
    }
}

/// Maps validator verification note codes to localized strings when recognized.
pub fn verification_note_localized<'a>(lang: Language, note: &'a str) -> &'a str {
    let t = tr(lang);
    match note {
        "Header OK" => t.verify_header_ok(),
        "RIFF/ANIM Verified" => t.verify_anim_webp_ok(),
        "Decode Clean" => t.verify_decode_clean(),
        _ => note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_14_languages_exist_and_have_100_percent_non_empty_translations() {
        assert_eq!(Language::ALL.len(), 14);
        for &lang in Language::ALL {
            assert!(!lang.short_code().trim().is_empty());
            assert!(!lang.native_name().trim().is_empty());

            let raw = raw_table(lang);
            let missing = raw.missing_or_empty_keys();
            assert!(
                missing.is_empty(),
                "Language {lang:?} is missing or has empty keys: {missing:?}"
            );

            let t = tr(lang);
            for (key, value) in t.all_values() {
                assert!(
                    !value.trim().is_empty(),
                    "Resolved translation for {lang:?}.{key} was empty"
                );
            }

            // Also verify enum helper functions return non-empty values for every language.
            for mode in ConvertMode::ALL {
                assert!(!mode_label(lang, *mode).trim().is_empty());
                for container in mode.containers() {
                    assert!(!container_label(lang, *container).trim().is_empty());
                }
            }
            for vc in VideoCodec::ALL {
                assert!(!video_codec_label(lang, *vc).trim().is_empty());
            }
            for ac in AudioCodec::ALL {
                assert!(!audio_codec_label(lang, *ac).trim().is_empty());
            }
            for sp in EncodeSpeed::ALL {
                assert!(!encode_speed_label(lang, *sp).trim().is_empty());
            }
            for pf in PixelFormat::ALL {
                assert!(!pixel_format_label(lang, *pf).trim().is_empty());
            }
            for res in ResolutionOption::PRESETS {
                assert!(!resolution_label(lang, *res).trim().is_empty());
            }
            assert!(
                !resolution_label(
                    lang,
                    ResolutionOption::Custom {
                        width: 1280,
                        height: 720
                    }
                )
                .trim()
                .is_empty()
            );
            for f in FpsOption::ALL {
                assert!(!fps_label(lang, *f).trim().is_empty());
            }
            for ch in AudioChannels::ALL {
                assert!(!audio_channels_label(lang, *ch).trim().is_empty());
            }
            for sr in AudioSampleRate::ALL {
                assert!(!audio_sample_rate_label(lang, *sr).trim().is_empty());
            }
            for preset in PresetProfile::ALL {
                assert!(!preset_label(lang, *preset).trim().is_empty());
            }
        }
    }

    #[test]
    fn missing_or_empty_key_falls_back_safely_to_english() {
        static PARTIAL: RawTranslations = RawTranslations {
            app_title: Some(""),
            ..RawTranslations::EMPTY
        };
        let t = LocaleText::from_raw(&PARTIAL);
        assert_eq!(t.app_title(), en::RAW.app_title.unwrap());
        assert_eq!(
            t.ai_translation_notice(),
            en::RAW.ai_translation_notice.unwrap()
        );
    }

    #[test]
    fn language_filter_matches_code_and_native_names() {
        assert!(Language::Ko.matches_filter("ko"));
        assert!(Language::Ko.matches_filter("한국"));
        assert!(Language::ZhTw.matches_filter("zh-tw"));
        assert!(Language::ZhTw.matches_filter("繁體"));
        assert!(Language::Ru.matches_filter("рус"));
        assert!(!Language::Th.matches_filter("deutsch"));
    }
}
