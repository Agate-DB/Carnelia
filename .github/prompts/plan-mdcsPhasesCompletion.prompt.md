# MDCS Phases Completion Plan

## Current Status Assessment

| Phase | Status | Completion |
|-------|--------|------------|
| **Phase 1** | INCOMPLETE | ~60% |
| **Phase 2** | INCOMPLETE | ~75% |

### Phase 1 — What's Done
- ✅ Lattice trait (`join`, `leq`, `bottom`) in `lattice.rs`
- ✅ GSet with property tests
- ✅ OR-Set (add-wins) with Tag-based tracking
- ✅ Basic serialization via serde derives

### Phase 1 — What's Missing
- ❌ PNCounter (commented out / broken)
- ❌ LWW Register (empty file)
- ❌ Multi-Value Register (empty file)
- ❌ Map CRDT (empty file)
- ❌ Property tests for ORSet and other types

### Phase 2 — What's Done
- ✅ Delta buffer with compaction
- ✅ Delta-group join
- ✅ Anti-entropy Algorithm 1 with NetworkSimulator
- ✅ Convergence tests under loss/dup/reorder
- ✅ GSet delta-mutators
- ✅ ORSet delta-mutators (partial)

### Phase 2 — What's Missing
- ❌ Delta-mutators for PNCounter (blocked by Phase 1)
- ❌ Delta-mutators for LWW/MVReg (blocked by Phase 1)

---

## Phase 1 & 2 Completion Steps

### 1. Implement PNCounter
**File:** `crates/mdcs-core/src/pncounter.rs`

- Define `PNCounter<K>` with `increments: HashMap<K, u64>` and `decrements: HashMap<K, u64>`
- Implement `Lattice` trait with component-wise max join
- Add `increment(replica_id)`, `decrement(replica_id)`, `value() -> i64`
- Add serde derives for serialization

### 2. Implement LWW Register
**File:** `crates/mdcs-core/src/lwwreg.rs`

- Define `LWWRegister<T>` with `value: Option<T>`, `timestamp: u64`, `replica_id: K`
- Implement `Lattice` where higher timestamp wins (tie-break on replica_id)
- Add `set(value, timestamp, replica_id)`, `get() -> Option<&T>`

### 3. Implement Multi-Value Register
**File:** `crates/mdcs-core/src/mvreg.rs`

- Define `MVRegister<T>` using `values: HashMap<Tag, T>` dot-store approach
- Implement `Lattice` with union semantics for concurrent writes
- Add `write(value, tag)`, `read() -> Vec<&T>`

### 4. Implement Map CRDT
**File:** `crates/mdcs-core/src/map.rs`

- Define `CRDTMap<K, V: Lattice>` with shared causal context
- Implement recursive nesting support
- Implement `Lattice` trait via per-key join
- Add `put(key, value)`, `get(key)`, `remove(key)`

### 5. Add Delta-Mutators for New Types
**File:** `crates/mdcs-delta/src/mutators.rs`

- PNCounter: `increment_delta(replica_id)`, `decrement_delta(replica_id)`
- LWWRegister: `set_delta(value, timestamp, replica_id)`
- MVRegister: `write_delta(value, tag)`

### 6. Enable Property Tests
**File:** `crates/mdcs-core/tests/properties.rs`

- Instantiate `lattice_laws!` macro for: GSet, ORSet, PNCounter, LWWRegister, MVRegister
- Add serialization round-trip tests
- Add convergence tests for random delivery order

---

## Phase 3 — Causal Consistency Mode (Weeks 9–12)

**Goal:** Optionally provide causal consistency using delta-interval discipline.

### Deliverables

1. **Extend DeltaBuffer with sequence-number tracking**
   - File: `crates/mdcs-delta/src/buffer.rs`
   - Add per-peer `(ci, Di, Ai)` state for delta-interval discipline (Algorithm 2)
   - Track durable counter `ci` per replica

2. **Implement delta-interval anti-entropy**
   - File: `crates/mdcs-delta/src/anti_entropy.rs`
   - Add `DeltaInterval` message type with sequence range
   - Track ack-maps per peer
   - GC deltas only when acked by all tracked neighbors

3. **Add durable state simulation**
   - Create crash/restart test scaffolding
   - Persist `(Xi, ci)` state
   - Verify no deltas skipped on recovery

4. **Causal ordering tests**
   - Verify causal delta-merging condition
   - Test under simulated partitions and restarts

---

## Phase 4 — Merkle-Clock Sync Substrate (Weeks 13–16)

**Goal:** Allow open membership, network unreliability, and gap repair.

### Deliverables

1. **Create new crate**
   - Path: `crates/mdcs-merkle/`
   - Cargo.toml with dependencies on mdcs-core, mdcs-delta

2. **Define MerkleNode struct**
   ```rust
   struct MerkleNode {
       cid: Hash,           // Content identifier (SHA-256)
       parents: Vec<Hash>,  // Causal predecessors
       payload: Payload,    // DeltaGroup or Snapshot
       timestamp: u64,      // Logical timestamp
   }
   ```

3. **Implement DAGStore trait**
   ```rust
   trait DAGStore {
       fn get(&self, cid: &Hash) -> Option<MerkleNode>;
       fn put(&mut self, node: MerkleNode) -> Hash;
       fn heads(&self) -> Vec<Hash>;
       fn contains(&self, cid: &Hash) -> bool;
   }
   ```

4. **Implement DAGSyncer**
   - Gap-repair logic: traverse predecessor hashes
   - Fetch missing nodes from peers recursively
   - Handle concurrent heads (multi-root)

5. **Implement Broadcaster**
   - Gossip new head CIDs to peers asynchronously
   - Configurable fanout and interval

6. **Integration tests**
   - Bootstrap new replica from root CID
   - Partition/heal scenario with multi-root merge
   - Verify identical state after sync

---

## Phase 5 — Compaction & Stability (Weeks 17–22)

**Goal:** Bound metadata (operation logs, causal context, Merkle history).

### Deliverables

1. **Create compaction crate**
   - Path: `crates/mdcs-compaction/`
   - Dedicated subsystem for snapshot and GC policies

2. **Implement snapshotting**
   - Serialize full CRDT state at stable frontiers
   - Derive compact version vector from causal context
   - Store snapshot as special MerkleNode

3. **Implement DAG pruning**
   - Policy to prune nodes older than last snapshot root
   - Configurable retention period
   - Safe deletion verification

4. **Stability monitor**
   - Track "known delivered frontier" across replicas
   - Track "stable frontier" for safe compaction
   - Expose metrics for monitoring

5. **"No resurrection" tests**
   - After compaction, removed items must stay removed
   - Test with late-arriving deltas

6. **Deterministic rebuild tests**
   - Rebuild from snapshot + subsequent deltas
   - Verify identical state to full replay

---

## Phase 6 — Database Layer (Weeks 23–30)

**Goal:** Expose a usable database API.

### Deliverables

1. **Create database crate**
   - Path: `crates/mdcs-db/`
   - High-level document API

2. **Define Document model**
   ```rust
   type Document = CRDTMap<Path, CRDTValue>;
   
   enum CRDTValue {
       Register(LWWRegister<Value>),
       Counter(PNCounter<ReplicaId>),
       Set(ORSet<Value>),
       Map(CRDTMap<String, CRDTValue>),
   }
   ```

3. **Implement API**
   ```rust
   trait DocumentStore {
       fn put(&mut self, doc_id: &str, path: &str, value: CRDTValue);
       fn get(&self, doc_id: &str, path: &str) -> Option<&CRDTValue>;
       fn increment(&mut self, doc_id: &str, path: &str, replica_id: K);
       fn add_to_set(&mut self, doc_id: &str, path: &str, value: V);
   }
   ```

4. **Query support**
   - Key-path lookups
   - Prefix scans within documents
   - Iterator over document keys

5. **Optional: Materialized views**
   - CRDT-derived indexes as secondary views
   - Auto-update on state change

---

## Phase 7 — Benchmarks & Evaluation (Weeks 31–36)

**Goal:** Measure system performance under realistic workloads.

### Workloads

1. **Collaborative editing** — Many small updates, high concurrency
2. **Offline-first mobile** — Bursty sync after prolonged offline
3. **Game state** — High-frequency counter/set operations

### Metrics to Measure

| Metric | Description |
|--------|-------------|
| Time-to-convergence | Time from partition heal to consistent state |
| Bytes transferred | Network overhead for sync operations |
| Storage growth | Size of deltas + Merkle DAG over time |
| Local apply latency | Time to apply a delta locally |
| Memory usage | RAM consumption under load |

### Implementation

1. **Create `benches/` directory** with Criterion.rs
2. **Micro-benchmarks** for each CRDT operation
3. **Macro-benchmarks** simulating multi-replica scenarios
4. **Generate reports** with graphs and tables

---

## Phase 8 — Documentation & Packaging (Weeks 37–40)

**Goal:** Prepare for publication and external use.

### Deliverables

1. **Reproducible evaluation harness**
   - Scripts to run all benchmarks
   - Docker container for consistent environment
   - CI integration for regression testing

2. **Architecture documentation**
   - Expand README with system design
   - API reference with examples
   - Migration guides

3. **Research report/paper**
   - Problem domain and motivation
   - Literature survey
   - Solution architecture
   - Evaluation results
   - Limitations and future work

---

## Open Questions for Refinement

1. **Serialization versioning:** Add version byte prefix now or defer?
   - Recommendation: Add now to avoid migration pain later

2. **ORSet `remove_delta` correctness:** Current implementation may not access internal tags properly
   - Action: Review and fix before Phase 3

3. **Test organization:** `properties.rs` macro unused, `convergence.rs` empty
   - Action: Consolidate and populate during Phase 1/2 completion

4. **Causal context sharing:** How to efficiently share context across nested Map CRDTs?
   - Research: Look at Automerge and Yjs approaches

5. **Hash algorithm choice:** SHA-256 vs BLAKE3 for Merkle nodes?
   - Trade-off: Compatibility vs performance
