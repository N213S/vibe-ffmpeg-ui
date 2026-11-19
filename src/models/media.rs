use std::path::PathBuf;

/// Parsed media stream information from `ffprobe`.
#[derive(Debug, Clone, Default)]
pub struct MediaInfo {
    pub format_name: String,
    pub duration_secs: Option<f64>,
    pub file_size_bytes: u64,
    pub bit_rate: Option<u64>,
    pub has_video: bool,
    pub has_audio: bool,
    pub has_subtitle: bool,
    pub has_text_subtitles: bool,
    pub is_still_image: bool,
    pub video_codec: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub pixel_format: Option<String>,
    pub fps: Option<f64>,
    pub audio_codec: Option<String>,
    pub audio_sample_rate: Option<u32>,
    pub audio_channels: Option<u32>,
    pub subtitle_codec: Option<String>,
}

impl MediaInfo {
    /// Human-readable one-line summary for queue cards.
    #[allow(dead_code)]
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if let (Some(w), Some(h)) = (self.width, self.height) {
            let vcodec = self.video_codec.as_deref().unwrap_or("video");
            if let Some(fps) = self.fps {
                parts.push(format!("{w}x{h} @ {fps:.2}fps ({vcodec})"));
            } else {
                parts.push(format!("{w}x{h} ({vcodec})"));
            }
        } else if let Some(ref vcodec) = self.video_codec {
            parts.push(format!("Video: {vcodec}"));
        }

        if let Some(ref acodec) = self.audio_codec {
            let ch = self
                .audio_channels
                .map(|c| format!(" {c}ch"))
                .unwrap_or_default();
            parts.push(format!("Audio: {acodec}{ch}"));
        }

        if let Some(d) = self.duration_secs {
            parts.push(format_duration(d));
        }

        if parts.is_empty() {
            "Unprobed media".to_string()
        } else {
            parts.join("  |  ")
        }
    }
}

/// Real-time progress snapshot emitted by `-progress pipe:1`.
#[derive(Debug, Clone, Default)]
pub struct ProgressStats {
    pub fraction: f32,
    pub out_time_secs: f64,
    pub speed: Option<String>,
    pub fps: Option<String>,
    pub bitrate: Option<String>,
}

/// Post-conversion verification result.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub size_bytes: u64,
    pub duration_secs: Option<f64>,
    pub stream_count: usize,
    pub deep_decode_passed: bool,
    pub verification_note: String,
}

/// Execution state of a queued media item.
#[derive(Debug, Clone)]
pub enum FileStatus {
    Probing,
    Ready,
    Converting(ProgressStats),
    Verifying,
    Done {
        output_path: PathBuf,
        report: ValidationReport,
    },
    Failed(String),
    Cancelled,
}

/// A single item in the batch conversion queue.
#[derive(Debug, Clone)]
pub struct QueuedFile {
    pub id: u64,
    pub input_path: PathBuf,
    /// Editable output base name (extension is synced automatically from the target container).
    pub custom_stem: String,
    pub media: Option<MediaInfo>,
    pub probe_error: Option<String>,
    pub status: FileStatus,
}

/// Local FFmpeg installation capabilities discovered at startup.
#[derive(Debug, Clone, Default)]
pub struct LocalFfmpegInfo {
    pub ffmpeg_path: Option<PathBuf>,
    pub ffprobe_path: Option<PathBuf>,
    pub raw_version_line: String,
    pub semantic_version: Option<String>,
    pub available_encoders: Vec<String>,
}

impl LocalFfmpegInfo {
    pub fn is_ready(&self) -> bool {
        self.ffmpeg_path.is_some() && self.ffprobe_path.is_some()
    }

    pub fn supports_encoder(&self, name: &str) -> bool {
        name == "copy" || self.available_encoders.iter().any(|e| e == name)
    }
}

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result of querying official GitHub releases for Vibe FFmpeg itself.
#[derive(Debug, Clone, Default)]
pub enum AppUpdateState {
    #[default]
    Idle,
    Checking,
    Completed(AppUpdateInfo),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct AppUpdateInfo {
    pub latest_version: String,
    pub current_version: String,
    pub release_url: String,
    pub update_available: bool,
}

/// Result of querying official Gyan.dev FFmpeg release endpoints.
#[derive(Debug, Clone)]
pub enum UpdateCheckState {
    Idle,
    Checking,
    Completed(UpdateCheckInfo),
    Error(String),
}

/// Status of automatic WinGet FFmpeg installation triggered from the setup window.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum FfmpegInstallState {
    #[default]
    Idle,
    Installing {
        latest_line: String,
    },
    Succeeded,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct UpdateCheckInfo {
    pub latest_release: String,
    pub latest_git: Option<String>,
    pub local_version: Option<String>,
    pub update_available: bool,
}

/// Asynchronous events posted from background threads to the UI.
#[derive(Debug, Clone)]
pub enum WorkerEvent {
    ProbeCompleted {
        file_id: u64,
        result: Result<MediaInfo, String>,
    },
    FileStarted {
        file_id: u64,
    },
    FileProgress {
        file_id: u64,
        stats: ProgressStats,
    },
    FileVerifying {
        file_id: u64,
    },
    FileCompleted {
        file_id: u64,
        output_path: PathBuf,
        report: ValidationReport,
    },
    FileFailed {
        file_id: u64,
        error: String,
    },
    FileCancelled {
        file_id: u64,
    },
    LogLine(String),
    BatchFinished,
    UpdateChecked(Result<UpdateCheckInfo, String>),
    AppUpdateChecked(Result<AppUpdateInfo, String>),
    FfmpegInstallProgress(String),
    FfmpegInstallFinished(Result<(), String>),
    FfmpegRediscovered(LocalFfmpegInfo),
}

/// Formats seconds as `HH:MM:SS` or `MM:SS`.
pub fn format_duration(secs: f64) -> String {
    if !secs.is_finite() || secs < 0.0 {
        return "00:00".to_string();
    }
    let total = secs.round() as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h:02}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

/// Formats bytes into human-readable MB/GB units.
pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.2} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}
