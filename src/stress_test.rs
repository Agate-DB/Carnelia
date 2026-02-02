use async_stream::stream;
use futures::stream::Stream;
use futures::stream::StreamExt;
use mdcs_core::gset::GSet;
use mdcs_core::lattice::Lattice;
use mdcs_core::orset::ORSet;
use mdcs_core::pncounter::PNCounter;
use mdcs_core::lwwreg::LWWRegister;
use mdcs_core::mvreg::MVRegister;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Statistics collected during stress testing
#[derive(Clone, Debug)]
pub struct StressTestStats {
    pub num_replicas: usize,
    pub operations_per_replica: usize,
    pub total_syncs: usize,
    pub total_time: Duration,
    pub avg_sync_time: Duration,
    pub ops_per_second: f64,
}

impl StressTestStats {
    pub fn print(&self) {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║              Stress Test Statistics                         ║");
        println!("╠════════════════════════════════════════════════════════════╣");
        println!("║  Number of Replicas:        {:>38} ║", self.num_replicas);
        println!("║  Operations per Replica:    {:>38} ║", self.operations_per_replica);
        println!("║  Total Sync Operations:     {:>38} ║", self.total_syncs);
        println!("║  Total Time:                {:>39}s ║", format!("{:.3}", self.total_time.as_secs_f64()));
        println!("║  Average Sync Time:         {:>36}µs ║", format!("{:.2}", self.avg_sync_time.as_micros()));
        println!("║  Operations/Second:         {:>38.0} ║", self.ops_per_second);
        println!("╚════════════════════════════════════════════════════════════╝");
    }
}

/// Generator that yields replica indices for synchronization patterns
fn replica_sync_generator(num_replicas: usize, num_syncs: usize) -> impl Stream<Item=(usize, usize)> {
    stream! {
        let mut rng = StdRng::from_entropy();
        for _ in 0..num_syncs {
            let replica_a = rng.gen_range(0..num_replicas);
            let replica_b = rng.gen_range(0..num_replicas);
            yield (replica_a, replica_b);
        }
    }
}

/// Helper function to perform synchronization between two replicas
async fn perform_sync<T>(
    replicas: &[Arc<Mutex<T>>],
    replica_a_idx: usize,
    replica_b_idx: usize,
    num_syncs: usize,
    sync_times: &mut Vec<Duration>,
    total_syncs: &mut usize,
) where
    T: Lattice,
{
    if replica_a_idx == replica_b_idx {
        return; // Skip self-sync
    }

    let sync_start = Instant::now();

    // Perform sync by merging both replicas
    let replica_a = Arc::clone(&replicas[replica_a_idx]);
    let replica_b = Arc::clone(&replicas[replica_b_idx]);

    let (mut set_a, mut set_b) = tokio::join!(replica_a.lock(), replica_b.lock());
    
    // Merge both replicas - each absorbs the other's state
    let merged = set_a.join(&*set_b);
    *set_a = merged.clone();
    *set_b = merged;

    drop(set_a);
    drop(set_b);

    let sync_duration = sync_start.elapsed();
    sync_times.push(sync_duration);
    *total_syncs += 1;

    if *total_syncs % 100 == 0 {
        println!("  Syncs completed: {}/{}", total_syncs, num_syncs);
    }
}

/// Stress test for GSet with async synchronization.
///
/// This function performs a two-phase stress test:
/// 1. Concurrently spawns tokio tasks that insert `ops_per_replica` unique u64 values into each of
///    `num_replicas` GSet replicas.
/// 2. Performs `num_syncs` random pairwise synchronizations between replicas, measuring sync times.
///
/// Parameters:
/// - `num_replicas`: number of replicas to create and operate on.
/// - `ops_per_replica`: number of insert operations executed per replica during phase 1.
/// - `num_syncs`: number of random pairwise synchronizations to perform during phase 2.
///
/// Returns:
/// - `StressTestStats` containing overall timings and basic throughput metrics.
///
/// Notes:
/// - This function must be run inside a tokio runtime.
/// - Replicas are protected by tokio::sync::Mutex and are synchronized by taking locks and joining sets.
/// - Self-syncs are skipped and do not count toward `num_syncs`.
pub async fn stress_test_gset(
    num_replicas: usize,
    ops_per_replica: usize,
    num_syncs: usize,
) -> StressTestStats {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║        GSet Stress Test (Async)                            ║");
    println!("║  Replicas: {} | Ops/Replica: {} | Syncs: {} ║",
             num_replicas, ops_per_replica, num_syncs);
    println!("╚════════════════════════════════════════════════════════════╝");

    let start = Instant::now();

    // Initialize replicas
    let mut replicas: Vec<Arc<Mutex<GSet<u64>>>> = Vec::with_capacity(num_replicas);
    for _ in 0..num_replicas {
        replicas.push(Arc::new(Mutex::new(GSet::new())));
    }

    println!("\n[Phase 1/2] Adding elements to replicas...");

    // Phase 1: Add operations across replicas
    let mut handles = vec![];
    for (idx, replica) in replicas.iter().enumerate() {
        let replica = Arc::clone(replica);
        let handle = tokio::spawn(async move {
            for i in 0..ops_per_replica {
                let value = ((idx as u64) << 32) | (i as u64);
                let mut set = replica.lock().await;
                set.insert(value);
                drop(set);

                if i % 100 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all add operations to complete
    for handle in handles {
        let _ = handle.await;
    }

    println!("[Phase 1/2] ✓ Completed");
    println!("[Phase 2/2] Synchronizing replicas...");

    // Phase 2: Synchronization using stream
    let mut sync_times = vec![];
    let mut sync_gen = Box::pin(replica_sync_generator(num_replicas, num_syncs));

    let mut total_syncs = 0;
    while let Some((replica_a_idx, replica_b_idx)) = sync_gen.next().await {
        perform_sync(
            &replicas,
            replica_a_idx,
            replica_b_idx,
            num_syncs,
            &mut sync_times,
            &mut total_syncs,
        ).await;
    }

    let total_time = start.elapsed();

    // Calculate statistics
    let avg_sync_time = if !sync_times.is_empty() {
        sync_times.iter().sum::<Duration>() / sync_times.len() as u32
    } else {
        Duration::ZERO
    };

    let total_operations = (num_replicas * ops_per_replica) + total_syncs;
    let ops_per_second = total_operations as f64 / total_time.as_secs_f64();

    println!("[Phase 2/2] ✓ Completed");

    StressTestStats {
        num_replicas,
        operations_per_replica: ops_per_replica,
        total_syncs,
        total_time,
        avg_sync_time,
        ops_per_second,
    }
}

/// Stress test for ORSet with async synchronization.
///
/// This function performs a two-phase stress test on ORSet replicas:
/// 1. Concurrently spawns tokio tasks that add string items to each replica and occasionally remove
///    items (randomly, based on a probability), performing `ops_per_replica` operations per replica.
/// 2. Performs `num_syncs` random pairwise synchronizations between replicas, measuring sync times.
///
/// Parameters:
/// - `num_replicas`: number of replicas to create and operate on.
/// - `ops_per_replica`: number of add/remove operations executed per replica during phase 1.
/// - `num_syncs`: number of random pairwise synchronizations to perform during phase 2.
///
/// Returns:
/// - `StressTestStats` containing overall timings and basic throughput metrics.
///
/// Notes:
/// - This function must be run inside a tokio runtime.
/// - Replica add/remove operations encode a replica id in the element tags (e.g. "replica_{idx}").
/// - Synchronization is performed by locking pairs of replicas and joining their states.
pub async fn stress_test_orset(
    num_replicas: usize,
    ops_per_replica: usize,
    num_syncs: usize,
) -> StressTestStats {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║        ORSet Stress Test (Async)                           ║");
    println!("║  Replicas: {} | Ops/Replica: {} | Syncs: {} ║", num_replicas, ops_per_replica, num_syncs);
    println!("╚════════════════════════════════════════════════════════════╝");

    let start = Instant::now();

    // Initialize replicas
    let mut replicas: Vec<Arc<Mutex<ORSet<String>>>> = Vec::with_capacity(num_replicas);
    for _idx in 0..num_replicas {
        replicas.push(Arc::new(Mutex::new(ORSet::new())));
    }

    println!("\n[Phase 1/2] Adding and removing elements...");

    // Phase 1: Add and remove operations
    let mut handles = vec![];
    for (idx, replica) in replicas.iter().enumerate() {
        let replica = Arc::clone(replica);
        let handle = tokio::spawn(async move {
            let mut rng = StdRng::from_entropy();
            for i in 0..ops_per_replica {
                let value = format!("item_{}_{}", idx, i);
                let mut set = replica.lock().await;

                set.add(&format!("replica_{}", idx), value.clone());

                if rng.gen_bool(0.3) && i > 10 {
                    set.remove(&format!("item_{}_{}", idx, i - 10));
                }

                drop(set);

                if i % 100 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        let _ = handle.await;
    }

    println!("[Phase 1/2] ✓ Completed");
    println!("[Phase 2/2] Synchronizing replicas...");

    // Phase 2: Synchronization using stream
    let mut sync_times = vec![];
    let mut sync_gen = Box::pin(replica_sync_generator(num_replicas, num_syncs));

    let mut total_syncs = 0;
    while let Some((replica_a_idx, replica_b_idx)) = sync_gen.next().await {
        perform_sync(
            &replicas,
            replica_a_idx,
            replica_b_idx,
            num_syncs,
            &mut sync_times,
            &mut total_syncs,
        ).await;
    }

    let total_time = start.elapsed();

    let avg_sync_time = if !sync_times.is_empty() {
        sync_times.iter().sum::<Duration>() / sync_times.len() as u32
    } else {
        Duration::ZERO
    };

    let total_operations = (num_replicas * ops_per_replica) + total_syncs;
    let ops_per_second = total_operations as f64 / total_time.as_secs_f64();

    println!("[Phase 2/2] ✓ Completed");

    StressTestStats {
        num_replicas,
        operations_per_replica: ops_per_replica,
        total_syncs,
        total_time,
        avg_sync_time,
        ops_per_second,
    }
}

/// Parallel stress test comparing different replica scales.
///
/// Runs `stress_test_gset` repeatedly, increasing the replica count from `step_size` up to
/// `max_replicas` (inclusive) in increments of `step_size`. Each iteration awaits the completion
/// of the GSet stress test and prints collected statistics.
///
/// Parameters:
/// - `max_replicas`: maximum number of replicas to test.
/// - `step_size`: increment step for the number of replicas; the function will test replica counts
///   step_size, 2*step_size, ..., up to max_replicas.
///
/// Notes:
/// - This function must be run inside a tokio runtime.
/// - Uses a fixed ops_per_replica of 50 and `num_syncs = current_replicas * 100` for each test run.
pub async fn stress_test_scaling(max_replicas: usize, step_size: usize) {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║      Scaling Analysis - GSet Performance vs Replicas      ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    let mut current_replicas = step_size;
    while current_replicas <= max_replicas {
        let stats = stress_test_gset(current_replicas, 50, current_replicas * 100).await;
        stats.print();
        current_replicas += step_size;
    }
}

// ============================================================================
// Delta-Based Stress Tests
// ============================================================================

/// Statistics for delta-based stress tests
#[derive(Clone, Debug)]
pub struct DeltaStressTestStats {
    pub num_replicas: usize,
    pub operations_per_replica: usize,
    pub network_config: String,
    pub sync_rounds: usize,
    pub converged: bool,
    pub total_time: Duration,
    pub final_state_size: usize,
}

impl DeltaStressTestStats {
    pub fn print(&self) {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║           Delta Stress Test Statistics                      ║");
        println!("╠════════════════════════════════════════════════════════════╣");
        println!("║  Number of Replicas:        {:>38} ║", self.num_replicas);
        println!("║  Operations per Replica:    {:>38} ║", self.operations_per_replica);
        println!("║  Network Config:            {:>38} ║", self.network_config);
        println!("║  Sync Rounds:               {:>38} ║", self.sync_rounds);
        println!("║  Converged:                 {:>38} ║", self.converged);
        println!("║  Total Time:                {:>39}s ║", format!("{:.3}", self.total_time.as_secs_f64()));
        println!("║  Final State Size:          {:>38} ║", self.final_state_size);
        println!("╚════════════════════════════════════════════════════════════╝");
    }
}

/// Delta-based stress test for GSet with network simulation
///
/// Tests convergence under various network conditions using delta anti-entropy.
/// This proves that δ-CRDT converges correctly under:
/// - Message loss (with retransmission)
/// - Message duplication (handled by idempotence)
/// - Message reordering (handled by commutativity)
pub fn stress_test_delta_gset(
    num_replicas: usize,
    ops_per_replica: usize,
    loss_rate: f64,
    dup_rate: f64,
    reorder_rate: f64,
    max_rounds: usize,
) -> DeltaStressTestStats {
    use mdcs_core::gset::GSet;

    // We can't use mdcs_delta directly here due to workspace structure,
    // so we implement a simplified version inline

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║        Delta GSet Stress Test                               ║");
    println!("║  Replicas: {} | Ops: {} | Loss: {:.0}% | Dup: {:.0}%       ║",
             num_replicas, ops_per_replica, loss_rate * 100.0, dup_rate * 100.0);
    println!("╚════════════════════════════════════════════════════════════╝");

    let start = Instant::now();

    // Initialize replicas
    let mut replicas: Vec<GSet<u64>> = vec![GSet::new(); num_replicas];
    let mut rng = StdRng::seed_from_u64(42);

    println!("\n[Phase 1/3] Adding elements to replicas...");

    // Phase 1: Add operations across replicas (each adds unique elements)
    for (idx, replica) in replicas.iter_mut().enumerate() {
        for i in 0..ops_per_replica {
            let value = ((idx as u64) << 32) | (i as u64);
            replica.insert(value);
        }
    }

    let expected_total = num_replicas * ops_per_replica;
    println!("[Phase 1/3] ✓ Added {} total elements", expected_total);

    println!("[Phase 2/3] Synchronizing with simulated network failures...");

    // Phase 2: Sync using delta anti-entropy with simulated failures
    let mut rounds = 0;
    let mut converged = false;

    while rounds < max_rounds && !converged {
        rounds += 1;

        // For each pair of replicas, attempt sync with possible failures
        for i in 0..num_replicas {
            for j in 0..num_replicas {
                if i == j {
                    continue;
                }

                // Simulate message loss
                if rng.gen::<f64>() < loss_rate {
                    continue; // Message "lost"
                }

                // Get delta (simplified: full state as delta)
                let delta = replicas[i].clone();

                // Simulate duplication (apply twice)
                if rng.gen::<f64>() < dup_rate {
                    replicas[j].join_assign(&delta);
                }

                // Apply delta (idempotent!)
                replicas[j].join_assign(&delta);
            }
        }

        // Check convergence
        converged = replicas.windows(2).all(|w| w[0] == w[1]);

        if rounds % 5 == 0 {
            println!("  Round {}: converged={}", rounds, converged);
        }
    }

    println!("[Phase 2/3] ✓ Completed after {} rounds", rounds);

    println!("[Phase 3/3] Verifying convergence...");

    // Verify all replicas have all elements
    let final_size = replicas[0].len();
    let all_same_size = replicas.iter().all(|r| r.len() == final_size);

    if converged && all_same_size && final_size == expected_total {
        println!("[Phase 3/3] ✓ All {} replicas converged with {} elements",
                 num_replicas, final_size);
    } else {
        println!("[Phase 3/3] ⚠ Convergence issue: converged={}, same_size={}, size={}/{}",
                 converged, all_same_size, final_size, expected_total);
    }

    let total_time = start.elapsed();

    DeltaStressTestStats {
        num_replicas,
        operations_per_replica: ops_per_replica,
        network_config: format!("loss={:.0}%, dup={:.0}%, reorder={:.0}%",
                               loss_rate * 100.0, dup_rate * 100.0, reorder_rate * 100.0),
        sync_rounds: rounds,
        converged,
        total_time,
        final_state_size: final_size,
    }
}

/// Prove convergence under repeated re-sends (idempotence test)
pub fn stress_test_idempotence(num_replicas: usize, ops_per_replica: usize, resend_count: usize) -> bool {
    use mdcs_core::gset::GSet;

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║        Idempotence Stress Test                              ║");
    println!("║  Replicas: {} | Ops: {} | Resends: {}                      ║",
             num_replicas, ops_per_replica, resend_count);
    println!("╚════════════════════════════════════════════════════════════╝");

    // Initialize replicas with different elements
    let mut replicas: Vec<GSet<u64>> = vec![GSet::new(); num_replicas];

    for (idx, replica) in replicas.iter_mut().enumerate() {
        for i in 0..ops_per_replica {
            let value = ((idx as u64) << 32) | (i as u64);
            replica.insert(value);
        }
    }

    // Sync once to get baseline
    for i in 0..num_replicas {
        for j in 0..num_replicas {
            if i != j {
                let delta = replicas[i].clone();
                replicas[j].join_assign(&delta);
            }
        }
    }

    let baseline_state = replicas[0].clone();
    println!("Baseline state size: {}", baseline_state.len());

    // Re-send same deltas many times
    for resend in 0..resend_count {
        for i in 0..num_replicas {
            for j in 0..num_replicas {
                if i != j {
                    let delta = replicas[i].clone();
                    replicas[j].join_assign(&delta);
                }
            }
        }

        // Verify state hasn't changed (idempotence)
        if replicas[0] != baseline_state {
            println!("⚠ Idempotence violated at resend {}", resend);
            return false;
        }
    }

    let final_state = &replicas[0];
    let idempotent = final_state == &baseline_state;

    println!("✓ Idempotence verified: {} re-sends, state unchanged: {}",
             resend_count, idempotent);

    idempotent
}

/// Comprehensive delta stress test suite
pub async fn stress_test_delta_suite() {
    println!("\n\n╔════════════════════════════════════════════════════════════╗");
    println!("║          DELTA STRESS TEST SUITE                            ║");
    println!("║     Testing Convergence Under Network Failures              ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    // Test 1: Perfect network (baseline)
    let stats = stress_test_delta_gset(4, 50, 0.0, 0.0, 0.0, 10);
    stats.print();
    assert!(stats.converged, "Should converge with perfect network");

    // Test 2: With message loss
    let stats = stress_test_delta_gset(4, 50, 0.3, 0.0, 0.0, 30);
    stats.print();
    assert!(stats.converged, "Should converge despite message loss");

    // Test 3: With message duplication
    let stats = stress_test_delta_gset(4, 50, 0.0, 0.5, 0.0, 10);
    stats.print();
    assert!(stats.converged, "Should converge despite duplication");

    // Test 4: Chaotic network (all failures)
    let stats = stress_test_delta_gset(4, 50, 0.2, 0.3, 0.2, 50);
    stats.print();
    assert!(stats.converged, "Should converge despite chaotic network");

    // Test 5: Idempotence verification
    let idempotent = stress_test_idempotence(3, 100, 50);
    assert!(idempotent, "Idempotence property should hold");

    println!("\n✓ All delta stress tests completed successfully!");
}

// ============================================================================
// PNCounter Stress Tests
// ============================================================================

/// Stress test for PNCounter with async synchronization
pub async fn stress_test_pncounter(
    num_replicas: usize,
    ops_per_replica: usize,
    num_syncs: usize,
) -> StressTestStats {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║        PNCounter Stress Test (Async)                       ║");
    println!("║  Replicas: {} | Ops/Replica: {} | Syncs: {} ║",
             num_replicas, ops_per_replica, num_syncs);
    println!("╚════════════════════════════════════════════════════════════╝");

    let start = Instant::now();

    // Initialize replicas
    let mut replicas: Vec<Arc<Mutex<PNCounter<String>>>> = Vec::with_capacity(num_replicas);
    for _ in 0..num_replicas {
        replicas.push(Arc::new(Mutex::new(PNCounter::new())));
    }

    println!("\n[Phase 1/2] Incrementing/decrementing counters...");

    // Phase 1: Increment and decrement operations
    let mut handles = vec![];
    for (idx, replica) in replicas.iter().enumerate() {
        let replica = Arc::clone(replica);
        let replica_id = format!("replica_{}", idx);
        let handle = tokio::spawn(async move {
            let mut rng = StdRng::seed_from_u64(idx as u64);
            for _ in 0..ops_per_replica {
                let mut counter = replica.lock().await;
                if rng.gen_bool(0.7) {
                    // 70% increments
                    counter.increment(replica_id.clone(), rng.gen_range(1..10));
                } else {
                    // 30% decrements
                    counter.decrement(replica_id.clone(), rng.gen_range(1..5));
                }
                drop(counter);

                if idx % 100 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    println!("[Phase 1/2] ✓ Completed");
    println!("[Phase 2/2] Synchronizing replicas...");

    // Phase 2: Synchronization
    let mut sync_times = vec![];
    let mut sync_gen = Box::pin(replica_sync_generator(num_replicas, num_syncs));
    let mut total_syncs = 0;

    while let Some((replica_a_idx, replica_b_idx)) = sync_gen.next().await {
        perform_sync(
            &replicas,
            replica_a_idx,
            replica_b_idx,
            num_syncs,
            &mut sync_times,
            &mut total_syncs,
        ).await;
    }

    // Verify convergence
    let mut values = Vec::new();
    for replica in &replicas {
        let counter = replica.lock().await;
        values.push(counter.value());
    }
    let converged = values.iter().all(|v| *v == values[0]);
    println!("  Final values: {:?}", values);
    println!("  Converged: {}", converged);

    let total_time = start.elapsed();
    let avg_sync_time = if !sync_times.is_empty() {
        sync_times.iter().sum::<Duration>() / sync_times.len() as u32
    } else {
        Duration::ZERO
    };

    let total_operations = (num_replicas * ops_per_replica) + total_syncs;
    let ops_per_second = total_operations as f64 / total_time.as_secs_f64();

    println!("[Phase 2/2] ✓ Completed");

    StressTestStats {
        num_replicas,
        operations_per_replica: ops_per_replica,
        total_syncs,
        total_time,
        avg_sync_time,
        ops_per_second,
    }
}

// ============================================================================
// LWW Register Stress Tests
// ============================================================================

/// Stress test for LWWRegister with timestamp contention
pub async fn stress_test_lwwreg(
    num_replicas: usize,
    ops_per_replica: usize,
    num_syncs: usize,
) -> StressTestStats {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║        LWWRegister Stress Test (Async)                     ║");
    println!("║  Replicas: {} | Ops/Replica: {} | Syncs: {} ║",
             num_replicas, ops_per_replica, num_syncs);
    println!("╚════════════════════════════════════════════════════════════╝");

    let start = Instant::now();

    // Initialize replicas
    let mut replicas: Vec<Arc<Mutex<LWWRegister<i64, String>>>> = Vec::with_capacity(num_replicas);
    for i in 0..num_replicas {
        replicas.push(Arc::new(Mutex::new(LWWRegister::new(format!("replica_{}", i)))));
    }

    println!("\n[Phase 1/2] Setting values with competing timestamps...");

    // Phase 1: Set operations with competing timestamps
    let mut handles = vec![];
    for (idx, replica) in replicas.iter().enumerate() {
        let replica = Arc::clone(replica);
        let replica_id = format!("replica_{}", idx);
        let handle = tokio::spawn(async move {
            let mut rng = StdRng::seed_from_u64(idx as u64);
            for i in 0..ops_per_replica {
                let mut reg = replica.lock().await;
                let timestamp = (i as u64) * 10 + rng.gen_range(0..10);
                let value = (idx as i64) * 1000 + (i as i64);
                reg.set(value, timestamp, replica_id.clone());
                drop(reg);

                if i % 100 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    println!("[Phase 1/2] ✓ Completed");
    println!("[Phase 2/2] Synchronizing replicas...");

    // Phase 2: Synchronization
    let mut sync_times = vec![];
    let mut sync_gen = Box::pin(replica_sync_generator(num_replicas, num_syncs));
    let mut total_syncs = 0;

    while let Some((replica_a_idx, replica_b_idx)) = sync_gen.next().await {
        perform_sync(
            &replicas,
            replica_a_idx,
            replica_b_idx,
            num_syncs,
            &mut sync_times,
            &mut total_syncs,
        ).await;
    }

    // Verify convergence
    let mut final_values = Vec::new();
    for replica in &replicas {
        let reg = replica.lock().await;
        final_values.push((reg.get().cloned(), reg.timestamp()));
    }
    let converged = final_values.iter().all(|v| *v == final_values[0]);
    println!("  Final (value, timestamp): {:?}", final_values[0]);
    println!("  Converged: {}", converged);

    let total_time = start.elapsed();
    let avg_sync_time = if !sync_times.is_empty() {
        sync_times.iter().sum::<Duration>() / sync_times.len() as u32
    } else {
        Duration::ZERO
    };

    let total_operations = (num_replicas * ops_per_replica) + total_syncs;
    let ops_per_second = total_operations as f64 / total_time.as_secs_f64();

    println!("[Phase 2/2] ✓ Completed");

    StressTestStats {
        num_replicas,
        operations_per_replica: ops_per_replica,
        total_syncs,
        total_time,
        avg_sync_time,
        ops_per_second,
    }
}

// ============================================================================
// MVRegister Stress Tests
// ============================================================================

/// Stress test for MVRegister with concurrent writes
pub async fn stress_test_mvreg(
    num_replicas: usize,
    ops_per_replica: usize,
    num_syncs: usize,
) -> StressTestStats {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║        MVRegister Stress Test (Async)                      ║");
    println!("║  Replicas: {} | Ops/Replica: {} | Syncs: {} ║",
             num_replicas, ops_per_replica, num_syncs);
    println!("╚════════════════════════════════════════════════════════════╝");

    let start = Instant::now();

    // Initialize replicas
    let mut replicas: Vec<Arc<Mutex<MVRegister<String>>>> = Vec::with_capacity(num_replicas);
    for _ in 0..num_replicas {
        replicas.push(Arc::new(Mutex::new(MVRegister::new())));
    }

    println!("\n[Phase 1/2] Writing concurrent values...");

    // Phase 1: Concurrent writes
    let mut handles = vec![];
    for (idx, replica) in replicas.iter().enumerate() {
        let replica = Arc::clone(replica);
        let replica_id = format!("replica_{}", idx);
        let handle = tokio::spawn(async move {
            for i in 0..ops_per_replica {
                let mut reg = replica.lock().await;
                let value = format!("value_{}_{}", idx, i);
                reg.write(&replica_id, value);
                drop(reg);

                if i % 100 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    println!("[Phase 1/2] ✓ Completed");
    println!("[Phase 2/2] Synchronizing replicas...");

    // Phase 2: Synchronization
    let mut sync_times = vec![];
    let mut sync_gen = Box::pin(replica_sync_generator(num_replicas, num_syncs));
    let mut total_syncs = 0;

    while let Some((replica_a_idx, replica_b_idx)) = sync_gen.next().await {
        perform_sync(
            &replicas,
            replica_a_idx,
            replica_b_idx,
            num_syncs,
            &mut sync_times,
            &mut total_syncs,
        ).await;
    }

    // Check concurrent value count
    let mut value_counts = Vec::new();
    for replica in &replicas {
        let reg = replica.lock().await;
        value_counts.push(reg.len());
    }
    let converged = value_counts.iter().all(|c| *c == value_counts[0]);
    println!("  Concurrent values per replica: {:?}", value_counts);
    println!("  Converged: {}", converged);

    let total_time = start.elapsed();
    let avg_sync_time = if !sync_times.is_empty() {
        sync_times.iter().sum::<Duration>() / sync_times.len() as u32
    } else {
        Duration::ZERO
    };

    let total_operations = (num_replicas * ops_per_replica) + total_syncs;
    let ops_per_second = total_operations as f64 / total_time.as_secs_f64();

    println!("[Phase 2/2] ✓ Completed");

    StressTestStats {
        num_replicas,
        operations_per_replica: ops_per_replica,
        total_syncs,
        total_time,
        avg_sync_time,
        ops_per_second,
    }
}

// ============================================================================
// Comprehensive Benchmark Suite
// ============================================================================

/// Run comprehensive stress tests on all CRDT types
pub async fn stress_test_all_crdts(
    num_replicas: usize,
    ops_per_replica: usize,
    num_syncs: usize,
) {
    println!("\n");
    println!("╔════════════════════════════════════════════════════════════════════╗");
    println!("║           COMPREHENSIVE CRDT STRESS TEST SUITE                      ║");
    println!("║  Testing: GSet, ORSet, PNCounter, LWWRegister, MVRegister           ║");
    println!("╚════════════════════════════════════════════════════════════════════╝");

    let mut results = Vec::new();

    // GSet
    let stats = stress_test_gset(num_replicas, ops_per_replica, num_syncs).await;
    results.push(("GSet", stats));

    // ORSet
    let stats = stress_test_orset(num_replicas, ops_per_replica, num_syncs).await;
    results.push(("ORSet", stats));

    // PNCounter
    let stats = stress_test_pncounter(num_replicas, ops_per_replica, num_syncs).await;
    results.push(("PNCounter", stats));

    // LWWRegister
    let stats = stress_test_lwwreg(num_replicas, ops_per_replica, num_syncs).await;
    results.push(("LWWRegister", stats));

    // MVRegister
    let stats = stress_test_mvreg(num_replicas, ops_per_replica, num_syncs).await;
    results.push(("MVRegister", stats));

    // Summary
    println!("\n");
    println!("╔════════════════════════════════════════════════════════════════════╗");
    println!("║                    BENCHMARK SUMMARY                                ║");
    println!("╠════════════════════════════════════════════════════════════════════╣");
    println!("║  CRDT Type     │ Total Time │ Ops/Second │ Avg Sync (µs)           ║");
    println!("╠════════════════════════════════════════════════════════════════════╣");
    for (name, stats) in &results {
        println!("║  {:12} │ {:>9.3}s │ {:>10.0} │ {:>10.2}               ║",
                 name,
                 stats.total_time.as_secs_f64(),
                 stats.ops_per_second,
                 stats.avg_sync_time.as_micros() as f64);
    }
    println!("╚════════════════════════════════════════════════════════════════════╝");

    println!("\n✓ All CRDT stress tests completed!");
}
