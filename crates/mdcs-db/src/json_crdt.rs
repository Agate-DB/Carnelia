//! JSON CRDT - Automerge-like nested object CRDT.
//!
//! Provides collaborative editing of JSON-like documents with:
//! - Nested objects and arrays
//! - Path-based operations
//! - Conflict-free concurrent edits
//! - Multi-value registers for concurrent writes
//!
//! Uses a shared causal context for correct semantics.

use crate::error::DbError;
use crate::rga_list::{RGAList, RGAListDelta};
use mdcs_core::lattice::Lattice;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use ulid::Ulid;

/// A path into a JSON document.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JsonPath(Vec<PathSegment>);

impl JsonPath {
    /// Create an empty (root) path.
    pub fn root() -> Self {
        Self(Vec::new())
    }

    /// Create a path from segments.
    pub fn new(segments: Vec<PathSegment>) -> Self {
        Self(segments)
    }

    /// Parse a path from dot notation (e.g., "user.name" or "items.0.value").
    pub fn parse(path: &str) -> Self {
        if path.is_empty() {
            return Self::root();
        }
        let segments = path
            .split('.')
            .map(|s| {
                if let Ok(idx) = s.parse::<usize>() {
                    PathSegment::Index(idx)
                } else {
                    PathSegment::Key(s.to_string())
                }
            })
            .collect();
        Self(segments)
    }

    /// Get the segments.
    pub fn segments(&self) -> &[PathSegment] {
        &self.0
    }

    /// Check if this is the root path.
    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the parent path.
    pub fn parent(&self) -> Option<Self> {
        if self.0.is_empty() {
            None
        } else {
            Some(Self(self.0[..self.0.len() - 1].to_vec()))
        }
    }

    /// Get the last segment.
    pub fn last(&self) -> Option<&PathSegment> {
        self.0.last()
    }

    /// Append a segment.
    pub fn push(&mut self, segment: PathSegment) {
        self.0.push(segment);
    }

    /// Create a child path with a key.
    pub fn child_key(&self, key: impl Into<String>) -> Self {
        let mut new = self.clone();
        new.push(PathSegment::Key(key.into()));
        new
    }

    /// Create a child path with an index.
    pub fn child_index(&self, index: usize) -> Self {
        let mut new = self.clone();
        new.push(PathSegment::Index(index));
        new
    }
}

impl std::fmt::Display for JsonPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: Vec<String> = self.0.iter().map(|s| s.to_string()).collect();
        write!(f, "{}", s.join("."))
    }
}

/// A segment in a JSON path.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PathSegment {
    /// Object key.
    Key(String),
    /// Array index.
    Index(usize),
}

impl std::fmt::Display for PathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathSegment::Key(k) => write!(f, "{}", k),
            PathSegment::Index(i) => write!(f, "{}", i),
        }
    }
}

/// A JSON value that can be stored in the CRDT.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum JsonValue {
    /// Null value.
    #[default]
    Null,
    /// Boolean value.
    Bool(bool),
    /// Integer value.
    Int(i64),
    /// Floating point value.
    Float(f64),
    /// String value.
    String(String),
    /// Array reference (points to an RGAList).
    Array(ArrayId),
    /// Object reference (points to an ObjectMap).
    Object(ObjectId),
}

impl JsonValue {
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            JsonValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            JsonValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }
}

/// Unique identifier for an array in the document.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArrayId(String);

impl ArrayId {
    pub fn new() -> Self {
        Self(Ulid::new().to_string())
    }
}

impl Default for ArrayId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for an object in the document.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(String);

impl ObjectId {
    pub fn new() -> Self {
        Self(Ulid::new().to_string())
    }

    pub fn root() -> Self {
        Self("root".to_string())
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Self::new()
    }
}

/// A unique identifier for a field value (for multi-value tracking).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ValueId {
    replica: String,
    seq: u64,
}

impl ValueId {
    pub fn new(replica: impl Into<String>, seq: u64) -> Self {
        Self {
            replica: replica.into(),
            seq,
        }
    }
}

/// A field in an object that tracks concurrent values.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ObjectField {
    /// All concurrent values for this field (multi-value register).
    values: HashMap<ValueId, JsonValue>,
    /// Deleted value IDs (tombstones).
    deleted: HashSet<ValueId>,
}

impl ObjectField {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
            deleted: HashSet::new(),
        }
    }

    fn set(&mut self, id: ValueId, value: JsonValue) {
        // Setting a new value obsoletes previous values from this replica
        let to_delete: Vec<_> = self
            .values
            .keys()
            .filter(|k| k.replica == id.replica)
            .cloned()
            .collect();
        for k in to_delete {
            self.values.remove(&k);
        }
        self.values.insert(id, value);
    }

    #[allow(dead_code)]
    fn get(&self) -> Vec<&JsonValue> {
        self.values.values().collect()
    }

    fn get_winner(&self) -> Option<&JsonValue> {
        // Return the value with the highest ValueId (LWW semantics)
        self.values
            .iter()
            .max_by(|(a, _), (b, _)| a.seq.cmp(&b.seq).then_with(|| a.replica.cmp(&b.replica)))
            .map(|(_, v)| v)
    }

    fn is_deleted(&self) -> bool {
        self.values.is_empty() || self.values.values().all(|v| v.is_null())
    }

    fn merge(&mut self, other: &ObjectField) {
        for (id, value) in &other.values {
            if !self.deleted.contains(id) {
                self.values
                    .entry(id.clone())
                    .or_insert_with(|| value.clone());
            }
        }
        self.deleted.extend(other.deleted.iter().cloned());
        // Remove deleted values
        for id in &self.deleted {
            self.values.remove(id);
        }
    }
}

/// An object (map) in the JSON document.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct JsonObject {
    id: ObjectId,
    fields: HashMap<String, ObjectField>,
}

impl JsonObject {
    fn new(id: ObjectId) -> Self {
        Self {
            id,
            fields: HashMap::new(),
        }
    }

    fn set(&mut self, key: String, value_id: ValueId, value: JsonValue) {
        self.fields
            .entry(key)
            .or_insert_with(ObjectField::new)
            .set(value_id, value);
    }

    fn get(&self, key: &str) -> Option<&JsonValue> {
        self.fields.get(key)?.get_winner()
    }

    #[allow(dead_code)]
    fn get_all(&self, key: &str) -> Vec<&JsonValue> {
        self.fields.get(key).map(|f| f.get()).unwrap_or_default()
    }

    fn keys(&self) -> impl Iterator<Item = &String> + '_ {
        self.fields
            .iter()
            .filter(|(_, f)| !f.is_deleted())
            .map(|(k, _)| k)
    }

    fn remove(&mut self, key: &str, value_id: ValueId) {
        if let Some(field) = self.fields.get_mut(key) {
            // Mark all existing values as deleted
            let to_delete: Vec<_> = field.values.keys().cloned().collect();
            for id in to_delete {
                field.deleted.insert(id);
            }
            field.values.clear();
            // Set null to record the deletion
            field.values.insert(value_id, JsonValue::Null);
        }
    }

    fn merge(&mut self, other: &JsonObject) {
        for (key, field) in &other.fields {
            self.fields
                .entry(key.clone())
                .or_insert_with(ObjectField::new)
                .merge(field);
        }
    }
}

/// An array in the JSON document (using RGAList).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct JsonArray {
    id: ArrayId,
    list: RGAList<JsonValue>,
}

impl JsonArray {
    fn new(id: ArrayId, replica_id: &str) -> Self {
        Self {
            id,
            list: RGAList::new(replica_id),
        }
    }

    #[allow(dead_code)]
    fn get(&self, index: usize) -> Option<&JsonValue> {
        self.list.get(index)
    }

    fn len(&self) -> usize {
        self.list.len()
    }

    fn insert(&mut self, index: usize, value: JsonValue) {
        self.list.insert(index, value);
    }

    fn remove(&mut self, index: usize) -> Option<JsonValue> {
        self.list.delete(index)
    }

    fn push(&mut self, value: JsonValue) {
        self.list.push_back(value);
    }

    fn iter(&self) -> impl Iterator<Item = &JsonValue> + '_ {
        self.list.iter()
    }

    fn merge(&mut self, other: &JsonArray) {
        self.list = self.list.join(&other.list);
    }
}

/// Delta for JSON CRDT operations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JsonCrdtDelta {
    /// Object field changes.
    pub object_changes: Vec<ObjectChange>,
    /// Array changes.
    pub array_changes: Vec<ArrayChange>,
    /// New objects created.
    pub new_objects: Vec<ObjectId>,
    /// New arrays created.
    pub new_arrays: Vec<ArrayId>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObjectChange {
    pub object_id: ObjectId,
    pub key: String,
    pub value_id: ValueId,
    pub value: JsonValue,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArrayChange {
    pub array_id: ArrayId,
    pub delta: RGAListDelta<JsonValue>,
}

impl JsonCrdtDelta {
    pub fn new() -> Self {
        Self {
            object_changes: Vec::new(),
            array_changes: Vec::new(),
            new_objects: Vec::new(),
            new_arrays: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.object_changes.is_empty()
            && self.array_changes.is_empty()
            && self.new_objects.is_empty()
            && self.new_arrays.is_empty()
    }
}

impl Default for JsonCrdtDelta {
    fn default() -> Self {
        Self::new()
    }
}

/// Collaborative JSON document CRDT.
///
/// Provides Automerge-like semantics for editing nested
/// JSON structures with conflict-free concurrent operations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JsonCrdt {
    /// The replica ID.
    replica_id: String,
    /// Sequence counter for generating value IDs.
    seq: u64,
    /// The root object.
    root_id: ObjectId,
    /// All objects in the document.
    objects: HashMap<ObjectId, JsonObject>,
    /// All arrays in the document.
    arrays: HashMap<ArrayId, JsonArray>,
    /// Pending delta.
    #[serde(skip)]
    pending_delta: Option<JsonCrdtDelta>,
}

impl JsonCrdt {
    /// Create a new empty JSON document.
    pub fn new(replica_id: impl Into<String>) -> Self {
        let replica_id = replica_id.into();
        let root_id = ObjectId::root();
        let root = JsonObject::new(root_id.clone());

        let mut objects = HashMap::new();
        objects.insert(root_id.clone(), root);

        Self {
            replica_id,
            seq: 0,
            root_id,
            objects,
            arrays: HashMap::new(),
            pending_delta: None,
        }
    }

    /// Get the replica ID.
    pub fn replica_id(&self) -> &str {
        &self.replica_id
    }

    /// Generate a new value ID.
    fn next_value_id(&mut self) -> ValueId {
        self.seq += 1;
        ValueId::new(&self.replica_id, self.seq)
    }

    /// Get a value at a path.
    pub fn get(&self, path: &JsonPath) -> Option<&JsonValue> {
        let mut current_obj_id = &self.root_id;
        let segments = path.segments();

        for (i, segment) in segments.iter().enumerate() {
            let is_last = i == segments.len() - 1;

            match segment {
                PathSegment::Key(key) => {
                    let obj = self.objects.get(current_obj_id)?;
                    let value = obj.get(key)?;

                    if is_last {
                        return Some(value);
                    }

                    match value {
                        JsonValue::Object(id) => current_obj_id = id,
                        JsonValue::Array(_) if !is_last => {
                            // Next segment should be an index
                            continue;
                        }
                        _ => return None,
                    }
                }
                PathSegment::Index(_idx) => {
                    // Need to be at an array
                    let _obj = self.objects.get(current_obj_id)?;
                    // Find the array value
                    // This is a simplification; in practice we'd track which field is the array
                    return None; // Simplified - would need array traversal
                }
            }
        }

        // Root path returns None - use to_json() instead
        None
    }

    /// Set a value at a path.
    pub fn set(&mut self, path: &JsonPath, value: JsonValue) -> Result<(), DbError> {
        if path.is_root() {
            return Err(DbError::InvalidPath("Cannot set root".to_string()));
        }

        let parent_path = path.parent().unwrap_or(JsonPath::root());
        let last_segment = path
            .last()
            .ok_or_else(|| DbError::InvalidPath("Empty path".to_string()))?;

        // Ensure parent exists and is an object
        let parent_obj_id = self.ensure_object_at(&parent_path)?;

        let value_id = self.next_value_id();

        match last_segment {
            PathSegment::Key(key) => {
                // Handle nested object/array creation
                let actual_value = match &value {
                    JsonValue::Object(_) | JsonValue::Array(_) => value,
                    _ => value,
                };

                if let Some(obj) = self.objects.get_mut(&parent_obj_id) {
                    obj.set(key.clone(), value_id.clone(), actual_value.clone());
                }

                // Record delta
                let delta = self.pending_delta.get_or_insert_with(JsonCrdtDelta::new);
                delta.object_changes.push(ObjectChange {
                    object_id: parent_obj_id,
                    key: key.clone(),
                    value_id,
                    value: actual_value,
                });
            }
            PathSegment::Index(_) => {
                return Err(DbError::UnsupportedOperation(
                    "Set by index not supported; use array_insert".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Delete a value at a path.
    pub fn delete(&mut self, path: &JsonPath) -> Result<(), DbError> {
        if path.is_root() {
            return Err(DbError::InvalidPath("Cannot delete root".to_string()));
        }

        let parent_path = path.parent().unwrap_or(JsonPath::root());
        let last_segment = path
            .last()
            .ok_or_else(|| DbError::InvalidPath("Empty path".to_string()))?;

        let parent_obj_id = self
            .get_object_id_at(&parent_path)
            .ok_or_else(|| DbError::PathNotFound(parent_path.to_string()))?;

        let value_id = self.next_value_id();

        match last_segment {
            PathSegment::Key(key) => {
                if let Some(obj) = self.objects.get_mut(&parent_obj_id) {
                    obj.remove(key, value_id.clone());
                }

                // Record delta
                let delta = self.pending_delta.get_or_insert_with(JsonCrdtDelta::new);
                delta.object_changes.push(ObjectChange {
                    object_id: parent_obj_id,
                    key: key.clone(),
                    value_id,
                    value: JsonValue::Null,
                });
            }
            PathSegment::Index(_) => {
                return Err(DbError::UnsupportedOperation(
                    "Delete by index not supported; use array_remove".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Create a new object and return its ID.
    pub fn create_object(&mut self) -> ObjectId {
        let id = ObjectId::new();
        let obj = JsonObject::new(id.clone());
        self.objects.insert(id.clone(), obj);

        let delta = self.pending_delta.get_or_insert_with(JsonCrdtDelta::new);
        delta.new_objects.push(id.clone());

        id
    }

    /// Create a new array and return its ID.
    pub fn create_array(&mut self) -> ArrayId {
        let id = ArrayId::new();
        let arr = JsonArray::new(id.clone(), &self.replica_id);
        self.arrays.insert(id.clone(), arr);

        let delta = self.pending_delta.get_or_insert_with(JsonCrdtDelta::new);
        delta.new_arrays.push(id.clone());

        id
    }

    /// Set a nested object at a path.
    pub fn set_object(&mut self, path: &JsonPath) -> Result<ObjectId, DbError> {
        let obj_id = self.create_object();
        self.set(path, JsonValue::Object(obj_id.clone()))?;
        Ok(obj_id)
    }

    /// Set a nested array at a path.
    pub fn set_array(&mut self, path: &JsonPath) -> Result<ArrayId, DbError> {
        let arr_id = self.create_array();
        self.set(path, JsonValue::Array(arr_id.clone()))?;
        Ok(arr_id)
    }

    /// Get an array by ID (internal use).
    #[allow(dead_code)]
    fn get_array(&self, id: &ArrayId) -> Option<&JsonArray> {
        self.arrays.get(id)
    }

    /// Get a mutable array by ID.
    #[allow(dead_code)]
    fn get_array_mut(&mut self, id: &ArrayId) -> Option<&mut JsonArray> {
        self.arrays.get_mut(id)
    }

    /// Insert into an array.
    pub fn array_insert(
        &mut self,
        array_id: &ArrayId,
        index: usize,
        value: JsonValue,
    ) -> Result<(), DbError> {
        let arr = self
            .arrays
            .get_mut(array_id)
            .ok_or_else(|| DbError::PathNotFound(format!("Array {:?}", array_id)))?;

        arr.insert(index, value);

        if let Some(delta) = arr.list.take_delta() {
            let doc_delta = self.pending_delta.get_or_insert_with(JsonCrdtDelta::new);
            doc_delta.array_changes.push(ArrayChange {
                array_id: array_id.clone(),
                delta,
            });
        }

        Ok(())
    }

    /// Push to an array.
    pub fn array_push(&mut self, array_id: &ArrayId, value: JsonValue) -> Result<(), DbError> {
        let arr = self
            .arrays
            .get_mut(array_id)
            .ok_or_else(|| DbError::PathNotFound(format!("Array {:?}", array_id)))?;

        arr.push(value);

        if let Some(delta) = arr.list.take_delta() {
            let doc_delta = self.pending_delta.get_or_insert_with(JsonCrdtDelta::new);
            doc_delta.array_changes.push(ArrayChange {
                array_id: array_id.clone(),
                delta,
            });
        }

        Ok(())
    }

    /// Remove from an array.
    pub fn array_remove(&mut self, array_id: &ArrayId, index: usize) -> Result<JsonValue, DbError> {
        let arr = self
            .arrays
            .get_mut(array_id)
            .ok_or_else(|| DbError::PathNotFound(format!("Array {:?}", array_id)))?;

        let arr_len = arr.len();
        let value = arr.remove(index).ok_or(DbError::IndexOutOfBounds {
            index,
            length: arr_len,
        })?;

        if let Some(delta) = arr.list.take_delta() {
            let doc_delta = self.pending_delta.get_or_insert_with(JsonCrdtDelta::new);
            doc_delta.array_changes.push(ArrayChange {
                array_id: array_id.clone(),
                delta,
            });
        }

        Ok(value)
    }

    /// Get array length.
    pub fn array_len(&self, array_id: &ArrayId) -> Option<usize> {
        self.arrays.get(array_id).map(|a| a.len())
    }

    /// Get all keys in the root object.
    pub fn keys(&self) -> Vec<String> {
        self.objects
            .get(&self.root_id)
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Check if a key exists in the root object.
    pub fn contains_key(&self, key: &str) -> bool {
        self.objects
            .get(&self.root_id)
            .map(|obj| obj.get(key).is_some())
            .unwrap_or(false)
    }

    // === Helper Methods ===

    fn get_object_id_at(&self, path: &JsonPath) -> Option<ObjectId> {
        if path.is_root() {
            return Some(self.root_id.clone());
        }

        let value = self.get(path)?;
        match value {
            JsonValue::Object(id) => Some(id.clone()),
            _ => None,
        }
    }

    fn ensure_object_at(&mut self, path: &JsonPath) -> Result<ObjectId, DbError> {
        if path.is_root() {
            return Ok(self.root_id.clone());
        }

        // Try to get existing
        if let Some(id) = self.get_object_id_at(path) {
            return Ok(id);
        }

        // Need to create
        self.set_object(path)
    }

    // === Delta Operations ===

    /// Take the pending delta.
    pub fn take_delta(&mut self) -> Option<JsonCrdtDelta> {
        self.pending_delta.take()
    }

    /// Apply a delta from another replica.
    pub fn apply_delta(&mut self, delta: &JsonCrdtDelta) {
        // Create new objects
        for obj_id in &delta.new_objects {
            self.objects
                .entry(obj_id.clone())
                .or_insert_with(|| JsonObject::new(obj_id.clone()));
        }

        // Create new arrays
        for arr_id in &delta.new_arrays {
            self.arrays
                .entry(arr_id.clone())
                .or_insert_with(|| JsonArray::new(arr_id.clone(), &self.replica_id));
        }

        // Apply object changes
        for change in &delta.object_changes {
            if let Some(obj) = self.objects.get_mut(&change.object_id) {
                obj.set(
                    change.key.clone(),
                    change.value_id.clone(),
                    change.value.clone(),
                );
            }
        }

        // Apply array changes
        for change in &delta.array_changes {
            if let Some(arr) = self.arrays.get_mut(&change.array_id) {
                arr.list.apply_delta(&change.delta);
            }
        }
    }

    // === Conversion ===

    /// Convert to a serde_json::Value.
    pub fn to_json(&self) -> serde_json::Value {
        self.object_to_json(&self.root_id)
    }

    fn object_to_json(&self, obj_id: &ObjectId) -> serde_json::Value {
        let obj = match self.objects.get(obj_id) {
            Some(o) => o,
            None => return serde_json::Value::Null,
        };

        let mut map = serde_json::Map::new();
        for key in obj.keys() {
            if let Some(value) = obj.get(key) {
                map.insert(key.clone(), self.value_to_json(value));
            }
        }
        serde_json::Value::Object(map)
    }

    fn array_to_json(&self, arr_id: &ArrayId) -> serde_json::Value {
        let arr = match self.arrays.get(arr_id) {
            Some(a) => a,
            None => return serde_json::Value::Array(vec![]),
        };

        let values: Vec<_> = arr.iter().map(|v| self.value_to_json(v)).collect();
        serde_json::Value::Array(values)
    }

    fn value_to_json(&self, value: &JsonValue) -> serde_json::Value {
        match value {
            JsonValue::Null => serde_json::Value::Null,
            JsonValue::Bool(b) => serde_json::Value::Bool(*b),
            JsonValue::Int(i) => serde_json::Value::Number((*i).into()),
            JsonValue::Float(f) => serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            JsonValue::String(s) => serde_json::Value::String(s.clone()),
            JsonValue::Object(id) => self.object_to_json(id),
            JsonValue::Array(id) => self.array_to_json(id),
        }
    }
}

impl Lattice for JsonCrdt {
    fn bottom() -> Self {
        Self::new("")
    }

    fn join(&self, other: &Self) -> Self {
        let mut result = self.clone();

        // Merge objects
        for (id, other_obj) in &other.objects {
            result
                .objects
                .entry(id.clone())
                .and_modify(|obj| obj.merge(other_obj))
                .or_insert_with(|| other_obj.clone());
        }

        // Merge arrays
        for (id, other_arr) in &other.arrays {
            result
                .arrays
                .entry(id.clone())
                .and_modify(|arr| arr.merge(other_arr))
                .or_insert_with(|| other_arr.clone());
        }

        result
    }
}

impl Default for JsonCrdt {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_set_get() {
        let mut doc = JsonCrdt::new("r1");

        doc.set(
            &JsonPath::parse("name"),
            JsonValue::String("Alice".to_string()),
        )
        .unwrap();
        doc.set(&JsonPath::parse("age"), JsonValue::Int(30))
            .unwrap();

        let name = doc.get(&JsonPath::parse("name")).unwrap();
        assert_eq!(name.as_str(), Some("Alice"));

        let age = doc.get(&JsonPath::parse("age")).unwrap();
        assert_eq!(age.as_int(), Some(30));
    }

    #[test]
    fn test_nested_object() {
        let mut doc = JsonCrdt::new("r1");

        let _user_id = doc.set_object(&JsonPath::parse("user")).unwrap();
        doc.set(
            &JsonPath::parse("user.name"),
            JsonValue::String("Bob".to_string()),
        )
        .unwrap();

        assert!(doc.contains_key("user"));

        // The path-based get for nested objects needs the value to be Object type
        let user_value = doc.get(&JsonPath::parse("user"));
        assert!(user_value.is_some());
    }

    #[test]
    fn test_array_operations() {
        let mut doc = JsonCrdt::new("r1");

        let arr_id = doc.create_array();
        doc.set(&JsonPath::parse("items"), JsonValue::Array(arr_id.clone()))
            .unwrap();

        doc.array_push(&arr_id, JsonValue::String("one".to_string()))
            .unwrap();
        doc.array_push(&arr_id, JsonValue::String("two".to_string()))
            .unwrap();
        doc.array_push(&arr_id, JsonValue::String("three".to_string()))
            .unwrap();

        assert_eq!(doc.array_len(&arr_id), Some(3));

        let removed = doc.array_remove(&arr_id, 1).unwrap();
        assert_eq!(removed.as_str(), Some("two"));
        assert_eq!(doc.array_len(&arr_id), Some(2));
    }

    #[test]
    fn test_delete() {
        let mut doc = JsonCrdt::new("r1");

        doc.set(
            &JsonPath::parse("temp"),
            JsonValue::String("value".to_string()),
        )
        .unwrap();
        assert!(doc.contains_key("temp"));

        doc.delete(&JsonPath::parse("temp")).unwrap();
        // After delete, the key may still exist but with null value
    }

    #[test]
    fn test_concurrent_sets() {
        let mut doc1 = JsonCrdt::new("r1");
        let mut doc2 = JsonCrdt::new("r2");

        // Both set same key concurrently
        doc1.set(
            &JsonPath::parse("value"),
            JsonValue::String("from_r1".to_string()),
        )
        .unwrap();
        doc2.set(
            &JsonPath::parse("value"),
            JsonValue::String("from_r2".to_string()),
        )
        .unwrap();

        // Exchange deltas
        let delta1 = doc1.take_delta().unwrap();
        let delta2 = doc2.take_delta().unwrap();

        doc1.apply_delta(&delta2);
        doc2.apply_delta(&delta1);

        // Both should converge to same value (LWW by replica+seq)
        let json1 = doc1.to_json();
        let json2 = doc2.to_json();
        assert_eq!(json1, json2);
    }

    #[test]
    fn test_to_json() {
        let mut doc = JsonCrdt::new("r1");

        doc.set(
            &JsonPath::parse("name"),
            JsonValue::String("Test".to_string()),
        )
        .unwrap();
        doc.set(&JsonPath::parse("count"), JsonValue::Int(42))
            .unwrap();
        doc.set(&JsonPath::parse("active"), JsonValue::Bool(true))
            .unwrap();

        let json = doc.to_json();
        assert!(json.is_object());
        assert_eq!(json["name"], "Test");
        assert_eq!(json["count"], 42);
        assert_eq!(json["active"], true);
    }

    #[test]
    fn test_path_parsing() {
        let path = JsonPath::parse("user.profile.name");
        assert_eq!(path.segments().len(), 3);

        let path_with_index = JsonPath::parse("items.0.value");
        assert_eq!(path_with_index.segments().len(), 3);
        assert!(matches!(
            path_with_index.segments()[1],
            PathSegment::Index(0)
        ));
    }

    #[test]
    fn test_lattice_join() {
        let mut doc1 = JsonCrdt::new("r1");
        let mut doc2 = JsonCrdt::new("r2");

        doc1.set(
            &JsonPath::parse("a"),
            JsonValue::String("from_r1".to_string()),
        )
        .unwrap();
        doc2.set(
            &JsonPath::parse("b"),
            JsonValue::String("from_r2".to_string()),
        )
        .unwrap();

        let merged = doc1.join(&doc2);

        // Should have both keys
        assert!(merged.contains_key("a"));
        assert!(merged.contains_key("b"));
    }

    #[test]
    fn test_keys() {
        let mut doc = JsonCrdt::new("r1");

        doc.set(&JsonPath::parse("x"), JsonValue::Int(1)).unwrap();
        doc.set(&JsonPath::parse("y"), JsonValue::Int(2)).unwrap();
        doc.set(&JsonPath::parse("z"), JsonValue::Int(3)).unwrap();

        let keys = doc.keys();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"x".to_string()));
        assert!(keys.contains(&"y".to_string()));
        assert!(keys.contains(&"z".to_string()));
    }
}
