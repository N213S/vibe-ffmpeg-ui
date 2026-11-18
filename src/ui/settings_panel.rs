use crate::ffmpeg::command_builder::build_command_plan;
use crate::i18n::{
    Language, audio_channels_label, audio_codec_label, audio_sample_rate_label, container_label,
    encode_speed_label, fps_label, incompatibility_tooltip, mode_label, pixel_format_label,
    preset_label, resolution_label, tr, video_codec_label,
};
use crate::models::config::{
    AudioChannels, AudioCodec, AudioSampleRate, ContainerFormat, ConvertMode, ConversionSettings,
    EncodeSpeed, FpsOption, PixelFormat, PresetProfile, ResolutionOption, VideoCodec,
    VideoRateControl, check_mode_compatibility,
};
use crate::models::media::{LocalFfmpegInfo, MediaInfo, QueuedFile};
use crate::security::path_guard::compose_output_filename;
use crate::ui::theme::{
    ACCENT_AMBER, ACCENT_BLUE, BG_MAIN, BG_PANEL, BORDER_SUBTLE, TEXT_MUTED, card_frame, status_pill,
};
use eframe::egui::{self, Button, CornerRadius, Frame, Label, Margin, RichText, Stroke, TextEdit};
use std::path::Path;

/// Renders the right-hand multi-mode encoding parameters, output naming, safety guards, and command preview.
pub fn render_settings_panel(
    ui: &mut egui::Ui,
    lang: Language,
    settings: &mut ConversionSettings,
    ffmpeg: &LocalFfmpegInfo,
    first_queued: Option<&QueuedFile>,
    locked: bool,
) {
    let first_media = first_queued.and_then(|q| q.media.as_ref());

    ui.add_enabled_ui(!locked, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                render_mode_selector(ui, lang, settings, first_media);
                ui.add_space(8.0);
                render_preset_section(ui, lang, settings, first_media);
                ui.add_space(8.0);
                render_container_output_section(ui, lang, settings, first_queued);
                ui.add_space(8.0);

                match settings.mode {
                    ConvertMode::Video => {
                        render_video_section(ui, lang, settings, ffmpeg, first_media);
                        ui.add_space(8.0);
                        render_audio_section(ui, lang, settings, ffmpeg);
                    }
                    ConvertMode::AudioOnly => {
                        render_audio_section(ui, lang, settings, ffmpeg);
                    }
                    ConvertMode::ImageGif => {
                        render_image_gif_section(ui, lang, settings);
                    }
                    ConvertMode::Subtitle => {
                        render_subtitle_section(ui, lang);
                    }
                    ConvertMode::Remux => {
                        render_remux_section(ui, lang);
                    }
                }

                ui.add_space(8.0);
                render_safety_section(ui, lang, settings);
                ui.add_space(8.0);
                render_command_preview(ui, lang, settings, first_queued);
            });
    });
}

fn render_mode_selector(
    ui: &mut egui::Ui,
    lang: Language,
    settings: &mut ConversionSettings,
    first_media: Option<&MediaInfo>,
) {
    let t = tr(lang);
    card_frame().show(ui, |ui| {
        ui.label(RichText::new(t.mode_section()).strong());
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for mode in ConvertMode::ALL {
                let selected = settings.mode == *mode;
                let compat = check_mode_compatibility(*mode, first_media);
                let btn = Button::new(RichText::new(mode_label(lang, *mode)).strong())
                    .selected(selected)
                    .fill(if selected {
                        ACCENT_BLUE
                    } else {
                        egui::Color32::from_rgb(35, 42, 58)
                    });
                let resp = ui.add_enabled(compat.is_ok(), btn);
                if let Err(reason) = compat {
                    resp.on_disabled_hover_text(incompatibility_tooltip(lang, reason));
                } else if resp.clicked() {
                    settings.set_mode(*mode);
                }
            }
        });
    });
}

fn render_preset_section(
    ui: &mut egui::Ui,
    lang: Language,
    settings: &mut ConversionSettings,
    first_media: Option<&MediaInfo>,
) {
    let t = tr(lang);
    card_frame().show(ui, |ui| {
        ui.label(RichText::new(t.quick_presets()).strong());
        ui.add_space(4.0);
        egui::ComboBox::from_id_salt("preset_combo")
            .width(ui.available_width())
            .selected_text(t.choose_preset_placeholder())
            .show_ui(ui, |ui| {
                for preset in PresetProfile::ALL {
                    let preview = ConversionSettings::from_preset(*preset);
                    let compat = check_mode_compatibility(preview.mode, first_media);
                    ui.add_enabled_ui(compat.is_ok(), |ui| {
                        let resp = ui.selectable_label(false, preset_label(lang, *preset));
                        if let Err(reason) = compat {
                            resp.on_disabled_hover_text(incompatibility_tooltip(lang, reason));
                        } else if resp.clicked() {
                            let out_dir = settings.custom_output_dir.clone();
                            let prefix = settings.output_prefix.clone();
                            let suffix = settings.output_suffix.clone();
                            let overwrite = settings.safety.overwrite_existing;
                            let deep = settings.safety.deep_decode_verify;
                            *settings = preview;
                            settings.custom_output_dir = out_dir;
                            settings.output_prefix = prefix;
                            settings.output_suffix = suffix;
                            settings.safety.overwrite_existing = overwrite;
                            settings.safety.deep_decode_verify = deep;
                        }
                    });
                }
            });
    });
}

fn render_container_output_section(
    ui: &mut egui::Ui,
    lang: Language,
    settings: &mut ConversionSettings,
    first_queued: Option<&QueuedFile>,
) {
    let t = tr(lang);
    let first_media = first_queued.and_then(|q| q.media.as_ref());

    card_frame().show(ui, |ui| {
        ui.label(RichText::new(t.format_output_section()).strong());
        ui.add_space(4.0);

        ui.label(RichText::new(t.target_format()).size(12.0).color(TEXT_MUTED));
        ui.add_space(2.0);

        let from_ext = first_queued
            .and_then(|q| q.input_path.extension())
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_uppercase())
            .unwrap_or_else(|| "ANY".to_string());

        ui.horizontal_wrapped(|ui| {
            status_pill(
                ui,
                &format!("{}: {from_ext}", t.from_label()),
                egui::Color32::from_rgb(100, 116, 139),
            );
            ui.label(RichText::new("->").strong().color(TEXT_MUTED));

            let arrow = if settings.format_picker_open { "▲" } else { "▾" };
            let to_btn = Button::new(
                RichText::new(format!(
                    "{}: {} {arrow}",
                    t.to_label(),
                    settings.container.short_name()
                ))
                .strong(),
            )
            .fill(ACCENT_BLUE);
            if ui
                .add(to_btn)
                .on_hover_text(container_label(lang, settings.container))
                .clicked()
            {
                settings.format_picker_open = !settings.format_picker_open;
            }
        });

        ui.add_space(2.0);
        ui.add(
            Label::new(
                RichText::new(container_label(lang, settings.container))
                    .size(11.5)
                    .color(TEXT_MUTED),
            )
            .wrap(),
        );

        if settings.format_picker_open {
            ui.add_space(6.0);
            render_format_picker_popover(ui, lang, settings, first_media);
        }

        ui.add_space(6.0);
        ui.label(
            RichText::new(t.global_affix_label())
                .size(12.0)
                .color(TEXT_MUTED),
        );
        ui.horizontal_wrapped(|ui| {
            ui.add(
                TextEdit::singleline(&mut settings.output_prefix)
                    .desired_width(120.0)
                    .hint_text(t.prefix_hint()),
            );
            ui.add(
                TextEdit::singleline(&mut settings.output_suffix)
                    .desired_width(120.0)
                    .hint_text(t.suffix_hint()),
            );
        });

        ui.add_space(6.0);
        ui.label(RichText::new(t.output_folder()).size(12.0).color(TEXT_MUTED));
        let folder_label = settings
            .custom_output_dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| t.same_folder_as_source().to_string());
        ui.add(Label::new(RichText::new(folder_label).size(11.5)).wrap());

        ui.horizontal_wrapped(|ui| {
            if ui.button(t.choose_folder()).clicked()
                && let Some(dir) = rfd::FileDialog::new().pick_folder()
            {
                settings.custom_output_dir = Some(dir);
            }
            if settings.custom_output_dir.is_some() && ui.button(t.reset_to_source()).clicked() {
                settings.custom_output_dir = None;
            }
        });
    });
}

fn render_format_picker_popover(
    ui: &mut egui::Ui,
    lang: Language,
    settings: &mut ConversionSettings,
    first_media: Option<&MediaInfo>,
) {
    let t = tr(lang);
    Frame::new()
        .fill(BG_PANEL)
        .stroke(Stroke::new(1.0, BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            // Search box
            ui.horizontal(|ui| {
                let clear_w = if settings.format_search_query.is_empty() {
                    0.0
                } else {
                    26.0
                };
                let w = (ui.available_width() - clear_w).max(120.0);
                ui.add(
                    TextEdit::singleline(&mut settings.format_search_query)
                        .desired_width(w)
                        .hint_text(t.search_format_hint()),
                );
                if !settings.format_search_query.is_empty() && ui.small_button("×").clicked() {
                    settings.format_search_query.clear();
                }
            });

            ui.add_space(6.0);

            // Category tabs
            ui.horizontal_wrapped(|ui| {
                for mode in ConvertMode::ALL {
                    let selected = settings.mode == *mode;
                    let compat = check_mode_compatibility(*mode, first_media);
                    let tab_btn = Button::new(RichText::new(mode_label(lang, *mode)).size(11.5))
                        .selected(selected)
                        .fill(if selected { ACCENT_BLUE } else { BG_MAIN });
                    let resp = ui.add_enabled(compat.is_ok(), tab_btn);
                    if let Err(reason) = compat {
                        resp.on_disabled_hover_text(incompatibility_tooltip(lang, reason));
                    } else if resp.clicked() {
                        settings.set_mode(*mode);
                    }
                }
            });

            ui.add_space(6.0);

            let q = settings.format_search_query.trim().to_ascii_lowercase();
            let candidates: Vec<ContainerFormat> = if q.is_empty() {
                settings.mode.containers().to_vec()
            } else {
                ContainerFormat::ALL
                    .iter()
                    .copied()
                    .filter(|c| {
                        c.short_name().to_ascii_lowercase().contains(&q)
                            || c.extension().to_ascii_lowercase().contains(&q)
                            || container_label(lang, *c).to_ascii_lowercase().contains(&q)
                    })
                    .collect()
            };

            let col_w = ((ui.available_width() - 24.0) / 4.0).max(56.0);
            egui::ScrollArea::vertical()
                .max_height(190.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    egui::Grid::new("format_picker_grid")
                        .num_columns(4)
                        .spacing(egui::vec2(6.0, 6.0))
                        .show(ui, |ui| {
                            for (idx, c) in candidates.iter().enumerate() {
                                let target_mode = if settings.mode == ConvertMode::Remux
                                    && ConvertMode::Remux.containers().contains(c)
                                {
                                    ConvertMode::Remux
                                } else {
                                    c.default_mode()
                                };
                                let compat = check_mode_compatibility(target_mode, first_media);
                                let selected =
                                    settings.container == *c && settings.mode == target_mode;
                                let btn = Button::new(RichText::new(c.short_name()).strong().size(11.5))
                                    .min_size(egui::vec2(col_w, 26.0))
                                    .fill(if selected { ACCENT_BLUE } else { BG_MAIN });

                                let resp = ui.add_enabled(compat.is_ok(), btn);
                                if let Err(reason) = compat {
                                    resp.on_disabled_hover_text(incompatibility_tooltip(
                                        lang, reason,
                                    ));
                                } else {
                                    let hovered = resp.on_hover_text(container_label(lang, *c));
                                    if hovered.clicked() {
                                        if settings.mode != target_mode {
                                            settings.set_mode(target_mode);
                                        }
                                        settings.container = *c;
                                        settings.enforce_compatibility();
                                        settings.format_picker_open = false;
                                        settings.format_search_query.clear();
                                    }
                                }

                                if (idx + 1) % 4 == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                });
        });
}

fn render_video_section(
    ui: &mut egui::Ui,
    lang: Language,
    settings: &mut ConversionSettings,
    ffmpeg: &LocalFfmpegInfo,
    first_media: Option<&MediaInfo>,
) {
    let t = tr(lang);
    card_frame().show(ui, |ui| {
        ui.label(RichText::new(t.video_params_section()).strong());
        ui.add_space(4.0);

        if first_media.is_some_and(|m| m.is_still_image && !m.has_audio) {
            ui.add(
                egui::Slider::new(&mut settings.image_to_video_duration_secs, 1..=60)
                    .text(t.image_duration_label()),
            );
            ui.add_space(6.0);
        }

        ui.label(RichText::new(t.video_codec()).size(12.0).color(TEXT_MUTED));
        egui::ComboBox::from_id_salt("vcodec_combo")
            .width(ui.available_width())
            .selected_text(video_codec_label(lang, settings.video_codec))
            .show_ui(ui, |ui| {
                for vc in VideoCodec::ALL {
                    if !vc.is_compatible_with(settings.container) {
                        continue;
                    }
                    let supported = ffmpeg.supports_encoder(vc.ffmpeg_name());
                    let label = video_codec_label(lang, *vc);
                    let text = if supported {
                        label.to_string()
                    } else {
                        format!("{label} {}", t.unavailable_tag())
                    };
                    ui.add_enabled_ui(supported, |ui| {
                        ui.selectable_value(&mut settings.video_codec, *vc, text);
                    });
                }
            });

        if settings.video_codec == VideoCodec::Copy {
            ui.add(
                Label::new(
                    RichText::new(t.stream_copy_video_note())
                        .size(11.5)
                        .color(TEXT_MUTED),
                )
                .wrap(),
            );
            return;
        }

        ui.add_space(6.0);
        ui.label(RichText::new(t.encoder_speed()).size(12.0).color(TEXT_MUTED));
        egui::ComboBox::from_id_salt("speed_combo")
            .width(ui.available_width())
            .selected_text(encode_speed_label(lang, settings.speed))
            .show_ui(ui, |ui| {
                for sp in EncodeSpeed::ALL {
                    ui.selectable_value(
                        &mut settings.speed,
                        *sp,
                        encode_speed_label(lang, *sp),
                    );
                }
            });

        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            let is_crf = matches!(settings.rate_control, VideoRateControl::Quality(_));
            if ui.radio(is_crf, t.rate_crf()).clicked() && !is_crf {
                settings.rate_control = VideoRateControl::Quality(20);
            }
            if ui.radio(!is_crf, t.rate_bitrate()).clicked() && is_crf {
                settings.rate_control = VideoRateControl::BitrateKbps(6000);
            }
        });

        match &mut settings.rate_control {
            VideoRateControl::Quality(q) => {
                let mut val = *q as u32;
                ui.add(egui::Slider::new(&mut val, 12..=40).text(t.crf_slider_label()));
                *q = val as u8;
            }
            VideoRateControl::BitrateKbps(kbps) => {
                ui.add(egui::Slider::new(kbps, 500..=50_000).text(t.kbps_unit()));
            }
        }

        ui.add_space(6.0);
        ui.label(RichText::new(t.pixel_format()).size(12.0).color(TEXT_MUTED));
        egui::ComboBox::from_id_salt("pixfmt_combo")
            .width(ui.available_width())
            .selected_text(pixel_format_label(lang, settings.pixel_format))
            .show_ui(ui, |ui| {
                for pf in PixelFormat::ALL {
                    ui.selectable_value(
                        &mut settings.pixel_format,
                        *pf,
                        pixel_format_label(lang, *pf),
                    );
                }
            });

        ui.add_space(6.0);
        ui.label(RichText::new(t.resolution()).size(12.0).color(TEXT_MUTED));
        egui::ComboBox::from_id_salt("res_combo")
            .width(ui.available_width())
            .selected_text(resolution_label(lang, settings.resolution))
            .show_ui(ui, |ui| {
                for res in ResolutionOption::PRESETS {
                    ui.selectable_value(
                        &mut settings.resolution,
                        *res,
                        resolution_label(lang, *res),
                    );
                }
                let is_custom = matches!(settings.resolution, ResolutionOption::Custom { .. });
                if ui.selectable_label(is_custom, t.custom_wxh()).clicked() && !is_custom {
                    settings.resolution = ResolutionOption::Custom {
                        width: 1920,
                        height: 1080,
                    };
                }
            });

        if let ResolutionOption::Custom { width, height } = &mut settings.resolution {
            ui.horizontal(|ui| {
                ui.label(t.width_short());
                ui.add(egui::DragValue::new(width).range(64..=7680).speed(2));
                ui.label(t.height_short());
                ui.add(egui::DragValue::new(height).range(64..=4320).speed(2));
            });
        }

        ui.add_space(6.0);
        ui.label(RichText::new(t.frame_rate()).size(12.0).color(TEXT_MUTED));
        egui::ComboBox::from_id_salt("fps_combo")
            .width(ui.available_width())
            .selected_text(fps_label(lang, settings.fps))
            .show_ui(ui, |ui| {
                for f in FpsOption::ALL {
                    ui.selectable_value(&mut settings.fps, *f, fps_label(lang, *f));
                }
            });
    });
}

fn render_image_gif_section(ui: &mut egui::Ui, lang: Language, settings: &mut ConversionSettings) {
    let t = tr(lang);
    card_frame().show(ui, |ui| {
        ui.label(RichText::new(t.image_gif_section()).strong());
        ui.add_space(4.0);

        if settings.container == ContainerFormat::Webp {
            ui.checkbox(
                &mut settings.image_gif.webp_animated,
                t.animated_webp_toggle(),
            );
            ui.add_space(4.0);
        }

        let is_animated = matches!(
            settings.container,
            ContainerFormat::Gif | ContainerFormat::Apng
        ) || (settings.container == ContainerFormat::Webp && settings.image_gif.webp_animated);

        if is_animated {
            ui.add(egui::Slider::new(&mut settings.image_gif.fps, 1..=30).text(t.fps_label()));
        }

        ui.checkbox(
            &mut settings.image_gif.limit_max_width,
            t.limit_max_width(),
        );
        if settings.image_gif.limit_max_width {
            ui.add(
                egui::Slider::new(&mut settings.image_gif.max_width, 64..=3840)
                    .text(t.max_width_px()),
            );
        }

        ui.add_space(4.0);
        match settings.container {
            ContainerFormat::Gif => {
                ui.add(
                    Label::new(
                        RichText::new(t.gif_palette_note())
                            .size(11.5)
                            .color(TEXT_MUTED),
                    )
                    .wrap(),
                );
            }
            ContainerFormat::Webp | ContainerFormat::Jpg => {
                ui.add(
                    egui::Slider::new(&mut settings.image_gif.quality, 10..=100)
                        .text(t.quality_label()),
                );
            }
            ContainerFormat::Png => {
                let mut level = settings.image_gif.png_compression as u32;
                ui.add(egui::Slider::new(&mut level, 0..=9).text(t.png_compression_label()));
                settings.image_gif.png_compression = level as u8;
            }
            ContainerFormat::Avif => {
                let mut crf = settings.image_gif.avif_crf as u32;
                ui.add(egui::Slider::new(&mut crf, 10..=55).text(t.avif_crf_label()));
                settings.image_gif.avif_crf = crf as u8;
            }
            _ => {}
        }
    });
}

fn render_subtitle_section(ui: &mut egui::Ui, lang: Language) {
    let t = tr(lang);
    card_frame().show(ui, |ui| {
        ui.label(RichText::new(t.subtitle_section()).strong());
        ui.add_space(4.0);
        ui.add(
            Label::new(
                RichText::new(t.subtitle_description())
                    .size(12.0)
                    .color(TEXT_MUTED),
            )
            .wrap(),
        );
    });
}

fn render_remux_section(ui: &mut egui::Ui, lang: Language) {
    let t = tr(lang);
    card_frame().show(ui, |ui| {
        ui.label(RichText::new(t.remux_section()).strong());
        ui.add_space(4.0);
        ui.add(
            Label::new(
                RichText::new(t.remux_description())
                    .size(12.0)
                    .color(TEXT_MUTED),
            )
            .wrap(),
        );
    });
}

fn render_audio_section(
    ui: &mut egui::Ui,
    lang: Language,
    settings: &mut ConversionSettings,
    ffmpeg: &LocalFfmpegInfo,
) {
    let t = tr(lang);
    card_frame().show(ui, |ui| {
        ui.label(RichText::new(t.audio_params_section()).strong());
        ui.add_space(4.0);

        if settings.mode == ConvertMode::AudioOnly {
            ui.add(
                Label::new(
                    RichText::new(t.audio_strip_note())
                        .size(11.5)
                        .color(TEXT_MUTED),
                )
                .wrap(),
            );
            ui.add_space(4.0);
        }

        ui.label(RichText::new(t.audio_codec()).size(12.0).color(TEXT_MUTED));
        let prev = settings.audio_codec;
        egui::ComboBox::from_id_salt("acodec_combo")
            .width(ui.available_width())
            .selected_text(audio_codec_label(lang, settings.audio_codec))
            .show_ui(ui, |ui| {
                for ac in AudioCodec::ALL {
                    if !ac.is_compatible_with(settings.container) {
                        continue;
                    }
                    let supported =
                        *ac == AudioCodec::None || ffmpeg.supports_encoder(ac.ffmpeg_name());
                    ui.add_enabled_ui(supported, |ui| {
                        ui.selectable_value(
                            &mut settings.audio_codec,
                            *ac,
                            audio_codec_label(lang, *ac),
                        );
                    });
                }
            });
        if settings.audio_codec != prev {
            settings.enforce_compatibility();
        }

        if matches!(settings.audio_codec, AudioCodec::Copy | AudioCodec::None) {
            return;
        }

        if !settings.audio_codec.is_lossless() {
            ui.add_space(4.0);
            ui.add(
                egui::Slider::new(&mut settings.audio_bitrate_kbps, 64..=320)
                    .step_by(32.0)
                    .text(t.bitrate_kbps()),
            );
        }

        ui.add_space(4.0);
        ui.label(RichText::new(t.sample_rate()).size(12.0).color(TEXT_MUTED));
        egui::ComboBox::from_id_salt("sr_combo")
            .width(ui.available_width())
            .selected_text(audio_sample_rate_label(lang, settings.audio_sample_rate))
            .show_ui(ui, |ui| {
                for sr in AudioSampleRate::ALL {
                    if settings.audio_codec == AudioCodec::LibOpus
                        && *sr == AudioSampleRate::Hz44100
                    {
                        continue;
                    }
                    ui.selectable_value(
                        &mut settings.audio_sample_rate,
                        *sr,
                        audio_sample_rate_label(lang, *sr),
                    );
                }
            });

        ui.add_space(4.0);
        ui.label(RichText::new(t.channels()).size(12.0).color(TEXT_MUTED));
        egui::ComboBox::from_id_salt("ch_combo")
            .width(ui.available_width())
            .selected_text(audio_channels_label(lang, settings.audio_channels))
            .show_ui(ui, |ui| {
                for ch in AudioChannels::ALL {
                    ui.selectable_value(
                        &mut settings.audio_channels,
                        *ch,
                        audio_channels_label(lang, *ch),
                    );
                }
            });
    });
}

fn render_safety_section(ui: &mut egui::Ui, lang: Language, settings: &mut ConversionSettings) {
    let t = tr(lang);
    card_frame().show(ui, |ui| {
        ui.label(RichText::new(t.safety_section()).strong());
        ui.add_space(4.0);

        if settings.container.supports_faststart() {
            ui.checkbox(&mut settings.safety.faststart, t.guard_faststart());
        }
        if settings.mode == ConvertMode::Video {
            ui.checkbox(
                &mut settings.safety.even_dimensions_guard,
                t.guard_even_dims(),
            );
        }
        if !matches!(settings.mode, ConvertMode::ImageGif | ConvertMode::Subtitle) {
            ui.checkbox(&mut settings.safety.repair_timestamps, t.guard_repair_ts());
        }
        ui.checkbox(&mut settings.safety.deep_decode_verify, t.guard_deep_verify());
        ui.checkbox(&mut settings.safety.preserve_metadata, t.guard_preserve_meta());
        if matches!(settings.mode, ConvertMode::Video | ConvertMode::Remux) {
            ui.checkbox(&mut settings.safety.keep_subtitles, t.guard_keep_subs());
        }
        ui.checkbox(&mut settings.safety.overwrite_existing, t.guard_overwrite());
    });
}

fn render_command_preview(
    ui: &mut egui::Ui,
    lang: Language,
    settings: &ConversionSettings,
    first_queued: Option<&QueuedFile>,
) {
    let t = tr(lang);
    card_frame().show(ui, |ui| {
        ui.label(RichText::new(t.command_preview_section()).strong());
        ui.add_space(4.0);

        let sample_in = first_queued
            .map(|q| q.input_path.as_path())
            .unwrap_or_else(|| Path::new("input.mkv"));
        let stem = first_queued
            .map(|q| q.custom_stem.as_str())
            .unwrap_or("output");
        let sample_out = compose_output_filename(
            stem,
            &settings.output_prefix,
            &settings.output_suffix,
            settings.container.extension(),
        );
        let media = first_queued.and_then(|q| q.media.as_ref());

        match build_command_plan(settings, media, sample_in, Path::new(&sample_out)) {
            Ok(plan) => {
                let preview = plan.display_preview();
                ui.add(Label::new(RichText::new(preview).monospace().size(11.0).color(TEXT_MUTED)).wrap());
                for note in &plan.notes {
                    ui.add(
                        Label::new(
                            RichText::new(format!("{} {note}", t.note_prefix()))
                                .size(11.0)
                                .color(ACCENT_AMBER),
                        )
                        .wrap(),
                    );
                }
            }
            Err(err) => {
                ui.add(Label::new(RichText::new(err).size(11.5).color(ACCENT_AMBER)).wrap());
            }
        }
    });
}
