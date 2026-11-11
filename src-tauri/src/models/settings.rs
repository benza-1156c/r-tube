use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::error::AppError;

pub const SETTINGS_STORE_PATH: &str = "settings.json";
pub const SETTINGS_KEY: &str = "settings";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "kebab-case")]
pub enum PlaybackMode {
    Normal,
    AudioOnly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub volume: f64,
    #[serde(alias = "language")]
    pub audio_language: String,
    pub default_resolution: String,
    pub playback_mode: PlaybackMode,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            audio_language: "en".to_string(),
            default_resolution: "1080p".to_string(),
            playback_mode: PlaybackMode::Normal,
        }
    }
}

impl AppSettings {
    /// Load settings from the persistent store, or return defaults if not found
    pub fn load_from_store(app: &AppHandle) -> Self {
        let Ok(store) = app.store(SETTINGS_STORE_PATH) else {
            return Self::default();
        };

        if let Some(val) = store.get(SETTINGS_KEY) {
            if let Ok(settings) = serde_json::from_value(val) {
                return settings;
            }
        }

        let default_settings = Self::default();
        if let Ok(json_val) = serde_json::to_value(&default_settings) {
            store.set(SETTINGS_KEY, json_val);
            let _ = store.save();
        }
        default_settings
    }

    /// Persist current settings to the store
    pub fn save_to_store(&self, app: &AppHandle) -> Result<(), AppError> {
        let store = app
            .store(SETTINGS_STORE_PATH)
            .map_err(|e| AppError::Internal(format!("Failed to open settings store: {e}")))?;

        let json_val = serde_json::to_value(self).map_err(|e| AppError::Internal(e.to_string()))?;

        store.set(SETTINGS_KEY, json_val);
        store
            .save()
            .map_err(|e| AppError::Internal(format!("Failed to save settings store: {e}")))?;

        Ok(())
    }
}
