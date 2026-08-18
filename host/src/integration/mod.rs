//! Integration modules — OBS WebSocket client, audio player, SFX importer.

pub mod audio;
pub mod obs;
pub mod sfx;

pub use audio::AudioPlayer;
pub use obs::{ObsManager, ObsSettings};
