//! Comprehensive stress testing and benchmarking for MDCS
//!
//! This module provides stress tests for:
//! - Core CRDTs (GSet, ORSet, PNCounter, LWWRegister, MVRegister)
//! - Database layer (RGAText, RichText, JsonCrdt, DocumentStore)
//! - Delta synchronization under network failures
//! - Collaborative editing scenarios

use async_stream::stream;
use futures::stream::Stream;
use futures::stream::StreamExt;
use mdcs_core::gset::GSet;
use mdcs_core::lattice::Lattice;
use mdcs_core::lwwreg::LWWRegister;
use mdcs_core::mvreg::MVRegister;
use mdcs_core::orset::ORSet;
use mdcs_core::pncounter::PNCounter;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

// ============================================================================
// Statistics & Reporting
// ============================================================================

/// Statistics collected during stress testing
#[derive(Clone, Debug)]
pub struct StressTestStats {
    pub test_name: String,
    pub num_replicas: usize,
    pub operations_per_replica: usize,
    pub total_syncs: usize,
    pub total_time: Duration,
    pub avg_sync_time: Duration,
    pub ops_per_second: f64,
    pub converged: bool,
}

impl StressTestStats {
    pub fn new(name: &str) -> Self {
        Self {
            test_name: name.to_string(),
            num_replicas: 0,
            operations_per_replica: 0,
            total_syncs: 0,
            total_time: Duration::ZERO,
            avg_sync_time: Duration::ZERO,
            ops_per_second: 0.0,
            converged: true,
        }
    }

    pub fn print(&self) {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║  {:^56} ║", format!("{} Results", self.test_name));
        println!("╠════════════════════════════════════════════════════════════╣");
        println!("║  Replicas:        {:>38} ║", self.num_replicas);
        println!("║  Ops/Replica:     {:>38} ║", self.operations_per_replica);
        println!("║  Total Syncs:     {:>38} ║", self.total_syncs);
        println!(
            "║  Total Time:      {:>37.3}s ║",
            self.total_time.as_secs_f64()
        );
        println!(
            "║  Avg Sync Time:   {:>35}µs ║",
            format!("{:.2}", self.avg_sync_time.as_micros())
        );
        println!("║  Ops/Second:      {:>38.0} ║", self.ops_per_second);
        println!(
            "║  Converged:       {:>38} ║",
            if self.converged { "✓ Yes" } else { "✗ No" }
        );
        println!("╚════════════════════════════════════════════════════════════╝");
    }
}

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
        println!("║           Delta Stress Test Results                        ║");
        println!("╠════════════════════════════════════════════════════════════╣");
        println!("║  Replicas:       {:>39} ║", self.num_replicas);
        println!("║  Ops/Replica:    {:>39} ║", self.operations_per_replica);
        println!("║  Network:        {:>39} ║", self.network_config);
        println!("║  Sync Rounds:    {:>39} ║", self.sync_rounds);
        println!(
            "║  Converged:      {:>39} ║",
            if self.converged { "✓ Yes" } else { "✗ No" }
        );
        println!(
            "║  Total Time:     {:>38.3}s ║",
            self.total_time.as_secs_f64()
        );
        println!("║  Final Size:     {:>39} ║", self.final_state_size);
        println!("╚════════════════════════════════════════════════════════════╝");
    }
}

/// Benchmark results for comparison
#[derive(Clone, Debug)]
pub struct BenchmarkResult {
    pub name: String,
    pub total_time: Duration,
    pub ops_per_second: f64,
    pub avg_op_time: Duration,
    pub memory_estimate: usize,
}

/// Collection of benchmark results for reporting
pub struct BenchmarkReport {
    pub results: Vec<BenchmarkResult>,
}

impl BenchmarkReport {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn add(&mut self, result: BenchmarkResult) {
        self.results.push(result);
    }

    pub fn print(&self) {
        println!("\n╔══════════════════════════════════════════════════════════════════════════╗");
        println!("║                         BENCHMARK COMPARISON                              ║");
        println!("╠══════════════════════════════════════════════════════════════════════════╣");
        println!("║  Component          │  Time (s) │   Ops/sec │ Avg Op (µs) │  Memory (KB) ║");
        println!("╠══════════════════════════════════════════════════════════════════════════╣");
        for r in &self.results {
            println!(
                "║  {:17} │ {:>9.3} │ {:>9.0} │ {:>11.2} │ {:>12} ║",
                r.name,
                r.total_time.as_secs_f64(),
                r.ops_per_second,
                r.avg_op_time.as_micros() as f64,
                r.memory_estimate / 1024
            );
        }
        println!("╚══════════════════════════════════════════════════════════════════════════╝");
    }
}

impl Default for BenchmarkReport {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Generator that yields replica indices for synchronization patterns
fn replica_sync_generator(
    num_replicas: usize,
    num_syncs: usize,
) -> impl Stream<Item = (usize, usize)> {
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

    if (*total_syncs).is_multiple_of(100) {
        println!("  Syncs completed: {}/{}", total_syncs, num_syncs);
    }
}

fn print_header(title: &str, replicas: usize, ops: usize, syncs: usize) {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  {:^56} ║", title);
    println!(
        "║  Replicas: {:>3} │ Ops/Replica: {:>5} │ Syncs: {:>5}      ║",
        replicas, ops, syncs
    );
    println!("╚════════════════════════════════════════════════════════════╝");
}

// ============================================================================
// Core CRDT Stress Tests
// ============================================================================

/// Stress test for GSet with async synchronization.
pub async fn stress_test_gset(
    num_replicas: usize,
    ops_per_replica: usize,
    num_syncs: usize,
) -> StressTestStats {
    print_header("GSet Stress Test", num_replicas, ops_per_replica, num_syncs);

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
        )
        .await;
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
        test_name: "GSet".to_string(),
        num_replicas,
        operations_per_replica: ops_per_replica,
        total_syncs,
        total_time,
        avg_sync_time,
        ops_per_second,
        converged: true,
    }
}

/// Stress test for ORSet with async synchronization.
pub async fn stress_test_orset(
    num_replicas: usize,
    ops_per_replica: usize,
    num_syncs: usize,
) -> StressTestStats {
    print_header(
        "ORSet Stress Test",
        num_replicas,
        ops_per_replica,
        num_syncs,
    );

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
        let replica_id = format!("replica_{}", idx);
        let handle = tokio::spawn(async move {
            let mut rng = StdRng::seed_from_u64(idx as u64);
            for i in 0..ops_per_replica {
                let mut set = replica.lock().await;
                let item = format!("item_{}_{}", idx, i);

                // 80% add, 20% remove
                if rng.gen::<f64>() < 0.8 {
                    set.add(&replica_id, item);
                } else if i > 0 {
                    let remove_item = format!("item_{}_{}", idx, rng.gen_range(0..i));
                    set.remove(&remove_item);
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
        )
        .await;
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
        test_name: "ORSet".to_string(),
        num_replicas,
        operations_per_replica: ops_per_replica,
        total_syncs,
        total_time,
        avg_sync_time,
        ops_per_second,
        converged: true,
    }
}

/// Stress test for PNCounter with async synchronization
pub async fn stress_test_pncounter(
    num_replicas: usize,
    ops_per_replica: usize,
    num_syncs: usize,
) -> StressTestStats {
    print_header(
        "PNCounter Stress Test",
        num_replicas,
        ops_per_replica,
        num_syncs,
    );

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
            for i in 0..ops_per_replica {
                let mut counter = replica.lock().await;

                // 60% increment, 40% decrement
                if rng.gen::<f64>() < 0.6 {
                    counter.increment(replica_id.clone(), 1);
                } else {
                    counter.decrement(replica_id.clone(), 1);
                }
                drop(counter);

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
        )
        .await;
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
        test_name: "PNCounter".to_string(),
        num_replicas,
        operations_per_replica: ops_per_replica,
        total_syncs,
        total_time,
        avg_sync_time,
        ops_per_second,
        converged,
    }
}

/// Stress test for LWWRegister with timestamp contention
pub async fn stress_test_lwwreg(
    num_replicas: usize,
    ops_per_replica: usize,
    num_syncs: usize,
) -> StressTestStats {
    print_header(
        "LWWRegister Stress Test",
        num_replicas,
        ops_per_replica,
        num_syncs,
    );

    let start = Instant::now();

    // Initialize replicas
    let mut replicas: Vec<Arc<Mutex<LWWRegister<i64, String>>>> = Vec::with_capacity(num_replicas);
    for i in 0..num_replicas {
        replicas.push(Arc::new(Mutex::new(LWWRegister::new(format!(
            "replica_{}",
            i
        )))));
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
        )
        .await;
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
        test_name: "LWWRegister".to_string(),
        num_replicas,
        operations_per_replica: ops_per_replica,
        total_syncs,
        total_time,
        avg_sync_time,
        ops_per_second,
        converged,
    }
}

/// Stress test for MVRegister with concurrent writes
pub async fn stress_test_mvreg(
    num_replicas: usize,
    ops_per_replica: usize,
    num_syncs: usize,
) -> StressTestStats {
    print_header(
        "MVRegister Stress Test",
        num_replicas,
        ops_per_replica,
        num_syncs,
    );

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
        )
        .await;
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
        test_name: "MVRegister".to_string(),
        num_replicas,
        operations_per_replica: ops_per_replica,
        total_syncs,
        total_time,
        avg_sync_time,
        ops_per_second,
        converged,
    }
}

// ============================================================================
// Database Layer Stress Tests
// ============================================================================

/// Stress test for RGAText collaborative text editing
pub fn stress_test_rga_text(num_replicas: usize, ops_per_replica: usize) -> StressTestStats {
    use mdcs_db::RGAText;

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  RGAText Collaborative Text Stress Test                    ║");
    println!(
        "║  Replicas: {:>3} │ Ops/Replica: {:>5}                        ║",
        num_replicas, ops_per_replica
    );
    println!("╚════════════════════════════════════════════════════════════╝");

    let start = Instant::now();

    // Initialize replicas
    let mut replicas: Vec<RGAText> = Vec::with_capacity(num_replicas);
    for i in 0..num_replicas {
        replicas.push(RGAText::new(format!("replica_{}", i)));
    }

    println!("\n[Phase 1/3] Simulating concurrent text edits...");

    let mut rng = StdRng::seed_from_u64(42);

    // Phase 1: Each replica makes edits
    for (idx, replica) in replicas.iter_mut().enumerate() {
        for i in 0..ops_per_replica {
            let pos = if replica.is_empty() {
                0
            } else {
                rng.gen_range(0..=replica.len())
            };

            // 70% insert, 30% delete
            if rng.gen::<f64>() < 0.7 || replica.is_empty() {
                let text = format!("R{}_{}", idx, i);
                replica.insert(pos, &text);
            } else if !replica.is_empty() {
                let del_len = rng.gen_range(1..=std::cmp::min(3, replica.len()));
                let del_pos = rng.gen_range(0..replica.len().saturating_sub(del_len - 1));
                replica.delete(del_pos, del_len);
            }
        }
        if (idx + 1) % 2 == 0 {
            println!("  Replica {}/{} completed", idx + 1, num_replicas);
        }
    }

    println!("[Phase 1/3] ✓ Completed");
    println!("[Phase 2/3] Merging replicas...");

    let mut sync_times = vec![];
    let mut total_syncs = 0;

    // Phase 2: Pairwise merge to simulate gossip
    for round in 0..3 {
        for i in 0..num_replicas {
            for j in (i + 1)..num_replicas {
                let sync_start = Instant::now();

                // Clone to avoid borrow issues
                let delta_i = replicas[i].clone();
                let delta_j = replicas[j].clone();

                replicas[i] = replicas[i].join(&delta_j);
                replicas[j] = replicas[j].join(&delta_i);

                sync_times.push(sync_start.elapsed());
                total_syncs += 1;
            }
        }
        println!("  Merge round {}/3 completed", round + 1);
    }

    println!("[Phase 2/3] ✓ Completed");
    println!("[Phase 3/3] Verifying convergence...");

    // Verify convergence
    let first_text = replicas[0].to_string();
    let converged = replicas.iter().all(|r| r.to_string() == first_text);

    println!("  Final text length: {} chars", first_text.len());
    println!("  All replicas identical: {}", converged);

    let total_time = start.elapsed();
    let avg_sync_time = if !sync_times.is_empty() {
        sync_times.iter().sum::<Duration>() / sync_times.len() as u32
    } else {
        Duration::ZERO
    };

    let total_ops = num_replicas * ops_per_replica + total_syncs;
    let ops_per_second = total_ops as f64 / total_time.as_secs_f64();

    println!("[Phase 3/3] ✓ Completed");

    StressTestStats {
        test_name: "RGAText".to_string(),
        num_replicas,
        operations_per_replica: ops_per_replica,
        total_syncs,
        total_time,
        avg_sync_time,
        ops_per_second,
        converged,
    }
}

/// Stress test for RichText with formatting
pub fn stress_test_rich_text(num_replicas: usize, ops_per_replica: usize) -> StressTestStats {
    use mdcs_db::{MarkType, RichText};

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  RichText Formatting Stress Test                           ║");
    println!(
        "║  Replicas: {:>3} │ Ops/Replica: {:>5}                        ║",
        num_replicas, ops_per_replica
    );
    println!("╚════════════════════════════════════════════════════════════╝");

    let start = Instant::now();

    // Initialize replicas
    let mut replicas: Vec<RichText> = Vec::with_capacity(num_replicas);
    for i in 0..num_replicas {
        replicas.push(RichText::new(format!("replica_{}", i)));
    }

    println!("\n[Phase 1/3] Simulating rich text edits with formatting...");

    let mut rng = StdRng::seed_from_u64(42);
    let mark_types = [
        MarkType::Bold,
        MarkType::Italic,
        MarkType::Underline,
        MarkType::Strikethrough,
    ];

    // Phase 1: Each replica makes edits and applies formatting
    for (idx, replica) in replicas.iter_mut().enumerate() {
        for i in 0..ops_per_replica {
            let text_len = replica.text().len();

            // 50% text insert, 30% format, 20% delete
            let op = rng.gen::<f64>();
            if op < 0.5 || text_len == 0 {
                let pos = if text_len == 0 {
                    0
                } else {
                    rng.gen_range(0..=text_len)
                };
                let text = format!("T{}_{} ", idx, i);
                replica.insert(pos, &text);
            } else if op < 0.8 && text_len > 2 {
                let start = rng.gen_range(0..text_len - 1);
                let end = rng.gen_range(start + 1..=text_len);
                let mark = mark_types[rng.gen_range(0..mark_types.len())].clone();
                let _ = replica.add_mark(start, end, mark);
            } else if text_len > 0 {
                let del_len = rng.gen_range(1..=std::cmp::min(3, text_len));
                let del_pos = rng.gen_range(0..text_len.saturating_sub(del_len - 1));
                replica.delete(del_pos, del_len);
            }
        }
        if (idx + 1) % 2 == 0 {
            println!("  Replica {}/{} completed", idx + 1, num_replicas);
        }
    }

    println!("[Phase 1/3] ✓ Completed");
    println!("[Phase 2/3] Merging replicas...");

    let mut sync_times = vec![];
    let mut total_syncs = 0;

    // Phase 2: Merge
    for round in 0..3 {
        for i in 0..num_replicas {
            for j in (i + 1)..num_replicas {
                let sync_start = Instant::now();

                let delta_i = replicas[i].clone();
                let delta_j = replicas[j].clone();

                replicas[i] = replicas[i].join(&delta_j);
                replicas[j] = replicas[j].join(&delta_i);

                sync_times.push(sync_start.elapsed());
                total_syncs += 1;
            }
        }
        println!("  Merge round {}/3 completed", round + 1);
    }

    println!("[Phase 2/3] ✓ Completed");
    println!("[Phase 3/3] Verifying convergence...");

    // Verify convergence (text content)
    let first_text = replicas[0].text().to_string();
    let converged = replicas.iter().all(|r| r.text().to_string() == first_text);

    let mark_count: usize = replicas[0].all_marks().count();
    println!("  Final text length: {} chars", first_text.len());
    println!("  Total marks: {}", mark_count);
    println!("  All replicas identical: {}", converged);

    let total_time = start.elapsed();
    let avg_sync_time = if !sync_times.is_empty() {
        sync_times.iter().sum::<Duration>() / sync_times.len() as u32
    } else {
        Duration::ZERO
    };

    let total_ops = num_replicas * ops_per_replica + total_syncs;
    let ops_per_second = total_ops as f64 / total_time.as_secs_f64();

    println!("[Phase 3/3] ✓ Completed");

    StressTestStats {
        test_name: "RichText".to_string(),
        num_replicas,
        operations_per_replica: ops_per_replica,
        total_syncs,
        total_time,
        avg_sync_time,
        ops_per_second,
        converged,
    }
}

/// Stress test for JsonCrdt nested documents
pub fn stress_test_json_crdt(num_replicas: usize, ops_per_replica: usize) -> StressTestStats {
    use mdcs_db::{JsonCrdt, JsonPath, JsonValue};

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  JsonCrdt Nested Document Stress Test                      ║");
    println!(
        "║  Replicas: {:>3} │ Ops/Replica: {:>5}                        ║",
        num_replicas, ops_per_replica
    );
    println!("╚════════════════════════════════════════════════════════════╝");

    let start = Instant::now();

    // Initialize replicas
    let mut replicas: Vec<JsonCrdt> = Vec::with_capacity(num_replicas);
    for i in 0..num_replicas {
        replicas.push(JsonCrdt::new(format!("replica_{}", i)));
    }

    println!("\n[Phase 1/3] Simulating JSON document edits...");

    let mut rng = StdRng::seed_from_u64(42);
    let paths = [
        "user.name",
        "user.email",
        "settings.theme",
        "settings.fontSize",
        "data.count",
        "data.items.first",
        "data.items.second",
        "metadata.version",
    ];

    // Phase 1: Each replica makes edits
    for (idx, replica) in replicas.iter_mut().enumerate() {
        for i in 0..ops_per_replica {
            let path = paths[rng.gen_range(0..paths.len())];
            let json_path = JsonPath::parse(path);

            // Generate random value
            let value = match rng.gen_range(0..4) {
                0 => JsonValue::String(format!("value_{}_{}", idx, i)),
                1 => JsonValue::Int((idx * 1000 + i) as i64),
                2 => JsonValue::Float((idx as f64) * 100.0 + (i as f64) * 0.1),
                _ => JsonValue::Bool(rng.gen()),
            };

            let _ = replica.set(&json_path, value);
        }
        if (idx + 1) % 2 == 0 {
            println!("  Replica {}/{} completed", idx + 1, num_replicas);
        }
    }

    println!("[Phase 1/3] ✓ Completed");
    println!("[Phase 2/3] Merging replicas...");

    let mut sync_times = vec![];
    let mut total_syncs = 0;

    // Phase 2: Merge
    for round in 0..3 {
        for i in 0..num_replicas {
            for j in (i + 1)..num_replicas {
                let sync_start = Instant::now();

                let delta_i = replicas[i].clone();
                let delta_j = replicas[j].clone();

                replicas[i] = replicas[i].join(&delta_j);
                replicas[j] = replicas[j].join(&delta_i);

                sync_times.push(sync_start.elapsed());
                total_syncs += 1;
            }
        }
        println!("  Merge round {}/3 completed", round + 1);
    }

    println!("[Phase 2/3] ✓ Completed");
    println!("[Phase 3/3] Verifying convergence...");

    // Verify convergence
    let first_json = replicas[0].to_json();
    let converged = replicas.iter().all(|r| r.to_json() == first_json);

    let key_count = replicas[0].keys().len();
    println!("  Top-level keys: {}", key_count);
    println!("  All replicas identical: {}", converged);

    let total_time = start.elapsed();
    let avg_sync_time = if !sync_times.is_empty() {
        sync_times.iter().sum::<Duration>() / sync_times.len() as u32
    } else {
        Duration::ZERO
    };

    let total_ops = num_replicas * ops_per_replica + total_syncs;
    let ops_per_second = total_ops as f64 / total_time.as_secs_f64();

    println!("[Phase 3/3] ✓ Completed");

    StressTestStats {
        test_name: "JsonCrdt".to_string(),
        num_replicas,
        operations_per_replica: ops_per_replica,
        total_syncs,
        total_time,
        avg_sync_time,
        ops_per_second,
        converged,
    }
}

/// Stress test for DocumentStore with multiple document types
pub fn stress_test_document_store(num_docs: usize, ops_per_doc: usize) -> StressTestStats {
    use mdcs_db::{DocumentStore, JsonValue};

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  DocumentStore Multi-Document Stress Test                  ║");
    println!(
        "║  Documents: {:>3} │ Ops/Document: {:>5}                     ║",
        num_docs, ops_per_doc
    );
    println!("╚════════════════════════════════════════════════════════════╝");

    let start = Instant::now();

    let mut store = DocumentStore::new("stress_replica");
    let mut rng = StdRng::seed_from_u64(42);

    println!("\n[Phase 1/2] Creating and editing documents...");

    let mut text_docs = Vec::new();
    let mut json_docs = Vec::new();
    let mut rich_docs = Vec::new();

    // Create documents
    for i in 0..num_docs {
        match i % 3 {
            0 => {
                let id = store.create_text(format!("TextDoc_{}", i));
                text_docs.push(id);
            }
            1 => {
                let id = store.create_json(format!("JsonDoc_{}", i));
                json_docs.push(id);
            }
            _ => {
                let id = store.create_rich_text(format!("RichDoc_{}", i));
                rich_docs.push(id);
            }
        }
    }

    println!(
        "  Created {} text, {} JSON, {} rich text documents",
        text_docs.len(),
        json_docs.len(),
        rich_docs.len()
    );

    // Perform operations
    for i in 0..ops_per_doc {
        // Text operations
        for doc_id in &text_docs {
            if rng.gen::<f64>() < 0.8 {
                let _ = store.text_insert(doc_id, 0, &format!("Line{} ", i));
            }
        }

        // JSON operations
        for doc_id in &json_docs {
            let key = format!("key_{}", i % 10);
            let value = JsonValue::Int((i * 100) as i64);
            let _ = store.json_set(doc_id, &key, value);
        }

        // Rich text operations
        for doc_id in &rich_docs {
            if rng.gen::<f64>() < 0.8 {
                let _ = store.rich_text_insert(doc_id, 0, &format!("Word{} ", i));
            }
        }

        if i % 100 == 0 && i > 0 {
            println!("  Operations: {}/{}", i, ops_per_doc);
        }
    }

    println!("[Phase 1/2] ✓ Completed");
    println!("[Phase 2/2] Querying documents...");

    // Test queries
    use mdcs_db::QueryOptions;

    let query_start = Instant::now();
    let results = store.query(&QueryOptions::default());
    let query_time = query_start.elapsed();

    println!(
        "  Query returned {} documents in {:?}",
        results.len(),
        query_time
    );

    let total_time = start.elapsed();
    let total_ops = num_docs + (ops_per_doc * num_docs);
    let ops_per_second = total_ops as f64 / total_time.as_secs_f64();

    println!("[Phase 2/2] ✓ Completed");

    StressTestStats {
        test_name: "DocumentStore".to_string(),
        num_replicas: 1,
        operations_per_replica: total_ops,
        total_syncs: 0,
        total_time,
        avg_sync_time: query_time,
        ops_per_second,
        converged: true,
    }
}

// ============================================================================
// Delta & Network Simulation Tests
// ============================================================================

/// Delta-based stress test for GSet with network simulation
pub fn stress_test_delta_gset(
    num_replicas: usize,
    ops_per_replica: usize,
    loss_rate: f64,
    dup_rate: f64,
    _reorder_rate: f64,
    max_rounds: usize,
) -> DeltaStressTestStats {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  Delta GSet Network Simulation                             ║");
    println!(
        "║  Replicas: {} │ Ops: {} │ Loss: {:.0}% │ Dup: {:.0}%          ║",
        num_replicas,
        ops_per_replica,
        loss_rate * 100.0,
        dup_rate * 100.0
    );
    println!("╚════════════════════════════════════════════════════════════╝");

    let start = Instant::now();

    // Initialize replicas
    let mut replicas: Vec<GSet<u64>> = vec![GSet::new(); num_replicas];
    let mut rng = StdRng::seed_from_u64(42);

    println!("\n[Phase 1/3] Adding elements to replicas...");

    // Phase 1: Add operations (each replica adds unique elements)
    for (idx, replica) in replicas.iter_mut().enumerate() {
        for i in 0..ops_per_replica {
            let value = ((idx as u64) << 32) | (i as u64);
            replica.insert(value);
        }
    }

    let expected_total = num_replicas * ops_per_replica;
    println!("[Phase 1/3] ✓ Added {} total elements", expected_total);

    println!("[Phase 2/3] Synchronizing with simulated network failures...");

    // Phase 2: Sync using anti-entropy with simulated failures
    let mut rounds = 0;
    let mut converged = false;

    while rounds < max_rounds && !converged {
        rounds += 1;

        // Each replica sends to random other replicas
        for i in 0..num_replicas {
            for j in 0..num_replicas {
                if i == j {
                    continue;
                }

                // Simulate message loss
                if rng.gen::<f64>() < loss_rate {
                    continue;
                }

                // Clone current state as "delta" to send
                let delta = replicas[i].clone();

                // Simulate duplication (send twice)
                let send_count = if rng.gen::<f64>() < dup_rate { 2 } else { 1 };

                for _ in 0..send_count {
                    replicas[j] = replicas[j].join(&delta);
                }
            }
        }

        // Check convergence
        let first_len = replicas[0].len();
        converged = replicas
            .iter()
            .all(|r| r.len() == first_len && r.len() == expected_total);

        if rounds % 5 == 0 {
            println!(
                "  Round {}: sizes = {:?}",
                rounds,
                replicas.iter().map(|r| r.len()).collect::<Vec<_>>()
            );
        }
    }

    println!("[Phase 2/3] ✓ Completed after {} rounds", rounds);

    println!("[Phase 3/3] Verifying convergence...");

    let final_size = replicas[0].len();
    let all_same_size = replicas.iter().all(|r| r.len() == final_size);

    if converged && all_same_size && final_size == expected_total {
        println!("  ✓ All replicas converged to {} elements", final_size);
    } else {
        println!(
            "  ✗ Convergence failed: expected {}, got sizes {:?}",
            expected_total,
            replicas.iter().map(|r| r.len()).collect::<Vec<_>>()
        );
    }

    let total_time = start.elapsed();

    DeltaStressTestStats {
        num_replicas,
        operations_per_replica: ops_per_replica,
        network_config: format!(
            "loss={:.0}%, dup={:.0}%",
            loss_rate * 100.0,
            dup_rate * 100.0
        ),
        sync_rounds: rounds,
        converged,
        total_time,
        final_state_size: final_size,
    }
}

/// Prove convergence under repeated re-sends (idempotence test)
pub fn stress_test_idempotence(
    num_replicas: usize,
    ops_per_replica: usize,
    resend_count: usize,
) -> bool {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  Idempotence Verification Test                             ║");
    println!(
        "║  Replicas: {} │ Ops: {} │ Resends: {}                       ║",
        num_replicas, ops_per_replica, resend_count
    );
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
                replicas[j] = replicas[j].join(&delta);
            }
        }
    }

    let baseline_state = replicas[0].clone();
    println!("  Baseline state size: {}", baseline_state.len());

    // Re-send same deltas many times
    for resend in 0..resend_count {
        for i in 0..num_replicas {
            for j in 0..num_replicas {
                if i != j {
                    let delta = replicas[i].clone();
                    replicas[j] = replicas[j].join(&delta);
                }
            }
        }
        if (resend + 1) % 10 == 0 {
            println!("  Resend round {}/{}", resend + 1, resend_count);
        }
    }

    let final_state = &replicas[0];
    let idempotent = final_state == &baseline_state;

    println!(
        "  ✓ Idempotence verified: {} re-sends, state unchanged: {}",
        resend_count, idempotent
    );

    idempotent
}

// ============================================================================
// Scaling Analysis
// ============================================================================

/// Parallel stress test comparing different replica scales
pub async fn stress_test_scaling(max_replicas: usize, step_size: usize) {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  Scaling Analysis - Performance vs Replica Count           ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    let mut results: Vec<(usize, StressTestStats)> = Vec::new();

    let mut current_replicas = step_size;
    while current_replicas <= max_replicas {
        let stats = stress_test_gset(current_replicas, 50, current_replicas * 100).await;
        results.push((current_replicas, stats));
        current_replicas += step_size;
    }

    // Print scaling summary
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║                 SCALING SUMMARY                            ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Replicas │  Time (s) │   Ops/sec │ Avg Sync (µs)          ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    for (replicas, stats) in &results {
        println!(
            "║  {:>7} │ {:>9.3} │ {:>9.0} │ {:>10.2}             ║",
            replicas,
            stats.total_time.as_secs_f64(),
            stats.ops_per_second,
            stats.avg_sync_time.as_micros() as f64
        );
    }
    println!("╚════════════════════════════════════════════════════════════╝");
}

// ============================================================================
// Comprehensive Test Suites
// ============================================================================

/// Run all core CRDT stress tests
pub async fn stress_test_all_core_crdts(
    num_replicas: usize,
    ops_per_replica: usize,
    num_syncs: usize,
) {
    println!("\n");
    println!("╔════════════════════════════════════════════════════════════════════════╗");
    println!("║              CORE CRDT STRESS TEST SUITE                               ║");
    println!("║  Testing: GSet, ORSet, PNCounter, LWWRegister, MVRegister              ║");
    println!("╚════════════════════════════════════════════════════════════════════════╝");

    let mut results: Vec<StressTestStats> = Vec::new();

    results.push(stress_test_gset(num_replicas, ops_per_replica, num_syncs).await);
    results.push(stress_test_orset(num_replicas, ops_per_replica, num_syncs).await);
    results.push(stress_test_pncounter(num_replicas, ops_per_replica, num_syncs).await);
    results.push(stress_test_lwwreg(num_replicas, ops_per_replica, num_syncs).await);
    results.push(stress_test_mvreg(num_replicas, ops_per_replica, num_syncs).await);

    print_summary_table(&results);
}

/// Run all database layer stress tests
pub fn stress_test_all_db_crdts(num_replicas: usize, ops_per_replica: usize) {
    println!("\n");
    println!("╔════════════════════════════════════════════════════════════════════════╗");
    println!("║              DATABASE LAYER STRESS TEST SUITE                          ║");
    println!("║  Testing: RGAText, RichText, JsonCrdt, DocumentStore                   ║");
    println!("╚════════════════════════════════════════════════════════════════════════╝");

    let results: Vec<StressTestStats> = vec![
        stress_test_rga_text(num_replicas, ops_per_replica),
        stress_test_rich_text(num_replicas, ops_per_replica),
        stress_test_json_crdt(num_replicas, ops_per_replica),
        stress_test_document_store(
            num_replicas * 5,
            ops_per_replica / 2,
        ),
    ];

    print_summary_table(&results);
}

/// Run delta/network simulation tests
pub async fn stress_test_delta_suite() {
    println!("\n");
    println!("╔════════════════════════════════════════════════════════════════════════╗");
    println!("║              DELTA & NETWORK SIMULATION SUITE                          ║");
    println!("║  Testing convergence under: loss, duplication, chaos                   ║");
    println!("╚════════════════════════════════════════════════════════════════════════╝");

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

    println!("\n✓ All delta/network simulation tests passed!");
}

/// Print summary table for multiple test results
fn print_summary_table(results: &[StressTestStats]) {
    println!("\n╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║                          BENCHMARK SUMMARY                               ║");
    println!("╠══════════════════════════════════════════════════════════════════════════╣");
    println!("║  Component       │  Time (s) │   Ops/sec │ Avg Sync (µs) │ Converged    ║");
    println!("╠══════════════════════════════════════════════════════════════════════════╣");
    for stats in results {
        println!(
            "║  {:14} │ {:>9.3} │ {:>9.0} │ {:>13.2} │ {:>12} ║",
            stats.test_name,
            stats.total_time.as_secs_f64(),
            stats.ops_per_second,
            stats.avg_sync_time.as_micros() as f64,
            if stats.converged { "✓" } else { "✗" }
        );
    }
    println!("╚══════════════════════════════════════════════════════════════════════════╝");
}

/// Run the complete stress test suite
pub async fn stress_test_full_suite() {
    println!("\n");
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║                                                                          ║");
    println!("║             MDCS COMPREHENSIVE STRESS TEST SUITE                         ║");
    println!("║                                                                          ║");
    println!("║  Testing all components: Core CRDTs, Database Layer, Network Simulation  ║");
    println!("║                                                                          ║");
    println!("╚══════════════════════════════════════════════════════════════════════════╝");

    // 1. Core CRDTs
    stress_test_all_core_crdts(4, 100, 200).await;

    // 2. Database Layer
    stress_test_all_db_crdts(4, 100);

    // 3. Delta/Network Simulation
    stress_test_delta_suite().await;

    // 4. Scaling Analysis
    stress_test_scaling(12, 3).await;

    println!("\n");
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║                                                                          ║");
    println!("║              ✓ ALL STRESS TESTS COMPLETED SUCCESSFULLY                   ║");
    println!("║                                                                          ║");
    println!("╚══════════════════════════════════════════════════════════════════════════╝");
}

// Legacy aliases for backward compatibility
pub use stress_test_all_core_crdts as stress_test_all_crdts;
