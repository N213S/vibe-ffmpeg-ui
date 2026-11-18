use crate::ffmpeg::discovery::app_executable_dir;
use crate::i18n::{Language, tr};
use crate::net::ffmpeg_update::{OFFICIAL_BUILDS_PAGE, open_trusted_url};
use crate::models::media::{FfmpegInstallState, LocalFfmpegInfo, UpdateCheckState};
use crate::ui::theme::{
    ACCENT_AMBER, ACCENT_BLUE, ACCENT_GREEN, ACCENT_RED, BG_MAIN, BG_PANEL, BORDER_SUBTLE,
    TEXT_MUTED, card_frame, status_pill,
};
use eframe::egui::{
    self, Button, CentralPanel, Color32, CornerRadius, Frame, Label, Margin, RichText, Stroke,
    ViewportBuilder, ViewportClass, ViewportCommand, ViewportId,
};

#[derive(Debug, Default)]
pub struct SetupWindowActions {
    pub install_winget: bool,
    pub check_official: bool,
    pub rescan_ffmpeg: bool,
}

/// Renders the missing-FFmpeg setup & installer dialog as an independent native OS window.
#[allow(clippy::too_many_arguments)]
pub fn render_ffmpeg_setup_window(
    ctx: &egui::Context,
    lang: Language,
    open: &mut bool,
    focus_requested: &mut bool,
    ffmpeg: &LocalFfmpegInfo,
    install_state: &FfmpegInstallState,
    install_logs: &[String],
    update_state: &UpdateCheckState,
    winget_available: bool,
) -> SetupWindowActions {
    let mut actions = SetupWindowActions::default();
    if !*open {
        return actions;
    }

    let t = tr(lang);
    let viewport_id = ViewportId::from_hash_of("ffmpeg_setup_viewport");
    let builder = ViewportBuilder::default()
        .with_title(t.setup_window_title())
        .with_inner_size([600.0, 620.0])
        .with_min_inner_size([460.0, 480.0])
        .with_active(true);

    ctx.show_viewport_immediate(viewport_id, builder, |ui, class| {
        if class == ViewportClass::EmbeddedWindow {
            *focus_requested = false;
            let mut embedded_open = *open;
            egui::Window::new(t.setup_window_title())
                .open(&mut embedded_open)
                .collapsible(false)
                .resizable(true)
                .default_size([580.0, 560.0])
                .show(ui.ctx(), |ui| {
                    render_setup_contents(
                        ui,
                        lang,
                        open,
                        ffmpeg,
                        install_state,
                        install_logs,
                        update_state,
                        winget_available,
                        &mut actions,
                    );
                });
            if !embedded_open {
                *open = false;
            }
            return;
        }

        if *focus_requested {
            ui.ctx().send_viewport_cmd(ViewportCommand::Focus);
            *focus_requested = false;
        }

        CentralPanel::default()
            .frame(
                Frame::central_panel(&ui.style())
                    .fill(BG_PANEL)
                    .inner_margin(Margin::same(14)),
            )
            .show(ui, |ui| {
                render_setup_contents(
                    ui,
                    lang,
                    open,
                    ffmpeg,
                    install_state,
                    install_logs,
                    update_state,
                    winget_available,
                    &mut actions,
                );
            });

        if ui.ctx().input(|i| i.viewport().close_requested()) {
            *open = false;
            ui.ctx().request_repaint_of(ViewportId::ROOT);
        }
    });

    actions
}

#[allow(clippy::too_many_arguments)]
fn render_setup_contents(
    ui: &mut egui::Ui,
    lang: Language,
    open: &mut bool,
    ffmpeg: &LocalFfmpegInfo,
    install_state: &FfmpegInstallState,
    install_logs: &[String],
    update_state: &UpdateCheckState,
    winget_available: bool,
    actions: &mut SetupWindowActions,
) {
    let t = tr(lang);
    let is_installing = matches!(install_state, FfmpegInstallState::Installing { .. });
    let is_checking_update = matches!(update_state, UpdateCheckState::Checking);

    // Top banner: red missing notice or green detected confirmation.
    if ffmpeg.is_ready() {
        Frame::new()
            .fill(Color32::from_rgba_unmultiplied(
                ACCENT_GREEN.r(),
                ACCENT_GREEN.g(),
                ACCENT_GREEN.b(),
                26,
            ))
            .stroke(Stroke::new(1.0, ACCENT_GREEN))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                let ver = ffmpeg
                    .semantic_version
                    .as_deref()
                    .unwrap_or(t.ffmpeg_detected());
                ui.horizontal_wrapped(|ui| {
                    status_pill(ui, &format!("FFmpeg {ver}"), ACCENT_GREEN);
                    ui.label(
                        RichText::new(t.setup_ready_banner())
                            .size(13.5)
                            .strong()
                            .color(ACCENT_GREEN),
                    );
                });
                if let Some(ref p) = ffmpeg.ffmpeg_path {
                    ui.add_space(4.0);
                    ui.add(
                        Label::new(
                            RichText::new(p.display().to_string())
                                .monospace()
                                .size(11.0)
                                .color(TEXT_MUTED),
                        )
                        .wrap(),
                    );
                }
            });
    } else {
        Frame::new()
            .fill(Color32::from_rgba_unmultiplied(
                ACCENT_RED.r(),
                ACCENT_RED.g(),
                ACCENT_RED.b(),
                24,
            ))
            .stroke(Stroke::new(1.0, ACCENT_RED))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                status_pill(ui, t.ffmpeg_missing(), ACCENT_RED);
                ui.add_space(4.0);
                ui.add(Label::new(RichText::new(t.setup_missing_intro()).size(12.5)).wrap());
            });
    }

    ui.add_space(10.0);

    egui::ScrollArea::vertical()
        .max_height((ui.available_height() - 64.0).max(160.0))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Option 1: Automatic WinGet Install
            card_frame().show(ui, |ui| {
                ui.label(RichText::new(t.setup_winget_title()).size(13.5).strong());
                ui.add_space(4.0);
                ui.add(
                    Label::new(RichText::new(t.setup_winget_desc()).size(12.0).color(TEXT_MUTED))
                        .wrap(),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new("winget install --id Gyan.FFmpeg --exact")
                        .monospace()
                        .size(11.5)
                        .color(ACCENT_BLUE),
                );
                ui.add_space(8.0);

                ui.horizontal_wrapped(|ui| {
                    let can_click = winget_available && !is_installing && !ffmpeg.is_ready();
                    let btn_label = if is_installing {
                        t.setup_winget_installing()
                    } else {
                        t.setup_winget_btn()
                    };
                    let install_btn = Button::new(RichText::new(btn_label).strong())
                        .fill(if can_click {
                            ACCENT_BLUE
                        } else {
                            Color32::from_rgb(45, 55, 72)
                        });
                    if ui.add_enabled(can_click, install_btn).clicked() {
                        actions.install_winget = true;
                        ui.ctx().request_repaint_of(ViewportId::ROOT);
                    }
                    if is_installing {
                        ui.spinner();
                    }
                });

                if !winget_available {
                    ui.add_space(6.0);
                    ui.add(
                        Label::new(
                            RichText::new(t.setup_winget_unavailable())
                                .size(11.5)
                                .color(ACCENT_AMBER),
                        )
                        .wrap(),
                    );
                }

                match install_state {
                    FfmpegInstallState::Idle => {}
                    FfmpegInstallState::Installing { latest_line } => {
                        ui.add_space(6.0);
                        ui.add(
                            Label::new(
                                RichText::new(latest_line)
                                    .monospace()
                                    .size(11.5)
                                    .color(TEXT_MUTED),
                            )
                            .wrap(),
                        );
                    }
                    FfmpegInstallState::Succeeded => {
                        ui.add_space(6.0);
                        ui.add(
                            Label::new(
                                RichText::new(t.setup_winget_success())
                                    .size(12.0)
                                    .color(ACCENT_GREEN),
                            )
                            .wrap(),
                        );
                    }
                    FfmpegInstallState::Failed(err) => {
                        ui.add_space(6.0);
                        ui.add(
                            Label::new(RichText::new(err).size(12.0).color(ACCENT_RED)).wrap(),
                        );
                    }
                }

                if !install_logs.is_empty() {
                    ui.add_space(6.0);
                    Frame::new()
                        .fill(BG_MAIN)
                        .stroke(Stroke::new(1.0, BORDER_SUBTLE))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin::same(8))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .id_salt("winget_output_scroll")
                                .max_height(105.0)
                                .stick_to_bottom(true)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    for line in install_logs {
                                        ui.add(
                                            Label::new(
                                                RichText::new(line)
                                                    .monospace()
                                                    .size(11.0)
                                                    .color(TEXT_MUTED),
                                            )
                                            .wrap(),
                                        );
                                    }
                                });
                        });
                }
            });

            ui.add_space(10.0);

            // Option 2: Check Official Release & Manual Download (gyan.dev)
            card_frame().show(ui, |ui| {
                ui.label(RichText::new(t.setup_official_title()).size(13.5).strong());
                ui.add_space(4.0);
                ui.add(
                    Label::new(
                        RichText::new(t.setup_official_desc())
                            .size(12.0)
                            .color(TEXT_MUTED),
                    )
                    .wrap(),
                );
                if let Some(app_dir) = app_executable_dir() {
                    ui.add_space(4.0);
                    ui.add(
                        Label::new(
                            RichText::new(app_dir.display().to_string())
                                .monospace()
                                .size(11.0)
                                .color(TEXT_MUTED),
                        )
                        .wrap(),
                    );
                }

                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    let check_btn = Button::new(if is_checking_update {
                        t.checking()
                    } else {
                        t.setup_check_official_btn()
                    });
                    if ui.add_enabled(!is_checking_update, check_btn).clicked() {
                        actions.check_official = true;
                        ui.ctx().request_repaint_of(ViewportId::ROOT);
                    }

                    let download_btn = Button::new(RichText::new(t.setup_open_download_btn()).strong())
                        .fill(Color32::from_rgb(35, 42, 58));
                    if ui.add(download_btn).clicked() {
                        let _ = open_trusted_url(OFFICIAL_BUILDS_PAGE);
                    }

                    if is_checking_update {
                        ui.spinner();
                    }
                });

                match update_state {
                    UpdateCheckState::Idle => {}
                    UpdateCheckState::Checking => {}
                    UpdateCheckState::Completed(info) => {
                        ui.add_space(6.0);
                        ui.horizontal_wrapped(|ui| {
                            status_pill(
                                ui,
                                &format!(
                                    "{} {}",
                                    t.setup_official_release_prefix(),
                                    info.latest_release
                                ),
                                ACCENT_BLUE,
                            );
                            if let Some(ref git_ver) = info.latest_git {
                                ui.label(
                                    RichText::new(format!("git: {git_ver}"))
                                        .size(11.5)
                                        .color(TEXT_MUTED),
                                );
                            }
                        });
                    }
                    UpdateCheckState::Error(err) => {
                        ui.add_space(6.0);
                        ui.add(
                            Label::new(RichText::new(err).size(11.5).color(ACCENT_RED)).wrap(),
                        );
                    }
                }
            });
        });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        if ui
            .add_enabled(!is_installing, Button::new(t.setup_rescan_btn()))
            .clicked()
        {
            actions.rescan_ffmpeg = true;
            ui.ctx().request_repaint_of(ViewportId::ROOT);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let close_btn = if ffmpeg.is_ready() {
                Button::new(RichText::new(t.close_btn()).strong()).fill(ACCENT_GREEN)
            } else {
                Button::new(t.close_btn())
            };
            if ui.add(close_btn).clicked() {
                *open = false;
                ui.ctx().request_repaint_of(ViewportId::ROOT);
            }
        });
    });
}
