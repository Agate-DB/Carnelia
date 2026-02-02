//! Stability monitoring for safe compaction.
//!
//! The stability monitor tracks which updates have been delivered to
//! all known replicas, enabling safe pruning of the DAG history.

use crate::version_vector::VersionVector;
use mdcs_merkle::Hash;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Update about a peer's frontier.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrontierUpdate {
    /// The peer that sent this update.
    pub peer_id: String,

    /// The peer's current version vector.
    pub version_vector: VersionVector,

    /// The peer's current DAG heads.
    pub heads: Vec<Hash>,

    /// Timestamp of the update.
    pub timestamp: u64,
}

/// State of stability tracking for a single item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StabilityState {
    /// Not yet delivered to any peer.
    Pending,

    /// Delivered to some but not all peers.
    Partial {
        delivered_to: HashSet<String>,
        pending_for: HashSet<String>,
    },

    /// Delivered to all tracked peers - safe to compact.
    Stable,

    /// Unknown state (no tracking info).
    Unknown,
}

/// Monitors stability across replicas for safe compaction decisions.
///
/// Stability is achieved when an update has been delivered to all
/// tracked peers. Only stable updates can be safely compacted.
pub struct StabilityMonitor {
    /// Our replica ID.
    replica_id: String,

    /// Known peer frontiers (version vectors).
    peer_frontiers: HashMap<String, VersionVector>,

    /// Known peer heads (DAG heads).
    peer_heads: HashMap<String, Vec<Hash>>,

    /// Timestamp of last update from each peer.
    last_update: HashMap<String, u64>,

    /// Our current version vector.
    local_frontier: VersionVector,

    /// Our current DAG heads.
    local_heads: Vec<Hash>,

    /// The computed stable frontier (min of all known frontiers).
    stable_frontier: VersionVector,

    /// Configuration.
    config: StabilityConfig,
}

/// Configuration for stability monitoring.
#[derive(Clone, Debug)]
pub struct StabilityConfig {
    /// Minimum number of peers required for stability.
    pub min_peers_for_stability: usize,

    /// Maximum age of peer frontier before considered stale.
    pub max_frontier_age: u64,

    /// Whether to require all peers for stability (vs quorum).
    pub require_all_peers: bool,

    /// Quorum fraction (0.0 - 1.0) if not requiring all peers.
    pub quorum_fraction: f64,
}

impl Default for StabilityConfig {
    fn default() -> Self {
        StabilityConfig {
            min_peers_for_stability: 1,
            max_frontier_age: 10000,
            require_all_peers: true,
            quorum_fraction: 0.67,
        }
    }
}

impl StabilityMonitor {
    /// Create a new stability monitor.
    pub fn new(replica_id: impl Into<String>) -> Self {
        StabilityMonitor {
            replica_id: replica_id.into(),
            peer_frontiers: HashMap::new(),
            peer_heads: HashMap::new(),
            last_update: HashMap::new(),
            local_frontier: VersionVector::new(),
            local_heads: Vec::new(),
            stable_frontier: VersionVector::new(),
            config: StabilityConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(replica_id: impl Into<String>, config: StabilityConfig) -> Self {
        StabilityMonitor {
            replica_id: replica_id.into(),
            peer_frontiers: HashMap::new(),
            peer_heads: HashMap::new(),
            last_update: HashMap::new(),
            local_frontier: VersionVector::new(),
            local_heads: Vec::new(),
            stable_frontier: VersionVector::new(),
            config,
        }
    }

    /// Get our replica ID.
    pub fn replica_id(&self) -> &str {
        &self.replica_id
    }

    /// Update our local frontier.
    pub fn update_local_frontier(&mut self, vv: VersionVector, heads: Vec<Hash>) {
        self.local_frontier = vv;
        self.local_heads = heads;
        self.recompute_stable_frontier();
    }

    /// Update a peer's frontier.
    pub fn update_peer_frontier(&mut self, update: FrontierUpdate) {
        self.peer_frontiers
            .insert(update.peer_id.clone(), update.version_vector);
        self.peer_heads.insert(update.peer_id.clone(), update.heads);
        self.last_update
            .insert(update.peer_id.clone(), update.timestamp);
        self.recompute_stable_frontier();
    }

    /// Remove a peer from tracking.
    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peer_frontiers.remove(peer_id);
        self.peer_heads.remove(peer_id);
        self.last_update.remove(peer_id);
        self.recompute_stable_frontier();
    }

    /// Get the list of tracked peers.
    pub fn tracked_peers(&self) -> Vec<&String> {
        self.peer_frontiers.keys().collect()
    }

    /// Get the number of tracked peers.
    pub fn peer_count(&self) -> usize {
        self.peer_frontiers.len()
    }

    /// Get a peer's frontier.
    pub fn peer_frontier(&self, peer_id: &str) -> Option<&VersionVector> {
        self.peer_frontiers.get(peer_id)
    }

    /// Get the stable frontier.
    pub fn stable_frontier(&self) -> &VersionVector {
        &self.stable_frontier
    }

    /// Get the local frontier.
    pub fn local_frontier(&self) -> &VersionVector {
        &self.local_frontier
    }

    /// Check if a specific operation is stable.
    pub fn is_operation_stable(&self, replica_id: &str, sequence: u64) -> bool {
        self.stable_frontier.contains(replica_id, sequence)
    }

    /// Check if a version vector is fully stable.
    pub fn is_stable(&self, vv: &VersionVector) -> bool {
        self.stable_frontier.dominates(vv)
    }

    /// Get the stability state for a version vector.
    pub fn stability_state(&self, vv: &VersionVector) -> StabilityState {
        if self.peer_frontiers.is_empty() {
            return StabilityState::Unknown;
        }

        if self.stable_frontier.dominates(vv) {
            return StabilityState::Stable;
        }

        let mut delivered_to = HashSet::new();
        let mut pending_for = HashSet::new();

        // Check local delivery
        if self.local_frontier.dominates(vv) {
            delivered_to.insert(self.replica_id.clone());
        } else {
            pending_for.insert(self.replica_id.clone());
        }

        // Check peer delivery
        for (peer_id, frontier) in &self.peer_frontiers {
            if frontier.dominates(vv) {
                delivered_to.insert(peer_id.clone());
            } else {
                pending_for.insert(peer_id.clone());
            }
        }

        if pending_for.is_empty() {
            StabilityState::Stable
        } else if delivered_to.is_empty() {
            StabilityState::Pending
        } else {
            StabilityState::Partial {
                delivered_to,
                pending_for,
            }
        }
    }

    /// Check if we have enough peers for meaningful stability.
    pub fn has_quorum(&self) -> bool {
        let total_peers = self.peer_frontiers.len() + 1; // +1 for self

        if total_peers < self.config.min_peers_for_stability {
            return false;
        }

        if self.config.require_all_peers {
            true // All peers are tracked
        } else {
            let required = (total_peers as f64 * self.config.quorum_fraction).ceil() as usize;
            total_peers >= required
        }
    }

    /// Get stale peers (those with old frontier updates).
    pub fn stale_peers(&self, current_time: u64) -> Vec<String> {
        self.last_update
            .iter()
            .filter(|(_, &update_time)| {
                current_time.saturating_sub(update_time) > self.config.max_frontier_age
            })
            .map(|(peer_id, _)| peer_id.clone())
            .collect()
    }

    /// Remove stale peers.
    pub fn gc_stale_peers(&mut self, current_time: u64) {
        let stale: Vec<_> = self.stale_peers(current_time);
        for peer_id in stale {
            self.remove_peer(&peer_id);
        }
    }

    /// Recompute the stable frontier.
    fn recompute_stable_frontier(&mut self) {
        if self.peer_frontiers.is_empty() {
            // No peers - stable frontier is local frontier
            self.stable_frontier = self.local_frontier.clone();
            return;
        }

        // Start with local frontier
        let mut stable = self.local_frontier.clone();

        // Compute minimum with all peer frontiers
        for frontier in self.peer_frontiers.values() {
            stable = stable.min_with(frontier);
        }

        self.stable_frontier = stable;
    }

    /// Get statistics about stability.
    pub fn stats(&self) -> StabilityStats {
        let unstable_ops = self
            .local_frontier
            .total_operations()
            .saturating_sub(self.stable_frontier.total_operations());

        StabilityStats {
            peer_count: self.peer_frontiers.len(),
            local_operations: self.local_frontier.total_operations(),
            stable_operations: self.stable_frontier.total_operations(),
            unstable_operations: unstable_ops,
            has_quorum: self.has_quorum(),
        }
    }

    /// Create a frontier update message for broadcasting.
    pub fn create_frontier_update(&self, timestamp: u64) -> FrontierUpdate {
        FrontierUpdate {
            peer_id: self.replica_id.clone(),
            version_vector: self.local_frontier.clone(),
            heads: self.local_heads.clone(),
            timestamp,
        }
    }
}

/// Statistics about stability.
#[derive(Clone, Debug)]
pub struct StabilityStats {
    pub peer_count: usize,
    pub local_operations: u64,
    pub stable_operations: u64,
    pub unstable_operations: u64,
    pub has_quorum: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stability_monitor_basic() {
        let mut monitor = StabilityMonitor::new("r1");

        let local_vv = VersionVector::from_entries([("r1".to_string(), 10), ("r2".to_string(), 5)]);
        monitor.update_local_frontier(local_vv.clone(), vec![]);

        // With no peers, local frontier is stable
        assert!(monitor.is_stable(&local_vv));
    }

    #[test]
    fn test_stability_with_peers() {
        let mut monitor = StabilityMonitor::new("r1");

        // Local frontier
        let local_vv = VersionVector::from_entries([("r1".to_string(), 10), ("r2".to_string(), 5)]);
        monitor.update_local_frontier(local_vv, vec![]);

        // Peer frontier (behind local)
        let peer_vv = VersionVector::from_entries([("r1".to_string(), 7), ("r2".to_string(), 5)]);
        monitor.update_peer_frontier(FrontierUpdate {
            peer_id: "r2".to_string(),
            version_vector: peer_vv,
            heads: vec![],
            timestamp: 100,
        });

        // Operations up to (r1:7, r2:5) should be stable
        let stable_vv = VersionVector::from_entries([("r1".to_string(), 7), ("r2".to_string(), 5)]);
        assert!(monitor.is_stable(&stable_vv));

        // Operations at (r1:10, r2:5) should NOT be stable
        let unstable_vv =
            VersionVector::from_entries([("r1".to_string(), 10), ("r2".to_string(), 5)]);
        assert!(!monitor.is_stable(&unstable_vv));
    }

    #[test]
    fn test_stability_state() {
        let mut monitor = StabilityMonitor::new("r1");

        let local_vv = VersionVector::from_entries([("r1".to_string(), 10)]);
        monitor.update_local_frontier(local_vv, vec![]);

        let peer_vv = VersionVector::from_entries([("r1".to_string(), 5)]);
        monitor.update_peer_frontier(FrontierUpdate {
            peer_id: "r2".to_string(),
            version_vector: peer_vv,
            heads: vec![],
            timestamp: 100,
        });

        // Check state for operation r1:3 (stable)
        let vv1 = VersionVector::from_entries([("r1".to_string(), 3)]);
        assert_eq!(monitor.stability_state(&vv1), StabilityState::Stable);

        // Check state for operation r1:7 (partial)
        let vv2 = VersionVector::from_entries([("r1".to_string(), 7)]);
        if let StabilityState::Partial {
            delivered_to,
            pending_for,
        } = monitor.stability_state(&vv2)
        {
            assert!(delivered_to.contains("r1"));
            assert!(pending_for.contains("r2"));
        } else {
            panic!("Expected Partial state");
        }
    }

    #[test]
    fn test_stale_peer_removal() {
        let mut monitor = StabilityMonitor::new("r1");

        monitor.update_peer_frontier(FrontierUpdate {
            peer_id: "r2".to_string(),
            version_vector: VersionVector::new(),
            heads: vec![],
            timestamp: 100,
        });

        // At time 200, peer is not stale (within max_frontier_age of 10000)
        assert!(monitor.stale_peers(200).is_empty());

        // At time 20000, peer is stale
        let stale = monitor.stale_peers(20000);
        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0], "r2");

        // GC stale peers
        monitor.gc_stale_peers(20000);
        assert_eq!(monitor.peer_count(), 0);
    }

    #[test]
    fn test_quorum() {
        let config = StabilityConfig {
            min_peers_for_stability: 2,
            require_all_peers: false,
            quorum_fraction: 0.5,
            ..Default::default()
        };

        let mut monitor = StabilityMonitor::with_config("r1", config);

        // Only self - no quorum
        assert!(!monitor.has_quorum());

        // Add one peer - now have quorum (2/2 >= 0.5)
        monitor.update_peer_frontier(FrontierUpdate {
            peer_id: "r2".to_string(),
            version_vector: VersionVector::new(),
            heads: vec![],
            timestamp: 100,
        });
        assert!(monitor.has_quorum());
    }

    #[test]
    fn test_create_frontier_update() {
        let mut monitor = StabilityMonitor::new("r1");

        let vv = VersionVector::from_entries([("r1".to_string(), 10)]);
        let heads = vec![mdcs_merkle::Hasher::hash(b"head1")];
        monitor.update_local_frontier(vv.clone(), heads.clone());

        let update = monitor.create_frontier_update(100);
        assert_eq!(update.peer_id, "r1");
        assert_eq!(update.version_vector, vv);
        assert_eq!(update.heads, heads);
        assert_eq!(update.timestamp, 100);
    }
}
