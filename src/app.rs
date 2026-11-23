use crate::ffmpeg::discovery::discover_local_ffmpeg;
use crate::ffmpeg::installer::{find_winget_exe, spawn_ffmpeg_rescan, spawn_winget_install};
use crate::ffmpeg::probe::probe_media;
use crate::ffmpeg::runner::{BatchJobItem, spawn_batch_conversion};
use crate::i18n::{Language, tr};
use crate::models::config::{ConversionSettings, ConvertMode, check_mode_compatibility};
use crate::models::media::{
    AppUpdateState, AppWorkspaceMode, FfmpegInstallState, FileStatus, LocalFfmpegInfo,
    ProgressStats, QueuedFile, UpdateCheckState, WorkerEvent,
};
use crate::net::app_update::spawn_app_update_check;
use crate::net::ffmpeg_update::{is_remote_newer, spawn_update_check};
use crate::security::path_guard::{
    AUDIO_EXTENSIONS, IMAGE_EXTENSIONS, SUBTITLE_EXTENSIONS, VIDEO_EXTENSIONS,
    is_supported_media_extension, load_saved_language, paths_refer_to_same_file, sanitize_file_stem,
    validate_input_file,
};
use crate::ui::ffmpeg_setup_window::render_ffmpeg_setup_window;
use crate::ui::inspector_panel::render_inspector_panel;
use crate::ui::language_window::render_language_window;
use crate::ui::log_panel::{render_log_panel, render_log_window};
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
use std::time::{Duration, Instant};

const MAX_LOG_LINES: usize = 400;
const MAX_INSTALL_LOG_LINES: usize = 120;
const STARTUP_SETUP_DELAY: Duration = Duration::from_millis(300);

/// Main desktop application state.
pub struct FfmpegStudioApp {
    ffmpeg_info: LocalFfmpegInfo,
    winget_available: bool,
    setup_window_open: bool,
    focus_setup_once: bool,
    pending_startup_setup_at: Option<Instant>,
    startup_frames: u32,
    install_state: FfmpegInstallState,
    install_logs: Vec<String>,
    language: Language,
    language_window_open: bool,
    language_search: String,
    settings: ConversionSettings,
    workspace_mode: AppWorkspaceMode,
    selected_file_id: Option<u64>,
    inspector_raw_json_open: bool,
    queue: Vec<QueuedFile>,
    next_file_id: u64,
    is_running: bool,
    cancel_flag: Arc<AtomicBool>,
    app_update_state: AppUpdateState,
    update_state: UpdateCheckState,
    logs: VecDeque<String>,
    log_expanded: bool,
    log_window_open: bool,
    tx: Sender<WorkerEvent>,
    rx: Receiver<WorkerEvent>,
}

impl FfmpegStudioApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_style_and_fonts(&cc.egui_ctx);
        let ffmpeg_info = discover_local_ffmpeg();
        let winget_available = find_winget_exe().is_some();
        let pending_startup_setup_at = if ffmpeg_info.is_ready() {
            None
        } else {
            Some(Instant::now())
        };
        let (tx, rx) = mpsc::channel();
        let language = load_saved_language();
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
            winget_available,
            setup_window_open: false,
            focus_setup_once: false,
            pending_startup_setup_at,
            startup_frames: 0,
            install_state: FfmpegInstallState::Idle,
            install_logs: Vec::new(),
            language,
            language_window_open: false,
            language_search: String::new(),
            settings: ConversionSettings::default(),
            workspace_mode: AppWorkspaceMode::Convert,
            selected_file_id: None,
            inspector_raw_json_open: false,
            queue: Vec::new(),
            next_file_id: 1,
            is_running: false,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            app_update_state: AppUpdateState::Idle,
            update_state: UpdateCheckState::Idle,
            logs,
            log_expanded: false,
            log_window_open: false,
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
            if self.selected_file_id.is_none() {
                self.selected_file_id = Some(file_id);
            }

            self.spawn_probe_worker(file_id, valid_path, ctx);
        }
    }

    fn sync_selected_file(&mut self) {
        if let Some(id) = self.selected_file_id
            && self.queue.iter().any(|q| q.id == id)
        {
            return;
        }
        self.selected_file_id = self.queue.first().map(|q| q.id);
    }

    fn spawn_probe_worker(&mut self, file_id: u64, input_path: PathBuf, ctx: &egui::Context) {
        let t = tr(self.language);
        if let Some(ffprobe) = self.ffmpeg_info.ffprobe_path.clone() {
            let tx = self.tx.clone();
            let ctx_clone = ctx.clone();
            thread::spawn(move || {
                let res = probe_media(&ffprobe, &input_path);
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

    fn reprobe_single(&mut self, file_id: u64, ctx: &egui::Context) {
        let Some(input_path) = self
            .queue
            .iter_mut()
            .find(|q| q.id == file_id)
            .map(|item| {
                item.media = None;
                item.probe_error = None;
                item.status = FileStatus::Probing;
                item.input_path.clone()
            })
        else {
            return;
        };
        self.spawn_probe_worker(file_id, input_path, ctx);
    }

    fn reprobe_all(&mut self, ctx: &egui::Context) {
        let targets: Vec<(u64, PathBuf)> = self
            .queue
            .iter_mut()
            .map(|item| {
                item.media = None;
                item.probe_error = None;
                item.status = FileStatus::Probing;
                (item.id, item.input_path.clone())
            })
            .collect();
        for (file_id, path) in targets {
            self.spawn_probe_worker(file_id, path, ctx);
        }
    }

    fn open_file_picker(&mut self, ctx: &egui::Context) {
        let t = tr(self.language);
        let mut all_media = Vec::with_capacity(
            VIDEO_EXTENSIONS.len()
                + AUDIO_EXTENSIONS.len()
                + IMAGE_EXTENSIONS.len()
                + SUBTITLE_EXTENSIONS.len(),
        );
        all_media.extend_from_slice(VIDEO_EXTENSIONS);
        all_media.extend_from_slice(AUDIO_EXTENSIONS);
        all_media.extend_from_slice(IMAGE_EXTENSIONS);
        all_media.extend_from_slice(SUBTITLE_EXTENSIONS);

        if let Some(files) = rfd::FileDialog::new()
            .add_filter(t.file_filter_all_media(), &all_media)
            .add_filter(t.file_filter_video(), VIDEO_EXTENSIONS)
            .add_filter(t.file_filter_audio(), AUDIO_EXTENSIONS)
            .add_filter(t.file_filter_image(), IMAGE_EXTENSIONS)
            .add_filter(t.file_filter_subtitle(), SUBTITLE_EXTENSIONS)
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
                    let first_id = self.queue.first().map(|q| q.id);
                    if let Some(item) = self.queue.iter_mut().find(|q| q.id == file_id) {
                        match result {
                            Ok(info) => {
                                if first_id == Some(file_id)
                                    && check_mode_compatibility(self.settings.mode, Some(&info))
                                        .is_err()
                                    && let Some(&fallback_mode) = ConvertMode::ALL.iter().find(|&&m| {
                                        check_mode_compatibility(m, Some(&info)).is_ok()
                                    })
                                {
                                    self.settings.set_mode(fallback_mode);
                                }
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
                WorkerEvent::AppUpdateChecked(res) => match res {
                    Ok(info) => {
                        self.push_log(format!(
                            "[App Update] GitHub latest: v{} (current: v{})",
                            info.latest_version, info.current_version
                        ));
                        self.app_update_state = AppUpdateState::Completed(info);
                    }
                    Err(err) => {
                        let t = tr(self.language);
                        self.push_log(format!("{} {err}", t.log_update_err()));
                        self.app_update_state = AppUpdateState::Error(err);
                    }
                },
                WorkerEvent::FfmpegInstallProgress(line) => {
                    if let FfmpegInstallState::Installing {
                        ref mut latest_line,
                    } = self.install_state
                    {
                        *latest_line = line.clone();
                    }
                    if self.install_logs.len() >= MAX_INSTALL_LOG_LINES {
                        self.install_logs.remove(0);
                    }
                    self.install_logs.push(line.clone());
                    self.push_log(format!("[WinGet] {line}"));
                }
                WorkerEvent::FfmpegInstallFinished(res) => match res {
                    Ok(()) => {
                        self.install_state = FfmpegInstallState::Succeeded;
                        self.push_log("[WinGet] Installation completed.".to_string());
                    }
                    Err(err) => {
                        self.push_log(format!("[WinGet Error] {err}"));
                        self.install_state = FfmpegInstallState::Failed(err);
                    }
                },
                WorkerEvent::FfmpegRediscovered(info) => {
                    let t = tr(self.language);
                    if info.is_ready() {
                        if !info.raw_version_line.is_empty() {
                            self.push_log(format!("[Init] {}", info.raw_version_line));
                            self.push_log(format!(
                                "{} {}",
                                t.log_init_encoders(),
                                info.available_encoders.len()
                            ));
                        }
                        if let UpdateCheckState::Completed(ref mut upd) = self.update_state
                            && upd.local_version.is_none()
                        {
                            upd.local_version = info.semantic_version.clone();
                            upd.update_available = upd
                                .local_version
                                .as_deref()
                                .is_some_and(|l| is_remote_newer(l, &upd.latest_release));
                        }
                    } else {
                        self.push_log(t.log_init_missing().to_string());
                    }
                    self.ffmpeg_info = info;
                }
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
        self.startup_frames = self.startup_frames.saturating_add(1);
        if let Some(started) = self.pending_startup_setup_at {
            if self.ffmpeg_info.is_ready() {
                self.pending_startup_setup_at = None;
            } else if self.startup_frames >= 3 && started.elapsed() >= STARTUP_SETUP_DELAY {
                self.pending_startup_setup_at = None;
                self.setup_window_open = true;
                self.focus_setup_once = true;
            } else {
                ui.ctx().request_repaint_after(Duration::from_millis(60));
            }
        }
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
                    &self.app_update_state,
                    self.workspace_mode,
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
            self.sync_selected_file();
        }
        if top_actions.clear_all && !self.is_running {
            self.queue.clear();
            self.sync_selected_file();
        }
        if let Some(mode) = top_actions.set_workspace_mode {
            self.workspace_mode = mode;
            self.sync_selected_file();
        }
        if top_actions.reprobe_all && !self.is_running {
            self.reprobe_all(ui.ctx());
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
            self.app_update_state = AppUpdateState::Checking;
            spawn_app_update_check(self.tx.clone(), ui.ctx().clone());
        }
        if top_actions.toggle_language {
            self.language_window_open = !self.language_window_open;
            if self.language_window_open {
                self.language_search.clear();
            }
        }
        if top_actions.open_setup {
            self.pending_startup_setup_at = None;
            self.setup_window_open = true;
            self.focus_setup_once = true;
        }

        let mut clear_log = false;
        let bottom_panel = if self.log_expanded {
            Panel::bottom("bottom_logs_expanded")
                .resizable(true)
                .default_size(220.0)
                .min_size(110.0)
                .max_size(480.0)
                .frame(panel_frame)
        } else {
            Panel::bottom("bottom_logs_collapsed")
                .resizable(false)
                .frame(panel_frame)
        };
        bottom_panel.show(ui, |ui| {
            render_log_panel(
                ui,
                self.language,
                &self.logs,
                &mut self.log_expanded,
                &mut self.log_window_open,
                &mut clear_log,
            );
        });

        let first_queued = self.queue.first().cloned();
        match self.workspace_mode {
            AppWorkspaceMode::Convert => {
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
            }
            AppWorkspaceMode::Inspect => {
                let selected_item = self
                    .selected_file_id
                    .and_then(|id| self.queue.iter().find(|q| q.id == id))
                    .or_else(|| self.queue.first())
                    .cloned();
                let mut reprobe_selected = false;
                Panel::right("right_inspector")
                    .resizable(true)
                    .default_size(480.0)
                    .min_size(360.0)
                    .max_size(680.0)
                    .frame(panel_frame)
                    .show(ui, |ui| {
                        reprobe_selected = render_inspector_panel(
                            ui,
                            self.language,
                            selected_item.as_ref(),
                            &mut self.inspector_raw_json_open,
                        );
                    });
                if reprobe_selected
                    && let Some(item) = selected_item
                {
                    self.reprobe_single(item.id, ui.ctx());
                }
            }
        }

        let queue_actions = CentralPanel::default()
            .frame(Frame::central_panel(&ui.style()).inner_margin(Margin::same(12)))
            .show(ui, |ui| {
                render_queue_panel(
                    ui,
                    self.language,
                    &mut self.queue,
                    &self.settings,
                    self.workspace_mode,
                    self.selected_file_id,
                    self.is_running,
                    hovering,
                )
            })
            .inner;

        if let Some(id) = queue_actions.remove_file_id {
            self.queue.retain(|q| q.id != id);
            self.sync_selected_file();
        }
        if let Some(id) = queue_actions.select_file_id {
            self.selected_file_id = Some(id);
        }
        if let Some(id) = queue_actions.inspect_file_id {
            self.selected_file_id = Some(id);
            self.workspace_mode = AppWorkspaceMode::Inspect;
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
        render_log_window(
            ui.ctx(),
            self.language,
            &self.logs,
            &mut self.log_window_open,
            &mut clear_log,
        );
        let setup_actions = render_ffmpeg_setup_window(
            ui.ctx(),
            self.language,
            &mut self.setup_window_open,
            &mut self.focus_setup_once,
            &self.ffmpeg_info,
            &self.install_state,
            &self.install_logs,
            &self.update_state,
            self.winget_available,
        );
        if setup_actions.install_winget
            && !matches!(self.install_state, FfmpegInstallState::Installing { .. })
        {
            let t = tr(self.language);
            self.install_logs.clear();
            self.install_state = FfmpegInstallState::Installing {
                latest_line: t.setup_winget_installing().to_string(),
            };
            spawn_winget_install(self.tx.clone(), ui.ctx().clone());
        }
        if setup_actions.check_official
            && !matches!(self.update_state, UpdateCheckState::Checking)
        {
            self.update_state = UpdateCheckState::Checking;
            spawn_update_check(
                self.ffmpeg_info.semantic_version.clone(),
                self.tx.clone(),
                ui.ctx().clone(),
            );
        }
        if setup_actions.rescan_ffmpeg {
            spawn_ffmpeg_rescan(self.tx.clone(), ui.ctx().clone());
        }
        if clear_log {
            self.logs.clear();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }
}
