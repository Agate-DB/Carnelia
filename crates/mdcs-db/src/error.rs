//! Error types for the database layer.

use thiserror::Error;

/// Errors that can occur in database operations.
#[derive(Error, Debug, Clone)]
pub enum DbError {
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("Invalid index: {index} (length: {length})")]
    IndexOutOfBounds { index: usize, length: usize },

    #[error("Invalid path segment: {0}")]
    InvalidPath(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Operation not supported: {0}")]
    UnsupportedOperation(String),

    #[error("Concurrent modification detected")]
    ConcurrentModification,
}

impl From<serde_json::Error> for DbError {
    fn from(err: serde_json::Error) -> Self {
        DbError::SerializationError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, DbError>;
