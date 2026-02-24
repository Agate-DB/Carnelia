# MDCS CRDT Benchmarks

Port of [dmonad/crdt-benchmarks](https://github.com/dmonad/crdt-benchmarks) targeting the MDCS stack (Rust).

## Quick start

```sh
# Run all benchmarks with default N=6000
cargo run -p mdcs-benchmarks

# Run with smaller N for quick iteration
cargo run -p mdcs-benchmarks -- --n 100

# Run a specific benchmark group
cargo run -p mdcs-benchmarks -- --only b1

# Run a single sub-benchmark
cargo run -p mdcs-benchmarks -- --only b1.1

# Include the very long-running B4x100
cargo run -p mdcs-benchmarks -- --full
```

> to run the full benchmarks with output log:
```sh
cargo run --release -p mdcs-benchmarks -- --n 6000 2>&1 | tee logs/benchmark_results.txt
```

> To convert logs to md format:
```sh
uv run python parse_log.py logs/benchmark_results.txt -o RESULTS.md
```

## Benchmark groups

### B1: No conflicts

Two clients, one performs all mutations then syncs to the other.

| ID     | Description                              | CRDT used   |
|--------|------------------------------------------|-------------|
| B1.1   | Append N characters                      | TextDoc     |
| B1.2   | Insert string of length N                | TextDoc     |
| B1.3   | Prepend N characters                     | TextDoc     |
| B1.4   | Insert N characters at random positions  | TextDoc     |
| B1.5   | Insert N words at random positions       | TextDoc     |
| B1.6   | Insert string, then delete it            | TextDoc     |
| B1.7   | Insert/Delete strings at random          | TextDoc     |
| B1.8   | Append N numbers                         | RGAList     |
| B1.9   | Insert Array of N numbers (bulk)         | RGAList     |
| B1.10  | Prepend N numbers                        | RGAList     |
| B1.11  | Insert N numbers at random positions     | RGAList     |

### B2: Two users producing conflicts

Both clients start synced with 100 characters, make concurrent edits, then merge.

| ID   | Description                                          | CRDT used |
|------|------------------------------------------------------|-----------|
| B2.1 | Concurrently insert string of length N at index 0    | TextDoc   |
| B2.2 | Concurrently insert N characters at random positions | TextDoc   |
| B2.3 | Concurrently insert N words at random positions      | TextDoc   |
| B2.4 | Concurrently insert & delete                         | TextDoc   |

### B3: Many conflicts

`20*sqrt(N)` concurrent clients each make one mutation, then all merge.

| ID   | Description                          | CRDT used |
|------|--------------------------------------|-----------|
| B3.1 | Set number in Map                    | JsonCrdt  |
| B3.2 | Set Object in Map                    | JsonCrdt  |
| B3.3 | Set String in Map                    | JsonCrdt  |
| B3.4 | Insert text in Array                 | RGAList   |

### B4: Real-world editing dataset

Replays ~260k character-by-character edits from a real LaTeX document editing trace.

| ID     | Description                              | CRDT used |
|--------|------------------------------------------|-----------|
| B4     | Apply editing dataset (1x)               | TextDoc   |
| B4x100 | Apply editing dataset 100x (--full only) | TextDoc   |

The dataset is downloaded from the [automerge-perf](https://github.com/automerge/automerge-perf) repository on first run and cached in `benchmarks/data/`.

## Metrics

Each benchmark reports:

| Metric              | Description                                           |
|---------------------|-------------------------------------------------------|
| `time`              | Wall-clock time for the mutation phase                 |
| `avgUpdateSize`     | Average size of individual update payloads (bytes)     |
| `encodeTime`        | Time to serialize the final document (all 3 formats)   |
| `docSize:json`      | Serialized document size via serde_json                |
| `docSize:bincode`   | Serialized document size via bincode                   |
| `docSize:postcard`  | Serialized document size via postcard                  |
| `parseTime`         | Time to deserialize the document back                  |
| `memUsed`           | Approximate heap memory used (tracking allocator)      |

## Serialization formats

Three formats are benchmarked to give a fair size comparison against JS CRDT libraries that use custom binary formats:

- **serde_json** — human-readable, largest
- **bincode** — compact binary, good general-purpose
- **postcard** — varint-encoded, smallest for integer-heavy data

## Architecture

```
benchmarks/
├── Cargo.toml
├── README.md
├── data/               # cached B4 dataset (gitignored)
│   └── edits.json
└── src/
    ├── main.rs          # entry point, CLI, global allocator
    ├── harness.rs       # TrackingAllocator, BenchmarkMetrics, logging
    ├── encoding.rs      # json/bincode/postcard wrappers
    ├── b1_no_conflicts.rs
    ├── b2_two_users.rs
    ├── b3_many_conflicts.rs
    ├── b4_real_world.rs
    └── dataset.rs       # B4 trace downloader/parser
```

## Comparison with crdt-benchmarks

The original JS benchmarks use N=6000 by default. This port defaults to the same value. Key differences:

- **Language**: Rust vs JavaScript/WASM — expect lower absolute times
- **Serialization**: We report 3 formats (json/bincode/postcard) vs the original's single binary format
- **Memory**: Uses a tracking global allocator (alloc/dealloc diff) rather than JS GC heap snapshots
- **Array ops** (B1.8–B1.11, B3.4): Use `RGAList<i64>` from mdcs-db directly (no SDK wrapper)
- **Map ops** (B3.1–B3.3): Use `JsonCrdt` from mdcs-db directly via the `Lattice::join` trait


## Full benchmark results

