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

mod compactor;
mod pruning;
mod snapshot;
mod stability;
mod version_vector;

pub use compactor::{CompactionConfig, CompactionError, CompactionStats, Compactor};
pub use pruning::{PrunableStore, Pruner, PruningPolicy, PruningResult, PruningVerifier};
pub use snapshot::{Snapshot, SnapshotError, SnapshotManager};
pub use stability::{FrontierUpdate, StabilityConfig, StabilityMonitor, StabilityState};
pub use version_vector::{VectorEntry, VersionVector};
