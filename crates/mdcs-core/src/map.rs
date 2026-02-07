//! Map CRDT - A composable container for nested CRDTs
//!
//! The Map CRDT allows mapping keys to other CRDT values, enabling
//! the construction of complex nested data structures like JSON documents.
//!
//! Key design: A single shared causal context ensures that causality is
//! tracked consistently across the entire map and all nested CRDTs.

use crate::lattice::Lattice;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

/// A unique identifier for a write operation (dot)
/// Tracks which replica created this value and when
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Dot {
    pub replica_id: String,
    pub seq: u64,
}

impl Dot {
    pub fn new(replica_id: impl Into<String>, seq: u64) -> Self {
        Self {
            replica_id: replica_id.into(),
            seq,
        }
    }
}

/// Causal context: tracks all known events for consistent removal
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CausalContext {
    /// Set of all dots that have been created
    dots: std::collections::BTreeSet<Dot>,
}

impl CausalContext {
    pub fn new() -> Self {
        Self {
            dots: std::collections::BTreeSet::new(),
        }
    }

    pub fn add_dot(&mut self, dot: Dot) {
        self.dots.insert(dot);
    }

    pub fn contains(&self, dot: &Dot) -> bool {
        self.dots.contains(dot)
    }

    pub fn join(&self, other: &CausalContext) -> CausalContext {
        let mut joined = self.clone();
        for dot in &other.dots {
            joined.add_dot(dot.clone());
        }
        joined
    }
}

impl Default for CausalContext {
    fn default() -> Self {
        Self::new()
    }
}

/// A generic value that can be stored in the map
/// This enables composing different CRDT types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapValue {
    Int(i64),
    Text(String),
    Bytes(Vec<u8>),
    // For nested maps: Box<CRDTMap>
    // For other CRDTs: Box<dyn Lattice>
}

/// Map CRDT - composable container for nested CRDTs
///
/// Maps keys to values, each value is tagged with a dot.
/// A value is "live" if its dot is in the context.
/// A value is "removed" if its dot is in the context but not in the store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CRDTMap<K: Ord + Clone> {
    /// Maps keys to dots that have been written to this key
    entries: BTreeMap<K, BTreeMap<Dot, MapValue>>,
    /// Shared causal context: all dots that have been created or seen
    context: CausalContext,
    /// Sequence number for generating dots on this replica
    local_seq: u64,
}

// Custom serialization for CRDTMap to handle nested BTreeMap with Dot keys
impl<K: Ord + Clone + Serialize> Serialize for CRDTMap<K> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert entries to a serializable format
        #[derive(Serialize)]
        struct SerializableCRDTMap<'a, K: Ord + Clone + Serialize> {
            entries: Vec<(&'a K, Vec<(&'a Dot, &'a MapValue)>)>,
            context: &'a CausalContext,
        }

        let entries: Vec<_> = self
            .entries
            .iter()
            .map(|(k, v)| (k, v.iter().collect::<Vec<_>>()))
            .collect();

        let serializable = SerializableCRDTMap {
            entries,
            context: &self.context,
        };

        serializable.serialize(serializer)
    }
}

impl<'de, K: Ord + Clone + Deserialize<'de>> Deserialize<'de> for CRDTMap<K> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct DeserializableCRDTMap<K: Ord + Clone> {
            entries: Vec<(K, Vec<(Dot, MapValue)>)>,
            context: CausalContext,
        }

        let deserialized = DeserializableCRDTMap::<K>::deserialize(deserializer)?;

        let entries: BTreeMap<K, BTreeMap<Dot, MapValue>> = deserialized
            .entries
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();

        Ok(Self {
            entries,
            context: deserialized.context,
            local_seq: 0,
        })
    }
}

impl<K: Ord + Clone> CRDTMap<K> {
    /// Create a new empty map
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            context: CausalContext::new(),
            local_seq: 0,
        }
    }

    /// Put a value at a key (from this replica)
    pub fn put(&mut self, replica_id: &str, key: K, value: MapValue) -> Dot {
        let dot = Dot::new(replica_id, self.local_seq);
        self.local_seq += 1;

        // Create entry for this key if it doesn't exist
        let entry = self.entries.entry(key).or_insert_with(BTreeMap::new);

        // Clear previous values for this key and insert new one
        entry.clear();
        entry.insert(dot.clone(), value);

        // Track dot in causal context
        self.context.add_dot(dot.clone());

        dot
    }

    /// Get the current value at a key
    /// Returns the value if the key exists and has live entries
    pub fn get(&self, key: &K) -> Option<&MapValue> {
        self.entries
            .get(key)
            .and_then(|entry| entry.values().next())
    }

    /// Get all values at a key (for concurrent writes)
    pub fn get_all(&self, key: &K) -> Vec<&MapValue> {
        self.entries
            .get(key)
            .map(|entry| entry.values().collect())
            .unwrap_or_default()
    }

    /// Remove a key by recording all its current dots as removed
    pub fn remove(&mut self, key: &K) {
        if let Some(entry) = self.entries.get_mut(key) {
            // Mark all dots as removed by clearing them but keeping them in context
            entry.clear();
        }
    }

    /// Check if a key exists with live values
    pub fn contains_key(&self, key: &K) -> bool {
        self.entries
            .get(key)
            .map(|entry| !entry.is_empty())
            .unwrap_or(false)
    }

    /// Get all keys that have live values
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.entries
            .iter()
            .filter_map(|(k, v)| if !v.is_empty() { Some(k) } else { None })
    }

    /// Get the causal context
    pub fn context(&self) -> &CausalContext {
        &self.context
    }

    /// Add a value with a specific dot (for merging)
    pub fn put_with_dot(&mut self, key: K, dot: Dot, value: MapValue) {
        let entry = self.entries.entry(key).or_insert_with(BTreeMap::new);
        entry.insert(dot.clone(), value);
        self.context.add_dot(dot);
    }
}

impl<K: Ord + Clone> Default for CRDTMap<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord + Clone> Lattice for CRDTMap<K> {
    fn bottom() -> Self {
        Self::new()
    }

    /// Join operation: merge all entries and contexts
    /// For each key, union all the dots and their values
    fn join(&self, other: &Self) -> Self {
        let mut entries = self.entries.clone();
        let mut context = self.context.clone();

        // Merge other's entries
        for (key, other_entry) in &other.entries {
            let entry = entries.entry(key.clone()).or_insert_with(BTreeMap::new);
            for (dot, value) in other_entry {
                entry.insert(dot.clone(), value.clone());
            }
        }

        // Merge contexts
        context = context.join(&other.context);

        Self {
            entries,
            context,
            local_seq: self.local_seq.max(other.local_seq),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_basic_operations() {
        let mut map: CRDTMap<String> = CRDTMap::new();

        map.put("replica1", "key1".to_string(), MapValue::Int(42));
        assert_eq!(map.get(&"key1".to_string()), Some(&MapValue::Int(42)));

        map.put(
            "replica1",
            "key2".to_string(),
            MapValue::Text("hello".to_string()),
        );
        assert_eq!(
            map.get(&"key2".to_string()),
            Some(&MapValue::Text("hello".to_string()))
        );
    }

    #[test]
    fn test_map_remove() {
        let mut map: CRDTMap<String> = CRDTMap::new();

        map.put("replica1", "key1".to_string(), MapValue::Int(42));
        assert!(map.contains_key(&"key1".to_string()));

        map.remove(&"key1".to_string());
        assert!(!map.contains_key(&"key1".to_string()));
    }

    #[test]
    fn test_map_join_idempotent() {
        let mut map1: CRDTMap<String> = CRDTMap::new();
        map1.put("replica1", "key1".to_string(), MapValue::Int(42));

        let joined = map1.join(&map1);
        assert_eq!(joined.get(&"key1".to_string()), Some(&MapValue::Int(42)));
    }

    #[test]
    fn test_map_join_commutative() {
        let mut map1: CRDTMap<String> = CRDTMap::new();
        map1.put("replica1", "key1".to_string(), MapValue::Int(42));

        let mut map2: CRDTMap<String> = CRDTMap::new();
        map2.put(
            "replica2",
            "key2".to_string(),
            MapValue::Text("world".to_string()),
        );

        let joined1 = map1.join(&map2);
        let joined2 = map2.join(&map1);

        assert_eq!(joined1.get(&"key1".to_string()), Some(&MapValue::Int(42)));
        assert_eq!(
            joined1.get(&"key2".to_string()),
            Some(&MapValue::Text("world".to_string()))
        );

        assert_eq!(joined2.get(&"key1".to_string()), Some(&MapValue::Int(42)));
        assert_eq!(
            joined2.get(&"key2".to_string()),
            Some(&MapValue::Text("world".to_string()))
        );
    }

    #[test]
    fn test_map_join_associative() {
        let mut map1: CRDTMap<String> = CRDTMap::new();
        map1.put("replica1", "key1".to_string(), MapValue::Int(1));

        let mut map2: CRDTMap<String> = CRDTMap::new();
        map2.put("replica2", "key2".to_string(), MapValue::Int(2));

        let mut map3: CRDTMap<String> = CRDTMap::new();
        map3.put("replica3", "key3".to_string(), MapValue::Int(3));

        let left = map1.join(&map2).join(&map3);
        let right = map1.join(&map2.join(&map3));

        assert_eq!(left.get(&"key1".to_string()), Some(&MapValue::Int(1)));
        assert_eq!(left.get(&"key2".to_string()), Some(&MapValue::Int(2)));
        assert_eq!(left.get(&"key3".to_string()), Some(&MapValue::Int(3)));

        assert_eq!(right.get(&"key1".to_string()), Some(&MapValue::Int(1)));
        assert_eq!(right.get(&"key2".to_string()), Some(&MapValue::Int(2)));
        assert_eq!(right.get(&"key3".to_string()), Some(&MapValue::Int(3)));
    }

    #[test]
    fn test_map_concurrent_writes_different_keys() {
        let mut map1: CRDTMap<String> = CRDTMap::new();
        map1.put("replica1", "key1".to_string(), MapValue::Int(10));

        let mut map2: CRDTMap<String> = CRDTMap::new();
        map2.put("replica2", "key2".to_string(), MapValue::Int(20));

        let merged = map1.join(&map2);
        assert_eq!(merged.get(&"key1".to_string()), Some(&MapValue::Int(10)));
        assert_eq!(merged.get(&"key2".to_string()), Some(&MapValue::Int(20)));
    }

    #[test]
    fn test_map_serialization() {
        let mut map: CRDTMap<String> = CRDTMap::new();
        map.put("replica1", "key1".to_string(), MapValue::Int(42));
        map.put(
            "replica1",
            "key2".to_string(),
            MapValue::Text("hello".to_string()),
        );

        let serialized = serde_json::to_string(&map).unwrap();
        let deserialized: CRDTMap<String> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            deserialized.get(&"key1".to_string()),
            Some(&MapValue::Int(42))
        );
        assert_eq!(
            deserialized.get(&"key2".to_string()),
            Some(&MapValue::Text("hello".to_string()))
        );
    }
}
