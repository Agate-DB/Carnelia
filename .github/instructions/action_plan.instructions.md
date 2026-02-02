---
applyTo: '**'
---

## 0) Non-negotiable principles (keep the project sane)

1) **Correctness first, performance second.** Distributed bugs are exponential; optimizations come after invariants.
2) **Model the system as hostile.** Messages can be lost/duplicated/reordered; partitions happen.
3) **Every optimization is a semantics change until proven otherwise.** Use a reference oracle (OpSets-style or a slow interpreter).
4) **Make “compaction/GC” an explicit subsystem** with tests. Metadata removal is correctness-sensitive.

---

## 1) Phase plan (recommended sequence)

### Phase 1 — Foundational CRDT kernel (Weeks 1–4)

**Deliverables**
- Core join-semilattice interface (`join`, `leq`, `bottom`).
- Implement a minimal portfolio of CRDTs:
  - Grow-only set (GSet)
  - GCounter / PNCounter
  - OR-Set or add-wins set variant
  - Register (multi-value register or LWW)
- Serialization format for states/deltas.

**Success criteria**
- Property-based tests prove:
  - commutativity, associativity, idempotence of merge where applicable (CvRDT).
  - convergence after random permutations of delivery order.

**Reading anchors**
- CvRDT/CmRDT and SEC conditions.


### Phase 2 — δ-CRDT layer (Weeks 5–8)

**Goal:** move from shipping “full state” to shipping “delta mutations”.

**Deliverables**
- For each CRDT type, implement delta-mutators `mδ` such that `m(X) = X ⊔ mδ(X)`.
- Delta buffer + delta-group join.
- Implement δ-CRDT anti-entropy Algorithm 1 (convergence-only).

**Tests**
- prove convergence under loss/dup/reorder by repeated re-sends (idempotence).


### Phase 3 — Causal consistency mode for δ-CRDT (Weeks 9–12)

**Goal:** optionally provide causal consistency using delta-interval discipline.

**Deliverables**
- Implement delta-interval based anti-entropy (Algorithm 2 with ack map, sequence numbers).
- Durable state: `(Xi, ci)`; volatile: `(Di, Ai)`.
- GC for deltas that are acked by all tracked neighbors (Algorithm 2’s GC step).

**Tests**
- simulate crashes/restarts: verify durable counter `ci` prevents skipping deltas.
- causal delta-merging condition holds.


### Phase 4 — Merkle-Clock sync substrate (Weeks 13–16)

**Goal:** allow open membership, network unreliability, and gap repair.

**Deliverables**
- Implement Merkle-Clock nodes referencing children and carrying payload references.
- Implement a `DAGSyncer` abstraction: `Get(CID)`, `Put(Node)`.
- Implement a `Broadcaster` abstraction: `Broadcast(rootCID)`.

**How to integrate with δ-CRDT:**
- Each “event” node in Merkle-Clock references either:
  - a delta-group blob, or
  - a snapshot blob (periodic).

**Tests**
- new replica bootstrap: start from root CID and reconstruct state.
- partitions: two roots, merge produces multi-root then new root referencing both (or union strategy).


### Phase 5 — Compaction & stability subsystem (Weeks 17–22)

**Goal:** bound metadata (operation logs, causal context, Merkle history).

**Deliverables**
- Define compaction boundaries:
  - delta compaction: drop delta intervals once stable/acked (from Phase 3).
  - Merkle compaction: periodic snapshots, prune DAG before snapshot root (policy).
- Add a stability monitor:
  - track “known delivered frontier” and “stable frontier”.
  - adopt insights from causal stability work (be explicit about overhead vs cleanup).

**Reactivity safeguards**
- If a causal middleware buffers operations, consider the reactivity approach of exposing buffered ops to evaluation.

**Tests**
- “no resurrection” tests: after compaction, removes remain removed.
- deterministic rebuild tests: rebuild from snapshot + remaining deltas yields same state.


### Phase 6 — Database layer (Weeks 23–30)

**Goal:** expose a usable database API.

**Deliverables**
- Data model: `Document = Map<Path, CRDTValue>`.
- API:
  - `put(docId, path, value)`
  - `get(docId, path)`
  - counters, sets operations
  - optional transactions: *only within a single document* as a first step.

**Querying strategy (minimal viable)**
- Start with:
  - key-path lookups
  - prefix scans within document
- Secondary indexing: treat indexes as CRDT-derived materialized views.

**Notes**
- SQL invariants over weak consistency are hard; Antidote SQL work illustrates the cost and design complexity.


### Phase 7 — Benchmarks & evaluation (Weeks 31–36)

**Workloads**
1) Collaborative editing style (many small updates)
2) Offline-first mobile (bursty sync after offline)
3) Game state (high-frequency counters/sets)

**Measure**
- time-to-convergence after healing
- bytes transferred for sync
- storage growth (deltas + merkle)
- latency for local apply


### Phase 8 — Documentation & research packaging (Weeks 37–40)

**Deliverables**
- A reproducible evaluation harness.
- A paper/report describing:
  - problem domain
  - literature survey
  - solution
  - evaluation
  - limitations

---

## 2) Test strategy (what “correct” looks like)

### 2.1 Property-based invariants
- Merge properties (CvRDT): idempotent, commutative, associative.
- Convergence: all replicas equal after receiving same deltas (order independent).

### 2.2 Fault models (must simulate)
- message loss
- duplication
- reordering
- partitions and heal
- node crash and restart
- new node joins from scratch

### 2.3 Differential oracle
- Use a slow reference interpreter:
  - either compute state by replaying all deltas in deterministic order,
  - or use OpSets-style reference semantics for small models.

---

## 3) Implementation choices (practical guidance)

### 3.1 Pick one language and stay there
For research velocity, TypeScript/Rust are common. Flec (TypeScript) exists as a framework for pure op-based CRDT experimentation, which may be a useful reference point for architecture patterns.

### 3.2 Separate concerns in code
- `crdt/` (data types)
- `delta/` (delta-mutators + buffers)
- `merkle/` (clock + DAG store)
- `sync/` (anti-entropy protocols)
- `compaction/` (snapshots, GC)
- `db/` (documents, API)
- `tests/` and `sim/`

### 3.3 Versioning and upgrades
CRDT storage formats evolve. Include a version byte in every serialized blob.