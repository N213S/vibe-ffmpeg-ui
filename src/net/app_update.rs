use crate::models::media::{APP_VERSION, AppUpdateInfo, WorkerEvent};
use crate::net::ffmpeg_update::is_remote_newer;
use crate::security::path_guard::{sanitize_remote_version, validate_trusted_https_url};
use serde::Deserialize;
use std::io::Read;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use ureq::tls::{TlsConfig, TlsProvider};

pub const APP_RELEASES_API_URL: &str =
    "https://api.github.com/repos/N213S/vibe-ffmpeg-ui/releases?per_page=5";
pub const APP_RELEASES_PAGE: &str = "https://github.com/N213S/vibe-ffmpeg-ui/releases";
const MAX_GITHUB_RESPONSE_BYTES: u64 = 32_768;

#[derive(Debug, Deserialize)]
struct GithubReleaseItem {
    tag_name: String,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    draft: bool,
}

/// Spawns a non-blocking HTTPS check against official GitHub releases for Vibe FFmpeg.
pub fn spawn_app_update_check(tx: Sender<WorkerEvent>, ctx: eframe::egui::Context) {
    thread::spawn(move || {
        let res = check_official_app_updates();
        let _ = tx.send(WorkerEvent::AppUpdateChecked(res));
        ctx.request_repaint();
    });
}

/// Queries the official GitHub repository releases and compares against the current app version.
pub fn check_official_app_updates() -> Result<AppUpdateInfo, String> {
    validate_trusted_https_url(APP_RELEASES_API_URL)?;

    let tls = TlsConfig::builder()
        .provider(TlsProvider::NativeTls)
        .build();

    let agent = ureq::Agent::config_builder()
        .tls_config(tls)
        .timeout_global(Some(Duration::from_secs(8)))
        .user_agent(concat!("ffmpeg-ui/", env!("CARGO_PKG_VERSION")))
        .build()
        .new_agent();

    let mut resp = agent
        .get(APP_RELEASES_API_URL)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("HTTPS request failed: {e}"))?;

    let mut raw_json = String::new();
    resp.body_mut()
        .as_reader()
        .take(MAX_GITHUB_RESPONSE_BYTES)
        .read_to_string(&mut raw_json)
        .map_err(|e| format!("Failed reading HTTPS response: {e}"))?;

    parse_latest_github_release(&raw_json, APP_VERSION)
}

/// Parses GitHub Releases JSON payload into validated `AppUpdateInfo`.
pub fn parse_latest_github_release(
    raw_json: &str,
    current_version: &str,
) -> Result<AppUpdateInfo, String> {
    let items: Vec<GithubReleaseItem> = serde_json::from_str(raw_json)
        .map_err(|e| format!("Failed parsing GitHub releases payload: {e}"))?;

    let release = items
        .into_iter()
        .find(|item| !item.draft)
        .ok_or_else(|| "No published GitHub releases found.".to_string())?;

    let sanitized_tag = sanitize_remote_version(&release.tag_name)
        .ok_or_else(|| "GitHub release tag returned unexpected format.".to_string())?;
    let latest_version = sanitized_tag
        .trim_start_matches(['v', 'V'])
        .to_string();
    if latest_version.is_empty() {
        return Err("GitHub release tag is empty.".to_string());
    }

    let release_url = if validate_trusted_https_url(&release.html_url).is_ok() {
        release.html_url
    } else {
        APP_RELEASES_PAGE.to_string()
    };

    let update_available = is_remote_newer(current_version, &latest_version);

    Ok(AppUpdateInfo {
        latest_version,
        current_version: current_version.to_string(),
        release_url,
        update_available,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_github_releases_json_and_detects_newer_version() {
        let json = r#"[
            {"tag_name": "v9.9.9", "html_url": "https://github.com/N213S/vibe-ffmpeg-ui/releases/tag/v9.9.9", "draft": true},
            {"tag_name": "v0.3.0", "html_url": "https://github.com/N213S/vibe-ffmpeg-ui/releases/tag/v0.3.0", "draft": false}
        ]"#;
        let info = parse_latest_github_release(json, "0.2.1").unwrap();
        assert_eq!(info.latest_version, "0.3.0");
        assert_eq!(info.current_version, "0.2.1");
        assert!(info.update_available);
        assert_eq!(
            info.release_url,
            "https://github.com/N213S/vibe-ffmpeg-ui/releases/tag/v0.3.0"
        );

        let same = parse_latest_github_release(json, "0.3.0").unwrap();
        assert!(!same.update_available);
    }
}
