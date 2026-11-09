use rtube_macros::app_command;
use tauri::AppHandle;
use tauri::Manager;

use crate::db::entities::profile;
use crate::error::AppError;
use crate::services::auth as auth_service;
use crate::services::profile as profile_service;
use crate::state::AppState;

#[app_command]
pub async fn login_yt(app: AppHandle) -> Result<profile::Model, AppError> {
    auth_service::login_yt(&app).await
}

#[app_command]
pub async fn logout(app: AppHandle) -> Result<(), AppError> {
    let state = app.state::<AppState>();
    profile_service::logout(&state.db).await
}
