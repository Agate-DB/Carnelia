//! High-level compaction orchestrator.
//!
//! The Compactor coordinates snapshotting, stability monitoring, and
//! pruning to manage metadata growth over time.

use crate::pruning::{PrunableStore, Pruner, PruningPolicy, PruningResult};
use crate::snapshot::{Snapshot, SnapshotConfig, SnapshotManager};
use crate::stability::{FrontierUpdate, StabilityConfig, StabilityMonitor};
use crate::version_vector::VersionVector;
use mdcs_merkle::{DAGStore, Hash};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during compaction.
#[derive(Error, Debug)]
pub enum CompactionError {
    #[error("No stable snapshot available for compaction")]
    NoStableSnapshot,

    #[error("Stability requirements not met: {0}")]
    StabilityNotMet(String),

    #[error("Pruning failed: {0}")]
    PruningFailed(String),

    #[error("Snapshot creation failed: {0}")]
    SnapshotFailed(String),

    #[error("State serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),
}

/// Configuration for the compactor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompactionConfig {
    /// Snapshot configuration.
    #[serde(default)]
    pub snapshot: SnapshotConfigSerializable,

    /// Pruning policy.
    #[serde(default)]
    pub pruning: PruningPolicy,

    /// Stability configuration.
    #[serde(default)]
    pub stability: StabilityConfigSerializable,

    /// Whether to automatically compact when thresholds are met.
    pub auto_compact: bool,

    /// Minimum operations before considering compaction.
    pub min_ops_for_compaction: u64,

    /// Whether to verify after compaction.
    pub verify_after_compaction: bool,
}

/// Serializable version of SnapshotConfig.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapshotConfigSerializable {
    pub min_operations_between: u64,
    pub max_time_between: u64,
    pub max_snapshots: usize,
    pub auto_snapshot: bool,
}

impl Default for SnapshotConfigSerializable {
    fn default() -> Self {
        SnapshotConfigSerializable {
            min_operations_between: 1000,
            max_time_between: 10000,
            max_snapshots: 10,
            auto_snapshot: true,
        }
    }
}

impl From<SnapshotConfigSerializable> for SnapshotConfig {
    fn from(s: SnapshotConfigSerializable) -> Self {
        SnapshotConfig {
            min_operations_between: s.min_operations_between,
            max_time_between: s.max_time_between,
            max_snapshots: s.max_snapshots,
            auto_snapshot: s.auto_snapshot,
        }
    }
}

/// Serializable version of StabilityConfig.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StabilityConfigSerializable {
    pub min_peers_for_stability: usize,
    pub max_frontier_age: u64,
    pub require_all_peers: bool,
    pub quorum_fraction: f64,
}

impl Default for StabilityConfigSerializable {
    fn default() -> Self {
        StabilityConfigSerializable {
            min_peers_for_stability: 1,
            max_frontier_age: 10000,
            require_all_peers: true,
            quorum_fraction: 0.67,
        }
    }
}

impl From<StabilityConfigSerializable> for StabilityConfig {
    fn from(s: StabilityConfigSerializable) -> Self {
        StabilityConfig {
            min_peers_for_stability: s.min_peers_for_stability,
            max_frontier_age: s.max_frontier_age,
            require_all_peers: s.require_all_peers,
            quorum_fraction: s.quorum_fraction,
        }
    }
}

impl Default for CompactionConfig {
    fn default() -> Self {
        CompactionConfig {
            snapshot: SnapshotConfigSerializable::default(),
            pruning: PruningPolicy::default(),
            stability: StabilityConfigSerializable::default(),
            auto_compact: true,
            min_ops_for_compaction: 500,
            verify_after_compaction: true,
        }
    }
}

/// Statistics about compaction operations.
#[derive(Clone, Debug, Default)]
pub struct CompactionStats {
    /// Total snapshots created.
    pub snapshots_created: u64,

    /// Total nodes pruned.
    pub nodes_pruned: u64,

    /// Last compaction timestamp.
    pub last_compaction: Option<u64>,

    /// Operations since last compaction.
    pub ops_since_compaction: u64,

    /// Current DAG size (nodes).
    pub current_dag_size: usize,

    /// Current snapshot count.
    pub snapshot_count: usize,
}

/// High-level compactor that orchestrates all compaction operations.
pub struct Compactor {
    /// Our replica ID.
    replica_id: String,

    /// Configuration.
    config: CompactionConfig,

    /// Snapshot manager.
    snapshots: SnapshotManager,

    /// Stability monitor.
    stability: StabilityMonitor,

    /// Pruner.
    pruner: Pruner,

    /// Statistics.
    stats: CompactionStats,

    /// Current logical time.
    current_time: u64,
}

impl Compactor {
    /// Create a new compactor.
    pub fn new(replica_id: impl Into<String>) -> Self {
        let replica_id = replica_id.into();

        Compactor {
            snapshots: SnapshotManager::new(),
            stability: StabilityMonitor::new(&replica_id),
            pruner: Pruner::new(),
            config: CompactionConfig::default(),
            stats: CompactionStats::default(),
            current_time: 0,
            replica_id,
        }
    }

    /// Create a compactor with custom configuration.
    pub fn with_config(replica_id: impl Into<String>, config: CompactionConfig) -> Self {
        let replica_id = replica_id.into();

        let snapshot_config: SnapshotConfig = config.snapshot.clone().into();
        let stability_config: StabilityConfig = config.stability.clone().into();

        Compactor {
            snapshots: SnapshotManager::with_config(snapshot_config),
            stability: StabilityMonitor::with_config(&replica_id, stability_config),
            pruner: Pruner::with_policy(config.pruning.clone()),
            config,
            stats: CompactionStats::default(),
            current_time: 0,
            replica_id,
        }
    }

    /// Get the replica ID.
    pub fn replica_id(&self) -> &str {
        &self.replica_id
    }

    /// Get the configuration.
    pub fn config(&self) -> &CompactionConfig {
        &self.config
    }

    /// Get the snapshot manager.
    pub fn snapshots(&self) -> &SnapshotManager {
        &self.snapshots
    }

    /// Get mutable snapshot manager.
    pub fn snapshots_mut(&mut self) -> &mut SnapshotManager {
        &mut self.snapshots
    }

    /// Get the stability monitor.
    pub fn stability(&self) -> &StabilityMonitor {
        &self.stability
    }

    /// Get mutable stability monitor.
    pub fn stability_mut(&mut self) -> &mut StabilityMonitor {
        &mut self.stability
    }

    /// Get the pruner.
    pub fn pruner(&self) -> &Pruner {
        &self.pruner
    }

    /// Get mutable pruner.
    pub fn pruner_mut(&mut self) -> &mut Pruner {
        &mut self.pruner
    }

    /// Get statistics.
    pub fn stats(&self) -> &CompactionStats {
        &self.stats
    }

    /// Update the current time.
    pub fn set_time(&mut self, time: u64) {
        self.current_time = time;
    }

    /// Update local frontier (call after state changes).
    pub fn update_local_frontier(&mut self, vv: VersionVector, heads: Vec<Hash>) {
        self.stability.update_local_frontier(vv, heads);
    }

    /// Process a frontier update from a peer.
    pub fn process_peer_update(&mut self, update: FrontierUpdate) {
        self.stability.update_peer_frontier(update);
    }

    /// Create a frontier update for broadcasting.
    pub fn create_frontier_update(&self) -> FrontierUpdate {
        self.stability.create_frontier_update(self.current_time)
    }

    /// Check if a snapshot should be created.
    pub fn should_snapshot(&self) -> bool {
        self.snapshots
            .should_snapshot(self.stability.local_frontier(), self.current_time)
    }

    /// Create a snapshot from the current state.
    ///
    /// The `state_serializer` function should serialize the current CRDT state.
    pub fn create_snapshot<F>(
        &mut self,
        superseded_roots: Vec<Hash>,
        state_serializer: F,
    ) -> Result<Hash, CompactionError>
    where
        F: FnOnce() -> Result<Vec<u8>, String>,
    {
        let state_data = state_serializer().map_err(|e| CompactionError::SerializationFailed(e))?;

        let snapshot = Snapshot::new(
            self.stability.local_frontier().clone(),
            superseded_roots,
            state_data,
            &self.replica_id,
            self.current_time,
        );

        let id = self.snapshots.store(snapshot);
        self.stats.snapshots_created += 1;
        self.stats.snapshot_count = self.snapshots.stats().count;

        Ok(id)
    }

    /// Check if compaction should be performed.
    pub fn should_compact<S: DAGStore>(&self, _store: &S) -> bool {
        if !self.config.auto_compact {
            return false;
        }

        // Need at least min_ops_for_compaction operations
        if self.stability.local_frontier().total_operations() < self.config.min_ops_for_compaction {
            return false;
        }

        // Need at least min_snapshots_before_prune snapshots
        if self.snapshots.stats().count < self.config.pruning.min_snapshots_before_prune {
            return false;
        }

        // Need a stable snapshot
        if let Some(snapshot) = self.snapshots.latest() {
            self.stability.is_stable(&snapshot.version_vector)
        } else {
            false
        }
    }

    /// Perform compaction (snapshot + prune if needed).
    pub fn compact<S, F>(
        &mut self,
        store: &mut S,
        state_serializer: F,
    ) -> Result<CompactionResult, CompactionError>
    where
        S: DAGStore + PrunableStore,
        F: FnOnce() -> Result<Vec<u8>, String>,
    {
        let mut result = CompactionResult::default();

        // Create snapshot if needed
        if self.should_snapshot() {
            let superseded = store.heads();
            let snapshot_id = self.create_snapshot(superseded, state_serializer)?;
            result.snapshot_created = Some(snapshot_id);
        }

        // Prune if we have a stable snapshot
        if let Some(snapshot) = self.snapshots.latest() {
            if self.stability.is_stable(&snapshot.version_vector) {
                let prune_result = self
                    .pruner
                    .execute_prune(store, snapshot, self.current_time);
                result.nodes_pruned = prune_result.nodes_pruned;
                result.pruning_result = Some(prune_result);
                self.stats.nodes_pruned += result.nodes_pruned as u64;
            }
        }

        // Verify if configured
        if self.config.verify_after_compaction {
            crate::pruning::PruningVerifier::verify_connectivity(store)
                .map_err(|e| CompactionError::VerificationFailed(e))?;
        }

        self.stats.last_compaction = Some(self.current_time);
        self.stats.current_dag_size = store.len();

        Ok(result)
    }

    /// Perform automatic maintenance (GC stale peers, auto-compact if needed).
    pub fn tick<S, F>(
        &mut self,
        store: &mut S,
        state_serializer: F,
        time: u64,
    ) -> Result<Option<CompactionResult>, CompactionError>
    where
        S: DAGStore + PrunableStore,
        F: FnOnce() -> Result<Vec<u8>, String>,
    {
        self.current_time = time;

        // GC stale peers
        self.stability.gc_stale_peers(time);

        // Auto-compact if needed
        if self.should_compact(store) {
            let result = self.compact(store, state_serializer)?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// Bootstrap from a snapshot.
    ///
    /// Returns the deserialized state data and the version vector.
    pub fn bootstrap_from_snapshot(
        &mut self,
        snapshot: Snapshot,
    ) -> Result<(Vec<u8>, VersionVector), CompactionError> {
        let state_data = snapshot.state_data.clone();
        let vv = snapshot.version_vector.clone();

        // Store the snapshot
        self.snapshots.store(snapshot);

        Ok((state_data, vv))
    }

    /// Get the best snapshot for bootstrapping a new replica.
    pub fn get_bootstrap_snapshot(&self) -> Option<&Snapshot> {
        self.snapshots.latest()
    }
}

/// Result of a compaction operation.
#[derive(Clone, Debug, Default)]
pub struct CompactionResult {
    /// ID of snapshot created, if any.
    pub snapshot_created: Option<Hash>,

    /// Number of nodes pruned.
    pub nodes_pruned: usize,

    /// Detailed pruning result.
    pub pruning_result: Option<PruningResult>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdcs_merkle::MemoryDAGStore;

    #[test]
    fn test_compactor_creation() {
        let compactor = Compactor::new("test_replica");

        assert_eq!(compactor.replica_id(), "test_replica");
        assert_eq!(compactor.stats().snapshots_created, 0);
    }

    #[test]
    fn test_compactor_with_config() {
        let config = CompactionConfig {
            auto_compact: false,
            min_ops_for_compaction: 1000,
            ..Default::default()
        };

        let compactor = Compactor::with_config("test", config);
        assert!(!compactor.config().auto_compact);
        assert_eq!(compactor.config().min_ops_for_compaction, 1000);
    }

    #[test]
    fn test_update_local_frontier() {
        let mut compactor = Compactor::new("test");

        let vv = VersionVector::from_entries([("test".to_string(), 10)]);
        let heads = vec![mdcs_merkle::Hasher::hash(b"head")];

        compactor.update_local_frontier(vv.clone(), heads);

        assert_eq!(compactor.stability().local_frontier(), &vv);
    }

    #[test]
    fn test_create_snapshot() {
        let mut compactor = Compactor::new("test");

        let vv = VersionVector::from_entries([("test".to_string(), 10)]);
        compactor.update_local_frontier(vv, vec![]);

        let result = compactor.create_snapshot(vec![], || Ok(b"test state".to_vec()));

        assert!(result.is_ok());
        assert_eq!(compactor.stats().snapshots_created, 1);
    }

    #[test]
    fn test_frontier_update_roundtrip() {
        let mut compactor1 = Compactor::new("r1");
        let mut compactor2 = Compactor::new("r2");

        // Update r1's frontier
        let vv = VersionVector::from_entries([("r1".to_string(), 10)]);
        compactor1.update_local_frontier(vv.clone(), vec![]);
        compactor1.set_time(100);

        // Create update and send to r2
        let update = compactor1.create_frontier_update();
        compactor2.process_peer_update(update);

        // r2 should now know about r1's frontier
        assert!(compactor2.stability().peer_frontier("r1").is_some());
    }

    #[test]
    fn test_should_compact() {
        let config = CompactionConfig {
            auto_compact: true,
            min_ops_for_compaction: 5,
            ..Default::default()
        };
        let mut compactor = Compactor::with_config("test", config);

        let (store, _) = MemoryDAGStore::with_genesis("test");

        // Not enough operations yet
        let vv = VersionVector::from_entries([("test".to_string(), 3)]);
        compactor.update_local_frontier(vv, vec![]);
        assert!(!compactor.should_compact(&store));

        // Add enough operations
        let vv2 = VersionVector::from_entries([("test".to_string(), 10)]);
        compactor.update_local_frontier(vv2, vec![]);

        // Still need snapshots
        assert!(!compactor.should_compact(&store));
    }

    #[test]
    fn test_bootstrap_from_snapshot() {
        let mut compactor = Compactor::new("new_replica");

        let vv = VersionVector::from_entries([("origin".to_string(), 100)]);
        let snapshot = Snapshot::new(vv.clone(), vec![], b"state data".to_vec(), "origin", 1000);

        let (state_data, recovered_vv) = compactor.bootstrap_from_snapshot(snapshot).unwrap();

        assert_eq!(state_data, b"state data");
        assert_eq!(recovered_vv, vv);
        assert_eq!(compactor.snapshots().stats().count, 1);
    }

    #[test]
    fn test_compaction_stats() {
        let mut compactor = Compactor::new("test");

        let vv = VersionVector::from_entries([("test".to_string(), 10)]);
        compactor.update_local_frontier(vv.clone(), vec![]);

        compactor
            .create_snapshot(vec![], || Ok(b"state1".to_vec()))
            .unwrap();

        // Advance time to get a different snapshot ID
        compactor.set_time(100);

        // Update frontier so second snapshot is different
        let vv2 = VersionVector::from_entries([("test".to_string(), 20)]);
        compactor.update_local_frontier(vv2, vec![]);

        compactor
            .create_snapshot(vec![], || Ok(b"state2".to_vec()))
            .unwrap();

        let stats = compactor.stats();
        assert_eq!(stats.snapshots_created, 2);
        assert_eq!(stats.snapshot_count, 2);
    }
}
