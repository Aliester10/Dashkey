//! Audio player (soundboard) — rodio 0.22.
//!
//! Memutar file audio lokal tanpa memblokir runtime.
//! Sink dijaga tetap hidup (playback berhenti jika handle di-drop).

use std::fs::File;
use std::path::Path;
use std::sync::Mutex;

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use tracing::info;

/// Player audio yang dipakai untuk aksi `play_sound`.
pub struct AudioPlayer {
    // _sink sengaja disimpan agar output tetap hidup.
    _sink: MixerDeviceSink,
    /// Player aktif; di-retain agar suara tidak terputus saat fungsi selesai.
    players: Mutex<Vec<Player>>,
}

impl AudioPlayer {
    pub fn new() -> anyhow::Result<Self> {
        let mut sink = DeviceSinkBuilder::open_default_sink()?;
        sink.log_on_drop(false);
        Ok(Self {
            _sink: sink,
            players: Mutex::new(Vec::new()),
        })
    }

    /// Putar satu file audio (non-blocking).
    pub fn play_file(&self, path: &Path) -> anyhow::Result<()> {
        let file = File::open(path)?;
        let source = Decoder::try_from(file)?;
        let player = Player::connect_new(self._sink.mixer());
        info!(path = %path.display(), "memutar audio");
        player.append(source);
        let mut players = self.players.lock().unwrap();
        players.retain(|p| !p.empty());
        players.push(player);
        Ok(())
    }
}
