use sea_orm::DatabaseConnection;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::settings::AppSettings;
use crate::services::youtube::YoutubeClient;

pub struct AppState {
    pub db: DatabaseConnection,
    pub app_path: PathBuf,
    pub yt: Arc<YoutubeClient>,
    pub settings: Arc<RwLock<AppSettings>>,
}

impl AppState {
    pub fn new(
        db: DatabaseConnection,
        app_path: PathBuf,
        yt: Arc<YoutubeClient>,
        settings: Arc<RwLock<AppSettings>>,
    ) -> Self {
        Self {
            db,
            app_path,
            yt,
            settings,
        }
    }
}
