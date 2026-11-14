use crate::i18n::{Language, media_summary_localized, tr, verification_note_localized};
use crate::models::config::ConversionSettings;
use crate::models::media::{FileStatus, QueuedFile, format_bytes, format_duration};
use crate::security::path_guard::compose_output_filename;
use crate::ui::theme::{
    ACCENT_AMBER, ACCENT_BLUE, ACCENT_GREEN, ACCENT_RED, TEXT_MUTED, card_frame, status_pill,
};
use eframe::egui::{self, Label, ProgressBar, RichText, TextEdit};
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
    lang: Language,
    queue: &mut [QueuedFile],
    settings: &ConversionSettings,
    is_running: bool,
    is_hovering_files: bool,
) -> QueueActions {
    let mut actions = QueueActions::default();
    let t = tr(lang);

    if is_hovering_files {
        status_pill(ui, t.drop_hover_hint(), ACCENT_BLUE);
        ui.add_space(6.0);
    }

    if queue.is_empty() {
        ui.centered_and_justified(|ui| {
            card_frame().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(32.0);
                    ui.add(
                        Label::new(
                            RichText::new(t.drop_empty_title())
                                .size(18.0)
                                .strong(),
                        )
                        .wrap(),
                    );
                    ui.add_space(8.0);
                    ui.add(
                        Label::new(RichText::new(t.drop_empty_subtitle()).color(TEXT_MUTED))
                            .wrap(),
                    );
                    ui.add_space(16.0);
                    if ui.button(t.browse_media_files()).clicked() {
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
                render_file_card(ui, lang, item, settings, is_running, &mut actions);
                ui.add_space(6.0);
            }
        });

    actions
}

fn render_file_card(
    ui: &mut egui::Ui,
    lang: Language,
    item: &mut QueuedFile,
    settings: &ConversionSettings,
    is_running: bool,
    actions: &mut QueueActions,
) {
    let t = tr(lang);
    let file_name = item
        .input_path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| item.input_path.display().to_string());

    card_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(&file_name).size(14.5).strong());

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let removable = !is_running
                    || matches!(
                        item.status,
                        FileStatus::Ready
                            | FileStatus::Done { .. }
                            | FileStatus::Failed(_)
                            | FileStatus::Cancelled
                    );
                if removable && ui.small_button(t.remove_btn()).clicked() {
                    actions.remove_file_id = Some(item.id);
                }
                render_status_badge(ui, lang, &item.status);
            });
        });

        if let Some(ref media) = item.media {
            let size_txt = if media.file_size_bytes > 0 {
                format!("  |  {}", format_bytes(media.file_size_bytes))
            } else {
                String::new()
            };
            let summary = media_summary_localized(lang, media);
            ui.add(
                Label::new(
                    RichText::new(format!("{summary}{size_txt}"))
                        .size(12.0)
                        .color(TEXT_MUTED),
                )
                .wrap(),
            );
        } else if let Some(ref err) = item.probe_error {
            ui.add(
                Label::new(
                    RichText::new(format!("{} {err}", t.probe_error_prefix()))
                        .size(12.0)
                        .color(ACCENT_RED),
                )
                .wrap(),
            );
        } else {
            ui.label(
                RichText::new(t.inspecting_streams())
                    .size(12.0)
                    .color(TEXT_MUTED),
            );
        }

        // Editable output stem + auto-synced container extension.
        ui.add_space(4.0);
        let ext = settings.container.extension();
        let editable = !is_running
            && !matches!(
                item.status,
                FileStatus::Converting(_) | FileStatus::Verifying
            );

        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(t.output_label()).size(12.0).color(TEXT_MUTED));
            ui.add_enabled(
                editable,
                TextEdit::singleline(&mut item.custom_stem)
                    .desired_width(210.0)
                    .hint_text(t.output_hint()),
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
                    metrics.push(format!("{}: {speed}", t.metric_speed()));
                }
                if let Some(ref fps) = stats.fps {
                    metrics.push(format!("{}: {fps}", t.metric_fps()));
                }
                if let Some(ref br) = stats.bitrate {
                    metrics.push(format!("{}: {br}", t.metric_bitrate()));
                }
                if !metrics.is_empty() {
                    ui.add(
                        Label::new(
                            RichText::new(metrics.join("  |  "))
                                .size(11.5)
                                .color(TEXT_MUTED),
                        )
                        .wrap(),
                    );
                }
            }
            FileStatus::Verifying => {
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spinner();
                    ui.label(
                        RichText::new(t.verifying_progress())
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
                    let stream_word = if report.stream_count == 1 {
                        t.stream_singular()
                    } else {
                        t.stream_plural()
                    };
                    let localized_note =
                        verification_note_localized(lang, &report.verification_note);
                    ui.label(
                        RichText::new(format!(
                            "{} {out_name} ({} | {} {stream_word}{dur_tag}) [{localized_note}]",
                            t.saved_label(),
                            format_bytes(report.size_bytes),
                            report.stream_count,
                        ))
                        .size(12.0)
                        .color(summary_color),
                    );
                    if ui.small_button(t.reveal_in_folder()).clicked() {
                        reveal_in_explorer(output_path);
                    }
                });
            }
            FileStatus::Failed(err) => {
                ui.add_space(4.0);
                ui.add(Label::new(RichText::new(err).size(12.0).color(ACCENT_RED)).wrap());
            }
            _ => {}
        }
    });
}

fn render_status_badge(ui: &mut egui::Ui, lang: Language, status: &FileStatus) {
    let t = tr(lang);
    match status {
        FileStatus::Probing => status_pill(ui, t.status_probing(), ACCENT_BLUE),
        FileStatus::Ready => status_pill(ui, t.status_ready(), ACCENT_BLUE),
        FileStatus::Converting(_) => status_pill(ui, t.status_converting(), ACCENT_BLUE),
        FileStatus::Verifying => status_pill(ui, t.status_verifying(), ACCENT_AMBER),
        FileStatus::Done { .. } => status_pill(ui, t.status_verified_ok(), ACCENT_GREEN),
        FileStatus::Failed(_) => status_pill(ui, t.status_failed(), ACCENT_RED),
        FileStatus::Cancelled => status_pill(ui, t.status_cancelled(), ACCENT_AMBER),
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
