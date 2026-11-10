use rtube_macros::app_command;
use tauri::State;

use crate::error::AppError;
use crate::models::youtube::{
    BrowseFeed, HomeFeed, PlayerResponse, SearchResults, TrendingFeed, WatchNextResults,
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
pub async fn get_video_info(
    state: State<'_, AppState>,
    video_id: String,
) -> Result<PlayerResponse, AppError> {
    state.yt.get_video_info(&video_id).await
}

#[app_command]
pub async fn get_watch_next(
    state: State<'_, AppState>,
    video_id: String,
) -> Result<WatchNextResults, AppError> {
    let next = state.yt.get_watch_next(&video_id).await?;
    Ok(next.into())
}

#[app_command]
pub async fn get_account_info(state: State<'_, AppState>) -> Result<CurrentUserProfile, AppError> {
    state.yt.get_account_info().await
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
