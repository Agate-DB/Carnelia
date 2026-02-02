
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

## Merkle-Clock DAG (`mdcs-merkle`)

The **Merkle-Clock** is a content-addressed DAG that provides verifiable, tamper-proof causal history for CRDT updates.

### Why Merkle-DAGs for Causality?

Traditional Vector Clocks have critical limitations in open-membership systems:

| Problem | Vector Clocks | Merkle-Clock |
|---------|--------------|---------------|
| **Metadata overhead** | Grows with replica count | Independent of replica count |
| **Byzantine tolerance** | Vulnerable to ID reuse | Hash-based, tamper-proof |
| **Verification** | Requires trust | Cryptographically verifiable |

### Structure

Each node in the Merkle-DAG contains:

```rust
struct MerkleNode {
    parents: Vec<Hash>,      // Links to predecessor nodes
    payload: Payload,        // Delta, snapshot, or genesis
    timestamp: u64,          // Logical timestamp
    creator: String,         // Replica that created this node
    signature: Option<Vec<u8>>,  // Optional cryptographic signature
}
```

The node's **Content Identifier (CID)** is the cryptographic hash of its contents:

$$\text{CID} = H(\text{parents} \| \text{payload} \| \text{timestamp} \| \text{creator})$$

### Causal Ordering

The DAG structure explicitly encodes the **happens-before** relationship:

```
       [Genesis]
           │
        [Op A]  ← Replica 1
        /    \
    [Op B]  [Op C]  ← Concurrent (Replica 1, Replica 2)
        \    /
        [Merge]  ← Joins concurrent branches
```

- **A → B** means A is an ancestor of B (A happened before B)
- **B ∥ C** means B and C are concurrent (neither is ancestor of the other)

### DAG-Syncer Protocol

Synchronization follows a **gossip + pull** pattern:

```
1. Broadcaster: gossip current head CIDs to peers
2. On receiving unknown CID:
   - Traverse backwards via parent links
   - Fetch missing nodes from peers
   - Stop at common ancestor (already have)
3. Apply deltas in topological order
```

| Property | Guarantee |
|----------|----------|
| **Consistency** | Same heads → same history |
| **Integrity** | CID = hash of contents |
| **Convergence** | Eventually all replicas sync |

---

## Compaction & Stability (`mdcs-compaction`)

The compaction subsystem bounds metadata growth while preserving correctness.

### The Problem: Unbounded Growth

Without compaction, the Merkle-DAG grows forever:
- Every operation adds a node
- Old tombstones accumulate
- Bootstrap time increases

### The Solution: Safe Pruning

MDCS uses a **stability-based compaction** strategy:

#### 1. Version Vectors

Compact representation of causal context:

```rust
VersionVector {
    entries: BTreeMap<ReplicaId, SequenceNumber>
}
```

Operations:
- **Dominates**: $VV_1 \geq VV_2$ iff $\forall r: VV_1[r] \geq VV_2[r]$
- **Concurrent**: Neither dominates the other
- **Merge**: Component-wise max (LUB)

#### 2. Stability Monitor

Tracks the **stable frontier** — updates that all replicas have received:

```rust
struct StabilityMonitor {
    local_frontier: VersionVector,
    peer_frontiers: HashMap<ReplicaId, VersionVector>,
}
```

**Stable Frontier** = min of all known frontiers:

$$\text{stable}[r] = \min_{p \in \text{peers}} \text{frontier}_p[r]$$

An update is **stable** when it's below the stable frontier — all replicas have it.

#### 3. Snapshots

Periodic snapshots capture full CRDT state:

```rust
struct Snapshot {
    version_vector: VersionVector,  // State coverage
    superseded_roots: Vec<Hash>,    // DAG heads at snapshot time
    state_data: Vec<u8>,            // Serialized CRDT state
    created_at: u64,
    creator: String,
}
```

Snapshots enable:
- **Fast bootstrap**: New replicas start from snapshot, not genesis
- **Safe pruning**: History before snapshot can be garbage collected

#### 4. Pruning Policy

```rust
struct PruningPolicy {
    min_node_age: u64,        // Don't prune recent nodes
    preserve_depth: usize,    // Keep N ancestors of heads
    max_nodes_per_prune: usize,
    preserve_genesis_path: bool,
}
```

**Safety Invariant**: Only prune nodes that are:
1. Ancestors of the snapshot's superseded roots
2. Older than `min_node_age`
3. Beyond `preserve_depth` from current heads

### No-Resurrection Guarantee

**Critical invariant**: Deleted items must stay deleted after compaction.

Achieved via **tombstone-free removal**:

| Component | Contents |
|-----------|----------|
| **Causal Context** | All dots (event IDs) ever created |
| **Dot Store** | Only dots for "live" data |

$$\text{deleted}(x) \iff \text{dot}(x) \in \text{context} \land \text{dot}(x) \notin \text{store}$$

The causal context grows monotonically, so a deleted item's dot remains "known" even after the item is gone. This prevents resurrection from late-arriving adds.

### Compaction Workflow

```
1. Track peer frontiers via gossip
2. Compute stable frontier (min of all)
3. When stable frontier advances:
   a. Create snapshot at stable point
   b. Identify prunable nodes (ancestors of snapshot)
   c. Verify no-resurrection invariant
   d. Execute pruning
```

---

## Database Layer (`mdcs-db`)

The database layer provides high-level collaborative data structures for building real-time applications like Google Docs, Figma, or Notion.

### RGA List (Replicated Growable Array)

An ordered list CRDT with support for insert, delete, and move operations.

```rust
let mut list = RGAList::new("replica-1");
list.push_back("Alice");
list.push_back("Bob");
list.insert(1, "Charlie");  // Insert at position 1

// Result: ["Alice", "Charlie", "Bob"]
```

**Key features**:
- Unique `ListId` identifiers with ULID-based ordering
- Deterministic conflict resolution for concurrent inserts
- Tombstone-based deletion (nodes marked deleted, not removed)
- Delta-based replication support

### RGA Text

A text CRDT built on RGAList for collaborative text editing.

```rust
let mut text = RGAText::new("replica-1");
text.insert(0, "Hello");
text.insert(5, " World");
text.delete(0, 6);  // Delete "Hello "

// Result: "World"
```

**Features**:
- Character-level operations
- Position ↔ ID conversion for cursor synchronization
- Efficient range operations (slice, delete range)
- Lattice-based merge for concurrent edits

### Rich Text

Rich text with formatting marks that survive concurrent edits.

```rust
let mut doc = RichText::new("replica-1");
doc.insert(0, "Hello World");
doc.bold(0, 5);        // Bold "Hello"
doc.italic(6, 11);     // Italic "World"
doc.link(0, 5, "https://example.com");

// Renders: <a href="..."><b>Hello</b></a> <i>World</i>
```

**Mark types**:
- **Bold**, **Italic**, **Underline**, **Strikethrough**
- **Links** with URL
- **Comments** with author and content
- **Custom attributes** for extensibility

**Key design**:
- Anchor-based positioning (marks reference text IDs, not positions)
- Marks expand automatically when text is inserted within them
- ULID-based conflict resolution for overlapping marks

### JSON CRDT

Automerge-like nested document CRDT with path-based operations.

```rust
let mut doc = JsonCrdt::new("replica-1");

// Set nested values
doc.set(&JsonPath::parse("user.name"), JsonValue::String("Alice".into()))?;
doc.set(&JsonPath::parse("user.age"), JsonValue::Int(30))?;

// Array operations
let items_id = doc.set_array(&JsonPath::parse("items"))?;
doc.array_push(&items_id, JsonValue::String("item1".into()))?;

// Convert to JSON
let json = doc.to_json();
// {"user": {"name": "Alice", "age": 30}, "items": ["item1"]}
```

**Features**:
- Path-based get/set operations
- Nested objects and arrays
- Multi-value registers for concurrent field writes (conflicts visible to app)
- Shared causal context across the entire document tree

### Document Store

High-level API for managing collections of documents.

```rust
let mut store = DocumentStore::new("replica-1");

// Create documents
let doc_id = store.create_text("doc-1", "Meeting Notes")?;
store.text_insert(&doc_id, 0, "# Agenda\n")?;
store.text_insert(&doc_id, 9, "1. Review\n2. Planning")?;

// Query documents
let results = store.query(QueryOptions {
    document_type: Some(DocumentType::Text),
    title_prefix: Some("Meeting".into()),
    limit: Some(10),
    ..Default::default()
})?;

// Get changes for replication
let changes = store.take_changes();
```

### Presence System

Real-time cursor and user presence tracking for collaborative UIs.

```rust
let mut tracker = PresenceTracker::new(
    UserId::new("user-1"),
    UserInfo { name: "Alice".into(), avatar_url: None }
);

// Update cursor position
tracker.set_cursor("doc-1", Cursor::new(42).with_selection(42, 50));

// Track status
tracker.set_status(UserStatus::Away);

// Get all users in a document
for (user_id, presence) in tracker.users_in_document("doc-1") {
    let color = tracker.colors().get(&user_id);
    // Render cursor with assigned color
}
```

**Features**:
- Cursor position and selection tracking
- User status (Online, Away, Busy, Offline)
- Automatic color assignment for visual distinction
- Stale presence detection and cleanup
- Delta-based sync for efficiency

### Undo/Redo System

Operation-based undo with causal grouping for collaborative editing.

```rust
let mut undo_manager = UndoManager::new("user-1");

// Record operations
undo_manager.record(UndoableOperation::TextInsert {
    position: 0,
    text: "Hello".into(),
});

// Group related operations
undo_manager.start_group();
undo_manager.record(UndoableOperation::TextDelete { position: 0, length: 5 });
undo_manager.record(UndoableOperation::TextInsert { position: 0, text: "Hi".into() });
undo_manager.end_group();

// Undo (returns inverse operations to apply)
let inverses = undo_manager.undo();

// Redo
let operations = undo_manager.redo();
```

**Features**:
- Automatic inverse operation generation
- Operation grouping for batch undo
- Configurable history limits
- Collaborative undo manager (per-user undo stacks)

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
| Ordered list with move | `RGAList` |
| Collaborative plain text | `RGAText` |
| Collaborative rich text | `RichText` |
| JSON-like documents | `JsonCrdt` |
| Document collections | `DocumentStore` |
| Real-time cursors | `PresenceTracker` |

---

## Project Structure Overview

```txt
mdcs/
├── crates/
│   ├── mdcs-core/           # Phase 1: CRDT kernel ✓
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
│   ├── mdcs-delta/          # Phase 2-3: Delta-state layer ✓
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── mutators.rs  # Delta-mutators for each type
│   │   │   ├── buffer.rs    # Delta buffer & grouping
│   │   │   └── anti_entropy.rs  # Sync protocol
│   │   └── tests/
│   │
│   ├── mdcs-merkle/         # Phase 4: Merkle-Clock ✓
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── hash.rs      # Content-addressed hashing
│   │   │   ├── node.rs      # DAG node structure
│   │   │   ├── store.rs     # DAG storage & operations
│   │   │   ├── syncer.rs    # DAGSyncer reconciliation
│   │   │   └── broadcaster.rs  # Gossip protocol
│   │   └── tests/
│   │
│   ├── mdcs-compaction/     # Phase 5: Compaction & Stability ✓
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── version_vector.rs  # Compact causal context
│   │   │   ├── stability.rs # Stability monitoring
│   │   │   ├── snapshot.rs  # Snapshot management
│   │   │   ├── pruning.rs   # Safe DAG pruning
│   │   │   └── compactor.rs # High-level orchestration
│   │   └── tests/
│   │
│   ├── mdcs-db/             # Phase 6: Database layer ✓
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── error.rs     # Error types
│   │   │   ├── rga_list.rs  # Replicated Growable Array
│   │   │   ├── rga_text.rs  # Collaborative text CRDT
│   │   │   ├── rich_text.rs # Rich text with formatting
│   │   │   ├── json_crdt.rs # Automerge-like nested JSON
│   │   │   ├── document.rs  # Document store API
│   │   │   ├── presence.rs  # Cursor & presence tracking
│   │   │   └── undo.rs      # Undo/redo system
│   │   └── tests/
│   │
│   └── mdcs-sim/            # Testing infrastructure (planned)
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
cargo test -p mdcs-merkle
cargo test -p mdcs-compaction
cargo test -p mdcs-db
```

---

## Implementation Status

| Phase | Component | Status | Tests |
|-------|-----------|--------|-------|
| 1 | CRDT Kernel (`mdcs-core`) | ✅ Complete | 34 passing |
| 2-3 | Delta-State Layer (`mdcs-delta`) | ✅ Complete | 67 passing |
| 4 | Merkle-Clock (`mdcs-merkle`) | ✅ Complete | 45 passing |
| 5 | Compaction (`mdcs-compaction`) | ✅ Complete | 46 passing |
| 6 | Database Layer (`mdcs-db`) | ✅ Complete | 66 passing |
| 7 | Benchmarks | 🔲 Planned | - |
| 8 | Documentation | 🔲 Planned | - |