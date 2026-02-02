//! Error types for the MDCS SDK.

use std::fmt;

/// Error type for SDK operations.
#[derive(Debug)]
pub enum SdkError {
    /// Document not found.
    DocumentNotFound(String),
    /// Peer not found.
    PeerNotFound(String),
    /// Connection failed.
    ConnectionFailed(String),
    /// Sync error.
    SyncError(String),
    /// Network error.
    NetworkError(String),
    /// Serialization error.
    SerializationError(String),
    /// Internal error.
    Internal(String),
}

impl fmt::Display for SdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SdkError::DocumentNotFound(id) => write!(f, "Document not found: {}", id),
            SdkError::PeerNotFound(id) => write!(f, "Peer not found: {}", id),
            SdkError::ConnectionFailed(e) => write!(f, "Connection failed: {}", e),
            SdkError::SyncError(e) => write!(f, "Sync error: {}", e),
            SdkError::NetworkError(e) => write!(f, "Network error: {}", e),
            SdkError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            SdkError::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for SdkError {}

/// Result type for SDK operations.
pub type Result<T> = std::result::Result<T, SdkError>;
