use std::time::Duration;

use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tokio::time::sleep;

use crate::error::AppError;

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

pub async fn login_yt(app: &AppHandle) -> Result<String, AppError> {
    let _window = open_login_window(app)?;

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

                    let cookie_header = cookies
                        .iter()
                        .map(|c| format!("{}={}", c.name(), c.value()))
                        .collect::<Vec<String>>()
                        .join("; ");

                    return Ok(cookie_header);
                }
            }
        }
    }
}
