use rtube_macros::app_command;
use tauri::State;

use crate::error::AppError;
use crate::models::youtube::{
    BrowseFeed, HomeFeed, SearchResults, SearchSuggestionItem, TrendingFeed, VideoItem,
};
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
    Ok(feed.into())
}

#[app_command]
pub async fn get_subscriptions_feed(
    state: State<'_, AppState>,
    continuation: Option<String>,
) -> Result<BrowseFeed, AppError> {
    let feed = state
        .yt
        .get_subscriptions_feed(continuation.as_deref())
        .await?;
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
    search_type: Option<String>,
    sort: Option<String>,
    upload_date: Option<String>,
) -> Result<SearchResults, AppError> {
    let res = state
        .yt
        .search_filtered(
            &query,
            continuation.as_deref(),
            search_type.as_deref(),
            sort.as_deref(),
            upload_date.as_deref(),
        )
        .await?;
    Ok(res.into())
}

#[app_command]
pub async fn get_search_suggestions(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchSuggestionItem>, AppError> {
    state.yt.get_search_suggestions(&query).await
}

/// Watch history of the signed-in account.
#[app_command]
pub async fn get_history(state: State<'_, AppState>) -> Result<Vec<VideoItem>, AppError> {
    state.yt.get_history().await
}