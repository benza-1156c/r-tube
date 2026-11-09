use std::time::Duration;

use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tokio::time::sleep;

use crate::AppState;
use crate::db::entities::profile;
use crate::error::AppError;
use crate::services::profile as profile_service;

const LOGIN_WINDOW_LABEL: &str = "yt-login";
const YOUTUBE_LOGIN_URL: &str = "https://accounts.google.com/ServiceLogin?service=youtube&hl=en&continue=https://www.youtube.com";

pub fn open_login_window(app: &AppHandle) -> Result<WebviewWindow, AppError> {
    if let Some(window) = app.get_webview_window(LOGIN_WINDOW_LABEL) {
        window
            .show()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        window
            .set_focus()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        return Ok(window);
    }

    let url: Url = YOUTUBE_LOGIN_URL
        .parse()
        .map_err(|e| AppError::Internal(format!("Invalid login URL: {e}")))?;

    let webview = WebviewWindowBuilder::new(app, LOGIN_WINDOW_LABEL, WebviewUrl::External(url))
        .title("Sign in to YouTube")
        .inner_size(500.0, 720.0)
        .center()
        .resizable(true)
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to create login window: {e}")))?;

    Ok(webview)
}

pub async fn login_yt(app: &AppHandle) -> Result<profile::Model, AppError> {
    let _window = open_login_window(app)?;
    let state = app.state::<AppState>();

    loop {
        sleep(Duration::from_secs(1)).await;

        if app.get_webview_window(LOGIN_WINDOW_LABEL).is_none() {
            return Err(AppError::not_found_item("window"));
        }

        if let Some(window) = app.get_webview_window(LOGIN_WINDOW_LABEL) {
            if let Ok(url) = window.url() {
                let host = url.host_str().unwrap_or("");

                if host == "www.youtube.com" || host == "youtube.com" {
                    let cookies = window
                        .cookies()
                        .map_err(|e| AppError::Internal(e.to_string()))?;

                    let has_auth = cookies.iter().any(|c| {
                        c.name() == "SAPISID"
                            || c.name() == "__Secure-3PAPISID"
                            || c.name() == "LOGIN_INFO"
                    });

                    if has_auth {
                        let cookie_header = cookies
                            .iter()
                            .map(|c| format!("{}={}", c.name(), c.value()))
                            .collect::<Vec<String>>()
                            .join("; ");

                        let profile = save_profile(&state, cookie_header).await?;

                        let _ = window.clear_all_browsing_data();
                        let _ = window.destroy();
                        return Ok(profile);
                    }
                }
            }
        }
    }
}

async fn save_profile(state: &AppState, cookie_header: String) -> Result<profile::Model, AppError> {
    let yt = state.yt.clone();

    yt.load_session(None, &cookie_header).await?;

    let user = yt.get_current_user_profile().await?;

    let google_id = if !user.email.is_empty() {
        user.email.clone()
    } else if let Some(ref handle) = user.handle {
        handle.clone()
    } else {
        user.name.clone()
    };

    let saved_profile = profile_service::upsert_profile(
        &state.db,
        google_id,
        user.name,
        user.email,
        user.avatar_url,
        cookie_header.clone(),
    )
    .await?;

    let active_profile = profile_service::set_active_profile(&state.db, saved_profile.id).await?;

    yt.load_session(Some(active_profile.id), &cookie_header)
        .await?;

    Ok(active_profile)
}
