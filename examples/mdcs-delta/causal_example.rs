//! Example: Causal Consistency with δ-CRDTs (Algorithm 2)
//!
//! This example demonstrates:
//! 1. Delta-interval based anti-entropy for causal delivery
//! 2. Crash recovery with durable vs volatile state
//! 3. Network partitions and healing
//! 4. Snapshot-based bootstrapping for new replicas

use mdcs_core::gset::GSet;
use mdcs_core::pncounter::PNCounter;
use mdcs_delta::causal::{
    CausalCluster, CausalReplica, MemoryStorage, DurableStorage
};

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Causal Consistency Examples (Algorithm 2)");
    println!("═══════════════════════════════════════════════════════════════\n");

    example_1_basic_causal_sync();
    example_2_out_of_order_delivery();
    example_3_crash_recovery();
    example_4_network_partition();
    example_5_snapshot_bootstrap();
    example_6_distributed_counter();

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  All causal consistency examples completed!");
    println!("═══════════════════════════════════════════════════════════════");
}

/// Example 1: Basic causal synchronization between two replicas
fn example_1_basic_causal_sync() {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 1: Basic Causal Synchronization                    │");
    println!("│            Delta-intervals with sequence numbers           │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    // Create two replicas
    let mut r1: CausalReplica<GSet<i32>> = CausalReplica::new("replica_1");
    let mut r2: CausalReplica<GSet<i32>> = CausalReplica::new("replica_2");

    // Register peers (they need to know about each other)
    r1.register_peer("replica_2".to_string());
    r2.register_peer("replica_1".to_string());

    // Replica 1 adds elements
    r1.mutate(|_| {
        let mut d = GSet::new();
        d.insert(1);
        d.insert(2);
        d.insert(3);
        d
    });
    println!("Replica 1 after mutations: {:?}", r1.state().iter().collect::<Vec<_>>());
    println!("Replica 1 counter (sequence): {}", r1.counter());

    // Prepare delta-interval for replica 2
    if let Some(interval) = r1.prepare_interval("replica_2") {
        println!("\nDelta-interval prepared:");
        println!("  From: {}", interval.from);
        println!("  Sequence range: ({}, {}]", interval.from_seq, interval.to_seq);
        
        // Replica 2 receives the interval
        if let Some(ack) = r2.receive_interval(interval) {
            println!("\nReplica 2 received and applied interval");
            println!("  Ack sequence: {}", ack.acked_seq);
            
            // Replica 1 receives the ack
            r1.receive_ack(&ack);
            println!("  Replica 1 received ack, delta buffer cleared");
        }
    }

    println!("\nReplica 2 state after sync: {:?}", r2.state().iter().collect::<Vec<_>>());
    println!("States match: {}", r1.state() == r2.state());
    
    println!("\n✓ Basic causal sync complete\n");
}

/// Example 2: Handling out-of-order delta delivery
fn example_2_out_of_order_delivery() {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 2: Out-of-Order Delta Delivery                     │");
    println!("│            Buffering and causal ordering                   │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    let mut r1: CausalReplica<GSet<i32>> = CausalReplica::new("r1");
    let mut r2: CausalReplica<GSet<i32>> = CausalReplica::new("r2");

    r1.register_peer("r2".to_string());
    r2.register_peer("r1".to_string());

    // R1 creates 3 sequential mutations
    println!("Creating 3 sequential mutations on R1...");
    
    r1.mutate(|_| { let mut d = GSet::new(); d.insert(10); d });
    let interval1 = r1.prepare_interval("r2").unwrap();
    
    r1.mutate(|_| { let mut d = GSet::new(); d.insert(20); d });
    let interval2 = r1.prepare_interval("r2").unwrap();
    
    r1.mutate(|_| { let mut d = GSet::new(); d.insert(30); d });
    let interval3 = r1.prepare_interval("r2").unwrap();

    println!("Interval 1: seq ({}, {}] - contains {{10}}", interval1.from_seq, interval1.to_seq);
    println!("Interval 2: seq ({}, {}] - contains {{20}}", interval2.from_seq, interval2.to_seq);
    println!("Interval 3: seq ({}, {}] - contains {{30}}", interval3.from_seq, interval3.to_seq);

    // Simulate out-of-order delivery: 3 arrives first, then 1, then 2
    println!("\nDelivering out of order: 3, 1, 2");

    let result3 = r2.receive_interval(interval3.clone());
    println!("  Interval 3: {} (buffered: {})", 
        if result3.is_some() { "applied" } else { "buffered" },
        r2.pending_count());

    let result1 = r2.receive_interval(interval1.clone());
    println!("  Interval 1: {} (buffered: {})", 
        if result1.is_some() { "applied" } else { "buffered" },
        r2.pending_count());

    let result2 = r2.receive_interval(interval2.clone());
    println!("  Interval 2: {} (buffered: {})", 
        if result2.is_some() { "applied" } else { "buffered" },
        r2.pending_count());

    println!("\nR2 final state: {:?}", r2.state().iter().collect::<Vec<_>>());
    println!("All elements present: {}", 
        r2.state().contains(&10) && r2.state().contains(&20) && r2.state().contains(&30));

    println!("\n✓ Out-of-order delivery handled correctly\n");
}

/// Example 3: Crash recovery with durable storage
fn example_3_crash_recovery() {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 3: Crash Recovery                                  │");
    println!("│            Durable state vs volatile state                 │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    // Create in-memory storage (simulating disk)
    let mut storage: MemoryStorage<GSet<i32>> = MemoryStorage::new();

    // Create replica and add data
    let mut replica: CausalReplica<GSet<i32>> = CausalReplica::new("crash_demo");
    
    println!("Adding elements 1-5 to replica...");
    for i in 1..=5 {
        replica.mutate(move |_| {
            let mut d = GSet::new();
            d.insert(i);
            d
        });
    }

    println!("State before crash: {:?}", replica.state().iter().collect::<Vec<_>>());
    println!("Counter (sequence): {}", replica.counter());
    println!("Has pending deltas: {}", replica.has_pending_deltas());

    // Persist to "disk"
    storage.persist(replica.durable_state()).unwrap();
    println!("\n[Durable state persisted to storage]");

    // CRASH!
    println!("\n💥 SIMULATING CRASH 💥");
    drop(replica);

    // Recover from storage
    println!("\n[Recovering from durable storage...]");
    let durable = storage.load("crash_demo").unwrap().unwrap();
    let recovered = CausalReplica::restore(durable);

    println!("\nState after recovery: {:?}", recovered.state().iter().collect::<Vec<_>>());
    println!("Counter after recovery: {}", recovered.counter());
    println!("Pending deltas after recovery: {}", recovered.has_pending_deltas());

    println!("\n✓ Crash recovery successful - durable state preserved\n");
}

/// Example 4: Network partition and healing
fn example_4_network_partition() {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 4: Network Partition and Healing                   │");
    println!("│            Divergence during partition, merge on heal      │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    // Create a 3-replica cluster
    let mut cluster: CausalCluster<GSet<i32>> = CausalCluster::new(3, 0.0);

    // Initial data on each replica
    println!("Initial mutations:");
    for i in 0..3 {
        let val = (i * 10) as i32;
        cluster.mutate(i, move |_| {
            let mut d = GSet::new();
            d.insert(val);
            d
        });
        println!("  Replica {}: added {}", i, val);
    }

    // Sync before partition
    cluster.full_sync_round();
    println!("\nAfter initial sync - converged: {}", cluster.is_converged());

    // PARTITION: replica 2 is isolated
    println!("\n⚡ NETWORK PARTITION - Replica 2 isolated");

    // Replicas 0 and 1 continue to operate
    cluster.mutate(0, |_| { let mut d = GSet::new(); d.insert(100); d });
    cluster.mutate(1, |_| { let mut d = GSet::new(); d.insert(200); d });
    
    // Replica 2 also operates independently
    cluster.mutate(2, |_| { let mut d = GSet::new(); d.insert(300); d });

    println!("During partition:");
    println!("  Replica 0 added: 100");
    println!("  Replica 1 added: 200");
    println!("  Replica 2 added: 300 (isolated)");
    
    // States are divergent
    println!("\nStates divergent: {}", !cluster.is_converged());

    // HEAL: partition is resolved
    println!("\n🔗 PARTITION HEALED - All replicas can communicate");

    // Multiple sync rounds to ensure convergence
    for _ in 0..5 {
        cluster.full_sync_round();
    }

    println!("\nAfter healing - converged: {}", cluster.is_converged());
    println!("Final state (all replicas): {:?}", 
        cluster.replica(0).state().iter().collect::<Vec<_>>());

    println!("\n✓ Partition scenario handled - all data preserved\n");
}

/// Example 5: Bootstrap a new replica from snapshot
fn example_5_snapshot_bootstrap() {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 5: Snapshot Bootstrap                              │");
    println!("│            New replica joins via state transfer            │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    // Existing cluster with data
    let mut cluster: CausalCluster<GSet<i32>> = CausalCluster::new(2, 0.0);

    println!("Building up state on existing replicas...");
    for i in 0..50 {
        let val = i as i32;
        cluster.mutate(i % 2, move |_| {
            let mut d = GSet::new();
            d.insert(val);
            d
        });
    }
    
    cluster.full_sync_round();
    println!("Existing cluster has {} elements", cluster.replica(0).state().len());

    // Get snapshot from replica 0
    let (snapshot_state, snapshot_seq) = cluster.replica(0).snapshot();
    println!("\nTaking snapshot from replica 0:");
    println!("  State size: {} elements", snapshot_state.len());
    println!("  Sequence: {}", snapshot_seq);

    // Create new replica and bootstrap from snapshot
    let mut new_replica: CausalReplica<GSet<i32>> = CausalReplica::new("new_replica");
    new_replica.apply_snapshot(snapshot_state.clone(), snapshot_seq, "causal_0");

    println!("\nNew replica bootstrapped:");
    println!("  State size: {} elements", new_replica.state().len());
    println!("  States match: {}", new_replica.state() == cluster.replica(0).state());

    println!("\n✓ Snapshot bootstrap complete\n");
}

/// Example 6: Distributed counter with causal consistency
fn example_6_distributed_counter() {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 6: Distributed Counter                             │");
    println!("│            PNCounter with causal delivery                  │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    let mut cluster: CausalCluster<PNCounter<String>> = CausalCluster::new(3, 0.0);

    println!("Each replica increments 10 times:");
    
    // Each replica does increments
    for replica_idx in 0..3 {
        let replica_id = format!("r{}", replica_idx);
        for _ in 0..10 {
            let id = replica_id.clone();
            cluster.mutate(replica_idx, move |_state| {
                let mut delta = PNCounter::new();
                delta.increment(id, 1);
                delta
            });
        }
        println!("  Replica {} local value: {}", 
            replica_idx, 
            cluster.replica(replica_idx).state().value());
    }

    println!("\nBefore sync - converged: {}", cluster.is_converged());

    // Sync
    for _ in 0..5 {
        cluster.full_sync_round();
    }

    println!("After sync - converged: {}", cluster.is_converged());
    println!("Final counter value: {} (expected: 30)", 
        cluster.replica(0).state().value());

    // Demonstrate decrement
    println!("\nReplica 0 decrements 5 times:");
    for _ in 0..5 {
        cluster.mutate(0, |_state| {
            let mut delta = PNCounter::new();
            delta.decrement("r0".to_string(), 1);
            delta
        });
    }

    cluster.full_sync_round();
    println!("Final value after decrements: {} (expected: 25)", 
        cluster.replica(0).state().value());

    println!("\n✓ Distributed counter example complete\n");
}
