//! Lecture audio (musique de fond) via rodio.

use rodio::source::Source;
use rodio::mixer::Mixer;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use std::fs::File;
use std::path::Path;

pub struct Audio {
    sink: MixerDeviceSink,
}

impl Audio {
    /// `None` si aucune sortie audio n'est disponible (l'appli fonctionne alors sans son).
    pub fn open() -> Option<Self> {
        DeviceSinkBuilder::open_default_sink().ok().map(|sink| Self { sink })
    }

    pub fn mixer(&self) -> Mixer {
        self.sink.mixer().clone()
    }

    /// Joue un fichier audio en boucle jusqu'à ce que le `Player` soit arrêté ou lâché.
    pub fn play_loop(&self, path: &Path) -> Option<Player> {
        let source = Decoder::try_from(File::open(path).ok()?).ok()?;
        let player = Player::connect_new(self.sink.mixer());
        player.append(source.repeat_infinite());
        Some(player)
    }
}
