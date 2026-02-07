//! # mdcs-db
//!
//! Database layer for the MDCS (Merkle-Delta CRDT Store).
//!
//! This crate provides:
//! - Document-based API with path operations
//! - Collaborative text (RGAText, RichText)
//! - JSON/Object CRDT for flexible schemas
//! - Presence and awareness for real-time collaboration
//! - Undo/Redo support
//!
//! ## Example
//!
//! ```rust,ignore
//! use mdcs_db::{DocumentStore, DocumentId, JsonValue};
//!
//! let mut store = DocumentStore::new("replica_1");
//!
//! // Create a text document
//! let doc_id = store.create_text("My Document");
//! store.text_insert(&doc_id, 0, "Hello World").unwrap();
//!
//! // Create a JSON document
//! let json_id = store.create_json("Config");
//! store.json_set(&json_id, "name", JsonValue::String("Test".into())).unwrap();
//!
//! // Rich text with formatting
//! let rich_id = store.create_rich_text("Formatted");
//! store.rich_text_insert(&rich_id, 0, "Bold text").unwrap();
//! store.rich_text_bold(&rich_id, 0, 4).unwrap();
//! ```

pub mod document;
pub mod error;
pub mod json_crdt;
pub mod presence;
pub mod rga_list;
pub mod rga_text;
pub mod rich_text;
pub mod undo;

// RGA List exports
pub use rga_list::{ListId, ListNode, RGAList, RGAListDelta};

// RGA Text exports
pub use rga_text::{RGAText, RGATextDelta, TextId};

// Rich Text exports
pub use rich_text::{Anchor, Mark, MarkId, MarkType, RichText, RichTextDelta};

// JSON CRDT exports
pub use json_crdt::{
    ArrayChange, ArrayId, JsonCrdt, JsonCrdtDelta, JsonPath, JsonValue, ObjectChange, ObjectId,
    PathSegment,
};

// Document Store exports
pub use document::{
    CrdtValue, Document, DocumentDelta, DocumentId, DocumentStore, DocumentType, QueryOptions,
    SortField, StoreChange,
};

// Presence exports
pub use presence::{
    Cursor, CursorBuilder, CursorColors, PresenceDelta, PresenceTracker, UserId, UserInfo,
    UserPresence, UserStatus,
};

// Undo/Redo exports
pub use undo::{
    CollaborativeUndoManager, FormatOperation, GroupId, JsonOperation, Operation, OperationId,
    TextOperation, UndoManager, UndoableOperation,
};

// Error exports
pub use error::DbError;
