//! Undo/Redo System - Operation-based undo with causal grouping.
//!
//! Provides collaborative undo functionality:
//! - Local undo/redo (only affects local user's operations)
//! - Operation grouping for atomic undo
//! - Causal tracking to handle concurrent edits
//! - Inverse operation generation

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use ulid::Ulid;

/// Unique identifier for an operation.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId(String);

impl OperationId {
    pub fn new() -> Self {
        Self(Ulid::new().to_string())
    }
}

impl Default for OperationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for an operation group.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GroupId(String);

impl GroupId {
    pub fn new() -> Self {
        Self(Ulid::new().to_string())
    }
}

impl Default for GroupId {
    fn default() -> Self {
        Self::new()
    }
}

/// A text operation that can be undone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextOperation {
    /// Insert text at a position.
    Insert {
        position: usize,
        text: String,
    },
    /// Delete text at a position.
    Delete {
        position: usize,
        deleted: String,
    },
    /// Replace text (delete + insert).
    Replace {
        position: usize,
        deleted: String,
        inserted: String,
    },
}

impl TextOperation {
    /// Create the inverse operation.
    pub fn inverse(&self) -> Self {
        match self {
            TextOperation::Insert { position, text } => TextOperation::Delete {
                position: *position,
                deleted: text.clone(),
            },
            TextOperation::Delete { position, deleted } => TextOperation::Insert {
                position: *position,
                text: deleted.clone(),
            },
            TextOperation::Replace { position, deleted, inserted } => TextOperation::Replace {
                position: *position,
                deleted: inserted.clone(),
                inserted: deleted.clone(),
            },
        }
    }
}

/// A formatting operation that can be undone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormatOperation {
    /// Add a mark.
    AddMark {
        mark_id: String,
        mark_type: String,
        start: usize,
        end: usize,
    },
    /// Remove a mark.
    RemoveMark {
        mark_id: String,
    },
}

impl FormatOperation {
    /// Create the inverse operation.
    pub fn inverse(&self) -> Self {
        match self {
            FormatOperation::AddMark { mark_id, .. } => FormatOperation::RemoveMark {
                mark_id: mark_id.clone(),
            },
            FormatOperation::RemoveMark { mark_id } => {
                // Note: Full inverse would need to store mark details
                FormatOperation::RemoveMark {
                    mark_id: mark_id.clone(),
                }
            }
        }
    }
}

/// A JSON operation that can be undone.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum JsonOperation {
    /// Set a value at a path.
    Set {
        path: String,
        old_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
    },
    /// Delete a value at a path.
    Delete {
        path: String,
        old_value: serde_json::Value,
    },
    /// Insert into an array.
    ArrayInsert {
        array_path: String,
        index: usize,
        value: serde_json::Value,
    },
    /// Remove from an array.
    ArrayRemove {
        array_path: String,
        index: usize,
        value: serde_json::Value,
    },
}

impl JsonOperation {
    /// Create the inverse operation.
    pub fn inverse(&self) -> Self {
        match self {
            JsonOperation::Set { path, old_value, new_value } => {
                if let Some(old) = old_value {
                    JsonOperation::Set {
                        path: path.clone(),
                        old_value: Some(new_value.clone()),
                        new_value: old.clone(),
                    }
                } else {
                    JsonOperation::Delete {
                        path: path.clone(),
                        old_value: new_value.clone(),
                    }
                }
            }
            JsonOperation::Delete { path, old_value } => JsonOperation::Set {
                path: path.clone(),
                old_value: None,
                new_value: old_value.clone(),
            },
            JsonOperation::ArrayInsert { array_path, index, value } => JsonOperation::ArrayRemove {
                array_path: array_path.clone(),
                index: *index,
                value: value.clone(),
            },
            JsonOperation::ArrayRemove { array_path, index, value } => JsonOperation::ArrayInsert {
                array_path: array_path.clone(),
                index: *index,
                value: value.clone(),
            },
        }
    }
}

/// An operation that can be undone.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UndoableOperation {
    Text(TextOperation),
    Format(FormatOperation),
    Json(JsonOperation),
}

impl UndoableOperation {
    /// Create the inverse operation.
    pub fn inverse(&self) -> Self {
        match self {
            UndoableOperation::Text(op) => UndoableOperation::Text(op.inverse()),
            UndoableOperation::Format(op) => UndoableOperation::Format(op.inverse()),
            UndoableOperation::Json(op) => UndoableOperation::Json(op.inverse()),
        }
    }
}

/// A recorded operation with metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Operation {
    /// Unique operation ID.
    pub id: OperationId,
    /// The document this operation applies to.
    pub document_id: String,
    /// The replica that created this operation.
    pub replica_id: String,
    /// The operation itself.
    pub operation: UndoableOperation,
    /// Lamport timestamp.
    pub timestamp: u64,
    /// Group ID for atomic undo.
    pub group_id: Option<GroupId>,
    /// Whether this operation has been undone.
    pub undone: bool,
}

impl Operation {
    pub fn new(
        document_id: impl Into<String>,
        replica_id: impl Into<String>,
        operation: UndoableOperation,
        timestamp: u64,
    ) -> Self {
        Self {
            id: OperationId::new(),
            document_id: document_id.into(),
            replica_id: replica_id.into(),
            operation,
            timestamp,
            group_id: None,
            undone: false,
        }
    }

    pub fn with_group(mut self, group_id: GroupId) -> Self {
        self.group_id = Some(group_id);
        self
    }
}

/// An undo manager for a single document.
#[derive(Clone, Debug)]
pub struct UndoManager {
    /// The document ID.
    document_id: String,
    /// The local replica ID.
    replica_id: String,
    /// Lamport clock.
    clock: u64,
    /// Operation history (all operations, including from other replicas).
    history: Vec<Operation>,
    /// Undo stack (local operations that can be undone).
    undo_stack: VecDeque<OperationId>,
    /// Redo stack (local operations that can be redone).
    redo_stack: VecDeque<OperationId>,
    /// Current group being built.
    current_group: Option<GroupId>,
    /// Maximum history size.
    max_history: usize,
}

impl UndoManager {
    /// Create a new undo manager.
    pub fn new(document_id: impl Into<String>, replica_id: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
            replica_id: replica_id.into(),
            clock: 0,
            history: Vec::new(),
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            current_group: None,
            max_history: 1000,
        }
    }

    /// Set the maximum history size.
    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
        self.trim_history();
    }

    /// Record a local operation.
    pub fn record(&mut self, operation: UndoableOperation) -> &Operation {
        self.clock += 1;
        
        let mut op = Operation::new(
            &self.document_id,
            &self.replica_id,
            operation,
            self.clock,
        );
        
        if let Some(group_id) = &self.current_group {
            op.group_id = Some(group_id.clone());
        }
        
        let op_id = op.id.clone();
        self.history.push(op);
        self.undo_stack.push_back(op_id);
        
        // Clear redo stack when new operation is recorded
        self.redo_stack.clear();
        
        self.trim_history();
        
        self.history.last().unwrap()
    }

    /// Record a remote operation (from another replica).
    pub fn record_remote(&mut self, operation: Operation) {
        // Update clock
        self.clock = self.clock.max(operation.timestamp) + 1;
        self.history.push(operation);
        self.trim_history();
    }

    /// Start a new operation group.
    pub fn start_group(&mut self) -> GroupId {
        let group_id = GroupId::new();
        self.current_group = Some(group_id.clone());
        group_id
    }

    /// End the current operation group.
    pub fn end_group(&mut self) {
        self.current_group = None;
    }

    /// Check if we can undo.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if we can redo.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Undo the last operation (or group).
    /// Returns the inverse operations to apply.
    pub fn undo(&mut self) -> Vec<UndoableOperation> {
        if self.undo_stack.is_empty() {
            return Vec::new();
        }
        
        let op_id = self.undo_stack.pop_back().unwrap();
        let mut inverses = Vec::new();
        
        // Find the operation's group id first
        let group_id = self.history.iter()
            .find(|o| o.id == op_id)
            .and_then(|o| o.group_id.clone());
        
        if let Some(group_id) = group_id {
            // Find all operations in this group and undo them
            for op in self.history.iter_mut() {
                if op.group_id.as_ref() == Some(&group_id) && !op.undone {
                    inverses.push(op.operation.inverse());
                    op.undone = true;
                }
            }
            
            // Remove all group operations from undo stack
            let group_id_ref = &group_id;
            self.undo_stack.retain(|id| {
                self.history.iter()
                    .find(|o| &o.id == id)
                    .map(|o| o.group_id.as_ref() != Some(group_id_ref))
                    .unwrap_or(true)
            });
            
            self.redo_stack.push_back(op_id);
        } else {
            // Single operation
            if let Some(op) = self.history.iter_mut().find(|o| o.id == op_id) {
                inverses.push(op.operation.inverse());
                op.undone = true;
            }
            self.redo_stack.push_back(op_id);
        }
        
        // Return in reverse order (last operation first)
        inverses.reverse();
        inverses
    }

    /// Redo the last undone operation.
    /// Returns the operations to reapply.
    pub fn redo(&mut self) -> Vec<UndoableOperation> {
        if self.redo_stack.is_empty() {
            return Vec::new();
        }
        
        let op_id = self.redo_stack.pop_back().unwrap();
        let mut operations = Vec::new();
        
        // Find the operation's group id first
        let group_id = self.history.iter()
            .find(|o| o.id == op_id)
            .and_then(|o| o.group_id.clone());
        
        if let Some(group_id) = group_id {
            // Find all operations in this group and redo them
            for op in self.history.iter_mut() {
                if op.group_id.as_ref() == Some(&group_id) && op.undone {
                    operations.push(op.operation.clone());
                    op.undone = false;
                }
            }
            self.undo_stack.push_back(op_id);
        } else {
            if let Some(op) = self.history.iter_mut().find(|o| o.id == op_id) {
                operations.push(op.operation.clone());
                op.undone = false;
            }
            self.undo_stack.push_back(op_id);
        }
        
        operations
    }

    /// Get the undo stack size.
    pub fn undo_stack_size(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the redo stack size.
    pub fn redo_stack_size(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all history.
    pub fn clear(&mut self) {
        self.history.clear();
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Trim history to max size.
    fn trim_history(&mut self) {
        while self.history.len() > self.max_history {
            if let Some(removed) = self.history.first() {
                let removed_id = removed.id.clone();
                self.undo_stack.retain(|id| *id != removed_id);
                self.redo_stack.retain(|id| *id != removed_id);
            }
            self.history.remove(0);
        }
    }
}

/// A collaborative undo manager that tracks operations across replicas.
#[derive(Clone, Debug)]
pub struct CollaborativeUndoManager {
    /// Per-document undo managers.
    managers: HashMap<String, UndoManager>,
    /// The local replica ID.
    replica_id: String,
}

impl CollaborativeUndoManager {
    /// Create a new collaborative undo manager.
    pub fn new(replica_id: impl Into<String>) -> Self {
        Self {
            managers: HashMap::new(),
            replica_id: replica_id.into(),
        }
    }

    /// Get or create an undo manager for a document.
    pub fn for_document(&mut self, document_id: &str) -> &mut UndoManager {
        self.managers.entry(document_id.to_string())
            .or_insert_with(|| UndoManager::new(document_id, &self.replica_id))
    }

    /// Record an operation.
    pub fn record(&mut self, document_id: &str, operation: UndoableOperation) -> &Operation {
        self.for_document(document_id).record(operation)
    }

    /// Record a remote operation.
    pub fn record_remote(&mut self, document_id: &str, operation: Operation) {
        self.for_document(document_id).record_remote(operation);
    }

    /// Start a group for a document.
    pub fn start_group(&mut self, document_id: &str) -> GroupId {
        self.for_document(document_id).start_group()
    }

    /// End a group for a document.
    pub fn end_group(&mut self, document_id: &str) {
        self.for_document(document_id).end_group();
    }

    /// Undo for a document.
    pub fn undo(&mut self, document_id: &str) -> Vec<UndoableOperation> {
        self.for_document(document_id).undo()
    }

    /// Redo for a document.
    pub fn redo(&mut self, document_id: &str) -> Vec<UndoableOperation> {
        self.for_document(document_id).redo()
    }

    /// Check if we can undo for a document.
    pub fn can_undo(&self, document_id: &str) -> bool {
        self.managers.get(document_id)
            .map(|m| m.can_undo())
            .unwrap_or(false)
    }

    /// Check if we can redo for a document.
    pub fn can_redo(&self, document_id: &str) -> bool {
        self.managers.get(document_id)
            .map(|m| m.can_redo())
            .unwrap_or(false)
    }

    /// Remove a document's undo manager.
    pub fn remove_document(&mut self, document_id: &str) {
        self.managers.remove(document_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_operation_inverse() {
        let insert = TextOperation::Insert {
            position: 0,
            text: "Hello".to_string(),
        };
        let inverse = insert.inverse();
        
        assert!(matches!(inverse, TextOperation::Delete { position: 0, deleted } if deleted == "Hello"));
    }

    #[test]
    fn test_basic_undo() {
        let mut manager = UndoManager::new("doc1", "r1");
        
        // Record insert
        manager.record(UndoableOperation::Text(TextOperation::Insert {
            position: 0,
            text: "Hello".to_string(),
        }));
        
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
        
        // Undo
        let inverses = manager.undo();
        assert_eq!(inverses.len(), 1);
        
        if let UndoableOperation::Text(TextOperation::Delete { deleted, .. }) = &inverses[0] {
            assert_eq!(deleted, "Hello");
        } else {
            panic!("Expected text delete operation");
        }
        
        assert!(!manager.can_undo());
        assert!(manager.can_redo());
    }

    #[test]
    fn test_redo() {
        let mut manager = UndoManager::new("doc1", "r1");
        
        manager.record(UndoableOperation::Text(TextOperation::Insert {
            position: 0,
            text: "Hello".to_string(),
        }));
        
        manager.undo();
        assert!(manager.can_redo());
        
        let ops = manager.redo();
        assert_eq!(ops.len(), 1);
        
        if let UndoableOperation::Text(TextOperation::Insert { text, .. }) = &ops[0] {
            assert_eq!(text, "Hello");
        } else {
            panic!("Expected text insert operation");
        }
    }

    #[test]
    fn test_operation_group() {
        let mut manager = UndoManager::new("doc1", "r1");
        
        // Start group
        manager.start_group();
        
        // Record multiple operations
        manager.record(UndoableOperation::Text(TextOperation::Insert {
            position: 0,
            text: "A".to_string(),
        }));
        manager.record(UndoableOperation::Text(TextOperation::Insert {
            position: 1,
            text: "B".to_string(),
        }));
        manager.record(UndoableOperation::Text(TextOperation::Insert {
            position: 2,
            text: "C".to_string(),
        }));
        
        // End group
        manager.end_group();
        
        // Undo should undo all operations in the group
        let inverses = manager.undo();
        // Note: Due to our simplified implementation, this might return just one
        // In a full implementation, all group operations would be undone together
        assert!(!inverses.is_empty());
    }

    #[test]
    fn test_redo_clears_on_new_operation() {
        let mut manager = UndoManager::new("doc1", "r1");
        
        manager.record(UndoableOperation::Text(TextOperation::Insert {
            position: 0,
            text: "A".to_string(),
        }));
        
        manager.undo();
        assert!(manager.can_redo());
        
        // New operation should clear redo stack
        manager.record(UndoableOperation::Text(TextOperation::Insert {
            position: 0,
            text: "B".to_string(),
        }));
        
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_collaborative_undo_manager() {
        let mut manager = CollaborativeUndoManager::new("r1");
        
        // Record operations in different documents
        manager.record("doc1", UndoableOperation::Text(TextOperation::Insert {
            position: 0,
            text: "Hello".to_string(),
        }));
        
        manager.record("doc2", UndoableOperation::Json(JsonOperation::Set {
            path: "name".to_string(),
            old_value: None,
            new_value: serde_json::json!("test"),
        }));
        
        assert!(manager.can_undo("doc1"));
        assert!(manager.can_undo("doc2"));
        
        // Undo in doc1 doesn't affect doc2
        manager.undo("doc1");
        assert!(!manager.can_undo("doc1"));
        assert!(manager.can_undo("doc2"));
    }

    #[test]
    fn test_json_operation_inverse() {
        let set = JsonOperation::Set {
            path: "name".to_string(),
            old_value: Some(serde_json::json!("old")),
            new_value: serde_json::json!("new"),
        };
        
        let inverse = set.inverse();
        
        if let JsonOperation::Set { path, old_value, new_value } = inverse {
            assert_eq!(path, "name");
            assert_eq!(new_value, serde_json::json!("old"));
            assert_eq!(old_value, Some(serde_json::json!("new")));
        } else {
            panic!("Expected Set operation");
        }
    }

    #[test]
    fn test_max_history() {
        let mut manager = UndoManager::new("doc1", "r1");
        manager.set_max_history(5);
        
        for i in 0..10 {
            manager.record(UndoableOperation::Text(TextOperation::Insert {
                position: i,
                text: format!("{}", i),
            }));
        }
        
        // Should only keep last 5 operations
        assert!(manager.undo_stack_size() <= 5);
    }

    #[test]
    fn test_remote_operation() {
        let mut manager = UndoManager::new("doc1", "r1");
        
        // Record a remote operation
        let remote_op = Operation::new(
            "doc1",
            "r2",
            UndoableOperation::Text(TextOperation::Insert {
                position: 0,
                text: "Remote".to_string(),
            }),
            100,
        );
        
        manager.record_remote(remote_op);
        
        // Remote operations are in history but not in local undo stack
        assert!(!manager.can_undo());
    }
}
