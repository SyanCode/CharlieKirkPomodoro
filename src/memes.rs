//! Chargement des memes depuis le dossier de l'utilisateur (avec cartes de secours).

use directories::ProjectDirs;
use eframe::egui::{ColorImage, Context, TextureHandle, TextureOptions};
use rand::seq::IndexedRandom;
use std::collections::HashMap;
use std::path::PathBuf;

const MAX_SIDE: u32 = 512;
const IMAGE_EXT: [&str; 5] = ["png", "jpg", "jpeg", "gif", "webp"];
const VIDEO_EXT: [&str; 5] = ["mp4", "webm", "mov", "mkv", "avi"];
const AUDIO_EXT: [&str; 4] = ["mp3", "wav", "ogg", "flac"];
pub const CARD_TEXTS: [&str; 4] = [
    "CHARLIE KIRK\nTE REGARDE",
    "REVIENS\nTRAVAILLER",
    "REGARDE\nL'ECRAN !",
    "TON POMODORO\nTE CHERCHE",
];

#[derive(Clone)]
pub enum Meme {
    Image(TextureHandle),
    /// Carte texte de secours (index dans `CARD_TEXTS`).
    Card(usize),
    /// Vidéo en cours de lecture (identifiant du lecteur).
    Video(u64),
}

pub struct Memes {
    pub dir: PathBuf,
    images: Vec<PathBuf>,
    pub videos: Vec<PathBuf>,
    pub audios: Vec<PathBuf>,
    cache: HashMap<usize, Option<TextureHandle>>,
}

fn has_ext(p: &std::path::Path, list: &[&str]) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| list.contains(&e.to_ascii_lowercase().as_str()))
}

impl Memes {
    pub fn load() -> Self {
        let dir = ProjectDirs::from("com", "SyanCode", "CharlieKirkPomodoro")
            .map(|d| d.data_dir().join("memes"))
            .unwrap_or_else(|| PathBuf::from("memes"));
        let _ = std::fs::create_dir_all(&dir);
        let mut all: Vec<PathBuf> =
            std::fs::read_dir(&dir).into_iter().flatten().flatten().map(|e| e.path()).collect();
        all.sort();
        let pick = |exts: &[&str]| all.iter().filter(|p| has_ext(p, exts)).cloned().collect::<Vec<_>>();
        Self {
            images: pick(&IMAGE_EXT),
            videos: pick(&VIDEO_EXT),
            audios: pick(&AUDIO_EXT),
            dir,
            cache: HashMap::new(),
        }
    }

    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    pub fn open_dir(&self) {
        let _ = open::that(&self.dir);
    }

    pub fn random_video(&self) -> Option<PathBuf> {
        self.videos.choose(&mut rand::rng()).cloned()
    }

    /// Musique de fond : uniquement s'il n'y a aucune vidéo (le son vient alors des vidéos).
    pub fn background_audio(&self) -> Option<PathBuf> {
        if self.videos.is_empty() {
            self.audios.choose(&mut rand::rng()).cloned()
        } else {
            None
        }
    }

    /// Choisit une image au hasard (décodée à la demande, puis en cache), sinon une carte texte.
    pub fn pick(&mut self, ctx: &Context) -> Meme {
        let mut rng = rand::rng();
        if let Some(&i) = (0..self.images.len()).collect::<Vec<_>>().choose(&mut rng) {
            let files = &self.images;
            let tex = self.cache.entry(i).or_insert_with(|| {
                let img = image::open(&files[i]).ok()?.thumbnail(MAX_SIDE, MAX_SIDE).to_rgba8();
                let size = [img.width() as usize, img.height() as usize];
                let color = ColorImage::from_rgba_unmultiplied(size, img.as_raw());
                Some(ctx.load_texture(format!("meme{i}"), color, TextureOptions::LINEAR))
            });
            if let Some(t) = tex {
                return Meme::Image(t.clone());
            }
        }
        let i = (0..CARD_TEXTS.len()).collect::<Vec<_>>().choose(&mut rng).copied().unwrap_or(0);
        Meme::Card(i)
    }
}
