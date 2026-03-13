/// Performance benchmarks for optimized hot paths.
/// Run with: cargo bench -p mdcs-benchmarks --bench perf_hotspots
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mdcs_core::{GSet, Lattice, PNCounter};
use mdcs_db::RGAList;
use mdcs_merkle::{DAGStore, MemoryDAGStore, NodeBuilder, Payload};

// ============================================================================
// CRDT Join/Merge Benchmarks
// ============================================================================

fn pncounter_join_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("pncounter_join");

    for replica_count in [2, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("replicas_{}", replica_count)),
            replica_count,
            |b, &replica_count| {
                let mut counter1 = PNCounter::<String>::new();
                let mut counter2 = PNCounter::<String>::new();

                // Fill with data from multiple replicas
                for i in 0..replica_count {
                    let replica_id = format!("r{}", i);
                    counter1.increment(replica_id.clone(), i as u64 * 100);
                    counter2.decrement(replica_id.clone(), i as u64 * 50);
                }

                b.iter(|| {
                    let _merged = black_box(&counter1).join(black_box(&counter2));
                });
            },
        );
    }
    group.finish();
}

fn gset_join_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("gset_join");

    for element_count in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("elements_{}", element_count)),
            element_count,
            |b, &element_count| {
                let mut set1 = GSet::<i32>::new();
                let mut set2 = GSet::<i32>::new();

                for i in 0..(element_count / 2) {
                    set1.insert(i);
                    set2.insert(i + element_count / 4);
                }

                b.iter(|| {
                    let _merged = black_box(&set1).join(black_box(&set2));
                });
            },
        );
    }
    group.finish();
}

// ============================================================================
// RGA List Delta Application Benchmarks
// ============================================================================

fn rga_list_apply_delta_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("rga_list_apply_delta");

    for delta_size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("inserts_{}", delta_size)),
            delta_size,
            |b, &delta_size| {
                let mut list = RGAList::<String>::new("replica1");

                // Build up base list state
                for i in 0..100 {
                    list.push_back(format!("item_{}", i));
                }
                // Clear local pending delta so clone cost does not include
                // unrelated local edit history.
                let _ = list.take_delta();

                // Create delta from another replica
                let mut delta_list = RGAList::<String>::new("replica2");
                for i in 0..delta_size {
                    delta_list.push_back(format!("remote_{}", i));
                }
                let delta = delta_list.take_delta().unwrap();

                b.iter(|| {
                    let mut list_copy = list.clone();
                    list_copy.apply_delta(black_box(&delta));
                });
            },
        );
    }
    group.finish();
}

// ============================================================================
// Merkle DAG Cache Benchmarks
// ============================================================================

fn merkle_sync_topological_order_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("merkle_topo_order");
    group.sample_size(10);

    for node_count in [100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("nodes_{}", node_count)),
            node_count,
            |b, &node_count| {
                let (mut store, mut parent) = MemoryDAGStore::with_genesis("r1");

                // Build a linear DAG chain
                for i in 1..node_count {
                    let node = NodeBuilder::new()
                        .with_parent(parent)
                        .with_payload(Payload::Delta(format!("delta_{}", i).into_bytes()))
                        .build();
                    parent = store.put(node).expect("Failed to store node");
                }

                b.iter(|| {
                    let _order = black_box(&store).topological_order();
                });
            },
        );
    }
    group.finish();
}

fn merkle_cache_effectiveness_benchmark(c: &mut Criterion) {
    c.bench_function("merkle_cache_sequential_hits", |b| {
        let (store, mut parent) = MemoryDAGStore::with_genesis("r1");
        let node_count = 500;

        // Build DAG
        let mut mutable_store = store.clone();
        for i in 1..node_count {
            let node = NodeBuilder::new()
                .with_parent(parent)
                .with_payload(Payload::Delta(format!("delta_{}", i).into_bytes()))
                .build();
            parent = mutable_store.put(node).expect("Failed to store node");
        }

        mutable_store.topological_order(); // Prime cache

        b.iter(|| {
            // Simulate multiple sync requests using cached order
            for _ in 0..10 {
                let _order = black_box(&mutable_store).topological_order();
            }
        });
    });
}

// ============================================================================
// Large Batch Operations
// ============================================================================

fn large_crdt_merge_benchmark(c: &mut Criterion) {
    c.bench_function("large_pncounter_merge_50_replicas", |b| {
        let mut counter1 = PNCounter::<String>::new();
        let mut counter2 = PNCounter::<String>::new();

        // Simulate high-replica system
        for i in 0..50 {
            let replica_id = format!("replica_{:03}", i);
            counter1.increment(replica_id.clone(), i as u64 * 1000);
            counter2.decrement(replica_id.clone(), i as u64 * 500);
        }

        b.iter(|| {
            let _merged = black_box(&counter1).join(black_box(&counter2));
        });
    });
}

criterion_group!(
    benches,
    pncounter_join_benchmark,
    gset_join_benchmark,
    rga_list_apply_delta_benchmark,
    merkle_sync_topological_order_benchmark,
    merkle_cache_effectiveness_benchmark,
    large_crdt_merge_benchmark
);

criterion_main!(benches);
