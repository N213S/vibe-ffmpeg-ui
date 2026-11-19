use crate::i18n::{Language, tr};
use crate::models::media::{APP_VERSION, AppUpdateState, LocalFfmpegInfo};
use crate::net::ffmpeg_update::open_trusted_url;
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

/// Renders the header bar: binary status, official app update checker, language switcher, and queue actions.
pub fn render_top_bar(
    ui: &mut egui::Ui,
    lang: Language,
    ffmpeg: &LocalFfmpegInfo,
    app_update_state: &AppUpdateState,
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
            let ready_bg =
                Color32::from_rgba_unmultiplied(ACCENT_GREEN.r(), ACCENT_GREEN.g(), ACCENT_GREEN.b(), 32);
            let ready_btn = Button::new(
                RichText::new(format!("FFmpeg {ver}"))
                    .size(11.5)
                    .strong()
                    .color(ACCENT_GREEN),
            )
            .fill(ready_bg)
            .stroke(Stroke::new(1.0, ACCENT_GREEN))
            .corner_radius(CornerRadius::same(6));
            if ui
                .add(ready_btn)
                .on_hover_text(t.setup_window_title())
                .clicked()
            {
                actions.open_setup = true;
            }
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

        let checking = matches!(app_update_state, AppUpdateState::Checking);
        if ui
            .add_enabled(
                !checking,
                Button::new(if checking { t.checking() } else { t.check_update() }),
            )
            .clicked()
        {
            actions.check_updates = true;
        }

        match app_update_state {
            AppUpdateState::Idle => {
                ui.label(
                    RichText::new(format!("v{APP_VERSION}  |  {}", t.official_source()))
                        .size(12.0)
                        .color(TEXT_MUTED),
                );
            }
            AppUpdateState::Checking => {
                ui.spinner();
            }
            AppUpdateState::Completed(info) => {
                if info.update_available {
                    status_pill(
                        ui,
                        &format!("{} v{}", t.update_available(), info.latest_version),
                        ACCENT_AMBER,
                    );
                    if ui.button(t.official_builds()).clicked() {
                        let _ = open_trusted_url(&info.release_url);
                    }
                } else {
                    status_pill(
                        ui,
                        &format!("{} (v{})", t.up_to_date(), info.current_version),
                        ACCENT_GREEN,
                    );
                }
            }
            AppUpdateState::Error(err) => {
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
