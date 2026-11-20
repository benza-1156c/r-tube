use serde::{Deserialize, Serialize};
use specta::Type;

use super::playlist::PlaylistItem;
use super::video::VideoItem;

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

// ---------------------------------------------------------------------------
// Conversions from innertube-rs types
// ---------------------------------------------------------------------------

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

