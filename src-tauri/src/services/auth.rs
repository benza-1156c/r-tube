use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tokio::time::sleep;

use crate::AppState;
use crate::db::entities::profile;
use crate::error::AppError;
use crate::services::profile as profile_service;

const LOGIN_WINDOW_LABEL: &str = "yt-login";
const YOUTUBE_LOGIN_URL: &str = "https://accounts.google.com/AddSession?service=youtube&hl=en&continue=https://www.youtube.com";

static LOGIN_CANCELLED: AtomicBool = AtomicBool::new(false);
static LOGIN_RETURN_URL: Mutex<Option<Url>> = Mutex::new(None);

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
    LOGIN_CANCELLED.store(true, Ordering::SeqCst);

    // On mobile the login page replaces the app's only webview, so it must be
    // navigated back. Destroying it would close the whole activity/app.
    #[cfg(target_os = "android")]
    let _ = restore_app_window(app);

    #[cfg(not(target_os = "android"))]
    if let Some(window) = app.get_webview_window(LOGIN_WINDOW_LABEL) {
        let _ = window.destroy();
    }
}

#[cfg(target_os = "android")]
fn restore_app_window(app: &AppHandle) -> Result<(), AppError> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };
    let target = LOGIN_RETURN_URL
        .lock()
        .ok()
        .and_then(|guard| app_root_url(guard.as_ref()))
        .unwrap_or_else(|| Url::parse("http://tauri.localhost/").expect("valid app URL"));
    window
        .navigate(target)
        .map_err(|e| AppError::Internal(format!("Failed to return to the app: {e}")))
}

#[cfg(target_os = "android")]
fn app_root_url(url: Option<&Url>) -> Option<Url> {
    let mut url = url?.clone();
    url.set_path("/");
    url.set_query(None);
    url.set_fragment(None);
    Some(url)
}

pub async fn login_yt(app: &AppHandle) -> Result<profile::Model, AppError> {
    LOGIN_CANCELLED.store(false, Ordering::SeqCst);
    *LOGIN_RETURN_URL.lock().unwrap_or_else(|e| e.into_inner()) = None;

    let state = app.state::<AppState>();
    let youtube_url = Url::parse("https://www.youtube.com/")
        .map_err(|e| AppError::Internal(format!("Invalid YouTube URL: {e}")))?;

    // Android has a single webview: navigate the app itself to the login page
    // and return once the cookies are readable. Desktop keeps the separate
    // login window.
    #[cfg(target_os = "android")]
    {
        let window = app
            .get_webview_window("main")
            .ok_or_else(|| AppError::not_found_item("main window"))?;
        let return_url = window.url().ok();
        *LOGIN_RETURN_URL.lock().unwrap_or_else(|e| e.into_inner()) = return_url;
        let login_url = Url::parse(YOUTUBE_LOGIN_URL)
            .map_err(|e| AppError::Internal(format!("Invalid login URL: {e}")))?;
        window
            .navigate(login_url)
            .map_err(|e| AppError::Internal(format!("Failed to open the login page: {e}")))?;
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = open_login_window(app)?;
    }

    #[cfg(target_os = "android")]
    let window_label = "main";
    #[cfg(not(target_os = "android"))]
    let window_label = LOGIN_WINDOW_LABEL;

    let started = std::time::Instant::now();
    #[cfg(target_os = "android")]
    let mut left_app = false;

    loop {
        if LOGIN_CANCELLED.load(Ordering::SeqCst) {
            return Err(AppError::Validation("Sign in cancelled".to_string()));
        }

        if started.elapsed() > Duration::from_secs(300) {
            cancel_login(app);
            return Err(AppError::Validation(
                "Sign in timed out. Please try again.".to_string(),
            ));
        }

        // Poll faster while the user is in the sign-in flow so the YouTube
        // landing page is replaced by the app before it becomes noticeable.
        #[cfg(target_os = "android")]
        sleep(Duration::from_millis(150)).await;
        #[cfg(not(target_os = "android"))]
        sleep(Duration::from_millis(500)).await;

        let Some(window) = app.get_webview_window(window_label) else {
            return Err(AppError::Validation("Sign in cancelled".to_string()));
        };

        let mut on_youtube = false;
        if let Ok(current) = window.url() {
            let host = current.host_str().unwrap_or("");
            on_youtube = host == "youtube.com" || host.ends_with(".youtube.com");

            // The user must see Google's form, but the post-sign-in YouTube
            // page is only needed to set cookies. Hide it immediately; the
            // loop returns to the app as soon as those cookies are readable.
            if on_youtube {
                let _ = window.eval(
                    "document.documentElement.style.visibility='hidden'; document.documentElement.style.background='#000'; if (document.body) document.body.style.visibility='hidden';",
                );
            }

            // If the user backs out of the login page on Android, the app's
            // own webview returns to the app URL. Treat that as a cancellation
            // once we have actually left the app at least once.
            #[cfg(target_os = "android")]
            {
                let current = current.as_str();
                let app_url = current.starts_with("http://tauri.localhost")
                    || current.starts_with("https://tauri.localhost")
                    || current.starts_with("http://localhost:")
                    || current.starts_with("https://localhost:");
                if !app_url {
                    left_app = true;
                } else if left_app {
                    return Err(AppError::Validation("Sign in cancelled".to_string()));
                }
            }
        }

        // Only capture after Google hands the user back to YouTube. Otherwise an
        // existing signed-in webview would satisfy the cookie check while still
        // on accounts.google.com and "add another account" would never open.
        if !on_youtube {
            continue;
        }

        // Poll the cookie store instead of the window URL. Android's login
        // webview can hand off to another page, so `url()` is not a reliable
        // signal, but the app-wide CookieManager is.
        //
        // Only cookies that are actually sent to youtube.com. Taking every
        // cookie in the webview mixes in google.com/accounts.google.com copies
        // of SID/SAPISID, and the last one wins when the header is rebuilt.
        let cookies = window
            .cookies_for_url(youtube_url.clone())
            .or_else(|_| window.cookies());
        let Ok(cookies) = cookies else {
            continue;
        };
        let Some(cookie_header) = youtube_cookie_header(&cookies) else {
            // Still signed out (or the auth cookies have not landed yet).
            continue;
        };

        #[cfg(not(target_os = "android"))]
        let _ = window.hide();

        match save_profile(&state, cookie_header).await {
            Ok(profile) => {
                #[cfg(target_os = "android")]
                let _ = restore_app_window(app);
                #[cfg(not(target_os = "android"))]
                let _ = window.destroy();
                return Ok(profile);
            }
            Err(err) => {
                #[cfg(target_os = "android")]
                let _ = restore_app_window(app);
                #[cfg(not(target_os = "android"))]
                cancel_login(app);
                return Err(err);
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
        // Android returns one `Cookie` header parsed from CookieManager, so
        // its cookies have no domain attribute (they are already filtered to
        // the requested URL). Desktop returns real Cookie entries.
        let domain = cookie.domain().unwrap_or("");
        if !(domain.is_empty() || domain == "youtube.com" || domain.ends_with(".youtube.com")) {
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
    fn accepts_android_header_cookies_without_a_domain() {
        let cookies: Vec<Cookie<'static>> =
            "SID=sid; SAPISID=sapisid; VISITOR_INFO1_LIVE=visitor"
                .split("; ")
                .filter_map(|value| Cookie::parse(value.to_string()).ok())
                .collect();
        let header = youtube_cookie_header(&cookies).expect("a header");
        assert!(header.contains("SID=sid"));
        assert!(header.contains("SAPISID=sapisid"));
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
