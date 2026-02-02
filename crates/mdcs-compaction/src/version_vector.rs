//! Version vector for compact causal context representation.
//!
//! A version vector summarizes the causal context by tracking the highest
//! sequence number seen from each replica. This is more compact than storing
//! all individual dots.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A single entry in a version vector.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorEntry {
    /// The replica ID.
    pub replica_id: String,
    /// The highest sequence number seen from this replica.
    pub sequence: u64,
}

/// A version vector tracking the frontier of seen updates per replica.
///
/// Version vectors provide a compact summary of causal history when
/// updates from each replica are contiguous (no gaps).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionVector {
    /// Map from replica ID to highest seen sequence number.
    entries: BTreeMap<String, u64>,
}

impl VersionVector {
    /// Create an empty version vector.
    pub fn new() -> Self {
        VersionVector {
            entries: BTreeMap::new(),
        }
    }

    /// Create a version vector from entries.
    pub fn from_entries(entries: impl IntoIterator<Item = (String, u64)>) -> Self {
        VersionVector {
            entries: entries.into_iter().collect(),
        }
    }

    /// Get the sequence number for a replica.
    pub fn get(&self, replica_id: &str) -> u64 {
        self.entries.get(replica_id).copied().unwrap_or(0)
    }

    /// Set the sequence number for a replica.
    pub fn set(&mut self, replica_id: impl Into<String>, sequence: u64) {
        let replica_id = replica_id.into();
        if sequence > 0 {
            self.entries.insert(replica_id, sequence);
        }
    }

    /// Increment the sequence number for a replica, returning the new value.
    pub fn increment(&mut self, replica_id: impl Into<String>) -> u64 {
        let replica_id = replica_id.into();
        let entry = self.entries.entry(replica_id).or_insert(0);
        *entry += 1;
        *entry
    }

    /// Check if this vector dominates another (is causally after or concurrent).
    /// Returns true if for all replicas, self[r] >= other[r].
    pub fn dominates(&self, other: &VersionVector) -> bool {
        // Check all entries in other
        for (replica_id, &seq) in &other.entries {
            if self.get(replica_id) < seq {
                return false;
            }
        }
        true
    }

    /// Check if this vector is strictly greater than another.
    /// Returns true if dominates(other) AND self != other.
    pub fn strictly_dominates(&self, other: &VersionVector) -> bool {
        self.dominates(other) && self != other
    }

    /// Check if two vectors are concurrent (neither dominates the other).
    pub fn is_concurrent_with(&self, other: &VersionVector) -> bool {
        !self.dominates(other) && !other.dominates(self)
    }

    /// Merge with another version vector (component-wise max).
    pub fn merge(&mut self, other: &VersionVector) {
        for (replica_id, &seq) in &other.entries {
            let current = self.entries.entry(replica_id.clone()).or_insert(0);
            *current = (*current).max(seq);
        }
    }

    /// Create a merged version vector without modifying self.
    pub fn merged_with(&self, other: &VersionVector) -> VersionVector {
        let mut result = self.clone();
        result.merge(other);
        result
    }

    /// Get the minimum across all replicas in both vectors.
    /// This represents the "stable" point that all replicas have seen.
    pub fn min_with(&self, other: &VersionVector) -> VersionVector {
        let mut result = VersionVector::new();
        
        // Get all replica IDs from both vectors
        let all_replicas: std::collections::BTreeSet<_> = self
            .entries
            .keys()
            .chain(other.entries.keys())
            .cloned()
            .collect();

        for replica_id in all_replicas {
            let self_seq = self.get(&replica_id);
            let other_seq = other.get(&replica_id);
            let min_seq = self_seq.min(other_seq);
            if min_seq > 0 {
                result.set(replica_id, min_seq);
            }
        }

        result
    }

    /// Iterate over all entries.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &u64)> {
        self.entries.iter()
    }

    /// Get the number of replicas tracked.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the version vector is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the sum of all sequence numbers (total operations seen).
    pub fn total_operations(&self) -> u64 {
        self.entries.values().sum()
    }

    /// Convert to a list of entries.
    pub fn to_entries(&self) -> Vec<VectorEntry> {
        self.entries
            .iter()
            .map(|(replica_id, &sequence)| VectorEntry {
                replica_id: replica_id.clone(),
                sequence,
            })
            .collect()
    }

    /// Create from a list of entries.
    pub fn from_entry_list(entries: Vec<VectorEntry>) -> Self {
        VersionVector {
            entries: entries
                .into_iter()
                .map(|e| (e.replica_id, e.sequence))
                .collect(),
        }
    }

    /// Check if a specific (replica_id, sequence) pair is included.
    pub fn contains(&self, replica_id: &str, sequence: u64) -> bool {
        self.get(replica_id) >= sequence
    }

    /// Get the difference: operations in self but not in other.
    /// Returns (replica_id, start_seq, end_seq) ranges.
    pub fn diff(&self, other: &VersionVector) -> Vec<(String, u64, u64)> {
        let mut diffs = Vec::new();

        for (replica_id, &self_seq) in &self.entries {
            let other_seq = other.get(replica_id);
            if self_seq > other_seq {
                diffs.push((replica_id.clone(), other_seq + 1, self_seq));
            }
        }

        diffs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_vector_basic() {
        let mut vv = VersionVector::new();
        assert_eq!(vv.get("r1"), 0);

        vv.set("r1", 5);
        assert_eq!(vv.get("r1"), 5);

        let seq = vv.increment("r1");
        assert_eq!(seq, 6);
        assert_eq!(vv.get("r1"), 6);
    }

    #[test]
    fn test_version_vector_dominates() {
        let vv1 = VersionVector::from_entries([("r1".to_string(), 5), ("r2".to_string(), 3)]);
        let vv2 = VersionVector::from_entries([("r1".to_string(), 3), ("r2".to_string(), 3)]);
        let vv3 = VersionVector::from_entries([("r1".to_string(), 5), ("r2".to_string(), 5)]);

        assert!(vv1.dominates(&vv2));
        assert!(!vv2.dominates(&vv1));
        assert!(vv3.dominates(&vv1));
        assert!(!vv1.dominates(&vv3));
    }

    #[test]
    fn test_version_vector_concurrent() {
        let vv1 = VersionVector::from_entries([("r1".to_string(), 5), ("r2".to_string(), 3)]);
        let vv2 = VersionVector::from_entries([("r1".to_string(), 3), ("r2".to_string(), 5)]);

        assert!(vv1.is_concurrent_with(&vv2));
        assert!(vv2.is_concurrent_with(&vv1));
    }

    #[test]
    fn test_version_vector_merge() {
        let vv1 = VersionVector::from_entries([("r1".to_string(), 5), ("r2".to_string(), 3)]);
        let vv2 = VersionVector::from_entries([("r1".to_string(), 3), ("r2".to_string(), 7)]);

        let merged = vv1.merged_with(&vv2);
        assert_eq!(merged.get("r1"), 5);
        assert_eq!(merged.get("r2"), 7);
    }

    #[test]
    fn test_version_vector_min() {
        let vv1 = VersionVector::from_entries([("r1".to_string(), 5), ("r2".to_string(), 3)]);
        let vv2 = VersionVector::from_entries([("r1".to_string(), 3), ("r2".to_string(), 7)]);

        let min = vv1.min_with(&vv2);
        assert_eq!(min.get("r1"), 3);
        assert_eq!(min.get("r2"), 3);
    }

    #[test]
    fn test_version_vector_diff() {
        let vv1 = VersionVector::from_entries([("r1".to_string(), 10), ("r2".to_string(), 5)]);
        let vv2 = VersionVector::from_entries([("r1".to_string(), 7), ("r2".to_string(), 5)]);

        let diff = vv1.diff(&vv2);
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0], ("r1".to_string(), 8, 10));
    }

    #[test]
    fn test_version_vector_serialization() {
        let vv = VersionVector::from_entries([
            ("r1".to_string(), 5),
            ("r2".to_string(), 10),
        ]);

        let json = serde_json::to_string(&vv).unwrap();
        let deserialized: VersionVector = serde_json::from_str(&json).unwrap();
        assert_eq!(vv, deserialized);
    }

    #[test]
    fn test_version_vector_contains() {
        let vv = VersionVector::from_entries([("r1".to_string(), 5)]);

        assert!(vv.contains("r1", 1));
        assert!(vv.contains("r1", 5));
        assert!(!vv.contains("r1", 6));
        assert!(!vv.contains("r2", 1));
    }
}
