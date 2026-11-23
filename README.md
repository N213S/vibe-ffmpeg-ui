# Vibe FFmpeg ⚡

> **Built with AI Vibe Coding 🤖✨** — Created and iterated through prompt-driven engineering in Rust to scratch a personal itch and fit a fast daily workflow—open-sourced in case you find it useful too.

**Vibe FFmpeg** is a lightweight, native desktop GUI for **FFmpeg** and **ffprobe** built in Rust (`eframe` / `egui` 0.37 + `rfd` 0.18). It focuses on fast batch conversions, read-only deep media inspection, custom output naming, and mode-aware file integrity verification.

![Vibe FFmpeg UI](assets/screenshot.png)

---

## Table of Contents

- [Supported Platforms](#supported-platforms)
- [Quick Start (Windows)](#quick-start-windows)
- [Privacy & Offline Guarantee](#privacy--offline-guarantee)
- [Key Features](#key-features)
- [Supported Formats & Conversions](#supported-formats--conversions)
- [Troubleshooting](#troubleshooting)
- [Known Limitations](#known-limitations)
- [Project Architecture](#project-architecture)
- [Build & Test](#build--test)
- [License & Attributions](#license--attributions)

---

## Supported Platforms

| Platform | Status | Notes |
| --- | --- | --- |
| **Windows 11 (`x86_64`)** | Supported & Tested | Primary tested platform; uses Windows fonts, `explorer.exe /select,`, and `native-tls` (SChannel). |
| **Windows 10 (`x86_64`)** | Untested *(Likely Works)* | Not directly tested yet, but uses the same Win32 APIs and should work (WinGet auto-install requires App Installer on older builds). |
| **Linux / macOS** | Untested / Unsupported | Not tested or supported due to Windows-specific font discovery, file reveal, and TLS integration. |

---

## Quick Start (Windows)

No traditional installer is required—just **`ffmpeg-ui.exe`** alongside **`ffmpeg.exe`** and **`ffprobe.exe`**.

### 1. Download Vibe FFmpeg

Grab the latest `ffmpeg-ui.exe` from the [GitHub Releases](https://github.com/N213S/vibe-ffmpeg-ui/releases) page (or build from source under [Build & Test](#build--test)).

### 2. Install FFmpeg & `ffprobe` (Both Required)

If FFmpeg is missing when you launch the app, the **FFmpeg Setup** window opens automatically (you can also open it anytime by clicking the `FFmpeg` status pill in the top bar).

| Method | Best For | How It Works |
| --- | --- | --- |
| **Option A — WinGet Auto-Install** *(Easiest)* | Most users | Click **Auto-Install FFmpeg (WinGet)** to silently install `Gyan.FFmpeg` (`winget install --id Gyan.FFmpeg --exact`). Vibe FFmpeg automatically scans WinGet links/packages and Registry `PATH`—no restart needed. |
| **Option B — Portable (`gyan.dev`)** | Zero `PATH` changes | Download `ffmpeg-release-full.7z` from [gyan.dev/ffmpeg/builds](https://www.gyan.dev/ffmpeg/builds/) (or via the **FFmpeg Setup** links), extract **`ffmpeg.exe`** and **`ffprobe.exe`** from `bin\` next to `ffmpeg-ui.exe`, then click **Re-scan FFmpeg**. |
| **Option C — Manual System `PATH`** | Existing setups | Extract FFmpeg (e.g. `C:\ffmpeg`), add `C:\ffmpeg\bin` to System/User `PATH` in Windows Environment Variables, then click **Re-scan FFmpeg** or relaunch. |

### 3. Windows SmartScreen Notice

> [!NOTE]
> `ffmpeg-ui.exe` is an unsigned open-source binary without a paid EV code-signing certificate, so Windows SmartScreen (*"Windows protected your PC"*) or antivirus heuristics may show a warning on first launch.
>
> Verify that you downloaded it from [`N213S/vibe-ffmpeg-ui/releases`](https://github.com/N213S/vibe-ffmpeg-ui/releases), then click **More info** → **Run anyway** (or inspect the source and compile locally with `cargo build --release`).

---

## Privacy & Offline Guarantee

- **100% Local Media Processing**: Probing (`ffprobe`), converting (`ffmpeg`), atomic staging, and integrity verification all run offline on your machine. **No media files, filenames, analytics, or telemetry are ever uploaded.**
- **Local Preferences Only**: The only file written for app state is your chosen UI language in `%APPDATA%\vibe-ffmpeg\preferences.json`.
- **Manual-Only Network Requests**: The app never phones home on startup. Outbound HTTPS requests (8-second timeout via Windows `native-tls`) run **only** when you click a button:

| Action | Endpoint / Mechanism | Purpose & Cap |
| --- | --- | --- |
| **Check Update** *(top bar)* | `https://api.github.com/repos/N213S/vibe-ffmpeg-ui/releases?per_page=5` | Checks for newer Vibe FFmpeg releases (`32 KB` read cap). |
| **Check Official Version** *(FFmpeg Setup)* | `https://www.gyan.dev/ffmpeg/builds/release-version` & `git-version` | Checks latest official FFmpeg builds (`2 KB` read cap). |
| **Auto-Install FFmpeg (WinGet)** | `winget.exe` (`Gyan.FFmpeg`) | Installs FFmpeg only when explicitly clicked. |

---

## Key Features

### Dual Workspace Modes
- **Convert Mode (`FFmpeg`)**: Batch conversion queue with presets, searchable `FROM -> TO` format picker, codec/quality controls, and live command preview.
- **Inspect Mode (`ffprobe`)**: Read-only deep file analyzer displaying container/demuxer details (`format`, `size`, `duration`, `bitrate`, `start_time`, `probe_score`), metadata tags, per-stream attributes (codec, profile, tag, resolution, pixel format, color space/range/transfer/primaries, FPS, frames, audio channels/sample rate, language, title, stream tags), and a collapsible raw `ffprobe` JSON viewer.
  - **Copy Report**: Copies the complete container, metadata, and stream breakdown to the clipboard.
  - **Export (`TXT`, `Markdown`, `HTML`, `JSON`, `CSV`)**: Saves the full inspection report via a native save dialog with HTML-escaped metadata and sanitized filenames.

### Smart Stream & Compatibility Gating
- `ffprobe` inspects queued files for video, audio, text subtitles (`has_text_subtitles`), embedded cover art, and still images (`is_still_image`).
- Automatically switches out of an incompatible mode when the first file finishes probing and disables unsupported categories or presets with clear hover tooltips.

### Safety, Naming & Integrity Verification
- **Collision & Input Protection**: Editable per-file output base names with auto-synced extensions, optional global prefix/suffix, and automatic `_out` / `(N)` collision naming so source files are never overwritten in-place.
- **Atomic Staging**: Writes output to `.<stem>.<pid>_<job_id>.tmp.<ext>` with explicit `-f <muxer>` flags and commits to the final filename only after verification passes.
- **Mode-Aware Deep Verification**: Checks output streams, duration, and subtitles with `ffprobe`, runs a deep decode pass (`ffmpeg -v error -xerror -f null -`), validates `RIFF`/`WEBP` signatures for animated WebP, or checks subtitle stream headers in Subtitle mode.

### Pop-Out Logs, Built-in Setup & 14 Languages
- **Resizable & Pop-Out Diagnostic Logs**: Drag the top border of the bottom log drawer to resize, or click **Pop Out Window** to open an independent native OS window (`egui` multi-viewport with embedded fallback) featuring **Copy All** and **Clear Log**.
- **Built-in FFmpeg Setup & App Update Checker**: Separate OS window for WinGet auto-install, `gyan.dev` version checks, and live binary re-scanning (sibling folder → `PATH` → WinGet Links/Packages → Registry `HKCU`/`HKLM` `Path`).
- **14 Persistent UI Languages**: `EN`, `TH`, `JA`, `ZH-CN`, `ZH-TW`, `KO`, `VI`, `ID`, `ES`, `PT-BR`, `FR`, `DE`, `IT`, and `RU` via a searchable language window (AI-generated translations).

---

## Supported Formats & Conversions

Includes **53 output formats**, **17 quick presets**, and a searchable **`FROM -> TO`** format picker:

| Category | Count | Formats & Highlights |
| --- | --- | --- |
| **Video** | 17 | `MP4`, `MKV`, `MOV`, `WEBM`, `AVI`, `FLV`, `TS`, `M2TS`, `MXF`, `3GP`, `M4V`, `MPG`, `MPEG`, `VOB`, `OGV`, `WMV`, `DV` — software & hardware encoders (`libx264`, `libx265`, `libsvtav1`, `libvpx-vp9`, `mpeg2video`, `libtheora`, `wmv2`, `dvvideo`, NVENC, AMF, QSV). |
| **Audio Only** | 18 | `MP3`, `WAV`, `FLAC`, `AAC`, `M4A`, `OGG`, `OPUS`, `WMA`, `AIFF`, `AC3`, `EAC3`, `DTS`, `ALAC`, `AMR`, `CAF`, `MKA`, `MP2`, `WV` — automatic `-vn` video stripping. |
| **Image / Animated** | 13 | `PNG`, `JPG`, `WEBP`, `GIF` (`palettegen` + `paletteuse`), `AVIF`, `APNG`, `BMP`, `TIFF`, `TGA`, `ICO` (auto-capped `<=256x256`), `JXL`, `EXR`, `HDR`. |
| **Subtitle** | 5 | `SRT`, `ASS`, `SSA`, `VTT`, `LRC` — text subtitle extraction and cross-conversion (`-vn -an`). |
| **Remux (Fast Copy)** | 10 | `MP4`, `MKV`, `MOV`, `WEBM`, `AVI`, `FLV`, `TS`, `M2TS`, `M4V`, `MKA` — lossless container switching (`-c copy`) without re-encoding. |

- **Cross-Category Conversions**:
  - **Image / GIF → Video**: `-loop 1`, configurable duration, and even-dimension scaling.
  - **Audio → Video**: `showwaves` 720p waveform visualizer.
  - **Audio → Image**: Extracts embedded cover art or generates a `showwavespic` waveform image.
  - **Video → Image / Animated Image** and **Video / Subtitle → Subtitle**.
- **Expanded Input Support**: Accepts legacy, camera, audio, RAW/HDR image, and subtitle inputs (`.rm`, `.rmvb`, `.heic`, `.heif`, `.ape`, `.mod`, `.tod`, `.bik`, `.mts`, `.m2ts`, `.vob`, `.asf`, `.dv`, `.swf`, `.mka`, `.amr`, `.caf`, `.wv`, `.tta`, `.tga`, `.exr`, `.hdr`, `.lrc`, `.srt`, `.ass`, `.ssa`, `.vtt`, `.sub`, etc.) via file picker filters (including `*` All Files) and drag-and-drop.

---

## Troubleshooting

When a job shows a red **Failed** badge, its temporary staging file (`.tmp.<ext>`) is deleted automatically so broken outputs are never left behind.

| Issue | Likely Cause & Fix |
| --- | --- |
| **Need the exact FFmpeg error** | Open **`▶ Diagnostic & Verification Log`** (drag the top border up to resize) or click **Pop Out Window** to inspect the `[Run]` command and raw `stderr`, or check **Live Complete Command Preview** in the right panel. |
| **`FFmpeg / ffprobe missing on PATH`** | Both `ffmpeg.exe` and `ffprobe.exe` must exist together in `PATH` or in the same folder as `ffmpeg-ui.exe`. |
| **Hardware encoder fails (`*_nvenc`, `*_amf`, `*_qsv`)** | Your GPU/driver may reject certain pixel formats (e.g. `yuv420p10le`) even if listed. Switch **Video Codec** to software `H.264 / AVC (libx264)` or `H.265 / HEVC (libx265)` with `yuv420p (8-bit Universal)`. |
| **Mixed-file batch failures** | Compatibility gating evaluates the **first queued file**; trailing incompatible files in a mixed batch fail cleanly—check the Diagnostic Log for per-file details. |
| **Remux (`-c copy`) fails** | Source codecs are incompatible with the target container (e.g., `H.264`/`AAC` into `WebM`, which requires `VP9`/`AV1` + `Opus`/`Vorbis`). Try **`MKV`** or switch to **Video** mode to re-encode. |
| **Subtitle extraction / muxing fails** | Image-based subtitles (Blu-ray PGS, DVD VobSub) cannot convert to `mov_text` (`MP4`/`MOV`/`M4V`) or text formats (`SRT`/`ASS`/`SSA`/`VTT`/`LRC`). Uncheck **Keep text subtitles** or output to **MKV**. |
| **Missing codec/demuxer errors** | Formats like `AVIF`, `AV1`, `JXL`, `HEIC`, `ALAC`, `AMR`, `RMVB`, or `WebP` may be missing from minimal FFmpeg builds. Install a **full** build (`ffmpeg-release-full.7z` from `gyan.dev`). |
| **Permission or file-lock errors** | Make sure the destination folder is writable and the target file is not open in another media player. |

---

## Known Limitations

- **Tested on Windows 11 only**: Windows 10 has not been directly tested yet (though it should likely work), and Linux/macOS are untested and unsupported.
- **Partial Preference Persistence**: Only the selected UI language persists (`%APPDATA%\vibe-ffmpeg\preferences.json`); output folder, prefix/suffix, and encoding parameters reset on launch.
- **Single Batch Profile**: Queue settings apply globally to all pending jobs (only the output filename stem is editable per file), compatibility gating checks the first queued item, and **Quick Presets** covers Video/Audio/Image/Remux (use `FROM -> TO` for Subtitle targets).
- **Non-Latin Text Layout**: Minor floating diacritics, tone marks, or alignment quirks may appear in scripts like Thai or Japanese due to `egui` layout behavior; all 14 translations are AI-generated and RTL scripts are not yet supported.
- **Animated WebP Verification**: FFmpeg's built-in `webp` decoder cannot decode multi-frame `ANIM` streams, so deep verification validates `RIFF` and `WEBP` container signatures (`RIFF/ANIM Verified`) rather than decoding individual frames.
- **No Timeline Editing**: Built for batch conversion, extraction, inspection, and remuxing—interactive trimming, visual cropping, and multi-file merging are not included.

---

## Project Architecture

| Module | Responsibility |
| --- | --- |
| `src/main.rs` | Window bootstrap and Windows GUI subsystem entry point. |
| `src/app.rs` | App state, worker event dispatch, auto-compatible mode selection, `rfd` dialogs, drag-and-drop, and report export wiring. |
| `src/models/` | `ConvertMode`, `ContainerFormat`, `ConversionSettings`, `PresetProfile`, `IncompatibilityReason`, `check_mode_compatibility`, `MediaInfo`, `QueuedFile`. |
| `src/security/path_guard.rs` | Path canonicalization, extension registries, `TRUSTED_HTTPS_PREFIXES` allowlist, language persistence, filename sanitization, collision protection, and `.tmp.<ext>` staging. |
| `src/ffmpeg/` | `discovery.rs` (sibling/`PATH`/WinGet/Registry), `installer.rs` (WinGet `Gyan.FFmpeg`), `probe.rs`, `report_export.rs` (TXT/MD/HTML/JSON/CSV), `command_builder.rs`, `validator.rs`, `runner.rs`. |
| `src/net/` | `app_update.rs` (GitHub Releases update check) and `ffmpeg_update.rs` (`gyan.dev` version check). |
| `src/i18n/` | 14-language localization tables with English fallback. |
| `src/ui/` | `theme.rs`, `top_bar.rs`, `queue_panel.rs`, `settings_panel.rs`, `inspector_panel.rs`, `log_panel.rs`, `language_window.rs`, `ffmpeg_setup_window.rs`. |

---

## Build & Test

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo test
cargo build --release
```

---

## License & Attributions

- **Repository**: [github.com/N213S/vibe-ffmpeg-ui](https://github.com/N213S/vibe-ffmpeg-ui)
- **License**: [MIT License](LICENSE)
- **Third-Party Credits**: Rust crates, embedded font licenses, and external tool credits are documented in [ATTRIBUTIONS.md](ATTRIBUTIONS.md).
