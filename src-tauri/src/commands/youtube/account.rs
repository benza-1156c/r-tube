use rtube_macros::app_command;
use tauri::State;

use crate::error::AppError;
use crate::services::youtube::CurrentUserProfile;
use crate::state::AppState;

#[app_command]
pub async fn get_account_info(state: State<'_, AppState>) -> Result<CurrentUserProfile, AppError> {
    state.yt.get_account_info().await
}