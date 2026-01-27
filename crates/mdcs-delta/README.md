# MDCS Delta - Delta-State CRDT Framework

This crate implements the δ-CRDT (delta-state CRDT) framework for efficient synchronization.

## Overview

The δ-CRDT framework provides:

1. **Delta-mutators** (`mδ`): Functions that compute minimal state changes
2. **Delta buffers**: Grouping and batching deltas for transmission  
3. **Anti-entropy Algorithm 1**: Convergence protocol from the δ-CRDT paper

## Key Property

For any delta-mutator `mδ`:
```
m(X) = X ⊔ mδ(X)
```

This means the full mutation result can be reconstructed by joining the delta with the original state.

## Algorithm 1: Convergence Mode

```
On local mutation m:
  d = mδ(X)     // compute delta
  X = X ⊔ d     // apply to local state
  D.push(d)     // buffer for sending

On send to peer j:
  send D[acked[j]..] to j

On receive delta d from peer i:
  X = X ⊔ d     // apply (idempotent!)
  send ack(seq) to i
```

## Modules

### `buffer`
- `DeltaBuffer<D>`: Buffering deltas with grouping and compaction
- `DeltaReplica<S, D>`: A replica with integrated delta management
- `AckTracker`: Tracking acknowledgments from peers

### `mutators`
- `gset`: Delta-mutators for GSet (grow-only set)
- `orset`: Delta-mutators for ORSet (observed-remove set)

### `anti_entropy`
- `AntiEntropyCluster<S>`: Multi-replica cluster with simulated network
- `NetworkSimulator<D>`: Simulates loss, duplication, reordering
- `NetworkConfig`: Configuration for network simulation

## Usage

```rust
use mdcs_delta::buffer::DeltaReplica;
use mdcs_core::gset::GSet;

// Create a replica
let mut replica: DeltaReplica<GSet<i32>> = DeltaReplica::new("replica1");

// Mutate using delta-mutator
replica.mutate(|_state| {
    let mut delta = GSet::new();
    delta.insert(42);
    delta
});

assert!(replica.state().contains(&42));
```

## Testing Convergence

The crate includes comprehensive tests proving convergence under:
- **Message loss**: Handled by retransmission
- **Message duplication**: Handled by idempotence (a ⊔ a = a)
- **Message reordering**: Handled by commutativity (a ⊔ b = b ⊔ a)

Run tests with:
```bash
cargo test -p mdcs-delta
```

## References

- Almeida, P. S., Shoker, A., & Baquero, C. (2018). Delta state replicated data types. *Journal of Parallel and Distributed Computing*, 111, 162-173.

