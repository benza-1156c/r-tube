use rtube_macros::app_command;
use tauri::AppHandle;

use crate::db::entities::profile;
use crate::error::AppError;
use crate::services::auth as auth_service;

#[app_command]
pub async fn login_yt(app: AppHandle) -> Result<profile::Model, AppError> {
    auth_service::login_yt(&app).await
}
