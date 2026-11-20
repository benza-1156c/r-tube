use serde::{Deserialize, Serialize};
use specta::Type;

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
    #[serde(default)]
    pub progress_percent: Option<u32>,
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
pub struct CurrentUserProfile {
    pub name: String,
    pub avatar_url: Option<String>,
    pub channel_id: Option<String>,
}

