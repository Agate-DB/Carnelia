//! Join-semilattice trait - the mathematical foundation of CRDTs
//!
//! A join-semilattice (S, ⊔) satisfies:
//!  - Commutativity: a ⊔ b = b ⊔ a
//! - Associativity: (a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)
//! - Idempotence:  a ⊔ a = a
//!
//!  These properties guarantee convergence regardless of message order.

use std::cmp::Ordering;

/// The core CRDT trait.  All state-based CRDTs implement this.
pub trait Lattice: Clone + PartialEq {
    /// The bottom element (identity for join)
    fn bottom() -> Self;

    /// Join operation (least upper bound)
    /// Must be commutative, associative, and idempotent
    fn join(&self, other: &Self) -> Self;

    /// Partial order derived from join:  a ≤ b iff a ⊔ b = b
    fn partial_cmp_lattice(&self, other: &Self) -> Option<Ordering> {
        let joined = self.join(other);
        if &joined == self && &joined == other {
            Some(Ordering::Equal)
        } else if &joined == other {
            Some(Ordering::Less)
        } else if &joined == self {
            Some(Ordering::Greater)
        } else {
            None // Concurrent/incomparable
        }
    }

    /// Check if self ≤ other in the lattice order
    fn leq(&self, other: &Self) -> bool {
        matches!(
            self.partial_cmp_lattice(other),
            Some(Ordering:: Less) | Some(Ordering::Equal)
        )
    }

    /// Join-assign:  self = self ⊔ other
    fn join_assign(&mut self, other: &Self) {
        *self = self.join(other);
    }
}

/// Marker trait for CRDTs that support delta operations
pub trait DeltaCRDT:  Lattice {
    /// The delta state type (often the same as Self)
    type Delta:  Lattice;

    /// Split off pending deltas, returning them and resetting internal delta buffer
    fn split_delta(&mut self) -> Option<Self::Delta>;

    /// Apply a delta to the state
    fn apply_delta(&mut self, delta: &Self::Delta);
}