use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "kebab-case")]
pub enum PlaybackMode {
    Normal,
    AudioOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub volume: f64,
    pub language: String,
    pub default_resolution: String,
    pub playback_mode: PlaybackMode,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            language: "th".to_string(),
            default_resolution: "1080p".to_string(),
            playback_mode: PlaybackMode::Normal,
        }
    }
}
