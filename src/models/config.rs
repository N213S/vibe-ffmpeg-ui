use crate::models::media::MediaInfo;
use serde::{Deserialize, Serialize};

/// Top-level conversion mode controlling visible parameters, presets, and verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvertMode {
    Video,
    AudioOnly,
    ImageGif,
    Subtitle,
    Remux,
}

impl ConvertMode {
    pub const ALL: &'static [Self] = &[
        Self::Video,
        Self::AudioOnly,
        Self::ImageGif,
        Self::Subtitle,
        Self::Remux,
    ];

    #[allow(dead_code)]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Video => "Video",
            Self::AudioOnly => "Audio Only",
            Self::ImageGif => "Image / GIF",
            Self::Subtitle => "Subtitle",
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
                ContainerFormat::Avi,
                ContainerFormat::Flv,
                ContainerFormat::Ts,
                ContainerFormat::M2ts,
                ContainerFormat::Mxf,
                ContainerFormat::ThreeGp,
                ContainerFormat::M4v,
                ContainerFormat::Mpg,
                ContainerFormat::Mpeg,
                ContainerFormat::Vob,
                ContainerFormat::Ogv,
                ContainerFormat::Wmv,
                ContainerFormat::Dv,
            ],
            Self::AudioOnly => &[
                ContainerFormat::Mp3,
                ContainerFormat::Wav,
                ContainerFormat::Flac,
                ContainerFormat::Aac,
                ContainerFormat::M4a,
                ContainerFormat::Ogg,
                ContainerFormat::Opus,
                ContainerFormat::Wma,
                ContainerFormat::Aiff,
                ContainerFormat::Ac3,
                ContainerFormat::Eac3,
                ContainerFormat::Dts,
                ContainerFormat::Alac,
                ContainerFormat::Amr,
                ContainerFormat::Caf,
                ContainerFormat::Mka,
                ContainerFormat::Mp2,
                ContainerFormat::Wv,
            ],
            Self::ImageGif => &[
                ContainerFormat::Png,
                ContainerFormat::Jpg,
                ContainerFormat::Webp,
                ContainerFormat::Gif,
                ContainerFormat::Avif,
                ContainerFormat::Apng,
                ContainerFormat::Bmp,
                ContainerFormat::Tiff,
                ContainerFormat::Tga,
                ContainerFormat::Ico,
                ContainerFormat::Jxl,
                ContainerFormat::Exr,
                ContainerFormat::Hdr,
            ],
            Self::Subtitle => &[
                ContainerFormat::Srt,
                ContainerFormat::Ass,
                ContainerFormat::Ssa,
                ContainerFormat::Vtt,
                ContainerFormat::Lrc,
            ],
            Self::Remux => &[
                ContainerFormat::Mp4,
                ContainerFormat::Mkv,
                ContainerFormat::Mov,
                ContainerFormat::WebM,
                ContainerFormat::Avi,
                ContainerFormat::Flv,
                ContainerFormat::Ts,
                ContainerFormat::M2ts,
                ContainerFormat::M4v,
                ContainerFormat::Mka,
            ],
        }
    }
}

/// Reason why a conversion mode / category is incompatible with the active input media.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncompatibilityReason {
    NoAudioStream,
    NoSubtitleStream,
    NoVisualOrAudioStream,
    NoRemuxableStream,
}

/// Evaluates whether `mode` is supported for `media` (returns `Ok(())` when queue is empty / unprobed).
pub fn check_mode_compatibility(
    mode: ConvertMode,
    media: Option<&MediaInfo>,
) -> Result<(), IncompatibilityReason> {
    let Some(m) = media else {
        return Ok(());
    };
    match mode {
        ConvertMode::Video | ConvertMode::ImageGif => {
            if m.has_video || m.is_still_image || m.has_audio {
                Ok(())
            } else {
                Err(IncompatibilityReason::NoVisualOrAudioStream)
            }
        }
        ConvertMode::AudioOnly => {
            if m.has_audio {
                Ok(())
            } else {
                Err(IncompatibilityReason::NoAudioStream)
            }
        }
        ConvertMode::Subtitle => {
            if m.has_subtitle {
                Ok(())
            } else {
                Err(IncompatibilityReason::NoSubtitleStream)
            }
        }
        ConvertMode::Remux => {
            if (m.has_video && !m.is_still_image) || m.has_audio {
                Ok(())
            } else {
                Err(IncompatibilityReason::NoRemuxableStream)
            }
        }
    }
}

/// Target output container, image, or subtitle format (53 formats across 4 categories).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerFormat {
    // Video (17)
    Mp4,
    Mkv,
    Mov,
    WebM,
    Avi,
    Flv,
    Ts,
    M2ts,
    Mxf,
    ThreeGp,
    M4v,
    Mpg,
    Mpeg,
    Vob,
    Ogv,
    Wmv,
    Dv,
    // Audio (18)
    Mp3,
    Wav,
    Flac,
    Aac,
    M4a,
    Ogg,
    Opus,
    Wma,
    Aiff,
    Ac3,
    Eac3,
    Dts,
    Alac,
    Amr,
    Caf,
    Mka,
    Mp2,
    Wv,
    // Image / Animated (13)
    Png,
    Jpg,
    Webp,
    Gif,
    Avif,
    Apng,
    Bmp,
    Tiff,
    Tga,
    Ico,
    Jxl,
    Exr,
    Hdr,
    // Subtitle (5)
    Srt,
    Ass,
    Ssa,
    Vtt,
    Lrc,
}

impl ContainerFormat {
    pub const ALL: &'static [Self] = &[
        // Video
        Self::Mp4,
        Self::Mkv,
        Self::Mov,
        Self::WebM,
        Self::Avi,
        Self::Flv,
        Self::Ts,
        Self::M2ts,
        Self::Mxf,
        Self::ThreeGp,
        Self::M4v,
        Self::Mpg,
        Self::Mpeg,
        Self::Vob,
        Self::Ogv,
        Self::Wmv,
        Self::Dv,
        // Audio
        Self::Mp3,
        Self::Wav,
        Self::Flac,
        Self::Aac,
        Self::M4a,
        Self::Ogg,
        Self::Opus,
        Self::Wma,
        Self::Aiff,
        Self::Ac3,
        Self::Eac3,
        Self::Dts,
        Self::Alac,
        Self::Amr,
        Self::Caf,
        Self::Mka,
        Self::Mp2,
        Self::Wv,
        // Image
        Self::Png,
        Self::Jpg,
        Self::Webp,
        Self::Gif,
        Self::Avif,
        Self::Apng,
        Self::Bmp,
        Self::Tiff,
        Self::Tga,
        Self::Ico,
        Self::Jxl,
        Self::Exr,
        Self::Hdr,
        // Subtitle
        Self::Srt,
        Self::Ass,
        Self::Ssa,
        Self::Vtt,
        Self::Lrc,
    ];

    pub const fn short_name(self) -> &'static str {
        match self {
            Self::Mp4 => "MP4",
            Self::Mkv => "MKV",
            Self::Mov => "MOV",
            Self::WebM => "WEBM",
            Self::Avi => "AVI",
            Self::Flv => "FLV",
            Self::Ts => "TS",
            Self::M2ts => "M2TS",
            Self::Mxf => "MXF",
            Self::ThreeGp => "3GP",
            Self::M4v => "M4V",
            Self::Mpg => "MPG",
            Self::Mpeg => "MPEG",
            Self::Vob => "VOB",
            Self::Ogv => "OGV",
            Self::Wmv => "WMV",
            Self::Dv => "DV",
            Self::Mp3 => "MP3",
            Self::Wav => "WAV",
            Self::Flac => "FLAC",
            Self::Aac => "AAC",
            Self::M4a => "M4A",
            Self::Ogg => "OGG",
            Self::Opus => "OPUS",
            Self::Wma => "WMA",
            Self::Aiff => "AIFF",
            Self::Ac3 => "AC3",
            Self::Eac3 => "EAC3",
            Self::Dts => "DTS",
            Self::Alac => "ALAC",
            Self::Amr => "AMR",
            Self::Caf => "CAF",
            Self::Mka => "MKA",
            Self::Mp2 => "MP2",
            Self::Wv => "WV",
            Self::Png => "PNG",
            Self::Jpg => "JPG",
            Self::Webp => "WEBP",
            Self::Gif => "GIF",
            Self::Avif => "AVIF",
            Self::Apng => "APNG",
            Self::Bmp => "BMP",
            Self::Tiff => "TIFF",
            Self::Tga => "TGA",
            Self::Ico => "ICO",
            Self::Jxl => "JXL",
            Self::Exr => "EXR",
            Self::Hdr => "HDR",
            Self::Srt => "SRT",
            Self::Ass => "ASS",
            Self::Ssa => "SSA",
            Self::Vtt => "VTT",
            Self::Lrc => "LRC",
        }
    }

    #[allow(dead_code)]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Mp4 => "MP4 (Universal)",
            Self::Mkv => "MKV (Matroska)",
            Self::Mov => "MOV (QuickTime)",
            Self::WebM => "WebM (VP9 / AV1)",
            Self::Avi => "AVI (Audio Video Interleave)",
            Self::Flv => "FLV (Flash Video)",
            Self::Ts => "TS (MPEG Transport Stream)",
            Self::M2ts => "M2TS (Blu-ray BDAV)",
            Self::Mxf => "MXF (Broadcast Exchange)",
            Self::ThreeGp => "3GP (Mobile Video)",
            Self::M4v => "M4V (Apple MPEG-4 Video)",
            Self::Mpg => "MPG (MPEG-2 Program)",
            Self::Mpeg => "MPEG (MPEG-2 Program)",
            Self::Vob => "VOB (DVD Video Object)",
            Self::Ogv => "OGV (Ogg Theora Video)",
            Self::Wmv => "WMV (Windows Media Video)",
            Self::Dv => "DV (Digital Video)",
            Self::Mp3 => "MP3 (MPEG Audio)",
            Self::Wav => "WAV (Uncompressed PCM)",
            Self::Flac => "FLAC (Lossless Audio)",
            Self::Aac => "AAC (ADTS Raw AAC)",
            Self::M4a => "M4A (AAC / Apple Audio)",
            Self::Ogg => "OGG (Ogg Vorbis)",
            Self::Opus => "Opus (Ogg Opus)",
            Self::Wma => "WMA (Windows Media Audio)",
            Self::Aiff => "AIFF (Apple Uncompressed)",
            Self::Ac3 => "AC3 (Dolby Digital)",
            Self::Eac3 => "EAC3 (Dolby Digital Plus)",
            Self::Dts => "DTS (Coherent Acoustics)",
            Self::Alac => "ALAC (Apple Lossless M4A)",
            Self::Amr => "AMR (Adaptive Multi-Rate)",
            Self::Caf => "CAF (Core Audio Format)",
            Self::Mka => "MKA (Matroska Audio)",
            Self::Mp2 => "MP2 (MPEG Layer II Audio)",
            Self::Wv => "WV (WavPack Lossless)",
            Self::Png => "PNG (Lossless Image)",
            Self::Jpg => "JPG (JPEG Image)",
            Self::Webp => "WebP (Static / Animated)",
            Self::Gif => "GIF (Animated Palette)",
            Self::Avif => "AVIF (AV1 Image)",
            Self::Apng => "APNG (Animated PNG)",
            Self::Bmp => "BMP (Windows Bitmap)",
            Self::Tiff => "TIFF (Tagged Image)",
            Self::Tga => "TGA (Truevision TARGA)",
            Self::Ico => "ICO (Windows Icon <=256px)",
            Self::Jxl => "JXL (JPEG XL Image)",
            Self::Exr => "EXR (OpenEXR HDR)",
            Self::Hdr => "HDR (Radiance RGBE)",
            Self::Srt => "SRT (SubRip Subtitle)",
            Self::Ass => "ASS (Advanced SubStation)",
            Self::Ssa => "SSA (SubStation Alpha)",
            Self::Vtt => "VTT (WebVTT Subtitle)",
            Self::Lrc => "LRC (Synchronized Lyrics)",
        }
    }

    pub const fn extension(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
            Self::Mov => "mov",
            Self::WebM => "webm",
            Self::Avi => "avi",
            Self::Flv => "flv",
            Self::Ts => "ts",
            Self::M2ts => "m2ts",
            Self::Mxf => "mxf",
            Self::ThreeGp => "3gp",
            Self::M4v => "m4v",
            Self::Mpg => "mpg",
            Self::Mpeg => "mpeg",
            Self::Vob => "vob",
            Self::Ogv => "ogv",
            Self::Wmv => "wmv",
            Self::Dv => "dv",
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
            Self::Flac => "flac",
            Self::Aac => "aac",
            Self::M4a => "m4a",
            Self::Ogg => "ogg",
            Self::Opus => "opus",
            Self::Wma => "wma",
            Self::Aiff => "aiff",
            Self::Ac3 => "ac3",
            Self::Eac3 => "eac3",
            Self::Dts => "dts",
            Self::Alac => "m4a",
            Self::Amr => "amr",
            Self::Caf => "caf",
            Self::Mka => "mka",
            Self::Mp2 => "mp2",
            Self::Wv => "wv",
            Self::Png => "png",
            Self::Jpg => "jpg",
            Self::Webp => "webp",
            Self::Gif => "gif",
            Self::Avif => "avif",
            Self::Apng => "apng",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
            Self::Tga => "tga",
            Self::Ico => "ico",
            Self::Jxl => "jxl",
            Self::Exr => "exr",
            Self::Hdr => "hdr",
            Self::Srt => "srt",
            Self::Ass => "ass",
            Self::Ssa => "ssa",
            Self::Vtt => "vtt",
            Self::Lrc => "lrc",
        }
    }

    pub const fn muxer_name(self) -> &'static str {
        match self {
            Self::Mp4 | Self::M4v => "mp4",
            Self::Mkv | Self::Mka => "matroska",
            Self::Mov => "mov",
            Self::WebM => "webm",
            Self::Avi => "avi",
            Self::Flv => "flv",
            Self::Ts | Self::M2ts => "mpegts",
            Self::Mxf => "mxf",
            Self::ThreeGp => "3gp",
            Self::Mpg | Self::Mpeg => "mpeg",
            Self::Vob => "vob",
            Self::Ogv | Self::Ogg => "ogg",
            Self::Wmv | Self::Wma => "asf",
            Self::Dv => "dv",
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
            Self::Flac => "flac",
            Self::Aac => "adts",
            Self::M4a | Self::Alac => "ipod",
            Self::Opus => "opus",
            Self::Aiff => "aiff",
            Self::Ac3 => "ac3",
            Self::Eac3 => "eac3",
            Self::Dts => "dts",
            Self::Amr => "amr",
            Self::Caf => "caf",
            Self::Mp2 => "mp2",
            Self::Wv => "wv",
            Self::Gif => "gif",
            Self::Webp => "webp",
            Self::Apng => "apng",
            Self::Ico => "ico",
            Self::Avif => "avif",
            Self::Png
            | Self::Jpg
            | Self::Bmp
            | Self::Tiff
            | Self::Tga
            | Self::Jxl
            | Self::Exr
            | Self::Hdr => "image2",
            Self::Srt => "srt",
            Self::Ass | Self::Ssa => "ass",
            Self::Vtt => "webvtt",
            Self::Lrc => "lrc",
        }
    }

    pub const fn default_mode(self) -> ConvertMode {
        match self {
            Self::Mp4
            | Self::Mkv
            | Self::Mov
            | Self::WebM
            | Self::Avi
            | Self::Flv
            | Self::Ts
            | Self::M2ts
            | Self::Mxf
            | Self::ThreeGp
            | Self::M4v
            | Self::Mpg
            | Self::Mpeg
            | Self::Vob
            | Self::Ogv
            | Self::Wmv
            | Self::Dv => ConvertMode::Video,
            Self::Mp3
            | Self::Wav
            | Self::Flac
            | Self::Aac
            | Self::M4a
            | Self::Ogg
            | Self::Opus
            | Self::Wma
            | Self::Aiff
            | Self::Ac3
            | Self::Eac3
            | Self::Dts
            | Self::Alac
            | Self::Amr
            | Self::Caf
            | Self::Mka
            | Self::Mp2
            | Self::Wv => ConvertMode::AudioOnly,
            Self::Png
            | Self::Jpg
            | Self::Webp
            | Self::Gif
            | Self::Avif
            | Self::Apng
            | Self::Bmp
            | Self::Tiff
            | Self::Tga
            | Self::Ico
            | Self::Jxl
            | Self::Exr
            | Self::Hdr => ConvertMode::ImageGif,
            Self::Srt | Self::Ass | Self::Ssa | Self::Vtt | Self::Lrc => ConvertMode::Subtitle,
        }
    }

    pub const fn is_image_or_gif(self) -> bool {
        matches!(self.default_mode(), ConvertMode::ImageGif)
    }

    pub const fn is_subtitle(self) -> bool {
        matches!(self.default_mode(), ConvertMode::Subtitle)
    }

    pub const fn supports_faststart(self) -> bool {
        matches!(
            self,
            Self::Mp4 | Self::Mov | Self::M4a | Self::M4v | Self::Alac | Self::ThreeGp
        )
    }

    pub const fn subtitle_codec_name(self) -> &'static str {
        match self {
            Self::Srt => "srt",
            Self::Ass => "ass",
            Self::Ssa => "ssa",
            Self::Vtt => "webvtt",
            Self::Lrc => "subrip",
            _ => "srt",
        }
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
    Mpeg2Video,
    LibTheora,
    Wmv2,
    DvVideo,
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
        Self::Mpeg2Video,
        Self::LibTheora,
        Self::Wmv2,
        Self::DvVideo,
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
            Self::Mpeg2Video => "mpeg2video",
            Self::LibTheora => "libtheora",
            Self::Wmv2 => "wmv2",
            Self::DvVideo => "dvvideo",
            Self::H264Nvenc => "h264_nvenc",
            Self::HevcNvenc => "hevc_nvenc",
            Self::Av1Nvenc => "av1_nvenc",
            Self::H264Amf => "h264_amf",
            Self::HevcAmf => "hevc_amf",
            Self::H264Qsv => "h264_qsv",
            Self::HevcQsv => "hevc_qsv",
        }
    }

    #[allow(dead_code)]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Copy => "Stream Copy (No Re-encode)",
            Self::Libx264 => "H.264 / AVC (libx264)",
            Self::Libx265 => "H.265 / HEVC (libx265)",
            Self::LibSvtAv1 => "AV1 (libsvtav1)",
            Self::LibVpxVp9 => "VP9 (libvpx-vp9)",
            Self::Mpeg2Video => "MPEG-2 Video (mpeg2video)",
            Self::LibTheora => "Theora (libtheora)",
            Self::Wmv2 => "Windows Media Video 8 (wmv2)",
            Self::DvVideo => "DV Video (dvvideo)",
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

    pub const fn is_h264(self) -> bool {
        matches!(
            self,
            Self::Libx264 | Self::H264Nvenc | Self::H264Amf | Self::H264Qsv
        )
    }

    pub const fn is_compatible_with(self, container: ContainerFormat) -> bool {
        match container {
            ContainerFormat::Mkv => true,
            ContainerFormat::Mp4 | ContainerFormat::Mov | ContainerFormat::M4v => {
                self.is_h264()
                    || self.is_hevc()
                    || matches!(self, Self::LibSvtAv1 | Self::Av1Nvenc | Self::Mpeg2Video | Self::Copy)
            }
            ContainerFormat::WebM => {
                matches!(self, Self::LibVpxVp9 | Self::LibSvtAv1 | Self::Av1Nvenc | Self::Copy)
            }
            ContainerFormat::Avi => {
                self.is_h264() || matches!(self, Self::Mpeg2Video | Self::Wmv2 | Self::Copy)
            }
            ContainerFormat::Flv => self.is_h264() || matches!(self, Self::Copy),
            ContainerFormat::Ts | ContainerFormat::M2ts => {
                self.is_h264() || self.is_hevc() || matches!(self, Self::Mpeg2Video | Self::Copy)
            }
            ContainerFormat::Mxf => matches!(self, Self::Mpeg2Video | Self::Copy) || self.is_h264(),
            ContainerFormat::ThreeGp => self.is_h264() || matches!(self, Self::Copy),
            ContainerFormat::Mpg | ContainerFormat::Mpeg | ContainerFormat::Vob => {
                matches!(self, Self::Mpeg2Video | Self::Copy)
            }
            ContainerFormat::Ogv => matches!(self, Self::LibTheora | Self::Copy),
            ContainerFormat::Wmv => matches!(self, Self::Wmv2 | Self::Copy),
            ContainerFormat::Dv => matches!(self, Self::DvVideo),
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
    Mp2,
    Wmav2,
    Ac3,
    Eac3,
    Dts,
    Alac,
    AmrNb,
    WavPack,
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
        Self::Mp2,
        Self::Wmav2,
        Self::Ac3,
        Self::Eac3,
        Self::Dts,
        Self::Alac,
        Self::AmrNb,
        Self::WavPack,
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
            Self::Mp2 => "mp2",
            Self::Wmav2 => "wmav2",
            Self::Ac3 => "ac3",
            Self::Eac3 => "eac3",
            Self::Dts => "dca",
            Self::Alac => "alac",
            Self::AmrNb => "libopencore_amrnb",
            Self::WavPack => "wavpack",
            Self::Copy => "copy",
            Self::None => "none",
        }
    }

    #[allow(dead_code)]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Aac => "AAC (Universal)",
            Self::LibOpus => "Opus (High Efficiency)",
            Self::LibMp3Lame => "MP3 (LAME)",
            Self::LibVorbis => "Vorbis (OGG)",
            Self::Flac => "FLAC (Lossless)",
            Self::PcmS16le => "PCM 16-bit (Uncompressed)",
            Self::Mp2 => "MP2 (MPEG Audio Layer II)",
            Self::Wmav2 => "WMA v2 (Windows Media)",
            Self::Ac3 => "AC-3 (Dolby Digital)",
            Self::Eac3 => "E-AC-3 (Dolby Digital Plus)",
            Self::Dts => "DTS (dca)",
            Self::Alac => "ALAC (Apple Lossless)",
            Self::AmrNb => "AMR-NB (8kHz Mono)",
            Self::WavPack => "WavPack (Lossless)",
            Self::Copy => "Stream Copy (No Re-encode)",
            Self::None => "Mute (Strip Audio -an)",
        }
    }

    pub const fn is_lossless(self) -> bool {
        matches!(
            self,
            Self::Flac
                | Self::PcmS16le
                | Self::Alac
                | Self::WavPack
                | Self::Copy
                | Self::None
        )
    }

    pub const fn is_compatible_with(self, container: ContainerFormat) -> bool {
        match container {
            ContainerFormat::Mp4 | ContainerFormat::M4v => matches!(
                self,
                Self::Aac
                    | Self::LibMp3Lame
                    | Self::LibOpus
                    | Self::Ac3
                    | Self::Eac3
                    | Self::Alac
                    | Self::Copy
                    | Self::None
            ),
            ContainerFormat::Mov => matches!(
                self,
                Self::Aac
                    | Self::LibMp3Lame
                    | Self::PcmS16le
                    | Self::Alac
                    | Self::Ac3
                    | Self::Copy
                    | Self::None
            ),
            ContainerFormat::Mkv => true,
            ContainerFormat::Mka => !matches!(self, Self::None),
            ContainerFormat::WebM => matches!(self, Self::LibOpus | Self::LibVorbis | Self::Copy | Self::None),
            ContainerFormat::Avi => matches!(
                self,
                Self::Aac | Self::LibMp3Lame | Self::PcmS16le | Self::Ac3 | Self::Mp2 | Self::Copy | Self::None
            ),
            ContainerFormat::Flv => matches!(self, Self::Aac | Self::LibMp3Lame | Self::Copy | Self::None),
            ContainerFormat::Ts | ContainerFormat::M2ts => matches!(
                self,
                Self::Aac | Self::Mp2 | Self::Ac3 | Self::Eac3 | Self::LibMp3Lame | Self::Copy | Self::None
            ),
            ContainerFormat::Mxf => matches!(self, Self::PcmS16le | Self::Copy | Self::None),
            ContainerFormat::ThreeGp => matches!(self, Self::Aac | Self::AmrNb | Self::Copy | Self::None),
            ContainerFormat::Mpg | ContainerFormat::Mpeg | ContainerFormat::Vob => {
                matches!(self, Self::Mp2 | Self::Ac3 | Self::Copy | Self::None)
            }
            ContainerFormat::Ogv => matches!(self, Self::LibVorbis | Self::LibOpus | Self::Copy | Self::None),
            ContainerFormat::Wmv => matches!(self, Self::Wmav2 | Self::Copy | Self::None),
            ContainerFormat::Dv => matches!(self, Self::PcmS16le),
            ContainerFormat::Mp3 => matches!(self, Self::LibMp3Lame | Self::Copy),
            ContainerFormat::Wav | ContainerFormat::Aiff => matches!(self, Self::PcmS16le),
            ContainerFormat::Flac => matches!(self, Self::Flac | Self::Copy),
            ContainerFormat::Aac => matches!(self, Self::Aac | Self::Copy),
            ContainerFormat::M4a => matches!(self, Self::Aac | Self::Alac | Self::Copy),
            ContainerFormat::Alac => matches!(self, Self::Alac),
            ContainerFormat::Ogg => matches!(self, Self::LibVorbis | Self::LibOpus | Self::Flac | Self::Copy),
            ContainerFormat::Opus => matches!(self, Self::LibOpus | Self::Copy),
            ContainerFormat::Wma => matches!(self, Self::Wmav2 | Self::Copy),
            ContainerFormat::Ac3 => matches!(self, Self::Ac3 | Self::Copy),
            ContainerFormat::Eac3 => matches!(self, Self::Eac3 | Self::Copy),
            ContainerFormat::Dts => matches!(self, Self::Dts | Self::Copy),
            ContainerFormat::Amr => matches!(self, Self::AmrNb),
            ContainerFormat::Caf => matches!(self, Self::PcmS16le | Self::Alac | Self::Aac | Self::LibOpus),
            ContainerFormat::Mp2 => matches!(self, Self::Mp2 | Self::Copy),
            ContainerFormat::Wv => matches!(self, Self::WavPack | Self::Copy),
            c if c.is_image_or_gif() || c.is_subtitle() => matches!(self, Self::None),
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
    pub image_to_video_duration_secs: u32,
    pub safety: SafetySettings,
    pub output_prefix: String,
    pub output_suffix: String,
    pub custom_output_dir: Option<std::path::PathBuf>,
    #[serde(skip)]
    pub format_picker_open: bool,
    #[serde(skip)]
    pub format_search_query: String,
}

impl Default for ConversionSettings {
    fn default() -> Self {
        Self::from_preset(PresetProfile::Mp4UniversalH264)
    }
}

/// Curated one-click presets spanning conversion modes.
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

    #[allow(dead_code)]
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
            image_to_video_duration_secs: 5,
            safety: SafetySettings::default(),
            output_prefix: String::new(),
            output_suffix: String::new(),
            custom_output_dir: None,
            format_picker_open: false,
            format_search_query: String::new(),
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
                        ContainerFormat::Mpg | ContainerFormat::Mpeg | ContainerFormat::Vob => {
                            VideoCodec::Mpeg2Video
                        }
                        ContainerFormat::Mxf => VideoCodec::Mpeg2Video,
                        ContainerFormat::Ogv => VideoCodec::LibTheora,
                        ContainerFormat::Wmv => VideoCodec::Wmv2,
                        ContainerFormat::Dv => VideoCodec::DvVideo,
                        _ => VideoCodec::Libx264,
                    };
                }
                if !self.audio_codec.is_compatible_with(self.container) {
                    self.audio_codec = match self.container {
                        ContainerFormat::WebM | ContainerFormat::Ogv => AudioCodec::LibOpus,
                        ContainerFormat::Mpg | ContainerFormat::Mpeg | ContainerFormat::Vob => {
                            AudioCodec::Mp2
                        }
                        ContainerFormat::Mxf | ContainerFormat::Dv => AudioCodec::PcmS16le,
                        ContainerFormat::Wmv => AudioCodec::Wmav2,
                        _ => AudioCodec::Aac,
                    };
                }
            }
            ConvertMode::AudioOnly => {
                if !self.audio_codec.is_compatible_with(self.container) {
                    self.audio_codec = match self.container {
                        ContainerFormat::Mp3 => AudioCodec::LibMp3Lame,
                        ContainerFormat::Wav | ContainerFormat::Aiff => AudioCodec::PcmS16le,
                        ContainerFormat::Flac => AudioCodec::Flac,
                        ContainerFormat::Aac | ContainerFormat::M4a | ContainerFormat::Mka => {
                            AudioCodec::Aac
                        }
                        ContainerFormat::Ogg => AudioCodec::LibVorbis,
                        ContainerFormat::Opus => AudioCodec::LibOpus,
                        ContainerFormat::Wma => AudioCodec::Wmav2,
                        ContainerFormat::Ac3 => AudioCodec::Ac3,
                        ContainerFormat::Eac3 => AudioCodec::Eac3,
                        ContainerFormat::Dts => AudioCodec::Dts,
                        ContainerFormat::Alac | ContainerFormat::Caf => AudioCodec::Alac,
                        ContainerFormat::Amr => AudioCodec::AmrNb,
                        ContainerFormat::Mp2 => AudioCodec::Mp2,
                        ContainerFormat::Wv => AudioCodec::WavPack,
                        _ => AudioCodec::LibMp3Lame,
                    };
                }
            }
            ConvertMode::ImageGif | ConvertMode::Subtitle => {
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
        // AMR-NB requires mono 8kHz; normalize defaults if selected.
        if self.audio_codec == AudioCodec::AmrNb {
            self.audio_channels = AudioChannels::Mono;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_compatibility_gating_matches_media_capabilities() {
        let video_only = MediaInfo {
            has_video: true,
            has_audio: false,
            has_subtitle: false,
            is_still_image: false,
            ..Default::default()
        };
        assert!(check_mode_compatibility(ConvertMode::Video, Some(&video_only)).is_ok());
        assert!(check_mode_compatibility(ConvertMode::ImageGif, Some(&video_only)).is_ok());
        assert!(check_mode_compatibility(ConvertMode::Remux, Some(&video_only)).is_ok());
        assert_eq!(
            check_mode_compatibility(ConvertMode::AudioOnly, Some(&video_only)),
            Err(IncompatibilityReason::NoAudioStream)
        );
        assert_eq!(
            check_mode_compatibility(ConvertMode::Subtitle, Some(&video_only)),
            Err(IncompatibilityReason::NoSubtitleStream)
        );

        let audio_only = MediaInfo {
            has_video: false,
            has_audio: true,
            has_subtitle: false,
            is_still_image: false,
            ..Default::default()
        };
        assert!(check_mode_compatibility(ConvertMode::Video, Some(&audio_only)).is_ok());
        assert!(check_mode_compatibility(ConvertMode::AudioOnly, Some(&audio_only)).is_ok());
        assert!(check_mode_compatibility(ConvertMode::ImageGif, Some(&audio_only)).is_ok());
        assert!(check_mode_compatibility(ConvertMode::Remux, Some(&audio_only)).is_ok());
        assert_eq!(
            check_mode_compatibility(ConvertMode::Subtitle, Some(&audio_only)),
            Err(IncompatibilityReason::NoSubtitleStream)
        );

        let still_img = MediaInfo {
            has_video: true,
            has_audio: false,
            has_subtitle: false,
            is_still_image: true,
            ..Default::default()
        };
        assert!(check_mode_compatibility(ConvertMode::Video, Some(&still_img)).is_ok());
        assert!(check_mode_compatibility(ConvertMode::ImageGif, Some(&still_img)).is_ok());
        assert_eq!(
            check_mode_compatibility(ConvertMode::Remux, Some(&still_img)),
            Err(IncompatibilityReason::NoRemuxableStream)
        );

        let sub_only = MediaInfo {
            has_video: false,
            has_audio: false,
            has_subtitle: true,
            ..Default::default()
        };
        assert!(check_mode_compatibility(ConvertMode::Subtitle, Some(&sub_only)).is_ok());
        assert_eq!(
            check_mode_compatibility(ConvertMode::Video, Some(&sub_only)),
            Err(IncompatibilityReason::NoVisualOrAudioStream)
        );
    }

    #[test]
    fn every_output_format_maps_to_valid_default_codecs() {
        for &c in ContainerFormat::ALL {
            let mut s = ConversionSettings::default();
            s.set_mode(c.default_mode());
            s.container = c;
            s.enforce_compatibility();
            if c.default_mode() == ConvertMode::Video {
                assert!(s.video_codec.is_compatible_with(c), "vcodec for {c:?}");
                assert!(s.audio_codec.is_compatible_with(c), "acodec for {c:?}");
            } else if c.default_mode() == ConvertMode::AudioOnly {
                assert!(s.audio_codec.is_compatible_with(c), "acodec for {c:?}");
            }
        }
    }
}
