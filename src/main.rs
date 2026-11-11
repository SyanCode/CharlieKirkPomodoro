#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod guard;
mod memes;
mod timer;
mod vision;

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, RichText, Vec2};
use guard::AwayGuard;
use memes::{Meme, Memes, CARD_TEXTS};
use rand::Rng;
use std::time::{Duration, Instant};
use timer::{Phase, Pomodoro};
use vision::{CamStatus, Vision};

const MAX_FLOOD_ITEMS: usize = 80;
const SPAWN_EVERY: Duration = Duration::from_millis(110);

struct FloodItem {
    meme: Meme,
    pos: Pos2,
    size: Vec2,
    angle: f32,
    hue: f32,
}

struct App {
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
    }

    fn spawn_meme(&mut self, ctx: &egui::Context, screen: Vec2) {
        let mut rng = rand::rng();
        let meme = self.memes.pick(ctx);
        let side = rng.random_range(160.0..440.0_f32);
        let aspect = match &meme {
            Meme::Image(t) => {
                let s = t.size_vec2();
                s.x / s.y.max(1.0)
            }
            Meme::Card(_) => 4.0 / 3.0,
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

        if watching {
            if self.guard.update(looking, now) {
                if now - self.last_spawn >= SPAWN_EVERY {
                    self.last_spawn = now;
                    let screen = ctx.input(|i| i.viewport().monitor_size).unwrap_or(Vec2::new(1920.0, 1080.0));
                    self.spawn_meme(ctx, screen);
                    self.spawn_meme(ctx, screen);
                }
            } else if !self.flood.is_empty() {
                self.flood.clear();
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

                    ui.add_space(12.0);
                    if ui.button("Ouvrir le dossier des memes").clicked() {
                        self.memes.open_dir();
                    }
                    ui.label(
                        RichText::new(format!(
                            "{} meme(s) trouvé(s) — dépose png/jpg/gif/webp dans le dossier, puis relance.",
                            self.memes.count()
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
