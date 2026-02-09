# CRDT Explainer — Audio Transcript

Record each segment as a separate audio file and place them in `content/public/audio/`.
The composition uses `<Audio>` from Remotion with `<Sequence>` timing.

**Subtitles** are embedded directly in the composition via `SubtitleOverlay` segments
in `CrdtExplainer.tsx`. They fade in/out automatically with each scene.

---

## Scene 1 — Presented by Carnelia (0:00 – 0:05.7)
**File:** `presentedby_narration.mp3` *(optional — can be ambient music only)*
**Duration:** ~5.7 seconds

> *(Ambient music / subtle tone. No narration needed — the visuals carry this title card.)*
> *(If narration is desired:)* Presented by Carnelia — the Merkle-Delta CRDT Store.

---

## Scene 2 — Intro (0:05.7 – 0:12.7)
**File:** `intro_narration.mp3`
**Duration:** ~7 seconds

> How can distributed replicas — devices, servers, or browsers — update data independently, without any coordination, and still converge to exactly the same state? This is the problem that CRDTs solve.

---

## Scene 3 — Optimistic Replication (0:12.7 – 0:20.7)
**File:** `replicas_narration.mp3`
**Duration:** ~8 seconds

> In optimistic replication, every replica accepts writes locally — no locks, no consensus, no waiting. Each node maintains a full copy of the data and applies updates immediately. This gives us high availability and partition tolerance.

---

## Scene 4 — Join-Semilattice (0:20.7 – 0:28.2)
**File:** `semilattice_narration.mp3`
**Duration:** ~7.5 seconds

> The mathematical foundation is a join-semilattice. Every state has a merge operation — called "join" — that computes the least upper bound of two divergent states. This merge is commutative, associative, and idempotent. You can apply it in any order, any number of times, and always arrive at the same result.

---

## Scene 5 — Delta Mutations (0:28.2 – 0:36.2)
**File:** `delta_narration.mp3`
**Duration:** ~8 seconds

> Traditional state-based CRDTs ship the entire object every time they sync — expensive. Delta CRDTs solve this by generating tiny, incremental mutations called deltas. These deltas are far smaller than the full state, dramatically reducing bandwidth. And because they're idempotent and commutative, they tolerate loss, duplication, and reordering on the network.

---

## Scene 6 — Strong Eventual Consistency (0:36.2 – 0:44.2)
**File:** `merge_narration.mp3`
**Duration:** ~8 seconds

> The payoff: Strong Eventual Consistency. Once all replicas have received the same set of updates — regardless of delivery order — they are mathematically guaranteed to converge to an identical state. No coordination, no consensus protocol, no conflict resolution needed.

---

## Scene 7 — Real-World CRDT Examples (0:44.2 – 0:54.2)
**File:** `realworld_narration.mp3`
**Duration:** ~10 seconds

> CRDTs aren't just theory — they power some of the most popular collaboration tools today. Figma uses a custom CRDT for multiplayer design. Google Docs runs on Operational Transformation — a related approach — with a central server. Apple Notes syncs across devices with CRDTs. Linear uses them for real-time issue tracking. And VS Code Live Share enables collaborative coding with CRDT-backed shared state. The pattern is the same: local-first writes with automatic convergence. But most of these rely on a central server. Carnelia goes fully peer-to-peer.

---

## Scene 8 — CRDT Limitations (0:54.2 – 1:06.9)
**File:** `limitations_narration.mp3`
**Duration:** ~12.7 seconds

> But traditional CRDTs have real-world problems. First, state bloat: state-based CRDTs must ship the entire object on every sync — and that payload only grows. Second, tombstone accumulation: many systems mark deletions with permanent markers that never get cleaned up. Third, vector clock fragility: their metadata scales linearly with replicas and is vulnerable to Byzantine manipulation. Fourth, transport assumptions: operation-based CRDTs demand reliable, exactly-once, causal delivery — a guarantee that rarely holds in peer-to-peer networks. And fifth, network partition fragility: when connectivity degrades, replica groups diverge with no built-in mechanism for automatic repair after the partition heals.

---

## Scene 9 — How Carnelia Fixes It (1:06.9 – 1:18.9)
**File:** `solution_narration.mp3`
**Duration:** ~12 seconds

> Carnelia's Merkle-Delta architecture addresses each of these problems directly. Instead of full state, we ship compact delta mutations — idempotent and commutative. Instead of tombstones, we use a dot store and causal context — absence is the deletion record, keeping metadata bounded. Instead of vector clocks, we use a Merkle-Clock DAG — content-addressed, tamper-proof, and independent of replica count. Instead of hoping for reliable transport, the DAG-Syncer performs pull-based gap repair — fetching exactly the missing blocks by hash. And for network partitions, the anti-entropy gossip protocol broadcasts head CIDs and repairs gaps automatically when connectivity returns.

---

## Scene 10 — Tombstone-Free Removal (1:18.9 – 1:26.9)
**File:** `dotstore_narration.mp3`
**Duration:** ~8 seconds

> Deletion in CRDTs is tricky. Many systems use tombstones — markers that say "this was deleted" — and they accumulate forever. The MDCS architecture avoids this entirely. It uses a causal context that tracks every event ever created, and a dot store that holds only the live data. If a dot exists in the context but not in the store, it's been removed. No tombstones, bounded metadata.

---

## Scene 11 — Merkle-Clock DAG (1:26.9 – 1:36.9)
**File:** `merkle_narration.mp3`
**Duration:** ~10 seconds

> Finally, causality. Instead of vector clocks — which grow with the number of replicas and are vulnerable to Byzantine manipulation — the MDCS uses a Merkle-Clock. Every update is hashed and linked to its predecessors, forming an immutable directed acyclic graph. Concurrent updates fork the DAG; merges rejoin it. When a new replica joins, the DAG-Syncer walks backward from the head, fetching any missing blocks by their content identifiers. Two replicas that share the same head hashes are guaranteed to have identical causal histories.

---

## Scene 12 — Carnelia Offline Sync (1:36.9 – 1:50.2)
**File:** `sync_narration.mp3`
**Duration:** ~13.3 seconds

> Let's see how Carnelia handles offline sync. Two devices — say a desktop and a mobile phone — start connected, editing the same document. Now the mobile goes offline. Both continue making edits independently — the document diverges. When connectivity returns, Carnelia's anti-entropy protocol kicks in. Step one: each replica gossips the CIDs of its DAG heads to peers. Step two: the DAG-Syncer compares these against its local graph. Step three: it fetches any missing blocks by hash — walking backward through predecessor links until it finds a common ancestor. Step four: the fetched deltas are applied in topological order. Both replicas converge — identical state, zero conflicts, no server required.

---

## Scene 13 — Collaborative Editing Demo (1:50.2 – 2:03.9)
**File:** `collab_narration.mp3`
**Duration:** ~13.7 seconds

> Now let's contrast Carnelia's approach to collaborative editing with traditional tools. In Figma or Google Docs, every edit routes through a central server — the server mediates conflicts, and you need an internet connection. In Carnelia, there is no server. Three team members — a project manager, a developer, and a designer — can all edit the same JSON configuration concurrently. One sets the project name and version, another adds the tech stack, a third defines the UI theme. When their replicas sync, the CRDT merge produces a single, consistent document with zero conflicts. The same principle applies to the rich text layer — concurrent character insertions resolve via unique position IDs in the RGA sequence, with no server arbitration needed.

---

## Scene 14 — PNCounter Step-by-Step Demo (2:03.9 – 2:17.9)
**File:** `increment_narration.mp3`
**Duration:** ~14 seconds

> Let's walk through a concrete example. Alice and Bob each have a replica tracking page views and likes. In phase one, Alice increments page views by five, then by three — her local counter reads eight. In phase two, Bob independently increments page views by ten and likes by two. Neither knows about the other's changes. In phase three, they sync — bob syncs to alice, alice syncs to bob. The CRDT merge resolves both replicas' contributions. The result: both replicas converge to page views equals ten, likes equals two. Identical state, arrived at independently, with no coordination.

---

## Audio Integration

### Background Soundtrack

Place the ambient background music at:
```
content/public/ambient_bg_soundtrack.mp3
```

This plays across **all 14 scenes** at low volume (18%) as an ambient bed.
It is loaded via `<Audio src={staticFile("ambient_bg_soundtrack.mp3")} volume={0.18} />`
in the root `CrdtExplainer.tsx` composition.

### Per-Scene Narration (optional)

Once you've recorded the narration files, place them at:
```
content/public/audio/presentedby_narration.mp3
content/public/audio/intro_narration.mp3
content/public/audio/replicas_narration.mp3
content/public/audio/semilattice_narration.mp3
content/public/audio/delta_narration.mp3
content/public/audio/merge_narration.mp3
content/public/audio/realworld_narration.mp3
content/public/audio/limitations_narration.mp3
content/public/audio/solution_narration.mp3
content/public/audio/dotstore_narration.mp3
content/public/audio/merkle_narration.mp3
content/public/audio/sync_narration.mp3
content/public/audio/collab_narration.mp3
content/public/audio/increment_narration.mp3
```

Add per-scene `<Audio>` elements as `<Sequence>` children once recorded.

### Subtitles

Subtitles are already embedded in the composition via the `SUBTITLES` array
in `CrdtExplainer.tsx`. Each scene has 1–4 timed text segments that appear
as a semi-transparent overlay at the bottom of the frame.
