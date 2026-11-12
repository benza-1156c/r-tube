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
        let _ = window.set_focus();
        return Ok(window);
    }

    let url: Url = YOUTUBE_LOGIN_URL
        .parse()
        .map_err(|e| AppError::Internal(format!("Invalid login URL: {e}")))?;

    let login_profile_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| AppError::Internal(e.to_string()))?
        .join("LoginProfile");

    if login_profile_dir.exists() {
        let _ = std::fs::remove_dir_all(&login_profile_dir);
    }

    let builder = WebviewWindowBuilder::new(app, LOGIN_WINDOW_LABEL, WebviewUrl::External(url))
        .title("Sign in to YouTube")
        .inner_size(500.0, 720.0)
        .data_directory(login_profile_dir);

    // `center` is part of Tauri's desktop-only builder API; mobile webviews
    // always fill the screen, so the call is skipped there.
    #[cfg(desktop)]
    let builder = builder.center();

    builder
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to create login window: {e}")))
}

pub async fn login_yt(app: &AppHandle) -> Result<profile::Model, AppError> {
    let _window = open_login_window(app)?;
    let state = app.state::<AppState>();

    loop {
        sleep(Duration::from_millis(500)).await;

        let Some(window) = app.get_webview_window(LOGIN_WINDOW_LABEL) else {
            return Err(AppError::not_found_item("window"));
        };

        if let Ok(url) = window.url() {
            if is_youtube_home(&url) {
                let _ = window.hide();
                let Ok(cookies) = window.cookies() else {
                    continue;
                };

                let has_auth = cookies.iter().any(|c| {
                    let n = c.name();
                    n == "LOGIN_INFO" || n == "SAPISID" || n == "SID" || n == "__Secure-3PAPISID"
                });

                if !has_auth {
                    continue;
                }

                sleep(Duration::from_millis(500)).await;
                let cookies = window.cookies().unwrap_or(cookies);

                let cookie_header = cookies
                    .iter()
                    .map(|c| format!("{}={}", c.name(), c.value()))
                    .collect::<Vec<String>>()
                    .join("; ");

                let _ = window.destroy();
                return save_profile(&state, cookie_header).await;
            }
        }
    }
}

async fn save_profile(state: &AppState, cookie_header: String) -> Result<profile::Model, AppError> {
    let yt = state.yt.clone();

    yt.load_session(None, &cookie_header).await?;

    let account = yt.get_account_info().await?;

    println!("{:?}", account);

    let saved_profile = profile_service::upsert_profile(
        &state.db,
        account.name,
        account.email,
        account.avatar_url,
        cookie_header.clone(),
    )
    .await?;

    let active_profile = profile_service::set_active_profile(&state.db, saved_profile.id).await?;

    yt.load_session(Some(active_profile.id), &cookie_header)
        .await?;

    Ok(active_profile)
}

pub fn is_youtube_home(url: &tauri::Url) -> bool {
    matches!(
        url.host_str(),
        Some("www.youtube.com" | "youtube.com" | "m.youtube.com")
    ) && !url.as_str().contains("accounts.google.com")
        && !url.path().starts_with("/signin")
        && !url.path().starts_with("/account")
}
