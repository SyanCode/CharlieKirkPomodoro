//! Vérification des outils externes : yt-dlp (téléchargements), ffmpeg et ffprobe (vidéos).

use crate::proc;
use eframe::egui;
use std::process::Stdio;
use std::sync::{Arc, Mutex};

pub struct ToolStatus {
    pub name: &'static str,
    pub ok: bool,
    /// Version si tout va bien, sinon la raison du problème.
    pub detail: String,
}

pub type Shared = Arc<Mutex<Option<Vec<ToolStatus>>>>;

const TOOLS: [(&str, &str); 3] = [("yt-dlp", "--version"), ("ffmpeg", "-version"), ("ffprobe", "-version")];

fn probe(name: &'static str, arg: &str) -> ToolStatus {
    match proc::command(name).arg(arg).stdout(Stdio::piped()).stderr(Stdio::piped()).output() {
        Ok(o) if o.status.success() => {
            let first = String::from_utf8_lossy(&o.stdout).lines().next().unwrap_or("").trim().to_string();
            ToolStatus { name, ok: true, detail: first.chars().take(40).collect() }
        }
        Ok(o) => {
            let reason = proc::last_line(&o.stderr);
            ToolStatus { name, ok: false, detail: if reason.is_empty() { format!("code {:?}", o.status.code()) } else { reason } }
        }
        Err(e) => ToolStatus { name, ok: false, detail: proc::explain(&e) },
    }
}

pub fn check_all() -> Vec<ToolStatus> {
    TOOLS.iter().map(|(n, a)| probe(n, a)).collect()
}

/// Lance la vérification en arrière-plan ; le résultat est déposé dans `slot`.
pub fn start(ctx: egui::Context, slot: Shared) {
    *slot.lock().unwrap() = None;
    std::thread::spawn(move || {
        let result = check_all();
        *slot.lock().unwrap() = Some(result);
        ctx.request_repaint();
    });
}

pub fn video_ready(statuses: &[ToolStatus]) -> bool {
    ["ffmpeg", "ffprobe"].iter().all(|n| statuses.iter().any(|s| s.name == *n && s.ok))
}

pub fn download_ready(statuses: &[ToolStatus]) -> bool {
    statuses.iter().any(|s| s.name == "yt-dlp" && s.ok) && video_ready(statuses)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_tool_is_reported_not_found() {
        let s = probe("ckp-tool-that-does-not-exist", "--version");
        assert!(!s.ok);
        assert_eq!(s.detail, "introuvable");
    }

    #[test]
    fn readiness_needs_all_tools() {
        let mk = |n, ok| ToolStatus { name: n, ok, detail: String::new() };
        let all = vec![mk("yt-dlp", true), mk("ffmpeg", true), mk("ffprobe", true)];
        assert!(download_ready(&all));
        let missing = vec![mk("yt-dlp", true), mk("ffmpeg", true), mk("ffprobe", false)];
        assert!(!video_ready(&missing) && !download_ready(&missing));
    }
}
