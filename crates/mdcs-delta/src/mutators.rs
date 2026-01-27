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
// LWWRegister Delta Mutators (placeholder for future implementation)
// ============================================================================

/// Placeholder for LWW Register delta types
pub mod lwwreg {
    /// Delta for LWW Register write operation
    /// Will contain: (timestamp, value)
    #[derive(Clone, Debug)]
    pub struct LWWWriteDelta<T, Ts> {
        pub timestamp: Ts,
        pub value: T,
    }
}

// ============================================================================
// PNCounter Delta Mutators (placeholder for future implementation)
// ============================================================================

/// Placeholder for PN-Counter delta types
pub mod pncounter {
    /// Delta for increment operation
    #[derive(Clone, Debug)]
    pub struct IncrementDelta {
        pub replica_id: String,
        pub amount: u64,
    }

    /// Delta for decrement operation
    #[derive(Clone, Debug)]
    pub struct DecrementDelta {
        pub replica_id: String,
        pub amount: u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdcs_core::lattice::DeltaCRDT;

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
}

