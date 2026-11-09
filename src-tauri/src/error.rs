use serde::Serialize;
use specta::Type;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Type)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("{0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Return default "Not found" error with no message needed:
    /// `AppError::not_found()` -> "Not found"
    pub fn not_found() -> Self {
        Self::NotFound("Not found".to_string())
    }

    /// Automatically appends "not found" to the item:
    /// `AppError::not_found_item("window")` -> "window not found"
    pub fn not_found_item(item: impl std::fmt::Display) -> Self {
        let text = item.to_string();
        if text.to_lowercase().ends_with("not found") {
            Self::NotFound(text)
        } else {
            Self::NotFound(format!("{text} not found"))
        }
    }

    /// Helper for internal errors:
    /// `AppError::internal("something broke")`
    pub fn internal(msg: impl std::fmt::Display) -> Self {
        Self::Internal(msg.to_string())
    }

    /// Helper for validation errors:
    /// `AppError::validation("invalid input")`
    pub fn validation(msg: impl std::fmt::Display) -> Self {
        Self::Validation(msg.to_string())
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        AppError::Database(err.to_string())
    }
}

#[macro_export]
macro_rules! not_found {
    () => {
        $crate::error::AppError::not_found()
    };
    ($item:expr) => {
        $crate::error::AppError::not_found_item($item)
    };
}
