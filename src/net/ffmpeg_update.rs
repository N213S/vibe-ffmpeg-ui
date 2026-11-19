use crate::models::media::{UpdateCheckInfo, WorkerEvent};
use crate::security::path_guard::{sanitize_remote_version, validate_trusted_https_url};
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use ureq::tls::{TlsConfig, TlsProvider};

pub const OFFICIAL_BUILDS_PAGE: &str = "https://www.gyan.dev/ffmpeg/builds/";
const RELEASE_VERSION_URL: &str = "https://www.gyan.dev/ffmpeg/builds/release-version";
const GIT_VERSION_URL: &str = "https://www.gyan.dev/ffmpeg/builds/git-version";
const MAX_RESPONSE_BYTES: u64 = 2048;

/// Spawns a non-blocking HTTPS check against official Gyan.dev release endpoints.
pub fn spawn_update_check(
    local_version: Option<String>,
    tx: Sender<WorkerEvent>,
    ctx: eframe::egui::Context,
) {
    thread::spawn(move || {
        let res = check_official_ffmpeg_updates(local_version);
        let _ = tx.send(WorkerEvent::UpdateChecked(res));
        ctx.request_repaint();
    });
}

/// Fetches latest FFmpeg release & git versions from official Gyan.dev HTTPS endpoints.
pub fn check_official_ffmpeg_updates(
    local_version: Option<String>,
) -> Result<UpdateCheckInfo, String> {
    validate_trusted_https_url(RELEASE_VERSION_URL)?;
    validate_trusted_https_url(GIT_VERSION_URL)?;

    let tls = TlsConfig::builder()
        .provider(TlsProvider::NativeTls)
        .build();

    let agent = ureq::Agent::config_builder()
        .tls_config(tls)
        .timeout_global(Some(Duration::from_secs(8)))
        .user_agent(concat!("ffmpeg-ui/", env!("CARGO_PKG_VERSION")))
        .build()
        .new_agent();

    let release_raw = fetch_capped_text(&agent, RELEASE_VERSION_URL)?;
    let latest_release = sanitize_remote_version(&release_raw)
        .ok_or_else(|| "Official release endpoint returned unexpected format.".to_string())?;

    let latest_git = fetch_capped_text(&agent, GIT_VERSION_URL)
        .ok()
        .and_then(|s| sanitize_remote_version(&s));

    let update_available = match local_version.as_deref() {
        Some(local) => is_remote_newer(local, &latest_release),
        None => false,
    };

    Ok(UpdateCheckInfo {
        latest_release,
        latest_git,
        local_version,
        update_available,
    })
}

fn fetch_capped_text(agent: &ureq::Agent, url: &str) -> Result<String, String> {
    let mut resp = agent
        .get(url)
        .call()
        .map_err(|e| format!("HTTPS request failed: {e}"))?;

    let mut buf = String::new();
    resp.body_mut()
        .as_reader()
        .take(MAX_RESPONSE_BYTES)
        .read_to_string(&mut buf)
        .map_err(|e| format!("Failed reading HTTPS response: {e}"))?;

    Ok(buf)
}

/// Compares dotted numeric versions (`8.0.1` vs `9.0.2`).
pub fn is_remote_newer(local: &str, remote: &str) -> bool {
    let parse_parts = |s: &str| -> Option<Vec<u32>> {
        let core = s.split(['-', '+']).next()?.trim();
        let nums: Vec<u32> = core
            .split('.')
            .map(|p| p.parse::<u32>().ok())
            .collect::<Option<Vec<_>>>()?;
        if nums.is_empty() {
            None
        } else {
            Some(nums)
        }
    };

    match (parse_parts(local), parse_parts(remote)) {
        (Some(l), Some(r)) => {
            let max_len = l.len().max(r.len());
            for i in 0..max_len {
                let lv = *l.get(i).unwrap_or(&0);
                let rv = *r.get(i).unwrap_or(&0);
                if rv > lv {
                    return true;
                }
                if rv < lv {
                    return false;
                }
            }
            false
        }
        _ => false,
    }
}

/// Safely opens an allowlisted official HTTPS URL in the user's browser without invoking a shell.
pub fn open_trusted_url(url: &str) -> Result<(), String> {
    validate_trusted_https_url(url)?;
    #[cfg(windows)]
    {
        Command::new("rundll32.exe")
            .args(["url.dll,FileProtocolHandler", url])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to open browser: {e}"))?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Command::new("xdg-open")
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to open browser: {e}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_ffmpeg_semver_correctly() {
        assert!(is_remote_newer("8.0.1", "9.0.2"));
        assert!(is_remote_newer("8.0", "8.0.1"));
        assert!(!is_remote_newer("9.0.2", "9.0.2"));
        assert!(!is_remote_newer("9.1.0", "9.0.2"));
    }
}
