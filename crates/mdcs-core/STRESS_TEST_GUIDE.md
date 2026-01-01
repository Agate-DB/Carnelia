# Async Stress Testing Implementation for Carnelia CRDT

## Overview

I've implemented a comprehensive async stress testing framework for the GSet and ORSet CRDTs using Tokio for async/await
and async generators for replica synchronization patterns.

## Key Features

### 1. **Async Generators for Replica Synchronization**

The `replica_sync_generator` function uses Rust's async generators to yield random replica pairs for synchronization:

- Generates unlimited replica synchronization pairs asynchronously
- Uses randomized replica selection to simulate realistic network patterns
- Yields tuples of `(replica_a_idx, replica_b_idx)` for each sync operation

```rust
#[async_gen]
async fn replica_sync_generator(num_replicas: usize, num_syncs: usize) {
    let mut rng = rand::thread_rng();
    for _ in 0..num_syncs {
        let replica_a = rng.gen_range(0..num_replicas);
        let replica_b = rng.gen_range(0..num_replicas);
        yield (replica_a, replica_b);
    }
}
```

### 2. **GSet Stress Test (`stress_test_gset`)**

Two-phase async stress testing for GSet:

**Phase 1: Concurrent Additions**

- Spawns N concurrent tokio tasks (one per replica)
- Each task adds `ops_per_replica` elements concurrently
- Uses Arc<Mutex<>> for thread-safe shared state
- Yields periodically to allow other tasks to run

**Phase 2: Replica Synchronization**

- Uses the async generator to get random replica pairs
- Performs concurrent join operations with timing
- Tracks sync latency in microseconds
- Measures total throughput in operations/second

### 3. **ORSet Stress Test (`stress_test_orset`)**

Similar two-phase approach but with add/remove operations:

**Phase 1: Mixed Operations**

- Concurrent adds and removes across replicas
- 30% probability of removal after 10+ operations
- Simulates realistic shopping cart/presence tracking scenarios

**Phase 2: Synchronization**

- Same async generator-based sync pattern as GSet
- Measures convergence time and sync overhead

### 4. **Scaling Analysis (`stress_test_scaling`)**

Tests performance across varying replica counts:

- Scales from `step_size` to `max_replicas`
- Automatically increases sync count with replica count
- Provides comparative performance metrics

### 5. **Performance Statistics (`StressTestStats`)**

Collects and displays comprehensive metrics:

- Number of replicas and operations per replica
- Total sync operations performed
- Total elapsed time
- Average sync time (in microseconds)
- Operations per second throughput
- Formatted table output for easy comparison

## Architecture

### Thread Safety

- Uses `Arc<Mutex<T>>` for shared replica state
- Tokio's async Mutex for non-blocking synchronization
- Safe concurrent access without data races

### Concurrency Model

- Tokio runtime with full feature set
- Spawns independent tasks for each replica's operations
- Join operations wait for both locks before proceeding
- Periodic yields prevent starvation

### Randomization

- Uses `rand` crate for realistic sync patterns
- Random replica pair selection simulates actual network behavior
- Variable operation patterns across replicas

## Example Usage

The `main.rs` now runs:

1. **Synchronous examples** (existing real-world use cases)
2. **GSet stress tests** (small and medium scale)
3. **ORSet stress tests** (small and medium scale)
4. **Scaling analysis** (20 replicas with 2-replica increments)

### Sample Test Configurations

```rust
// Small scale (4 replicas, 100 ops each, 200 syncs)
stress_test_gset(4, 100, 200).await

// Medium scale (10 replicas, 500 ops each, 1000 syncs)
stress_test_gset(10, 500, 1000).await

// Scaling analysis (2, 4, 6, ... 20 replicas)
stress_test_scaling(20, 2).await
```

## Metrics Captured

Each stress test returns `StressTestStats` containing:

- `num_replicas`: Number of concurrent replicas tested
- `operations_per_replica`: Add operations per replica
- `total_syncs`: Number of successful sync operations
- `total_time`: Wall-clock time for entire test
- `avg_sync_time`: Mean microseconds per sync operation
- `ops_per_second`: Overall throughput metric

## Output Format

Tests produce detailed progress output with:

- Test header with configuration
- Phase 1/2 progress indicators
- Periodic sync completion counts
- Formatted statistics table for each test
- Scaling analysis progression

## Dependencies Added

```toml
tokio = { version = "1.35", features = ["full"] }
async-generator = "0.1"
rand = "0.8"
chrono = "0.4"
```

## Performance Characteristics

The async implementation enables:

- **Scalable concurrency**: Handle many replicas without thread exhaustion
- **Realistic simulation**: Random sync patterns mimic distributed systems
- **Low latency**: Microsecond-resolution timing on sync operations
- **High throughput**: Measures operations/second across configurations
- **Memory efficiency**: Arc sharing reduces state duplication

## Next Steps

You can further enhance the stress testing by:

1. Adding network latency simulation (tokio timers)
2. Testing failure scenarios (dropped syncs, partial merges)
3. Measuring memory consumption per replica
4. Profiling convergence speed and tree complexity
5. Testing with larger datasets and longer operation sequences

