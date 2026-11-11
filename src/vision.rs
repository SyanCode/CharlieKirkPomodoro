//! Capture webcam + détection de visage de face (SeetaFace via `rustface`).
//!
//! Le détecteur ne reconnaît que les visages à peu près de face : si tu tournes la tête,
//! baisses la tête ou quittes l'écran, plus aucun visage n'est trouvé => "tu détournes le regard".

use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const MODEL: &[u8] = include_bytes!("../assets/seeta_fd_frontal_v1.0.bin");
/// Largeur max de l'image analysée (la capture est sous-échantillonnée).
const TARGET_WIDTH: u32 = 320;
/// Score minimal SeetaFace pour accepter un visage.
const MIN_SCORE: f64 = 2.0;
const FRAME_INTERVAL: Duration = Duration::from_millis(66);

pub struct Preview {
    pub size: [usize; 2],
    pub rgba: Vec<u8>,
}

#[derive(Clone, PartialEq)]
pub enum CamStatus {
    Starting,
    Running,
    Error(String),
}

pub struct Shared {
    pub status: CamStatus,
    /// Un visage de face est visible.
    pub looking: bool,
    pub preview: Option<Preview>,
}

pub struct Vision {
    pub shared: Arc<Mutex<Shared>>,
    stop: Arc<AtomicBool>,
}

impl Vision {
    pub fn start(ctx: eframe::egui::Context) -> Self {
        let shared = Arc::new(Mutex::new(Shared {
            status: CamStatus::Starting,
            looking: true,
            preview: None,
        }));
        let stop = Arc::new(AtomicBool::new(false));
        {
            let (shared, stop) = (shared.clone(), stop.clone());
            std::thread::spawn(move || {
                if let Err(e) = run(&ctx, &shared, &stop) {
                    let mut s = shared.lock().unwrap();
                    s.status = CamStatus::Error(e);
                    s.looking = true; // sans caméra, on ne punit pas l'utilisateur
                    s.preview = None;
                    ctx.request_repaint();
                }
            });
        }
        Self { shared, stop }
    }
}

impl Drop for Vision {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn run(ctx: &eframe::egui::Context, shared: &Mutex<Shared>, stop: &AtomicBool) -> Result<(), String> {
    let model = rustface::read_model(Cursor::new(MODEL)).map_err(|e| format!("modèle: {e}"))?;
    let mut detector = rustface::create_detector_with_model(model);
    detector.set_min_face_size(40);
    detector.set_score_thresh(MIN_SCORE);
    detector.set_pyramid_scale_factor(0.8);
    detector.set_slide_window_step(4, 4);

    let wanted = CameraFormat::new_from(640, 480, FrameFormat::MJPEG, 30);
    let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(wanted));
    let mut camera = Camera::new(CameraIndex::Index(0), format)
        .map_err(|e| format!("Impossible d'ouvrir la caméra : {e}"))?;
    camera
        .open_stream()
        .map_err(|e| format!("Impossible de démarrer la caméra : {e}"))?;
    shared.lock().unwrap().status = CamStatus::Running;

    while !stop.load(Ordering::Relaxed) {
        let started = Instant::now();
        let frame = camera.frame().map_err(|e| format!("Erreur caméra : {e}"))?;
        let img = frame
            .decode_image::<RgbFormat>()
            .map_err(|e| format!("Erreur de décodage : {e}"))?;

        let (w, h) = (img.width(), img.height());
        let step = w.div_ceil(TARGET_WIDTH).max(1) as usize;
        let (sw, sh) = (w as usize / step, h as usize / step);
        let raw = img.as_raw();
        let mut gray = Vec::with_capacity(sw * sh);
        // Aperçu miroir (comme un selfie).
        let mut rgba = vec![255u8; sw * sh * 4];
        for y in 0..sh {
            for x in 0..sw {
                let i = ((y * step) * w as usize + x * step) * 3;
                let (r, g, b) = (u32::from(raw[i]), u32::from(raw[i + 1]), u32::from(raw[i + 2]));
                gray.push(((299 * r + 587 * g + 114 * b) / 1000) as u8);
                let o = (y * sw + (sw - 1 - x)) * 4;
                rgba[o..o + 3].copy_from_slice(&raw[i..i + 3]);
            }
        }

        let image = rustface::ImageData::new(&gray, sw as u32, sh as u32);
        let looking = detector.detect(&image).iter().any(|f| f.score() >= MIN_SCORE);

        {
            let mut s = shared.lock().unwrap();
            s.looking = looking;
            s.preview = Some(Preview { size: [sw, sh], rgba });
        }
        ctx.request_repaint();

        if let Some(wait) = FRAME_INTERVAL.checked_sub(started.elapsed()) {
            std::thread::sleep(wait);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_model_loads_and_blank_image_has_no_face() {
        let model = rustface::read_model(Cursor::new(MODEL)).expect("model");
        let mut d = rustface::create_detector_with_model(model);
        d.set_min_face_size(40);
        d.set_score_thresh(MIN_SCORE);
        let blank = vec![128u8; 320 * 240];
        let faces = d.detect(&rustface::ImageData::new(&blank, 320, 240));
        assert!(faces.is_empty());
    }
}
