//! MDCS Delta - Delta-state CRDT machinery
//!
//! This crate implements the δ-CRDT framework including:
//! - Delta buffers for grouping and batching
//! - Delta-mutators for each CRDT type
//! - Anti-entropy Algorithm 1 (convergence mode)

pub mod buffer;
pub mod mutators;
pub mod anti_entropy;

// Re-export main types
pub use buffer::{DeltaBuffer, DeltaReplica, AckTracker, TaggedDelta, SeqNo, ReplicaId};
pub use anti_entropy::{AntiEntropyCluster, AntiEntropyMessage, NetworkSimulator, NetworkConfig};

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          MDCS Delta - δ-CRDT Framework Demo               ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    demo_delta_buffer();
    demo_anti_entropy();
}

fn demo_delta_buffer() {
    use mdcs_core::gset::GSet;
    use buffer::DeltaReplica;

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  Demo 1: Delta Buffer with GSet                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    let mut replica: DeltaReplica<GSet<i32>> = DeltaReplica::new("demo_replica");

    println!("\nCreated replica: {}", replica.id);
    println!("Initial state: {:?}", replica.state());

    // Perform mutations using delta-mutators
    println!("\nPerforming mutations...");

    replica.mutate(|_| {
        let mut d = GSet::new();
        d.insert(1);
        d.insert(2);
        d.insert(3);
        d
    });
    println!("After insert [1,2,3]: {:?}", replica.state());
    println!("Buffer sequence: {}", replica.current_seq());

    replica.mutate(|_| {
        let mut d = GSet::new();
        d.insert(4);
        d.insert(5);
        d
    });
    println!("After insert [4,5]: {:?}", replica.state());
    println!("Buffer sequence: {}", replica.current_seq());

    println!("\n✓ Delta buffer demo complete!");
}

fn demo_anti_entropy() {
    use mdcs_core::gset::GSet;
    use anti_entropy::{AntiEntropyCluster, NetworkConfig};

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  Demo 2: Anti-Entropy Algorithm 1 (Convergence)              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    // Create cluster with 3 replicas
    let mut cluster: AntiEntropyCluster<GSet<i32>> =
        AntiEntropyCluster::new(3, NetworkConfig::default());

    println!("\nCreated cluster with {} replicas", cluster.len());

    // Each replica adds different elements
    println!("\nPerforming concurrent mutations...");

    cluster.mutate(0, |_| {
        let mut d = GSet::new();
        d.insert(10);
        d.insert(11);
        d
    });
    println!("Replica 0 added: [10, 11]");

    cluster.mutate(1, |_| {
        let mut d = GSet::new();
        d.insert(20);
        d.insert(21);
        d
    });
    println!("Replica 1 added: [20, 21]");

    cluster.mutate(2, |_| {
        let mut d = GSet::new();
        d.insert(30);
        d.insert(31);
        d
    });
    println!("Replica 2 added: [30, 31]");

    println!("\nBefore sync:");
    for i in 0..3 {
        println!("  Replica {}: {:?}", i, cluster.replica(i).state().iter().collect::<Vec<_>>());
    }
    println!("Converged: {}", cluster.is_converged());

    // Perform anti-entropy sync
    println!("\nRunning anti-entropy sync...");
    cluster.full_sync_round();

    println!("\nAfter sync:");
    for i in 0..3 {
        println!("  Replica {}: {:?}", i, cluster.replica(i).state().iter().collect::<Vec<_>>());
    }
    println!("Converged: {}", cluster.is_converged());

    println!("\n✓ Anti-entropy demo complete!");

    // Demo with lossy network
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  Demo 3: Convergence Under Network Loss                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    let mut lossy_cluster: AntiEntropyCluster<GSet<i32>> =
        AntiEntropyCluster::new(3, NetworkConfig::lossy(0.3));

    println!("\nCreated cluster with 30% message loss");

    for i in 0..3 {
        let val = (i + 1) as i32 * 100;
        lossy_cluster.mutate(i, move |_| {
            let mut d = GSet::new();
            d.insert(val);
            d
        });
    }

    println!("Each replica added one element...");

    let mut rounds = 0;
    while !lossy_cluster.is_converged() && rounds < 20 {
        lossy_cluster.full_sync_round();
        lossy_cluster.retransmit_and_process();
        rounds += 1;
    }

    println!("Converged after {} rounds", rounds);
    println!("Final state: {:?}", lossy_cluster.replica(0).state().iter().collect::<Vec<_>>());

    println!("\n✓ All demos complete!");
}

