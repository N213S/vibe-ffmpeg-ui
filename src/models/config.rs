use serde::{Deserialize, Serialize};

/// Top-level conversion mode controlling visible parameters, presets, and verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvertMode {
    Video,
    AudioOnly,
    ImageGif,
    Remux,
}

impl ConvertMode {
    pub const ALL: &'static [Self] = &[Self::Video, Self::AudioOnly, Self::ImageGif, Self::Remux];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Video => "Video",
            Self::AudioOnly => "Audio Only",
            Self::ImageGif => "Image / GIF",
            Self::Remux => "Remux (Fast Copy)",
        }
    }

    pub const fn containers(self) -> &'static [ContainerFormat] {
        match self {
            Self::Video => &[
                ContainerFormat::Mp4,
                ContainerFormat::Mkv,
                ContainerFormat::Mov,
                ContainerFormat::WebM,
            ],
            Self::AudioOnly => &[
                ContainerFormat::Mp3,
                ContainerFormat::Flac,
                ContainerFormat::Wav,
                ContainerFormat::M4a,
                ContainerFormat::Opus,
                ContainerFormat::Ogg,
            ],
            Self::ImageGif => &[
                ContainerFormat::Gif,
                ContainerFormat::Webp,
                ContainerFormat::Png,
                ContainerFormat::Jpg,
                ContainerFormat::Avif,
            ],
            Self::Remux => &[
                ContainerFormat::Mp4,
                ContainerFormat::Mkv,
                ContainerFormat::Mov,
                ContainerFormat::WebM,
            ],
        }
    }
}

/// Target output container / image format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerFormat {
    Mp4,
    Mkv,
    Mov,
    WebM,
    Mp3,
    Flac,
    Wav,
    M4a,
    Opus,
    Ogg,
    Gif,
    Webp,
    Png,
    Jpg,
    Avif,
}

impl ContainerFormat {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Mp4 => "MP4 (Universal)",
            Self::Mkv => "MKV (Matroska)",
            Self::Mov => "MOV (QuickTime)",
            Self::WebM => "WebM (VP9 / AV1)",
            Self::Mp3 => "MP3 (MPEG Audio)",
            Self::Flac => "FLAC (Lossless Audio)",
            Self::Wav => "WAV (Uncompressed PCM)",
            Self::M4a => "M4A (AAC Audio)",
            Self::Opus => "Opus (Ogg Opus)",
            Self::Ogg => "OGG (Ogg Vorbis)",
            Self::Gif => "GIF (Animated Palette)",
            Self::Webp => "WebP (Static / Animated)",
            Self::Png => "PNG (Lossless Image)",
            Self::Jpg => "JPG (JPEG Image)",
            Self::Avif => "AVIF (AV1 Image)",
        }
    }

    pub const fn extension(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
            Self::Mov => "mov",
            Self::WebM => "webm",
            Self::Mp3 => "mp3",
            Self::Flac => "flac",
            Self::Wav => "wav",
            Self::M4a => "m4a",
            Self::Opus => "opus",
            Self::Ogg => "ogg",
            Self::Gif => "gif",
            Self::Webp => "webp",
            Self::Png => "png",
            Self::Jpg => "jpg",
            Self::Avif => "avif",
        }
    }

    pub const fn muxer_name(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "matroska",
            Self::Mov => "mov",
            Self::WebM => "webm",
            Self::Mp3 => "mp3",
            Self::Flac => "flac",
            Self::Wav => "wav",
            Self::M4a => "ipod",
            Self::Opus => "opus",
            Self::Ogg => "ogg",
            Self::Gif => "gif",
            Self::Webp => "webp",
            Self::Png | Self::Jpg => "image2",
            Self::Avif => "avif",
        }
    }

    pub const fn is_image_or_gif(self) -> bool {
        matches!(
            self,
            Self::Gif | Self::Webp | Self::Png | Self::Jpg | Self::Avif
        )
    }

    pub const fn supports_faststart(self) -> bool {
        matches!(self, Self::Mp4 | Self::Mov | Self::M4a)
    }
}

/// Video codec options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoCodec {
    Copy,
    Libx264,
    Libx265,
    LibSvtAv1,
    LibVpxVp9,
    H264Nvenc,
    HevcNvenc,
    Av1Nvenc,
    H264Amf,
    HevcAmf,
    H264Qsv,
    HevcQsv,
}

impl VideoCodec {
    pub const ALL: &'static [Self] = &[
        Self::Libx264,
        Self::Libx265,
        Self::LibSvtAv1,
        Self::LibVpxVp9,
        Self::H264Nvenc,
        Self::HevcNvenc,
        Self::Av1Nvenc,
        Self::H264Amf,
        Self::HevcAmf,
        Self::H264Qsv,
        Self::HevcQsv,
        Self::Copy,
    ];

    pub const fn ffmpeg_name(self) -> &'static str {
        match self {
            Self::Copy => "copy",
            Self::Libx264 => "libx264",
            Self::Libx265 => "libx265",
            Self::LibSvtAv1 => "libsvtav1",
            Self::LibVpxVp9 => "libvpx-vp9",
            Self::H264Nvenc => "h264_nvenc",
            Self::HevcNvenc => "hevc_nvenc",
            Self::Av1Nvenc => "av1_nvenc",
            Self::H264Amf => "h264_amf",
            Self::HevcAmf => "hevc_amf",
            Self::H264Qsv => "h264_qsv",
            Self::HevcQsv => "hevc_qsv",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Copy => "Stream Copy (No Re-encode)",
            Self::Libx264 => "H.264 / AVC (libx264)",
            Self::Libx265 => "H.265 / HEVC (libx265)",
            Self::LibSvtAv1 => "AV1 (libsvtav1)",
            Self::LibVpxVp9 => "VP9 (libvpx-vp9)",
            Self::H264Nvenc => "H.264 NVIDIA (h264_nvenc)",
            Self::HevcNvenc => "HEVC NVIDIA (hevc_nvenc)",
            Self::Av1Nvenc => "AV1 NVIDIA (av1_nvenc)",
            Self::H264Amf => "H.264 AMD (h264_amf)",
            Self::HevcAmf => "HEVC AMD (hevc_amf)",
            Self::H264Qsv => "H.264 Intel QSV (h264_qsv)",
            Self::HevcQsv => "HEVC Intel QSV (hevc_qsv)",
        }
    }

    pub const fn is_hevc(self) -> bool {
        matches!(
            self,
            Self::Libx265 | Self::HevcNvenc | Self::HevcAmf | Self::HevcQsv
        )
    }

    pub const fn is_compatible_with(self, container: ContainerFormat) -> bool {
        match container {
            ContainerFormat::Mkv => true,
            ContainerFormat::Mp4 | ContainerFormat::Mov => !matches!(self, Self::LibVpxVp9),
            ContainerFormat::WebM => matches!(
                self,
                Self::LibVpxVp9 | Self::LibSvtAv1 | Self::Av1Nvenc | Self::Copy
            ),
            _ => false,
        }
    }
}

/// Audio codec options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioCodec {
    Aac,
    LibOpus,
    LibMp3Lame,
    LibVorbis,
    Flac,
    PcmS16le,
    Copy,
    None,
}

impl AudioCodec {
    pub const ALL: &'static [Self] = &[
        Self::Aac,
        Self::LibOpus,
        Self::LibMp3Lame,
        Self::LibVorbis,
        Self::Flac,
        Self::PcmS16le,
        Self::Copy,
        Self::None,
    ];

    pub const fn ffmpeg_name(self) -> &'static str {
        match self {
            Self::Aac => "aac",
            Self::LibOpus => "libopus",
            Self::LibMp3Lame => "libmp3lame",
            Self::LibVorbis => "libvorbis",
            Self::Flac => "flac",
            Self::PcmS16le => "pcm_s16le",
            Self::Copy => "copy",
            Self::None => "none",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Aac => "AAC (Universal)",
            Self::LibOpus => "Opus (High Efficiency)",
            Self::LibMp3Lame => "MP3 (LAME)",
            Self::LibVorbis => "Vorbis (OGG)",
            Self::Flac => "FLAC (Lossless)",
            Self::PcmS16le => "PCM 16-bit (WAV/MOV)",
            Self::Copy => "Stream Copy (No Re-encode)",
            Self::None => "Mute (Strip Audio -an)",
        }
    }

    pub const fn is_lossless(self) -> bool {
        matches!(self, Self::Flac | Self::PcmS16le | Self::Copy | Self::None)
    }

    pub const fn is_compatible_with(self, container: ContainerFormat) -> bool {
        match container {
            ContainerFormat::Mp4 => matches!(
                self,
                Self::Aac | Self::LibMp3Lame | Self::LibOpus | Self::Copy | Self::None
            ),
            ContainerFormat::Mov => matches!(
                self,
                Self::Aac | Self::LibMp3Lame | Self::PcmS16le | Self::Copy | Self::None
            ),
            ContainerFormat::Mkv => true,
            ContainerFormat::WebM => matches!(self, Self::LibOpus | Self::Copy | Self::None),
            ContainerFormat::Mp3 => matches!(self, Self::LibMp3Lame | Self::Copy),
            ContainerFormat::Flac => matches!(self, Self::Flac | Self::Copy),
            ContainerFormat::Wav => matches!(self, Self::PcmS16le),
            ContainerFormat::M4a => matches!(self, Self::Aac | Self::Copy),
            ContainerFormat::Opus => matches!(self, Self::LibOpus | Self::Copy),
            ContainerFormat::Ogg => matches!(self, Self::LibVorbis | Self::LibOpus | Self::Copy),
            c if c.is_image_or_gif() => matches!(self, Self::None),
            _ => false,
        }
    }
}

/// Rate control mode for video encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoRateControl {
    Quality(u8),
    BitrateKbps(u32),
}

/// Encoding speed vs compression efficiency tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncodeSpeed {
    Ultrafast,
    Veryfast,
    Fast,
    Medium,
    Slow,
    Slower,
}

impl EncodeSpeed {
    pub const ALL: &'static [Self] = &[
        Self::Ultrafast,
        Self::Veryfast,
        Self::Fast,
        Self::Medium,
        Self::Slow,
        Self::Slower,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Ultrafast => "Ultrafast",
            Self::Veryfast => "Veryfast",
            Self::Fast => "Fast",
            Self::Medium => "Medium (Balanced)",
            Self::Slow => "Slow (Better Compression)",
            Self::Slower => "Slower",
        }
    }

    pub const fn x264_preset(self) -> &'static str {
        match self {
            Self::Ultrafast => "ultrafast",
            Self::Veryfast => "veryfast",
            Self::Fast => "fast",
            Self::Medium => "medium",
            Self::Slow => "slow",
            Self::Slower => "slower",
        }
    }

    pub const fn svt_av1_preset(self) -> &'static str {
        match self {
            Self::Ultrafast => "10",
            Self::Veryfast => "8",
            Self::Fast => "7",
            Self::Medium => "6",
            Self::Slow => "5",
            Self::Slower => "4",
        }
    }

    pub const fn vp9_cpu_used(self) -> &'static str {
        match self {
            Self::Ultrafast => "5",
            Self::Veryfast => "4",
            Self::Fast => "3",
            Self::Medium => "2",
            Self::Slow => "1",
            Self::Slower => "0",
        }
    }

    pub const fn nvenc_preset(self) -> &'static str {
        match self {
            Self::Ultrafast => "p1",
            Self::Veryfast => "p2",
            Self::Fast => "p3",
            Self::Medium => "p4",
            Self::Slow => "p5",
            Self::Slower => "p6",
        }
    }

    pub const fn amf_quality(self) -> &'static str {
        match self {
            Self::Ultrafast | Self::Veryfast | Self::Fast => "speed",
            Self::Medium => "balanced",
            Self::Slow | Self::Slower => "quality",
        }
    }
}

/// Output pixel format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PixelFormat {
    Yuv420p,
    Yuv420p10le,
    KeepOriginal,
}

impl PixelFormat {
    pub const ALL: &'static [Self] = &[Self::Yuv420p, Self::Yuv420p10le, Self::KeepOriginal];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Yuv420p => "yuv420p (8-bit Universal)",
            Self::Yuv420p10le => "yuv420p10le (10-bit HDR/Archive)",
            Self::KeepOriginal => "Keep Source Format",
        }
    }

    pub const fn ffmpeg_value(self) -> Option<&'static str> {
        match self {
            Self::Yuv420p => Some("yuv420p"),
            Self::Yuv420p10le => Some("yuv420p10le"),
            Self::KeepOriginal => None,
        }
    }
}

/// Output resolution scaling option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionOption {
    Original,
    Uhd2160p,
    Fhd1080p,
    Hd720p,
    Sd480p,
    Custom { width: u32, height: u32 },
}

impl ResolutionOption {
    pub const PRESETS: &'static [Self] = &[
        Self::Original,
        Self::Uhd2160p,
        Self::Fhd1080p,
        Self::Hd720p,
        Self::Sd480p,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Original => "Original Resolution",
            Self::Uhd2160p => "4K (2160p height, keep aspect)",
            Self::Fhd1080p => "1080p (FHD height, keep aspect)",
            Self::Hd720p => "720p (HD height, keep aspect)",
            Self::Sd480p => "480p (SD height, keep aspect)",
            Self::Custom { .. } => "Custom WxH",
        }
    }
}

/// Target frame rate option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FpsOption {
    Original,
    Fps24,
    Fps30,
    Fps60,
}

impl FpsOption {
    pub const ALL: &'static [Self] = &[Self::Original, Self::Fps24, Self::Fps30, Self::Fps60];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Original => "Original FPS",
            Self::Fps24 => "24 fps",
            Self::Fps30 => "30 fps",
            Self::Fps60 => "60 fps",
        }
    }

    pub const fn ffmpeg_value(self) -> Option<&'static str> {
        match self {
            Self::Original => None,
            Self::Fps24 => Some("24"),
            Self::Fps30 => Some("30"),
            Self::Fps60 => Some("60"),
        }
    }
}

/// Audio channel layout option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioChannels {
    KeepOriginal,
    Stereo,
    Mono,
}

impl AudioChannels {
    pub const ALL: &'static [Self] = &[Self::KeepOriginal, Self::Stereo, Self::Mono];

    pub const fn label(self) -> &'static str {
        match self {
            Self::KeepOriginal => "Original Channels",
            Self::Stereo => "Stereo (2.0)",
            Self::Mono => "Mono (1.0)",
        }
    }

    pub const fn ffmpeg_value(self) -> Option<&'static str> {
        match self {
            Self::KeepOriginal => None,
            Self::Stereo => Some("2"),
            Self::Mono => Some("1"),
        }
    }
}

/// Audio sample rate option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioSampleRate {
    KeepOriginal,
    Hz44100,
    Hz48000,
}

impl AudioSampleRate {
    pub const ALL: &'static [Self] = &[Self::KeepOriginal, Self::Hz44100, Self::Hz48000];

    pub const fn label(self) -> &'static str {
        match self {
            Self::KeepOriginal => "Original Sample Rate",
            Self::Hz44100 => "44.1 kHz",
            Self::Hz48000 => "48 kHz",
        }
    }

    pub const fn ffmpeg_value(self) -> Option<&'static str> {
        match self {
            Self::KeepOriginal => None,
            Self::Hz44100 => Some("44100"),
            Self::Hz48000 => Some("48000"),
        }
    }
}

/// Integrity and corruption-prevention guard flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetySettings {
    pub faststart: bool,
    pub even_dimensions_guard: bool,
    pub repair_timestamps: bool,
    pub preserve_metadata: bool,
    pub keep_subtitles: bool,
    pub deep_decode_verify: bool,
    pub overwrite_existing: bool,
}

impl Default for SafetySettings {
    fn default() -> Self {
        Self {
            faststart: true,
            even_dimensions_guard: true,
            repair_timestamps: true,
            preserve_metadata: true,
            keep_subtitles: false,
            deep_decode_verify: true,
            overwrite_existing: false,
        }
    }
}

/// Settings specific to Image / GIF mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageGifSettings {
    pub fps: u32,
    pub limit_max_width: bool,
    pub max_width: u32,
    /// WebP mode: `true` encodes multi-frame animated WebP (`libwebp_anim`), `false` encodes single-frame static WebP.
    pub webp_animated: bool,
    /// Quality (1..=100) used for WebP and JPG.
    pub quality: u32,
    /// PNG compression level (0..=9).
    pub png_compression: u8,
    /// AVIF CRF (10..=55, lower = higher quality).
    pub avif_crf: u8,
}

impl Default for ImageGifSettings {
    fn default() -> Self {
        Self {
            fps: 15,
            limit_max_width: true,
            max_width: 640,
            webp_animated: true,
            quality: 85,
            png_compression: 6,
            avif_crf: 28,
        }
    }
}

/// Complete conversion settings applied to queued files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionSettings {
    pub mode: ConvertMode,
    pub container: ContainerFormat,
    pub video_codec: VideoCodec,
    pub rate_control: VideoRateControl,
    pub speed: EncodeSpeed,
    pub pixel_format: PixelFormat,
    pub resolution: ResolutionOption,
    pub fps: FpsOption,
    pub audio_codec: AudioCodec,
    pub audio_bitrate_kbps: u32,
    pub audio_channels: AudioChannels,
    pub audio_sample_rate: AudioSampleRate,
    pub image_gif: ImageGifSettings,
    pub safety: SafetySettings,
    pub output_prefix: String,
    pub output_suffix: String,
    pub custom_output_dir: Option<std::path::PathBuf>,
}

impl Default for ConversionSettings {
    fn default() -> Self {
        Self::from_preset(PresetProfile::Mp4UniversalH264)
    }
}

/// Curated one-click presets spanning all four `ConvertMode`s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetProfile {
    Mp4UniversalH264,
    Mp4HevcCompact,
    MkvArchiveH265,
    WebmStreamingVp9,
    ExtractMp3320,
    LosslessFlac,
    UncompressedWav,
    AudioM4aAac,
    AudioOpus160,
    AudioOggVorbis,
    SmoothGif15fps,
    AnimatedWebp15fps,
    StaticPngLossless,
    StaticJpgHigh,
    StaticAvifCompact,
    InstantRemuxMp4,
    InstantRemuxMkv,
}

impl PresetProfile {
    pub const ALL: &'static [Self] = &[
        Self::Mp4UniversalH264,
        Self::Mp4HevcCompact,
        Self::MkvArchiveH265,
        Self::WebmStreamingVp9,
        Self::ExtractMp3320,
        Self::LosslessFlac,
        Self::UncompressedWav,
        Self::AudioM4aAac,
        Self::AudioOpus160,
        Self::AudioOggVorbis,
        Self::SmoothGif15fps,
        Self::AnimatedWebp15fps,
        Self::StaticPngLossless,
        Self::StaticJpgHigh,
        Self::StaticAvifCompact,
        Self::InstantRemuxMp4,
        Self::InstantRemuxMkv,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Mp4UniversalH264 => "[Video] MP4 Universal H.264 + AAC",
            Self::Mp4HevcCompact => "[Video] MP4 Compact H.265 + hvc1",
            Self::MkvArchiveH265 => "[Video] MKV Archive H.265 + Opus",
            Self::WebmStreamingVp9 => "[Video] WebM VP9 + Opus",
            Self::ExtractMp3320 => "[Audio] Extract MP3 320k",
            Self::LosslessFlac => "[Audio] Lossless FLAC",
            Self::UncompressedWav => "[Audio] Uncompressed WAV (PCM 16-bit)",
            Self::AudioM4aAac => "[Audio] M4A AAC 256k",
            Self::AudioOpus160 => "[Audio] Opus 160k",
            Self::AudioOggVorbis => "[Audio] OGG Vorbis 192k",
            Self::SmoothGif15fps => "[Image] Smooth GIF 15fps (PaletteGen)",
            Self::AnimatedWebp15fps => "[Image] Animated WebP 15fps",
            Self::StaticPngLossless => "[Image] PNG Lossless Frame",
            Self::StaticJpgHigh => "[Image] JPG High Quality",
            Self::StaticAvifCompact => "[Image] AVIF Compact Frame",
            Self::InstantRemuxMp4 => "[Remux] Instant Remux to MP4 (-c copy)",
            Self::InstantRemuxMkv => "[Remux] Instant Remux to MKV (-c copy)",
        }
    }
}

impl ConversionSettings {
    /// Builds complete, mode-aligned settings from a curated preset.
    pub fn from_preset(preset: PresetProfile) -> Self {
        let mut base = Self {
            mode: ConvertMode::Video,
            container: ContainerFormat::Mp4,
            video_codec: VideoCodec::Libx264,
            rate_control: VideoRateControl::Quality(20),
            speed: EncodeSpeed::Medium,
            pixel_format: PixelFormat::Yuv420p,
            resolution: ResolutionOption::Original,
            fps: FpsOption::Original,
            audio_codec: AudioCodec::Aac,
            audio_bitrate_kbps: 192,
            audio_channels: AudioChannels::Stereo,
            audio_sample_rate: AudioSampleRate::Hz48000,
            image_gif: ImageGifSettings::default(),
            safety: SafetySettings::default(),
            output_prefix: String::new(),
            output_suffix: String::new(),
            custom_output_dir: None,
        };

        match preset {
            PresetProfile::Mp4UniversalH264 => {}
            PresetProfile::Mp4HevcCompact => {
                base.video_codec = VideoCodec::Libx265;
                base.rate_control = VideoRateControl::Quality(24);
            }
            PresetProfile::MkvArchiveH265 => {
                base.container = ContainerFormat::Mkv;
                base.video_codec = VideoCodec::Libx265;
                base.rate_control = VideoRateControl::Quality(20);
                base.speed = EncodeSpeed::Slow;
                base.audio_codec = AudioCodec::LibOpus;
                base.audio_channels = AudioChannels::KeepOriginal;
                base.safety.keep_subtitles = true;
            }
            PresetProfile::WebmStreamingVp9 => {
                base.container = ContainerFormat::WebM;
                base.video_codec = VideoCodec::LibVpxVp9;
                base.rate_control = VideoRateControl::Quality(30);
                base.audio_codec = AudioCodec::LibOpus;
                base.audio_bitrate_kbps = 128;
            }
            PresetProfile::ExtractMp3320 => {
                base.mode = ConvertMode::AudioOnly;
                base.container = ContainerFormat::Mp3;
                base.audio_codec = AudioCodec::LibMp3Lame;
                base.audio_bitrate_kbps = 320;
                base.audio_sample_rate = AudioSampleRate::Hz44100;
            }
            PresetProfile::LosslessFlac => {
                base.mode = ConvertMode::AudioOnly;
                base.container = ContainerFormat::Flac;
                base.audio_codec = AudioCodec::Flac;
                base.audio_channels = AudioChannels::KeepOriginal;
                base.audio_sample_rate = AudioSampleRate::KeepOriginal;
            }
            PresetProfile::UncompressedWav => {
                base.mode = ConvertMode::AudioOnly;
                base.container = ContainerFormat::Wav;
                base.audio_codec = AudioCodec::PcmS16le;
                base.audio_channels = AudioChannels::KeepOriginal;
                base.audio_sample_rate = AudioSampleRate::Hz48000;
            }
            PresetProfile::AudioM4aAac => {
                base.mode = ConvertMode::AudioOnly;
                base.container = ContainerFormat::M4a;
                base.audio_codec = AudioCodec::Aac;
                base.audio_bitrate_kbps = 256;
            }
            PresetProfile::AudioOpus160 => {
                base.mode = ConvertMode::AudioOnly;
                base.container = ContainerFormat::Opus;
                base.audio_codec = AudioCodec::LibOpus;
                base.audio_bitrate_kbps = 160;
                base.audio_sample_rate = AudioSampleRate::Hz48000;
            }
            PresetProfile::AudioOggVorbis => {
                base.mode = ConvertMode::AudioOnly;
                base.container = ContainerFormat::Ogg;
                base.audio_codec = AudioCodec::LibVorbis;
                base.audio_bitrate_kbps = 192;
                base.audio_sample_rate = AudioSampleRate::Hz44100;
            }
            PresetProfile::SmoothGif15fps => {
                base.mode = ConvertMode::ImageGif;
                base.container = ContainerFormat::Gif;
                base.audio_codec = AudioCodec::None;
                base.image_gif.fps = 15;
                base.image_gif.limit_max_width = true;
                base.image_gif.max_width = 640;
            }
            PresetProfile::AnimatedWebp15fps => {
                base.mode = ConvertMode::ImageGif;
                base.container = ContainerFormat::Webp;
                base.audio_codec = AudioCodec::None;
                base.image_gif.webp_animated = true;
                base.image_gif.fps = 15;
                base.image_gif.limit_max_width = true;
                base.image_gif.max_width = 800;
                base.image_gif.quality = 85;
            }
            PresetProfile::StaticPngLossless => {
                base.mode = ConvertMode::ImageGif;
                base.container = ContainerFormat::Png;
                base.audio_codec = AudioCodec::None;
                base.image_gif.limit_max_width = false;
                base.image_gif.png_compression = 6;
            }
            PresetProfile::StaticJpgHigh => {
                base.mode = ConvertMode::ImageGif;
                base.container = ContainerFormat::Jpg;
                base.audio_codec = AudioCodec::None;
                base.image_gif.limit_max_width = false;
                base.image_gif.quality = 90;
            }
            PresetProfile::StaticAvifCompact => {
                base.mode = ConvertMode::ImageGif;
                base.container = ContainerFormat::Avif;
                base.audio_codec = AudioCodec::None;
                base.image_gif.limit_max_width = false;
                base.image_gif.avif_crf = 28;
            }
            PresetProfile::InstantRemuxMp4 => {
                base.mode = ConvertMode::Remux;
                base.container = ContainerFormat::Mp4;
                base.video_codec = VideoCodec::Copy;
                base.audio_codec = AudioCodec::Copy;
                base.audio_channels = AudioChannels::KeepOriginal;
                base.audio_sample_rate = AudioSampleRate::KeepOriginal;
            }
            PresetProfile::InstantRemuxMkv => {
                base.mode = ConvertMode::Remux;
                base.container = ContainerFormat::Mkv;
                base.video_codec = VideoCodec::Copy;
                base.audio_codec = AudioCodec::Copy;
                base.audio_channels = AudioChannels::KeepOriginal;
                base.audio_sample_rate = AudioSampleRate::KeepOriginal;
                base.safety.keep_subtitles = true;
            }
        }

        base
    }

    /// Switches `ConvertMode` cleanly and adjusts container/codecs to valid defaults for that mode.
    pub fn set_mode(&mut self, new_mode: ConvertMode) {
        if self.mode == new_mode {
            return;
        }
        self.mode = new_mode;
        if !new_mode.containers().contains(&self.container) {
            self.container = new_mode.containers()[0];
        }
        if new_mode == ConvertMode::Remux {
            self.video_codec = VideoCodec::Copy;
            self.audio_codec = AudioCodec::Copy;
        } else if new_mode == ConvertMode::Video && self.video_codec == VideoCodec::Copy {
            self.video_codec = VideoCodec::Libx264;
            if self.audio_codec == AudioCodec::Copy {
                self.audio_codec = AudioCodec::Aac;
            }
        }
        self.enforce_compatibility();
    }

    /// Ensures container, mode, and codec choices are mutually valid so muxing never fails.
    pub fn enforce_compatibility(&mut self) {
        if !self.mode.containers().contains(&self.container) {
            self.container = self.mode.containers()[0];
        }

        match self.mode {
            ConvertMode::Video => {
                if !self.video_codec.is_compatible_with(self.container) {
                    self.video_codec = match self.container {
                        ContainerFormat::WebM => VideoCodec::LibVpxVp9,
                        _ => VideoCodec::Libx264,
                    };
                }
                if !self.audio_codec.is_compatible_with(self.container) {
                    self.audio_codec = match self.container {
                        ContainerFormat::WebM => AudioCodec::LibOpus,
                        _ => AudioCodec::Aac,
                    };
                }
            }
            ConvertMode::AudioOnly => {
                if !self.audio_codec.is_compatible_with(self.container) {
                    self.audio_codec = match self.container {
                        ContainerFormat::Mp3 => AudioCodec::LibMp3Lame,
                        ContainerFormat::Flac => AudioCodec::Flac,
                        ContainerFormat::Wav => AudioCodec::PcmS16le,
                        ContainerFormat::M4a => AudioCodec::Aac,
                        ContainerFormat::Opus => AudioCodec::LibOpus,
                        ContainerFormat::Ogg => AudioCodec::LibVorbis,
                        _ => AudioCodec::LibMp3Lame,
                    };
                }
            }
            ConvertMode::ImageGif => {
                self.audio_codec = AudioCodec::None;
            }
            ConvertMode::Remux => {
                self.video_codec = VideoCodec::Copy;
                self.audio_codec = AudioCodec::Copy;
            }
        }

        // libopus rejects 44100 Hz; enforce 48000 Hz or original when Opus is active.
        if self.audio_codec == AudioCodec::LibOpus
            && self.audio_sample_rate == AudioSampleRate::Hz44100
        {
            self.audio_sample_rate = AudioSampleRate::Hz48000;
        }
    }
}
