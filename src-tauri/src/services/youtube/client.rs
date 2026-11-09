use innertube_rs::models::account::AccountChannelItem;
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

    pub async fn get_account_info(&self) -> Result<Vec<AccountChannelItem>, AppError> {
        let yt = self.innertube.read().await;
        let accounts = yt
            .account()
            .get_accounts()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch account info: {e}")))?;

        Ok(accounts)
    }
}
