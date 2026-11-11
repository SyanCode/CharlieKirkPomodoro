import {
  FaceLandmarker, FilesetResolver,
} from "../vendor/mediapipe/vision_bundle.mjs";

const WASM = new URL("../vendor/mediapipe/wasm", import.meta.url).href;
const MODEL = new URL("../vendor/mediapipe/face_landmarker.task", import.meta.url).href;

// Seuils de détournement du regard / de la tête
const MAX_YAW = 0.22;        // 0 = face, ±1 = profil
const MAX_PITCH_UP = 0.35;
const MAX_PITCH_DOWN = 0.75;
const AWAY_DELAY_MS = 700;   // temps détourné avant le flood
const BACK_DELAY_MS = 250;   // temps de retour avant l'arrêt

const $ = (id) => document.getElementById(id);
const els = {
  timer: $("timer"), phase: $("phase"), start: $("startBtn"), reset: $("resetBtn"),
  work: $("workMin"), brk: $("breakMin"), video: $("video"), status: $("status"),
  flood: $("flood"),
};

/* ---------------- Pomodoro ---------------- */
const state = { phase: "work", remaining: 25 * 60, running: false, tick: null, endAt: 0 };

const fmt = (s) => `${String(Math.floor(s / 60)).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`;
const phaseSeconds = (p) => Math.max(1, Number(p === "work" ? els.work.value : els.brk.value) || 1) * 60;

function render() {
  els.timer.textContent = fmt(state.remaining);
  els.phase.textContent = state.phase === "work" ? "Travail" : "Pause";
  els.start.textContent = state.running ? "Pause" : "Démarrer";
  document.body.className = state.phase;
  document.title = `${fmt(state.remaining)} · CharlieKirkPomodoro`;
}

function beep() {
  try {
    const ctx = new AudioContext();
    const o = ctx.createOscillator();
    o.connect(ctx.destination);
    o.frequency.value = 880;
    o.start();
    o.stop(ctx.currentTime + 0.4);
  } catch {}
}

function setPhase(p) {
  state.phase = p;
  state.remaining = phaseSeconds(p);
  if (p === "break") stopFlood();
  render();
}

function loop() {
  state.remaining = Math.max(0, Math.round((state.endAt - Date.now()) / 1000));
  if (state.remaining === 0) {
    beep();
    setPhase(state.phase === "work" ? "break" : "work");
    state.endAt = Date.now() + state.remaining * 1000;
  }
  render();
}

function startTimer() {
  state.running = true;
  state.endAt = Date.now() + state.remaining * 1000;
  state.tick = setInterval(loop, 250);
  startCamera();
  render();
}
function pauseTimer() {
  state.running = false;
  clearInterval(state.tick);
  stopFlood();
  render();
}

els.start.onclick = () => (state.running ? pauseTimer() : startTimer());
els.reset.onclick = () => { pauseTimer(); setPhase("work"); };
for (const input of [els.work, els.brk]) {
  input.onchange = () => { if (!state.running) setPhase(state.phase); };
}

/* ---------------- Memes ---------------- */
let memes = [];

function cardDataUrl(text, hue) {
  const c = document.createElement("canvas");
  c.width = 480; c.height = 360;
  const g = c.getContext("2d");
  g.fillStyle = `hsl(${hue} 70% 35%)`;
  g.fillRect(0, 0, c.width, c.height);
  g.font = "110px serif"; g.textAlign = "center";
  g.fillText("👀", 240, 140);
  g.fillStyle = "#fff"; g.font = "bold 38px Impact, sans-serif";
  text.split("\n").forEach((line, i) => g.fillText(line, 240, 230 + i * 48));
  return c.toDataURL();
}

async function loadMemes() {
  const files = await window.pomodoro.listMemes();
  memes = files.map((src) => ({ src, video: /\.(mp4|webm)$/i.test(src) }));
  if (!memes.length) {
    const lines = ["CHARLIE KIRK\nTE REGARDE", "REVIENS\nTRAVAILLER", "REGARDE\nL'ÉCRAN !", "TON POMODORO\nTE CHERCHE"];
    memes = lines.map((t, i) => ({ src: cardDataUrl(t, i * 70 + 10), video: false }));
  }
}

$("memesBtn").onclick = () => window.pomodoro.openMemesDir();

let floodTimer = null;
function spawnMeme() {
  const m = memes[Math.floor(Math.random() * memes.length)];
  const el = document.createElement(m.video ? "video" : "img");
  el.className = "meme";
  el.src = m.src;
  if (m.video) { el.autoplay = true; el.loop = true; el.muted = true; }
  const w = 160 + Math.random() * 280;
  const rot = (Math.random() * 40 - 20).toFixed(1) + "deg";
  el.style.cssText = `width:${w}px;left:${Math.random() * (innerWidth - w * 0.6) - w * 0.2}px;` +
    `top:${Math.random() * (innerHeight - w * 0.5) - w * 0.1}px;transform:rotate(${rot});--rot:${rot}`;
  els.flood.appendChild(el);
  while (els.flood.children.length > 80) els.flood.firstChild.remove();
}
function startFlood() {
  if (floodTimer) return;
  spawnMeme();
  floodTimer = setInterval(() => { spawnMeme(); spawnMeme(); }, 120);
}
function stopFlood() {
  clearInterval(floodTimer);
  floodTimer = null;
  els.flood.replaceChildren();
}

/* ---------------- Détection du regard ---------------- */
let landmarker = null;
let cameraStarted = false;
let awaySince = null, backSince = null, flooding = false;

// Estime l'orientation de la tête à partir des landmarks (coordonnées normalisées).
function headPose(lm) {
  const nose = lm[1], left = lm[234], right = lm[454], top = lm[10], chin = lm[152];
  const faceW = right.x - left.x || 1e-6;
  const yaw = ((nose.x - left.x) / faceW - 0.5) * 2;           // -1..1
  const faceH = chin.y - top.y || 1e-6;
  const pitch = ((nose.y - top.y) / faceH - 0.55) * 2;         // + = vers le bas
  return { yaw, pitch };
}

function isLookingAtScreen(result) {
  if (!result.faceLandmarks.length) return false;
  const { yaw, pitch } = headPose(result.faceLandmarks[0]);
  if (Math.abs(yaw) > MAX_YAW) return false;
  if (pitch < -MAX_PITCH_UP || pitch > MAX_PITCH_DOWN) return false;
  // Regard (yeux) via les blendshapes : iris tournés sur le côté / vers le haut
  const b = Object.fromEntries((result.faceBlendshapes[0]?.categories ?? []).map((c) => [c.categoryName, c.score]));
  const side = Math.max(
    (b.eyeLookOutLeft ?? 0) + (b.eyeLookInRight ?? 0),
    (b.eyeLookInLeft ?? 0) + (b.eyeLookOutRight ?? 0),
  ) / 2;
  const up = ((b.eyeLookUpLeft ?? 0) + (b.eyeLookUpRight ?? 0)) / 2;
  return side < 0.55 && up < 0.6;
}

function setStatus(t) { els.status.textContent = t; }

function update(looking, now) {
  const active = state.running && state.phase === "work";
  if (!active) { awaySince = backSince = null; if (flooding) { flooding = false; stopFlood(); } return; }
  if (looking) {
    awaySince = null;
    backSince ??= now;
    if (flooding && now - backSince >= BACK_DELAY_MS) { flooding = false; stopFlood(); setStatus("✅ Bien joué, continue"); }
    else if (!flooding) setStatus("✅ Tu regardes l'écran");
  } else {
    backSince = null;
    awaySince ??= now;
    if (!flooding && now - awaySince >= AWAY_DELAY_MS) { flooding = true; startFlood(); }
    if (flooding) setStatus("🚨 Reviens devant l'écran !");
  }
}

function detectLoop() {
  if (els.video.readyState >= 2) {
    const now = performance.now();
    const looking = isLookingAtScreen(landmarker.detectForVideo(els.video, now));
    update(looking, now);
  }
  requestAnimationFrame(detectLoop);
}

async function startCamera() {
  if (cameraStarted) return;
  cameraStarted = true;
  try {
    setStatus("Chargement du détecteur de visage…");
    const fileset = await FilesetResolver.forVisionTasks(WASM);
    landmarker = await FaceLandmarker.createFromOptions(fileset, {
      baseOptions: { modelAssetPath: MODEL, delegate: "GPU" },
      runningMode: "VIDEO", numFaces: 1, outputFaceBlendshapes: true,
    });
    const stream = await navigator.mediaDevices.getUserMedia({ video: { width: 640, height: 480, facingMode: "user" } });
    els.video.srcObject = stream;
    await els.video.play();
    setStatus("Caméra active");
    detectLoop();
  } catch (e) {
    cameraStarted = false;
    console.error(e);
    setStatus("⚠️ Caméra/détecteur indisponible : " + (e.message || e));
  }
}

await loadMemes();
render();
