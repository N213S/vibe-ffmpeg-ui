#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod ffmpeg;
mod i18n;
mod models;
mod net;
mod security;
mod ui;

use app::FfmpegStudioApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1160.0, 760.0])
            .with_min_inner_size([840.0, 560.0])
            .with_title("FFmpeg Studio (Rust)")
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "FFmpeg Studio",
        native_options,
        Box::new(|cc| Ok(Box::new(FfmpegStudioApp::new(cc)))),
    )
}
