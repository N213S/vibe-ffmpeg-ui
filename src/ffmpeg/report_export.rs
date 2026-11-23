use crate::models::media::{MediaInfo, format_bytes, format_duration};
use crate::security::path_guard::sanitize_file_stem;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// Supported export formats for `ffprobe` media inspection reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportExportFormat {
    Text,
    Markdown,
    Html,
    Json,
    Csv,
}

impl ReportExportFormat {
    pub const ALL: &'static [Self] = &[
        Self::Text,
        Self::Markdown,
        Self::Html,
        Self::Json,
        Self::Csv,
    ];

    pub const fn extension(self) -> &'static str {
        match self {
            Self::Text => "txt",
            Self::Markdown => "md",
            Self::Html => "html",
            Self::Json => "json",
            Self::Csv => "csv",
        }
    }

    pub const fn short_label(self) -> &'static str {
        match self {
            Self::Text => "TXT",
            Self::Markdown => "Markdown",
            Self::Html => "HTML",
            Self::Json => "JSON",
            Self::Csv => "CSV",
        }
    }

    pub const fn filter_name(self) -> &'static str {
        match self {
            Self::Text => "Plain Text (.txt)",
            Self::Markdown => "Markdown (.md)",
            Self::Html => "HTML Report (.html)",
            Self::Json => "ffprobe JSON (.json)",
            Self::Csv => "CSV Table (.csv)",
        }
    }
}

/// Renders a complete `ffprobe` inspection report in the requested format.
pub fn render_report(media: &MediaInfo, file_path: &Path, format: ReportExportFormat) -> String {
    match format {
        ReportExportFormat::Text => render_text_report(media, file_path),
        ReportExportFormat::Markdown => render_markdown_report(media, file_path),
        ReportExportFormat::Html => render_html_report(media, file_path),
        ReportExportFormat::Json => {
            if media.raw_json.trim().is_empty() {
                "{}".to_string()
            } else {
                media.raw_json.clone()
            }
        }
        ReportExportFormat::Csv => render_csv_report(media, file_path),
    }
}

/// Opens a native Save dialog and writes the formatted inspection report to disk.
pub fn export_report_to_file(
    media: &MediaInfo,
    source_path: &Path,
    format: ReportExportFormat,
) -> Result<Option<PathBuf>, String> {
    let raw_stem = source_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "media".to_string());
    let safe_stem = sanitize_file_stem(&raw_stem);
    let default_name = format!("{safe_stem}_ffprobe.{}", format.extension());

    let Some(dest) = rfd::FileDialog::new()
        .set_file_name(&default_name)
        .add_filter(format.filter_name(), &[format.extension()])
        .save_file()
    else {
        return Ok(None);
    };

    let content = render_report(media, source_path, format);
    fs::write(&dest, content).map_err(|e| format!("Failed writing report: {e}"))?;
    Ok(Some(dest))
}

fn container_rows(media: &MediaInfo, file_path: &Path) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    rows.push(("File".to_string(), file_path.display().to_string()));
    let fmt = match media.format_long_name.as_deref() {
        Some(long) if !long.is_empty() => format!("{} ({long})", media.format_name),
        _ => media.format_name.clone(),
    };
    rows.push(("Format / Demuxer".to_string(), fmt));
    rows.push((
        "Size".to_string(),
        format!(
            "{} ({} B)",
            format_bytes(media.file_size_bytes),
            media.file_size_bytes
        ),
    ));
    if let Some(dur) = media.duration_secs {
        rows.push((
            "Duration".to_string(),
            format!("{} ({dur:.3} s)", format_duration(dur)),
        ));
    }
    if let Some(start) = media.start_time_secs {
        rows.push(("Start Time".to_string(), format!("{start:.3} s")));
    }
    if let Some(br) = media.bit_rate {
        rows.push((
            "Overall Bitrate".to_string(),
            format!("{} kbps ({br} bps)", br / 1000),
        ));
    }
    if let Some(score) = media.probe_score {
        rows.push(("Probe Score".to_string(), format!("{score}/100")));
    }
    rows.push(("Streams".to_string(), media.streams.len().to_string()));
    rows
}

fn stream_rows(s: &crate::models::media::StreamDetail) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    let codec = match s.codec_long_name.as_deref() {
        Some(long) if !long.is_empty() => format!("{} ({long})", s.codec_name),
        _ => s.codec_name.clone(),
    };
    rows.push(("Codec".to_string(), codec));
    if let Some(ref p) = s.profile {
        rows.push(("Profile".to_string(), p.clone()));
    }
    if let Some(ref tag) = s.codec_tag_string
        && !tag.is_empty()
        && tag != "0x0000"
        && tag != "[0][0][0][0]"
    {
        rows.push(("Codec Tag".to_string(), tag.clone()));
    }
    if let (Some(w), Some(h)) = (s.width, s.height) {
        rows.push(("Resolution".to_string(), format!("{w}x{h}")));
    }
    if let Some(ref pix) = s.pixel_format {
        rows.push(("Pixel Format".to_string(), pix.clone()));
    }
    let color: Vec<&str> = [
        s.color_space.as_deref(),
        s.color_range.as_deref(),
        s.color_transfer.as_deref(),
        s.color_primaries.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter(|v| !v.is_empty())
    .collect();
    if !color.is_empty() {
        rows.push(("Color".to_string(), color.join(" / ")));
    }
    if let Some(fps) = s.fps {
        let avg = s.avg_frame_rate.as_deref().unwrap_or("-");
        let r = s.r_frame_rate.as_deref().unwrap_or("-");
        rows.push((
            "Frame Rate".to_string(),
            format!("{fps:.3} fps (avg: {avg}, r: {r})"),
        ));
    }
    if let Some(frames) = s.nb_frames {
        rows.push(("Frames".to_string(), frames.to_string()));
    }
    if let Some(sr) = s.sample_rate {
        rows.push(("Sample Rate".to_string(), format!("{sr} Hz")));
    }
    if let Some(ch) = s.channels {
        let layout = s
            .channel_layout
            .as_deref()
            .map(|l| format!(" ({l})"))
            .unwrap_or_default();
        rows.push(("Channels".to_string(), format!("{ch} ch{layout}")));
    }
    if let Some(br) = s.bit_rate {
        rows.push(("Bitrate".to_string(), format!("{} kbps ({br} bps)", br / 1000)));
    }
    if let Some(dur) = s.duration_secs {
        rows.push(("Duration".to_string(), format!("{dur:.3} s")));
    }
    if s.is_default {
        rows.push(("Default".to_string(), "Yes".to_string()));
    }
    if s.is_attached_pic {
        rows.push(("Attached Cover Art".to_string(), "Yes".to_string()));
    }
    if let Some(ref lang) = s.language {
        rows.push(("Language".to_string(), lang.clone()));
    }
    if let Some(ref title) = s.title {
        rows.push(("Title".to_string(), title.clone()));
    }
    for (k, v) in &s.tags {
        if k != "language" && k != "title" {
            rows.push((format!("Tag:{k}"), v.clone()));
        }
    }
    rows
}

fn render_text_report(media: &MediaInfo, file_path: &Path) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "=== ffprobe Media Inspection Report ===");
    for (k, v) in container_rows(media, file_path) {
        let _ = writeln!(out, "{k}: {v}");
    }

    if !media.format_tags.is_empty() {
        let _ = writeln!(out, "\n--- Container Metadata Tags ---");
        for (k, v) in &media.format_tags {
            let _ = writeln!(out, "{k}: {v}");
        }
    }

    let _ = writeln!(out, "\n--- Streams ({}) ---", media.streams.len());
    for s in &media.streams {
        let _ = writeln!(
            out,
            "\nStream #{} [{}] {}",
            s.index,
            s.codec_type.to_ascii_uppercase(),
            s.codec_name.to_ascii_uppercase()
        );
        for (k, v) in stream_rows(s) {
            let _ = writeln!(out, "  {k}: {v}");
        }
    }
    out
}

fn render_markdown_report(media: &MediaInfo, file_path: &Path) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# ffprobe Media Inspection Report\n");
    let _ = writeln!(out, "## Container & Format Details\n");
    let _ = writeln!(out, "| Property | Value |\n| --- | --- |");
    for (k, v) in container_rows(media, file_path) {
        let _ = writeln!(out, "| {} | {} |", escape_md(&k), escape_md(&v));
    }

    if !media.format_tags.is_empty() {
        let _ = writeln!(out, "\n## Metadata Tags\n");
        let _ = writeln!(out, "| Tag | Value |\n| --- | --- |");
        for (k, v) in &media.format_tags {
            let _ = writeln!(out, "| {} | {} |", escape_md(k), escape_md(v));
        }
    }

    let _ = writeln!(out, "\n## Streams ({})\n", media.streams.len());
    for s in &media.streams {
        let _ = writeln!(
            out,
            "### Stream #{} — {} ({})\n",
            s.index,
            s.codec_type.to_ascii_uppercase(),
            s.codec_name.to_ascii_uppercase()
        );
        let _ = writeln!(out, "| Property | Value |\n| --- | --- |");
        for (k, v) in stream_rows(s) {
            let _ = writeln!(out, "| {} | {} |", escape_md(&k), escape_md(&v));
        }
        let _ = writeln!(out);
    }
    out
}

fn render_html_report(media: &MediaInfo, file_path: &Path) -> String {
    let mut out = String::new();
    let title = file_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Media Report".to_string());

    let _ = writeln!(
        out,
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>ffprobe Report - {}</title>",
        escape_html(&title)
    );
    let _ = writeln!(
        out,
        "<style>body{{font-family:Segoe UI,system-ui,sans-serif;background:#0f1219;color:#e2e8f0;margin:24px;}}h1,h2,h3{{color:#60a5fa;}}.card{{background:#1c212e;border:1px solid #30384c;border-radius:8px;padding:16px;margin-bottom:16px;}}table{{width:100%;border-collapse:collapse;}}th,td{{text-align:left;padding:6px 10px;border-bottom:1px solid #2a3244;font-size:13px;}}th{{color:#94a3b8;width:220px;}}code{{font-family:Consolas,monospace;color:#cbd5e1;}}</style>\n</head>\n<body>"
    );
    let _ = writeln!(out, "<h1>ffprobe Media Inspection Report</h1>");

    let _ = writeln!(out, "<div class=\"card\"><h2>Container &amp; Format Details</h2><table>");
    for (k, v) in container_rows(media, file_path) {
        let _ = writeln!(
            out,
            "<tr><th>{}</th><td><code>{}</code></td></tr>",
            escape_html(&k),
            escape_html(&v)
        );
    }
    let _ = writeln!(out, "</table></div>");

    if !media.format_tags.is_empty() {
        let _ = writeln!(out, "<div class=\"card\"><h2>Metadata Tags</h2><table>");
        for (k, v) in &media.format_tags {
            let _ = writeln!(
                out,
                "<tr><th>{}</th><td>{}</td></tr>",
                escape_html(k),
                escape_html(v)
            );
        }
        let _ = writeln!(out, "</table></div>");
    }

    for s in &media.streams {
        let _ = writeln!(
            out,
            "<div class=\"card\"><h3>Stream #{} — {} ({})</h3><table>",
            s.index,
            escape_html(&s.codec_type.to_ascii_uppercase()),
            escape_html(&s.codec_name.to_ascii_uppercase())
        );
        for (k, v) in stream_rows(s) {
            let _ = writeln!(
                out,
                "<tr><th>{}</th><td>{}</td></tr>",
                escape_html(&k),
                escape_html(&v)
            );
        }
        let _ = writeln!(out, "</table></div>");
    }

    let _ = writeln!(out, "</body>\n</html>");
    out
}

fn render_csv_report(media: &MediaInfo, file_path: &Path) -> String {
    let mut out = String::from("section,stream_index,key,value\n");
    for (k, v) in container_rows(media, file_path) {
        let _ = writeln!(
            out,
            "container,,{},{}",
            escape_csv(&k),
            escape_csv(&v)
        );
    }
    for (k, v) in &media.format_tags {
        let _ = writeln!(
            out,
            "metadata,,{},{}",
            escape_csv(k),
            escape_csv(v)
        );
    }
    for s in &media.streams {
        let sec = format!("stream:{}", s.codec_type);
        for (k, v) in stream_rows(s) {
            let _ = writeln!(
                out,
                "{},{},{},{}",
                escape_csv(&sec),
                s.index,
                escape_csv(&k),
                escape_csv(&v)
            );
        }
    }
    out
}

fn escape_md(s: &str) -> String {
    s.replace('|', "\\|").replace(['\r', '\n'], " ")
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn escape_csv(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::media::StreamDetail;

    #[test]
    fn exports_all_formats_and_escapes_html_metadata() {
        let media = MediaInfo {
            format_name: "matroska,webm".to_string(),
            format_long_name: Some("Matroska / WebM".to_string()),
            duration_secs: Some(12.5),
            file_size_bytes: 1_048_576,
            bit_rate: Some(671_088),
            format_tags: vec![("title".to_string(), "<script>alert(1)</script>".to_string())],
            streams: vec![StreamDetail {
                index: 0,
                codec_type: "video".to_string(),
                codec_name: "h264".to_string(),
                width: Some(1920),
                height: Some(1080),
                ..Default::default()
            }],
            raw_json: "{\"format\":{}}".to_string(),
            ..Default::default()
        };
        let path = Path::new("clip.mkv");

        let txt = render_report(&media, path, ReportExportFormat::Text);
        assert!(txt.contains("Matroska / WebM"));
        assert!(txt.contains("1920x1080"));

        let md = render_report(&media, path, ReportExportFormat::Markdown);
        assert!(md.contains("| Format / Demuxer |"));

        let html = render_report(&media, path, ReportExportFormat::Html);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));

        let csv = render_report(&media, path, ReportExportFormat::Csv);
        assert!(csv.starts_with("section,stream_index,key,value\n"));
        assert!(csv.contains("\"matroska,webm (Matroska / WebM)\""));

        let json = render_report(&media, path, ReportExportFormat::Json);
        assert_eq!(json, "{\"format\":{}}");
    }
}
