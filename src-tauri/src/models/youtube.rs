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
    pub view_count: String,
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FilterChip {
    pub text: String,
    pub params: Option<String>,
    pub is_selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HomeFeed {
    pub filter_chips: Vec<FilterChip>,
    pub videos: Vec<VideoItem>,
    pub continuation_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BrowseFeed {
    pub browse_id: String,
    pub videos: Vec<VideoItem>,
    pub continuation_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TrendingFeed {
    pub current_tab: String,
    pub videos: Vec<VideoItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub query: String,
    pub videos: Vec<VideoItem>,
    pub continuation_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VideoStreamFormat {
    pub itag: u32,
    pub mime_type: String,
    pub quality_label: Option<String>,
    pub bitrate: u32,
    pub url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub is_audio: bool,
    pub is_video: bool,
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
    pub formats: Vec<VideoStreamFormat>,
    pub adaptive_formats: Vec<VideoStreamFormat>,
    pub hls_manifest_url: Option<String>,
    pub dash_manifest_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WatchNextResults {
    pub current_video_id: String,
    pub current_title: Option<String>,
    pub current_author: Option<String>,
    pub related_videos: Vec<VideoItem>,
    pub continuation_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Conversion from innertube-rs types
// ---------------------------------------------------------------------------

impl From<&innertube_rs::VideoNode> for VideoItem {
    fn from(v: &innertube_rs::VideoNode) -> Self {
        let thumbnail_url = v
            .thumbnails
            .best_url()
            .or_else(|| v.thumbnails.first_url())
            .unwrap_or_default()
            .to_string();

        let channel_avatar_url = v
            .author
            .as_ref()
            .and_then(|a| a.thumbnails.best_url().or_else(|| a.thumbnails.first_url()))
            .unwrap_or_default()
            .to_string();

        Self {
            id: v.id.clone(),
            title: v.title.clone(),
            thumbnail_url,
            duration: if v.is_live {
                "LIVE".to_string()
            } else {
                v.duration.clone().unwrap_or_default()
            },
            channel_name: v
                .author
                .as_ref()
                .map(|a| a.name.clone())
                .unwrap_or_default(),
            channel_avatar_url,
            is_verified: v.author.as_ref().map(|a| a.is_verified).unwrap_or(false),
            view_count: v.view_count.clone().unwrap_or_default(),
            published_at: v.published_time.clone().unwrap_or_default(),
        }
    }
}

impl From<innertube_rs::HomeFeed> for HomeFeed {
    fn from(feed: innertube_rs::HomeFeed) -> Self {
        Self {
            filter_chips: feed
                .filter_chips
                .into_iter()
                .map(|c| FilterChip {
                    text: c.text,
                    params: c.params,
                    is_selected: c.is_selected,
                })
                .collect(),
            videos: feed.videos.iter().map(VideoItem::from).collect(),
            continuation_token: feed.continuation_token,
        }
    }
}

impl From<innertube_rs::BrowseFeed> for BrowseFeed {
    fn from(feed: innertube_rs::BrowseFeed) -> Self {
        Self {
            browse_id: feed.browse_id,
            videos: feed.videos.iter().map(VideoItem::from).collect(),
            continuation_token: feed.continuation_token,
        }
    }
}

impl From<innertube_rs::TrendingFeed> for TrendingFeed {
    fn from(feed: innertube_rs::TrendingFeed) -> Self {
        Self {
            current_tab: feed.current_tab,
            videos: feed.videos.iter().map(VideoItem::from).collect(),
        }
    }
}

impl From<innertube_rs::SearchResults> for SearchResults {
    fn from(res: innertube_rs::SearchResults) -> Self {
        let videos = res
            .items
            .into_iter()
            .filter_map(|item| match item {
                innertube_rs::SearchResultItem::Video(v) => {
                    let thumb = v
                        .thumbnails
                        .last()
                        .or_else(|| v.thumbnails.first())
                        .map(|t| t.url.clone())
                        .unwrap_or_default();
                    Some(VideoItem {
                        id: v.video_id,
                        title: v.title,
                        thumbnail_url: thumb,
                        duration: v.duration.unwrap_or_default(),
                        channel_name: v.author,
                        channel_avatar_url: String::new(),
                        is_verified: false,
                        view_count: v.view_count.unwrap_or_default(),
                        published_at: v.published_time.unwrap_or_default(),
                    })
                }
                _ => None,
            })
            .collect();

        Self {
            query: res.query,
            videos,
            continuation_token: res.continuation_token,
        }
    }
}

impl From<&innertube_rs::StreamingFormat> for VideoStreamFormat {
    fn from(f: &innertube_rs::StreamingFormat) -> Self {
        Self {
            itag: f.itag,
            mime_type: f.mime_type.clone(),
            quality_label: f.quality_label.clone(),
            bitrate: f.bitrate as u32,
            url: f.url.clone(),
            width: f.width,
            height: f.height,
            is_audio: f.mime_type.starts_with("audio/"),
            is_video: f.mime_type.starts_with("video/"),
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

        let (formats, adaptive_formats, hls_manifest_url, dash_manifest_url) =
            if let Some(streaming) = res.streaming_data {
                (
                    streaming
                        .formats
                        .iter()
                        .map(VideoStreamFormat::from)
                        .collect(),
                    streaming
                        .adaptive_formats
                        .iter()
                        .map(VideoStreamFormat::from)
                        .collect(),
                    streaming.hls_manifest_url,
                    streaming.dash_manifest_url,
                )
            } else {
                (Vec::new(), Vec::new(), None, None)
            };

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
            formats,
            adaptive_formats,
            hls_manifest_url,
            dash_manifest_url,
        }
    }
}

impl From<innertube_rs::WatchNextResults> for WatchNextResults {
    fn from(res: innertube_rs::WatchNextResults) -> Self {
        let related_videos = res
            .related_videos
            .into_iter()
            .map(|r| {
                let thumb = r
                    .thumbnails
                    .last()
                    .or_else(|| r.thumbnails.first())
                    .map(|t| t.url.clone())
                    .unwrap_or_default();
                VideoItem {
                    id: r.video_id,
                    title: r.title,
                    thumbnail_url: thumb,
                    duration: if r.is_live {
                        "LIVE".to_string()
                    } else {
                        r.duration_text.unwrap_or_default()
                    },
                    channel_name: r.author,
                    channel_avatar_url: String::new(),
                    is_verified: false,
                    view_count: r.view_count_text.unwrap_or_default(),
                    published_at: r.published_time_text.unwrap_or_default(),
                }
            })
            .collect();

        Self {
            current_video_id: res.current_video_id,
            current_title: res.current_title,
            current_author: res.current_author,
            related_videos,
            continuation_token: res.continuation_token,
        }
    }
}
