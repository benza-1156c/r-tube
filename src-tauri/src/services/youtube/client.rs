use innertube_rs::{
    BrowseFeed, HomeFeed, Innertube, PlayerResponse, SearchResults, SessionOptions, TrendingFeed,
    WatchNextResults,
};
use reqwest::header::HeaderMap;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::entities::profile;
use crate::error::AppError;
use crate::services::youtube::cookie::CookieManager;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CurrentUserProfile {
    pub name: String,
    pub email: String,
    pub avatar_url: String,
    pub handle: Option<String>,
}

#[derive(Clone)]
pub struct YoutubeClient {
    innertube: Arc<RwLock<Innertube>>,
    pub cookie_manager: CookieManager,
    active_profile_id: Arc<RwLock<Option<i32>>>,
}

impl YoutubeClient {
    /// Create a new YoutubeClient instance with default anonymous session
    pub async fn new() -> Result<Self, AppError> {
        let innertube = Innertube::new()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to initialize InnerTube: {e}")))?;

        Ok(Self {
            innertube: Arc::new(RwLock::new(innertube)),
            cookie_manager: CookieManager::new(None),
            active_profile_id: Arc::new(RwLock::new(None)),
        })
    }

    /// Initialize or reload the client from the active profile stored in SQLite
    pub async fn init_from_active_profile(&self, db: &DatabaseConnection) -> Result<(), AppError> {
        let active = profile::Entity::find()
            .filter(profile::Column::IsActive.eq(true))
            .one(db)
            .await?;

        if let Some(prof) = active {
            let cookie_str = prof.cookies.trim();
            if !cookie_str.is_empty() {
                self.load_session(Some(prof.id), cookie_str).await?;
            } else {
                *self.active_profile_id.write().await = Some(prof.id);
            }
        }

        Ok(())
    }

    /// Switch or update the active profile session with fresh cookies
    pub async fn load_session(
        &self,
        profile_id: Option<i32>,
        cookie_str: &str,
    ) -> Result<(), AppError> {
        let options = SessionOptions {
            cookie: Some(cookie_str.to_string()),
            lang: Some("en".to_string()),
            location: Some("US".to_string()),
            ..Default::default()
        };

        let new_client = Innertube::with_options(options).await.map_err(|e| {
            AppError::Internal(format!("Failed to configure InnerTube session: {e}"))
        })?;

        self.cookie_manager.set_cookies(cookie_str).await;

        *self.innertube.write().await = new_client;
        *self.active_profile_id.write().await = profile_id;

        Ok(())
    }

    /// Post request to InnerTube endpoint and automatically sync any Set-Cookie headers to SQLite
    pub async fn post_with_cookie_sync(
        &self,
        endpoint: &str,
        payload: Value,
        db: Option<&DatabaseConnection>,
    ) -> Result<Value, AppError> {
        let yt = self.innertube.read().await;
        let response = yt
            .session
            .post_innertube(endpoint, payload)
            .await
            .map_err(|e| AppError::Internal(format!("InnerTube request failed: {e}")))?;

        if let Some(db) = db {
            let active_id = *self.active_profile_id.read().await;
            let _ = self
                .cookie_manager
                .sync_from_headers(response.headers(), active_id, db)
                .await;
        }

        let json: Value = response.json().await.map_err(|e| {
            AppError::Internal(format!("Failed to parse InnerTube response JSON: {e}"))
        })?;

        Ok(json)
    }

    /// Fetch YouTube Home Feed
    pub async fn get_home_feed(
        &self,
        continuation_token: Option<&str>,
    ) -> Result<HomeFeed, AppError> {
        let yt = self.innertube.read().await;
        if let Some(token) = continuation_token {
            yt.get_home_feed_continuation(token).await.map_err(|e| {
                AppError::Internal(format!("Failed to fetch home feed continuation: {e}"))
            })
        } else {
            yt.get_home_feed(None)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to fetch home feed: {e}")))
        }
    }

    /// Fetch YouTube Subscriptions Feed
    pub async fn get_subscriptions_feed(&self) -> Result<BrowseFeed, AppError> {
        let yt = self.innertube.read().await;
        yt.get_subscriptions_feed()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch subscriptions feed: {e}")))
    }

    /// Fetch YouTube Trending Feed
    pub async fn get_trending(&self) -> Result<TrendingFeed, AppError> {
        let yt = self.innertube.read().await;
        yt.get_trending(None)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch trending feed: {e}")))
    }

    /// Search YouTube
    pub async fn search(
        &self,
        query: &str,
        continuation_token: Option<&str>,
    ) -> Result<SearchResults, AppError> {
        let yt = self.innertube.read().await;
        yt.search(query, continuation_token)
            .await
            .map_err(|e| AppError::Internal(format!("YouTube search failed: {e}")))
    }

    /// Get video information and player response
    pub async fn get_video_info(&self, video_id: &str) -> Result<PlayerResponse, AppError> {
        let yt = self.innertube.read().await;
        yt.get_video_info(video_id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get video info: {e}")))
    }

    /// Get watch next details (related videos and playlist context)
    pub async fn get_watch_next(&self, video_id: &str) -> Result<WatchNextResults, AppError> {
        let yt = self.innertube.read().await;
        yt.get_watch_next(video_id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get watch next details: {e}")))
    }

    /// Sync Set-Cookie headers manually (e.g. from Webview or external response)
    pub async fn sync_headers(
        &self,
        headers: &HeaderMap,
        db: &DatabaseConnection,
    ) -> Result<Option<String>, AppError> {
        let active_id = *self.active_profile_id.read().await;
        self.cookie_manager
            .sync_from_headers(headers, active_id, db)
            .await
    }

    pub async fn get_account_info(&self) -> Result<CurrentUserProfile, AppError> {
        fn extract_text(v: &Value, ptr1: &str, ptr2: &str) -> Option<String> {
            v.pointer(ptr1)
                .or_else(|| v.pointer(ptr2))
                .and_then(Value::as_str)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        }

        fn scan_profile(
            v: &Value,
            name: &mut Option<String>,
            email: &mut Option<String>,
            photo: &mut Option<String>,
            handle: &mut Option<String>,
        ) {
            if let Some(header) = v
                .get("activeAccountHeaderRenderer")
                .or_else(|| v.get("accountHeaderRenderer"))
                .or_else(|| v.get("googleAccountHeaderRenderer"))
                .or_else(|| v.get("accountItemSectionHeaderRenderer"))
            {
                if name.is_none() {
                    *name = extract_text(
                        header,
                        "/accountName/simpleText",
                        "/accountName/runs/0/text",
                    )
                    .or_else(|| extract_text(header, "/title/simpleText", "/title/runs/0/text"))
                    .or_else(|| extract_text(header, "/name/simpleText", "/name/runs/0/text"));
                }
                if email.is_none() {
                    *email = extract_text(header, "/email/simpleText", "/email/runs/0/text");
                }
                if handle.is_none() {
                    *handle = extract_text(
                        header,
                        "/channelHandle/simpleText",
                        "/channelHandle/runs/0/text",
                    );
                }
                if photo.is_none() {
                    *photo = header
                        .pointer("/accountPhoto/thumbnails")
                        .or_else(|| header.pointer("/thumbnail/thumbnails"))
                        .and_then(Value::as_array)
                        .and_then(|arr| arr.last())
                        .and_then(|t| t.get("url"))
                        .and_then(Value::as_str)
                        .map(ToString::to_string);
                }
            }

            if let Some(item) = v.get("accountItemRenderer") {
                let is_selected = item
                    .get("isSelected")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if is_selected || name.is_none() {
                    if let Some(n) =
                        extract_text(item, "/accountName/simpleText", "/accountName/runs/0/text")
                    {
                        *name = Some(n);
                    }
                    if let Some(h) = extract_text(
                        item,
                        "/channelHandle/simpleText",
                        "/channelHandle/runs/0/text",
                    ) {
                        *handle = Some(h);
                    }
                    if let Some(p) = item
                        .pointer("/accountPhoto/thumbnails")
                        .and_then(Value::as_array)
                        .and_then(|arr| arr.last())
                        .and_then(|t| t.get("url"))
                        .and_then(Value::as_str)
                    {
                        *photo = Some(p.to_string());
                    }
                }
            }

            if let Some(s) = v.as_str() {
                let trimmed = s.trim();
                if email.is_none()
                    && trimmed.contains('@')
                    && !trimmed.starts_with('@')
                    && trimmed.contains('.')
                    && !trimmed.contains('/')
                    && !trimmed.contains(' ')
                {
                    *email = Some(trimmed.to_string());
                } else if handle.is_none()
                    && trimmed.starts_with('@')
                    && !trimmed.contains(' ')
                    && !trimmed.contains('/')
                    && trimmed.len() > 2
                {
                    *handle = Some(trimmed.to_string());
                }
            }

            match v {
                Value::Object(map) => {
                    for val in map.values() {
                        scan_profile(val, name, email, photo, handle);
                    }
                }
                Value::Array(arr) => {
                    for item in arr {
                        scan_profile(item, name, email, photo, handle);
                    }
                }
                _ => {}
            }
        }

        let mut name = None;
        let mut email = None;
        let mut photo = None;
        let mut handle = None;

        let menu_payload = serde_json::json!({
            "deviceTheme": "DEVICE_THEME_SUPPORTED",
            "userInterfaceTheme": "USER_INTERFACE_THEME_DARK"
        });
        if let Ok(menu_raw) = self
            .post_with_cookie_sync("/account/account_menu", menu_payload, None)
            .await
        {
            scan_profile(&menu_raw, &mut name, &mut email, &mut photo, &mut handle);
        }

        if email.is_none() || name.is_none() {
            let list_payload = serde_json::json!({
                "callCircumstance": "SWITCHING_USERS_FULL"
            });
            if let Ok(list_raw) = self
                .post_with_cookie_sync("/account/accounts_list", list_payload, None)
                .await
            {
                scan_profile(&list_raw, &mut name, &mut email, &mut photo, &mut handle);
                if email.is_none() {
                    println!(
                        "[Debug] /account/accounts_list JSON: {}",
                        serde_json::to_string(&list_raw).unwrap_or_default()
                    );
                }
            }
        }

        if name.is_none() {
            let yt = self.innertube.read().await;
            if let Ok(accounts) = yt.account().get_accounts().await {
                if let Some(acc) = accounts
                    .iter()
                    .find(|a| a.is_selected)
                    .or_else(|| accounts.first())
                {
                    name = Some(acc.account_name.clone());
                    if photo.is_none() {
                        photo = acc.account_photo.clone();
                    }
                }
            }
        }

        let final_name = name
            .or_else(|| email.clone())
            .or_else(|| handle.clone())
            .unwrap_or_else(|| "YouTube User".to_string());

        Ok(CurrentUserProfile {
            name: final_name,
            email: email.or_else(|| handle.clone()).unwrap_or_default(),
            avatar_url: photo.unwrap_or_default(),
            handle,
        })
    }
}
