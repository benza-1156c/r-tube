use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VideoItem {
    pub id: String,
    pub title: String,
    pub thumbnail_url: String,
    pub duration: String,
    pub channel_name: String,
    pub channel_avatar_url: String,
    pub is_verified: bool,
    pub is_live: bool,
    pub view_count: String,
    pub published_at: String,
    #[serde(default)]
    pub playlist_id: Option<String>,
    #[serde(default)]
    pub progress_percent: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CaptionTrack {
    pub language_code: String,
    pub name: String,
    pub vtt_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlayerResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub channel_name: String,
    pub channel_id: String,
    pub length_seconds: String,
    pub view_count: String,
    pub is_live: bool,
    pub thumbnail_url: String,
    /// Proxied HLS master playlist. The only playback source the player uses:
    /// HLS serves one URL per segment, which is what makes seeking work.
    pub hls_manifest_url: Option<String>,
    pub captions: Vec<CaptionTrack>,
}

/// Metadata + a proxied URL for the best audio-only stream of a video.
///
/// The URL is the local proxy endpoint (`/audio?url=…`), never the raw CDN URL:
/// the proxy is what adds `Range` support and re-mints expired URLs.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AudioStreamInfo {
    pub itag: u32,
    pub url: String,
    pub mime_type: String,
    pub bitrate: f64,
    pub content_length: Option<f64>,
    pub duration_ms: Option<f64>,
    pub language: Option<String>,
    pub title: String,
    pub author: String,
    pub channel_id: String,
    pub thumbnail_url: String,
    pub length_seconds: String,
    pub description: String,
    pub view_count: String,
    pub is_live: bool,
    pub captions: Vec<CaptionTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VideoChapter {
    pub title: String,
    pub start_ms: u32,
    pub thumbnail: Option<String>,
}

// ---------------------------------------------------------------------------
// Conversions from innertube-rs types
// ---------------------------------------------------------------------------

impl From<&innertube_rs::VideoNode> for VideoItem {
    fn from(v: &innertube_rs::VideoNode) -> Self {
        let thumbnail_url = v
            .thumbnails
            .best_url()
            .or_else(|| v.thumbnails.first_url())
            .unwrap_or_default()
            .to_string();

        let channel_name = v
            .author
            .as_ref()
            .map(|a| a.name.clone())
            .unwrap_or_default();
        let channel_avatar_url = v
            .author
            .as_ref()
            .and_then(|a| a.thumbnails.best_url().or_else(|| a.thumbnails.first_url()))
            .unwrap_or_default()
            .to_string();
        let is_verified = v.author.as_ref().map(|a| a.is_verified).unwrap_or(false);

        Self {
            id: v.id.clone(),
            title: v.title.clone(),
            thumbnail_url,
            duration: if v.is_live {
                "LIVE".to_string()
            } else {
                v.duration.clone().unwrap_or_default()
            },
            channel_name,
            channel_avatar_url,
            is_verified,
            is_live: v.is_live,
            view_count: v.view_count.clone().unwrap_or_default(),
            published_at: v.published_time.clone().unwrap_or_default(),
            playlist_id: None,
            progress_percent: v.progress_percent,
        }
    }
}

impl From<innertube_rs::PlayerResponse> for PlayerResponse {
    fn from(res: innertube_rs::PlayerResponse) -> Self {
        let (
            id,
            title,
            description,
            channel_name,
            channel_id,
            length_seconds,
            view_count,
            is_live,
            thumbnail_url,
        ) = if let Some(details) = res.video_details {
            let thumb = details
                .thumbnail
                .and_then(|t| t.thumbnails.last().cloned())
                .map(|t| t.url)
                .unwrap_or_default();
            (
                details.video_id,
                details.title,
                details.short_description.unwrap_or_default(),
                details.author,
                details.channel_id,
                details.length_seconds,
                details.view_count.unwrap_or_default(),
                details.is_live.unwrap_or(false),
                thumb,
            )
        } else {
            (
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                false,
                String::new(),
            )
        };

        let hls_manifest_url = res
            .streaming_data
            .and_then(|streaming| streaming.hls_manifest_url);

        Self {
            id,
            title,
            description,
            channel_name,
            channel_id,
            length_seconds,
            view_count,
            is_live,
            thumbnail_url,
            hls_manifest_url,
            captions: Vec::new(),
        }
    }
}

impl From<innertube_rs::VideoChapter> for VideoChapter {
    fn from(c: innertube_rs::VideoChapter) -> Self {
        Self {
            title: c.title,
            start_ms: c.start_ms.min(u32::MAX as u64) as u32,
            thumbnail: c.thumbnail,
        }
    }
}

