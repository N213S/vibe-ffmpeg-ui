use crate::ffmpeg::discovery::discover_local_ffmpeg;
use crate::ffmpeg::probe::probe_media;
use crate::ffmpeg::runner::{BatchJobItem, spawn_batch_conversion};
use crate::i18n::{Language, tr};
use crate::models::config::ConversionSettings;
use crate::models::media::{
    FileStatus, LocalFfmpegInfo, ProgressStats, QueuedFile, UpdateCheckState, WorkerEvent,
};
use crate::net::ffmpeg_update::spawn_update_check;
use crate::security::path_guard::{
    AUDIO_EXTENSIONS, IMAGE_EXTENSIONS, VIDEO_EXTENSIONS, is_supported_media_extension,
    paths_refer_to_same_file, sanitize_file_stem, validate_input_file,
};
use crate::ui::language_window::render_language_window;
use crate::ui::log_panel::render_log_panel;
use crate::ui::queue_panel::render_queue_panel;
use crate::ui::settings_panel::render_settings_panel;
use crate::ui::theme::{BG_PANEL, BORDER_SUBTLE, configure_style_and_fonts};
use crate::ui::top_bar::render_top_bar;
use eframe::egui::{self, CentralPanel, Frame, Margin, Panel, Stroke};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

const MAX_LOG_LINES: usize = 400;

/// Main desktop application state.
pub struct FfmpegStudioApp {
    ffmpeg_info: LocalFfmpegInfo,
    language: Language,
    language_window_open: bool,
    language_search: String,
    settings: ConversionSettings,
    queue: Vec<QueuedFile>,
    next_file_id: u64,
    is_running: bool,
    cancel_flag: Arc<AtomicBool>,
    update_state: UpdateCheckState,
    logs: VecDeque<String>,
    log_expanded: bool,
    tx: Sender<WorkerEvent>,
    rx: Receiver<WorkerEvent>,
}

impl FfmpegStudioApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_style_and_fonts(&cc.egui_ctx);
        let ffmpeg_info = discover_local_ffmpeg();
        let (tx, rx) = mpsc::channel();
        let language = Language::default();
        let t = tr(language);

        let mut logs = VecDeque::with_capacity(64);
        if !ffmpeg_info.raw_version_line.is_empty() {
            logs.push_back(format!("[Init] {}", ffmpeg_info.raw_version_line));
            logs.push_back(format!(
                "{} {}",
                t.log_init_encoders(),
                ffmpeg_info.available_encoders.len()
            ));
        } else {
            logs.push_back(t.log_init_missing().to_string());
        }

        Self {
            ffmpeg_info,
            language,
            language_window_open: false,
            language_search: String::new(),
            settings: ConversionSettings::default(),
            queue: Vec::new(),
            next_file_id: 1,
            is_running: false,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            update_state: UpdateCheckState::Idle,
            logs,
            log_expanded: false,
            tx,
            rx,
        }
    }

    fn push_log(&mut self, line: String) {
        if self.logs.len() >= MAX_LOG_LINES {
            self.logs.pop_front();
        }
        self.logs.push_back(line);
    }

    fn enqueue_paths(&mut self, paths: Vec<PathBuf>, ctx: &egui::Context, filter_dropped: bool) {
        let t = tr(self.language);
        for raw_path in paths {
            if filter_dropped && !is_supported_media_extension(&raw_path) {
                self.push_log(format!("{} {}", t.log_skipped_ext(), raw_path.display()));
                continue;
            }

            let valid_path = match validate_input_file(&raw_path) {
                Ok(p) => p,
                Err(err) => {
                    self.push_log(format!("{} {err}", t.log_skipped_prefix()));
                    continue;
                }
            };

            if self
                .queue
                .iter()
                .any(|existing| paths_refer_to_same_file(&existing.input_path, &valid_path))
            {
                continue;
            }

            let file_id = self.next_file_id;
            self.next_file_id += 1;

            let default_stem = valid_path
                .file_stem()
                .map(|s| sanitize_file_stem(&s.to_string_lossy()))
                .unwrap_or_else(|| "output".to_string());

            self.queue.push(QueuedFile {
                id: file_id,
                input_path: valid_path.clone(),
                custom_stem: default_stem,
                media: None,
                probe_error: None,
                status: FileStatus::Probing,
            });

            if let Some(ffprobe) = self.ffmpeg_info.ffprobe_path.clone() {
                let tx = self.tx.clone();
                let ctx_clone = ctx.clone();
                thread::spawn(move || {
                    let res = probe_media(&ffprobe, &valid_path);
                    let _ = tx.send(WorkerEvent::ProbeCompleted {
                        file_id,
                        result: res,
                    });
                    ctx_clone.request_repaint();
                });
            } else if let Some(item) = self.queue.iter_mut().find(|f| f.id == file_id) {
                item.status = FileStatus::Failed(t.ffprobe_not_found().to_string());
            }
        }
    }

    fn open_file_picker(&mut self, ctx: &egui::Context) {
        let t = tr(self.language);
        let mut all_media = Vec::with_capacity(
            VIDEO_EXTENSIONS.len() + AUDIO_EXTENSIONS.len() + IMAGE_EXTENSIONS.len(),
        );
        all_media.extend_from_slice(VIDEO_EXTENSIONS);
        all_media.extend_from_slice(AUDIO_EXTENSIONS);
        all_media.extend_from_slice(IMAGE_EXTENSIONS);

        if let Some(files) = rfd::FileDialog::new()
            .add_filter(t.file_filter_all_media(), &all_media)
            .add_filter(t.file_filter_video(), VIDEO_EXTENSIONS)
            .add_filter(t.file_filter_audio(), AUDIO_EXTENSIONS)
            .add_filter(t.file_filter_image(), IMAGE_EXTENSIONS)
            .add_filter(t.file_filter_all_files(), &["*"])
            .pick_files()
        {
            self.enqueue_paths(files, ctx, false);
        }
    }

    fn start_batch(&mut self, ctx: &egui::Context) {
        if self.is_running || !self.ffmpeg_info.is_ready() {
            return;
        }

        let items: Vec<BatchJobItem> = self
            .queue
            .iter()
            .filter(|q| {
                matches!(
                    q.status,
                    FileStatus::Ready | FileStatus::Failed(_) | FileStatus::Cancelled
                )
            })
            .map(|q| BatchJobItem {
                file_id: q.id,
                input_path: q.input_path.clone(),
                custom_stem: q.custom_stem.clone(),
                media: q.media.clone(),
            })
            .collect();

        if items.is_empty() {
            return;
        }

        self.cancel_flag = Arc::new(AtomicBool::new(false));
        self.is_running = true;

        spawn_batch_conversion(
            self.ffmpeg_info.clone(),
            items,
            self.settings.clone(),
            Arc::clone(&self.cancel_flag),
            self.tx.clone(),
            ctx.clone(),
        );
    }

    fn drain_worker_events(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                WorkerEvent::ProbeCompleted { file_id, result } => {
                    if let Some(item) = self.queue.iter_mut().find(|q| q.id == file_id) {
                        match result {
                            Ok(info) => {
                                item.media = Some(info);
                                item.probe_error = None;
                                if matches!(item.status, FileStatus::Probing) {
                                    item.status = FileStatus::Ready;
                                }
                            }
                            Err(err) => {
                                item.probe_error = Some(err.clone());
                                item.status = FileStatus::Failed(err);
                            }
                        }
                    }
                }
                WorkerEvent::FileStarted { file_id } => {
                    if let Some(item) = self.queue.iter_mut().find(|q| q.id == file_id) {
                        item.status = FileStatus::Converting(ProgressStats::default());
                    }
                }
                WorkerEvent::FileProgress { file_id, stats } => {
                    if let Some(item) = self.queue.iter_mut().find(|q| q.id == file_id) {
                        item.status = FileStatus::Converting(stats);
                    }
                }
                WorkerEvent::FileVerifying { file_id } => {
                    if let Some(item) = self.queue.iter_mut().find(|q| q.id == file_id) {
                        item.status = FileStatus::Verifying;
                    }
                }
                WorkerEvent::FileCompleted {
                    file_id,
                    output_path,
                    report,
                } => {
                    if let Some(item) = self.queue.iter_mut().find(|q| q.id == file_id) {
                        item.status = FileStatus::Done {
                            output_path,
                            report,
                        };
                    }
                }
                WorkerEvent::FileFailed { file_id, error } => {
                    if let Some(item) = self.queue.iter_mut().find(|q| q.id == file_id) {
                        item.status = FileStatus::Failed(error);
                    }
                }
                WorkerEvent::FileCancelled { file_id } => {
                    if let Some(item) = self.queue.iter_mut().find(|q| q.id == file_id) {
                        item.status = FileStatus::Cancelled;
                    }
                }
                WorkerEvent::LogLine(line) => {
                    self.push_log(line);
                }
                WorkerEvent::BatchFinished => {
                    self.is_running = false;
                }
                WorkerEvent::UpdateChecked(res) => match res {
                    Ok(info) => {
                        let t = tr(self.language);
                        let local_str = info
                            .local_version
                            .as_deref()
                            .unwrap_or(t.log_unknown_version());
                        self.push_log(format!(
                            "{} {} ({local_str})",
                            t.log_update_ok(),
                            info.latest_release,
                        ));
                        self.update_state = UpdateCheckState::Completed(info);
                    }
                    Err(err) => {
                        let t = tr(self.language);
                        self.push_log(format!("{} {err}", t.log_update_err()));
                        self.update_state = UpdateCheckState::Error(err);
                    }
                },
            }
        }
    }

    fn handle_drag_drop(&mut self, ui: &egui::Ui) -> bool {
        let (dropped, hovering) = ui.input(|i| {
            let dropped: Vec<PathBuf> = i
                .raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .collect();
            let hovering = !i.raw.hovered_files.is_empty();
            (dropped, hovering)
        });

        if !dropped.is_empty() && !self.is_running {
            self.enqueue_paths(dropped, ui.ctx(), true);
        }
        hovering
    }
}

impl Drop for FfmpegStudioApp {
    fn drop(&mut self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }
}

impl eframe::App for FfmpegStudioApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.drain_worker_events();
        let hovering = self.handle_drag_drop(ui);

        let has_pending_work = self.queue.iter().any(|q| {
            matches!(
                q.status,
                FileStatus::Ready | FileStatus::Failed(_) | FileStatus::Cancelled
            )
        });

        let panel_frame = Frame::new()
            .fill(BG_PANEL)
            .stroke(Stroke::new(1.0, BORDER_SUBTLE))
            .inner_margin(Margin::same(12));

        let top_actions = Panel::top("top_toolbar")
            .frame(panel_frame)
            .show(ui, |ui| {
                render_top_bar(
                    ui,
                    self.language,
                    &self.ffmpeg_info,
                    &self.update_state,
                    self.is_running,
                    has_pending_work,
                    self.queue.len(),
                )
            })
            .inner;

        if top_actions.add_files {
            self.open_file_picker(ui.ctx());
        }
        if top_actions.clear_done {
            self.queue
                .retain(|q| !matches!(q.status, FileStatus::Done { .. }));
        }
        if top_actions.clear_all && !self.is_running {
            self.queue.clear();
        }
        if top_actions.start_batch {
            self.start_batch(ui.ctx());
        }
        if top_actions.cancel_batch {
            let t = tr(self.language);
            self.cancel_flag.store(true, Ordering::Relaxed);
            self.push_log(t.log_batch_cancel().to_string());
        }
        if top_actions.check_updates {
            self.update_state = UpdateCheckState::Checking;
            spawn_update_check(
                self.ffmpeg_info.semantic_version.clone(),
                self.tx.clone(),
                ui.ctx().clone(),
            );
        }
        if top_actions.toggle_language {
            self.language_window_open = !self.language_window_open;
            if self.language_window_open {
                self.language_search.clear();
            }
        }

        let mut clear_log = false;
        Panel::bottom("bottom_logs")
            .frame(panel_frame)
            .show(ui, |ui| {
                render_log_panel(
                    ui,
                    self.language,
                    &self.logs,
                    &mut self.log_expanded,
                    &mut clear_log,
                );
            });
        if clear_log {
            self.logs.clear();
        }

        let first_queued = self.queue.first().cloned();
        Panel::right("right_settings")
            .resizable(true)
            .default_size(400.0)
            .min_size(340.0)
            .max_size(520.0)
            .frame(panel_frame)
            .show(ui, |ui| {
                render_settings_panel(
                    ui,
                    self.language,
                    &mut self.settings,
                    &self.ffmpeg_info,
                    first_queued.as_ref(),
                    self.is_running,
                );
            });

        let queue_actions = CentralPanel::default()
            .frame(Frame::central_panel(&ui.style()).inner_margin(Margin::same(12)))
            .show(ui, |ui| {
                render_queue_panel(
                    ui,
                    self.language,
                    &mut self.queue,
                    &self.settings,
                    self.is_running,
                    hovering,
                )
            })
            .inner;

        if let Some(id) = queue_actions.remove_file_id {
            self.queue.retain(|q| q.id != id);
        }
        if queue_actions.add_files_clicked && !self.is_running {
            self.open_file_picker(ui.ctx());
        }

        render_language_window(
            ui.ctx(),
            &mut self.language,
            &mut self.language_window_open,
            &mut self.language_search,
        );
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }
}
