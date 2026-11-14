use crate::ffmpeg::command_builder::build_command_plan;
use crate::ffmpeg::discovery::configure_background_command;
use crate::ffmpeg::validator::validate_output_file;
use crate::models::config::ConversionSettings;
use crate::models::media::{LocalFfmpegInfo, MediaInfo, ProgressStats, WorkerEvent};
use crate::security::path_guard::{
    build_safe_output_path, build_temp_staging_path, commit_temp_file,
};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread;

#[derive(Debug, Clone)]
pub struct BatchJobItem {
    pub file_id: u64,
    pub input_path: PathBuf,
    pub custom_stem: String,
    pub media: Option<MediaInfo>,
}

/// Spawns a background worker executing the batch conversion safely with atomic staging.
pub fn spawn_batch_conversion(
    ffmpeg_info: LocalFfmpegInfo,
    items: Vec<BatchJobItem>,
    settings: ConversionSettings,
    cancel_flag: Arc<AtomicBool>,
    tx: Sender<WorkerEvent>,
    ctx: eframe::egui::Context,
) {
    thread::spawn(move || {
        let Some(ffmpeg_path) = ffmpeg_info.ffmpeg_path.clone() else {
            let _ = tx.send(WorkerEvent::LogLine(
                "Error: ffmpeg binary not found.".to_string(),
            ));
            let _ = tx.send(WorkerEvent::BatchFinished);
            ctx.request_repaint();
            return;
        };
        let Some(ffprobe_path) = ffmpeg_info.ffprobe_path.clone() else {
            let _ = tx.send(WorkerEvent::LogLine(
                "Error: ffprobe binary not found.".to_string(),
            ));
            let _ = tx.send(WorkerEvent::BatchFinished);
            ctx.request_repaint();
            return;
        };

        for item in items {
            if cancel_flag.load(Ordering::Relaxed) {
                let _ = tx.send(WorkerEvent::FileCancelled {
                    file_id: item.file_id,
                });
                ctx.request_repaint();
                continue;
            }

            let _ = tx.send(WorkerEvent::FileStarted {
                file_id: item.file_id,
            });
            ctx.request_repaint();

            let result = run_single_item(
                &ffmpeg_path,
                &ffprobe_path,
                &item,
                &settings,
                &cancel_flag,
                &tx,
                &ctx,
            );

            match result {
                Ok((final_output, report)) => {
                    let _ = tx.send(WorkerEvent::FileCompleted {
                        file_id: item.file_id,
                        output_path: final_output,
                        report,
                    });
                }
                Err(SingleItemError::Cancelled) => {
                    let _ = tx.send(WorkerEvent::FileCancelled {
                        file_id: item.file_id,
                    });
                }
                Err(SingleItemError::Failed(msg)) => {
                    let _ = tx.send(WorkerEvent::LogLine(format!(
                        "[Error] {}: {msg}",
                        item.input_path.display()
                    )));
                    let _ = tx.send(WorkerEvent::FileFailed {
                        file_id: item.file_id,
                        error: msg,
                    });
                }
            }
            ctx.request_repaint();
        }

        let _ = tx.send(WorkerEvent::BatchFinished);
        ctx.request_repaint();
    });
}

enum SingleItemError {
    Cancelled,
    Failed(String),
}

fn run_single_item(
    ffmpeg_path: &PathBuf,
    ffprobe_path: &PathBuf,
    item: &BatchJobItem,
    settings: &ConversionSettings,
    cancel_flag: &Arc<AtomicBool>,
    tx: &Sender<WorkerEvent>,
    ctx: &eframe::egui::Context,
) -> Result<(PathBuf, crate::models::media::ValidationReport), SingleItemError> {
    let mut normalized = settings.clone();
    normalized.enforce_compatibility();

    let final_output = build_safe_output_path(
        &item.input_path,
        Some(item.custom_stem.as_str()),
        &normalized.output_prefix,
        &normalized.output_suffix,
        normalized.custom_output_dir.as_deref(),
        normalized.container.extension(),
        normalized.safety.overwrite_existing,
    )
    .map_err(SingleItemError::Failed)?;

    let temp_output =
        build_temp_staging_path(&final_output, item.file_id).map_err(SingleItemError::Failed)?;

    let _ = fs::remove_file(&temp_output);

    let plan = build_command_plan(
        &normalized,
        item.media.as_ref(),
        &item.input_path,
        &temp_output,
    )
    .map_err(SingleItemError::Failed)?;

    for note in &plan.notes {
        let _ = tx.send(WorkerEvent::LogLine(format!("[Note] {note}")));
    }
    let _ = tx.send(WorkerEvent::LogLine(format!(
        "[Run] {}",
        plan.display_preview()
    )));

    let mut cmd = Command::new(ffmpeg_path);
    cmd.args(&plan.args);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    configure_background_command(&mut cmd);

    let mut child = cmd
        .spawn()
        .map_err(|e| SingleItemError::Failed(format!("Failed to start ffmpeg: {e}")))?;

    let stderr_handle = child.stderr.take().map(|stderr| {
        let tx_err = tx.clone();
        let ctx_err = ctx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            let mut tail: Vec<String> = Vec::new();
            for line in reader.lines().map_while(Result::ok) {
                let trimmed = line.trim().to_string();
                if !trimmed.is_empty() {
                    let _ = tx_err.send(WorkerEvent::LogLine(trimmed.clone()));
                    ctx_err.request_repaint();
                    if tail.len() >= 15 {
                        tail.remove(0);
                    }
                    tail.push(trimmed);
                }
            }
            tail
        })
    });

    let duration_secs = item.media.as_ref().and_then(|m| m.duration_secs);
    let was_cancelled = stream_progress(&mut child, duration_secs, item.file_id, cancel_flag, tx, ctx);

    let status = child
        .wait()
        .map_err(|e| SingleItemError::Failed(format!("Failed waiting for ffmpeg: {e}")))?;

    let err_tail = stderr_handle
        .and_then(|h| h.join().ok())
        .unwrap_or_default();

    if was_cancelled || cancel_flag.load(Ordering::Relaxed) {
        let _ = fs::remove_file(&temp_output);
        return Err(SingleItemError::Cancelled);
    }

    if !status.success() {
        let _ = fs::remove_file(&temp_output);
        let detail = if err_tail.is_empty() {
            format!("ffmpeg exited with status {status}")
        } else {
            format!(
                "ffmpeg exited with status {status}: {}",
                err_tail.last().cloned().unwrap_or_default()
            )
        };
        return Err(SingleItemError::Failed(detail));
    }

    let _ = tx.send(WorkerEvent::FileVerifying {
        file_id: item.file_id,
    });
    let _ = tx.send(WorkerEvent::LogLine(format!(
        "[Verify] Checking {} integrity for {}...",
        normalized.mode.label(),
        final_output
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    )));
    ctx.request_repaint();

    let report = match validate_output_file(
        ffmpeg_path,
        ffprobe_path,
        &temp_output,
        &normalized,
        item.media.as_ref(),
    ) {
        Ok(rep) => rep,
        Err(err) => {
            let _ = fs::remove_file(&temp_output);
            return Err(SingleItemError::Failed(err));
        }
    };

    if let Err(err) = commit_temp_file(
        &temp_output,
        &final_output,
        &item.input_path,
        normalized.safety.overwrite_existing,
    ) {
        let _ = fs::remove_file(&temp_output);
        return Err(SingleItemError::Failed(err));
    }

    let _ = tx.send(WorkerEvent::LogLine(format!(
        "[Verified OK] Saved {}",
        final_output.display()
    )));

    Ok((final_output, report))
}

fn stream_progress(
    child: &mut Child,
    duration_secs: Option<f64>,
    file_id: u64,
    cancel_flag: &Arc<AtomicBool>,
    tx: &Sender<WorkerEvent>,
    ctx: &eframe::egui::Context,
) -> bool {
    let Some(stdout) = child.stdout.take() else {
        return false;
    };

    let reader = BufReader::new(stdout);
    let mut current = ProgressStats::default();

    for line in reader.lines().map_while(Result::ok) {
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = child.kill();
            return true;
        }

        if let Some((key, val)) = line.split_once('=') {
            let k = key.trim();
            let v = val.trim();
            match k {
                "out_time_us" => {
                    if let Ok(us) = v.parse::<u64>() {
                        current.out_time_secs = us as f64 / 1_000_000.0;
                        if let Some(total) = duration_secs
                            && total > 0.0
                        {
                            current.fraction =
                                (current.out_time_secs / total).clamp(0.0, 0.999) as f32;
                        }
                    }
                }
                "speed" if v != "N/A" => current.speed = Some(v.to_string()),
                "fps" if v != "N/A" => current.fps = Some(v.to_string()),
                "bitrate" if v != "N/A" => current.bitrate = Some(v.to_string()),
                "progress" => {
                    if v == "end" {
                        current.fraction = 1.0;
                    }
                    let _ = tx.send(WorkerEvent::FileProgress {
                        file_id,
                        stats: current.clone(),
                    });
                    ctx.request_repaint();
                }
                _ => {}
            }
        }
    }

    if cancel_flag.load(Ordering::Relaxed) {
        let _ = child.kill();
        return true;
    }

    false
}
