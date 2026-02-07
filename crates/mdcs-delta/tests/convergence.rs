//! Convergence tests for delta-state CRDTs
//!
//! These tests verify that δ-CRDTs converge correctly under various
//! network conditions including message loss, duplication, and reordering.

use mdcs_core::gset::GSet;
use mdcs_core::lattice::Lattice;
use mdcs_core::lwwreg::LWWRegister;
use mdcs_core::mvreg::MVRegister;
use mdcs_core::orset::ORSet;
use mdcs_core::pncounter::PNCounter;
use mdcs_delta::anti_entropy::{AntiEntropyCluster, NetworkConfig};
use mdcs_delta::mutators::gset;
use rand::seq::SliceRandom;
use rand::SeedableRng;

// ============================================================================
// GSet Convergence Tests
// ============================================================================

#[test]
fn test_gset_convergence_perfect_network() {
    let mut cluster: AntiEntropyCluster<GSet<i32>> =
        AntiEntropyCluster::new(3, NetworkConfig::default());

    // Each replica adds unique elements
    for i in 0..3 {
        cluster.mutate(i, move |_| gset::insert_delta((i + 1) as i32 * 10));
    }

    // Sync
    cluster.full_sync_round();

    assert!(cluster.is_converged());
    assert_eq!(cluster.replica(0).state().len(), 3);
}

#[test]
fn test_gset_convergence_with_loss() {
    let mut cluster: AntiEntropyCluster<GSet<i32>> =
        AntiEntropyCluster::new(4, NetworkConfig::lossy(0.5));

    // Add elements to each replica
    for i in 0..4 {
        cluster.mutate(i, move |_| gset::insert_delta(i as i32));
    }

    // Sync with retransmission until convergence
    let mut rounds = 0;
    while !cluster.is_converged() && rounds < 50 {
        cluster.full_sync_round();
        cluster.retransmit_and_process();
        rounds += 1;
    }

    assert!(
        cluster.is_converged(),
        "Failed to converge after {} rounds",
        rounds
    );
    assert_eq!(cluster.replica(0).state().len(), 4);
}

#[test]
fn test_gset_convergence_with_duplication() {
    let mut cluster: AntiEntropyCluster<GSet<i32>> =
        AntiEntropyCluster::new(3, NetworkConfig::with_dups(0.8));

    for i in 0..3 {
        cluster.mutate(i, move |_| gset::insert_delta(i as i32 * 100));
    }

    // Even with high duplication, should converge quickly due to idempotence
    cluster.full_sync_round();
    cluster.retransmit_and_process();

    assert!(cluster.is_converged());
}

#[test]
fn test_gset_convergence_chaotic_network() {
    let mut cluster: AntiEntropyCluster<GSet<String>> =
        AntiEntropyCluster::new(5, NetworkConfig::chaotic());

    // Multiple concurrent additions
    let items = vec!["alpha", "beta", "gamma", "delta", "epsilon"];
    for (i, item) in items.iter().enumerate() {
        let item_owned = item.to_string();
        cluster.mutate(i, move |_| gset::insert_delta(item_owned));
    }

    // Sync until convergence
    let mut rounds = 0;
    while !cluster.is_converged() && rounds < 100 {
        cluster.full_sync_round();
        cluster.retransmit_and_process();
        rounds += 1;
    }

    assert!(cluster.is_converged());
    assert_eq!(cluster.replica(0).state().len(), 5);
}

// ============================================================================
// ORSet Convergence Tests
// ============================================================================

#[test]
fn test_orset_convergence_add_only() {
    let mut cluster: AntiEntropyCluster<ORSet<String>> =
        AntiEntropyCluster::new(3, NetworkConfig::default());

    // Add unique elements
    cluster.mutate(0, |_| {
        let mut set = ORSet::new();
        set.add("r0", "apple".to_string());
        set
    });
    cluster.mutate(1, |_| {
        let mut set = ORSet::new();
        set.add("r1", "banana".to_string());
        set
    });
    cluster.mutate(2, |_| {
        let mut set = ORSet::new();
        set.add("r2", "cherry".to_string());
        set
    });

    cluster.full_sync_round();

    assert!(cluster.is_converged());
    assert_eq!(cluster.replica(0).state().len(), 3);
}

#[test]
fn test_orset_convergence_add_remove() {
    let mut cluster: AntiEntropyCluster<ORSet<String>> =
        AntiEntropyCluster::new(2, NetworkConfig::default());

    // R0 adds item
    cluster.mutate(0, |_| {
        let mut set = ORSet::new();
        set.add("r0", "item".to_string());
        set
    });

    // Sync so R1 sees it
    cluster.full_sync_round();

    // R1 removes item (after seeing it)
    let state = cluster.replica(1).state().clone();
    cluster.mutate(1, move |_| {
        let mut set = state;
        set.remove(&"item".to_string());
        set
    });

    // R0 adds same item again (concurrent with remove)
    cluster.mutate(0, |_| {
        let mut set = ORSet::new();
        set.add("r0", "item".to_string());
        set
    });

    cluster.full_sync_round();

    // Add-wins: item should exist
    assert!(cluster.replica(0).state().contains(&"item".to_string()));
}

// ============================================================================
// PNCounter Convergence Tests
// ============================================================================

#[test]
fn test_pncounter_convergence_increments() {
    let mut c1: PNCounter<String> = PNCounter::new();
    let mut c2: PNCounter<String> = PNCounter::new();
    let mut c3: PNCounter<String> = PNCounter::new();

    // Different replicas increment
    c1.increment("r1".to_string(), 100);
    c2.increment("r2".to_string(), 50);
    c3.increment("r3".to_string(), 25);

    // Merge in different orders
    let merge1 = c1.join(&c2).join(&c3);
    let merge2 = c3.join(&c1).join(&c2);
    let merge3 = c2.join(&c3).join(&c1);

    assert_eq!(merge1.value(), 175);
    assert_eq!(merge1.value(), merge2.value());
    assert_eq!(merge2.value(), merge3.value());
}

#[test]
fn test_pncounter_convergence_mixed_ops() {
    let mut c1: PNCounter<String> = PNCounter::new();
    let mut c2: PNCounter<String> = PNCounter::new();

    // R1: +100, -20
    c1.increment("r1".to_string(), 100);
    c1.decrement("r1".to_string(), 20);

    // R2: +50, -30
    c2.increment("r2".to_string(), 50);
    c2.decrement("r2".to_string(), 30);

    let merged = c1.join(&c2);
    // (100 - 20) + (50 - 30) = 80 + 20 = 100
    assert_eq!(merged.value(), 100);
}

#[test]
fn test_pncounter_idempotence() {
    let mut counter: PNCounter<String> = PNCounter::new();
    counter.increment("r1".to_string(), 50);
    counter.decrement("r1".to_string(), 10);

    let once = counter.join(&counter);
    let twice = once.join(&counter);
    let thrice = twice.join(&counter);

    assert_eq!(counter.value(), once.value());
    assert_eq!(once.value(), twice.value());
    assert_eq!(twice.value(), thrice.value());
}

// ============================================================================
// LWW Register Convergence Tests
// ============================================================================

#[test]
fn test_lwwreg_convergence_clear_winner() {
    let mut r1: LWWRegister<i32, String> = LWWRegister::new("r1".to_string());
    let mut r2: LWWRegister<i32, String> = LWWRegister::new("r2".to_string());
    let mut r3: LWWRegister<i32, String> = LWWRegister::new("r3".to_string());

    r1.set(10, 100, "r1".to_string());
    r2.set(20, 200, "r2".to_string());
    r3.set(30, 150, "r3".to_string());

    // All merge orders should give same result
    let m1 = r1.join(&r2).join(&r3);
    let m2 = r3.join(&r1).join(&r2);
    let m3 = r2.join(&r3).join(&r1);

    assert_eq!(m1.get(), Some(&20)); // Highest timestamp
    assert_eq!(m1.get(), m2.get());
    assert_eq!(m2.get(), m3.get());
}

#[test]
fn test_lwwreg_convergence_tie_breaking() {
    let mut r_a: LWWRegister<String, String> = LWWRegister::new("a".to_string());
    let mut r_b: LWWRegister<String, String> = LWWRegister::new("b".to_string());
    let mut r_c: LWWRegister<String, String> = LWWRegister::new("c".to_string());

    // All same timestamp
    r_a.set("from_a".to_string(), 1000, "a".to_string());
    r_b.set("from_b".to_string(), 1000, "b".to_string());
    r_c.set("from_c".to_string(), 1000, "c".to_string());

    let merged = r_a.join(&r_b).join(&r_c);

    // "c" has highest replica ID
    assert_eq!(merged.get(), Some(&"from_c".to_string()));
}

// ============================================================================
// MV Register Convergence Tests
// ============================================================================

#[test]
fn test_mvreg_convergence_preserves_concurrent() {
    let mut r1: MVRegister<String> = MVRegister::new();
    let mut r2: MVRegister<String> = MVRegister::new();

    r1.write("r1", "version_a".to_string());
    r2.write("r2", "version_b".to_string());

    let merged = r1.join(&r2);

    // Both versions preserved
    assert_eq!(merged.len(), 2);
    let values = merged.read();
    assert!(values.contains(&&"version_a".to_string()));
    assert!(values.contains(&&"version_b".to_string()));
}

#[test]
fn test_mvreg_convergence_commutative() {
    let mut r1: MVRegister<i32> = MVRegister::new();
    let mut r2: MVRegister<i32> = MVRegister::new();
    let mut r3: MVRegister<i32> = MVRegister::new();

    r1.write("r1", 100);
    r2.write("r2", 200);
    r3.write("r3", 300);

    let m1 = r1.join(&r2).join(&r3);
    let m2 = r3.join(&r1).join(&r2);

    assert_eq!(m1.len(), m2.len());
}

// ============================================================================
// Cross-Type Integration Tests
// ============================================================================

#[test]
fn test_multi_type_document_convergence() {
    use mdcs_core::map::{CRDTMap, MapValue};

    let mut doc1: CRDTMap<String> = CRDTMap::new();
    let mut doc2: CRDTMap<String> = CRDTMap::new();

    // Server 1 updates
    doc1.put(
        "s1",
        "title".to_string(),
        MapValue::Text("Hello".to_string()),
    );
    doc1.put("s1", "count".to_string(), MapValue::Int(1));

    // Server 2 updates (concurrent)
    doc2.put(
        "s2",
        "title".to_string(),
        MapValue::Text("World".to_string()),
    );
    doc2.put(
        "s2",
        "author".to_string(),
        MapValue::Text("Bob".to_string()),
    );

    let m1 = doc1.join(&doc2);
    let m2 = doc2.join(&doc1);

    // Both merges have same keys
    assert!(m1.contains_key(&"title".to_string()));
    assert!(m1.contains_key(&"count".to_string()));
    assert!(m1.contains_key(&"author".to_string()));

    assert_eq!(
        m1.contains_key(&"title".to_string()),
        m2.contains_key(&"title".to_string())
    );
}

// ============================================================================
// Stress / Randomized Tests
// ============================================================================

#[test]
fn test_gset_random_delivery_order() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(12345);

    // Create deltas
    let mut deltas: Vec<GSet<i32>> = Vec::new();
    for i in 0..20 {
        deltas.push(gset::insert_delta(i));
    }

    // Apply in different random orders
    let mut results = Vec::new();
    for _ in 0..10 {
        let mut shuffled = deltas.clone();
        shuffled.shuffle(&mut rng);

        let mut state = GSet::bottom();
        for delta in shuffled {
            state.join_assign(&delta);
        }
        results.push(state);
    }

    // All results should be equal
    for state in &results {
        assert_eq!(state, &results[0]);
    }
}

#[test]
fn test_pncounter_random_delivery_order() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(54321);

    // Create updates from different replicas
    let mut updates: Vec<PNCounter<String>> = Vec::new();
    for i in 0..10 {
        let mut c = PNCounter::new();
        c.increment(format!("r{}", i), (i + 1) as u64 * 10);
        if i % 3 == 0 {
            c.decrement(format!("r{}", i), 5);
        }
        updates.push(c);
    }

    // Apply in random orders
    let mut results = Vec::new();
    for _ in 0..5 {
        let mut shuffled = updates.clone();
        shuffled.shuffle(&mut rng);

        let mut state = PNCounter::bottom();
        for update in shuffled {
            state.join_assign(&update);
        }
        results.push(state.value());
    }

    // All results should be equal
    for value in &results {
        assert_eq!(value, &results[0]);
    }
}
