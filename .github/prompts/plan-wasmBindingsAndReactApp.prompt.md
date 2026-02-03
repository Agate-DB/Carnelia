# Building a Multi-Tenant Document Editor with Carnelia

## Architecture Decision: Rust SDK → TypeScript

You have two main options to connect your React app to the Carnelia CRDT backend:

### Option 1: WebAssembly (WASM) - Recommended ✅

Compile the Rust SDK to WASM and use it directly in the browser. This gives you:
- **Full CRDT logic in the browser** - offline-first by default
- **No server required for local operations** - only for sync
- **Type safety** - generate TypeScript bindings from Rust

### Option 2: HTTP/WebSocket Server

Run a Rust server that exposes the SDK via API. This is simpler but:
- Requires always-online connection
- Adds latency to every operation
- Loses offline-first benefits

**Recommendation: Option 1 (WASM)** since CRDTs are designed for offline-first, and having the logic in the browser is the natural fit.

---

## Implementation Structure

### Crate Structure

```
crates/
└── mdcs-wasm/           # WASM bindings crate
    ├── Cargo.toml
    ├── README.md
    └── src/
        └── lib.rs
```

### React App Structure

```
apps/carnelia-docs-sharing/
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/
│   │   ├── Editor.tsx
│   │   ├── RemoteCursors.tsx
│   │   ├── Toolbar.tsx
│   │   └── SyncStatus.tsx
│   ├── hooks/
│   │   └── useCollaborativeDocument.ts
│   ├── lib/
│   │   └── carnelia-client.ts
│   └── wasm/              # Built WASM output goes here
│       ├── mdcs_wasm.js
│       ├── mdcs_wasm.d.ts
│       └── mdcs_wasm_bg.wasm
├── public/
├── index.html
├── package.json
└── vite.config.ts
```

---

## Step-by-Step Implementation

### Step 1: WASM Bindings Crate

**`crates/mdcs-wasm/Cargo.toml`:**
```toml
[package]
name = "mdcs-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
mdcs-db = { path = "../mdcs-db" }
mdcs-core = { path = "../mdcs-core" }
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
serde_json = "1.0"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["console"] }
getrandom = { version = "0.2", features = ["js"] }

[dev-dependencies]
wasm-bindgen-test = "0.3"

[profile.release]
opt-level = "s"
lto = true
```

**Key exports from `mdcs-wasm`:**
- `CollaborativeDocument` - Main document class with insert/delete/format operations
- `UserPresence` - Cursor and selection tracking
- `DocumentDelta` - Serializable deltas for sync

### Step 2: Build WASM Package

```bash
# Install wasm-pack
cargo install wasm-pack

# Build for web target
cd crates/mdcs-wasm
wasm-pack build --target web --out-dir ../../apps/carnelia-docs-sharing/src/wasm
```

### Step 3: TypeScript Integration

**`src/lib/carnelia-client.ts`:**
```typescript
import init, { CollaborativeDocument, UserPresence } from '../wasm/mdcs_wasm';

let wasmInitialized = false;

export async function initCarnelia(): Promise<void> {
  if (!wasmInitialized) {
    await init();
    wasmInitialized = true;
  }
}

export { CollaborativeDocument, UserPresence };

export function generateReplicaId(): string {
  return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

export function generateUserColor(): string {
  const colors = [
    '#FF6B6B', '#4ECDC4', '#45B7D1', '#96CEB4', 
    '#FFEAA7', '#DDA0DD', '#98D8C8', '#F7DC6F'
  ];
  return colors[Math.floor(Math.random() * colors.length)];
}
```

### Step 4: React Hook

**`src/hooks/useCollaborativeDocument.ts`:**

The hook provides:
- `text` / `html` - Current document content
- `insert(position, text)` - Insert text
- `delete(position, length)` - Delete text  
- `applyBold(start, end)` - Bold formatting
- `applyItalic(start, end)` - Italic formatting
- `updatePresence(cursor, selectionStart?, selectionEnd?)` - Cursor sync
- `remoteUsers` - Other users' cursors
- `isConnected` - Sync status

### Step 5: Sync Server (Optional)

For multi-user collaboration, run a WebSocket server that:
1. Accepts connections per document room
2. Broadcasts state changes to all users in the room
3. Relays presence (cursor) updates

The server doesn't need CRDT logic - it just relays serialized states. The CRDT merge happens in each browser.

---

## Key Concepts

### CRDT Merge Flow

```
User A (Browser)          Sync Server          User B (Browser)
     |                        |                      |
     | insert("Hello")        |                      |
     |----------------------->|                      |
     |                        |--------------------->|
     |                        |    merge(stateA)     |
     |                        |                      |
     |                        |    insert("World")   |
     |<-----------------------|<---------------------|
     |    merge(stateB)       |                      |
     |                        |                      |
   [Both have "HelloWorld" - order determined by CRDT]
```

### Offline-First Flow

```
1. User edits while offline → changes stored locally
2. User reconnects → local state sent to server
3. Server broadcasts to other users
4. Each user merges → all converge to same state
```

---

## Should You Create a TypeScript SDK?

**Not yet.** The WASM approach gives you:
- Full Rust CRDT logic with proven correctness
- Type safety via wasm-bindgen
- Offline-first by default

A pure TypeScript SDK would mean reimplementing all the CRDT logic, which is:
- More work
- Risk of divergent behavior
- No benefit over WASM in browsers

**Later**, if you need Node.js server-side support without WASM, consider a TypeScript port.
