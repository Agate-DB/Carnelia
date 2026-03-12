---
name: carnelia-sdk
description: High-level Rust SDK for building collaborative, offline-first applications on the MDCS (Merkle-Delta CRDT Store). Use when working with CRDT documents, peer-to-peer sync, real-time presence/awareness, text/rich-text/JSON collaboration, network transport, or any type in the mdcs-sdk namespace.
metadata:
  tags: crdt, collaborative, sdk, real-time, offline-first, mdcs, delta, sync, presence, document, merkle, peer-to-peer
---

# Carnelia SDK (`mdcs-sdk`)

Ergonomic, async-first Rust SDK for building real-time collaborative applications on top of the MDCS (Merkle-Delta CRDT Store). Provides document management, peer-to-peer networking, synchronization, and presence awareness — all backed by formally convergent CRDTs.

**Internal crate dependencies:** `mdcs-core` (lattice primitives, CRDT types), `mdcs-delta` (delta buffer/interval logic), `mdcs-merkle` (Merkle-Clock DAG), `mdcs-compaction` (snapshot/GC), `mdcs-db` (RGAText, RichText, JsonCrdt, PresenceTracker).

## When to Use

Activate this skill when the user is:

- Building collaborative editing features (text, rich text, or JSON documents)
- Working with CRDT merge, delta, or lattice operations
- Managing peer-to-peer connections or network transports
- Implementing real-time presence, cursors, or user awareness
- Configuring sync managers or anti-entropy protocols
- Using any type exported from `mdcs_sdk` (`Client`, `Session`, `TextDoc`, `JsonDoc`, etc.)
- Writing tests with in-memory transport networks

---

## Architecture

The SDK is organized into seven modules with a clear layered responsibility model. Every module is re-exported at crate root and via a `prelude`.

### Module Map

| Module | Responsibility | Key Types |
|--------|---------------|-----------|
| `client` | Top-level entry point; manages sessions, generic over transport | `Client<T>`, `ClientConfig`, `ClientConfigBuilder` |
| `document` | CRDT document wrappers with event broadcasting | `TextDoc`, `RichTextDoc`, `JsonDoc`, `CollaborativeDoc` trait, `DocEvent` |
| `session` | Groups documents + awareness under a named collaborative session | `Session<T>`, `SessionEvent` |
| `network` | Abstract transport trait + in-memory implementation | `NetworkTransport` trait, `MemoryTransport`, `Message`, `Peer`, `PeerId` |
| `sync` | Synchronization engine with per-peer version tracking | `SyncManager<T>`, `SyncConfig`, `SyncConfigBuilder`, `SyncEvent` |
| `presence` | Real-time user awareness, cursors, selections | `Awareness`, `CursorInfo`, `UserPresenceInfo`, `AwarenessEvent` |
| `error` | SDK error types | `SdkError`, `Result<T>` |

### Concurrency Model

- **Synchronous shared state:** `parking_lot::RwLock` (sessions map, document maps, presence tracker)
- **Event subscriptions:** `tokio::sync::broadcast` — call `subscribe()` to get a new `Receiver<T>`
- **Network message passing:** `tokio::sync::mpsc` channels (100-capacity)
- **Document handles:** `Arc<RwLock<Doc>>` — shared, lock-protected concurrent access
- **Async runtime:** `tokio` with full features

---

## API Reference

### `ClientConfig`

Configuration for a `Client` instance.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `user_name` | `String` | `"Anonymous"` | Display name for presence |
| `auto_reconnect` | `bool` | `true` | Enable automatic reconnection |
| `max_reconnect_attempts` | `u32` | `5` | Max reconnection attempts |

`ClientConfig` also implements `Default` directly and can be used with struct update syntax:

```rust
ClientConfig {
    user_name: "Alice".to_string(),
    ..Default::default()
}
```

Or via `ClientConfigBuilder`:

```rust
ClientConfigBuilder::new()
    .user_name("Alice")
    .auto_reconnect(true)
    .max_reconnect_attempts(10)
    .build()
```

### `Client<T: NetworkTransport>`

Top-level SDK entry point. Generic over any `NetworkTransport`.

| Method | Signature | Notes |
|--------|-----------|-------|
| `new` | `(peer_id: PeerId, transport: Arc<T>, config: ClientConfig) -> Self` | Direct constructor |
| `new_with_memory_transport` | `(config: ClientConfig) -> Client<MemoryTransport>` | Convenience; generates peer ID from hex timestamp |
| `peer_id` | `(&self) -> &PeerId` | |
| `user_name` | `(&self) -> &str` | |
| `transport` | `(&self) -> &Arc<T>` | |
| `create_session` | `(&self, session_id: impl Into<String>) -> Arc<Session<T>>` | **Get-or-create** — returns existing `Arc` if session already exists |
| `get_session` | `(&self, session_id: &str) -> Option<Arc<Session<T>>>` | |
| `close_session` | `(&self, session_id: &str)` | Removes from local session map only |
| `session_ids` | `(&self) -> Vec<String>` | |
| `connect_peer` | `async (&self, peer_id: &PeerId) -> Result<()>` | Delegates to transport |
| `disconnect_peer` | `async (&self, peer_id: &PeerId) -> Result<()>` | |
| `connected_peers` | `async (&self) -> Vec<Peer>` | |

#### `client::quick` Module

| Function | Signature | Description |
|----------|-----------|-------------|
| `create_collaborative_clients` | `(user_names: &[&str]) -> Vec<Client<MemoryTransport>>` | Creates fully-connected in-memory clients using `create_network()` |

---

### `CollaborativeDoc` Trait

Unifying trait for all document types.

| Method | Signature | Description |
|--------|-----------|-------------|
| `id` | `(&self) -> &str` | Stable document identifier |
| `replica_id` | `(&self) -> &str` | Local replica identifier |
| `subscribe` | `(&self) -> broadcast::Receiver<DocEvent>` | Subscribe to change events |
| `take_pending_deltas` | `(&mut self) -> Vec<Vec<u8>>` | Drain accumulated deltas for sync |
| `apply_remote` | `(&mut self, delta: &[u8])` | Apply serialized remote delta |

### `DocEvent`

| Variant | Fields | Description |
|---------|--------|-------------|
| `Insert` | `position: usize, text: String` | Text inserted |
| `Delete` | `position: usize, length: usize` | Text deleted |
| `RemoteUpdate` | — | Remote changes applied |

### `TextDoc`

Wraps `mdcs_db::rga_text::RGAText`. Collaborative plain text with RGA (Replicated Growable Array) semantics.

| Method | Signature | Notes |
|--------|-----------|-------|
| `new` | `(id: impl Into<String>, replica_id: impl Into<String>) -> Self` | Creates broadcast channel (capacity 100) |
| `insert` | `(&mut self, position: usize, text: &str)` | Inserts via RGAText; emits `DocEvent::Insert` |
| `delete` | `(&mut self, position: usize, length: usize)` | Deletes via RGAText; emits `DocEvent::Delete` |
| `get_text` | `(&self) -> String` | Full text content |
| `len` | `(&self) -> usize` | |
| `is_empty` | `(&self) -> bool` | |
| `merge` | `(&mut self, other: &TextDoc)` | CRDT lattice join; emits `DocEvent::RemoteUpdate` |
| `clone_state` | `(&self) -> TextDoc` | Deep clone; clears pending deltas on the copy |

### `RichTextDoc`

Wraps `mdcs_db::rich_text::RichText`. Collaborative rich text with inline formatting marks.

All methods from `TextDoc` plus:

| Method | Signature | Notes |
|--------|-----------|-------|
| `format` | `(&mut self, start: usize, end: usize, mark: MarkType)` | Applies formatting mark via `add_mark`; **does not emit a `DocEvent`** (unlike `insert`/`delete`) |
| `unformat_by_id` | `(&mut self, mark_id: &mdcs_db::rich_text::MarkId)` | Removes a specific mark by ID; return value (`bool`) is discarded |
| `get_text` | `(&self) -> String` | Plain-text projection |
| `get_content` | `(&self) -> String` | Alias for `get_text` |

`MarkType` variants: `Bold`, `Italic`, `Underline`, `Strikethrough`, `Code`, `Link { url }`, `Comment { author, content }`, `Highlight { color }`, `Custom { name, value }`.

### `JsonDoc`

Wraps `mdcs_db::json_crdt::JsonCrdt`. Collaborative JSON document with dot-path access.

| Method | Signature | Notes |
|--------|-----------|-------|
| `new` | `(id: impl Into<String>, replica_id: impl Into<String>) -> Self` | |
| `set` | `(&mut self, path: &str, value: JsonValue)` | Parses dot-delimited path via `JsonPath::parse` |
| `get` | `(&self, path: &str) -> Option<JsonValue>` | |
| `delete` | `(&mut self, path: &str)` | |
| `root` | `(&self) -> serde_json::Value` | Full document as JSON |
| `keys` | `(&self) -> Vec<String>` | Top-level keys |
| `merge` | `(&mut self, other: &JsonDoc)` | CRDT lattice join |
| `clone_state` | `(&self) -> JsonDoc` | |

---

### `NetworkTransport` Trait

Async trait (`#[async_trait]`) requiring `Send + Sync + 'static`.

| Method | Signature |
|--------|-----------|
| `connect` | `async fn(&self, peer_id: &PeerId) -> Result<(), NetworkError>` |
| `disconnect` | `async fn(&self, peer_id: &PeerId) -> Result<(), NetworkError>` |
| `send` | `async fn(&self, peer_id: &PeerId, message: Message) -> Result<(), NetworkError>` |
| `broadcast` | `async fn(&self, message: Message) -> Result<(), NetworkError>` |
| `connected_peers` | `async fn(&self) -> Vec<Peer>` |
| `subscribe` | `fn(&self) -> mpsc::Receiver<(PeerId, Message)>` |

### `MemoryTransport`

In-memory transport for testing and simulation.

| Method | Signature | Notes |
|--------|-----------|-------|
| `new` | `(local_id: PeerId) -> Self` | Creates mpsc channel (capacity 100) |
| `local_id` | `(&self) -> &PeerId` | |
| `connect_to` | `(&self, other: &MemoryTransport)` | Bidirectional peer connection with cross-channels |

**`subscribe()` is single-use** — it consumes an internal `Option<Receiver>`. Calling it a second time will panic.

#### `create_network(count: usize) -> Vec<MemoryTransport>`

Creates `count` fully-interconnected transports (all-pairs `connect_to`).

### `Message`

Wire protocol message enum (`Serialize`, `Deserialize`).

| Variant | Fields | Description |
|---------|--------|-------------|
| `Hello` | `replica_id: String, user_name: String` | Handshake on session connect |
| `SyncRequest` | `document_id: String, version: u64` | Request sync for a document |
| `SyncResponse` | `document_id: String, deltas: Vec<Vec<u8>>, version: u64` | Delta batch response |
| `Update` | `document_id: String, delta: Vec<u8>, version: u64` | Incremental update broadcast |
| `Presence` | `user_id: String, document_id: String, cursor_pos: Option<usize>` | Presence/cursor update |
| `Ack` | `message_id: u64` | Acknowledgment |
| `Ping` | — | Keepalive |
| `Pong` | — | Keepalive response |

### `Peer` & `PeerId`

- `PeerId` — newtype around `String` with a **public inner field** `pub struct PeerId(pub String)`. Access the raw string via `.0`. Implements `Display`, `Serialize`, `Deserialize`.
- `Peer` — `{ id: PeerId, name: String, state: PeerState }`
- `PeerState` — `Disconnected | Connecting | Connected`

### `NetworkError`

| Variant | Description |
|---------|-------------|
| `ConnectionFailed(String)` | |
| `PeerNotFound(String)` | |
| `SendFailed(String)` | |
| `Disconnected` | |

---

### `Session<T: NetworkTransport>`

Groups documents and peers under a named collaborative session. Owns an `Awareness` instance.

| Method | Signature | Notes |
|--------|-----------|-------|
| `new` | `(session_id, local_peer_id, user_name, transport: Arc<T>) -> Self` | Creates `Awareness` using `local_peer_id.0` (the inner string) as the awareness user_id. Most callers should use `Client::create_session` instead. |
| `session_id` | `(&self) -> &str` | |
| `local_peer_id` | `(&self) -> &PeerId` | |
| `user_name` | `(&self) -> &str` | |
| `awareness` | `(&self) -> &Arc<Awareness>` | |
| `subscribe` | `(&self) -> broadcast::Receiver<SessionEvent>` | |
| `connect` | `async (&self) -> Result<()>` | Broadcasts `Message::Hello`; emits `SessionEvent::Connected` |
| `disconnect` | `async (&self) -> Result<()>` | Emits `SessionEvent::Disconnected` (local only) |
| `open_text_doc` | `(&self, document_id: impl Into<String>) -> Arc<RwLock<TextDoc>>` | **Get-or-create**; emits `DocumentOpened` |
| `open_rich_text_doc` | `(&self, document_id: impl Into<String>) -> Arc<RwLock<RichTextDoc>>` | **Get-or-create**; emits `DocumentOpened` |
| `open_json_doc` | `(&self, document_id: impl Into<String>) -> Arc<RwLock<JsonDoc>>` | **Get-or-create**; emits `DocumentOpened` |
| `close_doc` | `(&self, document_id: &str)` | Removes from all 3 doc maps; emits `DocumentClosed` |
| `open_documents` | `(&self) -> Vec<String>` | Union of all doc map keys |
| `peers` | `async (&self) -> Vec<Peer>` | Delegates to transport |

### `SessionEvent`

| Variant | Fields |
|---------|--------|
| `PeerJoined` | `{ peer_id: PeerId, user_name: String }` |
| `PeerLeft` | `{ peer_id: PeerId }` |
| `DocumentOpened` | `{ document_id: String }` |
| `DocumentClosed` | `{ document_id: String }` |
| `Connected` | — |
| `Disconnected` | — |

---

### `SyncConfig`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `sync_interval_ms` | `u64` | `1000` | How often to send sync requests |
| `presence_interval_ms` | `u64` | `500` | How often to send presence updates |
| `sync_timeout_ms` | `u64` | `5000` | Timeout for sync requests |
| `max_batch_size` | `usize` | `100` | Max deltas per batch |
| `auto_sync` | `bool` | `true` | Enable automatic background sync |

Built via `SyncConfigBuilder` (same builder pattern as `ClientConfigBuilder`).

### `SyncManager<T: NetworkTransport>`

| Method | Signature | Notes |
|--------|-----------|-------|
| `new` | `(transport: Arc<T>, config: SyncConfig) -> Self` | |
| `config` | `(&self) -> &SyncConfig` | |
| `broadcast_update` | `async (&mut self, document_id: &str, delta: Vec<u8>, version: u64) -> Result<()>` | Sends `Message::Update` to all peers |
| `request_sync` | `async (&mut self, peer_id: &PeerId, document_id: &str, version: u64) -> Result<()>` | Sends `Message::SyncRequest` to specific peer |
| `update_peer_state` | `(&mut self, peer_id: &PeerId, document_id: &str, version: u64)` | Tracks per-peer, per-document version + last sync time |
| `get_peer_state` | `(&self, peer_id: &PeerId) -> Option<&PeerSyncState>` | |

### `SyncEvent`

| Variant | Fields |
|---------|--------|
| `SyncStarted` | `PeerId` |
| `SyncCompleted` | `PeerId` |
| `ReceivedUpdate` | `{ peer_id: PeerId, document_id: String }` |
| `SentUpdate` | `{ peer_id: PeerId, document_id: String }` |
| `SyncError` | `{ peer_id: PeerId, error: String }` |

---

### `Awareness`

Wraps `mdcs_db::presence::PresenceTracker` behind `Arc<RwLock<...>>`. Tracks cursor positions, selections, and user status per document.

| Method | Signature | Notes |
|--------|-----------|-------|
| `new` | `(local_user_id: impl Into<String>, local_user_name: impl Into<String>) -> Self` | Default color: `"#0066cc"` |
| `local_user_id` | `(&self) -> &str` | |
| `local_user_name` | `(&self) -> &str` | |
| `set_cursor` | `(&self, document_id: &str, position: usize)` | Sets `Cursor::at(position)`; emits `AwarenessEvent::CursorMoved` |
| `set_selection` | `(&self, document_id: &str, start: usize, end: usize)` | Sets `Cursor::with_selection`; emits `CursorMoved` |
| `set_status` | `(&self, status: UserStatus)` | Updates status in tracker |
| `get_users` | `(&self) -> Vec<UserPresenceInfo>` | Snapshot of all users with cursor maps |
| `get_cursors` | `(&self, document_id: &str) -> Vec<CursorInfo>` | Cursors for a specific document |
| `get_local_color` | `(&self) -> &str` | |
| `subscribe` | `(&self) -> broadcast::Receiver<AwarenessEvent>` | |
| `cleanup_stale` | `(&self)` | Delegates to `PresenceTracker::cleanup_stale()` |

### `AwarenessEvent`

| Variant | Payload |
|---------|---------|
| `UserUpdated` | `UserPresenceInfo` |
| `UserOffline` | `String` (user_id) |
| `CursorMoved` | `CursorInfo` |

### `CursorInfo`

Fields: `user_id`, `user_name`, `document_id`, `position: usize`, `selection_start: Option<usize>`, `selection_end: Option<usize>`, `color: String`.

### `UserPresenceInfo`

Fields: `user_id`, `name`, `status: UserStatus`, `color`, `cursors: HashMap<String, CursorInfo>`.

---

### `SdkError`

| Variant | Display |
|---------|---------|
| `DocumentNotFound(String)` | `"Document not found: {}"` |
| `PeerNotFound(String)` | `"Peer not found: {}"` |
| `ConnectionFailed(String)` | `"Connection failed: {}"` |
| `SyncError(String)` | `"Sync error: {}"` |
| `NetworkError(String)` | `"Network error: {}"` |
| `SerializationError(String)` | `"Serialization error: {}"` |
| `Internal(String)` | `"Internal error: {}"` |

`pub type Result<T> = std::result::Result<T, SdkError>;`

---

## Re-exports

### Crate Root

All public types from every module are re-exported at crate root (`mdcs_sdk::Client`, etc.).

### From `mdcs-db`

The following types are re-exported at crate root for convenience:

`JsonPath`, `JsonValue`, `Cursor`, `UserId`, `UserInfo`, `UserStatus`, `MarkType`

### Prelude (`mdcs_sdk::prelude`)

`Client`, `ClientConfig`, `CollaborativeDoc`, `JsonDoc`, `RichTextDoc`, `TextDoc`, `SdkError`, `NetworkTransport`, `Peer`, `PeerId`, `Awareness`, `CursorInfo`, `Session`, `SyncConfig`, `SyncManager`

---

## Key Patterns

### Get-or-Create Semantics

Both `Client::create_session` and `Session::open_*_doc` use get-or-create: if a session or document with the given ID already exists, the existing `Arc` handle is returned. No duplicate instances are created.

### Builder Pattern

`ClientConfig` and `SyncConfig` are constructed via their respective builders (`ClientConfigBuilder`, `SyncConfigBuilder`), which implement `Default` for zero-config usage.

### Event Subscription

All event-emitting components (`TextDoc`, `RichTextDoc`, `JsonDoc`, `Session`, `Awareness`) expose a `subscribe()` method returning a `tokio::sync::broadcast::Receiver<T>`. Each call returns a new independent receiver. Events are fire-and-forget — if no receivers are active, events are silently dropped.

### Delta Protocol

The SDK follows the delta accumulation pattern from the underlying `mdcs-db` crate:

1. **Mutate** — call `insert()`, `delete()`, `set()`, etc. on a document
2. **Drain** — call `take_pending_deltas()` to extract accumulated deltas as `Vec<Vec<u8>>`
3. **Transmit** — send deltas to peers via `SyncManager::broadcast_update()`
4. **Integrate** — receiving peers call `apply_remote(delta)` to merge

### CRDT Merge via Lattice Join

All `merge()` methods on document types delegate to the `Lattice::join` operation from `mdcs-core`, which is commutative, associative, and idempotent. This guarantees convergence regardless of merge order.

---

## Underlying CRDT Primitives

The SDK wraps types built on two core traits from `mdcs-core`:

### `Lattice` Trait

```rust
pub trait Lattice: Clone + PartialEq {
    fn bottom() -> Self;                                    // identity element
    fn join(&self, other: &Self) -> Self;                   // least upper bound (⊔)
    fn partial_cmp_lattice(&self, other: &Self) -> Option<Ordering>;
    fn leq(&self, other: &Self) -> bool;                    // partial order ≤
    fn join_assign(&mut self, other: &Self);                // self = self ⊔ other
}
```

### `DeltaCRDT` Trait

```rust
pub trait DeltaCRDT: Lattice {
    type Delta: Lattice;
    fn split_delta(&mut self) -> Option<Self::Delta>;       // extract pending delta
    fn apply_delta(&mut self, delta: &Self::Delta);         // integrate remote delta
}
```

All `mdcs-db` document types (`RGAText`, `RichText`, `JsonCrdt`) implement these traits, ensuring formal convergence guarantees (SEC — Strong Eventual Consistency).

---

## Known Limitations

These are current implementation gaps identified in the source:

1. **`apply_remote()` is a stub** — On all document types (`TextDoc`, `RichTextDoc`, `JsonDoc`), `apply_remote()` emits a `DocEvent::RemoteUpdate` event but does **not** deserialize or apply the delta bytes. Use `merge()` with a cloned document state for actual CRDT integration.

2. **`uuid_simple()` is not a proper UUID** — `Client::new_with_memory_transport` generates peer IDs as `"peer-{hex_nanoseconds}"` (e.g. `"peer-17c3a9f2b1d"`), not RFC 4122 UUIDs. Collisions are possible under high-frequency creation. By contrast, `create_network(n)` uses sequential IDs `"peer-0"`, `"peer-1"`, etc.

3. **`MemoryTransport::subscribe()` is single-use** — The receiver is stored in an `Option` and consumed on first call. A second call will panic.

4. **`MemoryTransport::broadcast()` swallows errors** — Individual per-peer send failures during broadcast are silently ignored; the method returns `Ok(())` regardless.

5. **No background sync loop** — `SyncManager` provides primitives (`broadcast_update`, `request_sync`) but does not spawn a background task. The `auto_sync` config flag is defined but not yet wired to an automatic sync loop.