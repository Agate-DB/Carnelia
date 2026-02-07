//! MDCS Delta - Delta-state CRDT machinery
//!
//! This crate implements the δ-CRDT framework including:
//! - Delta buffers for grouping and batching
//! - Delta-mutators for each CRDT type
//! - Anti-entropy Algorithm 1 (convergence mode)
//! - Anti-entropy Algorithm 2 (causal consistency mode)
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
//! Guarantees eventual convergence without causal ordering.
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
//! ## Algorithm 2: Causal Consistency Mode
//!
//! Guarantees causal ordering of delta delivery using delta-intervals.
//!
//! ```text
//! Durable state: (Xᵢ, cᵢ)  - survives crashes
//! Volatile state: (Dᵢ, Aᵢ) - lost on crash
//!
//! On local mutation m:
//!   cᵢ := cᵢ + 1
//!   d := mδ(Xᵢ)
//!   Xᵢ := Xᵢ ⊔ d
//!   ∀j: Dᵢ[j] := Dᵢ[j] ⊔ d
//!
//! On send to peer j:
//!   send ⟨Dᵢ[j], Aᵢ[j]+1, cᵢ⟩ to j
//!
//! On receive ⟨d, n, m⟩ from peer j:
//!   if n = Aᵢ[j] + 1 then   // causally ready
//!     Xᵢ := Xᵢ ⊔ d
//!     Aᵢ[j] := m
//!     send ack(m) to j
//!   else
//!     buffer for later
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

pub mod anti_entropy;
pub mod buffer;
pub mod causal;
pub mod mutators;

// Re-export main types for convenience
pub use buffer::{AckTracker, DeltaBuffer, DeltaReplica, ReplicaId, SeqNo, TaggedDelta};

pub use anti_entropy::{AntiEntropyCluster, AntiEntropyMessage, NetworkConfig, NetworkSimulator};

pub use causal::{
    CausalCluster, CausalMessage, CausalNetworkSimulator, CausalReplica, DeltaInterval,
    DurableState, DurableStorage, IntervalAck, MemoryStorage, PeerDeltaBuffer, StorageError,
    VolatileState,
};

pub use mutators::{gset as gset_mutators, orset as orset_mutators};
