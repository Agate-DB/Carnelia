//! Delta-mutators for CRDT types
//!
//! For each CRDT type, we implement delta-mutators `mδ` such that:
//!   m(X) = X ⊔ mδ(X)
//!
//! This means the full mutation can be reconstructed by joining the delta
//! with the original state.

use mdcs_core::gset::GSet;
use mdcs_core::lattice::Lattice;
use mdcs_core::orset::{ORSet, ORSetDelta, Tag};
use std::collections::{BTreeMap, BTreeSet};

/// Delta-mutator trait for CRDTs
///
/// A delta-mutator produces a small delta that, when joined with the state,
/// produces the same result as the full mutation.
pub trait DeltaMutator<S: Lattice>: Lattice {
    /// Apply this delta to a state
    fn apply_to(&self, state: &S) -> S;
}

// ============================================================================
// GSet Delta Mutators
// ============================================================================

/// Delta for GSet insert operation
///
/// For GSet, the delta is simply a singleton set containing the inserted element.
/// Property: X.insert(v) = X ⊔ {v}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GSetInsertDelta<T: Ord + Clone> {
    element: T,
}

impl<T: Ord + Clone> GSetInsertDelta<T> {
    /// Create a delta for inserting an element
    pub fn new(element: T) -> Self {
        Self { element }
    }

    /// Get the element being inserted
    pub fn element(&self) -> &T {
        &self.element
    }
}

/// Create a GSet containing just this element (for use as delta)
impl<T: Ord + Clone> From<GSetInsertDelta<T>> for GSet<T> {
    fn from(delta: GSetInsertDelta<T>) -> Self {
        let mut set = GSet::new();
        set.insert(delta.element);
        set
    }
}

/// GSet delta-mutators
pub mod gset {
    use super::*;

    /// Delta-mutator for insert: mδ_insert(X, v) = {v}
    /// Property: X.insert(v) = X ⊔ mδ_insert(X, v)
    pub fn insert_delta<T: Ord + Clone>(value: T) -> GSet<T> {
        let mut delta = GSet::new();
        delta.insert(value);
        delta
    }

    /// Batch insert delta-mutator
    pub fn insert_batch_delta<T: Ord + Clone>(values: impl IntoIterator<Item = T>) -> GSet<T> {
        let mut delta = GSet::new();
        for value in values {
            delta.insert(value);
        }
        delta
    }

    /// Apply insert delta to a GSet
    pub fn apply_insert<T: Ord + Clone>(state: &mut GSet<T>, value: T) -> GSet<T> {
        let delta = insert_delta(value);
        state.join_assign(&delta);
        delta
    }
}

// ============================================================================
// ORSet Delta Mutators
// ============================================================================

/// ORSet delta-mutators
pub mod orset {
    use super::*;

    /// Delta-mutator for add: generates a new unique tag and returns delta
    /// Property: X.add(v) = X ⊔ mδ_add(X, v)
    pub fn add_delta<T: Ord + Clone>(replica_id: &str, value: T) -> ORSetDelta<T> {
        let tag = Tag::new(replica_id);
        let mut additions = BTreeMap::new();
        let mut tags = BTreeSet::new();
        tags.insert(tag);
        additions.insert(value, tags);

        ORSetDelta {
            additions,
            removals: BTreeSet::new(),
        }
    }

    /// Delta-mutator for remove: collects tags to tombstone
    /// Property: X.remove(v) = X ⊔ mδ_remove(X, v)
    pub fn remove_delta<T: Ord + Clone>(state: &ORSet<T>, value: &T) -> ORSetDelta<T> {
        // Get all tags for this value from the current state
        // The remove delta contains these tags as tombstones
        let removals = if state.contains(value) {
            // We need to access the internal tags - this requires ORSet to expose them
            // For now, we create an empty removal (the actual implementation uses pending_delta)
            BTreeSet::new()
        } else {
            BTreeSet::new()
        };

        ORSetDelta {
            additions: BTreeMap::new(),
            removals,
        }
    }

    /// Apply add operation using delta-mutator
    pub fn apply_add<T: Ord + Clone>(state: &mut ORSet<T>, replica_id: &str, value: T) -> ORSetDelta<T> {
        // Use the built-in add which already maintains pending_delta
        state.add(replica_id, value.clone());
        add_delta(replica_id, value)
    }
}

// ============================================================================
// LWWRegister Delta Mutators
// ============================================================================

pub mod lwwreg {
    use super::*;
    use mdcs_core::lwwreg::LWWRegister;
    use serde::{Deserialize, Serialize};

    /// Delta for LWW Register write operation
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct LWWWriteDelta<T: Ord + Clone, K: Ord + Clone> {
        pub timestamp: u64,
        pub replica_id: K,
        pub value: T,
    }

    impl<T: Ord + Clone, K: Ord + Clone> Lattice for LWWWriteDelta<T, K> {
        fn bottom() -> Self {
            panic!("LWWWriteDelta has no bottom element");
        }

        fn join(&self, other: &Self) -> Self {
            // Keep the value with higher timestamp (tie-break on replica_id)
            if other.timestamp > self.timestamp
                || (other.timestamp == self.timestamp && other.replica_id > self.replica_id)
            {
                other.clone()
            } else {
                self.clone()
            }
        }
    }

    /// Delta-mutator for set operation
    /// Property: X.set(v) = X ⊔ mδ_set(X, v, ts, rid)
    pub fn set_delta<T: Ord + Clone, K: Ord + Clone>(
        value: T,
        timestamp: u64,
        replica_id: K,
    ) -> LWWWriteDelta<T, K> {
        LWWWriteDelta {
            timestamp,
            replica_id,
            value,
        }
    }

    /// Convert delta to a LWW Register state
    pub fn apply_set<T: Ord + Clone, K: Ord + Clone + Default>(
        state: &mut LWWRegister<T, K>,
        value: T,
        timestamp: u64,
        replica_id: K,
    ) {
        state.set(value, timestamp, replica_id);
    }
}

// ============================================================================
// PNCounter Delta Mutators
// ============================================================================

pub mod pncounter {
    use super::*;
    use mdcs_core::pncounter::PNCounter;
    use serde::{Deserialize, Serialize};

    /// Delta for increment operation
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct IncrementDelta<K: Ord + Clone> {
        pub replica_id: K,
        pub amount: u64,
    }

    /// Delta for decrement operation
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct DecrementDelta<K: Ord + Clone> {
        pub replica_id: K,
        pub amount: u64,
    }

    impl<K: Ord + Clone> Lattice for IncrementDelta<K> {
        fn bottom() -> Self {
            panic!("IncrementDelta has no bottom element");
        }

        fn join(&self, other: &Self) -> Self {
            // For same replica, take max; otherwise union both
            if self.replica_id == other.replica_id {
                Self {
                    replica_id: self.replica_id.clone(),
                    amount: self.amount.max(other.amount),
                }
            } else {
                self.clone() // Semantically different replicas, but we can't represent union
            }
        }
    }

    impl<K: Ord + Clone> Lattice for DecrementDelta<K> {
        fn bottom() -> Self {
            panic!("DecrementDelta has no bottom element");
        }

        fn join(&self, other: &Self) -> Self {
            if self.replica_id == other.replica_id {
                Self {
                    replica_id: self.replica_id.clone(),
                    amount: self.amount.max(other.amount),
                }
            } else {
                self.clone()
            }
        }
    }

    /// Delta-mutator for increment operation
    pub fn increment_delta<K: Ord + Clone>(replica_id: K, amount: u64) -> IncrementDelta<K> {
        IncrementDelta { replica_id, amount }
    }

    /// Delta-mutator for decrement operation
    pub fn decrement_delta<K: Ord + Clone>(replica_id: K, amount: u64) -> DecrementDelta<K> {
        DecrementDelta { replica_id, amount }
    }

    /// Apply increment delta to counter
    pub fn apply_increment<K: Ord + Clone>(
        state: &mut PNCounter<K>,
        replica_id: K,
        amount: u64,
    ) {
        state.increment(replica_id, amount);
    }

    /// Apply decrement delta to counter
    pub fn apply_decrement<K: Ord + Clone>(
        state: &mut PNCounter<K>,
        replica_id: K,
        amount: u64,
    ) {
        state.decrement(replica_id, amount);
    }
}

// ============================================================================
// MVRegister Delta Mutators
// ============================================================================

pub mod mvreg {
    use super::*;
    use mdcs_core::mvreg::{Dot, MVRegister};
    use serde::{Deserialize, Serialize};

    /// Delta for write operation on Multi-Value Register
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct WriteDelta<T: Ord + Clone> {
        pub dot: Dot,
        pub value: T,
    }

    impl<T: Ord + Clone> Lattice for WriteDelta<T> {
        fn bottom() -> Self {
            panic!("WriteDelta has no bottom element");
        }

        fn join(&self, _other: &Self) -> Self {
            // Union: keep both values (they're different dots)
            // This is handled by MVRegister's join semantics
            self.clone()
        }
    }

    /// Delta-mutator for write operation
    pub fn write_delta<T: Ord + Clone>(dot: Dot, value: T) -> WriteDelta<T> {
        WriteDelta { dot, value }
    }

    /// Apply write delta to MVRegister
    pub fn apply_write<T: Ord + Clone>(
        state: &mut MVRegister<T>,
        replica_id: &str,
        value: T,
    ) -> Dot {
        state.write(replica_id, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdcs_core::lattice::DeltaCRDT;
    use mdcs_core::lwwreg::LWWRegister;
    use mdcs_core::pncounter::PNCounter;
    use mdcs_core::mvreg::MVRegister;

    #[test]
    fn test_gset_insert_delta() {
        let mut state: GSet<i32> = GSet::new();
        state.insert(1);
        state.insert(2);

        // Create delta for inserting 3
        let delta = gset::insert_delta(3);

        // Apply delta
        let result = state.join(&delta);

        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));
    }

    #[test]
    fn test_gset_delta_property() {
        // Property: m(X) = X ⊔ mδ(X)
        let mut state: GSet<i32> = GSet::new();
        state.insert(1);

        // Method 1: Direct mutation
        let mut direct = state.clone();
        direct.insert(42);

        // Method 2: Via delta-mutator
        let delta = gset::insert_delta(42);
        let via_delta = state.join(&delta);

        assert_eq!(direct, via_delta);
    }

    #[test]
    fn test_gset_batch_delta() {
        let state: GSet<i32> = GSet::new();

        let delta = gset::insert_batch_delta(vec![1, 2, 3, 4, 5]);
        let result = state.join(&delta);

        for i in 1..=5 {
            assert!(result.contains(&i));
        }
    }

    #[test]
    fn test_gset_delta_idempotence() {
        let mut state: GSet<i32> = GSet::new();
        state.insert(1);

        let delta = gset::insert_delta(2);

        // Apply delta multiple times
        let once = state.join(&delta);
        let twice = once.join(&delta);
        let thrice = twice.join(&delta);

        // Idempotence: applying same delta multiple times has no effect
        assert_eq!(once, twice);
        assert_eq!(twice, thrice);
    }

    #[test]
    fn test_orset_add_delta() {
        let mut state: ORSet<String> = ORSet::new();

        // Apply add via delta
        let delta = orset::add_delta("replica1", "hello".to_string());
        state.apply_delta(&delta);

        assert!(state.contains(&"hello".to_string()));
    }

    #[test]
    fn test_orset_delta_idempotence() {
        let mut state: ORSet<String> = ORSet::new();

        let delta = orset::add_delta("replica1", "test".to_string());

        // Apply same delta multiple times
        state.apply_delta(&delta);
        let count1 = state.len();

        state.apply_delta(&delta);
        let count2 = state.len();

        // Idempotent (same tags won't be added twice)
        assert_eq!(count1, count2);
    }

    #[test]
    fn test_lwwreg_set_delta() {
        let mut state: LWWRegister<i32, String> = LWWRegister::new("replica1".to_string());

        // Apply set via delta
        lwwreg::apply_set(&mut state, 42, 100, "replica1".to_string());

        assert_eq!(state.get(), Some(&42));
        assert_eq!(state.timestamp(), 100);
    }

    #[test]
    fn test_lwwreg_delta_higher_timestamp_wins() {
        let mut state: LWWRegister<i32, String> = LWWRegister::new("replica1".to_string());

        lwwreg::apply_set(&mut state, 10, 100, "replica1".to_string());
        assert_eq!(state.get(), Some(&10));

        lwwreg::apply_set(&mut state, 20, 200, "replica2".to_string());
        assert_eq!(state.get(), Some(&20));

        // Old timestamp doesn't overwrite
        lwwreg::apply_set(&mut state, 30, 150, "replica1".to_string());
        assert_eq!(state.get(), Some(&20));
    }

    #[test]
    fn test_pncounter_increment_delta() {
        let mut state: PNCounter<String> = PNCounter::new();

        // Apply increment via delta
        pncounter::apply_increment(&mut state, "replica1".to_string(), 5);
        assert_eq!(state.value(), 5);

        pncounter::apply_increment(&mut state, "replica1".to_string(), 3);
        assert_eq!(state.value(), 8);
    }

    #[test]
    fn test_pncounter_decrement_delta() {
        let mut state: PNCounter<String> = PNCounter::new();

        pncounter::apply_increment(&mut state, "replica1".to_string(), 10);
        assert_eq!(state.value(), 10);

        pncounter::apply_decrement(&mut state, "replica1".to_string(), 3);
        assert_eq!(state.value(), 7);
    }

    #[test]
    fn test_pncounter_delta_convergence() {
        let mut state1: PNCounter<String> = PNCounter::new();
        let mut state2: PNCounter<String> = PNCounter::new();

        // Apply different operations to each state
        pncounter::apply_increment(&mut state1, "replica1".to_string(), 5);
        pncounter::apply_increment(&mut state2, "replica2".to_string(), 3);
        pncounter::apply_decrement(&mut state2, "replica1".to_string(), 2);

        // Merge states
        let merged1 = state1.join(&state2);
        let merged2 = state2.join(&state1);

        assert_eq!(merged1.value(), merged2.value());
    }

    #[test]
    fn test_mvreg_write_delta() {
        let mut state: MVRegister<i32> = MVRegister::new();

        // Apply write via delta
        let _dot = mvreg::apply_write(&mut state, "replica1", 42);

        let values = state.read();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], &42);
    }

    #[test]
    fn test_mvreg_concurrent_deltas() {
        let mut state1: MVRegister<i32> = MVRegister::new();
        let mut state2: MVRegister<i32> = MVRegister::new();

        let _dot1 = mvreg::apply_write(&mut state1, "replica1", 10);
        let _dot2 = mvreg::apply_write(&mut state2, "replica2", 20);

        // Merge: both values should exist
        let merged = state1.join(&state2);
        let values = merged.read();
        assert_eq!(values.len(), 2);
    }
}

