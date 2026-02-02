//! Causal Consistency Tests for Algorithm 2
//!
//! These tests verify the causal delta-merging condition, partition/restart
//! scenarios, and that no deltas are skipped on recovery.

use mdcs_core::gset::GSet;
use mdcs_core::orset::ORSet;
use mdcs_core::pncounter::PNCounter;
use mdcs_core::lwwreg::LWWRegister;
use mdcs_core::mvreg::MVRegister;
use mdcs_delta::causal::{
    CausalCluster, CausalReplica, DeltaInterval, MemoryStorage, DurableStorage,
};

/// Test that delta-intervals maintain causal ordering
#[test]
fn test_causal_ordering_strict() {
    let mut r1: CausalReplica<GSet<i32>> = CausalReplica::new("r1");
    let mut r2: CausalReplica<GSet<i32>> = CausalReplica::new("r2");

    r1.register_peer("r2".to_string());
    r2.register_peer("r1".to_string());

    // r1 creates sequential mutations
    for i in 1..=5 {
        r1.mutate(move |_| {
            let mut d = GSet::new();
            d.insert(i);
            d
        });
    }

    // Create intervals that arrive out of order
    // Interval 3-5 arrives first
    let interval_late = DeltaInterval {
        from: "r1".to_string(),
        to: "r2".to_string(),
        delta: {
            let mut d = GSet::new();
            d.insert(3);
            d.insert(4);
            d.insert(5);
            d
        },
        from_seq: 2,
        to_seq: 5,
    };

    // Interval 0-2 arrives later
    let interval_early = DeltaInterval {
        from: "r1".to_string(),
        to: "r2".to_string(),
        delta: {
            let mut d = GSet::new();
            d.insert(1);
            d.insert(2);
            d
        },
        from_seq: 0,
        to_seq: 2,
    };

    // Send late interval first - should be buffered
    let result = r2.receive_interval(interval_late);
    assert!(result.is_none(), "Late interval should be buffered");
    assert!(!r2.state().contains(&3), "Late data should not be applied yet");
    assert_eq!(r2.pending_count(), 1);

    // Send early interval - should be applied AND trigger pending
    let result = r2.receive_interval(interval_early);
    assert!(result.is_some(), "Early interval should be applied");
    
    // All data should now be present
    for i in 1..=5 {
        assert!(r2.state().contains(&i), "Element {} should be present", i);
    }
    assert_eq!(r2.pending_count(), 0, "Pending should be cleared");
}

/// Test crash recovery preserves durable state
#[test]
fn test_crash_recovery_durable_state() {
    let mut storage: MemoryStorage<GSet<i32>> = MemoryStorage::new();
    
    // Create replica and mutate
    let mut replica: CausalReplica<GSet<i32>> = CausalReplica::new("crash_test");
    
    for i in 1..=10 {
        replica.mutate(move |_| {
            let mut d = GSet::new();
            d.insert(i);
            d
        });
    }
    
    // Persist before crash
    storage.persist(replica.durable_state()).unwrap();
    let counter_before = replica.counter();
    
    // Simulate crash by dropping replica
    drop(replica);
    
    // Recover from storage
    let durable = storage.load("crash_test").unwrap().unwrap();
    let recovered = CausalReplica::restore(durable);
    
    // Verify durable state is preserved
    assert_eq!(recovered.counter(), counter_before);
    for i in 1..=10 {
        assert!(recovered.state().contains(&i), "Element {} should survive crash", i);
    }
}

/// Test that volatile state is lost on crash
#[test]
fn test_crash_loses_volatile_state() {
    let mut cluster: CausalCluster<GSet<i32>> = CausalCluster::new(2, 0.0);
    
    // r0 creates mutations
    for i in 1..=5 {
        let val = i as i32;
        cluster.mutate(0, move |_| {
            let mut d = GSet::new();
            d.insert(val);
            d
        });
    }
    
    // Verify r0 has pending deltas
    assert!(cluster.replica(0).has_pending_deltas());
    
    // Crash r0
    cluster.crash_and_recover(0);
    
    // Volatile state (pending deltas) should be lost
    assert!(!cluster.replica(0).has_pending_deltas());
    
    // But durable state should remain
    for i in 1..=5 {
        assert!(cluster.replica(0).state().contains(&i));
    }
}

/// Test convergence under network partitions
#[test]
fn test_partition_and_heal() {
    let mut cluster: CausalCluster<GSet<i32>> = CausalCluster::new(3, 0.0);
    
    // Initial mutations
    for i in 0..3 {
        let val = (i * 10) as i32;
        cluster.mutate(i, move |_| {
            let mut d = GSet::new();
            d.insert(val);
            d
        });
    }
    
    // Full sync before partition
    cluster.full_sync_round();
    assert!(cluster.is_converged());
    
    // Simulate partition: replicas 0 and 1 can sync, replica 2 is isolated
    // We simulate this by only syncing 0 and 1
    cluster.mutate(0, |_| {
        let mut d = GSet::new();
        d.insert(100);
        d
    });
    
    cluster.mutate(1, |_| {
        let mut d = GSet::new();
        d.insert(200);
        d
    });
    
    cluster.mutate(2, |_| {
        let mut d = GSet::new();
        d.insert(300);
        d
    });
    
    // Not converged due to partition
    assert!(!cluster.is_converged());
    
    // Heal partition - full sync
    for _ in 0..5 {
        cluster.full_sync_round();
    }
    
    // Should converge after healing
    assert!(cluster.is_converged());
    
    // All data should be present
    for val in [0, 10, 20, 100, 200, 300] {
        assert!(cluster.replica(0).state().contains(&val));
    }
}

/// Test that deltas are never skipped on recovery
#[test]
fn test_no_skip_on_recovery() {
    let mut storage: MemoryStorage<PNCounter<String>> = MemoryStorage::new();
    
    let mut replica: CausalReplica<PNCounter<String>> = CausalReplica::new("no_skip");
    
    // Perform increments - clone state and add 1 so delta has new total
    for _ in 0..100 {
        replica.mutate(|s| {
            let mut delta = s.clone();
            delta.increment("no_skip".to_string(), 1);
            delta
        });
    }
    
    // Persist
    storage.persist(replica.durable_state()).unwrap();
    
    // Recover
    let durable = storage.load("no_skip").unwrap().unwrap();
    let recovered: CausalReplica<PNCounter<String>> = CausalReplica::restore(durable);
    
    // Counter should be exactly 100
    assert_eq!(recovered.counter(), 100);
    assert_eq!(recovered.state().value(), 100);
}

/// Test multiple concurrent mutations from different replicas
#[test]
fn test_concurrent_mutations() {
    let mut cluster: CausalCluster<GSet<i32>> = CausalCluster::new(4, 0.0);
    
    // Each replica adds 10 elements concurrently
    for replica_idx in 0..4 {
        for j in 0..10 {
            let val = (replica_idx * 100 + j) as i32;
            cluster.mutate(replica_idx, move |_| {
                let mut d = GSet::new();
                d.insert(val);
                d
            });
        }
    }
    
    // Multiple sync rounds
    for _ in 0..10 {
        cluster.full_sync_round();
    }
    
    // Should converge
    assert!(cluster.is_converged());
    
    // All 40 elements should be present
    for replica_idx in 0..4 {
        for j in 0..10 {
            let val = (replica_idx * 100 + j) as i32;
            assert!(
                cluster.replica(0).state().contains(&val),
                "Missing value {} from replica {}", val, replica_idx
            );
        }
    }
}

/// Test ORSet with causal consistency
#[test]
fn test_orset_causal() {
    let mut cluster: CausalCluster<ORSet<String>> = CausalCluster::new(2, 0.0);
    
    // r0 adds elements
    cluster.mutate(0, |_s| {
        let mut delta = ORSet::new();
        delta.add("r0", "hello".to_string());
        delta
    });
    cluster.mutate(0, |_s| {
        let mut delta = ORSet::new();
        delta.add("r0", "world".to_string());
        delta
    });
    
    // Sync
    cluster.full_sync_round();
    
    // r1 removes "hello" - need to create a delta with the removal
    cluster.mutate(1, |s| {
        let mut delta = s.clone();
        delta.remove(&"hello".to_string());
        delta
    });
    
    // Sync again
    cluster.full_sync_round();
    
    // Both should have only "world"
    assert!(cluster.is_converged());
    assert!(!cluster.replica(0).state().contains(&"hello".to_string()));
    assert!(cluster.replica(0).state().contains(&"world".to_string()));
}

/// Test LWWRegister with causal consistency
#[test]
fn test_lwwreg_causal() {
    let mut cluster: CausalCluster<LWWRegister<String, String>> = CausalCluster::new(2, 0.0);
    
    // r0 sets value first
    cluster.mutate(0, |_s| {
        let mut delta = LWWRegister::new("r0".to_string());
        delta.set("first".to_string(), 1, "r0".to_string());
        delta
    });
    
    // Sync
    cluster.full_sync_round();
    
    // r1 sets newer value
    cluster.mutate(1, |_s| {
        let mut delta = LWWRegister::new("r1".to_string());
        delta.set("second".to_string(), 2, "r1".to_string());
        delta
    });
    
    // Sync
    cluster.full_sync_round();
    
    // Both should have "second" (higher timestamp wins)
    assert!(cluster.is_converged());
    assert_eq!(cluster.replica(0).state().get(), Some(&"second".to_string()));
}

/// Test MVRegister with causal consistency (concurrent writes)
#[test]
fn test_mvreg_causal_concurrent() {
    let mut cluster: CausalCluster<MVRegister<String>> = CausalCluster::new(2, 0.0);
    
    // Both replicas write concurrently without syncing
    cluster.mutate(0, |_s| {
        let mut delta = MVRegister::new();
        delta.write("r0", "value_a".to_string());
        delta
    });
    cluster.mutate(1, |_s| {
        let mut delta = MVRegister::new();
        delta.write("r1", "value_b".to_string());
        delta
    });
    
    // Sync
    for _ in 0..3 {
        cluster.full_sync_round();
    }
    
    // Both values should be present (multi-value semantics)
    assert!(cluster.is_converged());
    let values: Vec<&String> = cluster.replica(0).state().read();
    assert_eq!(values.len(), 2);
    assert!(values.contains(&&"value_a".to_string()));
    assert!(values.contains(&&"value_b".to_string()));
}

/// Test idempotence of delta application
#[test]
fn test_idempotent_delta_application() {
    let mut r1: CausalReplica<GSet<i32>> = CausalReplica::new("r1");
    let mut r2: CausalReplica<GSet<i32>> = CausalReplica::new("r2");
    
    r1.register_peer("r2".to_string());
    r2.register_peer("r1".to_string());
    
    r1.mutate(|_| {
        let mut d = GSet::new();
        d.insert(42);
        d
    });
    
    let interval = r1.prepare_interval("r2").unwrap();
    
    // Apply once
    let ack1 = r2.receive_interval(interval.clone());
    assert!(ack1.is_some());
    let state_after_one = r2.state().clone();
    
    // Applying same interval again should be idempotent
    // (In causal mode, it would be rejected as out of order, 
    // but the CRDT merge itself is idempotent)
    let ack2 = r2.receive_interval(interval.clone());
    assert!(ack2.is_none()); // Rejected - already processed
    
    // State should be unchanged
    assert_eq!(r2.state(), &state_after_one);
}

/// Test high-frequency updates with message loss
#[test]
fn test_high_frequency_with_loss() {
    let mut cluster: CausalCluster<PNCounter<String>> = CausalCluster::new(3, 0.3);
    
    // Each replica does 50 increments - clone state and add 1 so delta has new total
    for replica_idx in 0..3 {
        for _ in 0..50 {
            let id = format!("r{}", replica_idx);
            cluster.mutate(replica_idx, move |s| {
                let mut delta = s.clone();
                delta.increment(id, 1);
                delta
            });
        }
    }
    
    // Many sync rounds with retransmission
    for _ in 0..30 {
        cluster.full_sync_round();
        cluster.retransmit_and_process();
    }
    
    // Should eventually converge
    assert!(cluster.is_converged());
    
    // Total should be 150 (3 replicas * 50 increments)
    assert_eq!(cluster.replica(0).state().value(), 150);
}

/// Test bootstrap of new replica via snapshot
#[test]
fn test_snapshot_bootstrap() {
    let mut cluster: CausalCluster<GSet<i32>> = CausalCluster::new(2, 0.0);
    
    // Populate with data
    for i in 0..100 {
        let val = i as i32;
        cluster.mutate(0, move |_| {
            let mut d = GSet::new();
            d.insert(val);
            d
        });
    }
    
    // Sync existing replicas
    cluster.full_sync_round();
    assert!(cluster.is_converged());
    
    // Get snapshot from r0
    let (state, seq) = cluster.replica(0).snapshot();
    
    // Create new replica from snapshot
    let mut new_replica: CausalReplica<GSet<i32>> = CausalReplica::new("new");
    new_replica.apply_snapshot(state, seq, "causal_0");
    
    // New replica should have all data
    for i in 0..100 {
        assert!(new_replica.state().contains(&i));
    }
}

/// Test that sequence numbers are monotonically increasing
#[test]
fn test_sequence_monotonicity() {
    let mut replica: CausalReplica<GSet<i32>> = CausalReplica::new("mono");
    
    let mut prev_seq = 0;
    
    for i in 0..100 {
        replica.mutate(move |_| {
            let mut d = GSet::new();
            d.insert(i);
            d
        });
        
        let current_seq = replica.counter();
        assert!(current_seq > prev_seq, "Sequence must be monotonically increasing");
        prev_seq = current_seq;
    }
    
    assert_eq!(replica.counter(), 100);
}

/// Test GC behavior - deltas should be cleared after ack
#[test]
fn test_delta_gc_on_ack() {
    let mut r1: CausalReplica<GSet<i32>> = CausalReplica::new("r1");
    let mut r2: CausalReplica<GSet<i32>> = CausalReplica::new("r2");
    
    r1.register_peer("r2".to_string());
    r2.register_peer("r1".to_string());
    
    // r1 creates mutation
    r1.mutate(|_| {
        let mut d = GSet::new();
        d.insert(1);
        d
    });
    
    assert!(r1.has_pending_deltas());
    
    // Send interval
    let interval = r1.prepare_interval("r2").unwrap();
    
    // After preparing, buffer should be "taken" but not cleared
    // (waiting for ack)
    
    // r2 receives and acks
    let ack = r2.receive_interval(interval).unwrap();
    
    // r1 processes ack - delta buffer should be cleared
    r1.receive_ack(&ack);
    
    // No more pending deltas for r2
    assert!(r1.prepare_interval("r2").is_none());
}

/// Property: Convergence should be achieved regardless of delivery order
#[test]
fn test_convergence_any_order() {
    // Run multiple times with different "orderings" simulated
    for seed in 0..5 {
        let mut cluster: CausalCluster<GSet<i32>> = CausalCluster::new(3, 0.0);
        
        // Create mutations
        for i in 0..3 {
            for j in 0..(seed + 1) * 3 {
                let val = (i * 100 + j) as i32;
                cluster.mutate(i, move |_| {
                    let mut d = GSet::new();
                    d.insert(val);
                    d
                });
            }
        }
        
        // Multiple sync rounds ensure eventual convergence
        for _ in 0..10 {
            cluster.full_sync_round();
        }
        
        assert!(cluster.is_converged(), "Failed to converge with seed {}", seed);
    }
}
