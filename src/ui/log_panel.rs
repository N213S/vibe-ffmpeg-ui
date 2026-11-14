use crate::ui::theme::TEXT_MUTED;
use eframe::egui::{self, RichText};
use std::collections::VecDeque;

/// Renders the collapsible FFmpeg execution and integrity verification log panel.
pub fn render_log_panel(
    ui: &mut egui::Ui,
    logs: &VecDeque<String>,
    expanded: &mut bool,
    on_clear: &mut bool,
) {
    ui.horizontal(|ui| {
        let arrow = if *expanded { "▼" } else { "▶" };
        if ui
            .button(format!("{arrow} Diagnostic & Verification Log ({})", logs.len()))
            .clicked()
        {
            *expanded = !*expanded;
        }
        if *expanded && !logs.is_empty() && ui.small_button("Clear Log").clicked() {
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
                ui.label(
                    RichText::new("No execution logs yet.")
                        .size(11.5)
                        .color(TEXT_MUTED),
                );
            } else {
                for line in logs {
                    ui.label(RichText::new(line).monospace().size(11.0).color(TEXT_MUTED));
                }
            }
        });
}
