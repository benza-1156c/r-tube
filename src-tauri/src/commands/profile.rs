use rtube_macros::app_command;
use tauri::State;

use crate::db::entities::profile;
use crate::error::AppError;
use crate::services::profile as profile_service;
use crate::state::AppState;

#[app_command]
pub async fn get_active_profile(
    state: State<'_, AppState>,
) -> Result<Option<profile::Model>, AppError> {
    profile_service::get_active_profile(&state.db).await
}

#[app_command]
pub async fn get_all_profiles(
    state: State<'_, AppState>,
) -> Result<Vec<profile::Model>, AppError> {
    profile_service::get_all_profiles(&state.db).await
}

#[app_command]
pub async fn upsert_profile(
    state: State<'_, AppState>,
    google_id: String,
    name: String,
    email: String,
    avatar_url: String,
    cookies: String,
) -> Result<profile::Model, AppError> {
    profile_service::upsert_profile(&state.db, google_id, name, email, avatar_url, cookies).await
}

#[app_command]
pub async fn set_active_profile(
    state: State<'_, AppState>,
    id: i32,
) -> Result<profile::Model, AppError> {
    profile_service::set_active_profile(&state.db, id).await
}

#[app_command]
pub async fn delete_profile(state: State<'_, AppState>, id: i32) -> Result<bool, AppError> {
    profile_service::delete_profile(&state.db, id).await
}
