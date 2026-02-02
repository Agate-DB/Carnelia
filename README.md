
## MDCS: Modular Distributed CRDTs in Rust

An architecture for lock-free, offline-first, open-membership databases
that systematically addresses the gaps in existing CRDT systems.

---

## Core Concepts

### What is a CRDT?

A **Conflict-free Replicated Data Type (CRDT)** is a data structure that can be replicated across multiple nodes, where each replica can be updated independently without coordination, and all replicas are guaranteed to **converge** to the same state.

This means: **No locks. No consensus. Always available. Eventually consistent.**

---

## Mathematical Foundation: The Lattice

At the heart of MDCS lies the **join-semilattice** — a mathematical structure that guarantees convergence.

### The Lattice Trait

A join-semilattice $(S, \sqcup)$ is a set $S$ with a **join** operation $\sqcup$ satisfying three laws:

| Property | Definition | Intuition |
|----------|------------|-----------|
| **Commutativity** | $a \sqcup b = b \sqcup a$ | Order doesn't matter |
| **Associativity** | $(a \sqcup b) \sqcup c = a \sqcup (b \sqcup c)$ | Grouping doesn't matter |
| **Idempotence** | $a \sqcup a = a$ | Duplicates are harmless |

These three properties guarantee that **no matter what order updates arrive**, all replicas converge to the same state.

```rust
pub trait Lattice: Clone + PartialEq {
    /// The bottom element ⊥ (identity for join: a ⊔ ⊥ = a)
    fn bottom() -> Self;
    
    /// Join operation (least upper bound)
    fn join(&self, other: &Self) -> Self;
    
    /// Partial order: a ≤ b iff a ⊔ b = b
    fn leq(&self, other: &Self) -> bool;
}
```

### The Partial Order

The join operation induces a **partial order** on states:

$$a \leq b \iff a \sqcup b = b$$

This means $b$ "contains" all information in $a$. When two states are **incomparable** (neither $a \leq b$ nor $b \leq a$), they represent **concurrent** updates that will be merged.

---

## CRDT Types in `mdcs-core`

### 1. GSet (Grow-only Set)

The simplest CRDT. Elements can only be added, never removed.

| Operation | Implementation |
|-----------|---------------|
| `insert(x)` | $S' = S \cup \{x\}$ |
| `join(A, B)` | $A \sqcup B = A \cup B$ |

**Use case**: Tracking unique events, user IDs, immutable logs.

---

### 2. ORSet (Observed-Remove Set)

A set that supports both **add** and **remove** with proper semantics.

**The Challenge**: In distributed systems, "add wins" vs "remove wins" creates conflicts.

**The Solution**: Tag each addition with a unique identifier (replica_id, unique_id).

| Operation | Behavior |
|-----------|----------|
| `add(x)` | Creates a new unique tag for $x$ |
| `remove(x)` | Removes all *observed* tags for $x$ |
| `join` | Union of entries, union of tombstones |

```
Replica A: add("apple") → {apple: {tag1}}
Replica B: add("apple") → {apple: {tag2}}
           remove("apple") → tombstones: {tag2}

After merge: {apple: {tag1}}  ← A's add survives!
```

**Semantics**: Add-wins. A concurrent add and remove results in the element being present.

---

### 3. PNCounter (Positive-Negative Counter)

A counter that supports both **increment** and **decrement**.

**Structure**: Two maps tracking increments and decrements per replica.

$$\text{value} = \sum_{r} P[r] - \sum_{r} N[r]$$

| Operation | Implementation |
|-----------|---------------|
| `increment(r, n)` | $P[r] = P[r] + n$ |
| `decrement(r, n)` | $N[r] = N[r] + n$ |
| `join` | Component-wise max: $P'[r] = \max(P_1[r], P_2[r])$ |

**Why it works**: Each replica only modifies its own entry. The max operation is idempotent and commutative.

---

### 4. LWWRegister (Last-Writer-Wins Register)

A single-value register where **the highest timestamp wins**.

**Structure**: `(value, timestamp, replica_id)`

| Comparison | Winner |
|------------|--------|
| $t_1 > t_2$ | Value with $t_1$ |
| $t_1 = t_2$ | Tie-break on `replica_id` |

```rust
join(reg1, reg2) = if reg1.timestamp > reg2.timestamp { reg1 } 
                   else if reg2.timestamp > reg1.timestamp { reg2 }
                   else { /* tie-break on replica_id */ }
```

**Trade-off**: Simple but can lose concurrent updates. Use when "last write" semantics are acceptable.

---

### 5. MVRegister (Multi-Value Register)

Preserves **all concurrent writes** instead of picking a winner.

**Structure**: Map from unique dots to values: `{Dot → Value}`

| Scenario | Result |
|----------|--------|
| Sequential writes | Latest value only |
| Concurrent writes | All values preserved |
| Resolve conflict | Application chooses one |

```
Replica A writes "Alice"  → {dot1: "Alice"}
Replica B writes "Bob"    → {dot2: "Bob"}

After merge: {dot1: "Alice", dot2: "Bob"}  ← Both preserved!

Application resolves: choose "Alice" → {dot3: "Alice"}
```

**Use case**: Collaborative editing where conflicts should be shown to users.

---

### 6. CRDTMap (Composable Document Container)

A map that can contain other CRDTs, enabling **nested document structures**.

**Key Design**: A **shared causal context** tracks all operations across the entire document tree.

```rust
struct CRDTMap<K> {
    entries: BTreeMap<K, BTreeMap<Dot, MapValue>>,
    context: CausalContext,  // Shared across all nested values
}
```

**Why shared context?** Ensures causality is tracked consistently when keys are removed and re-added concurrently.

---

## Delta-State CRDTs (`mdcs-delta`)

### The Problem with State-Based CRDTs

Sending **full state** on every sync is wasteful:

```
Replica A: {1, 2, 3, ..., 1000000}  ← 1MB
Add element 1000001
Send to B: {1, 2, 3, ..., 1000001}  ← Still 1MB!
```

### The δ-CRDT Solution

Send only the **delta** (change), not the full state:

```
Replica A: {1, 2, 3, ..., 1000000}
Add element 1000001
Send to B: {1000001}  ← Just 8 bytes!
```

### The Delta-Mutator Property

A **delta-mutator** $m^\delta$ satisfies:

$$m(X) = X \sqcup m^\delta(X)$$

Where:
- $m(X)$ = applying mutation $m$ directly to state $X$
- $m^\delta(X)$ = the delta representing the change
- $X \sqcup m^\delta(X)$ = joining the delta with the state

**This means**: You can either mutate directly, or compute a delta and join it — the result is identical.

### Delta Buffer & Grouping

Deltas are buffered and grouped before transmission:

```rust
struct DeltaBuffer<D: Lattice> {
    deltas: VecDeque<(SeqNo, D)>,  // (sequence number, delta)
    current_seq: SeqNo,
}
```

**Grouping**: Multiple deltas can be joined into one:
$$\Delta_{group} = \delta_1 \sqcup \delta_2 \sqcup \delta_3$$

This reduces message overhead while preserving correctness (thanks to associativity).

---

## Anti-Entropy Protocol

The synchronization protocol that ensures eventual convergence.

### Algorithm 1: Convergence Mode

```
On local mutation m:
    d = mδ(X)           // Compute delta
    X = X ⊔ d           // Apply locally
    buffer.push(d)      // Queue for sending

On send to peer j:
    send buffer[acked[j]..]   // Send un-acked deltas

On receive delta d from peer i:
    X = X ⊔ d           // Apply (idempotent!)
    send ack(seq) to i  // Acknowledge receipt
```

### Network Fault Tolerance

The protocol handles:

| Fault | Handling |
|-------|----------|
| **Message loss** | Retransmit until acked |
| **Duplication** | Idempotence: $X \sqcup d \sqcup d = X \sqcup d$ |
| **Reordering** | Commutativity: order doesn't matter |
| **Partitions** | Each partition progresses; merge on heal |

### Convergence Guarantee

**Theorem**: If all deltas are eventually delivered to all replicas, all replicas converge to the same state.

**Proof sketch**: 
1. All deltas are elements of a join-semilattice
2. The final state is $X = \bigsqcup_{i} \delta_i$ (join of all deltas)
3. By commutativity and associativity, delivery order doesn't affect the result
4. By idempotence, duplicates don't affect the result ∎

---

## Quick Reference

### Lattice Laws (must hold for correctness)

```
a ⊔ b = b ⊔ a                    // Commutativity
(a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)        // Associativity  
a ⊔ a = a                        // Idempotence
a ⊔ ⊥ = a                        // Bottom is identity
```

### CRDT Selection Guide

| Need | Use |
|------|-----|
| Add-only collection | `GSet` |
| Add/remove collection | `ORSet` |
| Distributed counter | `PNCounter` |
| Single value, last wins | `LWWRegister` |
| Single value, preserve conflicts | `MVRegister` |
| Nested documents | `CRDTMap` |

---

## Project Structure Overview

```txt
mdcs/
├── crates/
│   ├── mdcs-core/           # Phase 1: CRDT kernel
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── lattice.rs   # Join-semilattice trait
│   │   │   ├── gset.rs      # Grow-only set
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
│   │   │   ├── mutators.rs  # Delta-mutators for each type
│   │   │   ├── buffer.rs    # Delta buffer & grouping
│   │   │   └── anti_entropy.rs  # Sync protocol
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
└── docs/                    # Documentation
```

---

## Running Tests

```bash
# Run all tests
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture

# Run specific crate tests
cargo test -p mdcs-core
cargo test -p mdcs-delta
```