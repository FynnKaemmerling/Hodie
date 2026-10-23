use std::fmt;

use serde::{Serialize, Serializer};

use crate::position::InvalidKeys;

/// Errors cross the IPC boundary as plain codes (e.g. `validation.empty`).
/// The frontend translates them; the backend never sends user-facing text.
#[derive(Debug)]
pub enum AppError {
    NotFound,
    Validation(&'static str),
    Conflict(&'static str),
    DatabaseTooNew,
    Database(rusqlite::Error),
    /// Vault problems such as `vault.not_found` or `file.invalid`.
    Vault(&'static str),
    Io(std::io::Error),
    Internal(&'static str),
}

pub type Result<T> = std::result::Result<T, AppError>;

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::NotFound => "not_found",
            AppError::Validation(code) | AppError::Conflict(code) | AppError::Vault(code) => code,
            AppError::DatabaseTooNew => "db.too_new",
            AppError::Database(_) => "db.error",
            AppError::Io(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                "vault.read_only"
            }
            AppError::Io(_) => "vault.io",
            AppError::Internal(_) => "internal",
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(error) => write!(f, "database error: {error}"),
            AppError::Io(error) => write!(f, "file error: {error}"),
            AppError::Internal(detail) => write!(f, "internal error: {detail}"),
            other => f.write_str(other.code()),
        }
    }
}

impl std::error::Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        AppError::Database(error)
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::Io(error)
    }
}

impl From<InvalidKeys> for AppError {
    fn from(_: InvalidKeys) -> Self {
        AppError::Internal("invalid position keys")
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        if matches!(
            self,
            AppError::Database(_) | AppError::Io(_) | AppError::Internal(_)
        ) {
            eprintln!("{self}");
        }
        serializer.serialize_str(self.code())
    }
}
