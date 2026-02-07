//! # mdcs-merkle
//!
//! Merkle-Clock DAG implementation for the MDCS (Merkle-Delta CRDT Store).
//!
//! This crate provides:
//! - Content-addressed storage for causal history
//! - Merkle-DAG structure for verifiable, tamper-proof history
//! - DAGSyncer for gap-repair and synchronization
//! - Broadcaster for gossip-based head dissemination
//!
//! ## Architecture
//!
//! The Merkle-Clock serves as a logical clock that:
//! 1. Provides verifiable causal ordering via hash-linked nodes
//! 2. Enables gap-repair through content-addressed fetching
//! 3. Supports open membership without metadata bloat
//! 4. Handles concurrent updates via multi-root DAGs
//!
//! ## Example
//!
//! ```rust
//! use mdcs_merkle::{MerkleNode, Payload, MemoryDAGStore, DAGStore, NodeBuilder};
//!
//! // Create a DAG store
//! let mut store = MemoryDAGStore::new();
//!
//! // Create the genesis node
//! let genesis = NodeBuilder::new()
//!     .with_payload(Payload::genesis())
//!     .build();
//! let genesis_cid = store.put(genesis).unwrap();
//!
//! // Create a child node referencing the genesis
//! let child = NodeBuilder::new()
//!     .with_parents(vec![genesis_cid])
//!     .with_payload(Payload::delta(vec![1, 2, 3]))
//!     .build();
//! let child_cid = store.put(child).unwrap();
//!
//! // The child is now a head
//! assert_eq!(store.heads(), vec![child_cid]);
//! ```

mod broadcaster;
mod hash;
mod node;
mod store;
mod syncer;

pub use broadcaster::{BroadcastConfig, BroadcastMessage, BroadcastNetwork, Broadcaster};
pub use hash::{Hash, Hasher};
pub use node::{MerkleNode, NodeBuilder, Payload};
pub use store::{DAGError, DAGStore, MemoryDAGStore};
pub use syncer::{DAGSyncer, SyncError, SyncRequest, SyncResponse, SyncSimulator};
