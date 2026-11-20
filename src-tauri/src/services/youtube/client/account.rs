use reqwest::header::HeaderMap;
use sea_orm::DatabaseConnection;
use serde_json::Value;

use crate::error::AppError;
use crate::models::youtube::VideoRating;

use super::{CurrentUserProfile, YoutubeClient};

fn extract_text(v: &Value, ptr1: &str, ptr2: &str) -> Option<String> {
    v.pointer(ptr1)
        .or_else(|| v.pointer(ptr2))
        .and_then(Value::as_str)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn scan_profile(
    v: &Value,
    name: &mut Option<String>,
    email: &mut Option<String>,
    photo: &mut Option<String>,
    handle: &mut Option<String>,
) {
    if let Some(header) = v
        .get("activeAccountHeaderRenderer")
        .or_else(|| v.get("accountHeaderRenderer"))
        .or_else(|| v.get("googleAccountHeaderRenderer"))
        .or_else(|| v.get("accountItemSectionHeaderRenderer"))
    {
        if name.is_none() {
            *name = extract_text(
                header,
                "/accountName/simpleText",
                "/accountName/runs/0/text",
            )
            .or_else(|| extract_text(header, "/title/simpleText", "/title/runs/0/text"))
            .or_else(|| extract_text(header, "/name/simpleText", "/name/runs/0/text"));
        }
        if email.is_none() {
            *email = extract_text(header, "/email/simpleText", "/email/runs/0/text");
        }
        if handle.is_none() {
            *handle = extract_text(
                header,
                "/channelHandle/simpleText",
                "/channelHandle/runs/0/text",
            );
        }
        if photo.is_none() {
            *photo = header
                .pointer("/accountPhoto/thumbnails")
                .or_else(|| header.pointer("/thumbnail/thumbnails"))
                .and_then(Value::as_array)
                .and_then(|arr| arr.last())
                .and_then(|t| t.get("url"))
                .and_then(Value::as_str)
                .map(ToString::to_string);
        }
    }

    if let Some(item) = v.get("accountItemRenderer") {
        let is_selected = item
            .get("isSelected")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if is_selected || name.is_none() {
            if let Some(n) =
                extract_text(item, "/accountName/simpleText", "/accountName/runs/0/text")
            {
                *name = Some(n);
            }
            if let Some(h) = extract_text(
                item,
                "/channelHandle/simpleText",
                "/channelHandle/runs/0/text",
            ) {
                *handle = Some(h);
            }
            if let Some(p) = item
                .pointer("/accountPhoto/thumbnails")
                .and_then(Value::as_array)
                .and_then(|arr| arr.last())
                .and_then(|t| t.get("url"))
                .and_then(Value::as_str)
            {
                *photo = Some(p.to_string());
            }
        }
    }

    if let Some(s) = v.as_str() {
        let trimmed = s.trim();
        if email.is_none()
            && trimmed.contains('@')
            && !trimmed.starts_with('@')
            && trimmed.contains('.')
            && !trimmed.contains('/')
            && !trimmed.contains(' ')
        {
            *email = Some(trimmed.to_string());
        } else if handle.is_none()
            && trimmed.starts_with('@')
            && !trimmed.contains(' ')
            && !trimmed.contains('/')
            && trimmed.len() > 2
        {
            *handle = Some(trimmed.to_string());
        }
    }

    match v {
        Value::Object(map) => {
            for val in map.values() {
                scan_profile(val, name, email, photo, handle);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                scan_profile(item, name, email, photo, handle);
            }
        }
        _ => {}
    }
}

impl YoutubeClient {
    /// Sync Set-Cookie headers manually (e.g. from Webview or external response)
    pub async fn sync_headers(
        &self,
        headers: &HeaderMap,
        db: &DatabaseConnection,
    ) -> Result<Option<String>, AppError> {
        let active_id = *self.inner.active_profile_id.read().await;
        self.inner
            .cookie_manager
            .sync_from_headers(headers, active_id, db)
            .await
    }

    /// Like / dislike / clear the rating of a video.
    pub async fn rate_video(&self, video_id: &str, rating: VideoRating) -> Result<(), AppError> {
        let yt = self.signed().await?;
        let actions = yt.actions();
        let result = match rating {
            VideoRating::Like => actions.like(video_id).await,
            VideoRating::Dislike => actions.dislike(video_id).await,
            VideoRating::None => actions.remove_rating(video_id).await,
        };
        result.map_err(|e| AppError::Internal(format!("Failed to rate video: {e}")))?;
        Ok(())
    }

    /// Subscribe to or unsubscribe from a channel.
    pub async fn set_subscription(
        &self,
        channel_id: &str,
        subscribed: bool,
    ) -> Result<(), AppError> {
        let yt = self.signed().await?;
        let ids = [channel_id];
        let actions = yt.actions();
        let result = if subscribed {
            actions.subscribe(&ids).await
        } else {
            actions.unsubscribe(&ids).await
        };
        result.map_err(|e| AppError::Internal(format!("Failed to update subscription: {e}")))?;
        Ok(())
    }

    pub async fn get_account_info(&self) -> Result<CurrentUserProfile, AppError> {
        let mut name = None;
        let mut email = None;
        let mut photo = None;
        let mut handle = None;

        let menu_payload = serde_json::json!({
            "deviceTheme": "DEVICE_THEME_SUPPORTED",
            "userInterfaceTheme": "USER_INTERFACE_THEME_DARK"
        });
        if let Ok(menu_raw) = self
            .post_with_cookie_sync("/account/account_menu", menu_payload, None)
            .await
        {
            scan_profile(&menu_raw, &mut name, &mut email, &mut photo, &mut handle);
        }

        if email.is_none() || name.is_none() {
            let list_payload = serde_json::json!({
                "callCircumstance": "SWITCHING_USERS_FULL"
            });
            if let Ok(list_raw) = self
                .post_with_cookie_sync("/account/accounts_list", list_payload, None)
                .await
            {
                scan_profile(&list_raw, &mut name, &mut email, &mut photo, &mut handle);
            }
        }

        if name.is_none() {
            let yt = self.signed().await?;
            if let Ok(accounts) = yt.account().get_accounts().await {
                if let Some(acc) = accounts
                    .iter()
                    .find(|a| a.is_selected)
                    .or_else(|| accounts.first())
                {
                    name = Some(acc.account_name.clone());
                    if photo.is_none() {
                        photo = acc.account_photo.clone();
                    }
                }
            }
        }

        let final_name = name
            .or_else(|| email.clone())
            .or_else(|| handle.clone())
            .ok_or_else(|| {
                AppError::NotFound(
                    "Could not read the signed-in YouTube account. Please sign in again."
                        .to_string(),
                )
            })?;

        Ok(CurrentUserProfile {
            name: final_name,
            email: email.or_else(|| handle.clone()).unwrap_or_default(),
            avatar_url: photo.unwrap_or_default(),
            handle,
        })
    }
}
