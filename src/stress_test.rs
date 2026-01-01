use async_stream::stream;
use futures::stream::Stream;
use futures::stream::StreamExt;
use mdcs_core::gset::GSet;
use mdcs_core::lattice::Lattice;
use mdcs_core::orset::ORSet;
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

    // Perform sync
    let replica_a = Arc::clone(&replicas[replica_a_idx]);
    let replica_b = Arc::clone(&replicas[replica_b_idx]);

    let (set_a, set_b) = tokio::join!(replica_a.lock(), replica_b.lock());
    let _merged = set_a.join(&*set_b);

    drop(set_a);
    drop(set_b);

    let sync_duration = sync_start.elapsed();
    sync_times.push(sync_duration);
    *total_syncs += 1;

    if *total_syncs % 100 == 0 {
        println!("  Syncs completed: {}/{}", total_syncs, num_syncs);
    }
}

/// Stress test for GSet with async synchronization
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

/// Stress test for ORSet with async synchronization
pub async fn stress_test_orset(
    num_replicas: usize,
    ops_per_replica: usize,
    num_syncs: usize,
) -> StressTestStats {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║        ORSet Stress Test (Async)                           ║");
    println!("║  Replicas: {} | Ops/Replica: {} | Syncs: {} ║",
             num_replicas, ops_per_replica, num_syncs);
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

/// Parallel stress test comparing different replica scales
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

