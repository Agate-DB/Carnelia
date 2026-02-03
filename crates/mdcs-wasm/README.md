# MDCS WebAssembly Bindings

WebAssembly bindings for the Merkle-Delta CRDT Store (MDCS), enabling real-time collaborative editing in web browsers.

## Features

- **CollaborativeDocument**: Rich text document with CRDT-based conflict resolution
- **UserPresence**: Cursor and selection tracking for collaborative UIs
- **Offline-first**: All CRDT operations work locally, sync when connected
- **Zero dependencies at runtime**: Pure WASM, no JavaScript CRDT libraries needed

## Installation

### Building from source

```bash
# Install wasm-pack if you haven't already
cargo install wasm-pack

# Build the WASM package
cd crates/mdcs-wasm
wasm-pack build --target web --out-dir pkg
```

### Using in a project

After building, copy the `pkg` directory to your project or publish to npm:

```bash
# Link locally for development
cd pkg && npm link
cd /your/project && npm link mdcs-wasm
```

## Quick Start

### Basic Usage

```javascript
import init, { CollaborativeDocument, UserPresence } from 'mdcs-wasm';

async function main() {
  // Initialize WASM module
  await init();

  // Create a new document
  const doc = new CollaborativeDocument('doc-123', 'user-abc');
  
  // Insert text
  doc.insert(0, 'Hello, World!');
  
  // Apply formatting
  doc.apply_bold(0, 5);
  doc.apply_italic(7, 12);
  
  // Get content
  console.log(doc.get_text());  // "Hello, World!"
  console.log(doc.get_html());  // "<b>Hello</b>, <i>World</i>!"
  console.log(doc.len());       // 13
}

main();
```

### Multi-User Collaboration

```javascript
import init, { CollaborativeDocument } from 'mdcs-wasm';

await init();

// User A creates a document
const docA = new CollaborativeDocument('shared-doc', 'user-a');
docA.insert(0, 'Hello');

// User B creates their replica
const docB = new CollaborativeDocument('shared-doc', 'user-b');
docB.insert(0, 'World');

// Sync: serialize states
const stateA = docA.serialize();
const stateB = docB.serialize();

// Merge remote states (order doesn't matter!)
docA.merge(stateB);
docB.merge(stateA);

// Both converge to the same state
console.log(docA.get_text() === docB.get_text()); // true
```

### User Presence Tracking

```javascript
import init, { UserPresence, generate_user_color } from 'mdcs-wasm';

await init();

// Create presence for local user
const myPresence = new UserPresence(
  'user-123',
  'Alice',
  generate_user_color()
);

// Update cursor on text input
myPresence.set_cursor(42);

// Update selection on mouse drag
myPresence.set_selection(10, 25);

// Send to other users via WebSocket
const presenceJson = myPresence.to_json();
ws.send(JSON.stringify({ type: 'presence', data: presenceJson }));

// Receive and render remote presence
ws.onmessage = (event) => {
  const { type, data } = JSON.parse(event.data);
  if (type === 'presence') {
    const remotePresence = UserPresence.from_json(data);
    renderRemoteCursor(remotePresence);
  }
};
```

## API Reference

### CollaborativeDocument

| Method | Description |
|--------|-------------|
| `new(doc_id, replica_id)` | Create a new document |
| `insert(position, text)` | Insert text at position |
| `delete(position, length)` | Delete text range |
| `apply_bold(start, end)` | Apply bold formatting |
| `apply_italic(start, end)` | Apply italic formatting |
| `apply_underline(start, end)` | Apply underline formatting |
| `apply_strikethrough(start, end)` | Apply strikethrough |
| `apply_link(start, end, url)` | Apply hyperlink |
| `get_text()` | Get plain text content |
| `get_html()` | Get HTML with formatting |
| `len()` | Get character count |
| `is_empty()` | Check if document is empty |
| `version()` | Get current version number |
| `serialize()` | Export state for sync |
| `merge(remote_state)` | Merge remote state (CRDT merge) |
| `snapshot()` | Create full snapshot |
| `restore(snapshot)` | Restore from snapshot |

### UserPresence

| Property/Method | Description |
|-----------------|-------------|
| `new(user_id, user_name, color)` | Create presence |
| `user_id` | Get user ID |
| `user_name` | Get display name |
| `color` | Get cursor color |
| `cursor` | Get cursor position |
| `selection_start` | Get selection start |
| `selection_end` | Get selection end |
| `set_cursor(position)` | Update cursor |
| `set_selection(start, end)` | Update selection |
| `clear()` | Clear cursor/selection |
| `has_selection()` | Check if has selection |
| `to_json()` | Serialize for network |
| `from_json(data)` | Deserialize from network |

### Utility Functions

| Function | Description |
|----------|-------------|
| `generate_replica_id()` | Generate unique replica ID |
| `generate_user_color()` | Get random user color |
| `console_log(message)` | Log to browser console |

## React Integration Example

```tsx
// hooks/useCollaborativeDocument.ts
import { useState, useEffect, useCallback, useRef } from 'react';
import init, { CollaborativeDocument } from 'mdcs-wasm';

export function useCollaborativeDocument(docId: string, userId: string) {
  const [doc, setDoc] = useState<CollaborativeDocument | null>(null);
  const [content, setContent] = useState('');
  const [isReady, setIsReady] = useState(false);

  useEffect(() => {
    init().then(() => {
      const newDoc = new CollaborativeDocument(docId, userId);
      setDoc(newDoc);
      setIsReady(true);
    });
  }, [docId, userId]);

  const insert = useCallback((pos: number, text: string) => {
    if (doc) {
      doc.insert(pos, text);
      setContent(doc.get_text());
    }
  }, [doc]);

  const deleteText = useCallback((pos: number, len: number) => {
    if (doc) {
      doc.delete(pos, len);
      setContent(doc.get_text());
    }
  }, [doc]);

  return { doc, content, isReady, insert, deleteText };
}
```

## Architecture

```
┌──────────────────────────────────────────────────┐
│                  React App                        │
│  ┌──────────────┐  ┌──────────────────────────┐  │
│  │ React Hooks  │  │ Document Editor UI       │  │
│  └──────┬───────┘  └──────────┬───────────────┘  │
│         │                     │                   │
│         ▼                     ▼                   │
│  ┌────────────────────────────────────────────┐  │
│  │           mdcs-wasm (WASM Module)          │  │
│  │  ┌─────────────────┐ ┌─────────────────┐   │  │
│  │  │CollaborativeDoc │ │  UserPresence   │   │  │
│  │  └─────────────────┘ └─────────────────┘   │  │
│  └────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
                         │
                         │ WebSocket (optional)
                         ▼
              ┌──────────────────────┐
              │    Sync Server       │
              │  (State Relay Only)  │
              └──────────────────────┘
```

## Building for Production

```bash
# Optimized build with size reduction
wasm-pack build --target web --release --out-dir pkg

# The pkg/ directory will contain:
# - mdcs_wasm.js       (JavaScript glue code)
# - mdcs_wasm_bg.wasm  (WebAssembly binary)
# - mdcs_wasm.d.ts     (TypeScript definitions)
```

### Bundle Size Optimization

The crate is configured with optimizations in `Cargo.toml`:

```toml
[profile.release]
opt-level = "s"  # Optimize for size
lto = true       # Link-time optimization
```

Expected bundle size: ~50-100KB gzipped (varies with features).

## Testing

### Rust Unit Tests

```bash
cargo test -p mdcs-wasm
```

### WASM Integration Tests

```bash
wasm-pack test --headless --chrome
```

## License

MIT License - see repository root for details.

## Related Packages

- `mdcs-core` - Core CRDT implementations
- `mdcs-delta` - Delta-state synchronization
- `mdcs-merkle` - Merkle-DAG causal tracking
- `mdcs-db` - Database layer with RichText support
- `mdcs-sdk` - Rust SDK for server-side usage
