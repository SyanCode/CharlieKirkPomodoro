#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod defaults;
mod guard;
mod memes;
mod timer;
mod video;
mod vision;

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, RichText, Vec2};
use audio::Audio;
use guard::AwayGuard;
use memes::{Meme, Memes, CARD_TEXTS};
use rand::Rng;
use std::time::{Duration, Instant};
use timer::{Phase, Pomodoro};
use video::VideoPlayer;
use vision::{CamStatus, Vision};

const MAX_FLOOD_ITEMS: usize = 80;
const MAX_VIDEOS: usize = 3;
const SPAWN_EVERY: Duration = Duration::from_millis(110);

struct FloodItem {
    meme: Meme,
    pos: Pos2,
    size: Vec2,
    angle: f32,
    hue: f32,
}

struct Clip {
    id: u64,
    player: VideoPlayer,
    tex: Option<egui::TextureHandle>,
}

struct App {
    audio: Option<Audio>,
    ffmpeg: bool,
    clips: Vec<Clip>,
    next_clip_id: u64,
    background: Option<rodio::Player>,
    was_flooding: bool,
    /// Ce flood-ci utilise les vidéos (sinon : musique de fond si disponible).
    video_mode: bool,
    first_run_checked: bool,
    defaults: defaults::SharedProgress,
    pomodoro: Pomodoro,
    guard: AwayGuard,
    vision: Option<Vision>,
    memes: Memes,
    flood: Vec<FloodItem>,
    last_tick: Instant,
    last_spawn: Instant,
    preview: Option<egui::TextureHandle>,
    status_line: String,
}

impl App {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            audio: Audio::open(),
            ffmpeg: video::ffmpeg_available(),
            clips: Vec::new(),
            next_clip_id: 0,
            background: None,
            was_flooding: false,
            video_mode: true,
            first_run_checked: false,
            defaults: Default::default(),
            pomodoro: Pomodoro::new(25, 5),
            guard: AwayGuard::default(),
            vision: None,
            memes: Memes::load(),
            flood: Vec::new(),
            last_tick: now,
            last_spawn: now,
            preview: None,
            status_line: String::from("Appuie sur Démarrer"),
        }
    }

    fn start_camera(&mut self, ctx: &egui::Context) {
        if self.vision.is_none() {
            self.vision = Some(Vision::start(ctx.clone()));
        }
    }

    fn stop_flood(&mut self) {
        self.guard.reset();
        self.flood.clear();
        self.clips.clear(); // arrête image et son des vidéos
        self.background = None;
        self.was_flooding = false;
    }

    /// Choisit le mode du flood : vidéos (avec leur son) ou musique de fond.
    /// S'il y a les deux, on alterne au hasard pour ne pas superposer deux sons.
    fn start_flood_audio(&mut self) {
        let videos = self.ffmpeg && !self.memes.videos.is_empty();
        let music = self.audio.is_some() && !self.memes.audios.is_empty();
        self.video_mode = match (videos, music) {
            (true, true) => rand::rng().random_bool(0.5),
            (true, false) => true,
            (false, _) => false,
        };
        if !self.video_mode {
            if let (Some(audio), Some(path)) = (&self.audio, self.memes.background_audio()) {
                self.background = audio.play_loop(&path);
            }
        }
    }

    fn install_defaults(&mut self, ctx: &egui::Context) {
        defaults::install(self.memes.dir.clone(), ctx.clone(), self.defaults.clone());
    }

    /// Renvoie l'id d'une vidéo en cours de lecture (en démarre une nouvelle si besoin).
    fn pick_clip(&mut self, ctx: &egui::Context) -> Option<u64> {
        if !self.ffmpeg {
            return None;
        }
        let mut rng = rand::rng();
        let want_new = self.clips.is_empty() || (self.clips.len() < MAX_VIDEOS && rng.random_bool(0.3));
        if want_new {
            let path = self.memes.random_video()?;
            let mixer = self.audio.as_ref().map(Audio::mixer);
            let id = self.next_clip_id;
            self.next_clip_id += 1;
            self.clips.push(Clip { id, player: VideoPlayer::start(path, mixer, ctx.clone()), tex: None });
            return Some(id);
        }
        Some(self.clips[rng.random_range(0..self.clips.len())].id)
    }

    fn spawn_meme(&mut self, ctx: &egui::Context, screen: Vec2) {
        let mut rng = rand::rng();
        let mut rng_video = rand::rng();
        let video = if self.video_mode && !self.memes.videos.is_empty() && rng_video.random_bool(0.5) {
            self.pick_clip(ctx)
        } else {
            None
        };
        let meme = match video {
            Some(id) => Meme::Video(id),
            None => self.memes.pick(ctx),
        };
        let side = rng.random_range(160.0..440.0_f32);
        let aspect = match &meme {
            Meme::Image(t) => {
                let s = t.size_vec2();
                s.x / s.y.max(1.0)
            }
            Meme::Card(_) => 4.0 / 3.0,
            Meme::Video(_) => 16.0 / 9.0,
        };
        let size = if aspect >= 1.0 { Vec2::new(side, side / aspect) } else { Vec2::new(side * aspect, side) };
        let pos = Pos2::new(
            rng.random_range(-size.x * 0.2..(screen.x - size.x * 0.8).max(1.0)),
            rng.random_range(-size.y * 0.2..(screen.y - size.y * 0.8).max(1.0)),
        );
        let angle = rng.random_range(-0.35..0.35_f32);
        let hue = rng.random_range(0.0..1.0_f32);
        if self.flood.len() >= MAX_FLOOD_ITEMS {
            self.flood.remove(0);
        }
        self.flood.push(FloodItem { meme, pos, size, angle, hue });
    }

    /// Mise à jour de l'état (timer, caméra, flood), sans dessin.
    fn tick(&mut self, ctx: &egui::Context) {
        let changed = std::mem::take(&mut self.defaults.lock().unwrap().changed);
        if changed {
            self.memes.reload();
        }
        let now = Instant::now();
        let dt = now - self.last_tick;
        self.last_tick = now;

        if self.pomodoro.advance(dt) {
            self.stop_flood();
            ctx.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                egui::UserAttentionType::Critical,
            ));
        }

        let watching = self.pomodoro.running && self.pomodoro.phase == Phase::Work;
        let mut looking = true;
        if let Some(v) = &self.vision {
            let mut s = v.shared.lock().unwrap();
            looking = s.looking;
            self.status_line = match &s.status {
                CamStatus::Starting => "Démarrage de la caméra…".into(),
                CamStatus::Running if !watching => "Caméra active (en attente)".into(),
                CamStatus::Running if looking => "Tu regardes l'écran, continue.".into(),
                CamStatus::Running => "Reviens devant l'écran !".into(),
                CamStatus::Error(e) => e.clone(),
            };
            if let Some(p) = s.preview.take() {
                let img = egui::ColorImage::from_rgba_unmultiplied(p.size, &p.rgba);
                match &mut self.preview {
                    Some(t) => t.set(img, egui::TextureOptions::LINEAR),
                    None => self.preview = Some(ctx.load_texture("cam", img, egui::TextureOptions::LINEAR)),
                }
            }
        }

        for clip in &mut self.clips {
            if let Some(f) = clip.player.take_frame() {
                let img = egui::ColorImage::from_rgba_unmultiplied(f.size, &f.rgba);
                match &mut clip.tex {
                    Some(t) => t.set(img, egui::TextureOptions::LINEAR),
                    None => clip.tex = Some(ctx.load_texture(format!("clip{}", clip.id), img, egui::TextureOptions::LINEAR)),
                }
            }
        }

        if watching {
            let flooding = self.guard.update(looking, now);
            if flooding && !self.was_flooding {
                self.was_flooding = true;
                self.start_flood_audio();
            }
            if flooding {
                if now - self.last_spawn >= SPAWN_EVERY {
                    self.last_spawn = now;
                    let screen = ctx.input(|i| i.viewport().monitor_size).unwrap_or(Vec2::new(1920.0, 1080.0));
                    self.spawn_meme(ctx, screen);
                    self.spawn_meme(ctx, screen);
                }
            } else if self.was_flooding || !self.flood.is_empty() {
                self.stop_flood();
            }
        } else {
            self.stop_flood();
        }
    }

    fn show_flood(&self, ctx: &egui::Context) {
        let screen = ctx.input(|i| i.viewport().monitor_size).unwrap_or(Vec2::new(1920.0, 1080.0));
        let builder = egui::ViewportBuilder::default()
            .with_title("CharlieKirkPomodoro - flood")
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_mouse_passthrough(true)
            .with_taskbar(false)
            .with_position(Pos2::ZERO)
            .with_inner_size(screen);
        ctx.show_viewport_immediate(egui::ViewportId::from_hash_of("flood"), builder, |ui, _class| {
            egui::CentralPanel::default().frame(egui::Frame::new()).show(ui, |ui| {
                let painter = ui.painter();
                for it in &self.flood {
                    let rect = Rect::from_min_size(it.pos, it.size);
                    match &it.meme {
                        Meme::Image(tex) => {
                            egui::Image::new(tex)
                                .fit_to_exact_size(it.size)
                                .rotate(it.angle, Vec2::splat(0.5))
                                .paint_at(ui, rect);
                        }
                        Meme::Video(id) => {
                            if let Some(tex) = self.clips.iter().find(|c| c.id == *id).and_then(|c| c.tex.as_ref()) {
                                egui::Image::new(tex).fit_to_exact_size(it.size).paint_at(ui, rect);
                            }
                        }
                        Meme::Card(i) => {
                            let bg: Color32 = egui::ecolor::Hsva::new(it.hue, 0.7, 0.45, 1.0).into();
                            painter.rect_filled(rect, 8.0, bg);
                            painter.text(
                                rect.center(),
                                Align2::CENTER_CENTER,
                                CARD_TEXTS[*i],
                                FontId::proportional(it.size.y * 0.14),
                                Color32::WHITE,
                            );
                        }
                    }
                }
            });
        });
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick(ctx);
        let delay = if self.flood.is_empty() { 100 } else { 16 };
        ctx.request_repaint_after(Duration::from_millis(delay));
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if !self.first_run_checked {
            self.first_run_checked = true;
            if !defaults::already_installed(&self.memes.dir) {
                self.install_defaults(&ctx);
            }
        }
        let bg = match self.pomodoro.phase {
            Phase::Work => Color32::from_rgb(179, 38, 30),
            Phase::Break => Color32::from_rgb(30, 122, 70),
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(bg).inner_margin(16.0))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("CharlieKirkPomodoro").size(22.0).strong().color(Color32::WHITE));
                    ui.label(RichText::new(self.pomodoro.phase.label()).size(16.0).color(Color32::WHITE));
                    let secs = self.pomodoro.remaining_secs();
                    ui.label(
                        RichText::new(format!("{:02}:{:02}", secs / 60, secs % 60))
                            .size(96.0)
                            .strong()
                            .monospace()
                            .color(Color32::WHITE),
                    );

                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - 250.0).max(0.0) / 2.0);
                        let label = if self.pomodoro.running { "Pause" } else { "Démarrer" };
                        if ui.add_sized([120.0, 36.0], egui::Button::new(label)).clicked() {
                            self.pomodoro.toggle();
                            self.last_tick = Instant::now();
                            if self.pomodoro.running {
                                self.start_camera(&ctx);
                            }
                        }
                        if ui.add_sized([120.0, 36.0], egui::Button::new("Réinitialiser")).clicked() {
                            self.pomodoro.reset();
                            self.stop_flood();
                        }
                    });

                    ui.add_space(8.0);
                    ui.add_enabled_ui(!self.pomodoro.running, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space((ui.available_width() - 330.0).max(0.0) / 2.0);
                            ui.label(RichText::new("Travail (min)").color(Color32::WHITE));
                            let a = ui.add(egui::DragValue::new(&mut self.pomodoro.work_min).range(1..=180));
                            ui.label(RichText::new("Pause (min)").color(Color32::WHITE));
                            let b = ui.add(egui::DragValue::new(&mut self.pomodoro.break_min).range(1..=60));
                            if a.changed() || b.changed() {
                                self.pomodoro.apply_settings();
                            }
                        });
                    });

                    ui.add_space(12.0);
                    if let Some(tex) = &self.preview {
                        let w = ui.available_width().min(260.0);
                        let h = w * tex.size_vec2().y / tex.size_vec2().x.max(1.0);
                        ui.add(egui::Image::new(tex).fit_to_exact_size(Vec2::new(w, h)).corner_radius(10.0));
                    }
                    ui.label(RichText::new(&self.status_line).color(Color32::WHITE));
                    if matches!(self.vision.as_ref().map(|v| v.shared.lock().unwrap().status.clone()),
                        Some(CamStatus::Error(_)))
                        && ui.button("Réessayer").clicked()
                    {
                        self.vision = None;
                        self.start_camera(&ctx);
                    }

                    if !self.memes.videos.is_empty() && !self.ffmpeg {
                        ui.label(
                            RichText::new("Vidéos ignorées : installe ffmpeg (ffmpeg + ffprobe dans le PATH).")
                                .color(Color32::YELLOW),
                        );
                    }
                    if self.audio.is_none() {
                        ui.label(RichText::new("Aucune sortie audio détectée.").color(Color32::YELLOW));
                    }
                    ui.add_space(12.0);
                    {
                        let p = self.defaults.lock().unwrap();
                        let (running, msg) = (p.running, p.message.clone());
                        drop(p);
                        if !msg.is_empty() {
                            ui.label(RichText::new(msg).small().color(Color32::from_white_alpha(220)));
                        }
                        if !running && ui.button("Télécharger les assets par défaut").clicked() {
                            self.install_defaults(&ctx);
                        }
                    }
                    if ui.button("Ouvrir le dossier des memes").clicked() {
                        self.memes.open_dir();
                    }
                    ui.label(
                        RichText::new(format!(
                            "{} image(s), {} vidéo(s), {} audio — dépose-les dans le dossier, puis relance.\n\
                             Images : png jpg gif webp · Vidéos (avec son) : mp4 webm mov mkv avi · \
                             Audio de fond (si aucune vidéo) : mp3 wav ogg flac",
                            self.memes.image_count(),
                            self.memes.videos.len(),
                            self.memes.audios.len()
                        ))
                        .small()
                        .color(Color32::from_white_alpha(200)),
                    );
                });
            });

        if !self.flood.is_empty() {
            self.show_flood(&ctx);
        }
    }
}

fn main() -> eframe::Result {
    #[cfg(target_os = "macos")]
    nokhwa::nokhwa_initialize(|_| {});

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("CharlieKirkPomodoro")
            .with_inner_size([440.0, 720.0])
            .with_min_inner_size([380.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native("CharlieKirkPomodoro", options, Box::new(|_cc| Ok(Box::new(App::new()))))
}
