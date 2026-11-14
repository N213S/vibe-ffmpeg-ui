use crate::ffmpeg::command_builder::build_command_plan;
use crate::models::config::{
    AudioChannels, AudioCodec, AudioSampleRate, ContainerFormat, ConvertMode, ConversionSettings,
    EncodeSpeed, FpsOption, PixelFormat, PresetProfile, ResolutionOption, VideoCodec,
    VideoRateControl,
};
use crate::models::media::{LocalFfmpegInfo, QueuedFile};
use crate::security::path_guard::compose_output_filename;
use crate::ui::theme::{ACCENT_AMBER, ACCENT_BLUE, TEXT_MUTED, card_frame};
use eframe::egui::{self, Button, RichText, TextEdit};
use std::path::Path;

/// Renders the right-hand multi-mode encoding parameters, output naming, safety guards, and command preview.
pub fn render_settings_panel(
    ui: &mut egui::Ui,
    settings: &mut ConversionSettings,
    ffmpeg: &LocalFfmpegInfo,
    first_queued: Option<&QueuedFile>,
    locked: bool,
) {
    ui.add_enabled_ui(!locked, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                render_mode_selector(ui, settings);
                ui.add_space(8.0);
                render_preset_section(ui, settings);
                ui.add_space(8.0);
                render_container_output_section(ui, settings);
                ui.add_space(8.0);

                match settings.mode {
                    ConvertMode::Video => {
                        render_video_section(ui, settings, ffmpeg);
                        ui.add_space(8.0);
                        render_audio_section(ui, settings, ffmpeg);
                    }
                    ConvertMode::AudioOnly => {
                        render_audio_section(ui, settings, ffmpeg);
                    }
                    ConvertMode::ImageGif => {
                        render_image_gif_section(ui, settings);
                    }
                    ConvertMode::Remux => {
                        render_remux_section(ui);
                    }
                }

                ui.add_space(8.0);
                render_safety_section(ui, settings);
                ui.add_space(8.0);
                render_command_preview(ui, settings, first_queued);
            });
    });
}

fn render_mode_selector(ui: &mut egui::Ui, settings: &mut ConversionSettings) {
    card_frame().show(ui, |ui| {
        ui.label(RichText::new("Conversion Mode").strong());
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for mode in ConvertMode::ALL {
                let selected = settings.mode == *mode;
                let btn = Button::new(RichText::new(mode.label()).strong())
                    .selected(selected)
                    .fill(if selected {
                        ACCENT_BLUE
                    } else {
                        egui::Color32::from_rgb(35, 42, 58)
                    });
                if ui.add(btn).clicked() {
                    settings.set_mode(*mode);
                }
            }
        });
    });
}

fn render_preset_section(ui: &mut egui::Ui, settings: &mut ConversionSettings) {
    card_frame().show(ui, |ui| {
        ui.label(RichText::new("Quick Presets").strong());
        ui.add_space(4.0);
        egui::ComboBox::from_id_salt("preset_combo")
            .width(ui.available_width())
            .selected_text("Choose a curated mode preset...")
            .show_ui(ui, |ui| {
                for preset in PresetProfile::ALL {
                    if ui.selectable_label(false, preset.label()).clicked() {
                        let out_dir = settings.custom_output_dir.clone();
                        let prefix = settings.output_prefix.clone();
                        let suffix = settings.output_suffix.clone();
                        let overwrite = settings.safety.overwrite_existing;
                        let deep = settings.safety.deep_decode_verify;
                        *settings = ConversionSettings::from_preset(*preset);
                        settings.custom_output_dir = out_dir;
                        settings.output_prefix = prefix;
                        settings.output_suffix = suffix;
                        settings.safety.overwrite_existing = overwrite;
                        settings.safety.deep_decode_verify = deep;
                    }
                }
            });
    });
}

fn render_container_output_section(ui: &mut egui::Ui, settings: &mut ConversionSettings) {
    card_frame().show(ui, |ui| {
        ui.label(RichText::new("Format & Output").strong());
        ui.add_space(4.0);

        ui.label(RichText::new("Target Format").size(12.0).color(TEXT_MUTED));
        let prev = settings.container;
        egui::ComboBox::from_id_salt("container_combo")
            .width(ui.available_width())
            .selected_text(settings.container.label())
            .show_ui(ui, |ui| {
                for c in settings.mode.containers() {
                    ui.selectable_value(&mut settings.container, *c, c.label());
                }
            });
        if settings.container != prev {
            settings.enforce_compatibility();
        }

        ui.add_space(6.0);
        ui.label(
            RichText::new("Global Filename Prefix / Suffix")
                .size(12.0)
                .color(TEXT_MUTED),
        );
        ui.horizontal(|ui| {
            ui.add(
                TextEdit::singleline(&mut settings.output_prefix)
                    .desired_width(130.0)
                    .hint_text("Prefix (e.g. out_)"),
            );
            ui.add(
                TextEdit::singleline(&mut settings.output_suffix)
                    .desired_width(130.0)
                    .hint_text("Suffix (e.g. _1080p)"),
            );
        });

        ui.add_space(6.0);
        ui.label(RichText::new("Output Folder").size(12.0).color(TEXT_MUTED));
        let folder_label = settings
            .custom_output_dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "Same folder as each input file".to_string());
        ui.label(RichText::new(folder_label).size(11.5));

        ui.horizontal(|ui| {
            if ui.button("Choose Folder...").clicked()
                && let Some(dir) = rfd::FileDialog::new().pick_folder()
            {
                settings.custom_output_dir = Some(dir);
            }
            if settings.custom_output_dir.is_some() && ui.button("Reset to Source").clicked() {
                settings.custom_output_dir = None;
            }
        });
    });
}

fn render_video_section(
    ui: &mut egui::Ui,
    settings: &mut ConversionSettings,
    ffmpeg: &LocalFfmpegInfo,
) {
    card_frame().show(ui, |ui| {
        ui.label(RichText::new("Video Parameters").strong());
        ui.add_space(4.0);

        ui.label(RichText::new("Video Codec").size(12.0).color(TEXT_MUTED));
        egui::ComboBox::from_id_salt("vcodec_combo")
            .width(ui.available_width())
            .selected_text(settings.video_codec.label())
            .show_ui(ui, |ui| {
                for vc in VideoCodec::ALL {
                    if !vc.is_compatible_with(settings.container) {
                        continue;
                    }
                    let supported = ffmpeg.supports_encoder(vc.ffmpeg_name());
                    let text = if supported {
                        vc.label().to_string()
                    } else {
                        format!("{} [Unavailable]", vc.label())
                    };
                    ui.add_enabled_ui(supported, |ui| {
                        ui.selectable_value(&mut settings.video_codec, *vc, text);
                    });
                }
            });

        if settings.video_codec == VideoCodec::Copy {
            ui.label(
                RichText::new("Stream Copy preserves original video bitstream without re-encoding.")
                    .size(11.5)
                    .color(TEXT_MUTED),
            );
            return;
        }

        ui.add_space(6.0);
        ui.label(RichText::new("Encoder Speed").size(12.0).color(TEXT_MUTED));
        egui::ComboBox::from_id_salt("speed_combo")
            .width(ui.available_width())
            .selected_text(settings.speed.label())
            .show_ui(ui, |ui| {
                for sp in EncodeSpeed::ALL {
                    ui.selectable_value(&mut settings.speed, *sp, sp.label());
                }
            });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let is_crf = matches!(settings.rate_control, VideoRateControl::Quality(_));
            if ui.radio(is_crf, "Constant Quality (CRF/CQ)").clicked() && !is_crf {
                settings.rate_control = VideoRateControl::Quality(20);
            }
            if ui.radio(!is_crf, "Target Bitrate").clicked() && is_crf {
                settings.rate_control = VideoRateControl::BitrateKbps(6000);
            }
        });

        match &mut settings.rate_control {
            VideoRateControl::Quality(q) => {
                let mut val = *q as u32;
                ui.add(egui::Slider::new(&mut val, 12..=40).text("CRF (Lower = Better)"));
                *q = val as u8;
            }
            VideoRateControl::BitrateKbps(kbps) => {
                ui.add(egui::Slider::new(kbps, 500..=50_000).text("kbps"));
            }
        }

        ui.add_space(6.0);
        ui.label(RichText::new("Pixel Format").size(12.0).color(TEXT_MUTED));
        egui::ComboBox::from_id_salt("pixfmt_combo")
            .width(ui.available_width())
            .selected_text(settings.pixel_format.label())
            .show_ui(ui, |ui| {
                for pf in PixelFormat::ALL {
                    ui.selectable_value(&mut settings.pixel_format, *pf, pf.label());
                }
            });

        ui.add_space(6.0);
        ui.label(RichText::new("Resolution").size(12.0).color(TEXT_MUTED));
        egui::ComboBox::from_id_salt("res_combo")
            .width(ui.available_width())
            .selected_text(settings.resolution.label())
            .show_ui(ui, |ui| {
                for res in ResolutionOption::PRESETS {
                    ui.selectable_value(&mut settings.resolution, *res, res.label());
                }
                let is_custom = matches!(settings.resolution, ResolutionOption::Custom { .. });
                if ui.selectable_label(is_custom, "Custom WxH").clicked() && !is_custom {
                    settings.resolution = ResolutionOption::Custom {
                        width: 1920,
                        height: 1080,
                    };
                }
            });

        if let ResolutionOption::Custom { width, height } = &mut settings.resolution {
            ui.horizontal(|ui| {
                ui.label("W:");
                ui.add(egui::DragValue::new(width).range(64..=7680).speed(2));
                ui.label("H:");
                ui.add(egui::DragValue::new(height).range(64..=4320).speed(2));
            });
        }

        ui.add_space(6.0);
        ui.label(RichText::new("Frame Rate").size(12.0).color(TEXT_MUTED));
        egui::ComboBox::from_id_salt("fps_combo")
            .width(ui.available_width())
            .selected_text(settings.fps.label())
            .show_ui(ui, |ui| {
                for f in FpsOption::ALL {
                    ui.selectable_value(&mut settings.fps, *f, f.label());
                }
            });
    });
}

fn render_image_gif_section(ui: &mut egui::Ui, settings: &mut ConversionSettings) {
    card_frame().show(ui, |ui| {
        ui.label(RichText::new("Image / GIF Parameters").strong());
        ui.add_space(4.0);

        if settings.container == ContainerFormat::Webp {
            ui.checkbox(
                &mut settings.image_gif.webp_animated,
                "Animated WebP (Multi-frame loop)",
            );
            ui.add_space(4.0);
        }

        let is_animated = settings.container == ContainerFormat::Gif
            || (settings.container == ContainerFormat::Webp && settings.image_gif.webp_animated);

        if is_animated {
            ui.add(egui::Slider::new(&mut settings.image_gif.fps, 1..=30).text("FPS"));
        }

        ui.checkbox(
            &mut settings.image_gif.limit_max_width,
            "Limit Max Width (preserve aspect ratio)",
        );
        if settings.image_gif.limit_max_width {
            ui.add(
                egui::Slider::new(&mut settings.image_gif.max_width, 64..=3840)
                    .text("Max Width (px)"),
            );
        }

        ui.add_space(4.0);
        match settings.container {
            ContainerFormat::Gif => {
                ui.label(
                    RichText::new("Uses two-stage palettegen + paletteuse for crisp colors.")
                        .size(11.5)
                        .color(TEXT_MUTED),
                );
            }
            ContainerFormat::Webp | ContainerFormat::Jpg => {
                ui.add(
                    egui::Slider::new(&mut settings.image_gif.quality, 10..=100)
                        .text("Quality"),
                );
            }
            ContainerFormat::Png => {
                let mut level = settings.image_gif.png_compression as u32;
                ui.add(egui::Slider::new(&mut level, 0..=9).text("Compression Level"));
                settings.image_gif.png_compression = level as u8;
            }
            ContainerFormat::Avif => {
                let mut crf = settings.image_gif.avif_crf as u32;
                ui.add(egui::Slider::new(&mut crf, 10..=55).text("AVIF CRF (Lower = Better)"));
                settings.image_gif.avif_crf = crf as u8;
            }
            _ => {}
        }
    });
}

fn render_remux_section(ui: &mut egui::Ui) {
    card_frame().show(ui, |ui| {
        ui.label(RichText::new("Remux (Fast Stream Copy)").strong());
        ui.add_space(4.0);
        ui.label(
            RichText::new(
                "Copies existing video & audio bitstreams (-c copy) into the target container with zero quality loss and near-instant speed.",
            )
            .size(12.0)
            .color(TEXT_MUTED),
        );
    });
}

fn render_audio_section(
    ui: &mut egui::Ui,
    settings: &mut ConversionSettings,
    ffmpeg: &LocalFfmpegInfo,
) {
    card_frame().show(ui, |ui| {
        ui.label(RichText::new("Audio Parameters").strong());
        ui.add_space(4.0);

        if settings.mode == ConvertMode::AudioOnly {
            ui.label(
                RichText::new("Video tracks are automatically stripped (-vn).")
                    .size(11.5)
                    .color(TEXT_MUTED),
            );
            ui.add_space(4.0);
        }

        ui.label(RichText::new("Audio Codec").size(12.0).color(TEXT_MUTED));
        let prev = settings.audio_codec;
        egui::ComboBox::from_id_salt("acodec_combo")
            .width(ui.available_width())
            .selected_text(settings.audio_codec.label())
            .show_ui(ui, |ui| {
                for ac in AudioCodec::ALL {
                    if !ac.is_compatible_with(settings.container) {
                        continue;
                    }
                    let supported =
                        *ac == AudioCodec::None || ffmpeg.supports_encoder(ac.ffmpeg_name());
                    ui.add_enabled_ui(supported, |ui| {
                        ui.selectable_value(&mut settings.audio_codec, *ac, ac.label());
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
                    .text("Bitrate (kbps)"),
            );
        }

        ui.add_space(4.0);
        ui.label(RichText::new("Sample Rate").size(12.0).color(TEXT_MUTED));
        egui::ComboBox::from_id_salt("sr_combo")
            .width(ui.available_width())
            .selected_text(settings.audio_sample_rate.label())
            .show_ui(ui, |ui| {
                for sr in AudioSampleRate::ALL {
                    if settings.audio_codec == AudioCodec::LibOpus
                        && *sr == AudioSampleRate::Hz44100
                    {
                        continue;
                    }
                    ui.selectable_value(&mut settings.audio_sample_rate, *sr, sr.label());
                }
            });

        ui.add_space(4.0);
        ui.label(RichText::new("Channels").size(12.0).color(TEXT_MUTED));
        egui::ComboBox::from_id_salt("ch_combo")
            .width(ui.available_width())
            .selected_text(settings.audio_channels.label())
            .show_ui(ui, |ui| {
                for ch in AudioChannels::ALL {
                    ui.selectable_value(&mut settings.audio_channels, *ch, ch.label());
                }
            });
    });
}

fn render_safety_section(ui: &mut egui::Ui, settings: &mut ConversionSettings) {
    card_frame().show(ui, |ui| {
        ui.label(RichText::new("Integrity & Anti-Corruption Guards").strong());
        ui.add_space(4.0);

        if settings.container.supports_faststart() {
            ui.checkbox(
                &mut settings.safety.faststart,
                "MP4/MOV/M4A +faststart (relocate moov atom)",
            );
        }
        if settings.mode == ConvertMode::Video {
            ui.checkbox(
                &mut settings.safety.even_dimensions_guard,
                "Even dimensions guard (prevent yuv420p crash)",
            );
        }
        if settings.mode != ConvertMode::ImageGif {
            ui.checkbox(
                &mut settings.safety.repair_timestamps,
                "Repair timestamps (+genpts / avoid_negative_ts)",
            );
        }
        ui.checkbox(
            &mut settings.safety.deep_decode_verify,
            "Mode-aware deep verify (ffprobe + decode check)",
        );
        ui.checkbox(&mut settings.safety.preserve_metadata, "Preserve metadata");
        if matches!(settings.mode, ConvertMode::Video | ConvertMode::Remux) {
            ui.checkbox(
                &mut settings.safety.keep_subtitles,
                "Keep text subtitles (safe mov_text/copy)",
            );
        }
        ui.checkbox(
            &mut settings.safety.overwrite_existing,
            "Overwrite existing output files",
        );
    });
}

fn render_command_preview(
    ui: &mut egui::Ui,
    settings: &ConversionSettings,
    first_queued: Option<&QueuedFile>,
) {
    card_frame().show(ui, |ui| {
        ui.label(RichText::new("Live Complete Command Preview").strong());
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
                ui.label(RichText::new(preview).monospace().size(11.0).color(TEXT_MUTED));
                for note in &plan.notes {
                    ui.label(RichText::new(format!("Note: {note}")).size(11.0).color(ACCENT_AMBER));
                }
            }
            Err(err) => {
                ui.label(RichText::new(err).size(11.5).color(ACCENT_AMBER));
            }
        }
    });
}
