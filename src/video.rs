//! Lecture de vidéos (image + son) en s'appuyant sur `ffmpeg` / `ffprobe` installés sur la machine.
//!
//! L'image est décodée en RGBA par un sous-processus ffmpeg et lue image par image ;
//! le son est décodé en PCM puis joué en boucle via rodio.

use rodio::buffer::SamplesBuffer;
use rodio::source::Source;
use rodio::mixer::Mixer;
use rodio::Player as AudioPlayer;
use std::io::Read;
use std::num::NonZero;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const MAX_SIDE: u32 = 480;
const AUDIO_RATE: u32 = 44_100;
/// Durée max de son décodée en mémoire (secondes).
const MAX_AUDIO_SECS: u32 = 120;

fn command(program: &str) -> Command {
    let mut c = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    c.stdin(Stdio::null()).stderr(Stdio::null());
    c
}

pub fn ffmpeg_available() -> bool {
    ["ffmpeg", "ffprobe"]
        .iter()
        .all(|p| command(p).arg("-version").stdout(Stdio::null()).status().is_ok_and(|s| s.success()))
}

pub struct Frame {
    pub size: [usize; 2],
    pub rgba: Vec<u8>,
}

pub struct VideoPlayer {
    frame: Arc<Mutex<Option<Frame>>>,
    stop: Arc<AtomicBool>,
}

impl VideoPlayer {
    pub fn start(path: PathBuf, mixer: Option<Mixer>, ctx: eframe::egui::Context) -> Self {
        let frame = Arc::new(Mutex::new(None));
        let stop = Arc::new(AtomicBool::new(false));

        if let Some(mixer) = mixer {
            let (path, stop) = (path.clone(), stop.clone());
            std::thread::spawn(move || play_audio(&path, &mixer, &stop));
        }
        {
            let (frame, stop) = (frame.clone(), stop.clone());
            std::thread::spawn(move || play_video(&path, &frame, &stop, &ctx));
        }
        Self { frame, stop }
    }

    pub fn take_frame(&self) -> Option<Frame> {
        self.frame.lock().unwrap().take()
    }
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn probe_size(path: &Path) -> Option<(u32, u32)> {
    let out = command("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=s=x:p=0"])
        .arg(path)
        .output()
        .ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    let (w, h) = text.lines().next()?.trim().split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

/// Taille de sortie : côté max `MAX_SIDE`, dimensions paires.
fn fit(w: u32, h: u32) -> (u32, u32) {
    let k = (MAX_SIDE as f32 / w.max(h) as f32).min(1.0);
    let even = |v: f32| ((v.round() as u32).max(2)) & !1;
    (even(w as f32 * k), even(h as f32 * k))
}

fn play_video(path: &Path, slot: &Mutex<Option<Frame>>, stop: &AtomicBool, ctx: &eframe::egui::Context) {
    let Some((w, h)) = probe_size(path) else { return };
    let (ow, oh) = fit(w, h);
    let Ok(mut child) = command("ffmpeg")
        .args(["-v", "quiet", "-re", "-stream_loop", "-1", "-i"])
        .arg(path)
        .args(["-an", "-vf", &format!("scale={ow}:{oh}"), "-pix_fmt", "rgba", "-f", "rawvideo", "pipe:1"])
        .stdout(Stdio::piped())
        .spawn()
    else {
        return;
    };
    let mut out = child.stdout.take().expect("piped stdout");
    let mut buf = vec![0u8; (ow * oh * 4) as usize];
    while !stop.load(Ordering::Relaxed) && out.read_exact(&mut buf).is_ok() {
        *slot.lock().unwrap() = Some(Frame { size: [ow as usize, oh as usize], rgba: buf.clone() });
        ctx.request_repaint();
    }
    let _ = child.kill();
    let _ = child.wait();
}

fn decode_audio(path: &Path) -> Vec<f32> {
    let Ok(out) = command("ffmpeg")
        .args(["-v", "quiet", "-i"])
        .arg(path)
        .args(["-vn", "-t", &MAX_AUDIO_SECS.to_string(), "-f", "f32le", "-ac", "2", "-ar", &AUDIO_RATE.to_string(), "pipe:1"])
        .output()
    else {
        return Vec::new();
    };
    out.stdout.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect()
}

fn play_audio(path: &Path, mixer: &Mixer, stop: &AtomicBool) {
    let samples = decode_audio(path);
    if samples.is_empty() || stop.load(Ordering::Relaxed) {
        return; // vidéo sans piste audio
    }
    let player = AudioPlayer::connect_new(mixer);
    let source = SamplesBuffer::new(NonZero::new(2).unwrap(), NonZero::new(AUDIO_RATE).unwrap(), samples);
    player.append(source.repeat_infinite());
    while !stop.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(50));
    }
    player.stop();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_keeps_even_dimensions_and_caps_size() {
        assert_eq!(fit(1920, 1080), (480, 270));
        assert_eq!(fit(1080, 1920), (270, 480));
        assert_eq!(fit(100, 50), (100, 50));
        let (w, h) = fit(641, 361);
        assert!(w % 2 == 0 && h % 2 == 0);
    }

    fn make_clip(dir: &Path, with_audio: bool) -> PathBuf {
        let path = dir.join(if with_audio { "a.mp4" } else { "b.mp4" });
        let mut c = command("ffmpeg");
        c.args(["-y", "-v", "quiet", "-f", "lavfi", "-i", "testsrc=size=320x180:rate=15:duration=2"]);
        if with_audio {
            c.args(["-f", "lavfi", "-i", "sine=frequency=440:duration=2"]);
        }
        c.args(["-pix_fmt", "yuv420p"]).arg(&path);
        assert!(c.status().unwrap().success());
        path
    }

    #[test]
    fn plays_frames_and_decodes_sound() {
        if !ffmpeg_available() {
            return;
        }
        let dir = std::env::temp_dir().join(format!("ckp-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let with = make_clip(&dir, true);
        let without = make_clip(&dir, false);

        assert!(!decode_audio(&with).is_empty());
        assert!(decode_audio(&without).is_empty());

        let player = VideoPlayer::start(with, None, eframe::egui::Context::default());
        let mut frame = None;
        for _ in 0..100 {
            frame = player.take_frame();
            if frame.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let f = frame.expect("a decoded frame");
        assert_eq!(f.size, [320, 180]);
        assert_eq!(f.rgba.len(), 320 * 180 * 4);
        std::fs::remove_dir_all(dir).ok();
    }
}
