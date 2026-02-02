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

**Goal:** Expose a usable database API with rich support for collaborative applications.

### Deliverables

1. **Create database crate**
   - Path: `crates/mdcs-db/`
   - High-level document API

2. **Define Document model**
   ```rust
   type Document = CRDTMap<Path, CRDTValue>;
   
   enum CRDTValue {
       // Primitives
       Register(LWWRegister<Value>),
       MVRegister(MVRegister<Value>),      // Multi-value for conflict visibility
       Counter(PNCounter<ReplicaId>),
       Set(ORSet<Value>),
       
       // Nested structures
       Map(CRDTMap<String, CRDTValue>),
       List(RGAList<CRDTValue>),           // Ordered list with insert/delete
       
       // Collaborative text
       Text(RGAText),                       // Plain text sequences
       RichText(RichTextCRDT),             // Formatted text with annotations
       
       // Binary/Objects
       Binary(LWWRegister<Vec<u8>>),       // Raw bytes (images, files)
       Json(JsonCRDT),                      // Arbitrary JSON-like objects
   }
   ```

3. **Implement Core API**
   ```rust
   trait DocumentStore {
       // Basic operations
       fn put(&mut self, doc_id: &str, path: &str, value: CRDTValue);
       fn get(&self, doc_id: &str, path: &str) -> Option<&CRDTValue>;
       fn delete(&mut self, doc_id: &str, path: &str);
       
       // Counter operations
       fn increment(&mut self, doc_id: &str, path: &str, replica_id: K, amount: u64);
       fn decrement(&mut self, doc_id: &str, path: &str, replica_id: K, amount: u64);
       fn get_counter(&self, doc_id: &str, path: &str) -> Option<i64>;
       
       // Set operations
       fn add_to_set(&mut self, doc_id: &str, path: &str, value: V);
       fn remove_from_set(&mut self, doc_id: &str, path: &str, value: V);
       fn set_contains(&self, doc_id: &str, path: &str, value: &V) -> bool;
       
       // List operations
       fn list_insert(&mut self, doc_id: &str, path: &str, index: usize, value: CRDTValue);
       fn list_delete(&mut self, doc_id: &str, path: &str, index: usize);
       fn list_move(&mut self, doc_id: &str, path: &str, from: usize, to: usize);
       fn list_get(&self, doc_id: &str, path: &str) -> Option<Vec<&CRDTValue>>;
   }
   ```

4. **Query support**
   - Key-path lookups
   - Prefix scans within documents
   - Iterator over document keys
   - Range queries on lists

5. **Optional: Materialized views**
   - CRDT-derived indexes as secondary views
   - Auto-update on state change

---

### 6.1 — Collaborative Text Support (Google Docs-like)

**Goal:** Enable real-time collaborative text editing with proper conflict resolution.

#### Text CRDT Implementation
**File:** `crates/mdcs-core/src/text.rs`

```rust
/// RGA (Replicated Growable Array) for text sequences
struct RGAText {
    nodes: HashMap<TextId, TextNode>,
    head: Option<TextId>,
    context: CausalContext,
}

struct TextNode {
    id: TextId,              // (replica_id, sequence)
    char: char,
    deleted: bool,           // Tombstone flag
    left: Option<TextId>,    // Left neighbor at insertion time
    right: Option<TextId>,   // Computed link
}

struct TextId {
    replica: ReplicaId,
    seq: u64,
}
```

#### Text API
```rust
trait CollaborativeText {
    // Core editing
    fn insert(&mut self, position: usize, text: &str, replica_id: K);
    fn delete(&mut self, position: usize, length: usize, replica_id: K);
    
    // Bulk operations
    fn replace(&mut self, start: usize, end: usize, text: &str, replica_id: K);
    fn splice(&mut self, position: usize, delete_count: usize, insert: &str, replica_id: K);
    
    // Read operations
    fn to_string(&self) -> String;
    fn len(&self) -> usize;
    fn char_at(&self, position: usize) -> Option<char>;
    fn slice(&self, start: usize, end: usize) -> String;
    
    // Position mapping (for cursor sync)
    fn id_to_position(&self, id: &TextId) -> Option<usize>;
    fn position_to_id(&self, position: usize) -> Option<TextId>;
}
```

#### Rich Text / Formatting Support
**File:** `crates/mdcs-core/src/richtext.rs`

```rust
/// Rich text with inline formatting (like Peritext)
struct RichTextCRDT {
    text: RGAText,
    marks: MarkSet,          // Formatting annotations
}

/// A mark is a formatting span
struct Mark {
    id: MarkId,
    mark_type: MarkType,
    start: TextId,           // Anchor to text position
    end: TextId,
    attrs: HashMap<String, Value>,
}

enum MarkType {
    // Inline formatting
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Code,
    
    // Links and references
    Link { url: String },
    Mention { user_id: String },
    
    // Semantic
    Highlight { color: String },
    Comment { thread_id: String },
    
    // Custom
    Custom(String),
}

trait RichTextEditor {
    // Formatting operations
    fn add_mark(&mut self, start: usize, end: usize, mark_type: MarkType, replica_id: K);
    fn remove_mark(&mut self, start: usize, end: usize, mark_type: MarkType, replica_id: K);
    fn toggle_mark(&mut self, start: usize, end: usize, mark_type: MarkType, replica_id: K);
    
    // Query formatting
    fn marks_at(&self, position: usize) -> Vec<&Mark>;
    fn marks_in_range(&self, start: usize, end: usize) -> Vec<&Mark>;
    
    // Export
    fn to_html(&self) -> String;
    fn to_markdown(&self) -> String;
    fn to_json(&self) -> serde_json::Value;  // Portable format
}
```

---

### 6.2 — Presence & Awareness (Real-time Collaboration)

**Goal:** Track who's online, where their cursors are, and what they're doing.

**File:** `crates/mdcs-db/src/presence.rs`

```rust
/// Ephemeral presence state (not persisted, uses last-write-wins)
struct PresenceState {
    user_id: String,
    cursor: Option<CursorPosition>,
    selection: Option<Selection>,
    status: UserStatus,
    last_active: u64,
    custom: HashMap<String, Value>,  // App-specific data
}

struct CursorPosition {
    doc_id: String,
    path: String,               // Path within document
    position: usize,            // For text: character offset
    anchor_id: Option<TextId>,  // Stable anchor for text CRDTs
}

struct Selection {
    start: CursorPosition,
    end: CursorPosition,
}

enum UserStatus {
    Active,
    Idle,
    Away,
    Editing { field: String },
    Viewing,
}

trait PresenceManager {
    // Local updates
    fn update_cursor(&mut self, position: CursorPosition);
    fn update_selection(&mut self, selection: Selection);
    fn update_status(&mut self, status: UserStatus);
    fn set_custom(&mut self, key: &str, value: Value);
    
    // Remote awareness
    fn get_peers(&self) -> Vec<&PresenceState>;
    fn get_peer(&self, user_id: &str) -> Option<&PresenceState>;
    fn subscribe<F: Fn(PresenceEvent)>(&mut self, callback: F);
    
    // Cleanup
    fn gc_stale(&mut self, timeout: Duration);
}

enum PresenceEvent {
    PeerJoined(String),
    PeerLeft(String),
    CursorMoved { user_id: String, position: CursorPosition },
    SelectionChanged { user_id: String, selection: Selection },
    StatusChanged { user_id: String, status: UserStatus },
}
```

---

### 6.3 — JSON/Object CRDT (Automerge-like)

**Goal:** Support arbitrary nested JSON-like objects with CRDT semantics.

**File:** `crates/mdcs-core/src/json.rs`

```rust
/// A JSON-compatible CRDT value
enum JsonCRDT {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Text(RGAText),            // Collaborative string
    Array(RGAList<JsonCRDT>), // Ordered with insert/delete
    Object(CRDTMap<String, JsonCRDT>),
}

/// Operations on JSON CRDTs
trait JsonDocument {
    // Path-based access (like JSON pointer)
    fn get_path(&self, path: &[PathSegment]) -> Option<&JsonCRDT>;
    fn set_path(&mut self, path: &[PathSegment], value: JsonCRDT, replica_id: K);
    fn delete_path(&mut self, path: &[PathSegment], replica_id: K);
    
    // Type-specific operations at path
    fn increment_path(&mut self, path: &[PathSegment], amount: f64, replica_id: K);
    fn decrement_path(&mut self, path: &[PathSegment], amount: f64, replica_id: K);
    fn push_path(&mut self, path: &[PathSegment], value: JsonCRDT, replica_id: K);
    fn insert_path(&mut self, path: &[PathSegment], index: usize, value: JsonCRDT, replica_id: K);
    fn text_insert_path(&mut self, path: &[PathSegment], pos: usize, text: &str, replica_id: K);
    
    // Conversion
    fn from_json(json: serde_json::Value) -> Self;
    fn to_json(&self) -> serde_json::Value;
    
    // Diff/patch
    fn diff(&self, other: &Self) -> Vec<JsonPatch>;
    fn apply_patch(&mut self, patch: JsonPatch, replica_id: K);
}

enum PathSegment {
    Key(String),      // Object key
    Index(usize),     // Array index
}

struct JsonPatch {
    path: Vec<PathSegment>,
    op: JsonOp,
}

enum JsonOp {
    Set(JsonCRDT),
    Delete,
    Increment(f64),
    Decrement(f64),
    TextInsert { pos: usize, text: String },
    TextDelete { pos: usize, len: usize },
    ArrayInsert { index: usize, value: JsonCRDT },
    ArrayDelete { index: usize },
    ArrayMove { from: usize, to: usize },
}
```

---

### 6.4 — Undo/Redo Support

**Goal:** Enable local undo/redo while preserving CRDT convergence.

**File:** `crates/mdcs-db/src/undo.rs`

```rust
/// Per-replica undo stack
struct UndoManager {
    replica_id: ReplicaId,
    undo_stack: Vec<UndoGroup>,
    redo_stack: Vec<UndoGroup>,
    capture_timeout: Duration,  // Group rapid edits
}

struct UndoGroup {
    operations: Vec<InverseOp>,
    timestamp: u64,
}

trait Undoable {
    fn undo(&mut self, manager: &mut UndoManager) -> bool;
    fn redo(&mut self, manager: &mut UndoManager) -> bool;
    fn can_undo(&self, manager: &UndoManager) -> bool;
    fn can_redo(&self, manager: &UndoManager) -> bool;
    
    // Capture current operation for undo
    fn begin_capture(&mut self, manager: &mut UndoManager);
    fn end_capture(&mut self, manager: &mut UndoManager);
}
```

---

### 6.5 — Use Case Examples

#### Google Docs Clone
```rust
// Document structure
let doc = json!({
    "title": Text("Untitled Document"),
    "body": RichText::new(),
    "comments": Map::new(),
    "metadata": {
        "created_at": timestamp,
        "authors": Set::new(),
        "word_count": Counter::new(),
    }
});

// Collaborative editing
doc.text_insert_path(&["body"], 0, "Hello ", replica_a);
doc.text_insert_path(&["body"], 6, "World!", replica_b);
// Result: "Hello World!" regardless of arrival order

// Formatting
doc.rich_text_at(&["body"]).add_mark(0, 5, Bold, replica_a);

// Presence
presence.update_cursor(CursorPosition { 
    doc_id: "doc_1", 
    path: "body", 
    position: 6 
});
```

#### Figma/Whiteboard Clone
```rust
// Canvas with objects
let canvas = json!({
    "objects": Map<ObjectId, {
        "type": "rectangle" | "ellipse" | "text" | "image",
        "x": Counter,  // Position as counters for smooth collab
        "y": Counter,
        "width": Register,
        "height": Register,
        "rotation": Register,
        "fill": Register,
        "z_index": Counter,  // Layer ordering
        "locked": Register<bool>,
    }>,
    "selection": Map<UserId, Set<ObjectId>>,  // Per-user selection
});

// Move object (using counters for position)
canvas.increment_path(&["objects", obj_id, "x"], delta_x, replica);
canvas.increment_path(&["objects", obj_id, "y"], delta_y, replica);
```

#### Trello/Kanban Clone
```rust
let board = json!({
    "columns": RGAList<{
        "id": String,
        "title": Text,
        "cards": RGAList<CardId>,
    }>,
    "cards": Map<CardId, {
        "title": Text,
        "description": RichText,
        "labels": Set<LabelId>,
        "assignees": Set<UserId>,
        "due_date": Register<Option<Timestamp>>,
        "checklist": RGAList<{
            "text": Text,
            "done": Register<bool>,
        }>,
        "comments": RGAList<Comment>,
    }>,
});

// Move card between columns
board.list_delete(&["columns", 0, "cards"], card_index, replica);
board.list_insert(&["columns", 1, "cards"], new_index, card_id, replica);
```

---

### 6.6 — Phase 6 Feature Summary

| Feature | Description | Use Cases |
|---------|-------------|-----------|
| **Decrements** | Full `increment`/`decrement` with amounts on `PNCounter` | Likes, inventory, scores |
| **RGAText** | Collaborative plain text (insert/delete at position) | Notes, code editors |
| **RichTextCRDT** | Formatted text with marks (bold, italic, links, comments) | Google Docs, Notion |
| **Presence** | Cursor tracking, selection sync, user status | Any real-time collab |
| **JsonCRDT** | Automerge-like arbitrary nested objects | Flexible schemas |
| **RGAList** | Ordered lists with insert/delete/move | Kanban, todos, playlists |
| **Undo/Redo** | Per-replica undo stacks | Any editing application |
| **Binary** | Raw bytes storage | Images, files, attachments |

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
