# MDCS SDK

A high-level Rust SDK for building collaborative, local-first applications on top of the Merkle-Delta CRDT Store.

---

## Table of contents

- [What you get](#what-you-get)
- [Installation](#installation)
- [Quickstarts](#quickstarts)
  - [Collaborative text in 60 seconds](#collaborative-text-in-60-seconds)
  - [JSON document quickstart](#json-document-quickstart)
  - [Presence quickstart](#presence-quickstart)
  - [Network + sync quickstart](#network--sync-quickstart)
- [Mental model](#mental-model)
- [API reference](#api-reference)
  - [1) Client API (`client.rs`)](#1-client-api-clientrs)
  - [2) Session API (`session.rs`)](#2-session-api-sessionrs)
  - [3) Document API (`document.rs`)](#3-document-api-documentrs)
  - [4) Presence API (`presence.rs`)](#4-presence-api-presencers)
  - [5) Network API (`network.rs`)](#5-network-api-networkrs)
  - [6) Sync API (`sync.rs`)](#6-sync-api-syncrs)
  - [7) Error API (`error.rs`)](#7-error-api-errorrs)
- [End-to-end usage pattern](#end-to-end-usage-pattern)
- [Examples](#examples)
- [Best practices](#best-practices)
- [License](#license)

---

## What you get

The SDK exposes a layered API:

- **Client**: process-level entry point, peer identity, session registry
- **Session**: collaboration scope for peers + documents + awareness
- **Documents**: CRDT-backed `TextDoc`, `RichTextDoc`, and `JsonDoc`
- **Presence**: cursor, selection, and user status tracking
- **Network**: pluggable transport via `NetworkTransport`
- **Sync**: transport message helpers + per-peer sync state
- **Errors**: typed `SdkError` and `Result<T>`

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
mdcs-sdk = { path = "../crates/mdcs-sdk" }
```

---

## Quickstarts

### Collaborative text in 60 seconds

```rust
use mdcs_sdk::client::quick::create_collaborative_clients;

fn main() {
    // 1) Create connected in-memory clients
    let clients = create_collaborative_clients(&["Alice", "Bob"]);
    let alice = &clients[0];
    let bob = &clients[1];

    // 2) Join the same session and open the same document id
    let session_a = alice.create_session("room-1");
    let session_b = bob.create_session("room-1");

    let doc_a = session_a.open_text_doc("shared-doc");
    let doc_b = session_b.open_text_doc("shared-doc");

    // 3) Apply local edits
    doc_a.write().insert(0, "Hello ");
    doc_b.write().insert(0, "World!");

    // 4) Exchange state and merge (demo sync)
    let state_a = doc_a.read().clone_state();
    let state_b = doc_b.read().clone_state();

    doc_a.write().merge(&state_b);
    doc_b.write().merge(&state_a);

    // 5) Converged result
    println!("A: {}", doc_a.read().get_text());
    println!("B: {}", doc_b.read().get_text());
}
```

### JSON document quickstart

```rust
use mdcs_sdk::client::quick::create_collaborative_clients;
use mdcs_sdk::JsonValue;

fn main() {
  let clients = create_collaborative_clients(&["Alice", "Bob"]);
  let session_a = clients[0].create_session("project-room");
  let session_b = clients[1].create_session("project-room");

  let doc_a = session_a.open_json_doc("project-config");
  let doc_b = session_b.open_json_doc("project-config");

  // Local updates on Alice
  {
    let mut d = doc_a.write();
    d.set("project.name", JsonValue::String("Carnelia".into()));
    d.set("project.version", JsonValue::Float(1.0));
  }

  // Sync by exchanging state snapshots (demo flow)
  let state_a = doc_a.read().clone_state();
  doc_b.write().merge(&state_a);

  // Output
  println!("Bob sees: {}", doc_b.read().root());
}
```

### Presence quickstart

```rust
use mdcs_sdk::client::quick::create_collaborative_clients;
use mdcs_sdk::UserStatus;

fn main() {
  let clients = create_collaborative_clients(&["Alice"]);
  let session = clients[0].create_session("presence-room");

  // Inputs
  session.awareness().set_status(UserStatus::Online);
  session.awareness().set_cursor("shared-doc", 24);
  session.awareness().set_selection("shared-doc", 10, 24);

  // Output
  for user in session.awareness().get_users() {
    println!("{} ({:?})", user.name, user.status);
  }
}
```

### Network + sync quickstart

```rust
use mdcs_sdk::network::{MemoryTransport, PeerId};
use mdcs_sdk::{Client, ClientConfig, SyncConfig, SyncManager};
use std::sync::Arc;

#[tokio::main]
async fn main() {
  let transport = Arc::new(MemoryTransport::new(PeerId::new("peer-a")));
  let client = Client::new(
    PeerId::new("peer-a"),
    transport.clone(),
    ClientConfig::default(),
  );

  // Create sync manager
  let mut sync = SyncManager::new(transport, SyncConfig::default());

  // Example output type: Result<(), SdkError>
  let _ = sync
    .broadcast_update("doc-1", vec![1, 2, 3], 1)
    .await;

  println!("Client peer: {}", client.peer_id());
}
```

---

## Mental model

Use this sequence in production:

1. Create `Client<T: NetworkTransport>`
2. Create/get a `Session`
3. Open document handles from the session
4. Apply local edits through write locks
5. Ship updates over your transport
6. Merge remote state (or apply remote deltas)
7. Subscribe to session/document/awareness events for UI reactivity

---

## API reference

### 1) Client API (`client.rs`)

`Client` owns local identity, transport, and local session cache.

#### `ClientConfig`

| Field | Type | Input | Output / Effect |
|---|---|---|---|
| `user_name` | `String` | display name | exposed in session presence handshake |
| `auto_reconnect` | `bool` | reconnect policy | currently configuration metadata |
| `max_reconnect_attempts` | `u32` | retry cap | currently configuration metadata |

#### `ClientConfigBuilder`

- `new() -> ClientConfigBuilder`
- `user_name(name) -> ClientConfigBuilder`
- `auto_reconnect(enabled) -> ClientConfigBuilder`
- `max_reconnect_attempts(attempts) -> ClientConfigBuilder`
- `build() -> ClientConfig`

#### `Client<MemoryTransport>`

- `new_with_memory_transport(config) -> Client<MemoryTransport>`
  - **Input**: `ClientConfig`
  - **Output**: client with generated `PeerId`
  - **Use when**: tests, demos, local simulations

#### `Client<T: NetworkTransport>` core methods

- `new(peer_id, transport, config) -> Client<T>`
- `peer_id() -> &PeerId`
- `user_name() -> &str`
- `transport() -> &Arc<T>`
- `create_session(session_id) -> Arc<Session<T>>`
  - idempotent per `session_id` (returns existing local instance)
- `get_session(session_id) -> Option<Arc<Session<T>>>`
- `close_session(session_id)`
- `session_ids() -> Vec<String>`
- `connect_peer(peer_id) -> Result<(), SdkError>`
- `disconnect_peer(peer_id) -> Result<(), SdkError>`
- `connected_peers() -> Vec<Peer>`

#### Quick helper

- `quick::create_collaborative_clients(user_names: &[&str]) -> Vec<Client<MemoryTransport>>`
  - **Input**: ordered user names
  - **Output**: fully connected in-memory clients in same order

---

### 2) Session API (`session.rs`)

`Session` groups collaboration by `session_id` and manages opened docs + awareness.

#### Lifecycle and metadata

- `new(session_id, local_peer_id, user_name, transport) -> Session<T>`
- `session_id() -> &str`
- `local_peer_id() -> &PeerId`
- `user_name() -> &str`
- `awareness() -> &Arc<Awareness>`
- `subscribe() -> broadcast::Receiver<SessionEvent>`

#### Connectivity

- `connect().await -> Result<(), SdkError>`
  - broadcasts `Message::Hello { replica_id, user_name }`
- `disconnect().await -> Result<(), SdkError>`

#### Document management

- `open_text_doc(document_id) -> Arc<RwLock<TextDoc>>`
- `open_rich_text_doc(document_id) -> Arc<RwLock<RichTextDoc>>`
- `open_json_doc(document_id) -> Arc<RwLock<JsonDoc>>`
- `close_doc(document_id)`
- `open_documents() -> Vec<String>`
- `peers().await -> Vec<Peer>`

#### `SessionEvent`

- `PeerJoined { peer_id, user_name }`
- `PeerLeft { peer_id }`
- `DocumentOpened { document_id }`
- `DocumentClosed { document_id }`
- `Connected`
- `Disconnected`

---

### 3) Document API (`document.rs`)

All document handles are returned as `Arc<RwLock<_>>`.

Use:

- `doc.write()` for mutating calls
- `doc.read()` for read-only calls

#### 3.1 `TextDoc`

- `new(id, replica_id) -> TextDoc`
- `insert(position, text)`
- `delete(position, length)`
- `get_text() -> String`
- `len() -> usize`
- `is_empty() -> bool`
- `merge(other: &TextDoc)`
- `clone_state() -> TextDoc`

**Inputs / outputs**

- **Insert input**: zero-based `position`, UTF-8 string slice
- **Delete input**: zero-based `position`, `length`
- **Merge output**: in-place convergent state update

#### 3.2 `RichTextDoc`

- `new(id, replica_id) -> RichTextDoc`
- `insert(position, text)`
- `delete(position, length)`
- `format(start, end, mark: MarkType)`
- `unformat_by_id(mark_id)`
- `get_text() -> String`
- `get_content() -> String` (plain-text rendering)
- `len() -> usize`
- `is_empty() -> bool`
- `merge(other: &RichTextDoc)`
- `clone_state() -> RichTextDoc`

#### 3.3 `JsonDoc`

- `new(id, replica_id) -> JsonDoc`
- `set(path, value: JsonValue)`
- `get(path) -> Option<JsonValue>`
- `delete(path)`
- `root() -> serde_json::Value`
- `keys() -> Vec<String>`
- `merge(other: &JsonDoc)`
- `clone_state() -> JsonDoc`

**Path format**

- Dot-path strings (example: `"profile.name"`, `"settings.theme"`)

#### `DocEvent`

- `Insert { position, text }`
- `Delete { position, length }`
- `RemoteUpdate`

#### `CollaborativeDoc` trait

- `id() -> &str`
- `replica_id() -> &str`
- `subscribe() -> broadcast::Receiver<DocEvent>`
- `take_pending_deltas() -> Vec<Vec<u8>>`
- `apply_remote(delta: &[u8])`

---

### 4) Presence API (`presence.rs`)

Presence is managed per session via `session.awareness()`.

#### `Awareness`

- `new(local_user_id, local_user_name) -> Awareness`
- `local_user_id() -> &str`
- `local_user_name() -> &str`
- `set_cursor(document_id, position)`
- `set_selection(document_id, start, end)`
- `set_status(status: UserStatus)`
- `get_users() -> Vec<UserPresenceInfo>`
- `get_cursors(document_id) -> Vec<CursorInfo>`
- `get_local_color() -> &str`
- `subscribe() -> broadcast::Receiver<AwarenessEvent>`
- `cleanup_stale()`

#### Awareness data models

- `CursorInfo`
  - `user_id`, `user_name`, `document_id`, `position`, `selection_start`, `selection_end`, `color`
- `UserPresenceInfo`
  - `user_id`, `name`, `status`, `color`, `cursors`

#### `AwarenessEvent`

- `UserUpdated(UserPresenceInfo)`
- `UserOffline(String)`
- `CursorMoved(CursorInfo)`

---

### 5) Network API (`network.rs`)

`NetworkTransport` is the protocol boundary between the SDK and your networking layer.

#### `NetworkTransport` trait

- `connect(peer_id).await -> Result<(), NetworkError>`
- `disconnect(peer_id).await -> Result<(), NetworkError>`
- `send(peer_id, message).await -> Result<(), NetworkError>`
- `broadcast(message).await -> Result<(), NetworkError>`
- `connected_peers().await -> Vec<Peer>`
- `subscribe() -> mpsc::Receiver<(PeerId, Message)>`

#### Built-in transport: `MemoryTransport`

- `new(local_id: PeerId) -> MemoryTransport`
- `local_id() -> &PeerId`
- `connect_to(other: &MemoryTransport)`

#### Message envelope: `Message`

- `Hello { replica_id, user_name }`
- `SyncRequest { document_id, version }`
- `SyncResponse { document_id, deltas, version }`
- `Update { document_id, delta, version }`
- `Presence { user_id, document_id, cursor_pos }`
- `Ack { message_id }`
- `Ping`
- `Pong`

#### Helpers

- `create_network(count: usize) -> Vec<MemoryTransport>`
  - Returns a fully connected in-memory mesh

---

### 6) Sync API (`sync.rs`)

`SyncManager` helps send sync/update messages and track peer sync metadata.

#### `SyncConfig`

| Field | Type | Meaning |
|---|---|---|
| `sync_interval_ms` | `u64` | periodic sync frequency |
| `presence_interval_ms` | `u64` | periodic presence broadcast frequency |
| `sync_timeout_ms` | `u64` | timeout budget per sync request |
| `max_batch_size` | `usize` | max deltas per transfer |
| `auto_sync` | `bool` | enables background sync policy |

#### `SyncConfigBuilder`

- `new() -> SyncConfigBuilder`
- `sync_interval(ms) -> SyncConfigBuilder`
- `presence_interval(ms) -> SyncConfigBuilder`
- `sync_timeout(ms) -> SyncConfigBuilder`
- `max_batch_size(size) -> SyncConfigBuilder`
- `auto_sync(enabled) -> SyncConfigBuilder`
- `build() -> SyncConfig`

#### `SyncManager<T: NetworkTransport>`

- `new(transport, config) -> SyncManager<T>`
- `config() -> &SyncConfig`
- `broadcast_update(document_id, delta, version).await -> Result<(), SdkError>`
- `request_sync(peer_id, document_id, version).await -> Result<(), SdkError>`
- `update_peer_state(peer_id, document_id, version)`
- `get_peer_state(peer_id) -> Option<&PeerSyncState>`

#### `PeerSyncState`

- `document_versions: HashMap<String, u64>`
- `last_sync: Option<Instant>`

---

### 7) Error API (`error.rs`)

#### `SdkError`

- `DocumentNotFound(String)`
- `PeerNotFound(String)`
- `ConnectionFailed(String)`
- `SyncError(String)`
- `NetworkError(String)`
- `SerializationError(String)`
- `Internal(String)`

#### Alias

- `type Result<T> = std::result::Result<T, SdkError>`

---

## End-to-end usage pattern

```rust
use mdcs_sdk::{
    client::{Client, ClientConfig, ClientConfigBuilder},
    network::{MemoryTransport, PeerId},
    JsonValue,
};
use std::sync::Arc;

fn build_client(name: &str, peer: &str) -> Client<MemoryTransport> {
    let config = ClientConfigBuilder::new()
        .user_name(name)
        .auto_reconnect(true)
        .max_reconnect_attempts(5)
        .build();

    Client::new(
        PeerId::new(peer),
        Arc::new(MemoryTransport::new(PeerId::new(peer))),
        config,
    )
}

fn main() {
    let client = build_client("Alice", "peer-a");
    let session = client.create_session("workspace-1");

    // Text
    let text = session.open_text_doc("notes");
    text.write().insert(0, "MDCS SDK");

    // JSON
    let json = session.open_json_doc("settings");
    json.write().set("theme", JsonValue::String("dark".into()));

    // Presence
    session.awareness().set_cursor("notes", 4);
}
```

---

## Examples

Run from repository root:

```bash
cargo run --example collaborative_text
cargo run --example rich_text_collab
cargo run --example json_collab
cargo run --example presence_demo
cargo run --example network_simulation
cargo run --example offline_sync
cargo run --example plain_test
```

---

## Best practices

- Use stable `session_id` / `document_id` naming across peers.
- Keep network transport idempotent and resilient to retries.
- Treat merge/apply operations as potentially repeated (CRDT-safe path).
- Subscribe to events for UI updates instead of polling document state.
- Use `clone_state()` + `merge()` in demos; use delta transport in production paths.

---

## License

MIT
