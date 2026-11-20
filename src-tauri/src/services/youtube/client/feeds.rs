use innertube_rs::{BrowseFeed, HomeFeed, SearchResults, TrendingFeed};

use crate::error::AppError;
use crate::models::youtube::{
    PlaylistContinuationResult, PlaylistDetailView, SearchSuggestionItem, VideoItem,
};

use super::YoutubeClient;

impl YoutubeClient {
    /// Fetch YouTube Home Feed
    pub async fn get_home_feed(
        &self,
        continuation_token: Option<&str>,
        params: Option<&str>,
    ) -> Result<HomeFeed, AppError> {
        let yt = self.signed().await?;
        let token = continuation_token.or(params);
        if let Some(t) = token {
            yt.get_home_feed_continuation(t).await.map_err(|e| {
                AppError::Internal(format!("Failed to fetch home feed continuation: {e}"))
            })
        } else {
            yt.get_home_feed(None)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to fetch home feed: {e}")))
        }
    }

    /// Fetch YouTube Subscriptions Feed, or one of its continuation pages.
    pub async fn get_subscriptions_feed(
        &self,
        continuation: Option<&str>,
    ) -> Result<BrowseFeed, AppError> {
        let yt = self.signed().await?;
        match continuation {
            Some(token) => {
                innertube_rs::endpoints::feed::get_browse_continuation(&yt.session, token)
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!(
                            "Failed to fetch subscriptions continuation: {e}"
                        ))
                    })
            }
            None => yt.get_subscriptions_feed().await.map_err(|e| {
                AppError::Internal(format!("Failed to fetch subscriptions feed: {e}"))
            }),
        }
    }

    /// Watch history of the signed-in account, newest first.
    pub async fn get_history(&self) -> Result<Vec<VideoItem>, AppError> {
        let yt = self.signed().await?;
        let feed = yt
            .get_history(None)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get watch history: {e}")))?;
        Ok(feed.videos.iter().map(VideoItem::from).collect())
    }

    /// Fetch YouTube Trending Feed
    pub async fn get_trending(&self) -> Result<TrendingFeed, AppError> {
        let yt = self.signed().await?;
        yt.get_trending(None)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch trending feed: {e}")))
    }

    /// Search YouTube
    pub async fn search(
        &self,
        query: &str,
        continuation_token: Option<&str>,
    ) -> Result<SearchResults, AppError> {
        let yt = self.signed().await?;
        yt.search(query, continuation_token)
            .await
            .map_err(|e| AppError::Internal(format!("YouTube search failed: {e}")))
    }

    /// Fetch full YouTube playlist details
    pub async fn get_playlist(&self, playlist_id: &str) -> Result<PlaylistDetailView, AppError> {
        let yt = self.signed().await?;
        yt.get_playlist(playlist_id)
            .await
            .map(PlaylistDetailView::from)
            .map_err(|e| AppError::Internal(format!("Failed to fetch playlist: {e}")))
    }

    /// Fetch playlist continuation videos
    pub async fn get_playlist_continuation(
        &self,
        token: &str,
    ) -> Result<PlaylistContinuationResult, AppError> {
        let yt = self.signed().await?;
        yt.get_playlist_continuation(token)
            .await
            .map(PlaylistContinuationResult::from)
            .map_err(|e| AppError::Internal(format!("Failed to fetch playlist continuation: {e}")))
    }

    /// Get rich search autocomplete suggestions with subtitles and thumbnails from innertube-rs
    pub async fn get_search_suggestions(
        &self,
        query: &str,
    ) -> Result<Vec<SearchSuggestionItem>, AppError> {
        let yt = self.signed().await?;
        let res = yt
            .get_search_suggestions(query, false)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch search suggestions: {e}")))?;

        Ok(res
            .items
            .into_iter()
            .map(SearchSuggestionItem::from)
            .collect())
    }

    /// Search with YouTube's own filters (type, sort order, upload date).
    pub async fn search_filtered(
        &self,
        query: &str,
        continuation_token: Option<&str>,
        search_type: Option<&str>,
        sort: Option<&str>,
        upload_date: Option<&str>,
    ) -> Result<SearchResults, AppError> {
        let filters = innertube_rs::SearchFilters {
            prioritize: match sort {
                Some("popularity") => Some(innertube_rs::SearchPrioritize::Popularity),
                _ => None,
            },
            upload_date: match upload_date {
                Some("today") => Some(innertube_rs::UploadDateFilter::Today),
                Some("week") => Some(innertube_rs::UploadDateFilter::Week),
                Some("month") => Some(innertube_rs::UploadDateFilter::Month),
                Some("year") => Some(innertube_rs::UploadDateFilter::Year),
                _ => None,
            },
            search_type: match search_type {
                Some("video") => Some(innertube_rs::SearchTypeFilter::Video),
                Some("channel") => Some(innertube_rs::SearchTypeFilter::Channel),
                Some("playlist") => Some(innertube_rs::SearchTypeFilter::Playlist),
                _ => None,
            },
            duration: None,
            features: Vec::new(),
        };

        let has_filters = filters.prioritize.is_some()
            || filters.upload_date.is_some()
            || filters.search_type.is_some();

        let yt = self.signed().await?;
        yt.search_with_filters(query, has_filters.then_some(&filters), continuation_token)
            .await
            .map_err(|e| AppError::Internal(format!("YouTube search failed: {e}")))
    }
}
