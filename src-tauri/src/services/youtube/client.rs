use innertube_rs::{
    BrowseFeed, HomeFeed, Innertube, SearchResults, SessionOptions, StreamingFormat, TrendingFeed,
    VideoInfo,
};
use reqwest::header::HeaderMap;
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
use crate::models::youtube::{
    AudioStreamInfo, CaptionTrack, ChannelInfo, ChannelPage, ChannelVideo, CommentsPage,
    PlayerResponse, PlaylistContinuationResult, PlaylistDetailView, SearchSuggestionItem,
    VideoComment, VideoItem, WatchNextResults,
};
use crate::services::youtube::cookie::CookieManager;

/// How long a resolved audio URL is reused before re-minting it.
const AUDIO_CACHE_TTL: Duration = Duration::from_secs(20 * 60);

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

    /// Build both clients in the background so the first page is not the one
    /// paying for them.
    pub async fn warm_up(&self) {
        let _ = futures_util::join!(self.signed(), self.anon());
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
        let client = Arc::new(Innertube::with_options(options).await.map_err(|e| {
            AppError::Internal(format!("Failed to initialize InnerTube: {e}"))
        })?);
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

    /// Fetch YouTube Home Feed
    pub async fn get_home_feed(
        &self,
        continuation_token: Option<&str>,
        params: Option<&str>,
    ) -> Result<HomeFeed, AppError> {
        let yt = self.signed().await?;
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
        let yt = self.signed().await?;
        yt.get_subscriptions_feed()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch subscriptions feed: {e}")))
    }

    /// Fetch YouTube Trending Feed
    pub async fn get_trending(&self) -> Result<TrendingFeed, AppError> {
        let yt = self.signed().await?;
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
        let yt = self.signed().await?;
        yt.search(query, continuation_token)
            .await
            .map_err(|e| AppError::Internal(format!("YouTube search failed: {e}")))
    }

    /// Fetch full YouTube playlist details
    pub async fn get_playlist(&self, playlist_id: &str) -> Result<PlaylistDetailView, AppError> {
        let yt = self.signed().await?;
        yt.get_playlist(playlist_id)
            .await
            .map(PlaylistDetailView::from)
            .map_err(|e| AppError::Internal(format!("Failed to fetch playlist: {e}")))
    }

    /// Fetch playlist continuation videos
    pub async fn get_playlist_continuation(
        &self,
        token: &str,
    ) -> Result<PlaylistContinuationResult, AppError> {
        let yt = self.signed().await?;
        yt.get_playlist_continuation(token)
            .await
            .map(PlaylistContinuationResult::from)
            .map_err(|e| AppError::Internal(format!("Failed to fetch playlist continuation: {e}")))
    }

    /// Get rich search autocomplete suggestions with subtitles and thumbnails from innertube-rs
    pub async fn get_search_suggestions(
        &self,
        query: &str,
    ) -> Result<Vec<SearchSuggestionItem>, AppError> {
        let yt = self.signed().await?;
        let res = yt
            .get_search_suggestions(query, false)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch search suggestions: {e}")))?;

        Ok(res
            .items
            .into_iter()
            .map(SearchSuggestionItem::from)
            .collect())
    }

    /// HLS is the only playback path: every segment arrives as its own URL,
    pub async fn get_video_info(&self, video_id: &str) -> Result<PlayerResponse, AppError> {
        crate::utils::proxy::ensure_proxy()
            .await
            .map_err(|e| AppError::Internal(format!("Local stream proxy unavailable: {e}")))?;

        let raw = {
            let yt = self.signed().await?;
            yt.get_basic_info(video_id, None)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to get video info: {e}")))?
        };

        // Keep the playback session data (cpn + tracking URLs) for the
        // history and watch-time events this video will report.
        {
            let mut cached = self.inner.last_video_info.write().await;
            *cached = Some(raw.clone());
        }

        let mut player_res = PlayerResponse::from(raw.player_response);
        player_res.hls_manifest_url = self
            .hls_manifest_url(video_id)
            .await
            .map(|raw| crate::utils::proxy::wrap_hls_url(&raw, video_id));

        if let Ok(anon) = self.anon().await {
            if let Ok(tracks) = anon.get_transcript_tracks(video_id).await {
                player_res.captions = tracks
                    .into_iter()
                    .map(|t| CaptionTrack {
                        language_code: t.language_code,
                        name: t.name,
                        vtt_url: format!("{}&fmt=vtt", t.base_url),
                    })
                    .collect();
            }
        }

        Ok(player_res)
    }

    pub async fn hls_manifest_url(&self, video_id: &str) -> Option<String> {
        let anon = self.anon().await.ok()?;
        for client in ["VISIONOS", "IOS", "ANDROID"] {
            let options = innertube_rs::GetVideoInfoOptions {
                client: Some(client.to_string()),
                po_token: None,
                playback_context: None,
            };
            match anon
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

    /// Best audio-only stream for a video, with metadata for the listen UI.
    ///
    /// The returned URL is the raw CDN URL; callers wrap it for the local proxy.
    pub async fn audio_stream(
        &self,
        video_id: &str,
        language: Option<&str>,
    ) -> Result<AudioStreamInfo, AppError> {
        let key = format!("{video_id}:{}", language.unwrap_or_default());

        if let Some(cached) = self.inner.audio_cache.read().await.get(&key) {
            if cached.at.elapsed() < AUDIO_CACHE_TTL {
                return Ok(cached.info.clone());
            }
        }

        let info = self.resolve_audio(video_id, language, None).await?;
        self.inner.audio_cache.write().await.insert(
            key,
            CachedAudio {
                at: Instant::now(),
                info: info.clone(),
            },
        );
        Ok(info)
    }

    /// Re-mint an expired audio URL for the same track, so a paused song can
    /// resume hours later without switching formats.
    ///
    /// The player response is fetched fresh: the cached one still holds the
    /// URL that just expired.
    pub async fn refresh_audio_url(&self, video_id: &str, itag: u32) -> Option<String> {
        self.resolve_audio(video_id, None, Some(itag))
            .await
            .ok()
            .map(|info| info.url)
    }

    async fn resolve_audio(
        &self,
        video_id: &str,
        language: Option<&str>,
        wanted_itag: Option<u32>,
    ) -> Result<AudioStreamInfo, AppError> {
        let player_response = self.audio_player_response(video_id).await?;

        let streaming = player_response.streaming_data.as_ref().ok_or_else(|| {
            AppError::NotFound("This video has no streamable formats".to_string())
        })?;

        let format = pick_audio_format(&streaming.adaptive_formats, language, wanted_itag)
            .ok_or_else(|| {
                AppError::NotFound("This video has no audio-only stream".to_string())
            })?;

        // Live/DVR audio arrives as an open-ended chunked stream: there is no
        // length to range over, so it cannot be the lightweight `<audio>` path.
        if format.is_type_otf.unwrap_or(false) {
            return Err(AppError::NotFound(
                "Live streams are not available in audio-only mode".to_string(),
            ));
        }

        // The response comes from the anonymous VISIONOS client, so the
        // decipherer and PO token must come from that session too.
        let anon = self.anon().await?;
        let url = innertube_rs::endpoints::player::resolve_stream_url_full(
            &format,
            &anon.player.decipherer,
            anon.session.po_token.as_deref(),
            None,
        )
        .map_err(|e| AppError::Internal(format!("Failed to resolve audio stream: {e}")))?;

        let (
            title,
            author,
            channel_id,
            thumbnail_url,
            length_seconds,
            description,
            view_count,
            is_live,
        ) = player_response
            .video_details
            .as_ref()
            .map(|details| {
                (
                    details.title.clone(),
                    details.author.clone(),
                    details.channel_id.clone(),
                    details
                        .thumbnail
                        .as_ref()
                        .and_then(|thumbnail| thumbnail.thumbnails.last())
                        .map(|thumbnail| thumbnail.url.clone())
                        .unwrap_or_default(),
                    details.length_seconds.clone(),
                    details.short_description.clone().unwrap_or_default(),
                    details.view_count.clone().unwrap_or_default(),
                    details.is_live.unwrap_or(false),
                )
            })
            .unwrap_or_default();

        Ok(AudioStreamInfo {
            itag: format.itag,
            url,
            mime_type: format.mime_type.clone(),
            bitrate: format.bitrate as f64,
            content_length: format
                .content_length
                .as_deref()
                .and_then(|value| value.parse().ok()),
            duration_ms: format
                .approx_duration_ms
                .as_deref()
                .and_then(|value| value.parse().ok()),
            language: format.language(),
            title,
            author,
            channel_id,
            thumbnail_url,
            length_seconds,
            description,
            view_count,
            is_live,
        })
    }

    /// Player response to pick an audio stream from.
    ///
    /// The generic player chain ends on the iOS client, which only exposes AAC
    /// audio, and most other clients return CDN URLs that refuse any range
    /// except one starting at byte 0 — which makes `<audio>` seeking impossible.
    /// The anonymous VISIONOS client returns Opus 160 kbps *and* URLs that
    /// serve arbitrary byte ranges, so it is the one the listen mode uses.
    async fn audio_player_response(
        &self,
        video_id: &str,
    ) -> Result<innertube_rs::PlayerResponse, AppError> {
        let mut fallback: Option<innertube_rs::PlayerResponse> = None;
        let anon = self.anon().await?;

        for client in ["VISIONOS", "ANDROID_VR", "MWEB", "WEB"] {
            let options = innertube_rs::GetVideoInfoOptions {
                client: Some(client.to_string()),
                po_token: None,
                playback_context: None,
            };
            match anon
                .get_basic_info(video_id, Some(&options))
                .await
            {
                Ok(info)
                    if info.player_response.playability_status.status == "OK"
                        && has_playable_audio(&info.player_response) =>
                {
                    // Opus (251) is the point of audio-only mode; VISIONOS has it.
                    if has_opus_audio(&info.player_response) {
                        return Ok(info.player_response);
                    }
                    if fallback.is_none() {
                        fallback = Some(info.player_response);
                    }
                }
                Err(e) => eprintln!("[YouTube] audio via {client} failed: {e}"),
                _ => {}
            }
        }

        if let Some(response) = fallback {
            return Ok(response);
        }

        // Restricted videos may only answer a signed-in session. Their URLs may
        // not be seekable, but playing is better than failing outright.
        let info = self
            .signed()
            .await?
            .get_basic_info(video_id, None)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get video info: {e}")))?;
        Ok(info.player_response)
    }

    /// Get watch next details (related videos and playlist context)
    pub async fn get_watch_next(
        &self,
        video_id: &str,
        playlist_id: Option<&str>,
        playlist_index: Option<usize>,
    ) -> Result<WatchNextResults, AppError> {
        let yt = self.signed().await?;
        let res = if let Some(pl_id) = playlist_id {
            yt.get_playlist_watch_next(video_id, pl_id, playlist_index)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to get watch next details: {e}")))?
        } else {
            yt.get_watch_next(video_id)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to get watch next details: {e}")))?
        };
        Ok(WatchNextResults::from(res))
    }

    /// Player session data for a video: the cached one when it matches, a
    /// fresh one otherwise (history can be reported without a prior load).
    async fn playback_info(&self, video_id: &str) -> Result<VideoInfo, AppError> {
        if let Some(info) = self.inner.last_video_info.read().await.as_ref() {
            if info.id() == Some(video_id) {
                return Ok(info.clone());
            }
        }

        let yt = self.signed().await?;
        let info = yt
            .get_basic_info(video_id, None)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get video info: {e}")))?;
        *self.inner.last_video_info.write().await = Some(info.clone());
        Ok(info)
    }

    /// Publish the initial playback event, which is what adds the video to the
    /// signed-in account's watch history.
    pub async fn record_watch_start(&self, video_id: &str) -> Result<(), AppError> {
        let info = self.playback_info(video_id).await?;
        let yt = self.signed().await?;
        info.add_to_watch_history(yt.session.as_ref())
            .await
            .map_err(|e| AppError::Internal(format!("Failed to record watch history: {e}")))?;
        Ok(())
    }

    /// Report playback progress so the resume position is remembered.
    pub async fn record_watch_time(&self, video_id: &str, seconds: f64) -> Result<(), AppError> {
        let info = self.playback_info(video_id).await?;
        let yt = self.signed().await?;
        info.update_watch_time(yt.session.as_ref(), seconds)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to update watch time: {e}")))?;
        Ok(())
    }

    /// Watch history of the signed-in account, newest first.
    pub async fn get_history(&self) -> Result<Vec<VideoItem>, AppError> {
        let yt = self.signed().await?;
        let feed = yt
            .get_history(None)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get watch history: {e}")))?;
        Ok(feed.videos.iter().map(VideoItem::from).collect())
    }

    /// One page of comments for a video (pass a continuation for the next page).
    pub async fn get_comments(
        &self,
        video_id: &str,
        continuation: Option<&str>,
    ) -> Result<CommentsPage, AppError> {
        let yt = self.signed().await?;
        let res = yt
            .get_comments_with_options(video_id, None, None, continuation)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch comments: {e}")))?;

        Ok(CommentsPage {
            total: res.total_comments_text,
            comments: res.comments.into_iter().map(comment_thread_from).collect(),
            continuation: res.continuation_token,
        })
    }

    /// Channel header plus its Videos tab.
    pub async fn get_channel_page(&self, channel_id: &str) -> Result<ChannelPage, AppError> {
        let yt = self.signed().await?;
        let about = yt.get_channel_about(channel_id).await.ok();
        let videos = yt
            .get_channel_videos(channel_id, None)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch channel videos: {e}")))?;

        let info = ChannelInfo {
            id: about
                .as_ref()
                .map(|about| about.channel_id.clone())
                .unwrap_or_else(|| channel_id.to_string()),
            name: about.as_ref().map(|about| about.title.clone()).unwrap_or_default(),
            avatar: about.as_ref().and_then(|about| about.avatar.clone()),
            banner: about.as_ref().and_then(|about| about.banner.clone()),
            subscriber_count: about
                .as_ref()
                .and_then(|about| about.subscriber_count.clone()),
            video_count: about.as_ref().and_then(|about| about.video_count.clone()),
            description: about.as_ref().and_then(|about| about.description.clone()),
        };

        Ok(ChannelPage {
            info,
            videos: videos
                .videos
                .into_iter()
                .map(|video| ChannelVideo {
                    id: video.video_id,
                    title: video.title,
                    thumbnail: video.thumbnail,
                    duration: video.duration,
                    published: video.published_time,
                    views: video.views,
                })
                .collect(),
            continuation: videos.continuation_token,
        })
    }

    /// Like / dislike / clear the rating of a video.
    pub async fn rate_video(&self, video_id: &str, rating: &str) -> Result<(), AppError> {
        let yt = self.signed().await?;
        let actions = yt.actions();
        let result = match rating {
            "like" => actions.like(video_id).await,
            "dislike" => actions.dislike(video_id).await,
            _ => actions.remove_rating(video_id).await,
        };
        result.map_err(|e| AppError::Internal(format!("Failed to rate video: {e}")))?;
        Ok(())
    }

    /// Subscribe to or unsubscribe from a channel.
    pub async fn set_subscription(
        &self,
        channel_id: &str,
        subscribed: bool,
    ) -> Result<(), AppError> {
        let yt = self.signed().await?;
        let ids = [channel_id];
        let actions = yt.actions();
        let result = if subscribed {
            actions.subscribe(&ids).await
        } else {
            actions.unsubscribe(&ids).await
        };
        result.map_err(|e| AppError::Internal(format!("Failed to update subscription: {e}")))?;
        Ok(())
    }

    /// Search with YouTube's own filters (type, sort order, upload date).
    pub async fn search_filtered(
        &self,
        query: &str,
        continuation_token: Option<&str>,
        search_type: Option<&str>,
        sort: Option<&str>,
        upload_date: Option<&str>,
    ) -> Result<SearchResults, AppError> {
        let filters = innertube_rs::SearchFilters {
            prioritize: match sort {
                Some("popularity") => Some(innertube_rs::SearchPrioritize::Popularity),
                _ => None,
            },
            upload_date: match upload_date {
                Some("today") => Some(innertube_rs::UploadDateFilter::Today),
                Some("week") => Some(innertube_rs::UploadDateFilter::Week),
                Some("month") => Some(innertube_rs::UploadDateFilter::Month),
                Some("year") => Some(innertube_rs::UploadDateFilter::Year),
                _ => None,
            },
            search_type: match search_type {
                Some("video") => Some(innertube_rs::SearchTypeFilter::Video),
                Some("channel") => Some(innertube_rs::SearchTypeFilter::Channel),
                Some("playlist") => Some(innertube_rs::SearchTypeFilter::Playlist),
                _ => None,
            },
            duration: None,
            features: Vec::new(),
        };

        let has_filters = filters.prioritize.is_some()
            || filters.upload_date.is_some()
            || filters.search_type.is_some();

        let yt = self.signed().await?;
        yt.search_with_filters(
            query,
            has_filters.then_some(&filters),
            continuation_token,
        )
        .await
        .map_err(|e| AppError::Internal(format!("YouTube search failed: {e}")))
    }

    /// Sync Set-Cookie headers manually (e.g. from Webview or external response)
    pub async fn sync_headers(
        &self,
        headers: &HeaderMap,
        db: &DatabaseConnection,
    ) -> Result<Option<String>, AppError> {
        let active_id = *self.inner.active_profile_id.read().await;
        self.inner.cookie_manager
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
            let yt = self.signed().await?;
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

fn comment_from(comment: innertube_rs::Comment) -> VideoComment {
    VideoComment {
        id: comment.comment_id,
        author: comment.author_name,
        author_thumbnail: comment.author_thumbnail,
        author_channel_id: comment.author_channel_id,
        text: comment.text,
        likes: comment.like_count_text,
        published_time: comment.published_time,
        is_pinned: comment.is_pinned,
        is_owner: comment.is_author_channel_owner,
        reply_count: comment.reply_count.map(|count| count as f64),
        replies: Vec::new(),
    }
}

fn comment_thread_from(thread: innertube_rs::CommentThread) -> VideoComment {
    let mut top = comment_from(thread.comment);
    top.replies = thread.replies.into_iter().map(comment_from).collect();
    top
}

fn playable_audio(format: &StreamingFormat) -> bool {
    format.is_audio_only()
        && !format.is_drc()
        && (format.url.is_some() || format.signature_cipher.is_some() || format.cipher.is_some())
}

fn has_playable_audio(response: &innertube_rs::PlayerResponse) -> bool {
    response
        .streaming_data
        .as_ref()
        .map(|streaming| streaming.adaptive_formats.iter().any(playable_audio))
        .unwrap_or(false)
}

fn has_opus_audio(response: &innertube_rs::PlayerResponse) -> bool {
    response
        .streaming_data
        .as_ref()
        .map(|streaming| {
            streaming
                .adaptive_formats
                .iter()
                .any(|format| format.itag == 251 && playable_audio(format))
        })
        .unwrap_or(false)
}

/// Pick the audio-only format worth playing.
///
/// Priority: the original audio track (never an auto-dub), then Opus 160 kbps
/// (`itag 251`), then AAC 128 kbps (`itag 140`), then the highest bitrate left.
/// DRC variants are skipped because they are loudness-normalised copies.
fn pick_audio_format(
    formats: &[StreamingFormat],
    language: Option<&str>,
    wanted_itag: Option<u32>,
) -> Option<StreamingFormat> {
    // A refresh keeps the track the session was already playing.
    if let Some(itag) = wanted_itag {
        if let Some(found) = formats
            .iter()
            .find(|format| format.itag == itag && playable_audio(format))
        {
            return Some(found.clone());
        }
    }

    let mut pool: Vec<&StreamingFormat> = formats.iter().filter(|format| playable_audio(format)).collect();
    if pool.is_empty() {
        return None;
    }

    // Multi-audio videos label the untouched track "… original"; prefer it over
    // every language dub, whatever the bitrates say.
    let labelled_original: Vec<&StreamingFormat> = pool
        .iter()
        .copied()
        .filter(|format| {
            format
                .audio_track
                .as_ref()
                .map(|track| track.display_name.to_ascii_lowercase().contains("original"))
                .unwrap_or(false)
        })
        .collect();
    if !labelled_original.is_empty() {
        pool = labelled_original;
    } else {
        let originals: Vec<&StreamingFormat> = pool
            .iter()
            .copied()
            .filter(|format| format.is_original())
            .collect();
        if !originals.is_empty() {
            pool = originals;
        }
    }

    // The default track is the one the uploader picked for the video.
    let defaults: Vec<&StreamingFormat> = pool
        .iter()
        .copied()
        .filter(|format| {
            format
                .audio_track
                .as_ref()
                .map(|track| track.audio_is_default)
                .unwrap_or(false)
        })
        .collect();
    if !defaults.is_empty() {
        pool = defaults;
    }

    if let Some(language) = language {
        let matching: Vec<&StreamingFormat> = pool
            .iter()
            .copied()
            .filter(|format| format.language().as_deref() == Some(language))
            .collect();
        if !matching.is_empty() {
            pool = matching;
        }
    }

    pool.into_iter()
        .min_by_key(|format| (itag_rank(format.itag), std::cmp::Reverse(format.bitrate)))
        .cloned()
}

fn itag_rank(itag: u32) -> u8 {
    match itag {
        251 => 0, // Opus 160 kbps, 48 kHz â€” the best audio YouTube serves
        140 => 1, // AAC 128 kbps
        250 => 2, // Opus 70 kbps
        139 => 3, // AAC 48 kbps
        249 => 4, // Opus 50 kbps
        _ => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use innertube_rs::models::format::AudioTrack;

    fn audio(itag: u32, bitrate: u64) -> StreamingFormat {
        StreamingFormat {
            itag,
            bitrate,
            mime_type: "audio/webm; codecs=\"opus\"".to_string(),
            url: Some(format!("https://example.invalid/audio/{itag}")),
            ..Default::default()
        }
    }

    fn with_track(mut format: StreamingFormat, name: &str, is_default: bool) -> StreamingFormat {
        format.audio_track = Some(AudioTrack {
            audio_is_default: is_default,
            display_name: name.to_string(),
            id: name.to_string(),
        });
        format
    }

    #[test]
    fn prefers_opus_160_over_aac() {
        let formats = vec![audio(140, 129_000), audio(251, 160_000)];
        let picked = pick_audio_format(&formats, None, None).expect("an audio format");
        assert_eq!(picked.itag, 251);
    }

    #[test]
    fn ignores_video_only_formats() {
        let mut video = audio(137, 4_000_000);
        video.mime_type = "video/mp4".to_string();
        let formats = vec![video, audio(140, 129_000)];
        let picked = pick_audio_format(&formats, None, None).expect("an audio format");
        assert_eq!(picked.itag, 140);
    }

    #[test]
    fn refresh_keeps_the_requested_itag() {
        let formats = vec![audio(251, 160_000), audio(140, 129_000)];
        let picked = pick_audio_format(&formats, None, Some(140)).expect("an audio format");
        assert_eq!(picked.itag, 140);
    }

    #[test]
    fn labelled_original_beats_a_higher_bitrate_dub() {
        let dub = with_track(audio(251, 160_000), "English (US)", true);
        let original = with_track(audio(140, 129_000), "Thai original", false);
        let formats = vec![dub, original];

        let picked = pick_audio_format(&formats, Some("th"), None).expect("an audio format");
        assert_eq!(picked.itag, 140, "a dub must never win over the original");
    }

    #[test]
    fn prefers_the_default_track_of_equal_originals() {
        let a = with_track(audio(251, 160_000), "Thai", false);
        let b = with_track(audio(251, 160_001), "Thai", true);
        let formats = vec![a, b];

        let picked = pick_audio_format(&formats, None, None).expect("an audio format");
        assert!(picked.audio_track.unwrap().audio_is_default);
    }

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
