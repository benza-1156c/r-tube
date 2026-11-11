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
    pub is_mix: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchSuggestionItem {
    pub text: String,
    pub subtitle: Option<String>,
    pub thumbnail: Option<String>,
    pub is_history: bool,
    pub channel_id: Option<String>,
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
pub struct VideoComment {
    pub id: String,
    pub author: String,
    pub author_thumbnail: Option<String>,
    pub author_channel_id: Option<String>,
    pub text: String,
    pub likes: Option<String>,
    pub published_time: Option<String>,
    pub is_pinned: bool,
    pub is_owner: bool,
    pub reply_count: Option<f64>,
    pub replies: Vec<VideoComment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CommentsPage {
    pub total: Option<String>,
    pub comments: Vec<VideoComment>,
    pub continuation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChannelInfo {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub subscriber_count: Option<String>,
    pub video_count: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChannelVideo {
    pub id: String,
    pub title: String,
    pub thumbnail: Option<String>,
    pub duration: Option<String>,
    pub published: Option<String>,
    pub views: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChannelPage {
    pub info: ChannelInfo,
    pub videos: Vec<ChannelVideo>,
    pub continuation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistPanelItem {
    pub video_id: String,
    pub title: String,
    pub author: String,
    pub index: Option<u32>,
    pub is_selected: bool,
    pub thumbnail_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WatchNextResults {
    pub current_video_id: String,
    pub current_title: Option<String>,
    pub current_author: Option<String>,
    pub related_videos: Vec<VideoItem>,
    pub playlist_items: Vec<PlaylistPanelItem>,
    pub continuation_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CurrentUserProfile {
    pub name: String,
    pub avatar_url: Option<String>,
    pub channel_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistDetailView {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub author_id: Option<String>,
    pub author_avatar_url: Option<String>,
    pub description: Option<String>,
    pub video_count: Option<u32>,
    pub view_count: Option<String>,
    pub last_updated: Option<String>,
    pub thumbnail: Option<String>,
    pub videos: Vec<PlaylistVideoItem>,
    pub continuation_token: Option<String>,
    pub is_editable: bool,
    pub is_mix: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistVideoItem {
    pub id: String,
    pub title: String,
    pub author: String,
    pub author_id: Option<String>,
    pub duration: Option<String>,
    pub duration_ms: Option<f64>,
    pub thumbnail: Option<String>,
    pub index: Option<u32>,
    pub is_playable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistContinuationResult {
    pub videos: Vec<PlaylistVideoItem>,
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

        let badge_text = if p.is_mix {
            "Mix".to_string()
        } else if let Some(count) = p.video_count {
            format!("{count} videos")
        } else {
            "Playlist".to_string()
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
            is_mix: p.is_mix,
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

                    let badge_text = if p.is_mix {
                        "Mix".to_string()
                    } else {
                        p.video_count
                            .clone()
                            .unwrap_or_else(|| "Playlist".to_string())
                    };

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
                        is_mix: p.is_mix,
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

        let playlist_items = res
            .playlist_items
            .into_iter()
            .map(|p| {
                let thumb = p
                    .thumbnails
                    .last()
                    .or_else(|| p.thumbnails.first())
                    .map(|t| t.url.clone())
                    .unwrap_or_default();
                PlaylistPanelItem {
                    video_id: p.video_id,
                    title: p.title,
                    author: p.author,
                    index: p.index.map(|i| i as u32),
                    is_selected: p.is_selected,
                    thumbnail_url: thumb,
                }
            })
            .collect();

        Self {
            current_video_id: res.current_video_id,
            current_title: res.current_title,
            current_author: res.current_author,
            related_videos,
            playlist_items,
            continuation_token: res.continuation_token,
        }
    }
}

impl From<&innertube_rs::PlaylistVideoItem> for PlaylistVideoItem {
    fn from(pv: &innertube_rs::PlaylistVideoItem) -> Self {
        Self {
            id: pv.id.clone(),
            title: pv.title.clone(),
            author: pv.author.clone(),
            author_id: pv.author_id.clone(),
            duration: pv.duration.clone(),
            duration_ms: pv.duration_ms.map(|ms| ms as f64),
            thumbnail: pv.thumbnail.clone(),
            index: pv.index,
            is_playable: pv.is_playable,
        }
    }
}

impl From<innertube_rs::SearchSuggestion> for SearchSuggestionItem {
    fn from(s: innertube_rs::SearchSuggestion) -> Self {
        Self {
            text: s.text,
            subtitle: s.subtitle,
            thumbnail: s.thumbnail,
            is_history: s.is_history,
            channel_id: s.channel_id,
        }
    }
}

impl From<&innertube_rs::SearchSuggestion> for SearchSuggestionItem {
    fn from(s: &innertube_rs::SearchSuggestion) -> Self {
        Self {
            text: s.text.clone(),
            subtitle: s.subtitle.clone(),
            thumbnail: s.thumbnail.clone(),
            is_history: s.is_history,
            channel_id: s.channel_id.clone(),
        }
    }
}

impl From<innertube_rs::PlaylistView> for PlaylistDetailView {
    fn from(pv: innertube_rs::PlaylistView) -> Self {
        Self {
            id: pv.id,
            title: pv.title,
            author: pv.author,
            author_id: pv.author_id,
            author_avatar_url: None,
            description: pv.description,
            video_count: pv.video_count,
            view_count: pv.view_count,
            last_updated: pv.last_updated,
            thumbnail: pv.thumbnail,
            videos: pv.videos.iter().map(PlaylistVideoItem::from).collect(),
            continuation_token: pv.continuation_token,
            is_editable: pv.is_editable,
            is_mix: pv.is_mix,
        }
    }
}

impl From<innertube_rs::PlaylistContinuation> for PlaylistContinuationResult {
    fn from(pc: innertube_rs::PlaylistContinuation) -> Self {
        Self {
            videos: pc.videos.iter().map(PlaylistVideoItem::from).collect(),
            continuation_token: pc.continuation_token,
        }
    }
}
