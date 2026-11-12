use serde::{Deserialize, Serialize};
use specta::Type;

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
pub struct PlaylistContinuationResult {
    pub videos: Vec<PlaylistVideoItem>,
    pub continuation_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Conversions from innertube-rs types
// ---------------------------------------------------------------------------

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

