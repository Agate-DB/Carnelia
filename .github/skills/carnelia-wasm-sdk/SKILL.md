---
name: carnelia-wasm-sdk
description: JavaScript/TypeScript-facing WebAssembly bindings for MDCS collaborative CRDT documents, presence, and offline-first merge in browsers.
metadata:
	tags: wasm, javascript, typescript, crdt, collaborative, rich-text, json, presence, browser, mdcs
---

# Carnelia WASM SDK (`mdcs-wasm`)

WebAssembly bindings for using MDCS CRDTs from JavaScript/TypeScript in browser apps. Exposes ergonomic document APIs for rich text, plain text, and JSON collaboration, plus user-presence helpers and utility functions.

**Internal crate dependencies:** `mdcs-core` (lattice join), `mdcs-db` (`RichText`, `RGAText`, `JsonCrdt`, mark and JSON types), `serde_wasm_bindgen` (JS<->Rust serialization), `wasm-bindgen`/`js-sys`/`web-sys`.

## When to Use

Activate this skill when the user is:

- Building browser-based collaborative editors with Rust/WASM + JS/TS
- Working with `CollaborativeDocument`, `TextDocument`, `RichTextDocument`, or `JsonDocument`
- Implementing in-browser CRDT merge and snapshot/restore flows
- Managing collaborative cursor/selection state with `UserPresence`
- Designing JS/TS APIs around serialized document state exchange
- Integrating MDCS WASM exports into React/Vue/Svelte/vanilla web apps

---

## Architecture

The WASM SDK is a bindings layer over `mdcs-db` CRDT implementations, exposing JS-callable classes via `#[wasm_bindgen]`.

### Binding Surface

| Binding | Responsibility | Core Backing Type |
|--------|----------------|-------------------|
| `CollaborativeDocument` | Rich-text collaborative editing + formatting marks | `mdcs_db::RichText` |
| `TextDocument` | Plain-text collaborative editing API | `mdcs_db::RGAText` |
| `RichTextDocument` | Explicit rich-text wrapper (SDK-style naming) | wraps `CollaborativeDocument` |
| `JsonDocument` | Collaborative JSON CRDT operations (dot-path + arrays) | `mdcs_db::JsonCrdt` |
| `UserPresence` | Cursor/selection/user metadata helper for UI | Plain Rust struct serialized via Serde |
| Utility fns | Runtime and UX helpers | `init_panic_hook`, `generate_replica_id`, `generate_user_color`, `console_log` |

### Runtime Model

- **Execution environment:** Browser/WebAssembly target
- **State model:** Local-first mutable CRDT state in each JS-visible object instance
- **Sync model:** Caller-driven — serialize/merge manually, no built-in transport/network loop
- **Convergence model:** `Lattice::join` on underlying CRDT states (`join` is commutative, associative, idempotent)

---

## API Reference

### Startup / Runtime Utilities

| Function | Signature | Notes |
|----------|-----------|-------|
| `init_panic_hook` | `() -> void` | `#[wasm_bindgen(start)]`; enables better panic messages when feature enabled |
| `generate_replica_id` | `() -> string` | Timestamp + random bits string |
| `generate_user_color` | `() -> string` | Picks from fixed 16-color palette |
| `console_log` | `(message: string) -> void` | Writes to browser console |

---

### `CollaborativeDocument`

Rich-text collaborative document with formatting support and CRDT merge.

| Method | Signature | Notes |
|--------|-----------|-------|
| `new` | `(doc_id: string, replica_id: string)` | Constructor |
| `insert` | `(position: number, text: string)` | Clamps to current length |
| `delete` | `(position: number, length: number)` | Safe clamped delete |
| `apply_bold` / `apply_italic` / `apply_underline` / `apply_strikethrough` / `apply_code` | `(start: number, end: number)` | Applies corresponding `MarkType` |
| `apply_link` | `(start: number, end: number, url: string)` | Link mark |
| `apply_highlight` | `(start: number, end: number, color: string)` | Highlight mark |
| `apply_comment` | `(start: number, end: number, author: string, content: string)` | Comment mark |
| `apply_custom_mark` | `(start: number, end: number, name: string, value: string)` | Custom mark |
| `get_text` | `() -> string` | Plain text projection |
| `get_html` | `() -> string` | HTML rendering from `RichText` |
| `len` | `() -> number` | Character length |
| `is_empty` | `() -> boolean` | |
| `version` | `() -> number` | Local monotonic counter |
| `doc_id` | `() -> string` | Returns cloned ID |
| `replica_id` | `() -> string` | Returns cloned replica ID |
| `serialize` | `() -> Result<string, JsValue>` | JSON string of serialized `RichText` state |
| `merge` | `(remote_state: string) -> Result<void, JsValue>` | Parses remote JSON and joins state |
| `snapshot` | `() -> Result<JsValue, JsValue>` | `{ doc_id, replica_id, version, state }` |
| `restore` | `(snapshot: JsValue) -> Result<CollaborativeDocument, JsValue>` | Rehydrates document from snapshot |

---

### `TextDocument`

Plain-text collaborative wrapper over `RGAText`.

| Method | Signature | Notes |
|--------|-----------|-------|
| `new` | `(doc_id: string, replica_id: string)` | Constructor |
| `insert` | `(position: number, text: string)` | Clamped position |
| `delete` | `(position: number, length: number)` | Clamped delete |
| `replace` | `(start: number, end: number, text: string)` | Delegates to `RGAText::replace` |
| `splice` | `(position: number, delete_count: number, insert: string)` | Delegates to `RGAText::splice` |
| `get_text` | `() -> string` | |
| `len` | `() -> number` | |
| `is_empty` | `() -> boolean` | |
| `version` | `() -> number` | |
| `doc_id` / `replica_id` | `() -> string` | |
| `serialize` | `() -> Result<string, JsValue>` | JSON string |
| `merge` | `(remote_state: string) -> Result<void, JsValue>` | CRDT join |
| `snapshot` / `restore` | Snapshot object roundtrip | Same shape as `CollaborativeDocument` |

---

### `RichTextDocument`

Naming wrapper around `CollaborativeDocument` (delegates all methods).

| Method Surface | Notes |
|---------------|-------|
| Constructors, edit ops, format ops, reads, ids, `serialize`, `merge`, `snapshot`, `restore` | Fully forwarded to internal `CollaborativeDocument` |

Use this when API naming clarity matters (`RichTextDocument`) but behavior should match `CollaborativeDocument` exactly.

---

### `JsonDocument`

Collaborative JSON CRDT with dot-path convenience and typed setters.

| Method | Signature | Notes |
|--------|-----------|-------|
| `new` | `(doc_id: string, replica_id: string)` | Constructor |
| `set_string` / `set_int` / `set_float` / `set_bool` / `set_null` | `(path: string, value?) -> Result<void, JsValue>` | Uses `JsonPath::parse(path)` |
| `set_object` / `set_array` | `(path: string) -> Result<void, JsValue>` | Creates object/array node |
| `array_push_string` / `array_push_int` / `array_push_float` / `array_push_bool` / `array_push_null` | `(path: string, value?) -> Result<void, JsValue>` | Requires path to existing array |
| `array_remove` | `(path: string, index: number) -> Result<JsValue, JsValue>` | Returns removed primitive JSON value or placeholder for complex references |
| `delete` | `(path: string) -> Result<void, JsValue>` | Deletes node |
| `get` | `(path: string) -> Result<JsValue, JsValue>` | Dot-path lookup over `to_json()` projection |
| `to_json` | `() -> Result<JsValue, JsValue>` | Entire JSON projection |
| `keys` | `() -> Result<JsValue, JsValue>` | Top-level keys |
| `contains_key` | `(key: string) -> boolean` | Top-level key check |
| `version` / `doc_id` / `replica_id` | Accessors | |
| `serialize` / `merge` | State roundtrip + CRDT join | |
| `snapshot` / `restore` | Snapshot object roundtrip | |

---

### `UserPresence`

Lightweight user-awareness payload object for collaborative UI rendering.

| Method | Signature | Notes |
|--------|-----------|-------|
| `new` | `(user_id: string, user_name: string, color: string)` | Constructor |
| `set_cursor` | `(position: number)` | Clears selection |
| `set_selection` | `(start: number, end: number)` | Normalizes range and sets cursor at `end` |
| `clear` | `() -> void` | Clears cursor and selection |
| `user_id` / `user_name` / `color` | Getters returning strings | |
| `cursor` / `selection_start` / `selection_end` | Getters returning optional numbers | |
| `has_selection` | `() -> boolean` | |
| `to_json` | `() -> Result<JsValue, JsValue>` | Serialize presence payload |
| `from_json` | `(js: JsValue) -> Result<UserPresence, JsValue>` | Deserialize payload |

---

## JavaScript / TypeScript Usage Patterns

### Basic Rich Text Collaboration

```ts
import init, { CollaborativeDocument } from "mdcs-wasm";

await init();

const alice = new CollaborativeDocument("doc-1", "alice");
const bob = new CollaborativeDocument("doc-1", "bob");

alice.insert(0, "Hello");
alice.apply_bold(0, 5);

// transport payload from alice -> bob
const wire = alice.serialize();
if (typeof wire === "string") {
	bob.merge(wire);
}

console.log(bob.get_text());
console.log(bob.get_html());
```

### Snapshot / Restore

```ts
const snapshot = doc.snapshot();
if (snapshot) {
	const restored = CollaborativeDocument.restore(snapshot);
}
```

### JSON Collaboration Flow

```ts
const json = new JsonDocument("profile-doc", "replica-a");
json.set_object("profile");
json.set_string("profile.name", "Alice");
json.set_array("tags");
json.array_push_string("tags", "crdt");

const remoteState = json.serialize();
peer.merge(remoteState);
```

### Presence Payload Exchange

```ts
const presence = new UserPresence("u1", "Alice", "#FF6B6B");
presence.set_selection(4, 10);
const payload = presence.to_json();

// send payload via your transport, then:
const remotePresence = UserPresence.from_json(payload);
```

---

## Key Patterns

### Manual Sync Loop (Caller-Owned)

The WASM SDK intentionally does not include peer/network transport. Typical JS/TS flow:

1. Apply local mutation (`insert`, `set_*`, etc.)
2. Call `serialize()`
3. Send payload over your transport (WebSocket/WebRTC/etc.)
4. Remote side calls `merge(remote_state)`

### Version Counter Semantics

`version` increments for local write operations and on merge. Treat it as a local change counter, not a globally synchronized version clock.

### Snapshot Envelope

All document types use a snapshot envelope with this shape:

```ts
{
	doc_id: string;
	replica_id: string;
	version: number;
	state: string; // serialized CRDT JSON string
}
```

---

## Error Surface

All fallible bindings return `Result<_, JsValue>` to JS. Common failure classes:

- JSON parse/stringify failures in serialize/merge/restore
- Serde conversion failures (`serde_wasm_bindgen`)
- Invalid JSON path operations in `JsonDocument`
- Array-path mismatch (`path` exists but is not an array)

Use `try/catch` around calls such as `merge`, `restore`, and JSON setters when consuming from JS/TS.

---

## Known Limitations

1. **No built-in networking or auto-sync manager** — callers must provide transport and retry logic.
2. **`serialize()` docs mention base64, implementation emits JSON string** — treat payload as JSON text.
3. **Array removal of complex JSON values is lossy in return path** — returns placeholder string for object/array references in `array_remove` result conversion.
4. **`generate_replica_id()` is convenience-only** — timestamp + random bits, not a cryptographic or RFC-UUID guarantee.
5. **Presence is local payload modeling only** — no built-in shared tracker or stale-user cleanup in WASM layer.

---

## Testing Notes

The crate includes native unit tests for API behavior and convergence patterns. Full browser/WASM integration behavior (especially JS serialization edge cases) should be validated with `wasm-bindgen-test` in a WASM test target.
