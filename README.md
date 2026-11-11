# 🍅 CharlieKirkPomodoro

Un Pomodoro (25 min de travail / 5 min de pause par défaut, réglables) qui surveille ta
webcam. Si tu détournes le regard ou la tête pendant une session de travail, l'écran est
inondé de memes de Charlie Kirk. Dès que tu regardes de nouveau l'écran, ça s'arrête.

Tout tourne dans le navigateur (MediaPipe Face Landmarker) : aucune image de la caméra
ne quitte ta machine.

## Lancer

La caméra exige `localhost` ou HTTPS :

```bash
python3 -m http.server 8000
```

Puis ouvre <http://localhost:8000> et clique sur **Démarrer** (autorise la caméra).
Le premier lancement télécharge le modèle depuis un CDN (connexion requise).

## Memes

Sans configuration, des cartes texte générées sont utilisées. Pour de vrais memes :
dépose images/vidéos dans `memes/` et liste leurs noms dans `memes/memes.json` :

```json
["meme1.jpg", "meme2.png", "clip.mp4"]
```

ou charge-les à la volée via « Mes propres memes ».

## Réglages

Les seuils de détection (`MAX_YAW`, `AWAY_DELAY_MS`, …) sont en haut de `app.js`.
Le flood n'a lieu que pendant les phases de travail, pas pendant la pause.
