//! Integration tests for the Merkle-Clock sync substrate.
//!
//! Tests cover:
//! - Bootstrap new replica from root CID
//! - Partition/heal scenario with multi-root merge
//! - Verify identical state after sync
//! - Gap repair during concurrent updates

use mdcs_merkle::{
    BroadcastNetwork, DAGStore, DAGSyncer, Hasher, MemoryDAGStore, MerkleNode, NodeBuilder,
    Payload, SyncResponse, SyncSimulator,
};

/// Test bootstrapping a new replica from a root CID.
#[test]
fn test_bootstrap_from_root_cid() {
    // Create source replica with some history
    let (mut source_store, genesis) = MemoryDAGStore::with_genesis("source");

    // Build a chain of updates
    let mut last = genesis;
    for i in 1..=5 {
        let node = NodeBuilder::new()
            .with_parent(last)
            .with_payload(Payload::delta(format!("update_{}", i).into_bytes()))
            .with_timestamp(i as u64)
            .with_creator("source")
            .build();
        last = source_store.put(node).unwrap();
    }

    let source_heads = source_store.heads();
    assert_eq!(source_heads.len(), 1);

    // Create new replica (empty)
    let new_store = MemoryDAGStore::new();
    let mut new_syncer = DAGSyncer::new(new_store);

    // Bootstrap: request all nodes starting from the head
    let source_syncer = DAGSyncer::new(source_store);

    // Get all nodes in topological order from source
    let all_nodes: Vec<MerkleNode> = source_syncer
        .store()
        .topological_order()
        .iter()
        .filter_map(|cid| source_syncer.store().get(cid).cloned())
        .collect();

    // Apply to new replica
    let stored = new_syncer
        .apply_response(SyncResponse::with_nodes(all_nodes))
        .unwrap();

    // Verify complete bootstrap
    assert_eq!(stored.len(), 6); // genesis + 5 updates
    assert_eq!(new_syncer.heads(), source_syncer.heads());
    assert!(new_syncer.store().missing_nodes().is_empty());
}

/// Test partition and heal scenario with multi-root merge.
#[test]
fn test_partition_and_heal() {
    let mut sim = SyncSimulator::with_shared_genesis(3);
    let genesis = sim.syncer(0).heads()[0];

    // Phase 1: All replicas in sync
    let shared_update = NodeBuilder::new()
        .with_parent(genesis)
        .with_payload(Payload::delta(b"shared".to_vec()))
        .with_timestamp(1)
        .with_creator("replica_0")
        .build();
    sim.syncer_mut(0).store_mut().put(shared_update).unwrap();

    sim.full_sync_round();
    assert!(sim.is_converged());

    // Phase 2: Network partition - replica_2 isolated
    let pre_partition_head = sim.syncer(0).heads()[0];

    // Replicas 0 and 1 make updates
    let update_0 = NodeBuilder::new()
        .with_parent(pre_partition_head)
        .with_payload(Payload::delta(b"from_0_during_partition".to_vec()))
        .with_timestamp(2)
        .with_creator("replica_0")
        .build();
    sim.syncer_mut(0).store_mut().put(update_0).unwrap();

    let update_1 = NodeBuilder::new()
        .with_parent(pre_partition_head)
        .with_payload(Payload::delta(b"from_1_during_partition".to_vec()))
        .with_timestamp(2)
        .with_creator("replica_1")
        .build();
    sim.syncer_mut(1).store_mut().put(update_1).unwrap();

    // Replica 2 makes its own update (isolated)
    let update_2 = NodeBuilder::new()
        .with_parent(pre_partition_head)
        .with_payload(Payload::delta(b"from_2_isolated".to_vec()))
        .with_timestamp(2)
        .with_creator("replica_2")
        .build();
    sim.syncer_mut(2).store_mut().put(update_2).unwrap();

    // Sync only 0 <-> 1 (simulating partition)
    sim.sync_pair(0, 1);
    sim.sync_pair(1, 0);

    // Replicas 0 and 1 have 2 heads (concurrent), replica 2 has 1 head
    assert_eq!(sim.syncer(0).heads().len(), 2);
    assert_eq!(sim.syncer(1).heads().len(), 2);
    assert_eq!(sim.syncer(2).heads().len(), 1);

    // Phase 3: Partition heals - full sync
    sim.full_sync_round();
    sim.full_sync_round(); // Multiple rounds may be needed

    // All replicas should now have 3 heads (three concurrent branches)
    assert_eq!(sim.syncer(0).heads().len(), 3);
    assert!(sim.is_converged());

    // All replicas should have the same node count
    let node_count = sim.syncer(0).store().len();
    assert_eq!(sim.syncer(1).store().len(), node_count);
    assert_eq!(sim.syncer(2).store().len(), node_count);
}

/// Test that identical state is achieved after sync.
#[test]
fn test_identical_state_after_sync() {
    let mut sim = SyncSimulator::with_shared_genesis(4);
    let genesis = sim.syncer(0).heads()[0];

    // Each replica makes multiple updates
    for replica_idx in 0..4 {
        let mut parent = genesis;
        for update_num in 1..=3 {
            let node = NodeBuilder::new()
                .with_parent(parent)
                .with_payload(Payload::delta(
                    format!("r{}_u{}", replica_idx, update_num).into_bytes(),
                ))
                .with_timestamp(update_num as u64)
                .with_creator(format!("replica_{}", replica_idx))
                .build();
            parent = sim.syncer_mut(replica_idx).store_mut().put(node).unwrap();
        }
    }

    // Before sync: each replica has different state
    assert!(!sim.is_converged());

    // Multiple sync rounds
    for _ in 0..5 {
        sim.full_sync_round();
    }

    // After sync: all converged
    assert!(sim.is_converged());

    // Verify topological order is consistent
    let order_0: Vec<_> = sim.syncer(0).store().topological_order();
    let order_1: Vec<_> = sim.syncer(1).store().topological_order();
    let order_2: Vec<_> = sim.syncer(2).store().topological_order();
    let order_3: Vec<_> = sim.syncer(3).store().topological_order();

    // All should have the same nodes (though order may differ slightly)
    assert_eq!(order_0.len(), order_1.len());
    assert_eq!(order_0.len(), order_2.len());
    assert_eq!(order_0.len(), order_3.len());

    // All heads should be identical
    let heads_0: std::collections::HashSet<_> = sim.syncer(0).heads().into_iter().collect();
    let heads_1: std::collections::HashSet<_> = sim.syncer(1).heads().into_iter().collect();
    let heads_2: std::collections::HashSet<_> = sim.syncer(2).heads().into_iter().collect();
    let heads_3: std::collections::HashSet<_> = sim.syncer(3).heads().into_iter().collect();

    assert_eq!(heads_0, heads_1);
    assert_eq!(heads_0, heads_2);
    assert_eq!(heads_0, heads_3);
}

/// Test gap repair when nodes arrive out of order.
#[test]
fn test_gap_repair() {
    let (mut source_store, genesis) = MemoryDAGStore::with_genesis("source");

    // Build chain: genesis -> a -> b -> c -> d
    let node_a = NodeBuilder::new()
        .with_parent(genesis)
        .with_payload(Payload::delta(b"a".to_vec()))
        .with_timestamp(1)
        .with_creator("source")
        .build();
    let cid_a = source_store.put(node_a.clone()).unwrap();

    let node_b = NodeBuilder::new()
        .with_parent(cid_a)
        .with_payload(Payload::delta(b"b".to_vec()))
        .with_timestamp(2)
        .with_creator("source")
        .build();
    let cid_b = source_store.put(node_b.clone()).unwrap();

    let node_c = NodeBuilder::new()
        .with_parent(cid_b)
        .with_payload(Payload::delta(b"c".to_vec()))
        .with_timestamp(3)
        .with_creator("source")
        .build();
    let cid_c = source_store.put(node_c.clone()).unwrap();

    let node_d = NodeBuilder::new()
        .with_parent(cid_c)
        .with_payload(Payload::delta(b"d".to_vec()))
        .with_timestamp(4)
        .with_creator("source")
        .build();
    let cid_d = source_store.put(node_d.clone()).unwrap();

    // Target replica starts with only genesis
    let (target_store, _) = MemoryDAGStore::with_genesis("source");
    let mut target_syncer = DAGSyncer::new(target_store);

    // Receive nodes out of order: d, b, c, a
    // Use unchecked to allow gaps
    target_syncer
        .apply_nodes_unchecked(vec![node_d.clone()])
        .unwrap();

    // Should have missing nodes
    assert!(!target_syncer.store().missing_nodes().is_empty());

    // Add more nodes
    target_syncer
        .apply_nodes_unchecked(vec![node_b.clone()])
        .unwrap();
    target_syncer
        .apply_nodes_unchecked(vec![node_c.clone()])
        .unwrap();
    target_syncer
        .apply_nodes_unchecked(vec![node_a.clone()])
        .unwrap();

    // Now all gaps should be filled
    assert!(target_syncer.store().missing_nodes().is_empty());
    assert_eq!(target_syncer.heads(), vec![cid_d]);
}

/// Test merge node creation for multi-root DAGs.
#[test]
fn test_multi_root_merge() {
    let (mut store, genesis) = MemoryDAGStore::with_genesis("merger");

    // Create two concurrent branches
    let branch_a = NodeBuilder::new()
        .with_parent(genesis)
        .with_payload(Payload::delta(b"branch_a".to_vec()))
        .with_timestamp(1)
        .with_creator("replica_a")
        .build();
    let cid_a = store.put(branch_a).unwrap();

    let branch_b = NodeBuilder::new()
        .with_parent(genesis)
        .with_payload(Payload::delta(b"branch_b".to_vec()))
        .with_timestamp(1)
        .with_creator("replica_b")
        .build();
    let cid_b = store.put(branch_b).unwrap();

    // Both should be heads
    assert_eq!(store.heads().len(), 2);

    // Create merge node
    let merge = NodeBuilder::new()
        .with_parents(vec![cid_a, cid_b])
        .with_payload(Payload::delta(b"merged".to_vec()))
        .with_timestamp(2)
        .with_creator("merger")
        .build();
    let merge_cid = store.put(merge).unwrap();

    // Only merge should be head now
    assert_eq!(store.heads(), vec![merge_cid]);

    // Ancestors should include both branches
    let ancestors = store.ancestors(&merge_cid);
    assert!(ancestors.contains(&cid_a));
    assert!(ancestors.contains(&cid_b));
    assert!(ancestors.contains(&genesis));
}

/// Test broadcaster head dissemination.
#[test]
fn test_broadcaster_dissemination() {
    let mut network = BroadcastNetwork::fully_connected(5);

    // Replica 0 broadcasts a new head
    let head = Hasher::hash(b"new_head");
    network.broadcast("replica_0", vec![head]);

    // Deliver all messages
    network.deliver_all();

    // Check that heads propagated via gossip
    // At least some replicas should have received the heads
    let mut received_count = 0;
    for i in 1..5 {
        let received = network.received_heads(&format!("replica_{}", i));
        if received.contains(&head) {
            received_count += 1;
        }
    }

    // At least fanout replicas should have received it directly
    assert!(received_count >= 2);
}

/// Test snapshot-based bootstrap.
#[test]
fn test_snapshot_bootstrap() {
    let (mut store, genesis) = MemoryDAGStore::with_genesis("source");

    // Add many updates
    let mut last = genesis;
    for i in 1..=20 {
        let node = NodeBuilder::new()
            .with_parent(last)
            .with_payload(Payload::delta(format!("data_{}", i).into_bytes()))
            .with_timestamp(i as u64)
            .with_creator("source")
            .build();
        last = store.put(node).unwrap();
    }

    // Create a snapshot at the current head
    let snapshot_data = b"serialized_full_state".to_vec();
    let snapshot = NodeBuilder::new()
        .with_parent(last)
        .with_payload(Payload::snapshot(snapshot_data.clone()))
        .with_timestamp(21)
        .with_creator("source")
        .build();
    let snapshot_cid = store.put(snapshot.clone()).unwrap();

    // Verify snapshot is recognized
    assert!(store.get(&snapshot_cid).unwrap().payload.is_snapshot());

    // New replica can bootstrap from snapshot
    let mut new_store = MemoryDAGStore::new();

    // In a real scenario, we'd only need to fetch the snapshot and subsequent updates
    // For this test, we verify the snapshot node is stored correctly
    new_store.put_unchecked(snapshot).unwrap();

    assert!(new_store.contains(&snapshot_cid));

    // The snapshot payload contains the state
    if let Payload::Snapshot(data) = &new_store.get(&snapshot_cid).unwrap().payload {
        assert_eq!(data, &snapshot_data);
    } else {
        panic!("Expected snapshot payload");
    }
}

/// Test DAG statistics and depth calculation.
#[test]
fn test_dag_statistics() {
    let (mut store, genesis) = MemoryDAGStore::with_genesis("stats_test");

    // Create a tree structure:
    //       genesis
    //      /   |   \
    //     a    b    c
    //     |    |
    //     d    e

    let node_a = NodeBuilder::new()
        .with_parent(genesis)
        .with_payload(Payload::delta(b"a".to_vec()))
        .with_timestamp(1)
        .with_creator("test")
        .build();
    let cid_a = store.put(node_a).unwrap();

    let node_b = NodeBuilder::new()
        .with_parent(genesis)
        .with_payload(Payload::delta(b"b".to_vec()))
        .with_timestamp(1)
        .with_creator("test")
        .build();
    let cid_b = store.put(node_b).unwrap();

    let node_c = NodeBuilder::new()
        .with_parent(genesis)
        .with_payload(Payload::delta(b"c".to_vec()))
        .with_timestamp(1)
        .with_creator("test")
        .build();
    store.put(node_c).unwrap();

    let node_d = NodeBuilder::new()
        .with_parent(cid_a)
        .with_payload(Payload::delta(b"d".to_vec()))
        .with_timestamp(2)
        .with_creator("test")
        .build();
    store.put(node_d).unwrap();

    let node_e = NodeBuilder::new()
        .with_parent(cid_b)
        .with_payload(Payload::delta(b"e".to_vec()))
        .with_timestamp(2)
        .with_creator("test")
        .build();
    store.put(node_e).unwrap();

    let stats = store.stats();

    assert_eq!(stats.total_nodes, 6);
    assert_eq!(stats.head_count, 3); // c, d, e are heads
    assert_eq!(stats.max_depth, 3); // genesis -> a -> d or genesis -> b -> e
}

/// Test verification of tampered nodes.
#[test]
fn test_tampered_node_rejected() {
    let mut store = MemoryDAGStore::new();

    let mut node = NodeBuilder::new()
        .with_payload(Payload::delta(b"original".to_vec()))
        .with_timestamp(1)
        .with_creator("test")
        .build();

    // Tamper with the payload
    node.payload = Payload::delta(b"tampered".to_vec());

    // Should be rejected
    let result = store.put(node);
    assert!(matches!(
        result,
        Err(mdcs_merkle::DAGError::VerificationFailed(_))
    ));
}

/// Test that sync handles large DAGs efficiently.
#[test]
fn test_large_dag_sync() {
    let mut sim = SyncSimulator::with_shared_genesis(2);
    let genesis = sim.syncer(0).heads()[0];

    // Build a large linear chain on replica 0
    let mut last = genesis;
    for i in 1..=100 {
        let node = NodeBuilder::new()
            .with_parent(last)
            .with_payload(Payload::delta(vec![i as u8]))
            .with_timestamp(i as u64)
            .with_creator("replica_0")
            .build();
        last = sim.syncer_mut(0).store_mut().put(node).unwrap();
    }

    assert_eq!(sim.syncer(0).store().len(), 101); // genesis + 100
    assert_eq!(sim.syncer(1).store().len(), 1); // only genesis

    // Sync
    sim.full_sync_round();

    // Both should have all nodes
    assert_eq!(sim.syncer(1).store().len(), 101);
    assert!(sim.is_converged());
}
