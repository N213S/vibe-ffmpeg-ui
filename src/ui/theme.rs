use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Frame, Margin, RichText,
    Stroke,
};
use std::sync::Arc;

pub const BG_MAIN: Color32 = Color32::from_rgb(15, 18, 25);
pub const BG_PANEL: Color32 = Color32::from_rgb(22, 26, 36);
pub const BG_CARD: Color32 = Color32::from_rgb(28, 33, 46);
pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(48, 56, 76);

pub const ACCENT_BLUE: Color32 = Color32::from_rgb(59, 130, 246);
pub const ACCENT_GREEN: Color32 = Color32::from_rgb(34, 197, 94);
pub const ACCENT_AMBER: Color32 = Color32::from_rgb(245, 158, 11);
pub const ACCENT_RED: Color32 = Color32::from_rgb(239, 68, 68);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(148, 163, 184);

/// Applies dark studio styling and loads Windows Thai/Unicode fallback fonts.
pub fn configure_style_and_fonts(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = BG_PANEL;
    visuals.panel_fill = BG_MAIN;
    visuals.faint_bg_color = BG_CARD;
    visuals.extreme_bg_color = Color32::from_rgb(11, 13, 19);
    visuals.selection.bg_fill = ACCENT_BLUE;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER_SUBTLE);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(35, 42, 58);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(47, 57, 79);
    visuals.widgets.active.bg_fill = ACCENT_BLUE;
    ctx.set_visuals(visuals);

    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    ctx.set_global_style(style);

    load_unicode_fonts(ctx);
}

fn load_unicode_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // Load Thai + Unicode Windows system fonts if present so non-ASCII filenames render cleanly.
    let candidates = [
        ("segoe_ui", "C:\\Windows\\Fonts\\segoeui.ttf"),
        ("tahoma", "C:\\Windows\\Fonts\\tahoma.ttf"),
        ("leelawadee", "C:\\Windows\\Fonts\\leelawui.ttf"),
    ];

    for (name, path) in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert(name.to_string(), Arc::new(FontData::from_owned(bytes)));
            if let Some(prop) = fonts.families.get_mut(&FontFamily::Proportional) {
                prop.insert(0, name.to_string());
            }
            if let Some(mono) = fonts.families.get_mut(&FontFamily::Monospace) {
                mono.push(name.to_string());
            }
        }
    }

    ctx.set_fonts(fonts);
}

/// Standard rounded card frame used across queue and settings sections.
pub fn card_frame() -> Frame {
    Frame::new()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
}

/// Renders a compact colored status badge pill.
pub fn status_pill(ui: &mut egui::Ui, text: &str, color: Color32) {
    let bg = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 32);
    Frame::new()
        .fill(bg)
        .stroke(Stroke::new(1.0, color))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.5).strong().color(color));
        });
}
