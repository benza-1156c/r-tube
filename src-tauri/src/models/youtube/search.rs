use serde::{Deserialize, Serialize};
use specta::Type;

use super::playlist::PlaylistItem;
use super::video::VideoItem;

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
pub struct SearchResults {
    pub query: String,
    pub videos: Vec<VideoItem>,
    pub playlists: Vec<PlaylistItem>,
    pub continuation_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Conversions from innertube-rs types
// ---------------------------------------------------------------------------

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
                        progress_percent: v.progress_percent,
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

