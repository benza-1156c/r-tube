use std::collections::HashMap;
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

    #[cfg(desktop)]
    let builder = builder.center();

    builder
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to create login window: {e}")))
}

pub fn cancel_login(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(LOGIN_WINDOW_LABEL) {
        let _ = window.destroy();
    }
}

pub async fn login_yt(app: &AppHandle) -> Result<profile::Model, AppError> {
    let _window = open_login_window(app)?;
    let state = app.state::<AppState>();
    let youtube_url = Url::parse("https://www.youtube.com/")
        .map_err(|e| AppError::Internal(format!("Invalid YouTube URL: {e}")))?;
    let started = std::time::Instant::now();

    loop {
        if started.elapsed() > Duration::from_secs(300) {
            cancel_login(app);
            return Err(AppError::Validation(
                "Sign in timed out. Please try again.".to_string(),
            ));
        }

        sleep(Duration::from_millis(500)).await;

        let Some(window) = app.get_webview_window(LOGIN_WINDOW_LABEL) else {
            return Err(AppError::Validation("Sign in cancelled".to_string()));
        };

        if let Ok(url) = window.url() {
            if !is_youtube_home(&url) {
                continue;
            }

            // Only cookies that are actually sent to youtube.com. Taking every
            // cookie in the webview mixes in google.com/accounts.google.com
            // copies of SID/SAPISID, and the last one wins when the header is
            // rebuilt, so the API session ends up unauthenticated.
            let cookies = window
                .cookies_for_url(youtube_url.clone())
                .or_else(|_| window.cookies());
            let Ok(cookies) = cookies else {
                continue;
            };
            let Some(cookie_header) = youtube_cookie_header(&cookies) else {
                // Still signed out (or the auth cookies have not landed yet).
                // Keep the window visible so the user can finish signing in.
                continue;
            };

            let _ = window.hide();
            match save_profile(&state, cookie_header).await {
                Ok(profile) => {
                    let _ = window.destroy();
                    return Ok(profile);
                }
                Err(err) => {
                    // Leave no half-signed-in window behind; the caller shows
                    // the error and the user can start a fresh attempt.
                    cancel_login(app);
                    return Err(err);
                }
            }
        }
    }
}

/// Build the cookie header for YouTube from the webview's cookie store.
///
/// The login webview also holds google.com cookies with the same names
/// (`SID`, `SAPISID`, …). Sending those to youtube.com authenticates nothing,
/// so keep only YouTube domains, deduplicate by name (preferring a `/` path
/// on `.youtube.com`), and require both halves of the session credentials.
fn youtube_cookie_header(cookies: &[tauri::webview::Cookie<'static>]) -> Option<String> {
    let mut best: HashMap<String, (u8, String)> = HashMap::new();

    for cookie in cookies {
        let domain = cookie.domain().unwrap_or("");
        if domain != "youtube.com" && !domain.ends_with(".youtube.com") {
            continue;
        }

        let path = cookie.path().unwrap_or("/");
        let score = u8::from(domain == ".youtube.com") + 2 * u8::from(path == "/");

        let entry = best
            .entry(cookie.name().to_string())
            .or_insert((0, String::new()));
        if score >= entry.0 {
            *entry = (score, cookie.value().to_string());
        }
    }

    let has_sapisid = best.contains_key("SAPISID") || best.contains_key("__Secure-3PAPISID");
    let has_sid = best.contains_key("SID")
        || best.contains_key("__Secure-1PSID")
        || best.contains_key("__Secure-3PSID");
    if !has_sapisid || !has_sid {
        return None;
    }

    Some(
        best.into_iter()
            .map(|(name, (_, value))| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; "),
    )
}

async fn save_profile(state: &AppState, cookie_header: String) -> Result<profile::Model, AppError> {
    let yt = state.yt.clone();

    yt.load_session(None, &cookie_header).await?;

    let account = yt.get_account_info().await?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::webview::Cookie;

    fn cookie(name: &'static str, value: &'static str, domain: &'static str, path: &'static str) -> Cookie<'static> {
        Cookie::build((name, value))
            .domain(domain)
            .path(path)
            .build()
    }

    #[test]
    fn keeps_only_youtube_auth_cookies() {
        let header = youtube_cookie_header(&[
            cookie("SID", "google-sid", ".google.com", "/"),
            cookie("SAPISID", "google-sapisid", ".google.com", "/"),
            cookie("SID", "youtube-sid", ".youtube.com", "/"),
            cookie("SAPISID", "youtube-sapisid", ".youtube.com", "/"),
            cookie("SID", "watch-sid", "www.youtube.com", "/watch"),
            cookie("VISITOR_INFO1_LIVE", "visitor", ".youtube.com", "/"),
        ])
        .expect("a header");

        assert!(header.contains("SID=youtube-sid"));
        assert!(header.contains("SAPISID=youtube-sapisid"));
        assert!(header.contains("VISITOR_INFO1_LIVE=visitor"));
        assert!(!header.contains("google-sid"));
        assert!(!header.contains("watch-sid"));
    }

    #[test]
    fn refuses_a_session_without_both_credential_halves() {
        let header = youtube_cookie_header(&[
            cookie("SID", "sid", ".youtube.com", "/"),
            cookie("SAPISID", "sapisid", ".google.com", "/"),
        ]);
        assert!(header.is_none());
    }
}
