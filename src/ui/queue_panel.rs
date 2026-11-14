use crate::models::config::ConversionSettings;
use crate::models::media::{FileStatus, QueuedFile, format_bytes, format_duration};
use crate::security::path_guard::compose_output_filename;
use crate::ui::theme::{
    ACCENT_AMBER, ACCENT_BLUE, ACCENT_GREEN, ACCENT_RED, TEXT_MUTED, card_frame, status_pill,
};
use eframe::egui::{self, ProgressBar, RichText, TextEdit};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Default)]
pub struct QueueActions {
    pub remove_file_id: Option<u64>,
    pub add_files_clicked: bool,
}

/// Renders the central file queue with editable output filenames, auto-synced extension, and mode-aware verification status.
pub fn render_queue_panel(
    ui: &mut egui::Ui,
    queue: &mut [QueuedFile],
    settings: &ConversionSettings,
    is_running: bool,
    is_hovering_files: bool,
) -> QueueActions {
    let mut actions = QueueActions::default();

    if is_hovering_files {
        status_pill(
            ui,
            "Drop video, audio, or image files anywhere to add to queue",
            ACCENT_BLUE,
        );
        ui.add_space(6.0);
    }

    if queue.is_empty() {
        ui.centered_and_justified(|ui| {
            card_frame().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(32.0);
                    ui.label(
                        RichText::new("Drag & Drop Video, Audio, or Image Files Here")
                            .size(18.0)
                            .strong(),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(
                            "Supports Video, Audio Extraction, Image/GIF, and Fast Remux.\nFiles are probed, staged atomically (.tmp.<ext>), and verified cleanly.",
                        )
                        .color(TEXT_MUTED),
                    );
                    ui.add_space(16.0);
                    if ui.button("+ Browse Media Files").clicked() {
                        actions.add_files_clicked = true;
                    }
                    ui.add_space(32.0);
                });
            });
        });
        return actions;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for item in queue.iter_mut() {
                render_file_card(ui, item, settings, is_running, &mut actions);
                ui.add_space(6.0);
            }
        });

    actions
}

fn render_file_card(
    ui: &mut egui::Ui,
    item: &mut QueuedFile,
    settings: &ConversionSettings,
    is_running: bool,
    actions: &mut QueueActions,
) {
    let file_name = item
        .input_path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| item.input_path.display().to_string());

    card_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(&file_name).size(14.5).strong());
                if let Some(ref media) = item.media {
                    let size_txt = if media.file_size_bytes > 0 {
                        format!("  |  {}", format_bytes(media.file_size_bytes))
                    } else {
                        String::new()
                    };
                    ui.label(
                        RichText::new(format!("{}{}", media.summary(), size_txt))
                            .size(12.0)
                            .color(TEXT_MUTED),
                    );
                } else if let Some(ref err) = item.probe_error {
                    ui.label(
                        RichText::new(format!("Probe error: {err}"))
                            .size(12.0)
                            .color(ACCENT_RED),
                    );
                } else {
                    ui.label(
                        RichText::new("Inspecting media streams...")
                            .size(12.0)
                            .color(TEXT_MUTED),
                    );
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let removable = !is_running
                    || matches!(
                        item.status,
                        FileStatus::Ready
                            | FileStatus::Done { .. }
                            | FileStatus::Failed(_)
                            | FileStatus::Cancelled
                    );
                if removable && ui.small_button("Remove").clicked() {
                    actions.remove_file_id = Some(item.id);
                }
                render_status_badge(ui, &item.status);
            });
        });

        // Editable output stem + auto-synced container extension.
        ui.add_space(4.0);
        let ext = settings.container.extension();
        let editable = !is_running
            && !matches!(
                item.status,
                FileStatus::Converting(_) | FileStatus::Verifying
            );

        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Output:").size(12.0).color(TEXT_MUTED));
            ui.add_enabled(
                editable,
                TextEdit::singleline(&mut item.custom_stem)
                    .desired_width(210.0)
                    .hint_text("output filename"),
            );
            ui.label(
                RichText::new(format!(".{ext}"))
                    .monospace()
                    .size(12.5)
                    .strong()
                    .color(ACCENT_BLUE),
            );

            if !settings.output_prefix.trim().is_empty()
                || !settings.output_suffix.trim().is_empty()
            {
                let preview_name = compose_output_filename(
                    &item.custom_stem,
                    &settings.output_prefix,
                    &settings.output_suffix,
                    ext,
                );
                ui.label(
                    RichText::new(format!("-> {preview_name}"))
                        .size(11.5)
                        .color(TEXT_MUTED),
                );
            }
        });

        match &item.status {
            FileStatus::Converting(stats) => {
                ui.add_space(6.0);
                let pct = (stats.fraction * 100.0).clamp(0.0, 100.0);
                ui.add(
                    ProgressBar::new(stats.fraction.clamp(0.0, 1.0))
                        .text(format!("{pct:.1}%  ({})", format_duration(stats.out_time_secs))),
                );
                let mut metrics = Vec::new();
                if let Some(ref speed) = stats.speed {
                    metrics.push(format!("Speed: {speed}"));
                }
                if let Some(ref fps) = stats.fps {
                    metrics.push(format!("FPS: {fps}"));
                }
                if let Some(ref br) = stats.bitrate {
                    metrics.push(format!("Bitrate: {br}"));
                }
                if !metrics.is_empty() {
                    ui.label(
                        RichText::new(metrics.join("  |  "))
                            .size(11.5)
                            .color(TEXT_MUTED),
                    );
                }
            }
            FileStatus::Verifying => {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(
                        RichText::new("Running mode-aware ffprobe & integrity verification...")
                            .size(12.0)
                            .color(ACCENT_AMBER),
                    );
                });
            }
            FileStatus::Done {
                output_path,
                report,
            } => {
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    let out_name = output_path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    let dur_tag = report
                        .duration_secs
                        .filter(|d| *d > 0.0)
                        .map(|d| format!(" | {}", format_duration(d)))
                        .unwrap_or_default();
                    let summary_color = if report.deep_decode_passed {
                        ACCENT_GREEN
                    } else {
                        ACCENT_AMBER
                    };
                    ui.label(
                        RichText::new(format!(
                            "Saved: {out_name} ({} | {} stream{}{dur_tag}) [{}]",
                            format_bytes(report.size_bytes),
                            report.stream_count,
                            if report.stream_count == 1 { "" } else { "s" },
                            report.verification_note
                        ))
                        .size(12.0)
                        .color(summary_color),
                    );
                    if ui.small_button("Reveal in Folder").clicked() {
                        reveal_in_explorer(output_path);
                    }
                });
            }
            FileStatus::Failed(err) => {
                ui.add_space(4.0);
                ui.label(RichText::new(err).size(12.0).color(ACCENT_RED));
            }
            _ => {}
        }
    });
}

fn render_status_badge(ui: &mut egui::Ui, status: &FileStatus) {
    match status {
        FileStatus::Probing => status_pill(ui, "Probing", ACCENT_BLUE),
        FileStatus::Ready => status_pill(ui, "Ready", ACCENT_BLUE),
        FileStatus::Converting(_) => status_pill(ui, "Converting", ACCENT_BLUE),
        FileStatus::Verifying => status_pill(ui, "Verifying", ACCENT_AMBER),
        FileStatus::Done { .. } => status_pill(ui, "Verified OK", ACCENT_GREEN),
        FileStatus::Failed(_) => status_pill(ui, "Failed", ACCENT_RED),
        FileStatus::Cancelled => status_pill(ui, "Cancelled", ACCENT_AMBER),
    }
}

/// Opens Windows Explorer with the output file selected without invoking a shell.
fn reveal_in_explorer(path: &Path) {
    #[cfg(windows)]
    {
        let mut arg = std::ffi::OsString::from("/select,");
        arg.push(path.as_os_str());
        let _ = Command::new("explorer.exe")
            .arg(arg)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
    #[cfg(not(windows))]
    {
        if let Some(parent) = path.parent() {
            let _ = Command::new("xdg-open").arg(parent).spawn();
        }
    }
}
