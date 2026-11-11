# CharlieKirkPomodoro

Application de bureau native écrite en **Rust** (Windows / macOS / Linux) : un Pomodoro
(25 min de travail / 5 min de pause par défaut, réglables) qui surveille ta webcam.
Si tu détournes la tête ou quittes l'écran pendant une session de travail, l'écran est
inondé de memes de Charlie Kirk. Dès que tu remets ton visage devant l'écran, ça s'arrête.

La détection (SeetaFace via `rustface`) tourne en local, le modèle est embarqué dans le
binaire : aucune image de la caméra ne quitte ta machine, pas besoin de connexion.

Le détecteur ne reconnaît que les visages de face : tourner la tête, la baisser ou
sortir du cadre compte comme « détourner le regard ». Le suivi des yeux seuls n'est pas géré.

## Installer (Windows)

Télécharge **CharlieKirkPomodoro-Setup.exe** dans la page
[Releases](https://github.com/SyanCode/CharlieKirkPomodoro/releases/latest), lance-le et
suis l'assistant. Il installe l'appli avec yt-dlp et ffmpeg (rien d'autre à installer) ; autorise la
caméra si Windows le demande. Windows SmartScreen peut afficher un avertissement car
l'installateur n'est pas signé : « Informations complémentaires » puis « Exécuter quand même ».

## Compiler soi-même / autres systèmes


Il faut [Rust](https://rustup.rs) (stable). Sous Linux, installe aussi les en-têtes ALSA
(`libasound2-dev` sur Debian/Ubuntu).

```bash
cargo run --release
```

Le binaire autonome est dans `target/release/charliekirkpomodoro` (`.exe` sous Windows).
Copie-le où tu veux : c'est tout ce qu'il faut. Le workflow GitHub Actions
(`.github/workflows/build.yml`) produit aussi les binaires Windows, macOS et Linux.

Sous macOS, autorise l'accès caméra au premier lancement.

## Assets par défaut

Au premier lancement, l'appli télécharge sur ta machine (rien n'est dans le dépôt) des assets
par défaut dans le dossier des memes : 5 images, 3 vidéos (phonk, funk, phonk slowed + reverb) et
une piste audio (à partir de 0:42). La liste est dans `src/defaults.rs`. Le bouton
**Télécharger les assets par défaut** relance le téléchargement (les fichiers déjà présents
sont conservés). Les images n'ont besoin de rien ; les vidéos et l'audio nécessitent
[yt-dlp](https://github.com/yt-dlp/yt-dlp) et [ffmpeg](https://ffmpeg.org/download.html) dans le PATH.
Tu peux supprimer, remplacer ou ajouter des fichiers librement.

## Memes, vidéos et son

Clique sur **Ouvrir le dossier des memes** et dépose-y tes fichiers (rechargés au prochain
téléchargement ou au prochain lancement) :

| Type | Formats | Comportement |
|---|---|---|
| Images | png, jpg, gif, webp (GIF = image fixe) | affichées en flood |
| Vidéos | mp4, webm, mov, mkv, avi | lues **avec leur son**, en boucle (3 vidéos simultanées max) |
| Audio de fond | mp3, wav, ogg, flac | joué en boucle pendant le flood |

À chaque flood, s'il y a à la fois des vidéos et de l'audio, l'appli tire au sort : soit les
vidéos (avec leur son), soit la musique de fond, pour ne jamais superposer les deux.
S'il n'y a que des images et un fichier audio, la musique joue toujours. Les vidéos
nécessitent `ffmpeg` (et `ffprobe`) pour la lecture. Sans aucun fichier, des cartes texte
générées sont utilisées.

## Réglages

Les délais de déclenchement/arrêt sont dans `src/guard.rs`, le score de détection dans
`src/vision.rs`. Le flood n'a lieu que pendant les phases de travail, pas pendant la pause.
La caméra utilisée est la première détectée (index 0).
