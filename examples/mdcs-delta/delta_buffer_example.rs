//! Example: Using the Delta Buffer for CRDT Synchronization
//!
//! This example demonstrates:
//! 1. Delta-mutators: m(X) = X ⊔ mδ(X)
//! 2. Delta buffer with grouping
//! 3. Anti-entropy Algorithm 1 for convergence

use mdcs_core::gset::GSet;
use mdcs_core::orset::ORSet;
use mdcs_core::lattice::{Lattice, DeltaCRDT};
use mdcs_delta::buffer::{DeltaBuffer, DeltaReplica, AckTracker};
use mdcs_delta::anti_entropy::{AntiEntropyCluster, NetworkConfig};
use mdcs_delta::mutators::gset as gset_mutators;

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Delta Buffer Examples for MDCS (CRDT DB)");
    println!("═══════════════════════════════════════════════════════════════\n");

    example_1_delta_mutators();
    example_2_delta_buffer();
    example_3_anti_entropy_basic();
    example_4_convergence_under_failure();
    example_5_orset_with_deltas();

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  All examples completed successfully!");
    println!("═══════════════════════════════════════════════════════════════");
}

/// Example 1: Delta-mutators satisfy m(X) = X ⊔ mδ(X)
fn example_1_delta_mutators() {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 1: Delta-Mutator Property                          │");
    println!("│            m(X) = X ⊔ mδ(X)                                 │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    // Start with initial state
    let mut state: GSet<i32> = GSet::new();
    state.insert(1);
    state.insert(2);
    println!("Initial state X: {:?}", state.iter().collect::<Vec<_>>());

    // Method 1: Direct mutation
    let mut direct = state.clone();
    direct.insert(42);
    println!("Direct mutation m(X) = X.insert(42): {:?}", direct.iter().collect::<Vec<_>>());

    // Method 2: Via delta-mutator
    let delta = gset_mutators::insert_delta(42);
    println!("Delta mδ(X) = {{42}}: {:?}", delta.iter().collect::<Vec<_>>());

    let via_delta = state.join(&delta);
    println!("X ⊔ mδ(X): {:?}", via_delta.iter().collect::<Vec<_>>());

    // Verify the property
    assert_eq!(direct, via_delta, "Property m(X) = X ⊔ mδ(X) violated!");
    println!("\n✓ Property verified: m(X) = X ⊔ mδ(X)\n");
}

/// Example 2: Delta buffer with grouping and acknowledgments
fn example_2_delta_buffer() {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 2: Delta Buffer with Grouping                      │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    // Create a delta buffer with max size 5
    let mut buffer: DeltaBuffer<GSet<i32>> = DeltaBuffer::new(5);
    println!("Created buffer with max size 5");

    // Push several deltas
    for i in 1..=7 {
        let mut delta = GSet::new();
        delta.insert(i);
        buffer.push(delta);
        println!("Pushed delta {{{}}} - buffer seq: {}, len: {}",
            i, buffer.current_seq(), buffer.len());
    }

    // Get delta-group for a peer that has acked seq 3
    println!("\nPeer has acked up to seq 3");
    if let Some(group) = buffer.delta_group_since(3) {
        println!("Delta-group for peer: {:?}", group.iter().collect::<Vec<_>>());
    }

    // Acknowledge and garbage collect
    let removed = buffer.ack(5);
    println!("\nAfter ack(5): removed {} deltas, {} remaining", removed, buffer.len());

    println!("\n✓ Delta buffer demonstration complete\n");
}

/// Example 3: Basic anti-entropy synchronization
fn example_3_anti_entropy_basic() {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 3: Anti-Entropy Algorithm 1 (Basic)                │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    // Create 3 replicas with perfect network
    let mut cluster: AntiEntropyCluster<GSet<String>> =
        AntiEntropyCluster::new(3, NetworkConfig::default());

    println!("Created cluster with 3 replicas");

    // Concurrent mutations at different replicas
    cluster.mutate(0, |_| {
        let mut d = GSet::new();
        d.insert("apple".to_string());
        d.insert("banana".to_string());
        d
    });

    cluster.mutate(1, |_| {
        let mut d = GSet::new();
        d.insert("cherry".to_string());
        d
    });

    cluster.mutate(2, |_| {
        let mut d = GSet::new();
        d.insert("date".to_string());
        d.insert("elderberry".to_string());
        d
    });

    println!("\nAfter concurrent mutations:");
    for i in 0..3 {
        let items: Vec<_> = cluster.replica(i).state().iter().collect();
        println!("  Replica {}: {:?}", i, items);
    }
    println!("  Converged: {}", cluster.is_converged());

    // Run anti-entropy
    println!("\nRunning anti-entropy sync...");
    cluster.full_sync_round();

    println!("\nAfter sync:");
    for i in 0..3 {
        let items: Vec<_> = cluster.replica(i).state().iter().collect();
        println!("  Replica {}: {:?}", i, items);
    }
    println!("  Converged: {}", cluster.is_converged());

    println!("\n✓ Anti-entropy basic demonstration complete\n");
}

/// Example 4: Convergence under network failures
fn example_4_convergence_under_failure() {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 4: Convergence Under Network Failures              │");
    println!("│            (Loss, Duplication, Reordering)                 │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    // Test different failure modes
    let configs = vec![
        ("Perfect Network", NetworkConfig::default()),
        ("30% Loss", NetworkConfig::lossy(0.3)),
        ("50% Duplication", NetworkConfig::with_dups(0.5)),
        ("Chaotic (all failures)", NetworkConfig::chaotic()),
    ];

    for (name, config) in configs {
        let mut cluster: AntiEntropyCluster<GSet<i32>> =
            AntiEntropyCluster::new(4, config);

        // Each replica adds unique elements
        for i in 0..4 {
            let val = (i + 1) as i32 * 10;
            cluster.mutate(i, move |_| {
                let mut d = GSet::new();
                d.insert(val);
                d
            });
        }

        // Sync until convergence (with retransmission for lost messages)
        let mut rounds = 0;
        while !cluster.is_converged() && rounds < 30 {
            cluster.full_sync_round();
            cluster.retransmit_and_process();
            rounds += 1;
        }

        let converged = cluster.is_converged();
        let element_count = cluster.replica(0).state().len();

        println!("  {}: converged={}, rounds={}, elements={}",
            name, converged, rounds, element_count);

        assert!(converged, "Failed to converge with {}", name);
    }

    println!("\n✓ All network failure scenarios handled correctly\n");
}

/// Example 5: ORSet with delta operations
fn example_5_orset_with_deltas() {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 5: ORSet with Delta Operations                     │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    let mut set1: ORSet<String> = ORSet::new();
    let mut set2: ORSet<String> = ORSet::new();

    // Replica 1 adds elements
    set1.add("r1", "shared".to_string());
    set1.add("r1", "only_r1".to_string());
    println!("Replica 1 state: {:?}", set1.iter().collect::<Vec<_>>());

    // Replica 2 adds elements
    set2.add("r2", "shared".to_string());
    set2.add("r2", "only_r2".to_string());
    println!("Replica 2 state: {:?}", set2.iter().collect::<Vec<_>>());

    // Extract deltas
    let delta1 = set1.split_delta();
    let delta2 = set2.split_delta();
    println!("\nExtracted deltas from both replicas");

    // Apply deltas to each other (cross-sync)
    if let Some(d) = delta1 {
        set2.apply_delta(&d);
    }
    if let Some(d) = delta2 {
        set1.apply_delta(&d);
    }

    println!("\nAfter cross-applying deltas:");
    println!("Replica 1 state: {:?}", set1.iter().collect::<Vec<_>>());
    println!("Replica 2 state: {:?}", set2.iter().collect::<Vec<_>>());

    // Verify convergence (sets should have same keys, though tags differ)
    let keys1: std::collections::BTreeSet<_> = set1.iter().cloned().collect();
    let keys2: std::collections::BTreeSet<_> = set2.iter().cloned().collect();
    println!("\nKeys match: {}", keys1 == keys2);

    // Demonstrate idempotence - apply same delta again
    let mut set3 = set1.clone();
    let delta_again = set2.split_delta();
    if let Some(d) = delta_again {
        set3.apply_delta(&d);
    }
    // Re-apply by creating similar adds
    set3.add("r2", "shared".to_string()); // This creates new tags, but existing ones stay

    println!("\n✓ ORSet delta demonstration complete\n");
}

