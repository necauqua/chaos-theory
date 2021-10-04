use serde::{Serialize, Deserialize};
use ld_game_engine::util::Bitmap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredData {
    pub passed_tutorial: bool,
    pub sounds_enabled: bool,
    pub music_enabled: bool,
}

impl Default for StoredData {
    fn default() -> Self {
        Self {
            passed_tutorial: false,
            sounds_enabled: true,
            music_enabled: true,
        }
    }
}

impl StoredData {
    pub fn get_enabled_sounds(&self) -> Bitmap {
        Bitmap::empty()
            .with_set(0, self.sounds_enabled)
            .with_set(1, self.music_enabled)
    }
}
