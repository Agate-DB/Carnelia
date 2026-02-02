# mdcs-compaction

Compaction and stability subsystem for the MDCS (Merkle-Delta CRDT Store).

## Overview

This crate implements **Phase 5** of the MDCS architecture - the Compaction & Stability Subsystem. It provides mechanisms to bound metadata growth by:

1. **Tracking stability** - determining which updates have been durably replicated
2. **Creating snapshots** - capturing full CRDT state at stable frontiers
3. **Pruning history** - safely removing DAG nodes older than the last snapshot
4. **Preventing resurrection** - ensuring deleted items stay deleted after compaction

## Architecture

The compaction subsystem is organized into five main components:

### VersionVector

Compact representation of causal context using `(replica_id, sequence_number)` pairs:

```rust
use mdcs_compaction::VersionVector;

let mut vv = VersionVector::new();
vv.increment("replica_1");
vv.increment("replica_1");
vv.set("replica_2", 5);

// Check dominance (causal ordering)
let vv2 = VersionVector::from_entries([("replica_1".to_string(), 1)]);
assert!(vv.dominates(&vv2));
```

Key features:
- Dominance checking (`dominates`, `strictly_dominates`)
- Concurrency detection (`is_concurrent_with`)
- Merging for LUB computation
- Diff computation for sync optimization

### StabilityMonitor

Tracks the "known delivered frontier" across replicas to determine when updates are stable:

```rust
use mdcs_compaction::{StabilityMonitor, FrontierUpdate, VersionVector};

let mut monitor = StabilityMonitor::new("replica_1");

// Update local frontier after state changes
monitor.update_local_frontier(VersionVector::new(), vec![]);

// Process frontier updates from peers
monitor.update_peer_frontier(FrontierUpdate {
    peer_id: "replica_2".to_string(),
    version_vector: VersionVector::new(),
    heads: vec![],
    timestamp: 100,
});

// Check stability
if monitor.has_quorum() {
    let stable_frontier = monitor.stable_frontier();
    // Safe to compact up to this frontier
}
```

Configuration options:
- `min_peers_for_stability` - minimum peers needed before stability checks
- `require_all_peers` - whether all known peers must acknowledge
- `quorum_fraction` - fraction of peers needed for quorum
- `stale_peer_timeout` - when to consider peers as stale

### SnapshotManager

Manages periodic snapshots of CRDT state for efficient bootstrap and recovery:

```rust
use mdcs_compaction::{Snapshot, SnapshotManager, VersionVector};
use mdcs_merkle::Hash;

let mut manager = SnapshotManager::new();

// Create a snapshot
let vv = VersionVector::from_entries([("r1".to_string(), 100)]);
let snapshot = Snapshot::new(
    vv,
    vec![/* superseded DAG roots */],
    b"serialized CRDT state".to_vec(),
    "r1",
    current_timestamp,
);

// Store and retrieve
let id = manager.store(snapshot);
let latest = manager.latest();
```

Features:
- Automatic garbage collection of old snapshots
- Find snapshots that cover a given version vector
- Configurable snapshot frequency

### Pruner

Identifies and removes DAG nodes that are safely purgeable:

```rust
use mdcs_compaction::{Pruner, PruningPolicy, Snapshot};

// Configure pruning behavior
let policy = PruningPolicy {
    min_node_age: 3600,  // 1 hour minimum age
    preserve_depth: 10,  // Keep 10 ancestors of heads
    max_nodes_per_prune: 1000,
    preserve_genesis_path: true,
    ..Default::default()
};

let pruner = Pruner::with_policy(policy);

// Identify prunable nodes (without modifying store)
let prunable = pruner.identify_prunable(&store, &snapshot, current_time);

// Execute pruning on a PrunableStore
let result = pruner.execute_prune(&mut store, &snapshot, current_time);
```

Safety guarantees:
- Never prunes nodes referenced by current heads
- Preserves configurable depth of history
- Optional genesis path preservation
- Verification utilities to prevent resurrection

### Compactor

High-level orchestrator that coordinates all compaction components:

```rust
use mdcs_compaction::{Compactor, CompactionConfig};

let mut compactor = Compactor::new("replica_1");

// Update local state
compactor.update_local_frontier(version_vector, dag_heads);

// Process peer updates
compactor.process_peer_update(frontier_update);

// Create snapshots periodically
if compactor.should_snapshot() {
    compactor.create_snapshot(dag_heads, || {
        // Serialize your CRDT state
        Ok(crdt.serialize())
    })?;
}

// Run automatic compaction
let result = compactor.compact(&mut store, || {
    Ok(crdt.serialize())
})?;
```

## No-Resurrection Guarantee

A critical invariant of the compaction system is preventing "resurrection" of deleted items. This is achieved through:

1. **Tombstone-free removal** - The causal context tracks all created events, while the dot store only contains live data. An item is deleted when its dot is in the context but not the store.

2. **Snapshot boundaries** - Snapshots record which DAG roots they supersede. Nodes before the snapshot boundary can only be pruned if they're ancestors of those roots.

3. **Verification** - The `PruningVerifier` can check that pruning won't cause resurrection:

```rust
use mdcs_compaction::PruningVerifier;

// Before pruning, verify safety
let result = PruningVerifier::verify_no_resurrection(&store, &prunable_nodes, &snapshot);
if result.is_ok() {
    // Safe to prune
}
```

## Integration with Merkle-Clock

The compaction subsystem integrates with `mdcs-merkle` to prune the Merkle-DAG while preserving causal integrity:

```rust
// The DAG stores the causal history
let (store, genesis) = MemoryDAGStore::with_genesis("creator");

// Snapshots reference DAG heads they supersede
let snapshot = Snapshot::new(
    version_vector,
    store.heads(),  // superseded_roots
    state_data,
    creator,
    timestamp,
);

// Pruning removes ancestors of superseded roots
let prunable = pruner.identify_prunable(&store, &snapshot, current_time);
```

## Tests

The crate includes comprehensive tests:

- **Unit tests** - 33 tests covering all components
- **Integration tests** - 13 tests verifying:
  - No resurrection after compaction
  - Deterministic rebuild from snapshots
  - Multi-replica stability tracking
  - Quorum-based stability
  - Pruning depth preservation

Run tests:
```bash
cargo test -p mdcs-compaction
```

