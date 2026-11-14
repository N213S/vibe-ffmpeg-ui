use crate::i18n::{Language, tr};
use crate::ui::theme::TEXT_MUTED;
use eframe::egui::{self, Label, RichText};
use std::collections::VecDeque;

/// Renders the collapsible FFmpeg execution and integrity verification log panel.
pub fn render_log_panel(
    ui: &mut egui::Ui,
    lang: Language,
    logs: &VecDeque<String>,
    expanded: &mut bool,
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
        if *expanded && !logs.is_empty() && ui.small_button(t.clear_log()).clicked() {
            *on_clear = true;
        }
    });

    if !*expanded {
        return;
    }

    ui.add_space(4.0);
    egui::ScrollArea::vertical()
        .max_height(145.0)
        .stick_to_bottom(true)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if logs.is_empty() {
                ui.add(
                    Label::new(
                        RichText::new(t.no_logs_yet())
                            .size(11.5)
                            .color(TEXT_MUTED),
                    )
                    .wrap(),
                );
            } else {
                for line in logs {
                    ui.add(
                        Label::new(RichText::new(line).monospace().size(11.0).color(TEXT_MUTED))
                            .wrap(),
                    );
                }
            }
        });
}
