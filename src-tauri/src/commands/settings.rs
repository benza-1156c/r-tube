use rtube_macros::app_command;
use tauri::State;

use crate::error::AppError;
use crate::models::settings::AppSettings;
use crate::state::AppState;

#[app_command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    let settings = state.settings.read().await;
    Ok(settings.clone())
}
