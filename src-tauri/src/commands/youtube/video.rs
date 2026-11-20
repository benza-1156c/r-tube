use rtube_macros::app_command;
use tauri::State;

use crate::error::AppError;
use crate::models::youtube::{
    CommentsPage, PlayerResponse, VideoRating, WatchNextResults,
};
use crate::state::AppState;

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

/// One page of comments for a video.
#[app_command]
pub async fn get_comments(
    state: State<'_, AppState>,
    video_id: String,
    continuation: Option<String>,
) -> Result<CommentsPage, AppError> {
    state
        .yt
        .get_comments(&video_id, continuation.as_deref())
        .await
}

/// Like / dislike / clear a rating.
#[app_command]
pub async fn rate_video(
    state: State<'_, AppState>,
    video_id: String,
    rating: VideoRating,
) -> Result<(), AppError> {
    state.yt.rate_video(&video_id, rating).await
}

#[app_command]
pub async fn get_subtitle_content(url: String) -> Result<String, AppError> {
    let client = crate::utils::http::client();
    let res = client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .header("Referer", "https://www.youtube.com/")
        .header("Origin", "https://www.youtube.com")
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch subtitle: {e}")))?;

    if !res.status().is_success() {
        return Err(AppError::Internal(format!(
            "Subtitle request failed with HTTP {}",
            res.status()
        )));
    }

    let text = res
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read subtitle text: {e}")))?;

    // The URL asks for `fmt=vtt`; anything else (an error page, XML) becomes
    // an invalid caption track in the player.
    if !text.trim_start().starts_with("WEBVTT") {
        return Err(AppError::Internal(
            "Subtitle response was not WebVTT".to_string(),
        ));
    }

    Ok(text)
}