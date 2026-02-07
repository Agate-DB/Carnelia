//! # mdcs-core
//!
//! Core CRDT types and traits for the **Carnelia** Merkle-Delta CRDT Store.
//!
//! This crate provides the foundational building blocks for conflict-free
//! replicated data types (CRDTs). Every type implements the [`Lattice`] trait,
//! which models a join-semilattice and guarantees **Strong Eventual Consistency**:
//! all replicas converge to the same state regardless of message order.
//!
//! ## Mathematical Foundation
//!
//! A join-semilattice $(S, \sqcup)$ satisfies three properties:
//!
//! | Property | Definition |
//! |---|---|
//! | **Commutativity** | $a \sqcup b = b \sqcup a$ |
//! | **Associativity** | $(a \sqcup b) \sqcup c = a \sqcup (b \sqcup c)$ |
//! | **Idempotence** | $a \sqcup a = a$ |
//!
//! These guarantees mean that updates can be applied in **any order**, **any
//! number of times**, and the result is always deterministic.
//!
//! ## CRDT Types
//!
//! | Type | Module | Description |
//! |---|---|---|
//! | [`GSet`] | [`gset`] | Grow-only set — elements can only be added |
//! | [`ORSet`] | [`orset`] | Observed-Remove set — add-wins semantics |
//! | [`PNCounter`] | [`pncounter`] | Increment/decrement counter |
//! | [`LWWRegister`] | [`lwwreg`] | Last-Writer-Wins register |
//! | [`MVRegister`] | [`mvreg`] | Multi-Value register — preserves concurrent writes |
//! | [`CRDTMap`] | [`map`] | Composable map with shared causal context |
//!
//! ## Quick Start
//!
//! ```rust
//! use mdcs_core::prelude::*;
//!
//! // Create two replicas of a grow-only set
//! let mut a = GSet::new();
//! let mut b = GSet::new();
//!
//! a.insert(1);
//! b.insert(2);
//!
//! // Merge — order doesn't matter
//! let merged = a.join(&b);
//! assert!(merged.contains(&1));
//! assert!(merged.contains(&2));
//! ```
//!
//! ## Feature: Delta-State Support
//!
//! Types that implement [`DeltaCRDT`] support efficient delta-state
//! replication. Instead of shipping full state, only incremental changes
//! (deltas) are transmitted. See the [`mdcs-delta`](https://docs.rs/mdcs-delta)
//! crate for the anti-entropy protocol that drives synchronization.

pub mod gset;
pub mod lattice;
pub mod lwwreg;
pub mod map;
pub mod mvreg;
pub mod orset;
pub mod pncounter;

// Re-exports for convenience
pub use gset::GSet;
pub use lattice::{DeltaCRDT, Lattice};
pub use lwwreg::LWWRegister;
pub use map::{CRDTMap, CausalContext, MapValue};
pub use mvreg::MVRegister;
pub use orset::ORSet;
pub use pncounter::PNCounter;

/// Prelude module — import everything you need with `use mdcs_core::prelude::*`.
pub mod prelude {
    pub use crate::gset::GSet;
    pub use crate::lattice::{DeltaCRDT, Lattice};
    pub use crate::lwwreg::LWWRegister;
    pub use crate::map::{CRDTMap, CausalContext, MapValue};
    pub use crate::mvreg::MVRegister;
    pub use crate::orset::ORSet;
    pub use crate::pncounter::PNCounter;
}
