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

pub mod error;
pub mod rga_list;
pub mod rga_text;
pub mod rich_text;
pub mod json_crdt;
pub mod document;
pub mod presence;
pub mod undo;

// RGA List exports
pub use rga_list::{RGAList, RGAListDelta, ListId, ListNode};

// RGA Text exports
pub use rga_text::{RGAText, RGATextDelta, TextId};

// Rich Text exports
pub use rich_text::{RichText, RichTextDelta, Mark, MarkId, MarkType, Anchor};

// JSON CRDT exports
pub use json_crdt::{
    JsonCrdt, JsonCrdtDelta, JsonPath, PathSegment, JsonValue,
    ArrayId, ObjectId, ObjectChange, ArrayChange,
};

// Document Store exports
pub use document::{
    Document, DocumentStore, DocumentId, DocumentType,
    CrdtValue, DocumentDelta, StoreChange,
    QueryOptions, SortField,
};

// Presence exports
pub use presence::{
    PresenceTracker, PresenceDelta, UserPresence, UserId, UserInfo,
    Cursor, CursorBuilder, CursorColors, UserStatus,
};

// Undo/Redo exports
pub use undo::{
    UndoManager, CollaborativeUndoManager,
    Operation, OperationId, GroupId,
    UndoableOperation, TextOperation, FormatOperation, JsonOperation,
};

// Error exports
pub use error::DbError;
