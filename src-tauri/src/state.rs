use std::path::PathBuf;
use sea_orm::DatabaseConnection;

pub struct AppState {
    pub db: DatabaseConnection,
    pub app_path: PathBuf,
}

impl AppState {
    pub fn new(db: DatabaseConnection, app_path: PathBuf) -> Self {
        Self { db, app_path }
    }
}
