mod account;
mod audio;
mod channel;
mod feeds;
mod video;

use innertube_rs::{Innertube, SessionOptions, VideoInfo};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::db::entities::profile;
use crate::error::AppError;
use crate::models::youtube::AudioStreamInfo;
use crate::services::youtube::cookie::CookieManager;

/// How long a resolved audio URL is reused before re-minting it.
const AUDIO_CACHE_TTL: Duration = Duration::from_secs(20 * 60);

/// Options for a single InnerTube client override.
fn options_for_client(client: &str) -> innertube_rs::GetVideoInfoOptions {
    innertube_rs::GetVideoInfoOptions {
        client: Some(client.to_string()),
        ..Default::default()
    }
}

#[derive(Clone)]
struct CachedAudio {
    at: Instant,
    info: AudioStreamInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CurrentUserProfile {
    pub name: String,
    pub email: String,
    pub avatar_url: String,
    pub handle: Option<String>,
}

#[derive(Clone)]
pub struct YoutubeClient {
    inner: Arc<ClientInner>,
}

struct ClientInner {
    /// Options the signed-in client is built from. Kept so building it can be
    /// deferred until the first request instead of blocking app startup.
    options: RwLock<SessionOptions>,
    signed: RwLock<Option<Arc<Innertube>>>,
    anon: RwLock<Option<Arc<Innertube>>>,
    cookie_manager: CookieManager,
    active_profile_id: RwLock<Option<i32>>,
    last_video_info: RwLock<Option<VideoInfo>>,
    audio_cache: RwLock<HashMap<String, CachedAudio>>,
}

impl YoutubeClient {
    /// Cheap construction: the InnerTube clients are built on first use, so the
    /// window is never blocked on YouTube's bootstrap requests during startup.
    pub async fn new() -> Result<Self, AppError> {
        let _ = crate::utils::proxy::ensure_proxy().await;

        Ok(Self {
            inner: Arc::new(ClientInner {
                options: RwLock::new(SessionOptions::default()),
                signed: RwLock::new(None),
                anon: RwLock::new(None),
                cookie_manager: CookieManager::new(None),
                active_profile_id: RwLock::new(None),
                last_video_info: RwLock::new(None),
                audio_cache: RwLock::new(HashMap::new()),
            }),
        })
    }

    /// Build the signed-in client in the background so the first page is not
    /// the one paying for the bootstrap. The anonymous client is only needed
    /// for audio fallbacks and is built on first use.
    pub async fn warm_up(&self) {
        let _ = self.signed().await;
    }

    /// Signed-in client, created on first use and cached afterwards.
    async fn signed(&self) -> Result<Arc<Innertube>, AppError> {
        if let Some(client) = self.inner.signed.read().await.clone() {
            return Ok(client);
        }

        let mut slot = self.inner.signed.write().await;
        if let Some(client) = slot.clone() {
            return Ok(client);
        }

        let options = self.inner.options.read().await.clone();
        let client = Arc::new(
            Innertube::with_options(options)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to initialize InnerTube: {e}")))?,
        );
        *slot = Some(client.clone());
        Ok(client)
    }

    /// Anonymous client used for playback metadata and audio streams.
    async fn anon(&self) -> Result<Arc<Innertube>, AppError> {
        if let Some(client) = self.inner.anon.read().await.clone() {
            return Ok(client);
        }

        let mut slot = self.inner.anon.write().await;
        if let Some(client) = slot.clone() {
            return Ok(client);
        }

        let client = Arc::new(Innertube::new().await.map_err(|e| {
            AppError::Internal(format!("Failed to initialize Anonymous InnerTube: {e}"))
        })?);
        *slot = Some(client.clone());
        Ok(client)
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
                *self.inner.active_profile_id.write().await = Some(prof.id);
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

        // Store the session and drop the built client; the next request rebuilds
        // it with these cookies. No network work happens on a profile switch.
        *self.inner.options.write().await = options;
        *self.inner.signed.write().await = None;

        self.inner.cookie_manager.set_cookies(cookie_str).await;
        *self.inner.active_profile_id.write().await = profile_id;

        Ok(())
    }

    /// Post request to InnerTube endpoint and automatically sync any Set-Cookie headers to SQLite
    pub async fn post_with_cookie_sync(
        &self,
        endpoint: &str,
        payload: Value,
        db: Option<&DatabaseConnection>,
    ) -> Result<Value, AppError> {
        let yt = self.signed().await?;
        let response = yt
            .session
            .post_innertube(endpoint, payload)
            .await
            .map_err(|e| AppError::Internal(format!("InnerTube request failed: {e}")))?;

        if let Some(db) = db {
            let active_id = *self.inner.active_profile_id.read().await;
            let _ = self
                .inner
                .cookie_manager
                .sync_from_headers(response.headers(), active_id, db)
                .await;
        }

        let json: Value = response.json().await.map_err(|e| {
            AppError::Internal(format!("Failed to parse InnerTube response JSON: {e}"))
        })?;

        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Startup must not wait on YouTube: constructing the client is pure setup,
    /// the InnerTube handshake happens later (warm-up or the first request).
    #[tokio::test]
    async fn client_construction_does_not_hit_the_network() {
        let started = std::time::Instant::now();
        let _client = YoutubeClient::new().await.expect("Failed to create client");
        let elapsed = started.elapsed();
        assert!(
            elapsed.as_millis() < 500,
            "YoutubeClient::new blocked startup for {elapsed:?}"
        );
    }
}
