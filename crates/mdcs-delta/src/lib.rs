//! MDCS Delta - Delta-state CRDT machinery
//!
//! This crate implements the δ-CRDT framework including:
//! - Delta buffers for grouping and batching
//! - Delta-mutators for each CRDT type
//! - Anti-entropy Algorithm 1 (convergence mode)
//!
//! # δ-CRDT Framework
//!
//! The δ-CRDT (delta-state CRDT) framework provides efficient synchronization
//! by only transmitting the changes (deltas) rather than full state.
//!
//! ## Key Concepts
//!
//! - **Delta-mutator**: A function `mδ` such that `m(X) = X ⊔ mδ(X)`
//! - **Delta buffer**: Accumulates deltas for batched transmission
//! - **Anti-entropy**: Protocol for eventually consistent synchronization
//!
//! ## Algorithm 1: Convergence Mode
//!
//! ```text
//! On local mutation m:
//!   d = mδ(X)     // compute delta
//!   X = X ⊔ d     // apply to local state
//!   D.push(d)     // buffer for sending
//!
//! On send to peer j:
//!   send D[acked[j]..] to j
//!
//! On receive delta d from peer i:
//!   X = X ⊔ d     // apply (idempotent!)
//!   send ack(seq) to i
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use mdcs_delta::buffer::DeltaReplica;
//! use mdcs_core::gset::GSet;
//!
//! let mut replica: DeltaReplica<GSet<i32>> = DeltaReplica::new("replica1");
//!
//! // Mutate using delta-mutator
//! replica.mutate(|_state| {
//!     let mut delta = GSet::new();
//!     delta.insert(42);
//!     delta
//! });
//!
//! assert!(replica.state().contains(&42));
//! ```

pub mod buffer;
pub mod mutators;
pub mod anti_entropy;

// Re-export main types for convenience
pub use buffer::{
    DeltaBuffer,
    DeltaReplica,
    AckTracker,
    TaggedDelta,
    SeqNo,
    ReplicaId
};

pub use anti_entropy::{
    AntiEntropyCluster,
    AntiEntropyMessage,
    NetworkSimulator,
    NetworkConfig
};

pub use mutators::{
    gset as gset_mutators,
    orset as orset_mutators,
};

