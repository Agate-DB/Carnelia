
## MDCS: Modular Distributed CRDTs in Rust

An architecture for lock-free, offline-first, open-membership databases
that systematically addresses the gaps in existing CRDT systems.

## Project Structure Overview

```txt
mdcs/
├── crates/
│   ├── mdcs-core/           # Phase 1: CRDT kernel
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── lattice.rs   # Join-semilattice trait
│   │   │   ├── gset.rs      # Grow-only set
│   │   │   ├── gcounter.rs  # Grow-only counter
│   │   │   ├── pncounter.rs # PN-Counter
│   │   │   ├── orset.rs     # Observed-Remove Set
│   │   │   ├── lwwreg.rs    # Last-Writer-Wins Register
│   │   │   ├── mvreg.rs     # Multi-Value Register
│   │   │   └── map.rs       # CRDT Map composition
│   │   └── tests/
│   │
│   ├── mdcs-delta/          # Phase 2-3: Delta-state layer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── delta. rs     # Delta-mutator trait
│   │   │   ├── buffer.rs    # Delta buffer & grouping
│   │   │   ├── interval.rs  # Delta-interval tracking
│   │   │   └── causal.rs    # Causal delta-merging
│   │   └── tests/
│   │
│   ├── mdcs-merkle/         # Phase 4: Merkle-Clock
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── clock.rs     # Merkle-Clock implementation
│   │   │   ├── node.rs      # DAG node structure
│   │   │   ├── dag.rs       # DAG operations
│   │   │   └── syncer.rs    # DAGSyncer abstraction
│   │   └── tests/
│   │
│   ├── mdcs-sync/           # Phase 4:  Sync protocols
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── antientry.rs # Anti-entropy algorithms
│   │   │   ├── broadcast.rs # Broadcaster abstraction
│   │   │   └── protocol.rs  # Wire protocol
│   │   └── tests/
│   │
│   ├── mdcs-compact/        # Phase 5: Compaction
│   │   ├── src/
│   │   │   ├── lib. rs
│   │   │   ├── stability.rs # Causal stability tracking
│   │   │   ├── gc.rs        # Garbage collection
│   │   │   └── snapshot.rs  # Snapshot management
│   │   └── tests/
│   │
│   ├── mdcs-db/             # Phase 6: Database layer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── document.rs  # Document model
│   │   │   ├── api.rs       # Public API
│   │   │   ├── storage.rs   # Storage backend trait
│   │   │   └── index.rs     # Secondary indexes
│   │   └── tests/
│   │
│   └── mdcs-sim/            # Testing infrastructure
│       ├── src/
│       │   ├── lib.rs
│       │   ├── network.rs   # Simulated network
│       │   ├── faults.rs    # Fault injection
│       │   └── oracle.rs    # Reference interpreter
│       └── tests/
│
├── benches/                 # Criterion benchmarks
├── examples/                # Usage examples
└── docs/                    # Documentation & graphs
```