//! Assets par défaut : téléchargés sur la machine de l'utilisateur dans le dossier des memes
//! (rien n'est embarqué dans le dépôt). L'utilisateur peut ensuite les supprimer, les
//! remplacer ou en ajouter d'autres : un fichier déjà présent n'est jamais retéléchargé.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36";

pub enum Kind {
    Image,
    /// Vidéo complète (480p max).
    Video,
    /// Piste audio seule, à partir de `start`.
    Audio { start: &'static str },
}

pub struct Asset {
    pub kind: Kind,
    pub url: &'static str,
    /// Nom de fichier (sans extension pour la vidéo et l'audio).
    pub name: &'static str,
}

pub const DEFAULTS: &[Asset] = &[
    // Audio de fond (à partir de 0:42)
    Asset { kind: Kind::Audio { start: "0:42" }, url: "https://www.youtube.com/watch?v=ZUHpUXlsvbI", name: "default-audio" },
    // Vidéos
    Asset { kind: Kind::Video, url: "https://www.youtube.com/watch?v=7R5ncn93KT4", name: "default-phonk" },
    Asset { kind: Kind::Video, url: "https://www.youtube.com/watch?v=Pb7pjz8CqyE", name: "default-funk" },
    Asset { kind: Kind::Video, url: "https://www.youtube.com/watch?v=1Ir4BefHU4o", name: "default-phonk-slowed-reverb" },
    // Images
    Asset { kind: Kind::Image, url: "https://www.maltepedentalclinic.com/wp-content/uploads/2026/07/charlie-kirk-smile-1024x576.webp", name: "default-image-1.webp" },
    Asset { kind: Kind::Image, url: "https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcRRffoWVhjTWQSSAj3kS5BhdHEDqXvroQUQyw8DCXjHQA&s=10", name: "default-image-2.jpg" },
    Asset { kind: Kind::Image, url: "https://i.pinimg.com/736x/95/16/37/951637767fac7226bb2d82d47e3fe33a.jpg", name: "default-image-3.jpg" },
    Asset { kind: Kind::Image, url: "https://media.tenor.com/UgqVjSHIq9sAAAAe/charlie-kirk-charlie-kirk-meme.png", name: "default-image-4.png" },
    Asset { kind: Kind::Image, url: "https://i.redd.it/18ttzjr35cze1.jpg", name: "default-image-5.jpg" },
];

#[derive(Default)]
pub struct Progress {
    pub running: bool,
    pub message: String,
    /// Vrai quand de nouveaux fichiers sont arrivés (l'appli doit recharger le dossier).
    pub changed: bool,
}

pub type SharedProgress = Arc<Mutex<Progress>>;

fn marker(dir: &Path) -> PathBuf {
    dir.join(".defaults-installed")
}

/// Mémorise que la question a été traitée (acceptée ou refusée) : on ne la reposera pas.
pub fn mark_done(dir: &Path) {
    let _ = std::fs::write(marker(dir), "ok");
}

pub fn already_installed(dir: &Path) -> bool {
    marker(dir).exists()
}

/// Un fichier "name" (avec ou sans extension) existe-t-il déjà dans `dir` ?
fn exists(dir: &Path, name: &str) -> bool {
    let stem = Path::new(name).file_stem().and_then(|s| s.to_str()).unwrap_or(name);
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| e.path().file_stem().and_then(|s| s.to_str()) == Some(stem))
}

fn tool_available(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn download_image(url: &str, dest: &Path) -> Result<(), String> {
    let mut resp = ureq::get(url).header("User-Agent", USER_AGENT).call().map_err(|e| e.to_string())?;
    let bytes = resp.body_mut().read_to_vec().map_err(|e| e.to_string())?;
    image::load_from_memory(&bytes).map_err(|e| format!("image invalide: {e}"))?;
    std::fs::write(dest, bytes).map_err(|e| e.to_string())
}

fn yt_dlp(dir: &Path, args: &[&str], url: &str) -> Result<(), String> {
    let status = Command::new("yt-dlp")
        .current_dir(dir)
        .args(["--no-playlist", "--quiet", "--no-warnings"])
        .args(args)
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| e.to_string())?;
    status.success().then_some(()).ok_or_else(|| "yt-dlp a échoué".to_string())
}

fn fetch(asset: &Asset, dir: &Path) -> Result<(), String> {
    match &asset.kind {
        Kind::Image => download_image(asset.url, &dir.join(asset.name)),
        Kind::Video => yt_dlp(
            dir,
            &["-f", "bv*[height<=480]+ba/b[height<=480]/b", "--merge-output-format", "mp4", "-o", &format!("{}.%(ext)s", asset.name)],
            asset.url,
        ),
        Kind::Audio { start } => yt_dlp(
            dir,
            &[
                "-x", "--audio-format", "mp3",
                "--download-sections", &format!("*{start}-inf"),
                "-o", &format!("{}.%(ext)s", asset.name),
            ],
            asset.url,
        ),
    }
}

/// Télécharge en arrière-plan les assets manquants.
pub fn install(dir: PathBuf, ctx: eframe::egui::Context, progress: SharedProgress) {
    {
        let mut p = progress.lock().unwrap();
        if p.running {
            return;
        }
        p.running = true;
        p.message = "Téléchargement des assets par défaut…".into();
    }
    std::thread::spawn(move || {
        let set = |msg: String, changed: bool| {
            let mut p = progress.lock().unwrap();
            p.message = msg;
            p.changed |= changed;
            ctx.request_repaint();
        };
        let have_yt = tool_available("yt-dlp");
        let mut failures: Vec<String> = Vec::new();
        let total = DEFAULTS.len();
        for (i, asset) in DEFAULTS.iter().enumerate() {
            if exists(&dir, asset.name) {
                continue;
            }
            set(format!("Téléchargement {}/{} : {}", i + 1, total, asset.name), false);
            let result = match asset.kind {
                Kind::Image => fetch(asset, &dir),
                _ if have_yt => fetch(asset, &dir),
                _ => Err("yt-dlp introuvable".into()),
            };
            match result {
                Ok(()) => set(format!("{} téléchargé", asset.name), true),
                Err(e) => failures.push(format!("{} ({e})", asset.name)),
            }
        }
        let mut p = progress.lock().unwrap();
        p.running = false;
        p.changed = true;
        if failures.is_empty() {
            let _ = std::fs::write(marker(&dir), "ok");
            p.message = "Assets par défaut installés.".into();
        } else {
            p.message = format!(
                "Échecs : {}. Les vidéos/audio nécessitent yt-dlp et ffmpeg dans le PATH.",
                failures.join(", ")
            );
        }
        ctx.request_repaint();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exists_ignores_extension() {
        let dir = std::env::temp_dir().join(format!("ckp-defaults-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("default-phonk.webm"), b"x").unwrap();
        assert!(exists(&dir, "default-phonk"));
        assert!(!exists(&dir, "default-funk"));
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn manifest_has_expected_assets() {
        let count = |f: fn(&Kind) -> bool| DEFAULTS.iter().filter(|a| f(&a.kind)).count();
        assert_eq!(count(|k| matches!(k, Kind::Image)), 5);
        assert_eq!(count(|k| matches!(k, Kind::Video)), 3);
        assert_eq!(count(|k| matches!(k, Kind::Audio { .. })), 1);
    }

    /// Test réseau : `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn downloads_default_images() {
        let dir = std::env::temp_dir().join(format!("ckp-net-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for a in DEFAULTS.iter().filter(|a| matches!(a.kind, Kind::Image)) {
            download_image(a.url, &dir.join(a.name)).unwrap_or_else(|e| panic!("{}: {e}", a.name));
        }
        std::fs::remove_dir_all(dir).ok();
    }
}
