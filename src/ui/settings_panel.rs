use crate::ffmpeg::command_builder::build_command_plan;
use crate::i18n::{
    Language, audio_channels_label, audio_codec_label, audio_sample_rate_label, container_label,
    encode_speed_label, fps_label, mode_label, pixel_format_label, preset_label, resolution_label,
    tr, video_codec_label,
};
use crate::models::config::{
    AudioChannels, AudioCodec, AudioSampleRate, ContainerFormat, ConvertMode, ConversionSettings,
    EncodeSpeed, FpsOption, PixelFormat, PresetProfile, ResolutionOption, VideoCodec,
    VideoRateControl,
};
use crate::models::media::{LocalFfmpegInfo, QueuedFile};
use crate::security::path_guard::compose_output_filename;
use crate::ui::theme::{ACCENT_AMBER, ACCENT_BLUE, TEXT_MUTED, card_frame};
use eframe::egui::{self, Button, Label, RichText, TextEdit};
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
    ui.add_enabled_ui(!locked, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                render_mode_selector(ui, lang, settings);
                ui.add_space(8.0);
                render_preset_section(ui, lang, settings);
                ui.add_space(8.0);
                render_container_output_section(ui, lang, settings);
                ui.add_space(8.0);

                match settings.mode {
                    ConvertMode::Video => {
                        render_video_section(ui, lang, settings, ffmpeg);
                        ui.add_space(8.0);
                        render_audio_section(ui, lang, settings, ffmpeg);
                    }
                    ConvertMode::AudioOnly => {
                        render_audio_section(ui, lang, settings, ffmpeg);
                    }
                    ConvertMode::ImageGif => {
                        render_image_gif_section(ui, lang, settings);
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

fn render_mode_selector(ui: &mut egui::Ui, lang: Language, settings: &mut ConversionSettings) {
    let t = tr(lang);
    card_frame().show(ui, |ui| {
        ui.label(RichText::new(t.mode_section()).strong());
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for mode in ConvertMode::ALL {
                let selected = settings.mode == *mode;
                let btn = Button::new(RichText::new(mode_label(lang, *mode)).strong())
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

fn render_preset_section(ui: &mut egui::Ui, lang: Language, settings: &mut ConversionSettings) {
    let t = tr(lang);
    card_frame().show(ui, |ui| {
        ui.label(RichText::new(t.quick_presets()).strong());
        ui.add_space(4.0);
        egui::ComboBox::from_id_salt("preset_combo")
            .width(ui.available_width())
            .selected_text(t.choose_preset_placeholder())
            .show_ui(ui, |ui| {
                for preset in PresetProfile::ALL {
                    if ui.selectable_label(false, preset_label(lang, *preset)).clicked() {
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

fn render_container_output_section(
    ui: &mut egui::Ui,
    lang: Language,
    settings: &mut ConversionSettings,
) {
    let t = tr(lang);
    card_frame().show(ui, |ui| {
        ui.label(RichText::new(t.format_output_section()).strong());
        ui.add_space(4.0);

        ui.label(RichText::new(t.target_format()).size(12.0).color(TEXT_MUTED));
        let prev = settings.container;
        egui::ComboBox::from_id_salt("container_combo")
            .width(ui.available_width())
            .selected_text(container_label(lang, settings.container))
            .show_ui(ui, |ui| {
                for c in settings.mode.containers() {
                    ui.selectable_value(&mut settings.container, *c, container_label(lang, *c));
                }
            });
        if settings.container != prev {
            settings.enforce_compatibility();
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

fn render_video_section(
    ui: &mut egui::Ui,
    lang: Language,
    settings: &mut ConversionSettings,
    ffmpeg: &LocalFfmpegInfo,
) {
    let t = tr(lang);
    card_frame().show(ui, |ui| {
        ui.label(RichText::new(t.video_params_section()).strong());
        ui.add_space(4.0);

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

        let is_animated = settings.container == ContainerFormat::Gif
            || (settings.container == ContainerFormat::Webp && settings.image_gif.webp_animated);

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
        if settings.mode != ConvertMode::ImageGif {
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
