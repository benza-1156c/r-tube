use innertube_rs::{
    BrowseFeed, HomeFeed, Innertube, SearchResults, SessionOptions, TrendingFeed, VideoInfo,
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
use crate::models::youtube::{CaptionTrack, PlayerResponse, VideoItem};
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
    anon_innertube: Arc<Innertube>,
    pub cookie_manager: CookieManager,
    active_profile_id: Arc<RwLock<Option<i32>>>,
    last_video_info: Arc<RwLock<Option<VideoInfo>>>,
}

impl YoutubeClient {
    pub async fn new() -> Result<Self, AppError> {
        let _ = crate::utils::proxy::ensure_proxy().await;

        let innertube = Innertube::new()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to initialize InnerTube: {e}")))?;
        let anon_innertube = Innertube::new()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to initialize Anon InnerTube: {e}")))?;

        Ok(Self {
            innertube: Arc::new(RwLock::new(innertube)),
            anon_innertube: Arc::new(anon_innertube),
            cookie_manager: CookieManager::new(None),
            active_profile_id: Arc::new(RwLock::new(None)),
            last_video_info: Arc::new(RwLock::new(None)),
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
        params: Option<&str>,
    ) -> Result<HomeFeed, AppError> {
        let yt = self.innertube.read().await;
        let token = continuation_token.or(params);
        if let Some(t) = token {
            yt.get_home_feed_continuation(t).await.map_err(|e| {
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

    /// HLS is the only playback path: every segment arrives as its own URL,
    pub async fn get_video_info(&self, video_id: &str) -> Result<PlayerResponse, AppError> {
        crate::utils::proxy::ensure_proxy()
            .await
            .map_err(|e| AppError::Internal(format!("Local stream proxy unavailable: {e}")))?;

        let raw = {
            let yt = self.innertube.read().await;
            yt.get_basic_info(video_id, None)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to get video info: {e}")))?
        };

        // Keep the playback session data (cpn + tracking URLs) for the
        // history and watch-time events this video will report.
        {
            let mut cached = self.last_video_info.write().await;
            *cached = Some(raw.clone());
        }

        let mut player_res = PlayerResponse::from(raw.player_response);
        player_res.hls_manifest_url = self
            .hls_manifest_url(video_id)
            .await
            .map(|raw| crate::utils::proxy::wrap_hls_url(&raw, video_id));

        if let Ok(tracks) = self.anon_innertube.get_transcript_tracks(video_id).await {
            player_res.captions = tracks
                .into_iter()
                .map(|t| CaptionTrack {
                    language_code: t.language_code,
                    name: t.name,
                    vtt_url: format!("{}&fmt=vtt", t.base_url),
                })
                .collect();
        }

        Ok(player_res)
    }

    pub async fn hls_manifest_url(&self, video_id: &str) -> Option<String> {
        for client in ["VISIONOS", "IOS", "ANDROID"] {
            let options = innertube_rs::GetVideoInfoOptions {
                client: Some(client.to_string()),
                po_token: None,
                playback_context: None,
            };
            match self
                .anon_innertube
                .get_basic_info(video_id, Some(&options))
                .await
            {
                Ok(info) => {
                    if let Some(url) = info
                        .player_response
                        .streaming_data
                        .and_then(|sd| sd.hls_manifest_url)
                    {
                        return Some(url);
                    }
                }
                Err(e) => eprintln!("[YouTube] HLS via {client} failed: {e}"),
            }
        }
        None
    }
    /// Get watch next details (related videos and playlist context)
    pub async fn get_watch_next(&self, video_id: &str) -> Result<WatchNextResults, AppError> {
        let yt = self.innertube.read().await;
        yt.get_watch_next(video_id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get watch next details: {e}")))
    }

    /// Player session data for a video: the cached one when it matches, a
    /// fresh one otherwise (history can be reported without a prior load).
    async fn playback_info(&self, video_id: &str) -> Result<VideoInfo, AppError> {
        if let Some(info) = self.last_video_info.read().await.as_ref() {
            if info.id() == Some(video_id) {
                return Ok(info.clone());
            }
        }

        let yt = self.innertube.read().await;
        let info = yt
            .get_basic_info(video_id, None)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get video info: {e}")))?;
        *self.last_video_info.write().await = Some(info.clone());
        Ok(info)
    }

    /// Publish the initial playback event, which is what adds the video to the
    /// signed-in account's watch history.
    pub async fn record_watch_start(&self, video_id: &str) -> Result<(), AppError> {
        let info = self.playback_info(video_id).await?;
        let yt = self.innertube.read().await;
        info.add_to_watch_history(yt.session.as_ref())
            .await
            .map_err(|e| AppError::Internal(format!("Failed to record watch history: {e}")))?;
        Ok(())
    }

    /// Report playback progress so the resume position is remembered.
    pub async fn record_watch_time(&self, video_id: &str, seconds: f64) -> Result<(), AppError> {
        let info = self.playback_info(video_id).await?;
        let yt = self.innertube.read().await;
        info.update_watch_time(yt.session.as_ref(), seconds)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to update watch time: {e}")))?;
        Ok(())
    }

    /// Watch history of the signed-in account, newest first.
    pub async fn get_history(&self) -> Result<Vec<VideoItem>, AppError> {
        let yt = self.innertube.read().await;
        let feed = yt
            .get_history(None)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get watch history: {e}")))?;
        Ok(feed.videos.iter().map(VideoItem::from).collect())
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The playback contract: a proxied HLS master playlist, and its segments
    /// reachable through the proxy.
    #[tokio::test]
    async fn video_info_has_proxied_hls_manifest() {
        let client = YoutubeClient::new().await.expect("Failed to create client");
        let video_id =
            std::env::var("PROBE_VIDEO_ID").unwrap_or_else(|_| "8XXPxLLgwt8".to_string());
        let info = client
            .get_video_info(&video_id)
            .await
            .expect("Failed to get video info");

        assert!(!info.title.is_empty(), "expected a title");
        let hls = info
            .hls_manifest_url
            .as_deref()
            .expect("expected an HLS manifest");
        assert!(
            hls.starts_with("http://127.0.0.1:"),
            "HLS manifest must be proxied, got: {hls}"
        );

        let http = reqwest::Client::new();
        let master = http
            .get(hls)
            .send()
            .await
            .expect("proxy request failed")
            .text()
            .await
            .expect("body");
        assert!(master.starts_with("#EXTM3U"), "expected a playlist");
        assert!(
            master.contains("http://127.0.0.1:"),
            "every playlist URI should be rewritten through the proxy"
        );

        // Fetch one rewritten variant playlist: its segments must resolve too.
        let variant = master
            .lines()
            .find(|line| line.starts_with("http://127.0.0.1:"))
            .expect("a rewritten variant URI");
        let media = http
            .get(variant)
            .send()
            .await
            .expect("variant request failed")
            .text()
            .await
            .expect("variant body");
        assert!(media.starts_with("#EXTM3U"), "expected a media playlist");

        let segment = media
            .lines()
            .find(|line| line.starts_with("http://127.0.0.1:"))
            .expect("a rewritten segment URI");
        let resp = http.get(segment).send().await.expect("segment request");
        assert!(
            resp.status().is_success(),
            "segment request failed: {}",
            resp.status()
        );
        let bytes = resp.bytes().await.expect("segment bytes");
        assert!(bytes.len() > 1000, "segment too small: {}", bytes.len());
    }
}
