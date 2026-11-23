use crate::i18n::{Language, tr};
use crate::models::media::{MediaInfo, QueuedFile, StreamDetail, format_bytes, format_duration};
use crate::ui::theme::{
    ACCENT_AMBER, ACCENT_BLUE, ACCENT_GREEN, ACCENT_RED, BG_MAIN, BORDER_SUBTLE, TEXT_MUTED,
    card_frame, standard_button_height, status_pill, uniform_button, uniform_button_width,
};
use eframe::egui::{self, CornerRadius, Frame, Label, Margin, RichText, Stroke};

/// Renders the read-only `ffprobe` container, stream, metadata tag, and raw JSON inspector panel.
pub fn render_inspector_panel(
    ui: &mut egui::Ui,
    lang: Language,
    selected_file: Option<&QueuedFile>,
    raw_json_open: &mut bool,
) -> bool {
    let mut reprobe_clicked = false;
    let t = tr(lang);

    egui::ScrollArea::vertical()
        .id_salt("inspector_panel_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let Some(item) = selected_file else {
                inspector_card(ui, |ui| {
                    ui.label(RichText::new(t.inspector_title()).size(15.0).strong());
                    ui.add_space(6.0);
                    ui.add(
                        Label::new(
                            RichText::new(t.inspector_empty_hint())
                                .size(12.5)
                                .color(TEXT_MUTED),
                        )
                        .wrap(),
                    );
                });
                return;
            };

            let file_name = item
                .input_path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| item.input_path.display().to_string());

            // File Header Card
            inspector_card(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(t.inspector_title()).strong().color(ACCENT_BLUE));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let btn_w = uniform_button_width(
                            ui,
                            &[t.reprobe_btn(), t.inspector_copy_report()],
                            88.0,
                        );
                        let btn_h = standard_button_height(ui);
                        if let Some(ref media) = item.media
                            && uniform_button(ui, t.inspector_copy_report(), btn_w, btn_h, None)
                                .clicked()
                        {
                            ui.ctx().copy_text(media.formatted_report(&item.input_path));
                        }
                        if uniform_button(ui, t.reprobe_btn(), btn_w, btn_h, None).clicked() {
                            reprobe_clicked = true;
                        }
                    });
                });
                ui.add_space(4.0);
                ui.add(Label::new(RichText::new(file_name).size(14.5).strong()).wrap());
                ui.add(
                    Label::new(
                        RichText::new(item.input_path.display().to_string())
                            .monospace()
                            .size(11.0)
                            .color(TEXT_MUTED),
                    )
                    .wrap(),
                );
                if let Some(ref err) = item.probe_error {
                    ui.add_space(6.0);
                    ui.add(Label::new(RichText::new(err).size(12.0).color(ACCENT_RED)).wrap());
                }
            });

            let Some(ref media) = item.media else {
                ui.add_space(8.0);
                inspector_card(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            RichText::new(t.inspecting_streams())
                                .size(12.5)
                                .color(TEXT_MUTED),
                        );
                    });
                });
                return;
            };

            ui.add_space(8.0);
            render_container_card(ui, lang, media);

            if !media.format_tags.is_empty() {
                ui.add_space(8.0);
                render_metadata_tags_card(ui, lang, media);
            }

            for stream in &media.streams {
                ui.add_space(8.0);
                render_stream_card(ui, lang, stream);
            }

            ui.add_space(8.0);
            render_raw_json_card(ui, lang, media, raw_json_open);
        });

    reprobe_clicked
}

fn inspector_card(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        add_contents(ui);
    });
}

fn kv_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.label(RichText::new(label).size(12.0).color(TEXT_MUTED));
    ui.add(Label::new(RichText::new(value).size(12.0)).wrap());
    ui.end_row();
}

fn render_container_card(ui: &mut egui::Ui, lang: Language, media: &MediaInfo) {
    let t = tr(lang);
    inspector_card(ui, |ui| {
        ui.label(RichText::new(t.inspector_container_section()).strong());
        ui.add_space(6.0);

        let format_val = match media.format_long_name.as_deref() {
            Some(long) if !long.is_empty() => format!("{} ({long})", media.format_name),
            _ => media.format_name.clone(),
        };
        let size_val = format!(
            "{} ({} B)",
            format_bytes(media.file_size_bytes),
            media.file_size_bytes
        );
        let dur_val = media
            .duration_secs
            .map(|d| format!("{} ({d:.3} s)", format_duration(d)))
            .unwrap_or_else(|| "N/A".to_string());
        let br_val = media
            .bit_rate
            .map(|bps| format!("{} kbps ({bps} bps)", bps / 1000))
            .unwrap_or_else(|| "N/A".to_string());

        let mut caps = Vec::new();
        if media.has_video {
            caps.push("Video");
        }
        if media.has_audio {
            caps.push("Audio");
        }
        if media.has_subtitle {
            caps.push("Subtitle");
        }
        if media.is_still_image {
            caps.push("Still Image");
        }
        let streams_val = if caps.is_empty() {
            media.streams.len().to_string()
        } else {
            format!("{} ({})", media.streams.len(), caps.join(", "))
        };

        egui::Grid::new("inspector_container_grid")
            .num_columns(2)
            .spacing(egui::vec2(12.0, 4.0))
            .show(ui, |ui| {
                kv_row(ui, t.inspector_format_label(), &format_val);
                kv_row(ui, "Size", &size_val);
                kv_row(ui, t.inspector_duration_label(), &dur_val);
                kv_row(ui, t.inspector_bitrate_label(), &br_val);
                kv_row(ui, t.inspector_streams_section(), &streams_val);
                if let Some(start) = media.start_time_secs {
                    kv_row(ui, "Start Time", &format!("{start:.3} s"));
                }
                if let Some(score) = media.probe_score {
                    kv_row(ui, "Probe Score", &format!("{score}/100"));
                }
            });
    });
}

fn render_metadata_tags_card(ui: &mut egui::Ui, lang: Language, media: &MediaInfo) {
    let t = tr(lang);
    inspector_card(ui, |ui| {
        ui.label(RichText::new(t.inspector_metadata_section()).strong());
        ui.add_space(6.0);
        egui::Grid::new("inspector_format_tags_grid")
            .num_columns(2)
            .spacing(egui::vec2(12.0, 4.0))
            .show(ui, |ui| {
                for (k, v) in &media.format_tags {
                    kv_row(ui, k, v);
                }
            });
    });
}

fn render_stream_card(ui: &mut egui::Ui, lang: Language, stream: &StreamDetail) {
    let t = tr(lang);
    inspector_card(ui, |ui| {
        let type_upper = if stream.codec_type.is_empty() {
            "UNKNOWN".to_string()
        } else {
            stream.codec_type.to_ascii_uppercase()
        };
        let badge_color = match stream.codec_type.as_str() {
            "video" => ACCENT_BLUE,
            "audio" => ACCENT_GREEN,
            "subtitle" => ACCENT_AMBER,
            _ => TEXT_MUTED,
        };

        ui.horizontal_wrapped(|ui| {
            status_pill(ui, &type_upper, badge_color);
            ui.label(RichText::new(format!("Stream #{}", stream.index)).strong());
            ui.label(
                RichText::new(stream.codec_name.to_ascii_uppercase())
                    .strong()
                    .color(badge_color),
            );
            if stream.is_default {
                status_pill(ui, "Default", ACCENT_BLUE);
            }
            if stream.is_attached_pic {
                status_pill(ui, "Cover Art", ACCENT_AMBER);
            }
            if let Some(ref code) = stream.language {
                status_pill(ui, code, TEXT_MUTED);
            }
        });

        ui.add_space(6.0);
        egui::Grid::new(("inspector_stream_grid", stream.index))
            .num_columns(2)
            .spacing(egui::vec2(12.0, 4.0))
            .show(ui, |ui| {
                let codec_val = match stream.codec_long_name.as_deref() {
                    Some(long) if !long.is_empty() => format!("{} ({long})", stream.codec_name),
                    _ => stream.codec_name.clone(),
                };
                kv_row(ui, "Codec", &codec_val);

                if let Some(ref profile) = stream.profile {
                    kv_row(ui, "Profile", profile);
                }
                if let Some(ref tag) = stream.codec_tag_string
                    && !tag.is_empty()
                    && tag != "0x0000"
                    && tag != "[0][0][0][0]"
                {
                    kv_row(ui, "Codec Tag", tag);
                }
                if let (Some(w), Some(h)) = (stream.width, stream.height) {
                    kv_row(ui, t.resolution(), &format!("{w}x{h}"));
                }
                if let Some(ref pix) = stream.pixel_format {
                    kv_row(ui, t.pixel_format(), pix);
                }
                let color_parts: Vec<&str> = [
                    stream.color_space.as_deref(),
                    stream.color_range.as_deref(),
                    stream.color_transfer.as_deref(),
                    stream.color_primaries.as_deref(),
                ]
                .into_iter()
                .flatten()
                .filter(|s| !s.is_empty())
                .collect();
                if !color_parts.is_empty() {
                    kv_row(ui, "Color", &color_parts.join(" / "));
                }
                if let Some(fps) = stream.fps {
                    let avg = stream.avg_frame_rate.as_deref().unwrap_or("-");
                    let r = stream.r_frame_rate.as_deref().unwrap_or("-");
                    kv_row(ui, t.frame_rate(), &format!("{fps:.3} fps (avg: {avg}, r: {r})"));
                }
                if let Some(frames) = stream.nb_frames {
                    kv_row(ui, "Frames", &frames.to_string());
                }
                if let Some(sr) = stream.sample_rate {
                    kv_row(ui, t.sample_rate(), &format!("{sr} Hz"));
                }
                if let Some(ch) = stream.channels {
                    let layout = stream
                        .channel_layout
                        .as_deref()
                        .map(|l| format!(" ({l})"))
                        .unwrap_or_default();
                    kv_row(ui, t.channels(), &format!("{ch} ch{layout}"));
                }
                if let Some(br) = stream.bit_rate {
                    kv_row(ui, t.metric_bitrate(), &format!("{} kbps ({br} bps)", br / 1000));
                }
                if let Some(dur) = stream.duration_secs {
                    kv_row(ui, t.inspector_duration_label(), &format!("{dur:.3} s"));
                }
                if let Some(ref lang_code) = stream.language {
                    kv_row(ui, "Language", lang_code);
                }
                if let Some(ref title) = stream.title {
                    kv_row(ui, "Title", title);
                }
                for (k, v) in &stream.tags {
                    if k != "language" && k != "title" {
                        kv_row(ui, k, v);
                    }
                }
            });
    });
}

fn render_raw_json_card(
    ui: &mut egui::Ui,
    lang: Language,
    media: &MediaInfo,
    raw_json_open: &mut bool,
) {
    let t = tr(lang);
    inspector_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            let arrow = if *raw_json_open { "▼" } else { "▶" };
            let toggle_label = format!("{arrow} {}", t.inspector_raw_json());
            let btn_w = uniform_button_width(
                ui,
                &[toggle_label.as_str(), t.inspector_copy_json()],
                100.0,
            );
            let btn_h = standard_button_height(ui);
            if uniform_button(ui, &toggle_label, btn_w, btn_h, None).clicked() {
                *raw_json_open = !*raw_json_open;
            }
            if !media.raw_json.is_empty()
                && uniform_button(ui, t.inspector_copy_json(), btn_w, btn_h, None).clicked()
            {
                ui.ctx().copy_text(media.raw_json.clone());
            }
        });

        if *raw_json_open {
            ui.add_space(6.0);
            Frame::new()
                .fill(BG_MAIN)
                .stroke(Stroke::new(1.0, BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    egui::ScrollArea::vertical()
                        .id_salt("inspector_raw_json_scroll")
                        .min_scrolled_height(160.0)
                        .max_height(260.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add(
                                Label::new(
                                    RichText::new(&media.raw_json)
                                        .monospace()
                                        .size(11.0)
                                        .color(TEXT_MUTED),
                                )
                                .wrap(),
                            );
                        });
                });
        }
    });
}
