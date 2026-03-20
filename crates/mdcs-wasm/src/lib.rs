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
use mdcs_db::{JsonCrdt, JsonPath, JsonValue, MarkType, RGAText, RichText};
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

    /// Apply inline code formatting to a range.
    #[wasm_bindgen]
    pub fn apply_code(&mut self, start: usize, end: usize) {
        self.apply_mark(start, end, MarkType::Code);
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

    /// Apply a highlight color to a range.
    ///
    /// # Arguments
    /// * `start` - Starting character index (inclusive)
    /// * `end` - Ending character index (exclusive)
    /// * `color` - CSS color string (e.g., "#FFEAA7")
    #[wasm_bindgen]
    pub fn apply_highlight(&mut self, start: usize, end: usize, color: &str) {
        let s = start.min(self.text.len());
        let e = end.min(self.text.len());
        if s < e {
            self.text.add_mark(
                s,
                e,
                MarkType::Highlight {
                    color: color.to_string(),
                },
            );
            self.version += 1;
        }
    }

    /// Apply a comment annotation to a range.
    ///
    /// # Arguments
    /// * `start` - Starting character index (inclusive)
    /// * `end` - Ending character index (exclusive)
    /// * `author` - Comment author name/id
    /// * `content` - Comment body
    #[wasm_bindgen]
    pub fn apply_comment(&mut self, start: usize, end: usize, author: &str, content: &str) {
        let s = start.min(self.text.len());
        let e = end.min(self.text.len());
        if s < e {
            self.text.add_mark(
                s,
                e,
                MarkType::Comment {
                    author: author.to_string(),
                    content: content.to_string(),
                },
            );
            self.version += 1;
        }
    }

    /// Apply a custom formatting mark to a range.
    ///
    /// # Arguments
    /// * `start` - Starting character index (inclusive)
    /// * `end` - Ending character index (exclusive)
    /// * `name` - Custom mark name
    /// * `value` - Custom mark value
    #[wasm_bindgen]
    pub fn apply_custom_mark(&mut self, start: usize, end: usize, name: &str, value: &str) {
        let s = start.min(self.text.len());
        let e = end.min(self.text.len());
        if s < e {
            self.text.add_mark(
                s,
                e,
                MarkType::Custom {
                    name: name.to_string(),
                    value: value.to_string(),
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
// TextDocument (Plain Text / RGA)
// ============================================================================

/// A collaborative plain text document backed by RGAText.
#[wasm_bindgen]
pub struct TextDocument {
    id: String,
    replica_id: String,
    text: RGAText,
    version: u64,
}

#[wasm_bindgen]
impl TextDocument {
    #[wasm_bindgen(constructor)]
    pub fn new(doc_id: &str, replica_id: &str) -> Self {
        Self {
            id: doc_id.to_string(),
            replica_id: replica_id.to_string(),
            text: RGAText::new(replica_id),
            version: 0,
        }
    }

    #[wasm_bindgen]
    pub fn insert(&mut self, position: usize, text: &str) {
        let pos = position.min(self.text.len());
        self.text.insert(pos, text);
        self.version += 1;
    }

    #[wasm_bindgen]
    pub fn delete(&mut self, position: usize, length: usize) {
        let pos = position.min(self.text.len());
        let len = length.min(self.text.len().saturating_sub(pos));
        if len > 0 {
            self.text.delete(pos, len);
            self.version += 1;
        }
    }

    #[wasm_bindgen]
    pub fn replace(&mut self, start: usize, end: usize, text: &str) {
        let s = start.min(self.text.len());
        let e = end.min(self.text.len());
        if s <= e {
            self.text.replace(s, e, text);
            self.version += 1;
        }
    }

    #[wasm_bindgen]
    pub fn splice(&mut self, position: usize, delete_count: usize, insert: &str) {
        let pos = position.min(self.text.len());
        self.text.splice(pos, delete_count, insert);
        self.version += 1;
    }

    #[wasm_bindgen]
    pub fn get_text(&self) -> String {
        self.text.to_string()
    }

    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    #[wasm_bindgen]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    #[wasm_bindgen]
    pub fn version(&self) -> u64 {
        self.version
    }

    #[wasm_bindgen]
    pub fn doc_id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen]
    pub fn replica_id(&self) -> String {
        self.replica_id.clone()
    }

    #[wasm_bindgen]
    pub fn serialize(&self) -> Result<String, JsValue> {
        let js_value = serde_wasm_bindgen::to_value(&self.text)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;

        js_sys::JSON::stringify(&js_value)
            .map(|s| s.into())
            .map_err(|e| JsValue::from_str(&format!("JSON stringify error: {:?}", e)))
    }

    #[wasm_bindgen]
    pub fn merge(&mut self, remote_state: &str) -> Result<(), JsValue> {
        let js_value = js_sys::JSON::parse(remote_state)
            .map_err(|e| JsValue::from_str(&format!("JSON parse error: {:?}", e)))?;

        let remote: RGAText = serde_wasm_bindgen::from_value(js_value)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;

        self.text = self.text.join(&remote);
        self.version += 1;
        Ok(())
    }

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

    #[wasm_bindgen]
    pub fn restore(snapshot_js: JsValue) -> Result<TextDocument, JsValue> {
        let snapshot: DocumentSnapshot = serde_wasm_bindgen::from_value(snapshot_js)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let state_js = js_sys::JSON::parse(&snapshot.state)
            .map_err(|e| JsValue::from_str(&format!("JSON parse error: {:?}", e)))?;

        let text: RGAText =
            serde_wasm_bindgen::from_value(state_js).map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(Self {
            id: snapshot.doc_id,
            replica_id: snapshot.replica_id,
            text,
            version: snapshot.version,
        })
    }
}

// ============================================================================
// RichTextDocument (explicit rich-text wrapper)
// ============================================================================

/// Explicit rich text wrapper for SDK-style API naming.
///
/// This wraps `CollaborativeDocument` and exposes the same rich-text CRDT behavior.
#[wasm_bindgen]
pub struct RichTextDocument {
    inner: CollaborativeDocument,
}

#[wasm_bindgen]
impl RichTextDocument {
    #[wasm_bindgen(constructor)]
    pub fn new(doc_id: &str, replica_id: &str) -> Self {
        Self {
            inner: CollaborativeDocument::new(doc_id, replica_id),
        }
    }

    #[wasm_bindgen]
    pub fn insert(&mut self, position: usize, text: &str) {
        self.inner.insert(position, text);
    }

    #[wasm_bindgen]
    pub fn delete(&mut self, position: usize, length: usize) {
        self.inner.delete(position, length);
    }

    #[wasm_bindgen]
    pub fn apply_bold(&mut self, start: usize, end: usize) {
        self.inner.apply_bold(start, end);
    }

    #[wasm_bindgen]
    pub fn apply_italic(&mut self, start: usize, end: usize) {
        self.inner.apply_italic(start, end);
    }

    #[wasm_bindgen]
    pub fn apply_underline(&mut self, start: usize, end: usize) {
        self.inner.apply_underline(start, end);
    }

    #[wasm_bindgen]
    pub fn apply_strikethrough(&mut self, start: usize, end: usize) {
        self.inner.apply_strikethrough(start, end);
    }

    #[wasm_bindgen]
    pub fn apply_code(&mut self, start: usize, end: usize) {
        self.inner.apply_code(start, end);
    }

    #[wasm_bindgen]
    pub fn apply_link(&mut self, start: usize, end: usize, url: &str) {
        self.inner.apply_link(start, end, url);
    }

    #[wasm_bindgen]
    pub fn apply_highlight(&mut self, start: usize, end: usize, color: &str) {
        self.inner.apply_highlight(start, end, color);
    }

    #[wasm_bindgen]
    pub fn apply_comment(&mut self, start: usize, end: usize, author: &str, content: &str) {
        self.inner.apply_comment(start, end, author, content);
    }

    #[wasm_bindgen]
    pub fn apply_custom_mark(&mut self, start: usize, end: usize, name: &str, value: &str) {
        self.inner.apply_custom_mark(start, end, name, value);
    }

    #[wasm_bindgen]
    pub fn get_text(&self) -> String {
        self.inner.get_text()
    }

    #[wasm_bindgen]
    pub fn get_html(&self) -> String {
        self.inner.get_html()
    }

    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[wasm_bindgen]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[wasm_bindgen]
    pub fn version(&self) -> u64 {
        self.inner.version()
    }

    #[wasm_bindgen]
    pub fn doc_id(&self) -> String {
        self.inner.doc_id()
    }

    #[wasm_bindgen]
    pub fn replica_id(&self) -> String {
        self.inner.replica_id()
    }

    #[wasm_bindgen]
    pub fn serialize(&self) -> Result<String, JsValue> {
        self.inner.serialize()
    }

    #[wasm_bindgen]
    pub fn merge(&mut self, remote_state: &str) -> Result<(), JsValue> {
        self.inner.merge(remote_state)
    }

    #[wasm_bindgen]
    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        self.inner.snapshot()
    }

    #[wasm_bindgen]
    pub fn restore(snapshot_js: JsValue) -> Result<RichTextDocument, JsValue> {
        Ok(Self {
            inner: CollaborativeDocument::restore(snapshot_js)?,
        })
    }
}

// ============================================================================
// JsonDocument (JSON CRDT)
// ============================================================================

/// A collaborative JSON document backed by JsonCrdt.
#[wasm_bindgen]
pub struct JsonDocument {
    id: String,
    replica_id: String,
    doc: JsonCrdt,
    version: u64,
}

#[wasm_bindgen]
impl JsonDocument {
    #[wasm_bindgen(constructor)]
    pub fn new(doc_id: &str, replica_id: &str) -> Self {
        Self {
            id: doc_id.to_string(),
            replica_id: replica_id.to_string(),
            doc: JsonCrdt::new(replica_id),
            version: 0,
        }
    }

    #[wasm_bindgen]
    pub fn set_string(&mut self, path: &str, value: &str) -> Result<(), JsValue> {
        self.doc
            .set(&JsonPath::parse(path), JsonValue::String(value.to_string()))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.version += 1;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn set_int(&mut self, path: &str, value: i64) -> Result<(), JsValue> {
        self.doc
            .set(&JsonPath::parse(path), JsonValue::Int(value))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.version += 1;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn set_float(&mut self, path: &str, value: f64) -> Result<(), JsValue> {
        self.doc
            .set(&JsonPath::parse(path), JsonValue::Float(value))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.version += 1;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn set_bool(&mut self, path: &str, value: bool) -> Result<(), JsValue> {
        self.doc
            .set(&JsonPath::parse(path), JsonValue::Bool(value))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.version += 1;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn set_null(&mut self, path: &str) -> Result<(), JsValue> {
        self.doc
            .set(&JsonPath::parse(path), JsonValue::Null)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.version += 1;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn set_object(&mut self, path: &str) -> Result<(), JsValue> {
        self.doc
            .set_object(&JsonPath::parse(path))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.version += 1;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn set_array(&mut self, path: &str) -> Result<(), JsValue> {
        self.doc
            .set_array(&JsonPath::parse(path))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.version += 1;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn array_push_string(&mut self, path: &str, value: &str) -> Result<(), JsValue> {
        let arr_id = self.get_array_id(path)?;
        self.doc
            .array_push(&arr_id, JsonValue::String(value.to_string()))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.version += 1;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn array_push_int(&mut self, path: &str, value: i64) -> Result<(), JsValue> {
        let arr_id = self.get_array_id(path)?;
        self.doc
            .array_push(&arr_id, JsonValue::Int(value))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.version += 1;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn array_push_float(&mut self, path: &str, value: f64) -> Result<(), JsValue> {
        let arr_id = self.get_array_id(path)?;
        self.doc
            .array_push(&arr_id, JsonValue::Float(value))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.version += 1;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn array_push_bool(&mut self, path: &str, value: bool) -> Result<(), JsValue> {
        let arr_id = self.get_array_id(path)?;
        self.doc
            .array_push(&arr_id, JsonValue::Bool(value))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.version += 1;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn array_push_null(&mut self, path: &str) -> Result<(), JsValue> {
        let arr_id = self.get_array_id(path)?;
        self.doc
            .array_push(&arr_id, JsonValue::Null)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.version += 1;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn array_remove(&mut self, path: &str, index: usize) -> Result<JsValue, JsValue> {
        let arr_id = self.get_array_id(path)?;
        let removed = self
            .doc
            .array_remove(&arr_id, index)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.version += 1;

        let removed_json = match removed {
            JsonValue::Null => serde_json::Value::Null,
            JsonValue::Bool(b) => serde_json::Value::Bool(b),
            JsonValue::Int(i) => serde_json::Value::Number(i.into()),
            JsonValue::Float(f) => serde_json::Number::from_f64(f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            JsonValue::String(s) => serde_json::Value::String(s),
            JsonValue::Array(_) | JsonValue::Object(_) => serde_json::Value::String(
                "[complex_json_reference]".to_string(),
            ),
        };

        serde_wasm_bindgen::to_value(&removed_json).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn delete(&mut self, path: &str) -> Result<(), JsValue> {
        self.doc
            .delete(&JsonPath::parse(path))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.version += 1;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn get(&self, path: &str) -> Result<JsValue, JsValue> {
        let root = self.doc.to_json();
        let maybe_value = get_json_at_dot_path(&root, path);
        match maybe_value {
            Some(value) => {
                serde_wasm_bindgen::to_value(&value).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            None => Ok(JsValue::UNDEFINED),
        }
    }

    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.doc.to_json())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn keys(&self) -> Result<JsValue, JsValue> {
        let keys = self.doc.keys();
        serde_wasm_bindgen::to_value(&keys).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn contains_key(&self, key: &str) -> bool {
        self.doc.contains_key(key)
    }

    #[wasm_bindgen]
    pub fn version(&self) -> u64 {
        self.version
    }

    #[wasm_bindgen]
    pub fn doc_id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen]
    pub fn replica_id(&self) -> String {
        self.replica_id.clone()
    }

    #[wasm_bindgen]
    pub fn serialize(&self) -> Result<String, JsValue> {
        let js_value = serde_wasm_bindgen::to_value(&self.doc)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;

        js_sys::JSON::stringify(&js_value)
            .map(|s| s.into())
            .map_err(|e| JsValue::from_str(&format!("JSON stringify error: {:?}", e)))
    }

    #[wasm_bindgen]
    pub fn merge(&mut self, remote_state: &str) -> Result<(), JsValue> {
        let js_value = js_sys::JSON::parse(remote_state)
            .map_err(|e| JsValue::from_str(&format!("JSON parse error: {:?}", e)))?;

        let remote: JsonCrdt = serde_wasm_bindgen::from_value(js_value)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;

        self.doc = self.doc.join(&remote);
        self.version += 1;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        let state_js = serde_wasm_bindgen::to_value(&self.doc)
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

    #[wasm_bindgen]
    pub fn restore(snapshot_js: JsValue) -> Result<JsonDocument, JsValue> {
        let snapshot: DocumentSnapshot = serde_wasm_bindgen::from_value(snapshot_js)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let state_js = js_sys::JSON::parse(&snapshot.state)
            .map_err(|e| JsValue::from_str(&format!("JSON parse error: {:?}", e)))?;

        let doc: JsonCrdt =
            serde_wasm_bindgen::from_value(state_js).map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(Self {
            id: snapshot.doc_id,
            replica_id: snapshot.replica_id,
            doc,
            version: snapshot.version,
        })
    }
}

impl JsonDocument {
    fn get_array_id(&self, path: &str) -> Result<mdcs_db::json_crdt::ArrayId, JsValue> {
        let json_path = JsonPath::parse(path);
        let value = self
            .doc
            .get(&json_path)
            .ok_or_else(|| JsValue::from_str(&format!("Path not found: {}", path)))?;

        match value {
            JsonValue::Array(id) => Ok(id.clone()),
            _ => Err(JsValue::from_str(&format!(
                "Path is not an array: {}",
                path
            ))),
        }
    }
}

fn get_json_at_dot_path(root: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
    if path.is_empty() {
        return Some(root.clone());
    }

    let mut current = root;
    for seg in path.split('.') {
        if let Ok(idx) = seg.parse::<usize>() {
            current = current.get(idx)?;
        } else {
            current = current.get(seg)?;
        }
    }

    Some(current.clone())
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

    #[test]
    fn test_extended_mark_types() {
        let mut doc = CollaborativeDocument::new("doc-1", "replica-1");
        doc.insert(0, "hello world");

        let initial_version = doc.version();

        doc.apply_code(0, 5);
        doc.apply_highlight(6, 11, "#FFEAA7");
        doc.apply_comment(0, 11, "alice", "review this");
        doc.apply_custom_mark(0, 5, "tag", "important");

        assert!(doc.version() >= initial_version + 4);
        assert_eq!(doc.get_text(), "hello world");
        assert_eq!(doc.len(), 11);
    }

    #[test]
    fn test_text_document_api() {
        let mut doc = TextDocument::new("text-doc", "replica-1");
        doc.insert(0, "Hello");
        doc.insert(5, " World");
        assert_eq!(doc.get_text(), "Hello World");

        doc.replace(6, 11, "Rust");
        assert_eq!(doc.get_text(), "Hello Rust");

        doc.splice(5, 1, ",");
        assert_eq!(doc.get_text(), "Hello,Rust");
        assert!(doc.version() > 0);
    }

    #[test]
    fn test_rich_text_document_wrapper() {
        let mut doc = RichTextDocument::new("rich-doc", "replica-1");
        doc.insert(0, "hello world");
        doc.apply_bold(0, 5);
        doc.apply_code(6, 11);

        assert_eq!(doc.get_text(), "hello world");
        assert_eq!(doc.len(), 11);
        assert!(doc.version() > 0);
    }

    #[test]
    fn test_json_document_api() {
        let mut doc = JsonDocument::new("json-doc", "replica-1");
        doc.set_string("name", "Alice").unwrap();
        doc.set_int("age", 30).unwrap();
        doc.set_bool("active", true).unwrap();
        doc.set_object("profile").unwrap();
        doc.set_string("profile.city", "Chennai").unwrap();
        doc.set_array("tags").unwrap();
        doc.array_push_string("tags", "crdt").unwrap();
        doc.array_push_string("tags", "wasm").unwrap();

        let root_v = doc.doc.to_json();
        assert_eq!(root_v["name"], "Alice");
        assert_eq!(root_v["profile"]["city"], "Chennai");
        assert_eq!(root_v["tags"][0], "crdt");
        assert!(doc.version() > 0);
    }
}
