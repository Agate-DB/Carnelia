//! Multi-Value Register CRDT
//!
//! The Multi-Value Register (MV-Register) maintains a set of concurrent values
//! instead of choosing a single winner. Each value is tagged with a unique
//! identifier (dot) to distinguish different writes.
//!
//! When concurrent writes occur, the register contains all of them until
//! one of them is explicitly observed and the others are discarded.

use crate::lattice::Lattice;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use ulid::Ulid;

/// A unique identifier for a write operation
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Dot {
    pub replica_id: String,
    pub unique_id: Ulid,
}

impl Dot {
    pub fn new(replica_id: impl Into<String>) -> Self {
        Self {
            replica_id: replica_id.into(),
            unique_id: Ulid::new(),
        }
    }
}

/// A Multi-Value Register CRDT
///
/// Maintains a set of values, each with a unique dot. This allows
/// concurrent writes to coexist until explicitly resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MVRegister<T: Ord + Clone> {
    /// Current values, each tagged with a unique dot
    values: BTreeMap<Dot, T>,
}

// Custom serialization: serialize as Vec<(Dot, T)> for JSON compatibility
impl<T: Ord + Clone + Serialize> Serialize for MVRegister<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let entries: Vec<(&Dot, &T)> = self.values.iter().collect();
        entries.serialize(serializer)
    }
}

impl<'de, T: Ord + Clone + Deserialize<'de>> Deserialize<'de> for MVRegister<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let entries: Vec<(Dot, T)> = Vec::deserialize(deserializer)?;
        Ok(Self {
            values: entries.into_iter().collect(),
        })
    }
}

impl<T: Ord + Clone> MVRegister<T> {
    /// Create a new empty Multi-Value Register
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    /// Write a new value, generating a unique dot
    pub fn write(&mut self, replica_id: &str, value: T) -> Dot {
        let dot = Dot::new(replica_id);
        // Clear previous values and insert the new one
        self.values.clear();
        self.values.insert(dot.clone(), value);
        dot
    }

    /// Write a value with a specific dot (for merging)
    pub fn write_with_dot(&mut self, dot: Dot, value: T) {
        self.values.insert(dot, value);
    }

    /// Get all current values
    pub fn read(&self) -> Vec<&T> {
        self.values.values().collect()
    }

    /// Get all current values with their dots
    pub fn read_with_dots(&self) -> Vec<(&Dot, &T)> {
        self.values.iter().map(|(d, v)| (d, v)).collect()
    }

    /// Resolve concurrent values by choosing one (for write-after-read consistency)
    pub fn resolve(&mut self, replica_id: &str, value: T) -> Dot {
        let dot = Dot::new(replica_id);
        self.values.clear();
        self.values.insert(dot.clone(), value);
        dot
    }

    /// Remove a specific dot (value)
    pub fn remove_dot(&mut self, dot: &Dot) {
        self.values.remove(dot);
    }

    /// Check if register is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get the number of concurrent values
    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl<T: Ord + Clone> Default for MVRegister<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + Clone> Lattice for MVRegister<T> {
    fn bottom() -> Self {
        Self::new()
    }

    /// Join operation: union of all values from both registers
    /// This represents the concurrent state after a merge
    fn join(&self, other: &Self) -> Self {
        let mut values = self.values.clone();

        // Union all values from other
        for (dot, value) in &other.values {
            // Only insert if we don't already have a value with this dot
            values.entry(dot.clone()).or_insert_with(|| value.clone());
        }

        Self { values }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mvreg_basic_write() {
        let mut reg = MVRegister::new();

        assert!(reg.is_empty());

        let _dot1 = reg.write("replica1", 42);
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.read(), vec![&42]);
    }

    #[test]
    fn test_mvreg_concurrent_writes() {
        let mut reg = MVRegister::new();

        // First write
        let _dot1 = reg.write("replica1", 10);
        assert_eq!(reg.read(), vec![&10]);

        // Concurrent write (should clear previous)
        let _dot2 = reg.write("replica2", 20);
        assert_eq!(reg.read(), vec![&20]);
    }

    #[test]
    fn test_mvreg_merge_concurrent_values() {
        let mut reg1 = MVRegister::new();
        reg1.write("replica1", 10);

        let mut reg2 = MVRegister::new();
        reg2.write("replica2", 20);

        // Merge should have both values
        let merged = reg1.join(&reg2);
        let values = merged.read();
        assert_eq!(values.len(), 2);
        assert!(values.contains(&&10));
        assert!(values.contains(&&20));
    }

    #[test]
    fn test_mvreg_resolve_conflicts() {
        let mut reg = MVRegister::new();

        // Multiple concurrent writes
        reg.write("replica1", 10);
        let mut reg2 = MVRegister::new();
        reg2.write("replica2", 20);

        let merged = reg.join(&reg2);
        assert_eq!(merged.len(), 2);

        // Resolve by choosing one
        let mut resolved = merged.clone();
        resolved.resolve("replica3", 30);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved.read(), vec![&30]);
    }

    #[test]
    fn test_mvreg_join_idempotent() {
        let mut reg = MVRegister::new();
        reg.write("replica1", 42);

        let joined = reg.join(&reg);
        assert_eq!(joined.len(), reg.len());
        assert_eq!(joined.read(), reg.read());
    }

    #[test]
    fn test_mvreg_join_commutative() {
        let mut reg1 = MVRegister::new();
        reg1.write("replica1", 10);

        let mut reg2 = MVRegister::new();
        reg2.write("replica2", 20);

        let joined1 = reg1.join(&reg2);
        let joined2 = reg2.join(&reg1);

        assert_eq!(joined1.len(), joined2.len());

        let mut v1 = joined1.read();
        let mut v2 = joined2.read();
        v1.sort();
        v2.sort();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_mvreg_join_associative() {
        let mut reg1 = MVRegister::new();
        reg1.write("replica1", 10);

        let mut reg2 = MVRegister::new();
        reg2.write("replica2", 20);

        let mut reg3 = MVRegister::new();
        reg3.write("replica3", 30);

        let left = reg1.join(&reg2).join(&reg3);
        let right = reg1.join(&reg2.join(&reg3));

        assert_eq!(left.len(), right.len());
    }

    #[test]
    fn test_mvreg_bottom_is_identity() {
        let mut reg = MVRegister::new();
        reg.write("replica1", 42);

        let bottom = MVRegister::bottom();
        let joined = reg.join(&bottom);

        assert_eq!(joined.len(), reg.len());
        assert_eq!(joined.read(), reg.read());
    }

    #[test]
    fn test_mvreg_serialization() {
        let mut reg = MVRegister::new();
        reg.write("replica1", 42);

        let serialized = serde_json::to_string(&reg).unwrap();
        let deserialized: MVRegister<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.read(), vec![&42]);
    }
}
