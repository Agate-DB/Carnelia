# CRDT Explainer — Audio Transcript (ElevenLabs TTS)

Each scene's narration is stored in `content/public/audio/` and loaded via
`<Audio>` inside each `<Series.Sequence>` in `CrdtExplainer.tsx`.

**Subtitles** are embedded directly in the composition via `SubtitleOverlay` segments
in `CrdtExplainer.tsx`. They fade in/out automatically with each scene.

### ElevenLabs Annotation Guide

| Annotation | Effect | Example |
|---|---|---|
| `<break time="0.5s"/>` | Pause for X seconds | `idea. <break time="0.8s"/> Now,` |
| `...` | Soft trailing pause | `and that's the thing...` |
| `—` | Natural breath pause | `no locks — no consensus` |
| ALL CAPS on a word | Slight emphasis | `EVERY replica` |
| Short sentences | Clearer pacing | Split long runs |
| `?` at end | Rising intonation | Questions sound natural |
| Commas | Micro-pauses | Use generously |

**Voice settings recommendation:** Stability 0.45–0.55, Similarity 0.70, Style 0.15–0.25, Speaker boost ON.

---

## Scene 1 — Presented by Carnelia (0:00 – 0:08.5)
**File:** `presentedby_narration.mp3`
**Duration:** ~8.5 seconds

> Presented by Carnelia <break time="0.4s"/> — the Merkle-Delta CRDT Store.

---

## Scene 2 — Intro (0:08.5 – 0:19.0)
**File:** `intro_narration.mp3`
**Duration:** ~10.5 seconds

> How can distributed replicas update data independently, <break time="0.3s"/> without any coordination, <break time="0.3s"/> and still converge to exactly the same state? <break time="0.5s"/> This is the problem that CRDTs solve.

---

## Scene 3 — Optimistic Replication (0:19.0 – 0:31.0)
**File:** `replicas_narration.mp3`
**Duration:** ~12 seconds

> In optimistic replication, every replica accepts writes locally <break time="0.3s"/> — no locks, no consensus, no waiting. <break time="0.5s"/> Each node maintains a full copy and applies updates immediately. <break time="0.3s"/> High availability, <break time="0.2s"/> partition tolerance.

---

## Scene 4 — Join-Semilattice (0:31.0 – 0:42.25)
**File:** `semilattice_narration.mp3`
**Duration:** ~11.25 seconds

> The mathematical foundation is a join-semilattice. <break time="0.3s"/> Every state has a merge operation that computes the least upper bound. <break time="0.5s"/> This merge is commutative, associative, and idempotent <break time="0.3s"/> — apply it in any order, any number of times.

---

## Scene 5 — Delta Mutations (0:42.25 – 0:54.25)
**File:** `delta_narration.mp3`
**Duration:** ~12 seconds

> Traditional state-based CRDTs ship the entire object every sync <break time="0.3s"/> — expensive. <break time="0.5s"/> Delta CRDTs generate tiny incremental mutations. <break time="0.3s"/> Far smaller than full state, dramatically reducing bandwidth. <break time="0.5s"/> Idempotent and commutative <break time="0.3s"/> — they tolerate loss, duplication, and reordering.

---

## Scene 6 — Strong Eventual Consistency (0:54.25 – 1:06.25)
**File:** `merge_narration.mp3`
**Duration:** ~12 seconds

> The payoff: <break time="0.3s"/> Strong Eventual Consistency. <break time="0.5s"/> Once all replicas receive the same updates <break time="0.3s"/> — regardless of delivery order — they converge to an identical state. <break time="0.5s"/> No coordination, no consensus protocol, no conflict resolution needed.

---

## Scene 7 — Real-World CRDT Examples (1:06.25 – 1:21.25)
**File:** `realworld_narration.mp3`
**Duration:** ~15 seconds

> CRDTs power some of the most popular collaboration tools today. <break time="0.4s"/> Figma, Google Docs, Apple Notes, Linear, VS Code Live Share <break time="0.3s"/> — local-first writes with automatic convergence. <break time="0.5s"/> But most rely on a central server. <break time="0.3s"/> Carnelia goes fully peer-to-peer.

---

## Scene 8 — CRDT Limitations (1:21.25 – 1:40.25)
**File:** `limitations_narration.mp3`
**Duration:** ~19 seconds

> But traditional CRDTs have real-world problems. <break time="0.4s"/> State bloat, <break time="0.2s"/> tombstone accumulation, <break time="0.2s"/> vector clock fragility, <break time="0.2s"/> transport assumptions. <break time="0.5s"/> And network partition fragility <break time="0.5s"/> These gaps prevent production deployment in open-membership networks.

---

## Scene 9 — How Carnelia Fixes It (1:40.25 – 1:58.25)
**File:** `solution_narration.mp3`
**Duration:** ~18 seconds

> Carnelia's Merkle-Delta architecture addresses each problem directly. <break time="0.4s"/> Delta-CRDT deltas instead of full state. <break time="0.3s"/> Dot store instead of tombstones. <break time="0.3s"/> Merkle-Clock instead of vector clocks. <break time="0.5s"/> DAG-Syncer for reliable transport. <break time="0.3s"/> Anti-entropy gossip for partition recovery. <break time="0.5s"/> A complete architectural synthesis.

---

## Scene 10 — Tombstone-Free Removal (1:58.25 – 2:10.25)
**File:** `dotstore_narration.mp3`
**Duration:** ~12 seconds

> Many systems use tombstones <break time="0.3s"/> — markers that say "this was deleted" — and they accumulate forever. <break time="0.5s"/> MDCS uses a causal context and a dot store. <break time="0.3s"/> Absence in the store means deleted. <break time="0.3s"/> No tombstones, <break time="0.2s"/> bounded metadata.

---

## Scene 11 — Merkle-Clock DAG (2:10.25 – 2:25.25)
**File:** `merkle_narration.mp3`
**Duration:** ~15 seconds

> Instead of vector clocks, the MDCS uses a Merkle-Clock <break time="0.3s"/> — an immutable DAG of hashed updates. <break time="0.5s"/> Concurrent updates fork the DAG; <break time="0.3s"/> merges rejoin it. <break time="0.3s"/> The DAG-Syncer fetches missing blocks by content ID. <break time="0.5s"/> Same head hashes equals identical causal histories. <break time="0.3s"/> Guaranteed.

---

## Scene 12 — Carnelia Offline Sync (2:25.25 – 2:45.25)
**File:** `sync_narration.mp3`
**Duration:** ~20 seconds

> Two devices start connected, editing the same document. <break time="0.3s"/> Then the mobile goes offline. <break time="0.5s"/> Both continue editing independently — the document diverges. <break time="0.6s"/> When connectivity returns, the anti-entropy protocol kicks in: <break time="0.3s"/> gossip head CIDs, fetch missing blocks, apply deltas in topological order. <break time="0.5s"/> Both replicas converge <break time="0.3s"/> — identical state, zero conflicts, no server required.

---

## Scene 13 — Collaborative Editing Demo (2:45.25 – 3:05.75)
**File:** `collab_narration.mp3`
**Duration:** ~20.5 seconds

> In Figma or Google Docs, every edit routes through a central server. <break time="0.5s"/> In Carnelia, there is no server. <break time="0.3s"/> Three team members edit JSON config concurrently. <break time="0.5s"/> After CRDT merge: <break time="0.3s"/> a single consistent document with zero conflicts. <break time="0.5s"/> Rich text works the same way <break time="0.3s"/> — concurrent insertions resolve via unique position IDs.

---

## Scene 14 — PNCounter Step-by-Step Demo (3:05.75 – 3:26.75)
**File:** `increment_narration.mp3`
**Duration:** ~21 seconds

> Alice increments page views by five, then three <break time="0.3s"/> — her local counter reads eight. <break time="0.5s"/> Bob independently increments page views by ten and likes by two. <break time="0.3s"/> Neither knows about the other. <break time="0.5s"/> They sync <break time="0.3s"/> — Bob to Alice, Alice to Bob. <break time="0.3s"/> Bidirectional CRDT merge via delta exchange. <break time="0.5s"/> Both replicas converge: <break time="0.3s"/> page views equals ten, likes equals two. <break time="0.3s"/> Identical state, no coordination.

---

## Scene 15 — End Screen / Summary (3:26.75 – 3:44.75)
**File:** `end_narration.mp3`
**Duration:** ~18 seconds

> CRDTs guarantee convergence without consensus <break time="0.3s"/> — and Carnelia makes it practical. <break time="0.5s"/> Open-membership, <break time="0.2s"/> offline-first, <break time="0.2s"/> peer-to-peer, <break time="0.2s"/> Byzantine-tolerant. <break time="0.5s"/> github.com/Agate-DB/Carnelia.

---

## Audio Integration

### Background Soundtrack

Place the ambient background music at:
```
content/public/ambient_bg_soundtrack.ogg
```

This plays across **all 15 scenes** at low volume (10%) as an ambient bed.
It is loaded via `<Audio src={staticFile("ambient_bg_soundtrack.ogg")} volume={0.10} />`
in the root `CrdtExplainer.tsx` composition.

### Per-Scene Narration

Narration files are loaded via `<Audio>` inside each `<Series.Sequence>`:
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
content/public/audio/end_narration.mp3
```

### ElevenLabs Settings

| Setting | Recommended Value |
|---|---|
| **Voice** | Any clean narrator voice (e.g., Adam, Antoni, Josh) |
| **Model** | Eleven Multilingual v2 or Turbo v2.5 |
| **Stability** | 0.45–0.55 (allows natural variation) |
| **Similarity** | 0.70 |
| **Style** | 0.15–0.25 (subtle expressiveness) |
| **Speaker Boost** | ON |

**Tips:**
- Paste each scene's blockquote text directly into ElevenLabs
- `<break time="Xs"/>` tags are supported — they insert real pauses
- Keep each scene as a separate generation for consistent pacing
- Trim silence at start/end of each clip before placing in `content/public/audio/`

### Subtitles

Subtitles are already embedded in the composition via the `SUBTITLES` array
in `CrdtExplainer.tsx`. Each scene has 1–4 timed text segments that appear
as a semi-transparent overlay at the bottom or top of the frame.
