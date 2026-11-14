use crate::models::media::{LocalFfmpegInfo, UpdateCheckState};
use crate::net::ffmpeg_update::{OFFICIAL_BUILDS_PAGE, open_trusted_url};
use crate::ui::theme::{ACCENT_AMBER, ACCENT_BLUE, ACCENT_GREEN, ACCENT_RED, TEXT_MUTED, status_pill};
use eframe::egui::{self, Button, RichText};

/// Actions triggered by top toolbar buttons.
#[derive(Debug, Default)]
pub struct TopBarActions {
    pub add_files: bool,
    pub clear_done: bool,
    pub clear_all: bool,
    pub start_batch: bool,
    pub cancel_batch: bool,
    pub check_updates: bool,
}

/// Renders the header bar: binary status, official update checker, and queue actions.
pub fn render_top_bar(
    ui: &mut egui::Ui,
    ffmpeg: &LocalFfmpegInfo,
    update_state: &UpdateCheckState,
    is_running: bool,
    has_pending_work: bool,
    queue_len: usize,
) -> TopBarActions {
    let mut actions = TopBarActions::default();

    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("FFmpeg Studio").size(19.0).strong());
        ui.add_space(6.0);

        if ffmpeg.is_ready() {
            let ver = ffmpeg
                .semantic_version
                .as_deref()
                .unwrap_or("detected");
            status_pill(ui, &format!("FFmpeg {ver}"), ACCENT_GREEN);
        } else {
            status_pill(ui, "FFmpeg / ffprobe missing on PATH", ACCENT_RED);
        }

        ui.separator();

        // Official update check control & result badge.
        let checking = matches!(update_state, UpdateCheckState::Checking);
        if ui
            .add_enabled(!checking, Button::new(if checking { "Checking..." } else { "Check Update" }))
            .clicked()
        {
            actions.check_updates = true;
        }

        match update_state {
            UpdateCheckState::Idle => {
                ui.label(
                    RichText::new("Official source: gyan.dev")
                        .size(12.0)
                        .color(TEXT_MUTED),
                );
            }
            UpdateCheckState::Checking => {
                ui.spinner();
            }
            UpdateCheckState::Completed(info) => {
                if info.update_available {
                    status_pill(
                        ui,
                        &format!("Update available: {}", info.latest_release),
                        ACCENT_AMBER,
                    );
                    if ui.button("Official Builds").clicked() {
                        let _ = open_trusted_url(OFFICIAL_BUILDS_PAGE);
                    }
                } else {
                    status_pill(
                        ui,
                        &format!("Up to date ({})", info.latest_release),
                        ACCENT_GREEN,
                    );
                }
                if let Some(ref git_ver) = info.latest_git {
                    ui.label(
                        RichText::new(format!("git: {git_ver}"))
                            .size(11.0)
                            .color(TEXT_MUTED),
                    );
                }
            }
            UpdateCheckState::Error(err) => {
                status_pill(ui, "Update check failed", ACCENT_RED);
                ui.label(RichText::new(err).size(11.0).color(ACCENT_RED));
            }
        }
    });

    ui.add_space(6.0);

    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(!is_running, Button::new("+ Add Media Files"))
            .clicked()
        {
            actions.add_files = true;
        }
        if ui
            .add_enabled(!is_running && queue_len > 0, Button::new("Clear Finished"))
            .clicked()
        {
            actions.clear_done = true;
        }
        if ui
            .add_enabled(!is_running && queue_len > 0, Button::new("Clear All"))
            .clicked()
        {
            actions.clear_all = true;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if is_running {
                let stop_btn = Button::new(RichText::new("Stop Batch").strong())
                    .fill(ACCENT_RED);
                if ui.add(stop_btn).clicked() {
                    actions.cancel_batch = true;
                }
            } else {
                let can_start = ffmpeg.is_ready() && has_pending_work;
                let start_btn = Button::new(RichText::new("Convert & Verify All").strong())
                    .fill(if can_start { ACCENT_BLUE } else { egui::Color32::from_gray(55) });
                if ui.add_enabled(can_start, start_btn).clicked() {
                    actions.start_batch = true;
                }
            }
        });
    });

    actions
}
