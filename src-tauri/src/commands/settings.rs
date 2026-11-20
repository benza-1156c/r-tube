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

/// Mirror the settings the user just changed into the copy Rust keeps in
/// memory, so the next track is resolved with the new choice without a restart.
///
/// Persistence stays in the frontend (`plugin-store`): this command never
/// writes to the store, so playback can never overwrite a preference on its
/// own. The user is the only one who changes settings.
#[app_command]
pub async fn update_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, AppError> {
    let mut current = state.settings.write().await;
    *current = settings;
    Ok(current.clone())
}
