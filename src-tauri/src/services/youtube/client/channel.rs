use crate::error::AppError;
use crate::models::youtube::{ChannelInfo, ChannelPage, ChannelVideo};

use super::YoutubeClient;

impl YoutubeClient {
    /// Channel header plus its Videos tab.
    pub async fn get_channel_page(&self, channel_id: &str) -> Result<ChannelPage, AppError> {
        let yt = self.signed().await?;
        let about = yt.get_channel_about(channel_id).await.ok();
        let videos = yt
            .get_channel_videos(channel_id, None)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch channel videos: {e}")))?;

        let info = ChannelInfo {
            id: about
                .as_ref()
                .map(|about| about.channel_id.clone())
                .unwrap_or_else(|| channel_id.to_string()),
            name: about
                .as_ref()
                .map(|about| about.title.clone())
                .unwrap_or_default(),
            avatar: about.as_ref().and_then(|about| about.avatar.clone()),
            banner: about.as_ref().and_then(|about| about.banner.clone()),
            subscriber_count: about
                .as_ref()
                .and_then(|about| about.subscriber_count.clone()),
            video_count: about.as_ref().and_then(|about| about.video_count.clone()),
            description: about.as_ref().and_then(|about| about.description.clone()),
        };

        Ok(ChannelPage {
            info,
            videos: videos
                .videos
                .into_iter()
                .map(|video| ChannelVideo {
                    id: video.video_id,
                    title: video.title,
                    thumbnail: video.thumbnail,
                    duration: video.duration,
                    published: video.published_time,
                    views: video.views,
                    progress_percent: video.progress_percent,
                })
                .collect(),
            continuation: videos.continuation_token,
        })
    }
}