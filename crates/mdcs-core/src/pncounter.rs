//! PN-Counter (Positive-Negative Counter) CRDT
//!
//! A PN-Counter supports both increment and decrement operations by maintaining
//! two separate counters: one for increments (P) and one for decrements (N).
//! The value is P - N.
//!
//! Each replica has its own counter entry, and the join operation performs
//! component-wise max across all replicas.

use crate::lattice::Lattice;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A Positive-Negative Counter CRDT
///
/// Supports both increment and decrement by maintaining two separate counters.
/// Value = sum(increments) - sum(decrements)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PNCounter<K: Ord + Clone> {
    /// Per-replica increment counters
    increments: BTreeMap<K, u64>,
    /// Per-replica decrement counters
    decrements: BTreeMap<K, u64>,
}

impl<K: Ord + Clone> PNCounter<K> {
    /// Create a new PN-Counter
    pub fn new() -> Self {
        Self {
            increments: BTreeMap::new(),
            decrements: BTreeMap::new(),
        }
    }

    /// Increment the counter for a specific replica
    pub fn increment(&mut self, replica_id: K, amount: u64) {
        let entry = self.increments.entry(replica_id).or_insert(0);
        *entry = entry.saturating_add(amount);
    }

    /// Decrement the counter for a specific replica
    pub fn decrement(&mut self, replica_id: K, amount: u64) {
        let entry = self.decrements.entry(replica_id).or_insert(0);
        *entry = entry.saturating_add(amount);
    }

    /// Get the current value (sum of increments - sum of decrements)
    pub fn value(&self) -> i64 {
        let inc_sum: u64 = self.increments.values().sum();
        let dec_sum: u64 = self.decrements.values().sum();
        (inc_sum as i64).saturating_sub(dec_sum as i64)
    }

    /// Get the increment counter for a replica
    pub fn get_increment(&self, replica_id: &K) -> u64 {
        self.increments.get(replica_id).copied().unwrap_or(0)
    }

    /// Get the decrement counter for a replica
    pub fn get_decrement(&self, replica_id: &K) -> u64 {
        self.decrements.get(replica_id).copied().unwrap_or(0)
    }

    /// Get a reference to all increment counters
    pub fn increments(&self) -> &BTreeMap<K, u64> {
        &self.increments
    }

    /// Get a reference to all decrement counters
    pub fn decrements(&self) -> &BTreeMap<K, u64> {
        &self.decrements
    }
}

impl<K: Ord + Clone> Default for PNCounter<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord + Clone> Lattice for PNCounter<K> {
    fn bottom() -> Self {
        Self::new()
    }

    /// Join operation performs component-wise max on both counters
    /// This ensures that concurrent updates always converge to the same value
    fn join(&self, other: &Self) -> Self {
        let mut increments = self.increments.clone();
        let mut decrements = self.decrements.clone();

        // Merge other's increments (take max for each replica)
        for (k, v) in &other.increments {
            increments
                .entry(k.clone())
                .and_modify(|e| *e = (*e).max(*v))
                .or_insert(*v);
        }

        // Merge other's decrements (take max for each replica)
        for (k, v) in &other.decrements {
            decrements
                .entry(k.clone())
                .and_modify(|e| *e = (*e).max(*v))
                .or_insert(*v);
        }

        Self {
            increments,
            decrements,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pncounter_basic_operations() {
        let mut counter = PNCounter::new();

        // Increment from replica "A"
        counter.increment("A", 5);
        assert_eq!(counter.value(), 5);

        // Decrement from replica "B"
        counter.decrement("B", 2);
        assert_eq!(counter.value(), 3);

        // Increment again
        counter.increment("A", 3);
        assert_eq!(counter.value(), 6);
    }

    #[test]
    fn test_pncounter_join_idempotent() {
        let mut c1 = PNCounter::new();
        c1.increment("A", 5);
        c1.decrement("B", 2);

        let joined = c1.join(&c1);
        assert_eq!(joined.value(), c1.value());
        assert_eq!(joined.value(), 3);
    }

    #[test]
    fn test_pncounter_join_commutative() {
        let mut c1 = PNCounter::new();
        c1.increment("A", 5);

        let mut c2 = PNCounter::new();
        c2.increment("B", 3);
        c2.decrement("A", 1);

        let joined1 = c1.join(&c2);
        let joined2 = c2.join(&c1);

        assert_eq!(joined1.value(), joined2.value());
        assert_eq!(joined1.get_increment(&"A"), 5);
        assert_eq!(joined1.get_increment(&"B"), 3);
        assert_eq!(joined1.get_decrement(&"A"), 1);
    }

    #[test]
    fn test_pncounter_join_associative() {
        let mut c1 = PNCounter::new();
        c1.increment("A", 1);

        let mut c2 = PNCounter::new();
        c2.increment("B", 2);

        let mut c3 = PNCounter::new();
        c3.decrement("C", 1);

        let left = c1.join(&c2).join(&c3);
        let right = c1.join(&c2.join(&c3));

        assert_eq!(left.value(), right.value());
    }

    #[test]
    fn test_pncounter_bottom_is_identity() {
        let mut counter = PNCounter::new();
        counter.increment("A", 5);
        counter.decrement("B", 2);

        let bottom = PNCounter::bottom();
        let joined = counter.join(&bottom);

        assert_eq!(joined.value(), counter.value());
    }

    #[test]
    fn test_pncounter_convergence_different_order() {
        let mut c1 = PNCounter::new();
        c1.increment("X", 10);
        c1.decrement("Y", 3);

        let mut c2 = PNCounter::new();
        c2.increment("Z", 5);
        c2.decrement("X", 2);

        // Apply updates in different order
        let mut state1 = PNCounter::bottom();
        state1.join_assign(&c1);
        state1.join_assign(&c2);

        let mut state2 = PNCounter::bottom();
        state2.join_assign(&c2);
        state2.join_assign(&c1);

        assert_eq!(state1.value(), state2.value());
    }

    #[test]
    fn test_pncounter_serialization() {
        let mut counter = PNCounter::new();
        counter.increment("replica1", 100);
        counter.decrement("replica2", 25);

        let serialized = serde_json::to_string(&counter).unwrap();
        let deserialized: PNCounter<String> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.value(), counter.value());
        assert_eq!(
            deserialized.get_increment(&"replica1".to_string()),
            100
        );
        assert_eq!(
            deserialized.get_decrement(&"replica2".to_string()),
            25
        );
    }
}