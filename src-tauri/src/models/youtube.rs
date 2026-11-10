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
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItem {
    pub id: String,
    pub title: String,
    pub thumbnail_url: String,
    pub author_name: String,
    pub author_avatar_url: String,
    pub video_count: Option<u32>,
    pub first_video_id: Option<String>,
    pub last_updated: Option<String>,
    pub view_count: Option<String>,
    pub badge_text: String,
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
    pub playlists: Vec<PlaylistItem>,
    pub continuation_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BrowseFeed {
    pub browse_id: String,
    pub videos: Vec<VideoItem>,
    pub playlists: Vec<PlaylistItem>,
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
    pub playlists: Vec<PlaylistItem>,
    pub continuation_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrackInfo {
    pub id: String,
    pub display_name: String,
    pub is_default: bool,
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
    pub audio_track: Option<AudioTrackInfo>,
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
    pub captions: Vec<CaptionTrack>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CurrentUserProfile {
    pub name: String,
    pub avatar_url: Option<String>,
    pub channel_id: Option<String>,
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
        }
    }
}

impl From<&innertube_rs::PlaylistNode> for PlaylistItem {
    fn from(p: &innertube_rs::PlaylistNode) -> Self {
        let thumbnail_url = p
            .thumbnails
            .best_url()
            .or_else(|| p.thumbnails.first_url())
            .unwrap_or_default()
            .to_string();

        let author_name = p
            .author
            .as_ref()
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "YouTube".to_string());

        let author_avatar_url = p
            .author
            .as_ref()
            .and_then(|a| a.thumbnails.best_url().or_else(|| a.thumbnails.first_url()))
            .unwrap_or_default()
            .to_string();

        let watch_ep = p.endpoint.as_ref().and_then(|e| e.watch.as_ref());
        let first_video_id = watch_ep
            .map(|w| w.video_id.clone())
            .filter(|id| !id.is_empty());

        let badge_text = if let Some(count) = p.video_count {
            format!("{count} videos")
        } else {
            "Mix".to_string()
        };

        Self {
            id: p.id.clone(),
            title: p.title.clone(),
            thumbnail_url,
            author_name,
            author_avatar_url,
            video_count: p.video_count,
            first_video_id,
            last_updated: p.last_updated.clone(),
            view_count: p.view_count.clone(),
            badge_text,
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
            playlists: feed.playlists.iter().map(PlaylistItem::from).collect(),
            continuation_token: feed.continuation_token,
        }
    }
}

impl From<innertube_rs::BrowseFeed> for BrowseFeed {
    fn from(feed: innertube_rs::BrowseFeed) -> Self {
        Self {
            browse_id: feed.browse_id,
            videos: feed.videos.iter().map(VideoItem::from).collect(),
            playlists: feed.playlists.iter().map(PlaylistItem::from).collect(),
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
        let mut videos = Vec::new();
        let mut playlists = Vec::new();

        for item in res.items {
            match item {
                innertube_rs::SearchResultItem::Video(v) => {
                    let thumb = v
                        .thumbnails
                        .last()
                        .or_else(|| v.thumbnails.first())
                        .map(|t| t.url.clone())
                        .unwrap_or_default();

                    videos.push(VideoItem {
                        id: v.video_id,
                        title: v.title,
                        thumbnail_url: thumb,
                        duration: v.duration.unwrap_or_default(),
                        channel_name: v.author,
                        channel_avatar_url: String::new(),
                        is_verified: false,
                        is_live: false,
                        view_count: v.view_count.unwrap_or_default(),
                        published_at: v.published_time.unwrap_or_default(),
                        playlist_id: None,
                    });
                }
                innertube_rs::SearchResultItem::Playlist(p) => {
                    let thumb = p
                        .thumbnails
                        .last()
                        .or_else(|| p.thumbnails.first())
                        .map(|t| t.url.clone())
                        .unwrap_or_default();

                    let video_count_num = p.video_count.as_deref().and_then(|vc| {
                        vc.chars()
                            .filter(|c| c.is_ascii_digit())
                            .collect::<String>()
                            .parse()
                            .ok()
                    });

                    let badge_text = p
                        .video_count
                        .clone()
                        .unwrap_or_else(|| "Playlist".to_string());

                    playlists.push(PlaylistItem {
                        id: p.playlist_id.clone(),
                        title: p.title,
                        thumbnail_url: thumb,
                        author_name: p.author,
                        author_avatar_url: String::new(),
                        video_count: video_count_num,
                        first_video_id: None,
                        last_updated: None,
                        view_count: None,
                        badge_text,
                    });
                }
                _ => {}
            }
        }

        Self {
            query: res.query,
            videos,
            playlists,
            continuation_token: res.continuation_token,
        }
    }
}

impl From<&innertube_rs::StreamingFormat> for VideoStreamFormat {
    fn from(f: &innertube_rs::StreamingFormat) -> Self {
        let is_audio = f.mime_type.starts_with("audio/")
            || f.audio_track.is_some()
            || f.mime_type.contains("mp4a")
            || f.mime_type.contains("opus");
        let is_video = f.mime_type.starts_with("video/");
        Self {
            itag: f.itag,
            mime_type: f.mime_type.clone(),
            quality_label: f.quality_label.clone(),
            bitrate: f.bitrate as u32,
            url: f.url.clone(),
            width: f.width,
            height: f.height,
            is_audio,
            is_video,
            audio_track: f.audio_track.as_ref().map(|t| AudioTrackInfo {
                id: t.id.clone(),
                display_name: t.display_name.clone(),
                is_default: t.audio_is_default,
            }),
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
            captions: Vec::new(),
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
                    is_live: r.is_live,
                    view_count: r.view_count_text.unwrap_or_default(),
                    published_at: r.published_time_text.unwrap_or_default(),
                    playlist_id: None,
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
