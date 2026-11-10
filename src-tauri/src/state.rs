use sea_orm::DatabaseConnection;
use std::path::PathBuf;
use std::sync::Arc;

use crate::services::youtube::YoutubeClient;

pub struct AppState {
    pub db: DatabaseConnection,
    pub app_path: PathBuf,
    pub yt: Arc<YoutubeClient>,
}

impl AppState {
    pub fn new(db: DatabaseConnection, app_path: PathBuf, yt: Arc<YoutubeClient>) -> Self {
        Self { db, app_path, yt }
    }
}
