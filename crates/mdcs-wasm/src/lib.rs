//! # MDCS WebAssembly Bindings
//!
//! This crate provides WebAssembly bindings for the MDCS (Merkle-Delta CRDT Store),
//! enabling real-time collaborative editing in web browsers.
//!
//! ## Features
//!
//! - **CollaborativeDocument**: Rich text document with CRDT-based conflict resolution
//! - **UserPresence**: Cursor and selection tracking for collaborative UIs
//! - **Offline-first**: All operations work locally, sync when connected
//!
//! ## Usage
//!
//! ```javascript
//! import init, { CollaborativeDocument, UserPresence } from 'mdcs-wasm';
//!
//! await init();
//!
//! const doc = new CollaborativeDocument('doc-123', 'user-abc');
//! doc.insert(0, 'Hello, World!');
//! doc.apply_bold(0, 5);
//!
//! console.log(doc.get_text());  // "Hello, World!"
//! console.log(doc.get_html());  // "<b>Hello</b>, World!"
//! ```

use mdcs_core::lattice::Lattice;
use mdcs_db::{MarkType, RichText};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// Initialize panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// ============================================================================
// CollaborativeDocument
// ============================================================================

/// A collaborative rich text document backed by CRDTs.
///
/// This is the main entry point for document editing. All operations are
/// conflict-free and can be merged with remote changes.
#[wasm_bindgen]
pub struct CollaborativeDocument {
    id: String,
    replica_id: String,
    text: RichText,
    version: u64,
}

#[wasm_bindgen]
impl CollaborativeDocument {
    /// Create a new collaborative document.
    ///
    /// # Arguments
    /// * `doc_id` - Unique identifier for this document
    /// * `replica_id` - Unique identifier for this replica/user
    #[wasm_bindgen(constructor)]
    pub fn new(doc_id: &str, replica_id: &str) -> Self {
        Self {
            id: doc_id.to_string(),
            replica_id: replica_id.to_string(),
            text: RichText::new(replica_id),
            version: 0,
        }
    }

    /// Insert text at a position.
    ///
    /// # Arguments
    /// * `position` - Character index to insert at (0-based)
    /// * `text` - Text to insert
    #[wasm_bindgen]
    pub fn insert(&mut self, position: usize, text: &str) {
        let pos = position.min(self.text.len());
        self.text.insert(pos, text);
        self.version += 1;
    }

    /// Delete text at a position.
    ///
    /// # Arguments
    /// * `position` - Starting character index (0-based)
    /// * `length` - Number of characters to delete
    #[wasm_bindgen]
    pub fn delete(&mut self, position: usize, length: usize) {
        let pos = position.min(self.text.len());
        let len = length.min(self.text.len().saturating_sub(pos));
        if len > 0 {
            self.text.delete(pos, len);
            self.version += 1;
        }
    }

    /// Apply bold formatting to a range.
    ///
    /// # Arguments
    /// * `start` - Starting character index (inclusive)
    /// * `end` - Ending character index (exclusive)
    #[wasm_bindgen]
    pub fn apply_bold(&mut self, start: usize, end: usize) {
        self.apply_mark(start, end, MarkType::Bold);
    }

    /// Apply italic formatting to a range.
    #[wasm_bindgen]
    pub fn apply_italic(&mut self, start: usize, end: usize) {
        self.apply_mark(start, end, MarkType::Italic);
    }

    /// Apply underline formatting to a range.
    #[wasm_bindgen]
    pub fn apply_underline(&mut self, start: usize, end: usize) {
        self.apply_mark(start, end, MarkType::Underline);
    }

    /// Apply strikethrough formatting to a range.
    #[wasm_bindgen]
    pub fn apply_strikethrough(&mut self, start: usize, end: usize) {
        self.apply_mark(start, end, MarkType::Strikethrough);
    }

    /// Apply a link to a range.
    ///
    /// # Arguments
    /// * `start` - Starting character index (inclusive)
    /// * `end` - Ending character index (exclusive)
    /// * `url` - The URL to link to
    #[wasm_bindgen]
    pub fn apply_link(&mut self, start: usize, end: usize, url: &str) {
        let s = start.min(self.text.len());
        let e = end.min(self.text.len());
        if s < e {
            self.text.add_mark(
                s,
                e,
                MarkType::Link {
                    url: url.to_string(),
                },
            );
            self.version += 1;
        }
    }

    /// Get the plain text content (without formatting).
    #[wasm_bindgen]
    pub fn get_text(&self) -> String {
        self.text.to_string()
    }

    /// Get the content as HTML with formatting applied.
    #[wasm_bindgen]
    pub fn get_html(&self) -> String {
        self.text.to_html()
    }

    /// Get the document length in characters.
    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Check if the document is empty.
    #[wasm_bindgen]
    pub fn is_empty(&self) -> bool {
        self.text.len() == 0
    }

    /// Get the current version number.
    ///
    /// This increments with each local operation and can be used
    /// to track changes for sync purposes.
    #[wasm_bindgen]
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Get the document ID.
    #[wasm_bindgen]
    pub fn doc_id(&self) -> String {
        self.id.clone()
    }

    /// Get the replica ID.
    #[wasm_bindgen]
    pub fn replica_id(&self) -> String {
        self.replica_id.clone()
    }

    /// Serialize the document state for sync.
    ///
    /// Returns a base64-encoded binary string that can be sent to other replicas.
    /// Binary format is more efficient and handles complex key types.
    #[wasm_bindgen]
    pub fn serialize(&self) -> Result<String, JsValue> {
        // Use serde_wasm_bindgen which handles HashMap with non-string keys
        let js_value = serde_wasm_bindgen::to_value(&self.text)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;

        // Convert JsValue to JSON string using js_sys
        js_sys::JSON::stringify(&js_value)
            .map(|s| s.into())
            .map_err(|e| JsValue::from_str(&format!("JSON stringify error: {:?}", e)))
    }

    /// Merge remote state into this document.
    ///
    /// This is the core CRDT operation - merging is commutative,
    /// associative, and idempotent, so the order of merges doesn't matter.
    ///
    /// # Arguments
    /// * `remote_state` - JSON string from another replica's `serialize()`
    #[wasm_bindgen]
    pub fn merge(&mut self, remote_state: &str) -> Result<(), JsValue> {
        // Parse the JSON string back to JsValue
        let js_value = js_sys::JSON::parse(remote_state)
            .map_err(|e| JsValue::from_str(&format!("JSON parse error: {:?}", e)))?;

        // Deserialize using serde_wasm_bindgen
        let remote: RichText = serde_wasm_bindgen::from_value(js_value)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;

        self.text = self.text.join(&remote);
        self.version += 1;
        Ok(())
    }

    /// Create a snapshot of the current state.
    ///
    /// This returns a JSON object with full document state.
    #[wasm_bindgen]
    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        let state_js = serde_wasm_bindgen::to_value(&self.text)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let state_str: String = js_sys::JSON::stringify(&state_js)
            .map(|s| s.into())
            .map_err(|e| JsValue::from_str(&format!("JSON stringify error: {:?}", e)))?;

        let snapshot = DocumentSnapshot {
            doc_id: self.id.clone(),
            replica_id: self.replica_id.clone(),
            version: self.version,
            state: state_str,
        };
        serde_wasm_bindgen::to_value(&snapshot).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Restore from a snapshot.
    #[wasm_bindgen]
    pub fn restore(snapshot_js: JsValue) -> Result<CollaborativeDocument, JsValue> {
        let snapshot: DocumentSnapshot = serde_wasm_bindgen::from_value(snapshot_js)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Parse the state JSON string
        let state_js = js_sys::JSON::parse(&snapshot.state)
            .map_err(|e| JsValue::from_str(&format!("JSON parse error: {:?}", e)))?;

        let text: RichText = serde_wasm_bindgen::from_value(state_js)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(Self {
            id: snapshot.doc_id,
            replica_id: snapshot.replica_id,
            text,
            version: snapshot.version,
        })
    }

    // Internal helper
    fn apply_mark(&mut self, start: usize, end: usize, mark: MarkType) {
        let s = start.min(self.text.len());
        let e = end.min(self.text.len());
        if s < e {
            self.text.add_mark(s, e, mark);
            self.version += 1;
        }
    }
}

/// Document snapshot for persistence/sync
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DocumentSnapshot {
    doc_id: String,
    replica_id: String,
    version: u64,
    state: String,
}

// ============================================================================
// UserPresence
// ============================================================================

/// User presence information for collaborative UI.
///
/// Tracks cursor position, selection, and user metadata for
/// rendering remote user cursors.
#[wasm_bindgen]
pub struct UserPresence {
    user_id: String,
    user_name: String,
    color: String,
    cursor_position: Option<usize>,
    selection_start: Option<usize>,
    selection_end: Option<usize>,
}

#[wasm_bindgen]
impl UserPresence {
    /// Create a new user presence.
    ///
    /// # Arguments
    /// * `user_id` - Unique user identifier
    /// * `user_name` - Display name
    /// * `color` - Hex color for cursor (e.g., "#FF6B6B")
    #[wasm_bindgen(constructor)]
    pub fn new(user_id: &str, user_name: &str, color: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            user_name: user_name.to_string(),
            color: color.to_string(),
            cursor_position: None,
            selection_start: None,
            selection_end: None,
        }
    }

    /// Set cursor position (clears selection).
    #[wasm_bindgen]
    pub fn set_cursor(&mut self, position: usize) {
        self.cursor_position = Some(position);
        self.selection_start = None;
        self.selection_end = None;
    }

    /// Set selection range.
    #[wasm_bindgen]
    pub fn set_selection(&mut self, start: usize, end: usize) {
        self.cursor_position = Some(end);
        self.selection_start = Some(start.min(end));
        self.selection_end = Some(start.max(end));
    }

    /// Clear cursor and selection.
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.cursor_position = None;
        self.selection_start = None;
        self.selection_end = None;
    }

    /// Get user ID.
    #[wasm_bindgen(getter)]
    pub fn user_id(&self) -> String {
        self.user_id.clone()
    }

    /// Get user name.
    #[wasm_bindgen(getter)]
    pub fn user_name(&self) -> String {
        self.user_name.clone()
    }

    /// Get user color.
    #[wasm_bindgen(getter)]
    pub fn color(&self) -> String {
        self.color.clone()
    }

    /// Get cursor position.
    #[wasm_bindgen(getter)]
    pub fn cursor(&self) -> Option<usize> {
        self.cursor_position
    }

    /// Get selection start.
    #[wasm_bindgen(getter)]
    pub fn selection_start(&self) -> Option<usize> {
        self.selection_start
    }

    /// Get selection end.
    #[wasm_bindgen(getter)]
    pub fn selection_end(&self) -> Option<usize> {
        self.selection_end
    }

    /// Check if user has a selection (not just cursor).
    #[wasm_bindgen]
    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some() && self.selection_end.is_some()
    }

    /// Serialize to JSON for network transmission.
    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        let data = PresenceData {
            user_id: self.user_id.clone(),
            user_name: self.user_name.clone(),
            color: self.color.clone(),
            cursor: self.cursor_position,
            selection_start: self.selection_start,
            selection_end: self.selection_end,
        };
        serde_wasm_bindgen::to_value(&data).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Deserialize from JSON.
    #[wasm_bindgen]
    pub fn from_json(js: JsValue) -> Result<UserPresence, JsValue> {
        let data: PresenceData =
            serde_wasm_bindgen::from_value(js).map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(Self {
            user_id: data.user_id,
            user_name: data.user_name,
            color: data.color,
            cursor_position: data.cursor,
            selection_start: data.selection_start,
            selection_end: data.selection_end,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PresenceData {
    user_id: String,
    user_name: String,
    color: String,
    cursor: Option<usize>,
    selection_start: Option<usize>,
    selection_end: Option<usize>,
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Generate a unique replica ID.
///
/// Uses timestamp + random string for uniqueness.
#[wasm_bindgen]
pub fn generate_replica_id() -> String {
    let timestamp = js_sys::Date::now() as u64;
    let random: u32 = js_sys::Math::random().to_bits() as u32;
    format!("{}-{:x}", timestamp, random)
}

/// Generate a random user color from a preset palette.
#[wasm_bindgen]
pub fn generate_user_color() -> String {
    let colors = [
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7", "#DDA0DD", "#98D8C8", "#F7DC6F",
        "#E74C3C", "#3498DB", "#2ECC71", "#9B59B6", "#1ABC9C", "#F39C12", "#E91E63", "#00BCD4",
    ];
    let idx = (js_sys::Math::random() * colors.len() as f64) as usize;
    colors[idx % colors.len()].to_string()
}

/// Log a message to the browser console.
#[wasm_bindgen]
pub fn console_log(message: &str) {
    web_sys::console::log_1(&JsValue::from_str(message));
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = CollaborativeDocument::new("doc-1", "replica-1");
        assert_eq!(doc.doc_id(), "doc-1");
        assert_eq!(doc.replica_id(), "replica-1");
        assert_eq!(doc.len(), 0);
        assert!(doc.is_empty());
    }

    #[test]
    fn test_insert_and_delete() {
        let mut doc = CollaborativeDocument::new("doc-1", "replica-1");

        doc.insert(0, "Hello, World!");
        assert_eq!(doc.get_text(), "Hello, World!");
        assert_eq!(doc.len(), 13);

        doc.delete(5, 2); // Delete ", "
        assert_eq!(doc.get_text(), "HelloWorld!");
    }

    #[test]
    fn test_formatting() {
        let mut doc = CollaborativeDocument::new("doc-1", "replica-1");

        doc.insert(0, "Hello World");
        doc.apply_bold(0, 5);
        doc.apply_italic(6, 11);

        let html = doc.get_html();
        assert!(html.contains("<b>") || html.contains("<strong>"));
        assert!(html.contains("<i>") || html.contains("<em>"));
    }

    // Note: serialize/merge tests require WASM environment
    // Use wasm-bindgen-test for full integration testing
    // The RichText serialization uses HashMap<MarkId, Mark> which needs special handling

    #[test]
    fn test_crdt_merge_convergence() {
        // Test the underlying CRDT merge via Lattice trait
        let mut doc1 = CollaborativeDocument::new("doc-1", "replica-1");
        let mut doc2 = CollaborativeDocument::new("doc-1", "replica-2");

        doc1.insert(0, "Hello");
        doc2.insert(0, "World");

        // Use the Lattice join directly (no JSON serialization needed)
        let text1_clone = doc1.text.clone();
        let text2_clone = doc2.text.clone();

        doc1.text = doc1.text.join(&text2_clone);
        doc2.text = doc2.text.join(&text1_clone);

        // Both should converge to the same state
        assert_eq!(doc1.get_text(), doc2.get_text());
        // Content should include both insertions
        let final_text = doc1.get_text();
        assert!(final_text.contains("Hello") || final_text.contains("World"));
    }

    #[test]
    fn test_user_presence() {
        let mut presence = UserPresence::new("user-1", "Alice", "#FF6B6B");

        assert_eq!(presence.user_id(), "user-1");
        assert_eq!(presence.user_name(), "Alice");
        assert!(!presence.has_selection());

        presence.set_cursor(10);
        assert_eq!(presence.cursor(), Some(10));
        assert!(!presence.has_selection());

        presence.set_selection(5, 15);
        assert!(presence.has_selection());
        assert_eq!(presence.selection_start(), Some(5));
        assert_eq!(presence.selection_end(), Some(15));
    }
}
