# CharlieKirkPomodoro

Application de bureau native écrite en **Rust** (Windows / macOS / Linux) : un Pomodoro
(25 min de travail / 5 min de pause par défaut, réglables) qui surveille ta webcam.
Si tu détournes la tête ou quittes l'écran pendant une session de travail, l'écran est
inondé de memes de Charlie Kirk. Dès que tu remets ton visage devant l'écran, ça s'arrête.

La détection (SeetaFace via `rustface`) tourne en local, le modèle est embarqué dans le
binaire : aucune image de la caméra ne quitte ta machine, pas besoin de connexion.

Le détecteur ne reconnaît que les visages de face : tourner la tête, la baisser ou
sortir du cadre compte comme « détourner le regard ». Le suivi des yeux seuls n'est pas géré.

## Compiler / lancer

Il faut [Rust](https://rustup.rs) (stable). Sous Linux, installe aussi les en-têtes ALSA
(`libasound2-dev` sur Debian/Ubuntu).

```bash
cargo run --release
```

Le binaire autonome est dans `target/release/charliekirkpomodoro` (`.exe` sous Windows).
Copie-le où tu veux : c'est tout ce qu'il faut. Le workflow GitHub Actions
(`.github/workflows/build.yml`) produit aussi les binaires Windows, macOS et Linux.

Sous macOS, autorise l'accès caméra au premier lancement.

## Memes, vidéos et son

Clique sur **Ouvrir le dossier des memes** et dépose-y tes fichiers, puis relance l'appli :

| Type | Formats | Comportement |
|---|---|---|
| Images | png, jpg, gif, webp (GIF = image fixe) | affichées en flood |
| Vidéos | mp4, webm, mov, mkv, avi | lues **avec leur son**, en boucle (3 vidéos simultanées max, dupliquées à l'écran) |
| Audio de fond | mp3, wav, ogg, flac | joué en boucle pendant le flood, **seulement s'il n'y a aucune vidéo** |

Les vidéos nécessitent [ffmpeg](https://ffmpeg.org/download.html) (`ffmpeg` et `ffprobe` dans le
PATH) ; sans lui elles sont ignorées et l'appli l'indique. Sans aucun fichier, des cartes
texte générées sont utilisées.

## Réglages

Les délais de déclenchement/arrêt sont dans `src/guard.rs`, le score de détection dans
`src/vision.rs`. Le flood n'a lieu que pendant les phases de travail, pas pendant la pause.
La caméra utilisée est la première détectée (index 0).
