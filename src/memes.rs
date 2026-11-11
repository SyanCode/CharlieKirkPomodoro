//! Chargement des memes depuis le dossier de l'utilisateur (avec cartes de secours).

use directories::ProjectDirs;
use eframe::egui::{ColorImage, Context, TextureHandle, TextureOptions};
use rand::seq::IndexedRandom;
use std::collections::HashMap;
use std::path::PathBuf;

const MAX_SIDE: u32 = 512;
const EXTENSIONS: [&str; 5] = ["png", "jpg", "jpeg", "gif", "webp"];
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
}

pub struct Memes {
    pub dir: PathBuf,
    files: Vec<PathBuf>,
    cache: HashMap<usize, Option<TextureHandle>>,
}

impl Memes {
    pub fn load() -> Self {
        let dir = ProjectDirs::from("com", "SyanCode", "CharlieKirkPomodoro")
            .map(|d| d.data_dir().join("memes"))
            .unwrap_or_else(|| PathBuf::from("memes"));
        let _ = std::fs::create_dir_all(&dir);
        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
            })
            .collect();
        files.sort();
        Self { dir, files, cache: HashMap::new() }
    }

    pub fn count(&self) -> usize {
        self.files.len()
    }

    pub fn open_dir(&self) {
        let _ = open::that(&self.dir);
    }

    /// Choisit un meme au hasard (décode l'image à la demande, puis la garde en cache).
    pub fn pick(&mut self, ctx: &Context) -> Meme {
        let mut rng = rand::rng();
        if let Some(&i) = (0..self.files.len()).collect::<Vec<_>>().choose(&mut rng) {
            let files = &self.files;
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
