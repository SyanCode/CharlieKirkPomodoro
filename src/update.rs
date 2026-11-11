//! Vérification (et installation sous Windows) des nouvelles versions publiées sur GitHub.

use crate::proc;
use eframe::egui;
use std::io::Write;
use std::sync::{Arc, Mutex};

pub const CURRENT: &str = env!("CARGO_PKG_VERSION");
const API: &str = "https://api.github.com/repos/SyanCode/CharlieKirkPomodoro/releases/latest";
const USER_AGENT: &str = "CharlieKirkPomodoro-update-check";

#[cfg(windows)]
const ASSET: &str = "CharlieKirkPomodoro-Setup.exe";
#[cfg(target_os = "macos")]
const ASSET: &str = "CharlieKirkPomodoro-macos.dmg";
#[cfg(not(any(windows, target_os = "macos")))]
const ASSET: &str = "CharlieKirkPomodoro-linux-amd64.deb";

#[derive(Clone, Debug, PartialEq)]
pub struct Release {
    pub version: String,
    pub page: String,
    pub asset_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum State {
    #[default]
    Idle,
    Checking,
    UpToDate,
    Available(Release),
    Downloading,
    /// Installateur lancé : l'appli peut se fermer.
    Launched,
    Error(String),
}

pub type Shared = Arc<Mutex<State>>;

fn parse(v: &str) -> Vec<u64> {
    v.trim().trim_start_matches('v').split('.').map(|p| p.parse().unwrap_or(0)).collect()
}

/// `remote` est-elle strictement plus récente que `local` ?
pub fn is_newer(remote: &str, local: &str) -> bool {
    let (mut r, mut l) = (parse(remote), parse(local));
    let n = r.len().max(l.len());
    r.resize(n, 0);
    l.resize(n, 0);
    r > l
}

fn parse_release(json: &serde_json::Value) -> Option<Release> {
    let version = json.get("tag_name")?.as_str()?.trim_start_matches('v').to_string();
    let page = json.get("html_url")?.as_str()?.to_string();
    let asset_url = json
        .get("assets")
        .and_then(|a| a.as_array())
        .and_then(|a| a.iter().find(|x| x.get("name").and_then(|n| n.as_str()) == Some(ASSET)))
        .and_then(|x| x.get("browser_download_url")?.as_str().map(String::from));
    Some(Release { version, page, asset_url })
}

fn fetch_latest() -> Result<Release, String> {
    let mut resp = ureq::get(API)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("Impossible de contacter GitHub : {e}"))?;
    let text = resp.body_mut().read_to_string().map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    parse_release(&json).ok_or_else(|| "Réponse GitHub inattendue".to_string())
}

pub fn check(ctx: egui::Context, state: Shared) {
    {
        let mut s = state.lock().unwrap();
        if matches!(*s, State::Checking | State::Downloading) {
            return;
        }
        *s = State::Checking;
    }
    std::thread::spawn(move || {
        let result = match fetch_latest() {
            Ok(r) if is_newer(&r.version, CURRENT) => State::Available(r),
            Ok(_) => State::UpToDate,
            Err(e) => State::Error(e),
        };
        *state.lock().unwrap() = result;
        ctx.request_repaint();
    });
}

/// Windows : télécharge l'installateur et le lance. Ailleurs : ouvre la page de la release.
pub fn install(ctx: egui::Context, state: Shared, release: Release) {
    let Some(url) = release.asset_url.clone().filter(|_| cfg!(windows)) else {
        let _ = open::that(&release.page);
        *state.lock().unwrap() = State::Available(release);
        return;
    };
    *state.lock().unwrap() = State::Downloading;
    std::thread::spawn(move || {
        let result = download_and_launch(&url, &release.version);
        *state.lock().unwrap() = match result {
            Ok(()) => State::Launched,
            Err(e) => {
                // Repli : page de la release dans le navigateur.
                let _ = open::that(&release.page);
                State::Error(format!("{e} — la page de téléchargement a été ouverte dans ton navigateur."))
            }
        };
        ctx.request_repaint();
    });
}

fn download_and_launch(url: &str, version: &str) -> Result<(), String> {
    let path = std::env::temp_dir().join(format!("CharlieKirkPomodoro-Setup-{version}.exe"));
    let mut resp = ureq::get(url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| format!("Téléchargement impossible : {e}"))?;
    let mut reader = resp.body_mut().with_config().limit(1 << 30).reader();
    let mut file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    std::io::copy(&mut reader, &mut file).map_err(|e| format!("Téléchargement interrompu : {e}"))?;
    file.flush().map_err(|e| e.to_string())?;
    drop(file);
    proc::command(&path)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Impossible de lancer l'installateur ({})", proc::explain(&e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions_numerically() {
        assert!(is_newer("1.0.10", "1.0.9"));
        assert!(is_newer("v1.1", "1.0.99"));
        assert!(!is_newer("1.0.0", "1.0.0"));
        assert!(!is_newer("1.0", "1.0.0"));
        assert!(!is_newer("0.9.9", "1.0.0"));
    }

    #[test]
    fn parses_github_release() {
        let json = serde_json::json!({
            "tag_name": "v1.0.7",
            "html_url": "https://github.com/x/y/releases/tag/v1.0.7",
            "assets": [{"name": ASSET, "browser_download_url": "https://example.com/a"}]
        });
        let r = parse_release(&json).unwrap();
        assert_eq!(r.version, "1.0.7");
        assert_eq!(r.asset_url.as_deref(), Some("https://example.com/a"));
    }
}
