//! Snapshot management for CRDT state persistence.
//!
//! Snapshots capture the full state of a CRDT at a stable point,
//! allowing for efficient bootstrapping and DAG pruning.

use crate::version_vector::VersionVector;
use mdcs_merkle::{Hash, Hasher, MerkleNode, NodeBuilder, Payload};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during snapshot operations.
#[derive(Error, Debug)]
pub enum SnapshotError {
    #[error("Snapshot not found: {0}")]
    NotFound(String),

    #[error("Invalid snapshot data: {0}")]
    InvalidData(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u8, actual: u8 },

    #[error("Snapshot too old: {0}")]
    TooOld(String),
}

/// Current snapshot format version.
pub const SNAPSHOT_VERSION: u8 = 1;

/// A snapshot of CRDT state at a specific point in causal history.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    /// Format version for compatibility.
    pub version: u8,

    /// Unique identifier for this snapshot.
    pub id: Hash,

    /// The version vector at the time of the snapshot.
    /// This represents the causal frontier.
    pub version_vector: VersionVector,

    /// The CIDs of DAG nodes that this snapshot supersedes.
    /// These can be safely pruned after the snapshot is stable.
    pub superseded_roots: Vec<Hash>,

    /// The serialized CRDT state.
    pub state_data: Vec<u8>,

    /// Timestamp when the snapshot was created.
    pub created_at: u64,

    /// The replica that created this snapshot.
    pub creator: String,

    /// Optional metadata about the snapshot.
    pub metadata: HashMap<String, String>,
}

impl Snapshot {
    /// Create a new snapshot from serialized state.
    pub fn new(
        version_vector: VersionVector,
        superseded_roots: Vec<Hash>,
        state_data: Vec<u8>,
        creator: impl Into<String>,
        created_at: u64,
    ) -> Self {
        let creator = creator.into();

        // Compute snapshot ID from contents
        let mut hasher = Hasher::new();
        hasher.update(&[SNAPSHOT_VERSION]);
        hasher.update(&state_data);
        for entry in version_vector.to_entries() {
            hasher.update(entry.replica_id.as_bytes());
            hasher.update(&entry.sequence.to_le_bytes());
        }
        hasher.update(&created_at.to_le_bytes());
        hasher.update(creator.as_bytes());
        let id = hasher.finalize();

        Snapshot {
            version: SNAPSHOT_VERSION,
            id,
            version_vector,
            superseded_roots,
            state_data,
            created_at,
            creator,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the snapshot.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Convert this snapshot to a MerkleNode for storage in the DAG.
    pub fn to_merkle_node(&self) -> Result<MerkleNode, SnapshotError> {
        let payload_data = serde_json::to_vec(self)
            .map_err(|e| SnapshotError::SerializationError(e.to_string()))?;

        Ok(NodeBuilder::new()
            .with_parents(self.superseded_roots.clone())
            .with_payload(Payload::snapshot(payload_data))
            .with_timestamp(self.created_at)
            .with_creator(&self.creator)
            .build())
    }

    /// Deserialize a snapshot from a MerkleNode payload.
    pub fn from_merkle_node(node: &MerkleNode) -> Result<Self, SnapshotError> {
        match &node.payload {
            Payload::Snapshot(data) => {
                let snapshot: Snapshot = serde_json::from_slice(data)
                    .map_err(|e| SnapshotError::SerializationError(e.to_string()))?;

                if snapshot.version != SNAPSHOT_VERSION {
                    return Err(SnapshotError::VersionMismatch {
                        expected: SNAPSHOT_VERSION,
                        actual: snapshot.version,
                    });
                }

                Ok(snapshot)
            }
            _ => Err(SnapshotError::InvalidData(
                "Node does not contain snapshot payload".to_string(),
            )),
        }
    }

    /// Check if this snapshot covers a given version vector.
    pub fn covers(&self, vv: &VersionVector) -> bool {
        self.version_vector.dominates(vv)
    }

    /// Get the total size of the snapshot in bytes.
    pub fn size(&self) -> usize {
        self.state_data.len()
    }
}

/// Manages snapshot creation and retrieval.
pub struct SnapshotManager {
    /// All known snapshots, indexed by ID.
    snapshots: HashMap<Hash, Snapshot>,

    /// Snapshots indexed by creator.
    by_creator: HashMap<String, Vec<Hash>>,

    /// The latest snapshot ID.
    latest: Option<Hash>,

    /// Configuration for snapshot creation.
    config: SnapshotConfig,
}

/// Configuration for snapshot management.
#[derive(Clone, Debug)]
pub struct SnapshotConfig {
    /// Minimum operations between snapshots.
    pub min_operations_between: u64,

    /// Maximum time between snapshots (in logical time units).
    pub max_time_between: u64,

    /// Maximum number of snapshots to retain.
    pub max_snapshots: usize,

    /// Whether to automatically create snapshots.
    pub auto_snapshot: bool,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        SnapshotConfig {
            min_operations_between: 1000,
            max_time_between: 10000,
            max_snapshots: 10,
            auto_snapshot: true,
        }
    }
}

impl SnapshotManager {
    /// Create a new snapshot manager.
    pub fn new() -> Self {
        SnapshotManager {
            snapshots: HashMap::new(),
            by_creator: HashMap::new(),
            latest: None,
            config: SnapshotConfig::default(),
        }
    }

    /// Create a snapshot manager with custom configuration.
    pub fn with_config(config: SnapshotConfig) -> Self {
        SnapshotManager {
            snapshots: HashMap::new(),
            by_creator: HashMap::new(),
            latest: None,
            config,
        }
    }

    /// Get the configuration.
    pub fn config(&self) -> &SnapshotConfig {
        &self.config
    }

    /// Store a new snapshot.
    pub fn store(&mut self, snapshot: Snapshot) -> Hash {
        let id = snapshot.id;

        self.by_creator
            .entry(snapshot.creator.clone())
            .or_default()
            .push(id);

        // Update latest if this is newer
        if let Some(latest_id) = self.latest {
            if let Some(latest) = self.snapshots.get(&latest_id) {
                if snapshot.version_vector.dominates(&latest.version_vector) {
                    self.latest = Some(id);
                }
            }
        } else {
            self.latest = Some(id);
        }

        self.snapshots.insert(id, snapshot);

        // Enforce max snapshots limit
        self.gc_old_snapshots();

        id
    }

    /// Get a snapshot by ID.
    pub fn get(&self, id: &Hash) -> Option<&Snapshot> {
        self.snapshots.get(id)
    }

    /// Get the latest snapshot.
    pub fn latest(&self) -> Option<&Snapshot> {
        self.latest.and_then(|id| self.snapshots.get(&id))
    }

    /// Get the latest snapshot ID.
    pub fn latest_id(&self) -> Option<Hash> {
        self.latest
    }

    /// Get all snapshots by a specific creator.
    pub fn by_creator(&self, creator: &str) -> Vec<&Snapshot> {
        self.by_creator
            .get(creator)
            .map(|ids| ids.iter().filter_map(|id| self.snapshots.get(id)).collect())
            .unwrap_or_default()
    }

    /// Find the best snapshot that covers a given version vector.
    pub fn find_covering(&self, vv: &VersionVector) -> Option<&Snapshot> {
        self.snapshots
            .values()
            .filter(|s| s.covers(vv))
            .max_by_key(|s| s.version_vector.total_operations())
    }

    /// Check if a new snapshot should be created based on configuration.
    pub fn should_snapshot(&self, current_vv: &VersionVector, current_time: u64) -> bool {
        if !self.config.auto_snapshot {
            return false;
        }

        match self.latest() {
            None => true, // No snapshots yet
            Some(latest) => {
                let ops_since =
                    current_vv.total_operations() - latest.version_vector.total_operations();
                let time_since = current_time.saturating_sub(latest.created_at);

                ops_since >= self.config.min_operations_between
                    || time_since >= self.config.max_time_between
            }
        }
    }

    /// Remove old snapshots to stay within limits.
    fn gc_old_snapshots(&mut self) {
        while self.snapshots.len() > self.config.max_snapshots {
            // Find oldest snapshot that isn't the latest
            let oldest = self
                .snapshots
                .iter()
                .filter(|(id, _)| Some(**id) != self.latest)
                .min_by_key(|(_, s)| s.created_at)
                .map(|(id, _)| *id);

            if let Some(id) = oldest {
                if let Some(snapshot) = self.snapshots.remove(&id) {
                    if let Some(creator_snapshots) = self.by_creator.get_mut(&snapshot.creator) {
                        creator_snapshots.retain(|&sid| sid != id);
                    }
                }
            } else {
                break;
            }
        }
    }

    /// Get statistics about managed snapshots.
    pub fn stats(&self) -> SnapshotStats {
        let total_size: usize = self.snapshots.values().map(|s| s.size()).sum();
        let oldest = self.snapshots.values().map(|s| s.created_at).min();
        let newest = self.snapshots.values().map(|s| s.created_at).max();

        SnapshotStats {
            count: self.snapshots.len(),
            total_size,
            oldest_timestamp: oldest,
            newest_timestamp: newest,
        }
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about managed snapshots.
#[derive(Clone, Debug)]
pub struct SnapshotStats {
    pub count: usize,
    pub total_size: usize,
    pub oldest_timestamp: Option<u64>,
    pub newest_timestamp: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let vv = VersionVector::from_entries([("r1".to_string(), 10), ("r2".to_string(), 5)]);
        let state_data = b"test state data".to_vec();
        let roots = vec![Hasher::hash(b"root1")];

        let snapshot = Snapshot::new(vv.clone(), roots.clone(), state_data.clone(), "r1", 100);

        assert_eq!(snapshot.version, SNAPSHOT_VERSION);
        assert_eq!(snapshot.version_vector, vv);
        assert_eq!(snapshot.state_data, state_data);
        assert_eq!(snapshot.created_at, 100);
        assert_eq!(snapshot.creator, "r1");
    }

    #[test]
    fn test_snapshot_covers() {
        let vv1 = VersionVector::from_entries([("r1".to_string(), 10), ("r2".to_string(), 5)]);
        let vv2 = VersionVector::from_entries([("r1".to_string(), 5), ("r2".to_string(), 3)]);
        let vv3 = VersionVector::from_entries([("r1".to_string(), 15), ("r2".to_string(), 5)]);

        let snapshot = Snapshot::new(vv1, vec![], vec![], "r1", 100);

        assert!(snapshot.covers(&vv2));
        assert!(!snapshot.covers(&vv3));
    }

    #[test]
    fn test_snapshot_to_merkle_node() {
        let vv = VersionVector::from_entries([("r1".to_string(), 10)]);
        let snapshot = Snapshot::new(vv, vec![], b"data".to_vec(), "r1", 100);

        let node = snapshot.to_merkle_node().unwrap();
        assert!(matches!(node.payload, Payload::Snapshot(_)));

        let recovered = Snapshot::from_merkle_node(&node).unwrap();
        assert_eq!(recovered.id, snapshot.id);
        assert_eq!(recovered.version_vector, snapshot.version_vector);
    }

    #[test]
    fn test_snapshot_manager_store_and_get() {
        let mut manager = SnapshotManager::new();

        let vv = VersionVector::from_entries([("r1".to_string(), 10)]);
        let snapshot = Snapshot::new(vv, vec![], b"data".to_vec(), "r1", 100);
        let id = snapshot.id;

        manager.store(snapshot);

        assert!(manager.get(&id).is_some());
        assert!(manager.latest().is_some());
        assert_eq!(manager.latest_id(), Some(id));
    }

    #[test]
    fn test_snapshot_manager_gc() {
        let config = SnapshotConfig {
            max_snapshots: 3,
            ..Default::default()
        };
        let mut manager = SnapshotManager::with_config(config);

        // Add 5 snapshots
        for i in 0..5 {
            let vv = VersionVector::from_entries([("r1".to_string(), i as u64 + 1)]);
            let snapshot = Snapshot::new(vv, vec![], b"data".to_vec(), "r1", i as u64);
            manager.store(snapshot);
        }

        // Should only have 3 snapshots
        assert_eq!(manager.snapshots.len(), 3);

        // Latest should still exist
        assert!(manager.latest().is_some());
    }

    #[test]
    fn test_should_snapshot() {
        let config = SnapshotConfig {
            min_operations_between: 100,
            max_time_between: 1000,
            auto_snapshot: true,
            ..Default::default()
        };
        let mut manager = SnapshotManager::with_config(config);

        // No snapshots yet - should snapshot
        let vv = VersionVector::from_entries([("r1".to_string(), 10)]);
        assert!(manager.should_snapshot(&vv, 100));

        // Add a snapshot
        let snapshot = Snapshot::new(vv.clone(), vec![], b"data".to_vec(), "r1", 100);
        manager.store(snapshot);

        // Not enough operations - shouldn't snapshot
        let vv2 = VersionVector::from_entries([("r1".to_string(), 50)]);
        assert!(!manager.should_snapshot(&vv2, 200));

        // Enough operations - should snapshot
        let vv3 = VersionVector::from_entries([("r1".to_string(), 150)]);
        assert!(manager.should_snapshot(&vv3, 200));
    }
}
