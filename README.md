# 🍅 CharlieKirkPomodoro

Application de bureau (Windows / macOS / Linux) : un Pomodoro (25 min de travail /
5 min de pause par défaut, réglables) qui surveille ta webcam. Si tu détournes le regard
ou la tête pendant une session de travail, l'écran est inondé de memes de Charlie Kirk.
Dès que tu regardes de nouveau l'écran, ça s'arrête.

La détection (MediaPipe Face Landmarker) tourne en local et hors ligne : aucune image
de la caméra ne quitte ta machine.

## Lancer en développement

```bash
npm install
npm start
```

## Créer l'installateur

```bash
npm run dist
```

L'installateur apparaît dans `dist/` (`.exe` sous Windows, `.dmg` sous macOS,
`.AppImage` / `.deb` sous Linux). Lance la commande sur l'OS visé.

## Memes

Sans configuration, des cartes texte générées sont utilisées. Pour de vrais memes :
menu **Fichier → Ouvrir le dossier des memes** (ou le bouton dans l'appli), dépose-y tes
images/vidéos (png, jpg, gif, webp, mp4, webm) et relance l'appli.

## Réglages

Les seuils de détection (`MAX_YAW`, `AWAY_DELAY_MS`, …) sont en haut de `src/app.js`.
Le flood n'a lieu que pendant les phases de travail, pas pendant la pause.
