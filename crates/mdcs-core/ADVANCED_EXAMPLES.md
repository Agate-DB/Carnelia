# Advanced Stress Testing Examples

This document provides examples for creating custom stress test scenarios.

## 1. Stress Testing Specific Replica Counts

Modify `src/main.rs` to test specific configurations:

```rust
async fn async_main() {
    // ... existing code ...

    // Test different replica configurations
    let configs = vec![
        (2, 100, 200),      // 2 replicas, minimal
        (5, 200, 500),      // Small cluster
        (16, 1000, 5000),   // Medium cluster
        (32, 500, 10000),   // Large scale
    ];

    for (replicas, ops, syncs) in configs {
        println!("\n>>> Testing with {} replicas <<<", replicas);
        let stats = stress_test_gset(replicas, ops, syncs).await;
        stats.print();
    }
}
```

## 2. Comparing GSet and ORSet Performance

```rust
async fn compare_implementations() {
    let replicas = 10;
    let ops = 500;
    let syncs = 1000;

    println!("\n📊 Performance Comparison: GSet vs ORSet\n");

    let gset_stats = stress_test_gset(replicas, ops, syncs).await;
    let orset_stats = stress_test_orset(replicas, ops, syncs).await;

    println!("\n┌─────────────────────────────────────┐");
    println!("│ GSet Avg Sync Time: {:?}", gset_stats.avg_sync_time);
    println!("│ ORSet Avg Sync Time: {:?}", orset_stats.avg_sync_time);

    let overhead_percent =
        (orset_stats.avg_sync_time.as_micros() as f64
            / gset_stats.avg_sync_time.as_micros() as f64 - 1.0) * 100.0;
    println!("│ ORSet Overhead: {:.1}%", overhead_percent);
    println!("└─────────────────────────────────────┘");
}
```

## 3. Horizontal Scaling Analysis

Create a new function to test scaling behavior:

```rust
pub async fn detailed_scaling_analysis() {
    use std::fs::File;
    use std::io::Write;

    let mut results = File::create("scaling_results.csv").unwrap();
    writeln!(results, "replicas,ops_per_replica,ops_per_second,avg_sync_us").unwrap();

    for replicas in (2..=50).step_by(2) {
        let stats = stress_test_gset(replicas, 100, replicas * 200).await;
        writeln!(
            results,
            "{},{},{:.0},{:.2}",
            stats.num_replicas,
            stats.operations_per_replica,
            stats.ops_per_second,
            stats.avg_sync_time.as_micros() as f64
        ).unwrap();
    }

    println!("✓ Results written to scaling_results.csv");
}
```

## 4. Stress Test with Progress Tracking

Add a custom stress test with periodic reporting:

```rust
pub async fn stress_test_with_metrics(
    num_replicas: usize,
    ops_per_replica: usize,
    num_syncs: usize,
) {
    use std::time::Instant;

    let start = Instant::now();
    let checkpoint_interval = Duration::from_secs(5);
    let mut last_checkpoint = start;

    println!("Starting stress test...");

    // ... test code ...

    loop {
        let now = Instant::now();
        if now.duration_since(last_checkpoint) >= checkpoint_interval {
            let elapsed = now.duration_since(start);
            println!(
                "  {:?} elapsed - {} syncs completed",
                elapsed, completed_syncs
            );
            last_checkpoint = now;
        }
    }
}
```

## 5. Stress Testing with Deterministic Seeds

For reproducible results:

```rust
pub async fn stress_test_deterministic(
    num_replicas: usize,
    ops_per_replica: usize,
    num_syncs: usize,
    seed: u64,
) -> StressTestStats {
    // Initialize random generator with seed
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    // Use seeded RNG for replica sync selection
    // ... rest of test ...

    // Results will be identical across runs with same seed
}
```

## 6. Memory Usage Monitoring

Add memory tracking to stress tests:

```rust
use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

pub async fn stress_test_with_memory_tracking(
    num_replicas: usize,
    ops_per_replica: usize,
) {
    let before = ALLOCATED.load(Ordering::SeqCst);

    let stats = stress_test_gset(num_replicas, ops_per_replica, 100).await;

    let after = ALLOCATED.load(Ordering::SeqCst);
    let memory_used = (after - before) as f64 / 1_000_000.0;

    println!("Memory used: {:.2} MB", memory_used);
    println!("Per replica: {:.2} KB", memory_used * 1000.0 / num_replicas as f64);
}
```

## 7. Latency Distribution Analysis

Collect detailed latency metrics:

```rust
pub async fn analyze_latency_distribution(
    num_replicas: usize,
    num_syncs: usize,
) {
    let mut sync_times = vec![];

    // ... run stress test collecting sync_times ...

    // Calculate percentiles
    sync_times.sort();
    let p50 = sync_times[sync_times.len() / 2];
    let p95 = sync_times[sync_times.len() * 95 / 100];
    let p99 = sync_times[sync_times.len() * 99 / 100];

    println!("Latency Distribution:");
    println!("  P50: {:?}", p50);
    println!("  P95: {:?}", p95);
    println!("  P99: {:?}", p99);
}
```

## 8. Stress Test with Variable Load

Simulate increasing load over time:

```rust
pub async fn variable_load_test() {
    println!("Starting variable load test...");

    let load_levels = vec![
        (4, 50),
        (8, 100),
        (16, 200),
        (32, 500),
    ];

    for (replicas, ops) in load_levels {
        println!("\n→ Increasing load: {} replicas, {} ops", replicas, ops);
        let stats = stress_test_gset(replicas, ops, replicas * 100).await;
        stats.print();
    }
}
```

## 9. Long-Running Stability Test

Test stability over extended periods:

```rust
pub async fn stability_test(
    num_replicas: usize,
    duration_secs: u64,
) {
    use std::time::Duration;
    use tokio::time::sleep;

    let start = Instant::now();
    let test_duration = Duration::from_secs(duration_secs);
    let mut iteration = 0;

    while start.elapsed() < test_duration {
        iteration += 1;
        println!("Iteration {}", iteration);

        let stats = stress_test_gset(num_replicas, 100, 200).await;

        if iteration > 1 {
            println!("  Ops/sec: {:.0}", stats.ops_per_second);
        }

        sleep(Duration::from_millis(100)).await;
    }
}
```

## 10. Comparing Different Sync Patterns

Create tests with different synchronization patterns:

```rust
pub async fn test_sync_patterns() {
    println!("Testing different sync patterns...\n");

    // All-to-all synchronization
    let stats = stress_test_gset(10, 100, 10 * 9).await;
    println!("All-to-all syncs: {:.0} ops/sec", stats.ops_per_second);

    // Linear chain synchronization
    let stats = stress_test_gset(10, 100, 9).await;
    println!("Linear chain syncs: {:.0} ops/sec", stats.ops_per_second);

    // Star topology (central node)
    let stats = stress_test_gset(10, 100, 10).await;
    println!("Star topology syncs: {:.0} ops/sec", stats.ops_per_second);
}
```

## Running Custom Tests

To run any of these custom tests, modify `async_main()` in `src/main.rs`:

```rust
async fn async_main() {
    // Replace or add custom test calls:
    compare_implementations().await;
    detailed_scaling_analysis().await;
    stability_test(8, 30).await;  // Run for 30 seconds with 8 replicas
}
```

Then run:

```bash
cargo run --release
```

## Tips for Custom Stress Testing

1. **Start small**: Test with 2-4 replicas before scaling
2. **Measure baseline**: Run on unloaded system for accurate results
3. **Multiple runs**: Run tests 3+ times, discard first warm-up run
4. **Control variables**: Change one parameter at a time
5. **Use release builds**: Always use `--release` for performance testing
6. **Monitor system**: Watch CPU/memory with `htop` during tests
7. **Log results**: Save metrics to files for analysis
8. **Compare versions**: Use git branches to compare implementations

