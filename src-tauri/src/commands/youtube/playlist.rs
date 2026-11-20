use rtube_macros::app_command;
use tauri::State;

use crate::error::AppError;
use crate::models::youtube::{PlaylistContinuationResult, PlaylistDetailView};
use crate::state::AppState;

#[app_command]
pub async fn get_playlist(
    state: State<'_, AppState>,
    playlist_id: String,
) -> Result<PlaylistDetailView, AppError> {
    state.yt.get_playlist(&playlist_id).await
}

#[app_command]
pub async fn get_playlist_continuation(
    state: State<'_, AppState>,
    continuation: String,
) -> Result<PlaylistContinuationResult, AppError> {
    state.yt.get_playlist_continuation(&continuation).await
}