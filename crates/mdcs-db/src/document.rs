//! Document Store - High-level API for managing CRDT documents.
//!
//! Provides a unified interface for:
//! - Creating and managing documents
//! - Path-based queries
//! - Document versioning and snapshots
//! - Prefix scans and queries

use crate::error::DbError;
use crate::json_crdt::{JsonCrdt, JsonCrdtDelta, JsonPath, JsonValue};
use crate::rga_text::{RGAText, RGATextDelta};
use crate::rich_text::{RichText, RichTextDelta};
use mdcs_core::lattice::Lattice;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use ulid::Ulid;

/// Unique identifier for a document.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocumentId(pub String);

impl DocumentId {
    pub fn new() -> Self {
        Self(Ulid::new().to_string())
    }

    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The type of a document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    /// Plain text document.
    Text,
    /// Rich text with formatting.
    RichText,
    /// JSON-like structured document.
    Json,
}

/// A CRDT value that can be stored in a document.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CrdtValue {
    /// Plain text.
    Text(RGAText),
    /// Rich text with formatting.
    RichText(RichText),
    /// Structured JSON data.
    Json(JsonCrdt),
}

impl CrdtValue {
    pub fn document_type(&self) -> DocumentType {
        match self {
            CrdtValue::Text(_) => DocumentType::Text,
            CrdtValue::RichText(_) => DocumentType::RichText,
            CrdtValue::Json(_) => DocumentType::Json,
        }
    }

    pub fn as_text(&self) -> Option<&RGAText> {
        match self {
            CrdtValue::Text(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_text_mut(&mut self) -> Option<&mut RGAText> {
        match self {
            CrdtValue::Text(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_rich_text(&self) -> Option<&RichText> {
        match self {
            CrdtValue::RichText(rt) => Some(rt),
            _ => None,
        }
    }

    pub fn as_rich_text_mut(&mut self) -> Option<&mut RichText> {
        match self {
            CrdtValue::RichText(rt) => Some(rt),
            _ => None,
        }
    }

    pub fn as_json(&self) -> Option<&JsonCrdt> {
        match self {
            CrdtValue::Json(j) => Some(j),
            _ => None,
        }
    }

    pub fn as_json_mut(&mut self) -> Option<&mut JsonCrdt> {
        match self {
            CrdtValue::Json(j) => Some(j),
            _ => None,
        }
    }
}

impl Lattice for CrdtValue {
    fn bottom() -> Self {
        CrdtValue::Json(JsonCrdt::bottom())
    }

    fn join(&self, other: &Self) -> Self {
        match (self, other) {
            (CrdtValue::Text(a), CrdtValue::Text(b)) => CrdtValue::Text(a.join(b)),
            (CrdtValue::RichText(a), CrdtValue::RichText(b)) => CrdtValue::RichText(a.join(b)),
            (CrdtValue::Json(a), CrdtValue::Json(b)) => CrdtValue::Json(a.join(b)),
            // Type mismatch - prefer self
            _ => self.clone(),
        }
    }
}

/// Delta for document changes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DocumentDelta {
    Text(RGATextDelta),
    RichText(RichTextDelta),
    Json(JsonCrdtDelta),
}

/// A document with metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    /// Document ID.
    pub id: DocumentId,
    /// Document title/name.
    pub title: String,
    /// The CRDT value.
    pub value: CrdtValue,
    /// Creation timestamp.
    pub created_at: u64,
    /// Last modified timestamp.
    pub modified_at: u64,
    /// Document metadata.
    pub metadata: HashMap<String, String>,
}

impl Document {
    /// Create a new text document.
    pub fn new_text(id: DocumentId, title: impl Into<String>, replica_id: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id,
            title: title.into(),
            value: CrdtValue::Text(RGAText::new(replica_id)),
            created_at: now,
            modified_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Create a new rich text document.
    pub fn new_rich_text(id: DocumentId, title: impl Into<String>, replica_id: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id,
            title: title.into(),
            value: CrdtValue::RichText(RichText::new(replica_id)),
            created_at: now,
            modified_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Create a new JSON document.
    pub fn new_json(id: DocumentId, title: impl Into<String>, replica_id: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id,
            title: title.into(),
            value: CrdtValue::Json(JsonCrdt::new(replica_id)),
            created_at: now,
            modified_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Get the document type.
    pub fn document_type(&self) -> DocumentType {
        self.value.document_type()
    }

    /// Touch the modified timestamp.
    pub fn touch(&mut self) {
        self.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }

    /// Set metadata.
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Get metadata.
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// Options for querying documents.
#[derive(Clone, Debug, Default)]
pub struct QueryOptions {
    /// Filter by document type.
    pub document_type: Option<DocumentType>,
    /// Filter by title prefix.
    pub title_prefix: Option<String>,
    /// Sort by field.
    pub sort_by: Option<SortField>,
    /// Sort direction.
    pub sort_desc: bool,
    /// Limit results.
    pub limit: Option<usize>,
    /// Skip results.
    pub offset: Option<usize>,
}

#[derive(Clone, Debug)]
pub enum SortField {
    Title,
    CreatedAt,
    ModifiedAt,
}

/// A document store for managing multiple CRDT documents.
#[derive(Clone, Debug)]
pub struct DocumentStore {
    /// The replica ID for this store.
    replica_id: String,
    /// All documents indexed by ID.
    documents: BTreeMap<DocumentId, Document>,
    /// Index by title for prefix queries.
    title_index: BTreeMap<String, DocumentId>,
    /// Pending changes for replication.
    pending_changes: Vec<StoreChange>,
}

/// A change to the store.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StoreChange {
    /// A new document was created.
    Create {
        id: DocumentId,
        doc_type: DocumentType,
        title: String,
    },
    /// A document was updated.
    Update {
        id: DocumentId,
        delta: DocumentDelta,
    },
    /// A document was deleted.
    Delete { id: DocumentId },
    /// Document metadata changed.
    MetadataChange {
        id: DocumentId,
        key: String,
        value: Option<String>,
    },
}

impl DocumentStore {
    /// Create a new document store.
    pub fn new(replica_id: impl Into<String>) -> Self {
        Self {
            replica_id: replica_id.into(),
            documents: BTreeMap::new(),
            title_index: BTreeMap::new(),
            pending_changes: Vec::new(),
        }
    }

    /// Get the replica ID.
    pub fn replica_id(&self) -> &str {
        &self.replica_id
    }

    // === Document CRUD ===

    /// Create a new text document.
    pub fn create_text(&mut self, title: impl Into<String>) -> DocumentId {
        let id = DocumentId::new();
        let title = title.into();
        let doc = Document::new_text(id.clone(), &title, &self.replica_id);

        self.title_index.insert(title.clone(), id.clone());
        self.documents.insert(id.clone(), doc);

        self.pending_changes.push(StoreChange::Create {
            id: id.clone(),
            doc_type: DocumentType::Text,
            title,
        });

        id
    }

    /// Create a new rich text document.
    pub fn create_rich_text(&mut self, title: impl Into<String>) -> DocumentId {
        let id = DocumentId::new();
        let title = title.into();
        let doc = Document::new_rich_text(id.clone(), &title, &self.replica_id);

        self.title_index.insert(title.clone(), id.clone());
        self.documents.insert(id.clone(), doc);

        self.pending_changes.push(StoreChange::Create {
            id: id.clone(),
            doc_type: DocumentType::RichText,
            title,
        });

        id
    }

    /// Create a new JSON document.
    pub fn create_json(&mut self, title: impl Into<String>) -> DocumentId {
        let id = DocumentId::new();
        let title = title.into();
        let doc = Document::new_json(id.clone(), &title, &self.replica_id);

        self.title_index.insert(title.clone(), id.clone());
        self.documents.insert(id.clone(), doc);

        self.pending_changes.push(StoreChange::Create {
            id: id.clone(),
            doc_type: DocumentType::Json,
            title,
        });

        id
    }

    /// Get a document by ID.
    pub fn get(&self, id: &DocumentId) -> Option<&Document> {
        self.documents.get(id)
    }

    /// Get a mutable document by ID.
    pub fn get_mut(&mut self, id: &DocumentId) -> Option<&mut Document> {
        self.documents.get_mut(id)
    }

    /// Delete a document.
    pub fn delete(&mut self, id: &DocumentId) -> Option<Document> {
        if let Some(doc) = self.documents.remove(id) {
            self.title_index.remove(&doc.title);
            self.pending_changes
                .push(StoreChange::Delete { id: id.clone() });
            Some(doc)
        } else {
            None
        }
    }

    /// Check if a document exists.
    pub fn contains(&self, id: &DocumentId) -> bool {
        self.documents.contains_key(id)
    }

    /// Get the number of documents.
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    // === Text Operations ===

    /// Insert text into a text document.
    pub fn text_insert(
        &mut self,
        id: &DocumentId,
        position: usize,
        text: &str,
    ) -> Result<(), DbError> {
        let doc = self
            .documents
            .get_mut(id)
            .ok_or_else(|| DbError::DocumentNotFound(id.to_string()))?;

        let doc_type = doc.value.document_type();
        let rga_text = doc.value.as_text_mut().ok_or(DbError::TypeMismatch {
            expected: "Text".to_string(),
            found: format!("{:?}", doc_type),
        })?;

        rga_text.insert(position, text);
        let delta = rga_text.take_delta();
        doc.touch();

        if let Some(delta) = delta {
            self.pending_changes.push(StoreChange::Update {
                id: id.clone(),
                delta: DocumentDelta::Text(delta),
            });
        }

        Ok(())
    }

    /// Delete text from a text document.
    pub fn text_delete(
        &mut self,
        id: &DocumentId,
        start: usize,
        length: usize,
    ) -> Result<(), DbError> {
        let doc = self
            .documents
            .get_mut(id)
            .ok_or_else(|| DbError::DocumentNotFound(id.to_string()))?;

        let doc_type = doc.value.document_type();
        let rga_text = doc.value.as_text_mut().ok_or(DbError::TypeMismatch {
            expected: "Text".to_string(),
            found: format!("{:?}", doc_type),
        })?;

        rga_text.delete(start, length);
        let delta = rga_text.take_delta();
        doc.touch();

        if let Some(delta) = delta {
            self.pending_changes.push(StoreChange::Update {
                id: id.clone(),
                delta: DocumentDelta::Text(delta),
            });
        }

        Ok(())
    }

    /// Get text content.
    pub fn text_content(&self, id: &DocumentId) -> Result<String, DbError> {
        let doc = self
            .documents
            .get(id)
            .ok_or_else(|| DbError::DocumentNotFound(id.to_string()))?;

        let rga_text = doc.value.as_text().ok_or(DbError::TypeMismatch {
            expected: "Text".to_string(),
            found: format!("{:?}", doc.value.document_type()),
        })?;

        Ok(rga_text.to_string())
    }

    // === Rich Text Operations ===

    /// Insert text into a rich text document.
    pub fn rich_text_insert(
        &mut self,
        id: &DocumentId,
        position: usize,
        text: &str,
    ) -> Result<(), DbError> {
        let doc = self
            .documents
            .get_mut(id)
            .ok_or_else(|| DbError::DocumentNotFound(id.to_string()))?;

        let doc_type = doc.value.document_type();
        let rich_text = doc.value.as_rich_text_mut().ok_or(DbError::TypeMismatch {
            expected: "RichText".to_string(),
            found: format!("{:?}", doc_type),
        })?;

        rich_text.insert(position, text);
        let delta = rich_text.take_delta();
        doc.touch();

        if let Some(delta) = delta {
            self.pending_changes.push(StoreChange::Update {
                id: id.clone(),
                delta: DocumentDelta::RichText(delta),
            });
        }

        Ok(())
    }

    /// Apply bold formatting.
    pub fn rich_text_bold(
        &mut self,
        id: &DocumentId,
        start: usize,
        end: usize,
    ) -> Result<(), DbError> {
        let doc = self
            .documents
            .get_mut(id)
            .ok_or_else(|| DbError::DocumentNotFound(id.to_string()))?;

        let doc_type = doc.value.document_type();
        let rich_text = doc.value.as_rich_text_mut().ok_or(DbError::TypeMismatch {
            expected: "RichText".to_string(),
            found: format!("{:?}", doc_type),
        })?;

        rich_text.bold(start, end);
        let delta = rich_text.take_delta();
        doc.touch();

        if let Some(delta) = delta {
            self.pending_changes.push(StoreChange::Update {
                id: id.clone(),
                delta: DocumentDelta::RichText(delta),
            });
        }

        Ok(())
    }

    /// Apply italic formatting.
    pub fn rich_text_italic(
        &mut self,
        id: &DocumentId,
        start: usize,
        end: usize,
    ) -> Result<(), DbError> {
        let doc = self
            .documents
            .get_mut(id)
            .ok_or_else(|| DbError::DocumentNotFound(id.to_string()))?;

        let doc_type = doc.value.document_type();
        let rich_text = doc.value.as_rich_text_mut().ok_or(DbError::TypeMismatch {
            expected: "RichText".to_string(),
            found: format!("{:?}", doc_type),
        })?;

        rich_text.italic(start, end);
        let delta = rich_text.take_delta();
        doc.touch();

        if let Some(delta) = delta {
            self.pending_changes.push(StoreChange::Update {
                id: id.clone(),
                delta: DocumentDelta::RichText(delta),
            });
        }

        Ok(())
    }

    /// Get rich text as HTML.
    pub fn rich_text_html(&self, id: &DocumentId) -> Result<String, DbError> {
        let doc = self
            .documents
            .get(id)
            .ok_or_else(|| DbError::DocumentNotFound(id.to_string()))?;

        let doc_type = doc.value.document_type();
        let rich_text = doc.value.as_rich_text().ok_or(DbError::TypeMismatch {
            expected: "RichText".to_string(),
            found: format!("{:?}", doc_type),
        })?;

        Ok(rich_text.to_html())
    }

    // === JSON Operations ===

    /// Set a value in a JSON document.
    pub fn json_set(
        &mut self,
        id: &DocumentId,
        path: &str,
        value: JsonValue,
    ) -> Result<(), DbError> {
        let doc = self
            .documents
            .get_mut(id)
            .ok_or_else(|| DbError::DocumentNotFound(id.to_string()))?;

        let doc_type = doc.value.document_type();
        let json = doc.value.as_json_mut().ok_or(DbError::TypeMismatch {
            expected: "Json".to_string(),
            found: format!("{:?}", doc_type),
        })?;

        json.set(&JsonPath::parse(path), value)?;
        let delta = json.take_delta();
        doc.touch();

        if let Some(delta) = delta {
            self.pending_changes.push(StoreChange::Update {
                id: id.clone(),
                delta: DocumentDelta::Json(delta),
            });
        }

        Ok(())
    }

    /// Get a value from a JSON document.
    pub fn json_get(&self, id: &DocumentId, path: &str) -> Result<Option<&JsonValue>, DbError> {
        let doc = self
            .documents
            .get(id)
            .ok_or_else(|| DbError::DocumentNotFound(id.to_string()))?;

        let doc_type = doc.value.document_type();
        let json = doc.value.as_json().ok_or(DbError::TypeMismatch {
            expected: "Json".to_string(),
            found: format!("{:?}", doc_type),
        })?;

        Ok(json.get(&JsonPath::parse(path)))
    }

    /// Get JSON document as serde_json::Value.
    pub fn json_to_value(&self, id: &DocumentId) -> Result<serde_json::Value, DbError> {
        let doc = self
            .documents
            .get(id)
            .ok_or_else(|| DbError::DocumentNotFound(id.to_string()))?;

        let json = doc.value.as_json().ok_or(DbError::TypeMismatch {
            expected: "Json".to_string(),
            found: format!("{:?}", doc.value.document_type()),
        })?;

        Ok(json.to_json())
    }

    // === Query Operations ===

    /// Find a document by title.
    pub fn find_by_title(&self, title: &str) -> Option<&Document> {
        self.title_index
            .get(title)
            .and_then(|id| self.documents.get(id))
    }

    /// List all documents.
    pub fn list(&self) -> Vec<&Document> {
        self.documents.values().collect()
    }

    /// Query documents with options.
    pub fn query(&self, options: &QueryOptions) -> Vec<&Document> {
        let mut results: Vec<_> = self
            .documents
            .values()
            .filter(|doc| {
                // Type filter
                if let Some(ref doc_type) = options.document_type {
                    if &doc.document_type() != doc_type {
                        return false;
                    }
                }
                // Title prefix filter
                if let Some(ref prefix) = options.title_prefix {
                    if !doc.title.starts_with(prefix) {
                        return false;
                    }
                }
                true
            })
            .collect();

        // Sort
        if let Some(ref sort_by) = options.sort_by {
            match sort_by {
                SortField::Title => {
                    results.sort_by(|a, b| a.title.cmp(&b.title));
                }
                SortField::CreatedAt => {
                    results.sort_by(|a, b| a.created_at.cmp(&b.created_at));
                }
                SortField::ModifiedAt => {
                    results.sort_by(|a, b| a.modified_at.cmp(&b.modified_at));
                }
            }
            if options.sort_desc {
                results.reverse();
            }
        }

        // Pagination
        if let Some(offset) = options.offset {
            results = results.into_iter().skip(offset).collect();
        }
        if let Some(limit) = options.limit {
            results.truncate(limit);
        }

        results
    }

    /// Prefix scan for titles.
    pub fn scan_prefix(&self, prefix: &str) -> Vec<&Document> {
        self.title_index
            .range(prefix.to_string()..)
            .take_while(|(k, _)| k.starts_with(prefix))
            .filter_map(|(_, id)| self.documents.get(id))
            .collect()
    }

    // === Replication ===

    /// Take pending changes for replication.
    pub fn take_changes(&mut self) -> Vec<StoreChange> {
        std::mem::take(&mut self.pending_changes)
    }

    /// Apply changes from another replica.
    pub fn apply_changes(&mut self, changes: &[StoreChange]) {
        for change in changes {
            match change {
                StoreChange::Create {
                    id,
                    doc_type,
                    title,
                } => {
                    if !self.documents.contains_key(id) {
                        let doc = match doc_type {
                            DocumentType::Text => {
                                Document::new_text(id.clone(), title, &self.replica_id)
                            }
                            DocumentType::RichText => {
                                Document::new_rich_text(id.clone(), title, &self.replica_id)
                            }
                            DocumentType::Json => {
                                Document::new_json(id.clone(), title, &self.replica_id)
                            }
                        };
                        self.title_index.insert(title.clone(), id.clone());
                        self.documents.insert(id.clone(), doc);
                    }
                }
                StoreChange::Update { id, delta } => {
                    if let Some(doc) = self.documents.get_mut(id) {
                        match (delta, &mut doc.value) {
                            (DocumentDelta::Text(d), CrdtValue::Text(t)) => {
                                t.apply_delta(d);
                            }
                            (DocumentDelta::RichText(d), CrdtValue::RichText(rt)) => {
                                rt.apply_delta(d);
                            }
                            (DocumentDelta::Json(d), CrdtValue::Json(j)) => {
                                j.apply_delta(d);
                            }
                            _ => {} // Type mismatch, ignore
                        }
                        doc.touch();
                    }
                }
                StoreChange::Delete { id } => {
                    if let Some(doc) = self.documents.remove(id) {
                        self.title_index.remove(&doc.title);
                    }
                }
                StoreChange::MetadataChange { id, key, value } => {
                    if let Some(doc) = self.documents.get_mut(id) {
                        match value {
                            Some(v) => {
                                doc.metadata.insert(key.clone(), v.clone());
                            }
                            None => {
                                doc.metadata.remove(key);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get all document IDs.
    pub fn document_ids(&self) -> impl Iterator<Item = &DocumentId> + '_ {
        self.documents.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_documents() {
        let mut store = DocumentStore::new("r1");

        let text_id = store.create_text("My Text");
        let rich_id = store.create_rich_text("My Rich Text");
        let json_id = store.create_json("My JSON");

        assert_eq!(store.len(), 3);
        assert!(store.contains(&text_id));
        assert!(store.contains(&rich_id));
        assert!(store.contains(&json_id));
    }

    #[test]
    fn test_text_operations() {
        let mut store = DocumentStore::new("r1");
        let id = store.create_text("Test");

        store.text_insert(&id, 0, "Hello").unwrap();
        store.text_insert(&id, 5, " World").unwrap();

        let content = store.text_content(&id).unwrap();
        assert_eq!(content, "Hello World");

        store.text_delete(&id, 5, 6).unwrap();
        let content = store.text_content(&id).unwrap();
        assert_eq!(content, "Hello");
    }

    #[test]
    fn test_json_operations() {
        let mut store = DocumentStore::new("r1");
        let id = store.create_json("Config");

        store
            .json_set(&id, "name", JsonValue::String("Test".to_string()))
            .unwrap();
        store.json_set(&id, "count", JsonValue::Int(42)).unwrap();

        let name = store.json_get(&id, "name").unwrap();
        assert_eq!(name.unwrap().as_str(), Some("Test"));

        let json = store.json_to_value(&id).unwrap();
        assert_eq!(json["name"], "Test");
        assert_eq!(json["count"], 42);
    }

    #[test]
    fn test_find_by_title() {
        let mut store = DocumentStore::new("r1");

        store.create_text("Document A");
        store.create_text("Document B");
        store.create_text("Other");

        let doc = store.find_by_title("Document A").unwrap();
        assert_eq!(doc.title, "Document A");

        assert!(store.find_by_title("Not Found").is_none());
    }

    #[test]
    fn test_query() {
        let mut store = DocumentStore::new("r1");

        store.create_text("Text 1");
        store.create_text("Text 2");
        store.create_json("Json 1");

        let options = QueryOptions {
            document_type: Some(DocumentType::Text),
            ..Default::default()
        };

        let results = store.query(&options);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_prefix_scan() {
        let mut store = DocumentStore::new("r1");

        store.create_text("project/doc1");
        store.create_text("project/doc2");
        store.create_text("other/doc1");

        let results = store.scan_prefix("project/");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_delete() {
        let mut store = DocumentStore::new("r1");

        let id = store.create_text("To Delete");
        assert!(store.contains(&id));

        store.delete(&id);
        assert!(!store.contains(&id));
    }

    #[test]
    fn test_replication() {
        let mut store1 = DocumentStore::new("r1");
        let mut store2 = DocumentStore::new("r2");

        // Create on store1
        let id = store1.create_text("Shared Doc");
        store1.text_insert(&id, 0, "Hello").unwrap();

        // Replicate to store2
        let changes = store1.take_changes();
        store2.apply_changes(&changes);

        // Verify
        assert!(store2.contains(&id));
        let content = store2.text_content(&id).unwrap();
        assert_eq!(content, "Hello");
    }

    #[test]
    fn test_metadata() {
        let mut store = DocumentStore::new("r1");
        let id = store.create_text("With Metadata");

        let doc = store.get_mut(&id).unwrap();
        doc.set_metadata("author", "Alice");
        doc.set_metadata("version", "1.0");

        let doc = store.get(&id).unwrap();
        assert_eq!(doc.get_metadata("author"), Some(&"Alice".to_string()));
        assert_eq!(doc.get_metadata("version"), Some(&"1.0".to_string()));
    }
}
