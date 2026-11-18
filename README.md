# Vibe FFmpeg ⚡

> **Built with AI Vibe Coding 🤖✨** — This project was created and iterated through AI-assisted vibe coding (prompt-driven engineering in Rust).
>
> Built purely to scratch my own itch and fit my personal workflow—open-sourced in case others find it useful too!

Lightweight, native multi-mode desktop GUI for FFmpeg written in Rust (`eframe` / `egui` 0.37 + `rfd` 0.18), engineered for speed, complete parameter generation, custom output naming, and mode-aware integrity verification.

![Vibe FFmpeg UI](assets/screenshot.png)

## Supported Platforms

- **Windows 10 / 11 (64-bit `x86_64`)**: Primary supported and tested platform.
- **Linux / macOS**: Currently **untested and unsupported** (UI font discovery reads from `C:\Windows\Fonts`, file reveal uses `explorer.exe /select,`, and TLS uses Windows `native-tls` / SChannel).

## Installation & Setup (Windows)

No installer is needed — just `ffmpeg-ui.exe` plus `ffmpeg.exe` and `ffprobe.exe`:

1. **Download Vibe FFmpeg**: Grab `ffmpeg-ui.exe` from the [GitHub Releases](https://github.com/N213S/vibe-ffmpeg-ui/releases) page (or compile from source below).
2. **Install FFmpeg + ffprobe** *(both binaries are required)* using either option:
   - **Option A — Portable (Zero `PATH` setup, easiest)**:
     1. Download an official FFmpeg Windows build from [gyan.dev/ffmpeg/builds](https://www.gyan.dev/ffmpeg/builds/) (recommended: `ffmpeg-release-full.7z`).
     2. Extract and copy **`ffmpeg.exe`** and **`ffprobe.exe`** (from the `bin\` folder) into the **same folder** next to `ffmpeg-ui.exe`. Vibe FFmpeg checks sibling executables first.
   - **Option B — System `PATH` (via `winget` or Manual Setup)**:
     - **Via `winget`**: Run `winget install Gyan.FFmpeg` in PowerShell/Terminal, then **close and reopen** Vibe FFmpeg so it picks up the updated `PATH`.
     - **Manual `PATH`**: Extract FFmpeg to a folder (e.g. `C:\ffmpeg`), open **Start** -> search **"Edit the system environment variables"** -> **Environment Variables** -> edit **Path** -> add `C:\ffmpeg\bin` -> click **OK**, then relaunch Vibe FFmpeg.
3. **Bypass Windows SmartScreen (Unsigned Executable Notice)**:
   - Because `ffmpeg-ui.exe` is an unsigned indie open-source binary without a paid EV code-signing certificate, Windows SmartScreen (*"Windows protected your PC"*) or antivirus heuristics may flag it on first launch.
   - Make sure you downloaded it from [`N213S/vibe-ffmpeg-ui/releases`](https://github.com/N213S/vibe-ffmpeg-ui/releases), then click **More info** -> **Run anyway** (or inspect the Rust source and build locally with `cargo build --release`).

## Privacy & Offline Guarantee

- **100% Local Processing**: Media probing (`ffprobe`), conversion (`ffmpeg`), atomic temporary staging, and output verification run completely offline on your machine. **No media files, filenames, usage analytics, or telemetry are ever uploaded to any cloud server.** The only file saved locally is your UI language preference (`%APPDATA%\vibe-ffmpeg\preferences.json`).
- **Minimal Network Usage (Manual Update Check Only)**: The app never phones home on startup. Outbound HTTPS connections occur **only** when you click **Check Update** in the top toolbar, querying strict allowlisted `https://www.gyan.dev/ffmpeg/builds/release-version` and `git-version` endpoints (read limit capped at 2 KB, 8-second timeout via Windows `native-tls`).

## Features & Architecture

- **53 Output Formats, 17 Quick Presets & Searchable `FROM -> TO` Picker (`ConvertMode`)**:
  - **Video (17 formats)**: `MP4`, `MKV`, `MOV`, `WEBM`, `AVI`, `FLV`, `TS`, `M2TS`, `MXF`, `3GP`, `M4V`, `MPG`, `MPEG`, `VOB`, `OGV`, `WMV`, `DV` with software/hardware encoders (`libx264`, `libx265`, `libsvtav1`, `libvpx-vp9`, `mpeg2video`, `libtheora`, `wmv2`, `dvvideo`, NVENC, AMF, QSV).
  - **Audio Only (18 formats)**: `MP3`, `WAV`, `FLAC`, `AAC`, `M4A`, `OGG`, `OPUS`, `WMA`, `AIFF`, `AC3`, `EAC3`, `DTS`, `ALAC`, `AMR`, `CAF`, `MKA`, `MP2`, `WV` with automatic `-vn` video stripping.
  - **Image / Animated (13 formats)**: `PNG`, `JPG`, `WEBP`, `GIF` (`palettegen` + `paletteuse`), `AVIF`, `APNG`, `BMP`, `TIFF`, `TGA`, `ICO` (auto-capped `<=256x256`), `JXL`, `EXR`, `HDR`.
  - **Subtitle (5 formats)**: `SRT`, `ASS`, `SSA`, `VTT`, `LRC` extraction and cross-conversion (`-vn -an`).
  - **Remux (Fast Copy, 10 containers)**: Container switching (`-c copy`) for `MP4`, `MKV`, `MOV`, `WEBM`, `AVI`, `FLV`, `TS`, `M2TS`, `M4V`, `MKA` without re-encoding.
  - **Expanded Input Support**: Accepts legacy, camera, audio, RAW/HDR image, and subtitle formats (`.rm`, `.rmvb`, `.heic`, `.heif`, `.ape`, `.mod`, `.tod`, `.bik`, `.mts`, `.m2ts`, `.vob`, `.asf`, `.dv`, `.swf`, `.mka`, `.amr`, `.caf`, `.wv`, `.tta`, `.tga`, `.exr`, `.hdr`, `.lrc`, `.srt`, `.ass`, `.ssa`, `.vtt`, `.sub`, etc.) via `rfd` filters (including `*` All Files) and drag-and-drop.
- **Stream-Driven Compatibility & Cross-Category Conversions**:
  - `ffprobe` inspects queued files for video, audio, subtitle (`has_text_subtitles`), embedded cover art, and still images (`is_still_image`), automatically switching an incompatible mode when the first file finishes probing and disabling unsupported categories/presets with hover tooltips.
  - Supports **Image/GIF -> Video** (`-loop 1` + configurable duration + even-dimension scaling), **Audio -> Video** (`showwaves` 720p visualizer), **Audio -> Image** (embedded cover art extraction or `showwavespic`), **Video -> Static/Animated Image**, and **Video/Subtitle -> Subtitle**.
- **Custom Output Filenames & Collision Safety**:
  - Per-file editable output base name in the queue with auto-synced extension when switching formats, plus optional global Prefix/Suffix.
  - Automatic `_out` / `(N)` collision protection so an active input file is never overwritten in-place.
- **Atomic Staging & Mode-Aware Verification**:
  - Stages output to `.<stem>.<pid>_<job_id>.tmp.<ext>` with explicit `-f <muxer>` flags, verifies streams/duration/subtitles via `ffprobe` + deep decode check (`ffmpeg -v error -xerror -f null -`, `RIFF`/`WEBP` header check for animated WebP, or `ffprobe` header check for Subtitle mode) according to `ConvertMode`, and atomically commits only on success.
- **Resizable & Pop-out Native OS Diagnostic Logs (`src/ui/log_panel.rs`)**:
  - Drag the top edge of the expanded bottom log drawer to resize vertically, or click **Pop Out Window** to open a real, independent OS window (`egui` multi-viewport via `show_viewport_immediate`, with `EmbeddedWindow` fallback) that can be dragged outside the main window or across monitors, with **Copy All** and **Clear Log** actions.
- **14 Persistent UI Languages (`src/i18n/`)**:
  - Supports `EN`, `TH`, `JA`, `ZH-CN`, `ZH-TW`, `KO`, `VI`, `ID`, `ES`, `PT-BR`, `FR`, `DE`, `IT`, and `RU` in an independent searchable OS window, persisted across restarts in `%APPDATA%\vibe-ffmpeg\preferences.json` (all translations generated by AI).

## Troubleshooting Failed Conversions

When a job shows a red **Failed** badge, the temporary staging file (`.tmp.<ext>`) is deleted automatically so partial/corrupted files are never kept. Check the exact cause quickly:

1. **Open or Pop Out the Diagnostic Log**: Click **`▶ Diagnostic & Verification Log`** (drag its top border up to resize) or click **Pop Out Window** at the bottom-left to view the full `[Run]` command and FFmpeg's raw `stderr` error message in a separate OS window (use **Copy All** to copy logs cleanly; also check **Live Complete Command Preview** in the right panel).
2. **`FFmpeg / ffprobe missing on PATH` (or `ffprobe not found`)**: Both `ffmpeg.exe` and `ffprobe.exe` must be present together in `PATH` or beside `ffmpeg-ui.exe`.
3. **Hardware encoder fails (`*_nvenc`, `*_amf`, `*_qsv`)**: Drivers or older GPUs may reject certain pixel formats (such as `yuv420p10le`) or settings even when `-encoders` lists them. Switch **Video Codec** to software `H.264 / AVC (libx264)` or `H.265 / HEVC (libx265)` with `yuv420p (8-bit Universal)`.
4. **Mode / Stream gating**: UI compatibility gating evaluates the **first queued file's** probed streams (`has_video`, `has_audio`, `has_subtitle`, `is_still_image`) to disable incompatible categories/presets. If a mixed batch includes different media types, trailing incompatible items fail cleanly — check the Diagnostic Log for per-file details.
5. **Remux (`-c copy`) container incompatibility**: Switching containers without re-encoding fails if the source codec is incompatible with the target format (for example, copying `H.264`/`AAC` into `WebM`, which requires `VP9`/`AV1` + `Opus`/`Vorbis`). Try **`MKV`** for remuxing, or switch to **Video** mode to re-encode.
6. **Subtitle extraction / muxing failures**: Image-based subtitles (such as Blu-ray PGS or DVD VobSub) cannot convert to `mov_text` (in MP4/MOV/M4V) or text-only subtitle targets (`SRT`, `ASS`, `SSA`, `VTT`, `LRC`). Uncheck **Keep text subtitles** or output to **MKV** when converting video; use text-based subtitle inputs for **Subtitle** mode.
7. **Missing codecs/demuxers in minimal FFmpeg builds**: If `AVIF`, `AV1`, `JXL`, `HEIC`, `ALAC`, `AMR`, `RMVB`, or `WebP` fails with unknown/missing encoder or decoder errors, switch to a **full** FFmpeg build from `gyan.dev` (`ffmpeg-release-full.7z`).
8. **File permissions / path issues**: Ensure the selected output folder is writable and not locked by another media player.

## Known Limitations

- **Windows-only tested**: No Linux or macOS verification or prebuilt binaries yet.
- **Partial preference persistence**: The selected UI language persists across restarts (`%APPDATA%\vibe-ffmpeg\preferences.json`), while output folder, prefix/suffix, and encoding parameters currently reset on launch.
- **Single batch profile & first-item gating**: Queue settings apply globally to all pending items in the batch (only the output filename stem is customizable per file), UI compatibility gating inspects the first queued file, and **Quick Presets** covers Video/Audio/Image/Remux modes (use the `FROM -> TO` picker for Subtitle targets).
- **Non-Latin glyph alignment quirks**: Minor floating diacritics, tone marks, or alignment glitches may appear in some non-Latin scripts (such as Thai or Japanese) due to `egui` text layout limitations; translations across all 14 languages are AI-generated and may contain minor phrasing inaccuracies (no RTL support yet).
- **Animated WebP decode verification**: FFmpeg's built-in `webp` decoder does not support decoding multi-frame `ANIM` WebP streams, so deep verification checks `RIFF` and `WEBP` container header signature bytes (`RIFF/ANIM Verified`) rather than decoding individual frames.
- **No timeline editing**: Designed for batch conversion, extraction, image/GIF creation, subtitle extraction, and remuxing — does not support interactive trimming, cropping UI, or merging multiple files.

## Modules

- `src/main.rs` — Window bootstrap and Windows GUI subsystem entry point.
- `src/app.rs` — App state, worker event dispatch, automatic compatible-mode selection on probe, `rfd` multi-format file dialogs, and drag-and-drop handling.
- `src/models/` — `ConvertMode`, `ContainerFormat`, `ConversionSettings`, `PresetProfile`, `IncompatibilityReason`, `check_mode_compatibility`, `MediaInfo`, `QueuedFile`.
- `src/security/path_guard.rs` — Path canonicalization, media extension registries (`VIDEO_EXTENSIONS`, `AUDIO_EXTENSIONS`, `IMAGE_EXTENSIONS`, `SUBTITLE_EXTENSIONS`), `%APPDATA%\vibe-ffmpeg\preferences.json` persistence, filename sanitization, prefix/suffix composition, collision protection, and `.tmp.<ext>` staging.
- `src/ffmpeg/` — `discovery.rs`, `probe.rs`, `command_builder.rs`, `validator.rs`, `runner.rs`.
- `src/net/ffmpeg_update.rs` — HTTPS official FFmpeg update check.
- `src/i18n/` — 14-language localization tables with English fallback.
- `src/ui/` — `theme.rs`, `top_bar.rs`, `queue_panel.rs`, `settings_panel.rs` (presets & searchable `FROM -> TO` picker), `log_panel.rs` (docked drawer & native OS window), `language_window.rs` (searchable native OS window).

## Build & Test

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo test
cargo build --release
```

## License & Attributions

- Repository: [github.com/N213S/vibe-ffmpeg-ui](https://github.com/N213S/vibe-ffmpeg-ui)
- Licensed under the [MIT License](LICENSE).
- Third-party Rust crates, embedded font licenses, and external tool credits are listed in [ATTRIBUTIONS.md](ATTRIBUTIONS.md).
