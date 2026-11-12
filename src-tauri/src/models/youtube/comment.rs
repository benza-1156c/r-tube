use serde::{Deserialize, Serialize};
use specta::Type;

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

