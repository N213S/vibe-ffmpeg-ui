use crate::i18n::{Language, tr};
use crate::ui::theme::{ACCENT_BLUE, BG_PANEL, TEXT_MUTED};
use eframe::egui::{
    self, Button, CentralPanel, Frame, Label, Margin, RichText, ViewportBuilder, ViewportClass,
    ViewportId,
};
use std::collections::VecDeque;

/// Renders the collapsible, vertically resizable docked log bar and toolbar actions.
pub fn render_log_panel(
    ui: &mut egui::Ui,
    lang: Language,
    logs: &VecDeque<String>,
    expanded: &mut bool,
    window_open: &mut bool,
    on_clear: &mut bool,
) {
    let t = tr(lang);

    ui.horizontal_wrapped(|ui| {
        let arrow = if *expanded { "▼" } else { "▶" };
        if ui
            .button(format!("{arrow} {} ({})", t.diagnostic_log_title(), logs.len()))
            .clicked()
        {
            *expanded = !*expanded;
        }

        let pop_btn = Button::new(t.pop_out_window())
            .small()
            .fill(if *window_open {
                ACCENT_BLUE
            } else {
                egui::Color32::from_rgb(35, 42, 58)
            });
        if ui.add(pop_btn).clicked() {
            *window_open = !*window_open;
            if *window_open {
                *expanded = false;
            }
        }

        if !logs.is_empty() && ui.small_button(t.copy_log()).clicked() {
            copy_logs_to_clipboard(ui.ctx(), logs);
        }
        if !logs.is_empty() && ui.small_button(t.clear_log()).clicked() {
            *on_clear = true;
        }
    });

    if !*expanded {
        return;
    }

    ui.add_space(4.0);
    render_log_entries(ui, lang, logs, ui.available_height().max(90.0));
}

/// Renders the standalone draggable, resizable Diagnostic & Verification Log as a true OS window.
pub fn render_log_window(
    ctx: &egui::Context,
    lang: Language,
    logs: &VecDeque<String>,
    open: &mut bool,
    on_clear: &mut bool,
) {
    if !*open {
        return;
    }

    let t = tr(lang);
    let title = format!("{} ({})", t.diagnostic_log_title(), logs.len());
    let viewport_id = ViewportId::from_hash_of("diagnostic_log_viewport");
    let builder = ViewportBuilder::default()
        .with_title(title.clone())
        .with_inner_size([780.0, 480.0])
        .with_min_inner_size([400.0, 220.0]);

    ctx.show_viewport_immediate(viewport_id, builder, |ui, class| {
        if class == ViewportClass::EmbeddedWindow {
            let mut embedded_open = *open;
            egui::Window::new(title.as_str())
                .id(egui::Id::unique("diagnostic_log_window"))
                .open(&mut embedded_open)
                .collapsible(true)
                .resizable(true)
                .default_size(egui::vec2(760.0, 440.0))
                .min_size(egui::vec2(380.0, 220.0))
                .show(ui.ctx(), |ui| {
                    render_log_window_contents(ui, lang, logs, open, on_clear);
                });
            if !embedded_open {
                *open = false;
            }
            return;
        }

        CentralPanel::default()
            .frame(
                Frame::central_panel(&ui.style())
                    .fill(BG_PANEL)
                    .inner_margin(Margin::same(12)),
            )
            .show(ui, |ui| {
                render_log_window_contents(ui, lang, logs, open, on_clear);
            });

        if ui.ctx().input(|i| i.viewport().close_requested()) {
            *open = false;
            ui.ctx().request_repaint_of(ViewportId::ROOT);
        }
    });
}

fn render_log_window_contents(
    ui: &mut egui::Ui,
    lang: Language,
    logs: &VecDeque<String>,
    open: &mut bool,
    on_clear: &mut bool,
) {
    let t = tr(lang);
    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(!logs.is_empty(), Button::new(t.copy_log()).small())
            .clicked()
        {
            copy_logs_to_clipboard(ui.ctx(), logs);
        }
        if ui
            .add_enabled(!logs.is_empty(), Button::new(t.clear_log()).small())
            .clicked()
        {
            *on_clear = true;
            ui.ctx().request_repaint_of(ViewportId::ROOT);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button(t.close_btn()).clicked() {
                *open = false;
                ui.ctx().request_repaint_of(ViewportId::ROOT);
            }
        });
    });
    ui.separator();
    render_log_entries(ui, lang, logs, ui.available_height().max(140.0));
}

fn render_log_entries(
    ui: &mut egui::Ui,
    lang: Language,
    logs: &VecDeque<String>,
    max_height: f32,
) {
    let t = tr(lang);
    egui::ScrollArea::vertical()
        .max_height(max_height)
        .stick_to_bottom(true)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if logs.is_empty() {
                ui.add(
                    Label::new(
                        RichText::new(t.no_logs_yet())
                            .size(12.0)
                            .color(TEXT_MUTED),
                    )
                    .wrap(),
                );
            } else {
                for line in logs {
                    ui.add(
                        Label::new(RichText::new(line).monospace().size(11.5).color(TEXT_MUTED))
                            .wrap(),
                    );
                }
            }
        });
}

fn copy_logs_to_clipboard(ctx: &egui::Context, logs: &VecDeque<String>) {
    let mut joined = String::with_capacity(logs.len() * 64);
    for (idx, line) in logs.iter().enumerate() {
        if idx > 0 {
            joined.push('\n');
        }
        joined.push_str(line);
    }
    ctx.copy_text(joined);
}
