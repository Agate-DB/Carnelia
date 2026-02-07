//! DAG pruning for bounded history growth.
//!
//! Pruning removes nodes from the Merkle-DAG that are older than
//! the last stable snapshot, reducing storage requirements.

use crate::snapshot::Snapshot;
use crate::stability::StabilityMonitor;
use crate::version_vector::VersionVector;
use mdcs_merkle::{DAGStore, Hash};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Policy for DAG pruning decisions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PruningPolicy {
    /// Minimum number of snapshots to retain before pruning.
    pub min_snapshots_before_prune: usize,

    /// Minimum age of nodes (in logical time) before they can be pruned.
    pub min_node_age: u64,

    /// Maximum number of nodes to prune in one operation.
    pub max_nodes_per_prune: usize,

    /// Whether to require stability before pruning.
    pub require_stability: bool,

    /// Whether to keep at least one path to genesis.
    pub preserve_genesis_path: bool,

    /// Depth of history to preserve beyond the snapshot.
    pub preserve_depth: usize,
}

impl Default for PruningPolicy {
    fn default() -> Self {
        PruningPolicy {
            min_snapshots_before_prune: 2,
            min_node_age: 5000,
            max_nodes_per_prune: 1000,
            require_stability: true,
            preserve_genesis_path: true,
            preserve_depth: 10,
        }
    }
}

/// Result of a pruning operation.
#[derive(Clone, Debug)]
pub struct PruningResult {
    /// Number of nodes pruned.
    pub nodes_pruned: usize,

    /// CIDs of pruned nodes.
    pub pruned_cids: Vec<Hash>,

    /// The snapshot root used as the pruning boundary.
    pub snapshot_root: Option<Hash>,

    /// Nodes that couldn't be pruned (and why).
    pub skipped: Vec<(Hash, String)>,

    /// Whether pruning completed fully or was limited.
    pub completed: bool,
}

impl PruningResult {
    /// Create an empty result (nothing pruned).
    pub fn empty() -> Self {
        PruningResult {
            nodes_pruned: 0,
            pruned_cids: Vec::new(),
            snapshot_root: None,
            skipped: Vec::new(),
            completed: true,
        }
    }
}

/// Pruner for removing old DAG nodes.
pub struct Pruner {
    /// Pruning policy.
    policy: PruningPolicy,

    /// Set of CIDs that must be preserved (e.g., recent snapshots).
    preserved: HashSet<Hash>,

    /// The stable frontier at the time of pruning.
    stable_frontier: Option<VersionVector>,
}

impl Pruner {
    /// Create a new pruner with default policy.
    pub fn new() -> Self {
        Pruner {
            policy: PruningPolicy::default(),
            preserved: HashSet::new(),
            stable_frontier: None,
        }
    }

    /// Create a pruner with custom policy.
    pub fn with_policy(policy: PruningPolicy) -> Self {
        Pruner {
            policy,
            preserved: HashSet::new(),
            stable_frontier: None,
        }
    }

    /// Get the current policy.
    pub fn policy(&self) -> &PruningPolicy {
        &self.policy
    }

    /// Set the stable frontier for pruning decisions.
    pub fn set_stable_frontier(&mut self, frontier: VersionVector) {
        self.stable_frontier = Some(frontier);
    }

    /// Mark a CID as preserved (cannot be pruned).
    pub fn preserve(&mut self, cid: Hash) {
        self.preserved.insert(cid);
    }

    /// Clear preserved CIDs.
    pub fn clear_preserved(&mut self) {
        self.preserved.clear();
    }

    /// Identify nodes that can be safely pruned.
    ///
    /// This doesn't actually modify the store - it returns the list of
    /// nodes that would be pruned if `execute_prune` is called.
    pub fn identify_prunable<S: DAGStore>(
        &self,
        store: &S,
        snapshot: &Snapshot,
        current_time: u64,
    ) -> Vec<Hash> {
        let mut prunable = Vec::new();

        // Get all nodes in topological order (oldest first)
        let all_nodes = store.topological_order();

        // Find ancestors of the snapshot's superseded roots (these are candidates for pruning)
        // We use superseded_roots because snapshot.id is a hash of the snapshot content,
        // not a node in the DAG. The superseded_roots are the actual DAG heads that
        // the snapshot was created from.
        let mut snapshot_ancestors: HashSet<_> = HashSet::new();
        for root in &snapshot.superseded_roots {
            // Include the root itself and all its ancestors
            snapshot_ancestors.insert(*root);
            snapshot_ancestors.extend(store.ancestors(root));
        }

        // Find nodes to preserve (heads and their recent ancestors)
        let mut preserved = self.preserved.clone();

        // Preserve current heads
        for head in store.heads() {
            preserved.insert(head);
        }

        // Preserve nodes within preserve_depth of heads
        for head in store.heads() {
            let ancestors = self.ancestors_within_depth(store, &head, self.policy.preserve_depth);
            preserved.extend(ancestors);
        }

        // Preserve the snapshot's superseded roots (they are the snapshot boundary nodes)
        for root in &snapshot.superseded_roots {
            preserved.insert(*root);
        }

        // If preserving genesis path, mark it
        if self.policy.preserve_genesis_path {
            if let Some(genesis_path) = self.find_genesis_path(store) {
                preserved.extend(genesis_path);
            }
        }

        for cid in all_nodes {
            // Skip if already at limit
            if prunable.len() >= self.policy.max_nodes_per_prune {
                break;
            }

            // Skip preserved nodes
            if preserved.contains(&cid) {
                continue;
            }

            // Skip if not an ancestor of the snapshot
            if !snapshot_ancestors.contains(&cid) {
                continue;
            }

            // Skip if not old enough
            if let Some(node) = store.get(&cid) {
                if current_time.saturating_sub(node.timestamp) < self.policy.min_node_age {
                    continue;
                }
            }

            prunable.push(cid);
        }

        prunable
    }

    /// Execute pruning on a mutable store.
    ///
    /// Returns the result of the pruning operation.
    pub fn execute_prune<S: DAGStore + PrunableStore>(
        &self,
        store: &mut S,
        snapshot: &Snapshot,
        current_time: u64,
    ) -> PruningResult {
        let prunable = self.identify_prunable(store, snapshot, current_time);

        if prunable.is_empty() {
            return PruningResult::empty();
        }

        let mut result = PruningResult {
            nodes_pruned: 0,
            pruned_cids: Vec::new(),
            snapshot_root: Some(snapshot.id),
            skipped: Vec::new(),
            completed: true,
        };

        for cid in prunable {
            match store.remove(&cid) {
                Ok(()) => {
                    result.nodes_pruned += 1;
                    result.pruned_cids.push(cid);
                }
                Err(e) => {
                    result.skipped.push((cid, e));
                }
            }
        }

        if !result.skipped.is_empty() {
            result.completed = false;
        }

        result
    }

    /// Check if pruning should be performed.
    pub fn should_prune<S: DAGStore>(
        &self,
        store: &S,
        snapshot: &Snapshot,
        stability_monitor: Option<&StabilityMonitor>,
    ) -> bool {
        // Check stability requirement
        if self.policy.require_stability {
            if let Some(monitor) = stability_monitor {
                if !monitor.is_stable(&snapshot.version_vector) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check if there are any prunable nodes
        let node_count = store.topological_order().len();
        node_count > self.policy.preserve_depth + 1
    }

    /// Get ancestors within a certain depth.
    fn ancestors_within_depth<S: DAGStore>(
        &self,
        store: &S,
        cid: &Hash,
        depth: usize,
    ) -> HashSet<Hash> {
        let mut result = HashSet::new();
        let mut frontier = vec![*cid];
        let mut current_depth = 0;

        while current_depth < depth && !frontier.is_empty() {
            let mut next_frontier = Vec::new();

            for node_cid in frontier {
                if let Some(node) = store.get(&node_cid) {
                    for parent in &node.parents {
                        if result.insert(*parent) {
                            next_frontier.push(*parent);
                        }
                    }
                }
            }

            frontier = next_frontier;
            current_depth += 1;
        }

        result
    }

    /// Find a path from any head to genesis.
    fn find_genesis_path<S: DAGStore>(&self, store: &S) -> Option<Vec<Hash>> {
        let heads = store.heads();
        if heads.is_empty() {
            return None;
        }

        let mut path = Vec::new();
        let mut current = heads[0];

        while let Some(node) = store.get(&current) {
            path.push(current);

            if node.parents.is_empty() {
                // Reached genesis
                break;
            }

            // Follow first parent
            current = node.parents[0];
        }

        if path.is_empty() {
            None
        } else {
            Some(path)
        }
    }
}

impl Default for Pruner {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for stores that support node removal.
pub trait PrunableStore: DAGStore {
    /// Remove a node from the store.
    fn remove(&mut self, cid: &Hash) -> Result<(), String>;

    /// Remove multiple nodes.
    fn remove_batch(&mut self, cids: &[Hash]) -> Result<usize, String> {
        let mut removed = 0;
        for cid in cids {
            if self.remove(cid).is_ok() {
                removed += 1;
            }
        }
        Ok(removed)
    }
}

// Note: MemoryDAGStore doesn't actually support removal (immutable by design).
// For testing purposes, we use wrapper types that track "pruned" nodes.
// In production, a proper store implementation would handle removal.

/// Verification utilities for pruning safety.
pub struct PruningVerifier;

impl PruningVerifier {
    /// Verify that no "live" items would be resurrected after pruning.
    ///
    /// This checks that all remove operations that reference pruned nodes
    /// are still correctly represented in the post-prune state.
    pub fn verify_no_resurrection<S: DAGStore>(
        store: &S,
        pruned: &[Hash],
        _snapshot: &Snapshot,
    ) -> Result<(), String> {
        // The snapshot should contain the complete state at its point
        // Any removes that happened before the snapshot are captured
        // We verify that no pruned nodes are referenced by non-pruned nodes

        let pruned_set: HashSet<_> = pruned.iter().copied().collect();

        for cid in store.topological_order() {
            if let Some(node) = store.get(&cid) {
                for parent in &node.parents {
                    if pruned_set.contains(parent) && !pruned_set.contains(&cid) {
                        return Err(format!(
                            "Node {} references pruned parent {}",
                            cid.to_hex(),
                            parent.to_hex()
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Verify that the DAG is still connected after pruning.
    pub fn verify_connectivity<S: DAGStore>(store: &S) -> Result<(), String> {
        let heads = store.heads();
        if heads.is_empty() {
            return Err("No heads in store".to_string());
        }

        // Check that all heads can reach some common ancestor
        // (or at least don't have dangling references)
        for head in &heads {
            let ancestors = store.ancestors(head);
            for ancestor in &ancestors {
                if !store.contains(ancestor) {
                    return Err(format!(
                        "Head {} references missing ancestor {}",
                        head.to_hex(),
                        ancestor.to_hex()
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdcs_merkle::{MemoryDAGStore, NodeBuilder, Payload};

    #[test]
    fn test_pruner_creation() {
        let pruner = Pruner::new();
        assert_eq!(pruner.policy().min_snapshots_before_prune, 2);

        let custom_policy = PruningPolicy {
            min_snapshots_before_prune: 5,
            ..Default::default()
        };
        let custom_pruner = Pruner::with_policy(custom_policy);
        assert_eq!(custom_pruner.policy().min_snapshots_before_prune, 5);
    }

    #[test]
    fn test_pruning_policy_defaults() {
        let policy = PruningPolicy::default();

        assert_eq!(policy.min_snapshots_before_prune, 2);
        assert_eq!(policy.min_node_age, 5000);
        assert_eq!(policy.max_nodes_per_prune, 1000);
        assert!(policy.require_stability);
        assert!(policy.preserve_genesis_path);
        assert_eq!(policy.preserve_depth, 10);
    }

    #[test]
    fn test_identify_prunable() {
        let (mut store, genesis) = MemoryDAGStore::with_genesis("test");

        // Create a chain: genesis -> a -> b -> c -> d
        let node_a = NodeBuilder::new()
            .with_parent(genesis)
            .with_payload(Payload::delta(b"a".to_vec()))
            .with_timestamp(100)
            .with_creator("test")
            .build();
        let cid_a = store.put(node_a).unwrap();

        let node_b = NodeBuilder::new()
            .with_parent(cid_a)
            .with_payload(Payload::delta(b"b".to_vec()))
            .with_timestamp(200)
            .with_creator("test")
            .build();
        let cid_b = store.put(node_b).unwrap();

        let node_c = NodeBuilder::new()
            .with_parent(cid_b)
            .with_payload(Payload::delta(b"c".to_vec()))
            .with_timestamp(300)
            .with_creator("test")
            .build();
        let cid_c = store.put(node_c).unwrap();

        let node_d = NodeBuilder::new()
            .with_parent(cid_c)
            .with_payload(Payload::delta(b"d".to_vec()))
            .with_timestamp(400)
            .with_creator("test")
            .build();
        let _cid_d = store.put(node_d).unwrap();

        // Create a snapshot at node_c
        let vv = VersionVector::from_entries([("test".to_string(), 3)]);
        let snapshot = Snapshot::new(vv, vec![cid_c], b"state".to_vec(), "test", 300);

        // Create pruner with low age requirement for testing
        let policy = PruningPolicy {
            min_node_age: 50,
            preserve_depth: 1,
            preserve_genesis_path: false,
            ..Default::default()
        };
        let pruner = Pruner::with_policy(policy);

        let prunable = pruner.identify_prunable(&store, &snapshot, 500);

        // Genesis and cid_a should be prunable (they're ancestors of snapshot, old enough)
        // cid_b and cid_c are within preserve_depth of heads
        assert!(!prunable.is_empty());
    }

    #[test]
    fn test_preserve_nodes() {
        let mut pruner = Pruner::new();
        let cid = mdcs_merkle::Hasher::hash(b"test");

        pruner.preserve(cid);
        assert!(pruner.preserved.contains(&cid));

        pruner.clear_preserved();
        assert!(pruner.preserved.is_empty());
    }

    #[test]
    fn test_pruning_result_empty() {
        let result = PruningResult::empty();

        assert_eq!(result.nodes_pruned, 0);
        assert!(result.pruned_cids.is_empty());
        assert!(result.snapshot_root.is_none());
        assert!(result.completed);
    }
}
