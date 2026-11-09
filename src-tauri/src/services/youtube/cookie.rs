use reqwest::header::{HeaderMap, SET_COOKIE};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::db::entities::profile;
use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct CookieManager {
    cookies: Arc<RwLock<HashMap<String, String>>>,
    sync_lock: Arc<Mutex<()>>,
}

impl CookieManager {
    pub fn new(initial_cookies: Option<&str>) -> Self {
        let map = initial_cookies.map(parse_cookie_header).unwrap_or_default();

        Self {
            cookies: Arc::new(RwLock::new(map)),
            sync_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Load fresh cookies into the manager
    pub async fn set_cookies(&self, cookie_str: &str) {
        let _guard = self.sync_lock.lock().await;
        let mut map = self.cookies.write().await;
        *map = parse_cookie_header(cookie_str);
    }

    /// Get current cookie header string (e.g. "SID=...; SAPISID=...")
    pub async fn get_cookie_header(&self) -> String {
        let map = self.cookies.read().await;
        format_cookie_header(&map)
    }

    /// Check if critical authentication cookies are present
    pub async fn is_authenticated(&self) -> bool {
        let map = self.cookies.read().await;
        (map.contains_key("SAPISID") || map.contains_key("__Secure-3PAPISID"))
            && (map.contains_key("SID") || map.contains_key("__Secure-1PSID"))
    }

    /// Extract Set-Cookie from HTTP response headers, merge with existing cookies,
    /// and atomically persist to SQLite profile under lock.
    pub async fn sync_from_headers(
        &self,
        headers: &HeaderMap,
        active_profile_id: Option<i32>,
        db: &DatabaseConnection,
    ) -> Result<Option<String>, AppError> {
        let set_cookies: Vec<&str> = headers
            .get_all(SET_COOKIE)
            .iter()
            .filter_map(|val| val.to_str().ok())
            .collect();

        if set_cookies.is_empty() {
            return Ok(None);
        }

        let updates = parse_set_cookie_headers(set_cookies);
        if updates.is_empty() {
            return Ok(None);
        }

        // Acquire lock to prevent race conditions during cookie rotation
        let _guard = self.sync_lock.lock().await;

        let mut current_map = self.cookies.write().await;
        for (k, v) in updates {
            current_map.insert(k, v);
        }

        let merged_cookie = format_cookie_header(&current_map);

        // Update active profile in SQLite
        if let Some(profile_id) = active_profile_id {
            if let Ok(Some(existing)) = profile::Entity::find_by_id(profile_id).one(db).await {
                let mut active: profile::ActiveModel = existing.into();
                active.cookies = Set(merged_cookie.clone());
                active.updated_at = Set(chrono::Utc::now());
                let _ = active.update(db).await;
            }
        }

        Ok(Some(merged_cookie))
    }
}

pub fn parse_cookie_header(header: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for part in header.split(';') {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=') {
            let k = k.trim().to_string();
            let v = v.trim().to_string();
            if !k.is_empty() {
                map.insert(k, v);
            }
        }
    }
    map
}

pub fn format_cookie_header(map: &HashMap<String, String>) -> String {
    map.iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<String>>()
        .join("; ")
}

pub fn parse_set_cookie_headers<'a, I>(set_cookies: I) -> Vec<(String, String)>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut updates = Vec::new();
    for header in set_cookies {
        let first = header.split(';').next().unwrap_or("").trim();
        if let Some((k, v)) = first.split_once('=') {
            let k = k.trim().to_string();
            let v = v.trim().to_string();
            if !k.is_empty() && !v.is_empty() {
                updates.push((k, v));
            }
        }
    }
    updates
}
