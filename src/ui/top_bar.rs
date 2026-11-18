use crate::i18n::{Language, tr};
use crate::models::media::{LocalFfmpegInfo, UpdateCheckState};
use crate::net::ffmpeg_update::{OFFICIAL_BUILDS_PAGE, open_trusted_url};
use crate::ui::theme::{ACCENT_AMBER, ACCENT_BLUE, ACCENT_GREEN, ACCENT_RED, TEXT_MUTED, status_pill};
use eframe::egui::{self, Button, Color32, CornerRadius, Label, RichText, Stroke};

/// Actions triggered by top toolbar buttons.
#[derive(Debug, Default)]
pub struct TopBarActions {
    pub add_files: bool,
    pub clear_done: bool,
    pub clear_all: bool,
    pub start_batch: bool,
    pub cancel_batch: bool,
    pub check_updates: bool,
    pub toggle_language: bool,
    pub open_setup: bool,
}

/// Renders the header bar: binary status, official update checker, language switcher, and queue actions.
pub fn render_top_bar(
    ui: &mut egui::Ui,
    lang: Language,
    ffmpeg: &LocalFfmpegInfo,
    update_state: &UpdateCheckState,
    is_running: bool,
    has_pending_work: bool,
    queue_len: usize,
) -> TopBarActions {
    let mut actions = TopBarActions::default();
    let t = tr(lang);

    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(t.app_title()).size(19.0).strong());
        ui.add_space(6.0);

        if ffmpeg.is_ready() {
            let ver = ffmpeg
                .semantic_version
                .as_deref()
                .unwrap_or(t.ffmpeg_detected());
            status_pill(ui, &format!("FFmpeg {ver}"), ACCENT_GREEN);
        } else {
            let missing_bg =
                Color32::from_rgba_unmultiplied(ACCENT_RED.r(), ACCENT_RED.g(), ACCENT_RED.b(), 36);
            let missing_btn = Button::new(
                RichText::new(format!("{} ({})", t.ffmpeg_missing(), t.setup_btn()))
                    .size(11.5)
                    .strong()
                    .color(ACCENT_RED),
            )
            .fill(missing_bg)
            .stroke(Stroke::new(1.0, ACCENT_RED))
            .corner_radius(CornerRadius::same(6));
            if ui
                .add(missing_btn)
                .on_hover_text(t.setup_window_title())
                .clicked()
            {
                actions.open_setup = true;
            }
        }

        ui.separator();

        let checking = matches!(update_state, UpdateCheckState::Checking);
        if ui
            .add_enabled(
                !checking,
                Button::new(if checking { t.checking() } else { t.check_update() }),
            )
            .clicked()
        {
            actions.check_updates = true;
        }

        match update_state {
            UpdateCheckState::Idle => {
                ui.label(
                    RichText::new(t.official_source())
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
                        &format!("{} {}", t.update_available(), info.latest_release),
                        ACCENT_AMBER,
                    );
                    if ui.button(t.official_builds()).clicked() {
                        let _ = open_trusted_url(OFFICIAL_BUILDS_PAGE);
                    }
                } else if info.local_version.is_none() {
                    status_pill(
                        ui,
                        &format!("{} {}", t.latest_official_label(), info.latest_release),
                        ACCENT_BLUE,
                    );
                    if ui.button(t.official_builds()).clicked() {
                        let _ = open_trusted_url(OFFICIAL_BUILDS_PAGE);
                    }
                } else {
                    status_pill(
                        ui,
                        &format!("{} ({})", t.up_to_date(), info.latest_release),
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
                status_pill(ui, t.update_failed(), ACCENT_RED);
                ui.add(Label::new(RichText::new(err).size(11.0).color(ACCENT_RED)).wrap());
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let lang_btn =
                Button::new(format!("{} - {}", lang.short_code(), t.language_button()));
            if ui
                .add(lang_btn)
                .on_hover_text(t.ai_translation_notice())
                .clicked()
            {
                actions.toggle_language = true;
            }
        });
    });

    ui.add_space(6.0);

    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(!is_running, Button::new(t.add_media_files()))
            .clicked()
        {
            actions.add_files = true;
        }
        if ui
            .add_enabled(!is_running && queue_len > 0, Button::new(t.clear_finished()))
            .clicked()
        {
            actions.clear_done = true;
        }
        if ui
            .add_enabled(!is_running && queue_len > 0, Button::new(t.clear_all()))
            .clicked()
        {
            actions.clear_all = true;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if is_running {
                let stop_btn = Button::new(RichText::new(t.stop_batch()).strong())
                    .fill(ACCENT_RED);
                if ui.add(stop_btn).clicked() {
                    actions.cancel_batch = true;
                }
            } else {
                let can_start = ffmpeg.is_ready() && has_pending_work;
                let start_btn = Button::new(RichText::new(t.convert_verify_all()).strong())
                    .fill(if can_start {
                        ACCENT_BLUE
                    } else {
                        egui::Color32::from_gray(55)
                    });
                if ui.add_enabled(can_start, start_btn).clicked() {
                    actions.start_batch = true;
                }
            }
        });
    });

    actions
}
