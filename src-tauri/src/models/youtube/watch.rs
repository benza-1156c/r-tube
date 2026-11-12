use serde::{Deserialize, Serialize};
use specta::Type;

use super::playlist::PlaylistPanelItem;
use super::video::{VideoChapter, VideoItem};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WatchNextResults {
    pub current_video_id: String,
    pub current_title: Option<String>,
    pub current_author: Option<String>,
    pub related_videos: Vec<VideoItem>,
    pub playlist_items: Vec<PlaylistPanelItem>,
    #[serde(default)]
    pub chapters: Vec<VideoChapter>,
    pub continuation_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Conversions from innertube-rs types
// ---------------------------------------------------------------------------

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
                    progress_percent: r.progress_percent,
                }
            })
            .collect();

        let chapters = res
            .chapters
            .into_iter()
            .map(VideoChapter::from)
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
            chapters,
            continuation_token: res.continuation_token,
        }
    }
}

