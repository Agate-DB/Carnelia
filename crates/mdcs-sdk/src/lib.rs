//! MDCS SDK - High-level SDK for building collaborative applications
//!
//! This SDK provides a simple, ergonomic API for building real-time
//! collaborative applications using the MDCS (Merkle-Delta CRDT Store).
//!
//! # Quick Start
//!
//! ```rust
//! use mdcs_sdk::{Client, ClientConfig};
//!
//! fn main() {
//!     // Create a client with a unique replica ID
//!     let config = ClientConfig {
//!         user_name: "Alice".to_string(),
//!         ..Default::default()
//!     };
//!     let client = Client::new_with_memory_transport(config);
//!
//!     // Create a collaborative session
//!     let session = client.create_session("my-session");
//!
//!     // Open a text document
//!     let doc = session.open_text_doc("meeting-notes");
//!
//!     // Edit the document
//!     doc.write().insert(0, "# Meeting Notes\n");
//!
//!     // Read the content
//!     let content = doc.read().get_text();
//! }
//! ```
//!
//! # Architecture
//!
//! The SDK is organized into several modules:
//!
//! - [`client`] - Main entry point for creating and managing collaborative sessions
//! - [`document`] - Document types (text, rich text, JSON)
//! - [`presence`] - Real-time cursor and user presence
//! - [`sync`] - Network synchronization and peer management
//! - [`network`] - Network transport abstractions
//! - [`session`] - Session management for collaborative editing
//! - [`error`] - Error types

pub mod client;
pub mod document;
pub mod error;
pub mod network;
pub mod presence;
pub mod session;
pub mod sync;

// Re-exports for convenience
pub use client::{Client, ClientConfig, ClientConfigBuilder};
pub use document::{CollaborativeDoc, DocEvent, JsonDoc, RichTextDoc, TextDoc};
pub use error::{Result, SdkError};
pub use network::{MemoryTransport, Message, NetworkTransport, Peer, PeerId, PeerState};
pub use presence::{Awareness, AwarenessEvent, CursorInfo, UserPresenceInfo};
pub use session::{Session, SessionEvent};
pub use sync::{SyncConfig, SyncConfigBuilder, SyncEvent, SyncManager};

// Re-export commonly used types from mdcs-db
pub use mdcs_db::{
    json_crdt::{JsonPath, JsonValue},
    presence::{Cursor, UserId, UserInfo, UserStatus},
    rich_text::MarkType,
};

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::client::{Client, ClientConfig};
    pub use crate::document::{CollaborativeDoc, JsonDoc, RichTextDoc, TextDoc};
    pub use crate::error::SdkError;
    pub use crate::network::{NetworkTransport, Peer, PeerId};
    pub use crate::presence::{Awareness, CursorInfo};
    pub use crate::session::Session;
    pub use crate::sync::{SyncConfig, SyncManager};
}
