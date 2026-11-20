use rtube_macros::app_command;
use tauri::State;

use crate::error::AppError;
use crate::models::youtube::ChannelPage;
use crate::state::AppState;

/// Channel header and its Videos tab.
#[app_command]
pub async fn get_channel_page(
    state: State<'_, AppState>,
    channel_id: String,
) -> Result<ChannelPage, AppError> {
    state.yt.get_channel_page(&channel_id).await
}

/// Subscribe to or unsubscribe from a channel.
#[app_command]
pub async fn set_subscription(
    state: State<'_, AppState>,
    channel_id: String,
    subscribed: bool,
) -> Result<(), AppError> {
    state.yt.set_subscription(&channel_id, subscribed).await
}