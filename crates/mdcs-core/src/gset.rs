//! Grow-only Set - elements can only be added, never removed
//!  This is the simplest useful CRDT and a good starting point.

use crate::lattice::Lattice;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
// use std::hash:: Hash;

/// A Grow-only Set (GSet) CRDT.
///
/// The simplest useful CRDT: elements can only be added, never removed.
/// The join operation is set union, which is commutative, associative, and
/// idempotent by definition.
///
/// # Example
///
/// ```rust
/// use mdcs_core::GSet;
/// use mdcs_core::Lattice;
///
/// let mut a = GSet::new();
/// a.insert("hello");
///
/// let mut b = GSet::new();
/// b.insert("world");
///
/// let merged = a.join(&b);
/// assert!(merged.contains(&"hello"));
/// assert!(merged.contains(&"world"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GSet<T: Ord + Clone> {
    elements: BTreeSet<T>,
}

impl<T: Ord + Clone> GSet<T> {
    /// Create a new empty GSet.
    pub fn new() -> Self {
        Self {
            elements: BTreeSet::new(),
        }
    }

    /// Add an element (the only mutation allowed)
    pub fn insert(&mut self, value: T) {
        self.elements.insert(value);
    }

    /// Check whether `value` is a member of this set.
    pub fn contains(&self, value: &T) -> bool {
        self.elements.contains(value)
    }

    /// Iterate over all elements in the set.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.elements.iter()
    }

    /// Return the number of elements in the set.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Return `true` if the set contains no elements.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

impl<T: Ord + Clone> Default for GSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + Clone> Lattice for GSet<T> {
    fn bottom() -> Self {
        Self::new()
    }

    fn join(&self, other: &Self) -> Self {
        Self {
            elements: self.elements.union(&other.elements).cloned().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Property-based tests for lattice laws
    proptest! {
        #[test]
        fn gset_join_is_commutative(
            a in prop::collection::btree_set(0i32..100, 0.. 20),
            b in prop::collection::btree_set(0i32..100, 0..20)
        ) {
            let set_a = GSet { elements: a };
            let set_b = GSet { elements: b };

            prop_assert_eq!(set_a.join(&set_b), set_b.join(&set_a));
        }

        #[test]
        fn gset_join_is_associative(
            a in prop::collection::btree_set(0i32.. 100, 0.. 10),
            b in prop::collection::btree_set(0i32..100, 0..10),
            c in prop::collection::btree_set(0i32..100, 0..10)
        ) {
            let set_a = GSet { elements:  a };
            let set_b = GSet { elements: b };
            let set_c = GSet { elements: c };

            let left = set_a. join(&set_b).join(&set_c);
            let right = set_a.join(&set_b. join(&set_c));

            prop_assert_eq!(left, right);
        }

        #[test]
        fn gset_join_is_idempotent(
            a in prop:: collection::btree_set(0i32..100, 0..20)
        ) {
            let set_a = GSet { elements: a };

            prop_assert_eq!(set_a.join(&set_a), set_a);
        }
    }
}
