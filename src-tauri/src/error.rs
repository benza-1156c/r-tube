use serde::Serialize;
use specta::Type;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Type)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[specta(type = String)] #[serde(serialize_with = "log_database")] String),

    #[error("{0}")]
    NotFound(#[specta(type = String)] #[serde(serialize_with = "log_not_found")] String),

    #[error("Validation error: {0}")]
    Validation(#[specta(type = String)] #[serde(serialize_with = "log_validation")] String),

    #[error("Internal error: {0}")]
    Internal(#[specta(type = String)] #[serde(serialize_with = "log_internal")] String),
}

fn log_error<S>(variant: &str, msg: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    eprintln!("\x1b[1;31m[AppError::{variant}]\x1b[0m {msg}");
    serializer.serialize_str(msg)
}

fn log_database<S>(msg: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    log_error("Database", msg, serializer)
}

fn log_not_found<S>(msg: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    log_error("NotFound", msg, serializer)
}

fn log_validation<S>(msg: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    log_error("Validation", msg, serializer)
}

fn log_internal<S>(msg: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    log_error("Internal", msg, serializer)
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

    /// Explicitly log this error to console and return self
    pub fn log(self) -> Self {
        let variant = match &self {
            AppError::Database(_) => "Database",
            AppError::NotFound(_) => "NotFound",
            AppError::Validation(_) => "Validation",
            AppError::Internal(_) => "Internal",
        };
        eprintln!("\x1b[1;31m[AppError::{variant}]\x1b[0m {self}");
        self
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
