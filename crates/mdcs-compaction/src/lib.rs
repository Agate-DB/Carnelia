//! # mdcs-compaction
//!
//! Compaction and stability subsystem for the MDCS (Merkle-Delta CRDT Store).
//!
//! This crate provides:
//! - Snapshotting: Serialize full CRDT state at stable frontiers
//! - DAG pruning: Remove nodes older than the last snapshot
//! - Stability monitoring: Track delivered and stable frontiers
//! - Version vectors: Compact representation of causal context
//!
//! ## Architecture
//!
//! The compaction subsystem ensures bounded metadata growth by:
//! 1. Tracking which updates have been durably replicated (stability)
//! 2. Creating periodic snapshots at stable points
//! 3. Pruning DAG history before the snapshot root
//! 4. Preventing resurrection of deleted items
//!
//! ## Example
//!
//! ```rust,ignore
//! use mdcs_compaction::{StabilityMonitor, SnapshotManager, PruningPolicy};
//!
//! // Create a stability monitor
//! let mut monitor = StabilityMonitor::new("replica_1");
//!
//! // Track frontier updates from peers
//! monitor.update_peer_frontier("replica_2", frontier_2);
//! monitor.update_peer_frontier("replica_3", frontier_3);
//!
//! // Check if a node is stable (delivered to all tracked peers)
//! if monitor.is_stable(&node_cid) {
//!     // Safe to compact
//! }
//! ```

mod version_vector;
mod snapshot;
mod pruning;
mod stability;
mod compactor;

pub use version_vector::{VersionVector, VectorEntry};
pub use snapshot::{Snapshot, SnapshotManager, SnapshotError};
pub use pruning::{PruningPolicy, PruningResult, Pruner, PrunableStore, PruningVerifier};
pub use stability::{StabilityMonitor, StabilityState, FrontierUpdate, StabilityConfig};
pub use compactor::{Compactor, CompactionConfig, CompactionStats, CompactionError};
