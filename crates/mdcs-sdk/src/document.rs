//! Document wrappers for collaborative editing.

use mdcs_db::{
    json_crdt::{JsonCrdt, JsonPath, JsonValue},
    rga_text::RGAText,
    rich_text::{MarkType, RichText},
};
use tokio::sync::broadcast;

/// Events emitted when a document changes.
#[derive(Clone, Debug)]
pub enum DocEvent {
    /// Text was inserted.
    Insert { position: usize, text: String },
    /// Text was deleted.
    Delete { position: usize, length: usize },
    /// Remote changes were applied.
    RemoteUpdate,
}

/// Trait for collaborative documents.
pub trait CollaborativeDoc {
    /// Get the document ID.
    fn id(&self) -> &str;
    
    /// Get the replica ID.
    fn replica_id(&self) -> &str;
    
    /// Subscribe to document events.
    fn subscribe(&self) -> broadcast::Receiver<DocEvent>;
    
    /// Take pending deltas for sync.
    fn take_pending_deltas(&mut self) -> Vec<Vec<u8>>;
    
    /// Apply a remote delta.
    fn apply_remote(&mut self, delta: &[u8]);
}

/// A collaborative plain text document.
pub struct TextDoc {
    id: String,
    replica_id: String,
    text: RGAText,
    event_tx: broadcast::Sender<DocEvent>,
    pending_deltas: Vec<Vec<u8>>,
}

impl TextDoc {
    /// Create a new text document.
    pub fn new(id: impl Into<String>, replica_id: impl Into<String>) -> Self {
        let replica_id = replica_id.into();
        let (event_tx, _) = broadcast::channel(100);
        
        Self {
            id: id.into(),
            replica_id: replica_id.clone(),
            text: RGAText::new(&replica_id),
            event_tx,
            pending_deltas: Vec::new(),
        }
    }
    
    /// Insert text at position.
    pub fn insert(&mut self, position: usize, text: &str) {
        self.text.insert(position, text);
        let _ = self.event_tx.send(DocEvent::Insert {
            position,
            text: text.to_string(),
        });
        // Note: In a real implementation, we'd serialize the delta here
    }
    
    /// Delete text at position.
    pub fn delete(&mut self, position: usize, length: usize) {
        self.text.delete(position, length);
        let _ = self.event_tx.send(DocEvent::Delete { position, length });
    }
    
    /// Get the current text content.
    pub fn get_text(&self) -> String {
        self.text.to_string()
    }
    
    /// Get the text length.
    pub fn len(&self) -> usize {
        self.text.len()
    }
    
    /// Check if the document is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

impl CollaborativeDoc for TextDoc {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn replica_id(&self) -> &str {
        &self.replica_id
    }
    
    fn subscribe(&self) -> broadcast::Receiver<DocEvent> {
        self.event_tx.subscribe()
    }
    
    fn take_pending_deltas(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.pending_deltas)
    }
    
    fn apply_remote(&mut self, _delta: &[u8]) {
        // In a real implementation, deserialize and apply delta
        let _ = self.event_tx.send(DocEvent::RemoteUpdate);
    }
}

/// A collaborative rich text document with formatting.
pub struct RichTextDoc {
    id: String,
    replica_id: String,
    text: RichText,
    event_tx: broadcast::Sender<DocEvent>,
    pending_deltas: Vec<Vec<u8>>,
}

impl RichTextDoc {
    /// Create a new rich text document.
    pub fn new(id: impl Into<String>, replica_id: impl Into<String>) -> Self {
        let replica_id = replica_id.into();
        let (event_tx, _) = broadcast::channel(100);
        
        Self {
            id: id.into(),
            replica_id: replica_id.clone(),
            text: RichText::new(&replica_id),
            event_tx,
            pending_deltas: Vec::new(),
        }
    }
    
    /// Insert text at position.
    pub fn insert(&mut self, position: usize, text: &str) {
        self.text.insert(position, text);
        let _ = self.event_tx.send(DocEvent::Insert {
            position,
            text: text.to_string(),
        });
    }
    
    /// Delete text at position.
    pub fn delete(&mut self, position: usize, length: usize) {
        self.text.delete(position, length);
        let _ = self.event_tx.send(DocEvent::Delete { position, length });
    }
    
    /// Apply formatting to a range.
    pub fn format(&mut self, start: usize, end: usize, mark: MarkType) {
        self.text.add_mark(start, end, mark);
    }
    
    /// Remove formatting by mark ID.
    pub fn unformat_by_id(&mut self, mark_id: &mdcs_db::rich_text::MarkId) {
        self.text.remove_mark(mark_id);
    }
    
    /// Get the plain text content.
    pub fn get_text(&self) -> String {
        self.text.to_string()
    }
    
    /// Get the plain text as spans with marks.
    /// Note: For full mark information, use the underlying RichText directly.
    pub fn get_content(&self) -> String {
        self.text.to_string()
    }
    
    /// Get the text length.
    pub fn len(&self) -> usize {
        self.text.len()
    }
    
    /// Check if the document is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

impl CollaborativeDoc for RichTextDoc {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn replica_id(&self) -> &str {
        &self.replica_id
    }
    
    fn subscribe(&self) -> broadcast::Receiver<DocEvent> {
        self.event_tx.subscribe()
    }
    
    fn take_pending_deltas(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.pending_deltas)
    }
    
    fn apply_remote(&mut self, _delta: &[u8]) {
        let _ = self.event_tx.send(DocEvent::RemoteUpdate);
    }
}

/// A collaborative JSON document.
pub struct JsonDoc {
    id: String,
    replica_id: String,
    doc: JsonCrdt,
    event_tx: broadcast::Sender<DocEvent>,
    pending_deltas: Vec<Vec<u8>>,
}

impl JsonDoc {
    /// Create a new JSON document.
    pub fn new(id: impl Into<String>, replica_id: impl Into<String>) -> Self {
        let replica_id = replica_id.into();
        let (event_tx, _) = broadcast::channel(100);
        
        Self {
            id: id.into(),
            replica_id: replica_id.clone(),
            doc: JsonCrdt::new(&replica_id),
            event_tx,
            pending_deltas: Vec::new(),
        }
    }
    
    /// Set a value at a path.
    pub fn set(&mut self, path: &str, value: JsonValue) {
        let json_path = JsonPath::parse(path);
        let _ = self.doc.set(&json_path, value);
    }
    
    /// Get a value at a path.
    pub fn get(&self, path: &str) -> Option<JsonValue> {
        let json_path = JsonPath::parse(path);
        self.doc.get(&json_path).cloned()
    }
    
    /// Delete a value at a path.
    pub fn delete(&mut self, path: &str) {
        let json_path = JsonPath::parse(path);
        let _ = self.doc.delete(&json_path);
    }
    
    /// Get the root value as a serde JSON Value.
    pub fn root(&self) -> serde_json::Value {
        self.doc.to_json()
    }
    
    /// Get keys at a path.
    pub fn keys(&self) -> Vec<String> {
        self.doc.keys()
    }
}

impl CollaborativeDoc for JsonDoc {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn replica_id(&self) -> &str {
        &self.replica_id
    }
    
    fn subscribe(&self) -> broadcast::Receiver<DocEvent> {
        self.event_tx.subscribe()
    }
    
    fn take_pending_deltas(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.pending_deltas)
    }
    
    fn apply_remote(&mut self, _delta: &[u8]) {
        let _ = self.event_tx.send(DocEvent::RemoteUpdate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_doc() {
        let mut doc = TextDoc::new("doc-1", "replica-1");
        doc.insert(0, "Hello");
        doc.insert(5, " World");
        
        assert_eq!(doc.get_text(), "Hello World");
        assert_eq!(doc.len(), 11);
    }

    #[test]
    fn test_rich_text_doc() {
        let mut doc = RichTextDoc::new("doc-1", "replica-1");
        doc.insert(0, "Hello World");
        doc.format(0, 5, MarkType::Bold);
        
        assert_eq!(doc.get_text(), "Hello World");
    }

    #[test]
    fn test_json_doc() {
        let mut doc = JsonDoc::new("doc-1", "replica-1");
        doc.set("name", JsonValue::String("Alice".to_string()));
        doc.set("age", JsonValue::Float(30.0));
        
        assert_eq!(
            doc.get("name"),
            Some(JsonValue::String("Alice".to_string()))
        );
    }
}
