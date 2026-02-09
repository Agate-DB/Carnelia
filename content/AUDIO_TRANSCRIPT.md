# CRDT Explainer — Audio Transcript

Record each segment as a separate audio file and place them in `content/public/audio/`.
The composition will use `<Audio>` from `@remotion/media` with `<Sequence>` timing.

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

## Scene 7 — CRDT Limitations (0:44.2 – 0:54.9)
**File:** `limitations_narration.mp3`
**Duration:** ~10.7 seconds

> But traditional CRDTs have real-world problems. First, state bloat: state-based CRDTs must ship the entire object on every sync — and that payload only grows. Second, tombstone accumulation: many systems mark deletions with permanent markers that never get cleaned up. Third, vector clock fragility: their metadata scales linearly with replicas and is vulnerable to Byzantine manipulation. And fourth, transport assumptions: operation-based CRDTs demand reliable, exactly-once, causal delivery — a guarantee that rarely holds in peer-to-peer networks.

---

## Scene 8 — How Carnelia Fixes It (0:54.9 – 1:05.6)
**File:** `solution_narration.mp3`
**Duration:** ~10.7 seconds

> Carnelia's Merkle-Delta architecture addresses each of these problems directly. Instead of full state, we ship compact delta mutations — idempotent and commutative. Instead of tombstones, we use a dot store and causal context — absence is the deletion record, keeping metadata bounded. Instead of vector clocks, we use a Merkle-Clock DAG — content-addressed, tamper-proof, and independent of replica count. And instead of hoping for reliable transport, the DAG-Syncer performs pull-based gap repair — fetching exactly the missing blocks by hash.

---

## Scene 9 — Tombstone-Free Removal (1:05.6 – 1:13.6)
**File:** `dotstore_narration.mp3`
**Duration:** ~8 seconds

> Deletion in CRDTs is tricky. Many systems use tombstones — markers that say "this was deleted" — and they accumulate forever. The MDCS architecture avoids this entirely. It uses a causal context that tracks every event ever created, and a dot store that holds only the live data. If a dot exists in the context but not in the store, it's been removed. No tombstones, bounded metadata.

---

## Scene 10 — Merkle-Clock DAG (1:13.6 – 1:23.6)
**File:** `merkle_narration.mp3`
**Duration:** ~10 seconds

> Finally, causality. Instead of vector clocks — which grow with the number of replicas and are vulnerable to Byzantine manipulation — the MDCS uses a Merkle-Clock. Every update is hashed and linked to its predecessors, forming an immutable directed acyclic graph. Concurrent updates fork the DAG; merges rejoin it. When a new replica joins, the DAG-Syncer walks backward from the head, fetching any missing blocks by their content identifiers. Two replicas that share the same head hashes are guaranteed to have identical causal histories.

---

## Audio Integration

Once you've recorded the files, place them at:
```
content/public/audio/presentedby_narration.mp3
content/public/audio/intro_narration.mp3
content/public/audio/replicas_narration.mp3
content/public/audio/semilattice_narration.mp3
content/public/audio/delta_narration.mp3
content/public/audio/merge_narration.mp3
content/public/audio/limitations_narration.mp3
content/public/audio/solution_narration.mp3
content/public/audio/dotstore_narration.mp3
content/public/audio/merkle_narration.mp3
```

Then I can add `<Audio>` elements wrapped in `<Sequence>` components to each scene in
the `CrdtExplainer.tsx` composition, timed to the scene offsets.
