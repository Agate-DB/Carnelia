# MDCS SDK

A high-level SDK for building collaborative applications with the Merkle-Delta CRDT Store.

## Overview

The MDCS SDK provides easy-to-use abstractions for:

- **Client Management**: Create and manage database clients
- **Sessions**: Organize collaborative editing sessions
- **Documents**: Work with text, rich text, and JSON documents
- **Presence**: Track user cursors, selections, and status
- **Network**: Pluggable network transport for peer-to-peer communication
- **Sync**: Automatic delta synchronization between peers

## Quick Start

```rust
use mdcs_sdk::{Client, quick};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create two connected collaborative clients
    let (client_a, client_b) = quick::create_collaborative_clients();
    
    // Create sessions
    let session_a = client_a.create_session("room-1").await?;
    let session_b = client_b.create_session("room-1").await?;
    
    // Open text documents
    let doc_a = session_a.open_text_doc("shared-doc")?;
    let doc_b = session_b.open_text_doc("shared-doc")?;
    
    // User A types
    doc_a.insert(0, "Hello ")?;
    
    // User B types (concurrent edit)
    doc_b.insert(0, "World!")?;
    
    // Both documents will converge to the same state
    // via CRDT merge semantics
    
    Ok(())
}
```

## Architecture

### Modules

```
mdcs-sdk/
├── client.rs      # Client entry point and factory
├── session.rs     # Session management
├── document.rs    # Document type wrappers (Text, RichText, JSON)
├── presence.rs    # User awareness and presence
├── network.rs     # Network transport abstraction
├── sync.rs        # Synchronization configuration
└── error.rs       # SDK error types
```

### Document Types

#### TextDoc
Simple collaborative text editing:

```rust
let doc = session.open_text_doc("notes")?;
doc.insert(0, "Hello")?;
doc.delete(0, 5)?;
let content = doc.get_text();
```

#### RichTextDoc
Text with formatting support:

```rust
use mdcs_sdk::MarkType;

let doc = session.open_rich_text_doc("article")?;
doc.insert(0, "Hello World")?;
doc.add_mark(0, 5, MarkType::Bold)?;     // "Hello" is bold
doc.add_mark(6, 11, MarkType::Italic)?;  // "World" is italic
```

#### JsonDoc
Structured JSON data:

```rust
use mdcs_sdk::JsonValue;

let doc = session.open_json_doc("config")?;
doc.set("theme", JsonValue::String("dark".into()))?;
doc.set("fontSize", JsonValue::Float(14.0))?;
doc.set("features.enabled", JsonValue::Bool(true))?;

if let Some(value) = doc.get("theme") {
    println!("Theme: {:?}", value);
}
```

### Presence System

Track user awareness across sessions:

```rust
// Set user info
session.awareness().set_user_name("Alice");
session.awareness().set_status(UserStatus::Online);

// Set cursor position
session.awareness().set_cursor(CursorInfo {
    position: 42,
    document_id: "shared-doc".to_string(),
});

// Set selection
session.awareness().set_selection(0, 10, "doc-1");

// Get all connected users
for user in session.awareness().get_users() {
    println!("{}: {:?}", user.user_id, user.status);
}
```

### Network Transport

The SDK uses a pluggable network transport:

```rust
use mdcs_sdk::network::{NetworkTransport, MemoryTransport, PeerId, Message};

// Create in-memory transport (for testing/local simulation)
let transport = MemoryTransport::new(PeerId::new("client-1"));

// Connect to another peer
transport.connect_to(&other_transport);

// Send messages
transport.send(&peer_id, Message::Update {
    document_id: "doc".into(),
    delta: vec![1, 2, 3],
    version: 1,
}).await?;

// Receive messages
let mut rx = transport.subscribe();
while let Some((from, msg)) = rx.recv().await {
    println!("Message from {}: {:?}", from, msg);
}
```

## Examples

Run examples with:

```bash
# Multi-user text collaboration
cargo run --example collaborative_text

# Rich text with formatting
cargo run --example rich_text_collab

# JSON document editing
cargo run --example json_collab

# Presence and awareness
cargo run --example presence_demo

# Network layer simulation
cargo run --example network_simulation

# Offline-first sync demo
cargo run --example offline_sync
```

## Configuration

### Sync Configuration

```rust
use mdcs_sdk::SyncConfig;

let config = SyncConfig::builder()
    .sync_interval(Duration::from_millis(100))
    .batch_size(50)
    .retry_count(3)
    .build();
```

## Error Handling

```rust
use mdcs_sdk::SdkError;

match result {
    Err(SdkError::DocumentNotFound(id)) => {
        println!("Document {} not found", id);
    }
    Err(SdkError::PeerNotFound(id)) => {
        println!("Peer {} disconnected", id);
    }
    Err(SdkError::SyncError(msg)) => {
        println!("Sync failed: {}", msg);
    }
    _ => {}
}
```

## Integration with mdcs-db

The SDK builds on top of the lower-level `mdcs-db` crate:

```rust
// Direct database access when needed
use mdcs_db::MdcsDb;

let db = MdcsDb::new();
// Use db directly for low-level operations
```

Re-exported types from mdcs-db:
- `JsonValue` - JSON value type
- `MarkType` - Text formatting marks
- `UserStatus` - Presence status
- `Cursor` - Cursor position

## Testing

```bash
# Run SDK tests
cargo test -p mdcs-sdk

# Run with output
cargo test -p mdcs-sdk -- --nocapture
```

## License

MIT
