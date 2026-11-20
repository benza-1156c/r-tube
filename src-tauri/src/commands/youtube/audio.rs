use rtube_macros::app_command;
use tauri::State;

use crate::error::AppError;
use crate::models::youtube::AudioStreamInfo;
use crate::state::AppState;

/// Best audio-only stream for the lightweight listen player.
///
/// The URL is proxied so `<audio>` gets HTTP Range support and expired CDN URLs
/// are re-minted on the Rust side.
#[app_command]
pub async fn get_audio_stream(
    state: State<'_, AppState>,
    video_id: String,
) -> Result<AudioStreamInfo, AppError> {
    crate::utils::proxy::ensure_proxy()
        .await
        .map_err(|e| AppError::Internal(format!("Local stream proxy unavailable: {e}")))?;

    let language = state.settings.read().await.audio_language.clone();
    let mut info = state.yt.audio_stream(&video_id, Some(&language)).await?;
    info.url = crate::utils::proxy::wrap_audio_url(
        &info.url,
        &video_id,
        info.itag,
        info.content_length.unwrap_or(0.0) as u64,
    );
    Ok(info)
}