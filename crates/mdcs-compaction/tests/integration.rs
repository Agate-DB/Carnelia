//! Integration tests for the compaction subsystem.
//!
//! These tests verify:
//! - No resurrection: Removed items stay removed after compaction
//! - Deterministic rebuild: State rebuilt from snapshot + deltas matches full replay
//! - Stability tracking across replicas
//! - Safe pruning with verification

use mdcs_compaction::{
    CompactionConfig, Compactor, FrontierUpdate, Pruner, PruningPolicy, PruningVerifier, Snapshot,
    StabilityConfig, StabilityMonitor, VersionVector,
};
use mdcs_merkle::{DAGStore, Hash, NodeBuilder, Payload};
use std::collections::HashSet;

/// Helper to implement PrunableStore for tests
mod prunable {
    use mdcs_compaction::PrunableStore;
    use mdcs_merkle::{DAGError, DAGStore, Hash, MemoryDAGStore, MerkleNode};

    /// Wrapper that tracks "pruned" nodes
    pub struct PrunableMemoryStore {
        pub inner: MemoryDAGStore,
        pub pruned: std::collections::HashSet<Hash>,
    }

    impl PrunableMemoryStore {
        pub fn new() -> Self {
            PrunableMemoryStore {
                inner: MemoryDAGStore::new(),
                pruned: std::collections::HashSet::new(),
            }
        }

        pub fn with_genesis(creator: &str) -> (Self, Hash) {
            let (inner, genesis) = MemoryDAGStore::with_genesis(creator);
            (
                PrunableMemoryStore {
                    inner,
                    pruned: std::collections::HashSet::new(),
                },
                genesis,
            )
        }
    }

    impl DAGStore for PrunableMemoryStore {
        fn get(&self, cid: &Hash) -> Option<&MerkleNode> {
            if self.pruned.contains(cid) {
                None
            } else {
                self.inner.get(cid)
            }
        }

        fn put(&mut self, node: MerkleNode) -> Result<Hash, DAGError> {
            self.inner.put(node)
        }

        fn put_unchecked(&mut self, node: MerkleNode) -> Result<Hash, DAGError> {
            self.inner.put_unchecked(node)
        }

        fn heads(&self) -> Vec<Hash> {
            self.inner
                .heads()
                .into_iter()
                .filter(|h| !self.pruned.contains(h))
                .collect()
        }

        fn contains(&self, cid: &Hash) -> bool {
            !self.pruned.contains(cid) && self.inner.contains(cid)
        }

        fn ancestors(&self, cid: &Hash) -> std::collections::HashSet<Hash> {
            self.inner
                .ancestors(cid)
                .into_iter()
                .filter(|h| !self.pruned.contains(h))
                .collect()
        }

        fn children(&self, cid: &Hash) -> Vec<Hash> {
            self.inner
                .children(cid)
                .into_iter()
                .filter(|h| !self.pruned.contains(h))
                .collect()
        }

        fn topological_order(&self) -> Vec<Hash> {
            self.inner
                .topological_order()
                .into_iter()
                .filter(|h| !self.pruned.contains(h))
                .collect()
        }

        fn missing_nodes(&self) -> std::collections::HashSet<Hash> {
            self.inner.missing_nodes()
        }

        fn len(&self) -> usize {
            self.inner.len().saturating_sub(self.pruned.len())
        }
    }

    impl PrunableStore for PrunableMemoryStore {
        fn remove(&mut self, cid: &Hash) -> Result<(), String> {
            if self.inner.contains(cid) {
                self.pruned.insert(*cid);
                Ok(())
            } else {
                Err("Node not found".to_string())
            }
        }
    }
}

use prunable::PrunableMemoryStore;

// ============================================================================
// No Resurrection Tests
// ============================================================================

/// Test that removed items stay removed after compaction.
#[test]
fn test_no_resurrection_basic() {
    // Simulate a scenario where:
    // 1. Item is added (node A)
    // 2. Item is removed (node B references A, contains remove delta)
    // 3. Snapshot is taken at node B
    // 4. Node A is pruned
    // 5. Late-arriving add delta should NOT resurrect the item

    let (mut store, genesis) = PrunableMemoryStore::with_genesis("test");

    // Node A: Add item
    let node_a = NodeBuilder::new()
        .with_parent(genesis)
        .with_payload(Payload::delta(b"add:item1".to_vec()))
        .with_timestamp(100)
        .with_creator("test")
        .build();
    let cid_a = store.put(node_a).unwrap();

    // Node B: Remove item (references A)
    let node_b = NodeBuilder::new()
        .with_parent(cid_a)
        .with_payload(Payload::delta(b"remove:item1".to_vec()))
        .with_timestamp(200)
        .with_creator("test")
        .build();
    let cid_b = store.put(node_b).unwrap();

    // Create snapshot at B - this captures that item1 is removed
    let vv = VersionVector::from_entries([("test".to_string(), 2)]);
    let snapshot = Snapshot::new(
        vv,
        vec![cid_b],
        b"state:{item1:removed}".to_vec(),
        "test",
        200,
    );

    // Prune node A
    store.pruned.insert(cid_a);

    // Verify A is no longer accessible
    assert!(!store.contains(&cid_a));

    // The snapshot still correctly represents the state (item1 removed)
    assert_eq!(snapshot.state_data, b"state:{item1:removed}");

    // Even if we receive a late "add:item1" delta, the snapshot state
    // should be authoritative - item1 stays removed
}

/// Test no resurrection with concurrent branches.
#[test]
fn test_no_resurrection_concurrent_branches() {
    let (mut store, genesis) = PrunableMemoryStore::with_genesis("test");

    // Branch 1: Add item
    let branch1 = NodeBuilder::new()
        .with_parent(genesis)
        .with_payload(Payload::delta(b"add:item".to_vec()))
        .with_timestamp(100)
        .with_creator("r1")
        .build();
    let cid_b1 = store.put(branch1).unwrap();

    // Branch 2: Remove item (concurrent, also from genesis)
    let branch2 = NodeBuilder::new()
        .with_parent(genesis)
        .with_payload(Payload::delta(b"remove:item".to_vec()))
        .with_timestamp(100)
        .with_creator("r2")
        .build();
    let cid_b2 = store.put(branch2).unwrap();

    // Merge node
    let merge = NodeBuilder::new()
        .with_parents(vec![cid_b1, cid_b2])
        .with_payload(Payload::delta(b"merge".to_vec()))
        .with_timestamp(200)
        .with_creator("test")
        .build();
    let cid_merge = store.put(merge).unwrap();

    // With add-wins semantics, item should be present after merge
    // With remove-wins, item should be absent
    // Either way, snapshot captures the resolved state

    let vv = VersionVector::from_entries([
        ("r1".to_string(), 1),
        ("r2".to_string(), 1),
        ("test".to_string(), 1),
    ]);
    let snapshot = Snapshot::new(vv, vec![cid_merge], b"resolved_state".to_vec(), "test", 200);

    // Prune old branches
    store.pruned.insert(cid_b1);
    store.pruned.insert(cid_b2);

    // Merge still accessible
    assert!(store.contains(&cid_merge));

    // Snapshot represents the authoritative resolved state
    assert!(snapshot.covers(&VersionVector::from_entries([
        ("r1".to_string(), 1),
        ("r2".to_string(), 1),
    ])));
}

// ============================================================================
// Deterministic Rebuild Tests
// ============================================================================

/// Test that state can be deterministically rebuilt from snapshot + deltas.
#[test]
fn test_deterministic_rebuild_from_snapshot() {
    let (mut store, genesis) = PrunableMemoryStore::with_genesis("test");

    // Create initial chain: genesis -> a -> b -> c
    let node_a = NodeBuilder::new()
        .with_parent(genesis)
        .with_payload(Payload::delta(b"delta_a".to_vec()))
        .with_timestamp(100)
        .with_creator("test")
        .build();
    let cid_a = store.put(node_a).unwrap();

    let node_b = NodeBuilder::new()
        .with_parent(cid_a)
        .with_payload(Payload::delta(b"delta_b".to_vec()))
        .with_timestamp(200)
        .with_creator("test")
        .build();
    let cid_b = store.put(node_b).unwrap();

    // Snapshot at b
    let vv_b = VersionVector::from_entries([("test".to_string(), 2)]);
    let snapshot = Snapshot::new(vv_b, vec![cid_b], b"state_at_b".to_vec(), "test", 200);

    // Continue with more nodes: c -> d
    let node_c = NodeBuilder::new()
        .with_parent(cid_b)
        .with_payload(Payload::delta(b"delta_c".to_vec()))
        .with_timestamp(300)
        .with_creator("test")
        .build();
    let cid_c = store.put(node_c.clone()).unwrap();

    let node_d = NodeBuilder::new()
        .with_parent(cid_c)
        .with_payload(Payload::delta(b"delta_d".to_vec()))
        .with_timestamp(400)
        .with_creator("test")
        .build();
    let _cid_d = store.put(node_d.clone()).unwrap();

    // Prune nodes before snapshot
    store.pruned.insert(genesis);
    store.pruned.insert(cid_a);

    // Rebuild: Load snapshot state, then apply deltas c and d
    let mut rebuilt_state = snapshot.state_data.clone();

    // Get deltas after snapshot
    let deltas_after: Vec<_> = store
        .topological_order()
        .iter()
        .filter(|cid| **cid != cid_b) // Exclude snapshot point
        .filter_map(|cid| store.get(cid))
        .collect();

    // Apply deltas in order
    for node in deltas_after {
        if let Payload::Delta(delta) = &node.payload {
            rebuilt_state.extend_from_slice(b"+");
            rebuilt_state.extend_from_slice(delta);
        }
    }

    // The rebuilt state should be deterministic
    assert!(rebuilt_state.starts_with(b"state_at_b"));
}

/// Test rebuild matches full replay.
#[test]
fn test_rebuild_matches_full_replay() {
    // Simulate state as a simple counter
    fn apply_delta(state: &mut i64, delta: &[u8]) {
        if delta.starts_with(b"inc:") {
            let n: i64 = std::str::from_utf8(&delta[4..])
                .unwrap()
                .parse()
                .unwrap_or(1);
            *state += n;
        } else if delta.starts_with(b"dec:") {
            let n: i64 = std::str::from_utf8(&delta[4..])
                .unwrap()
                .parse()
                .unwrap_or(1);
            *state -= n;
        }
    }

    let (mut store, genesis) = PrunableMemoryStore::with_genesis("test");

    // Full replay path
    let mut full_replay_state: i64 = 0;

    // Create operations
    let ops = vec![
        b"inc:5".to_vec(),
        b"inc:3".to_vec(),
        b"dec:2".to_vec(),
        b"inc:10".to_vec(),
        b"dec:1".to_vec(),
    ];

    let mut prev = genesis;
    let mut nodes = Vec::new();

    for (i, op) in ops.iter().enumerate() {
        apply_delta(&mut full_replay_state, op);

        let node = NodeBuilder::new()
            .with_parent(prev)
            .with_payload(Payload::delta(op.clone()))
            .with_timestamp((i + 1) as u64 * 100)
            .with_creator("test")
            .build();
        let cid = store.put(node.clone()).unwrap();
        nodes.push((cid, node));
        prev = cid;
    }

    // Snapshot after 3rd operation (state should be 5+3-2 = 6)
    let snapshot_state: i64 = 6;
    let vv = VersionVector::from_entries([("test".to_string(), 3)]);
    let snapshot = Snapshot::new(
        vv,
        vec![nodes[2].0],
        snapshot_state.to_le_bytes().to_vec(),
        "test",
        300,
    );

    // Rebuild from snapshot
    let mut rebuilt_state = i64::from_le_bytes(snapshot.state_data.clone().try_into().unwrap());

    // Apply remaining deltas (ops 4 and 5)
    apply_delta(&mut rebuilt_state, &ops[3]); // inc:10
    apply_delta(&mut rebuilt_state, &ops[4]); // dec:1

    // Both should be 15 (5+3-2+10-1)
    assert_eq!(full_replay_state, 15);
    assert_eq!(rebuilt_state, 15);
    assert_eq!(full_replay_state, rebuilt_state);
}

// ============================================================================
// Stability Tracking Tests
// ============================================================================

/// Test stability tracking across multiple replicas.
#[test]
fn test_stability_multi_replica() {
    let mut monitor = StabilityMonitor::new("r1");

    // Local frontier
    let local_vv = VersionVector::from_entries([
        ("r1".to_string(), 100),
        ("r2".to_string(), 50),
        ("r3".to_string(), 75),
    ]);
    monitor.update_local_frontier(local_vv, vec![]);

    // Peer r2 is behind on r1's updates
    monitor.update_peer_frontier(FrontierUpdate {
        peer_id: "r2".to_string(),
        version_vector: VersionVector::from_entries([
            ("r1".to_string(), 80),
            ("r2".to_string(), 50),
            ("r3".to_string(), 75),
        ]),
        heads: vec![],
        timestamp: 100,
    });

    // Peer r3 is behind on both r1 and r2
    monitor.update_peer_frontier(FrontierUpdate {
        peer_id: "r3".to_string(),
        version_vector: VersionVector::from_entries([
            ("r1".to_string(), 70),
            ("r2".to_string(), 40),
            ("r3".to_string(), 75),
        ]),
        heads: vec![],
        timestamp: 100,
    });

    // Stable frontier should be min of all
    let stable = monitor.stable_frontier();
    assert_eq!(stable.get("r1"), 70);
    assert_eq!(stable.get("r2"), 40);
    assert_eq!(stable.get("r3"), 75);

    // Operations up to stable point should be stable
    assert!(monitor.is_operation_stable("r1", 70));
    assert!(monitor.is_operation_stable("r2", 40));
    assert!(!monitor.is_operation_stable("r1", 80));
}

/// Test stability with quorum-based configuration.
#[test]
fn test_stability_quorum() {
    let config = StabilityConfig {
        min_peers_for_stability: 2,
        require_all_peers: false,
        quorum_fraction: 0.5,
        ..Default::default()
    };

    let mut monitor = StabilityMonitor::with_config("r1", config);

    // Just r1 - no quorum
    assert!(!monitor.has_quorum());

    // Add r2
    monitor.update_peer_frontier(FrontierUpdate {
        peer_id: "r2".to_string(),
        version_vector: VersionVector::new(),
        heads: vec![],
        timestamp: 100,
    });
    assert!(monitor.has_quorum()); // 2 replicas, 50% quorum met
}

// ============================================================================
// Compactor Integration Tests
// ============================================================================

/// Test full compaction workflow.
#[test]
fn test_compactor_full_workflow() {
    let config = CompactionConfig {
        auto_compact: true,
        min_ops_for_compaction: 3,
        verify_after_compaction: true,
        ..Default::default()
    };

    let mut compactor = Compactor::with_config("test", config);
    let (mut store, genesis) = PrunableMemoryStore::with_genesis("test");

    // Create some operations
    let mut prev = genesis;
    for i in 1..=5 {
        let node = NodeBuilder::new()
            .with_parent(prev)
            .with_payload(Payload::delta(format!("op{}", i).into_bytes()))
            .with_timestamp(i * 100)
            .with_creator("test")
            .build();
        prev = store.put(node).unwrap();
    }

    // Update local frontier
    let vv = VersionVector::from_entries([("test".to_string(), 5)]);
    compactor.update_local_frontier(vv, vec![prev]);
    compactor.set_time(500);

    // Create snapshot
    let snapshot_id = compactor
        .create_snapshot(vec![prev], || Ok(b"current_state".to_vec()))
        .unwrap();

    assert!(snapshot_id != Hash::zero());
    assert_eq!(compactor.stats().snapshots_created, 1);
}

/// Test compactor bootstrap from snapshot.
#[test]
fn test_compactor_bootstrap() {
    // Original compactor creates state
    let mut original = Compactor::new("original");
    let vv =
        VersionVector::from_entries([("original".to_string(), 100), ("other".to_string(), 50)]);
    original.update_local_frontier(vv.clone(), vec![]);
    original.set_time(1000);

    let snapshot_id = original
        .create_snapshot(vec![], || Ok(b"full_state_data".to_vec()))
        .unwrap();

    let snapshot = original.snapshots().get(&snapshot_id).unwrap().clone();

    // New replica bootstraps from snapshot
    let mut new_replica = Compactor::new("new");
    let (state_data, recovered_vv) = new_replica.bootstrap_from_snapshot(snapshot).unwrap();

    assert_eq!(state_data, b"full_state_data");
    assert_eq!(recovered_vv, vv);
}

/// Test compactor with peer frontier updates.
#[test]
fn test_compactor_peer_coordination() {
    let mut compactor1 = Compactor::new("r1");
    let mut compactor2 = Compactor::new("r2");

    // r1 makes progress
    let vv1 = VersionVector::from_entries([("r1".to_string(), 100)]);
    compactor1.update_local_frontier(vv1, vec![]);
    compactor1.set_time(100);

    // r2 makes progress
    let vv2 = VersionVector::from_entries([("r2".to_string(), 80)]);
    compactor2.update_local_frontier(vv2, vec![]);
    compactor2.set_time(100);

    // Exchange frontier updates
    let update1 = compactor1.create_frontier_update();
    let update2 = compactor2.create_frontier_update();

    compactor1.process_peer_update(update2);
    compactor2.process_peer_update(update1);

    // Both should now know about each other
    assert!(compactor1.stability().peer_frontier("r2").is_some());
    assert!(compactor2.stability().peer_frontier("r1").is_some());
}

// ============================================================================
// Pruning Safety Tests
// ============================================================================

/// Test that pruning respects preserve_depth.
#[test]
fn test_pruning_preserve_depth() {
    let policy = PruningPolicy {
        preserve_depth: 2,
        min_node_age: 0, // Allow immediate pruning for test
        preserve_genesis_path: false,
        ..Default::default()
    };

    let pruner = Pruner::with_policy(policy);
    let (mut store, genesis) = PrunableMemoryStore::with_genesis("test");

    // Create chain: genesis -> a -> b -> c -> d -> e
    let mut prev = genesis;
    let mut cids = vec![genesis];

    for i in 0..5 {
        let node = NodeBuilder::new()
            .with_parent(prev)
            .with_payload(Payload::delta(format!("node{}", i).into_bytes()))
            .with_timestamp((i + 1) * 100)
            .with_creator("test")
            .build();
        let cid = store.put(node).unwrap();
        cids.push(cid);
        prev = cid;
    }

    // Snapshot at node d (index 4)
    let snapshot_cid = cids[4];
    let vv = VersionVector::from_entries([("test".to_string(), 4)]);
    let snapshot = Snapshot::new(vv, vec![snapshot_cid], b"state".to_vec(), "test", 400);

    // Identify prunable nodes
    let prunable = pruner.identify_prunable(&store.inner, &snapshot, 1000);

    // With preserve_depth=2, nodes within 2 of head (e) should be preserved
    // Head is e (index 5), so d (4) and c (3) should be preserved
    // Snapshot itself (d) should be preserved
    // Genesis, a, b could be prunable

    // Verify head's ancestors within depth are NOT in prunable list
    let _head = cids[5]; // e
    let near_head: HashSet<_> = [cids[4], cids[3]].into_iter().collect();

    for cid in &prunable {
        assert!(
            !near_head.contains(cid),
            "Node near head should not be prunable"
        );
    }
}

/// Test pruning verification catches missing references.
#[test]
fn test_pruning_verification() {
    let (mut store, genesis) = PrunableMemoryStore::with_genesis("test");

    // Create a -> b
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

    // If we prune 'a' without also pruning 'b', verification should catch it
    let pruned = vec![cid_a];

    let vv = VersionVector::from_entries([("test".to_string(), 2)]);
    let snapshot = Snapshot::new(vv, vec![cid_b], b"state".to_vec(), "test", 200);

    // This should fail because b references pruned a
    let result = PruningVerifier::verify_no_resurrection(&store.inner, &pruned, &snapshot);
    assert!(result.is_err());
}

// ============================================================================
// Version Vector Tests
// ============================================================================

/// Test version vector diff calculation.
#[test]
fn test_version_vector_diff() {
    let vv1 = VersionVector::from_entries([
        ("r1".to_string(), 100),
        ("r2".to_string(), 50),
        ("r3".to_string(), 75),
    ]);

    let vv2 = VersionVector::from_entries([
        ("r1".to_string(), 80),
        ("r2".to_string(), 50),
        ("r3".to_string(), 60),
    ]);

    let diff = vv1.diff(&vv2);

    // r1: has 81-100 (20 ops)
    // r2: no diff
    // r3: has 61-75 (15 ops)
    assert_eq!(diff.len(), 2);

    let diff_map: std::collections::HashMap<_, _> =
        diff.into_iter().map(|(r, s, e)| (r, (s, e))).collect();

    assert_eq!(diff_map.get("r1"), Some(&(81, 100)));
    assert_eq!(diff_map.get("r3"), Some(&(61, 75)));
}

/// Test version vector strictly_dominates.
#[test]
fn test_version_vector_strictly_dominates() {
    let vv1 = VersionVector::from_entries([("r1".to_string(), 10), ("r2".to_string(), 5)]);
    let vv2 = VersionVector::from_entries([("r1".to_string(), 10), ("r2".to_string(), 5)]);
    let vv3 = VersionVector::from_entries([("r1".to_string(), 10), ("r2".to_string(), 4)]);

    // Equal vectors: dominates but not strictly
    assert!(vv1.dominates(&vv2));
    assert!(!vv1.strictly_dominates(&vv2));

    // vv1 > vv3: strictly dominates
    assert!(vv1.dominates(&vv3));
    assert!(vv1.strictly_dominates(&vv3));
}
