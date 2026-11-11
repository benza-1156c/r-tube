use rtube_macros::app_command;
use tauri::State;

use crate::error::AppError;
use crate::models::youtube::{
    BrowseFeed, HomeFeed, PlayerResponse, PlaylistContinuationResult, PlaylistDetailView,
    SearchResults, SearchSuggestionItem, TrendingFeed, VideoItem, WatchNextResults,
};
use crate::services::youtube::CurrentUserProfile;
use crate::state::AppState;

#[app_command]
pub async fn get_home_feed(
    state: State<'_, AppState>,
    continuation: Option<String>,
    params: Option<String>,
) -> Result<HomeFeed, AppError> {
    let feed = state
        .yt
        .get_home_feed(continuation.as_deref(), params.as_deref())
        .await?;
    let res: HomeFeed = feed.into();
    Ok(res)
}

#[app_command]
pub async fn get_subscriptions_feed(state: State<'_, AppState>) -> Result<BrowseFeed, AppError> {
    let feed = state.yt.get_subscriptions_feed().await?;
    Ok(feed.into())
}

#[app_command]
pub async fn get_trending(state: State<'_, AppState>) -> Result<TrendingFeed, AppError> {
    let feed = state.yt.get_trending().await?;
    Ok(feed.into())
}

#[app_command]
pub async fn search(
    state: State<'_, AppState>,
    query: String,
    continuation: Option<String>,
) -> Result<SearchResults, AppError> {
    let res = state.yt.search(&query, continuation.as_deref()).await?;
    Ok(res.into())
}

#[app_command]
pub async fn get_search_suggestions(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchSuggestionItem>, AppError> {
    state.yt.get_search_suggestions(&query).await
}

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

#[app_command]
pub async fn get_video_info(
    state: State<'_, AppState>,
    video_id: String,
) -> Result<PlayerResponse, AppError> {
    let res = state.yt.get_video_info(&video_id).await;
    res
}

#[app_command]
pub async fn get_watch_next(
    state: State<'_, AppState>,
    video_id: String,
    playlist_id: Option<String>,
    playlist_index: Option<u32>,
) -> Result<WatchNextResults, AppError> {
    state
        .yt
        .get_watch_next(
            &video_id,
            playlist_id.as_deref(),
            playlist_index.map(|i| i as usize),
        )
        .await
}

#[app_command]
pub async fn get_account_info(state: State<'_, AppState>) -> Result<CurrentUserProfile, AppError> {
    state.yt.get_account_info().await
}

/// Record the playback event that puts a video into the watch history.
#[app_command]
pub async fn record_watch_start(
    state: State<'_, AppState>,
    video_id: String,
) -> Result<(), AppError> {
    state.yt.record_watch_start(&video_id).await
}

/// Report the playback position so the account remembers where it stopped.
#[app_command]
pub async fn update_watch_time(
    state: State<'_, AppState>,
    video_id: String,
    seconds: f64,
) -> Result<(), AppError> {
    state.yt.record_watch_time(&video_id, seconds).await
}

/// Watch history of the signed-in account.
#[app_command]
pub async fn get_history(state: State<'_, AppState>) -> Result<Vec<VideoItem>, AppError> {
    state.yt.get_history().await
}

#[app_command]
pub async fn get_subtitle_content(url: String) -> Result<String, AppError> {
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch subtitle: {e}")))?;
    let text = res
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read subtitle text: {e}")))?;
    Ok(text)
}
