use innertube_rs::VideoInfo;

use crate::error::AppError;
use crate::models::youtube::{CommentsPage, PlayerResponse, VideoComment, WatchNextResults};

use super::{options_for_client, YoutubeClient};

impl YoutubeClient {
    /// HLS is the only playback path: every segment arrives as its own URL,
    /// proxied so seeking and expired CDN URLs keep working.
    pub async fn get_video_info(&self, video_id: &str) -> Result<PlayerResponse, AppError> {
        crate::utils::proxy::ensure_proxy()
            .await
            .map_err(|e| AppError::Internal(format!("Local stream proxy unavailable: {e}")))?;

        // `/next` is not fetched here: the watch page requests it through
        // `get_watch_next` at the same time, and the player must not wait for
        // it. The player response already carries everything playback needs.
        let raw = self.fetch_playback_info(video_id).await?;

        // Keep the playback session data (cpn + tracking URLs) for the
        // history and watch-time events this video will report.
        *self.inner.last_video_info.write().await = Some(raw.clone());

        let mut player_res = PlayerResponse::from(raw.player_response);

        // googlevideo sends no CORS headers, so the manifest is only reachable
        // through the local proxy.
        player_res.hls_manifest_url = player_res
            .hls_manifest_url
            .take()
            .map(|raw| crate::utils::proxy::wrap_hls_url(&raw, video_id));

        Ok(player_res)
    }

    /// Fresh player session data from the client that carries an HLS manifest.
    ///
    /// The library's `/player` call defaults to the web client, which returns
    /// no `hlsManifestUrl` for regular videos (its fallback chain only runs
    /// when the web response is unplayable). VISIONOS is the client that
    /// serves HLS and it also carries the Opus audio track, captions and the
    /// playback tracking used for history.
    async fn fetch_playback_info(&self, video_id: &str) -> Result<VideoInfo, AppError> {
        let yt = self.signed().await?;
        yt.get_basic_info(video_id, Some(&options_for_client("VISIONOS")))
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get video info: {e}")))
    }

    /// Re-mint the master playlist with the client that served it originally.
    pub async fn hls_manifest_url(&self, video_id: &str) -> Option<String> {
        self.fetch_playback_info(video_id)
            .await
            .ok()
            .and_then(|info| {
                info.player_response
                    .streaming_data
                    .and_then(|streaming| streaming.hls_manifest_url)
            })
    }

    /// Get watch next details (related videos and playlist context)
    pub async fn get_watch_next(
        &self,
        video_id: &str,
        playlist_id: Option<&str>,
        playlist_index: Option<usize>,
    ) -> Result<WatchNextResults, AppError> {
        let yt = self.signed().await?;
        let res = if let Some(pl_id) = playlist_id {
            yt.get_playlist_watch_next(video_id, pl_id, playlist_index)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to get watch next details: {e}")))?
        } else {
            yt.get_watch_next(video_id)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to get watch next details: {e}")))?
        };
        Ok(WatchNextResults::from(res))
    }

    /// The session data of the video that was loaded last, when it is this one.
    pub(super) async fn cached_info(&self, video_id: &str) -> Option<VideoInfo> {
        self.inner
            .last_video_info
            .read()
            .await
            .as_ref()
            .filter(|info| info.id() == Some(video_id))
            .cloned()
    }

    /// Player session data for a video: the cached one when it matches, a
    /// fresh one otherwise (history can be reported without a prior load).
    pub(super) async fn playback_info(&self, video_id: &str) -> Result<VideoInfo, AppError> {
        if let Some(info) = self.cached_info(video_id).await {
            return Ok(info);
        }

        let info = self.fetch_playback_info(video_id).await?;
        *self.inner.last_video_info.write().await = Some(info.clone());
        Ok(info)
    }

    /// Publish the initial playback event, which is what adds the video to the
    /// signed-in account's watch history.
    pub async fn record_watch_start(&self, video_id: &str) -> Result<(), AppError> {
        let info = self.playback_info(video_id).await?;
        let yt = self.signed().await?;
        info.add_to_watch_history(yt.session.as_ref())
            .await
            .map_err(|e| AppError::Internal(format!("Failed to record watch history: {e}")))?;
        Ok(())
    }

    /// Report playback progress so the resume position is remembered.
    pub async fn record_watch_time(&self, video_id: &str, seconds: f64) -> Result<(), AppError> {
        let info = self.playback_info(video_id).await?;
        let yt = self.signed().await?;
        info.update_watch_time(yt.session.as_ref(), seconds)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to update watch time: {e}")))?;
        Ok(())
    }

    /// One page of comments for a video (pass a continuation for the next page).
    pub async fn get_comments(
        &self,
        video_id: &str,
        continuation: Option<&str>,
    ) -> Result<CommentsPage, AppError> {
        let yt = self.signed().await?;
        let res = yt
            .get_comments_with_options(video_id, None, None, continuation)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch comments: {e}")))?;

        Ok(CommentsPage {
            total: res.total_comments_text,
            comments: res.comments.into_iter().map(comment_thread_from).collect(),
            continuation: res.continuation_token,
        })
    }
}

fn comment_from(comment: innertube_rs::Comment) -> VideoComment {
    VideoComment {
        id: comment.comment_id,
        author: comment.author_name,
        author_thumbnail: comment.author_thumbnail,
        author_channel_id: comment.author_channel_id,
        text: comment.text,
        likes: comment.like_count_text,
        published_time: comment.published_time,
        is_pinned: comment.is_pinned,
        is_owner: comment.is_author_channel_owner,
        reply_count: comment.reply_count.map(|count| count as f64),
        replies: Vec::new(),
    }
}

fn comment_thread_from(thread: innertube_rs::CommentThread) -> VideoComment {
    let mut top = comment_from(thread.comment);
    top.replies = thread.replies.into_iter().map(comment_from).collect();
    top
}
