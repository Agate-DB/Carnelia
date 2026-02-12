import React from "react";
import { AbsoluteFill, Audio, Sequence, Series, staticFile, useCurrentFrame, useVideoConfig, interpolate } from "remotion";
import { z } from "zod";
import { BirdWatchScene } from "./scenes/BirdWatchScene";
import { CrdtBreakthroughScene } from "./scenes/CrdtBreakthroughScene";
import { ReplicaScene } from "./scenes/ReplicaScene";
import { MergeScene } from "./scenes/MergeScene";
import { SemilatticeScene } from "./scenes/SemilatticeScene";
import { GCounterScene } from "./scenes/GCounterScene";
import { LimitationsScene } from "./scenes/LimitationsScene";
import { CarneliaSolutionScene } from "./scenes/CarneliaSolutionScene";
import { MerkleScene } from "./scenes/MerkleScene";
import { DotStoreScene } from "./scenes/DotStoreScene";
import { EndScene } from "./scenes/EndScene";
import { DeltaScene } from "./scenes/DeltaScene";
import { CarneliaSyncScene } from "./scenes/CarneliaSyncScene";
import { CollabDemoScene } from "./scenes/CollabDemoScene";
import { RealWorldCrdtScene } from "./scenes/RealWorldCrdtScene";
import { FONT_PRIMARY } from "./fonts";
import { PresentedByScene } from "./scenes/PresentedByScene";

/**
 * CrdtExplainer — main composition (v3: BirdWatch narrative)
 *
 * 16 scenes at 20 fps — hybrid: ThreeCanvas + remotion-bits
 *   0. Presented By (prologue)              220 frames  (11s)
 *   1. Coordination Bottleneck (remotion-bits) 700 frames (35s)
 *   2. CRDT Breakthrough (remotion-bits)    500 frames  (25s)
 *   3. Scaling Problem (ThreeCanvas)        500 frames  (25s)
 *   4. CRDT Solution (ThreeCanvas)          500 frames  (25s)
 *   5. Join Semi-Lattice (ThreeCanvas)      400 frames  (20s)
 *   6. G-Counter (remotion-bits)            500 frames  (25s)
 *   7. Limits of Basic CRDTs (ThreeCanvas)  400 frames  (20s)
 *   8. Enter MDCS (ThreeCanvas)             300 frames  (15s)
 *   9. Merkle-Clock (ThreeCanvas)           500 frames  (25s)
 *  10. Tombstone-Free (ThreeCanvas)         400 frames  (20s)
 *  11. Delta Propagation (ThreeCanvas)      430 frames  (21.5s)
 *  12. Offline Sync (ThreeCanvas)           590 frames  (29.5s)
 *  13. Collab Demo (ThreeCanvas)            460 frames  (23s)
 *  14. Real-World CRDTs (ThreeCanvas)       410 frames  (20.5s)
 *  15. Conclusion (ThreeCanvas)             400 frames  (20s)
 *                                         ──────
 *                                  total: 7210 frames (360.5s / ~6:00)
 */

/* ── Scene durations ────────────────────────────────────── */
const SCENE_DURATIONS = [220, 700, 500, 500, 500, 400, 500, 400, 300, 500, 400, 430, 590, 460, 410, 400] as const;

/* ── Subtitle segments: each mapped to a scene ────────── */
/*  pos: "bottom" (default) | "top" | "topLeft" | "topRight"       */
type SubSeg = { text: string; fadeIn: number; fadeOut: number; pos?: "top" | "bottom" | "topLeft" | "topRight" };
const SUBTITLES: SubSeg[][] = [
    [{ text: "Presented by Carnelia — the Merkle-Delta CRDT Store.", fadeIn: 30, fadeOut: 150 }],
  /* 1  Coordination Bottleneck */    [
    { text: "Meet BirdWatch, the future of social media. Watcher 302 posts a photo of a falcon, and it goes viral.", fadeIn: 10, fadeOut: 180 },
    { text: "To handle this traffic, we scale out, adding dozens of servers to our cluster.", fadeIn: 200, fadeOut: 370 },
    { text: "But the click count is now split across all these nodes. Your server doesn't know the total.", fadeIn: 370, fadeOut: 520, pos: "top" },
    { text: "This is coordination. It is slow, it is fragile, and latency gets exponentially worse.", fadeIn: 520, fadeOut: 680, pos: "top" },
  ],
  /* 2  CRDT Breakthrough */    [
    { text: "Users don't need the perfect global total instantly — they just need immediate feedback.", fadeIn: 10, fadeOut: 150 },
    { text: "CRDTs — Conflict-free Replicated Data Types — break the deadlock.", fadeIn: 155, fadeOut: 300, pos: "top" },
    { text: "Every node accepts updates locally and instantly. They gossip in the background.", fadeIn: 300, fadeOut: 420, pos: "top" },
    { text: "Even with delays, duplication, or reordering, the mathematics guarantee convergence.", fadeIn: 420, fadeOut: 480, pos: "top" },
  ],
  /* 2  Scaling */      [
    { text: "We need to scale. We add more servers so clients can connect to any node.", fadeIn: 10, fadeOut: 140, pos: "top" },
    { text: "Each node maintains a local view of that click count. When a user clicks, the local node updates.", fadeIn: 140, fadeOut: 300, pos: "top" },
    { text: "In a traditional system, we stop everything. We coordinate. This coordination is slow and kills performance.", fadeIn: 300, fadeOut: 470, pos: "top" },
  ],
  /* 3  CRDT Solution */[
    { text: "In BirdWatch, users don't need the exact global truth instantly — they just need immediate feedback.", fadeIn: 10, fadeOut: 140 },
    { text: "CRDTs, Conflict-free Replicated Data Types, change the game.", fadeIn: 140, fadeOut: 270, pos: "top" },
    { text: "Nodes update locally and gossip in the background. Even with delays, duplication, or reordering, all nodes converge.", fadeIn: 270, fadeOut: 470, pos: "top" },
  ],
  /* 4  SemiLattice */  [
    { text: "How does this magic work? It relies on a Join Semi-Lattice.", fadeIn: 10, fadeOut: 110 },
    { text: "A one-way street always moving upward. Whether we merge A then B, or B then A, we reach the same Least Upper Bound.", fadeIn: 110, fadeOut: 270, pos: "top" },
    { text: "Merging two counters always results in a higher, unified number — never losing a single click.", fadeIn: 270, fadeOut: 380, pos: "top" },
  ],
  /* 5  GCounter */     [
    { text: "The G-Counter: a Grow-Only Counter. We store a vector — one slot per server.", fadeIn: 10, fadeOut: 130 },
    { text: "Server A receives a click? It only increments its own slot.", fadeIn: 130, fadeOut: 260, pos: "top" },
    { text: "When gossiping, they merge by taking the max of each slot. The total is the sum of all slots.", fadeIn: 260, fadeOut: 410, pos: "top" },
    { text: "Every server writes independently. The final total is mathematically correct.", fadeIn: 410, fadeOut: 480, pos: "top" },
  ],
  /* 6  Limits */       [
    { text: "However, basic CRDTs have flaws. Sending the entire vector every sync wastes bandwidth — state bloat.", fadeIn: 10, fadeOut: 140, pos: "top" },
    { text: "Deleting data requires tombstones — markers that accumulate forever, cluttering storage.", fadeIn: 140, fadeOut: 280, pos: "top" },
    { text: "This is where we need a more advanced architecture.", fadeIn: 280, fadeOut: 380, pos: "top" },
  ],
  /* 7  Enter MDCS */   [
    { text: "This brings us to MDCS — the Merkle-Delta CRDT Store.", fadeIn: 10, fadeOut: 100 },
    { text: "Instead of shipping full state, MDCS generates tiny incremental updates called deltas.", fadeIn: 100, fadeOut: 220, pos: "top" },
    { text: "Lightweight mutations, dramatically reducing the cost of synchronization.", fadeIn: 220, fadeOut: 280, pos: "top" },
  ],
  /* 8  MerkleClock */  [
    { text: "Traditional G-Counters use Vector Clocks — fragile in open networks.", fadeIn: 10, fadeOut: 130 },
    { text: "MDCS replaces them with a Merkle-Clock: an immutable DAG of hashed updates.", fadeIn: 130, fadeOut: 290, pos: "top" },
    { text: "Same hash at the head = identical history. Divergence? Sync only the missing blocks.", fadeIn: 290, fadeOut: 470, pos: "top" },
  ],
  /* 9  Tombstone */    [
    { text: "MDCS solves the 'trash' problem. Instead of tombstones, it uses a Dot Store and Causal Context.", fadeIn: 10, fadeOut: 150 },
    { text: "If a data point is missing from the active store, it is deleted. Old metadata is cleaned up automatically.", fadeIn: 150, fadeOut: 300, pos: "top" },
    { text: "Storage stays small even after millions of updates.", fadeIn: 300, fadeOut: 380, pos: "top" },
  ],
  /* 10 Delta Propagation */[
    { text: "MDCS doesn't ship the full state — it generates tiny delta mutations.", fadeIn: 10, fadeOut: 130 },
    { text: "A delta-mutator produces only the change: m(X) = X ⊔ mδ(X).", fadeIn: 130, fadeOut: 270, pos: "top" },
    { text: "Dramatically lower bandwidth — deltas are idempotent, commutative, and associative.", fadeIn: 270, fadeOut: 410, pos: "top" },
  ],
  /* 11 Offline Sync */   [
    { text: "What happens when a device goes offline? Both replicas keep editing independently.", fadeIn: 10, fadeOut: 150 },
    { text: "States diverge — a network partition. But CRDTs handle this gracefully.", fadeIn: 150, fadeOut: 290, pos: "top" },
    { text: "On reconnection, the DAG-Syncer performs bidirectional gap repair.", fadeIn: 290, fadeOut: 440, pos: "top" },
    { text: "Missing deltas are fetched by hash and applied in topological order — zero data loss.", fadeIn: 440, fadeOut: 570, pos: "top" },
  ],
  /* 12 Collab Demo */    [
    { text: "Traditional collaborative editing relies on central servers — a single point of failure.", fadeIn: 10, fadeOut: 150 },
    { text: "Carnelia uses peer-to-peer δ-CRDTs: no server needed, full offline support.", fadeIn: 150, fadeOut: 290, pos: "top" },
    { text: "Multiple editors modify JSON documents simultaneously — all changes merge conflict-free.", fadeIn: 290, fadeOut: 440, pos: "top" },
  ],
  /* 13 Real-World CRDTs */[
    { text: "CRDTs already power the tools you use every day.", fadeIn: 10, fadeOut: 130 },
    { text: "Figma, Google Docs, Apple Notes, Linear — all use convergence-based replication.", fadeIn: 130, fadeOut: 280, pos: "top" },
    { text: "The pattern: local-first writes + automatic convergence. Carnelia goes fully peer-to-peer.", fadeIn: 280, fadeOut: 390, pos: "top" },
  ],
  /* 14 Conclusion */     [
    { text: "By combining optimistic updates with the efficiency of MDCS, we get the best of both worlds.", fadeIn: 10, fadeOut: 140 },
    { text: "Partition-tolerant, offline-first, rigorously consistent — without the bloat.", fadeIn: 140, fadeOut: 270, pos: "top" },
    { text: "Your data always converges, no matter how chaotic the network gets.", fadeIn: 270, fadeOut: 380, pos: "top" },
  ],
];

/** Subtitle overlay — renders the narration text for a given scene */
const SubtitleOverlay: React.FC<{
  segments: SubSeg[];
  sceneDuration: number;
}> = ({ segments, sceneDuration }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Suppress unused vars
  void fps;

  return (
    <AbsoluteFill style={{ pointerEvents: "none" }}>
      {segments.map((seg, i) => {
        const entryDuration = Math.min(12, (seg.fadeOut - seg.fadeIn) * 0.15);
        const exitStart = Math.min(seg.fadeOut, sceneDuration - 10);
        const opacity =
          interpolate(frame, [seg.fadeIn, seg.fadeIn + entryDuration], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }) *
          interpolate(frame, [exitStart - 8, exitStart], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

        if (frame < seg.fadeIn - 1 || frame > seg.fadeOut + 1 || opacity < 0.01) return null;

        const pos = seg.pos ?? "bottom";

        /* Position styles for each placement variant */
        const posStyle: React.CSSProperties =
          pos === "top"
            ? { top: 50, left: "10%", right: "10%", textAlign: "center" as const }
            : pos === "topLeft"
              ? { top: 50, left: 40, right: "50%", textAlign: "left" as const }
              : pos === "topRight"
                ? { top: 50, left: "50%", right: 40, textAlign: "right" as const }
                : { bottom: 20, left: "10%", right: "10%", textAlign: "center" as const };

        return (
          <div
            key={i}
            style={{
              position: "absolute",
              ...posStyle,
              opacity,
            }}
          >
            <span
              style={{
                fontFamily: FONT_PRIMARY,
                fontSize: 16,
                color: "rgba(255,255,255,0.85)",
                background: "rgba(0,0,0,0.55)",
                padding: "8px 20px",
                borderRadius: 6,
                lineHeight: 1.6,
                display: "inline-block",
                maxWidth: 900,
              }}
            >
              {seg.text}
            </span>
          </div>
        );
      })}
    </AbsoluteFill>
  );
};

export const crdtExplainerSchema = z.object({});

export const CRDT_EXPLAINER_DURATION = 7210;
export const CRDT_EXPLAINER_FPS = 20;

export const CrdtExplainer: React.FC<z.infer<typeof crdtExplainerSchema>> = () => {
  /* Compute cumulative offsets for subtitle sequences */
  const offsets: number[] = [];
  let acc = 0;
  for (const d of SCENE_DURATIONS) {
    offsets.push(acc);
    acc += d;
  }

  return (
    <AbsoluteFill style={{ backgroundColor: "#1e1e1e" }}>
      {/* Background soundtrack — loops across entire composition */}
      <Audio
        src={staticFile("ambient_bg_soundtrack.mp3")}
        volume={0.10}
        startFrom={0}
      />

      <Series>
        <Series.Sequence durationInFrames={220}>
          <PresentedByScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={700}>
          <BirdWatchScene />
          {/* <Audio src={staticFile("audio/problem_narration.mp3")} volume={0.9} /> */}
        </Series.Sequence>

        {/* 2. CRDT Breakthrough (remotion-bits) */}
        <Series.Sequence durationInFrames={500}>
          <CrdtBreakthroughScene />
          {/* <Audio src={staticFile("audio/solution_intro_narration.mp3")} volume={0.9} /> */}
        </Series.Sequence>

        {/* 3. Scaling Problem (ThreeCanvas — ReplicaScene) */}
        <Series.Sequence durationInFrames={500}>
          <ReplicaScene />
          {/* <Audio src={staticFile("audio/scaling_narration.mp3")} volume={0.9} /> */}
        </Series.Sequence>

        {/* 3. CRDT Solution (ThreeCanvas — MergeScene) */}
        <Series.Sequence durationInFrames={500}>
          <MergeScene />
          {/* <Audio src={staticFile("audio/crdt_solution_narration.mp3")} volume={0.9} /> */}
        </Series.Sequence>

        {/* 4. Join Semi-Lattice (ThreeCanvas) */}
        <Series.Sequence durationInFrames={400}>
          <SemilatticeScene />
          {/* <Audio src={staticFile("audio/semilattice_narration.mp3")} volume={0.9} /> */}
        </Series.Sequence>

        {/* 5. G-Counter (remotion-bits) */}
        <Series.Sequence durationInFrames={500}>
          <GCounterScene />
          {/* <Audio src={staticFile("audio/gcounter_narration.mp3")} volume={0.9} /> */}
        </Series.Sequence>

        {/* 6. Limits of Basic CRDTs (ThreeCanvas) */}
        <Series.Sequence durationInFrames={400}>
          <LimitationsScene />
          {/* <Audio src={staticFile("audio/limitations_narration.mp3")} volume={0.9} /> */}
        </Series.Sequence>

        {/* 7. Enter MDCS (ThreeCanvas — CarneliaSolutionScene) */}
        <Series.Sequence durationInFrames={300}>
          <CarneliaSolutionScene />
          {/* <Audio src={staticFile("audio/mdcs_narration.mp3")} volume={0.9} /> */}
        </Series.Sequence>

        {/* 8. Merkle-Clock (ThreeCanvas) */}
        <Series.Sequence durationInFrames={500}>
          <MerkleScene />
          {/* <Audio src={staticFile("audio/merkle_narration.mp3")} volume={0.9} /> */}
        </Series.Sequence>

        {/* 9. Tombstone-Free (ThreeCanvas — DotStoreScene) */}
        <Series.Sequence durationInFrames={400}>
          <DotStoreScene />
          {/* <Audio src={staticFile("audio/tombstone_narration.mp3")} volume={0.9} /> */}
        </Series.Sequence>

        {/* 11. Delta Propagation (ThreeCanvas — DeltaScene) */}
        <Series.Sequence durationInFrames={430}>
          <DeltaScene />
          {/* <Audio src={staticFile("audio/delta_narration.mp3")} volume={0.9} /> */}
        </Series.Sequence>

        {/* 12. Offline Sync (ThreeCanvas — CarneliaSyncScene) */}
        <Series.Sequence durationInFrames={590}>
          <CarneliaSyncScene />
          {/* <Audio src={staticFile("audio/sync_narration.mp3")} volume={0.9} /> */}
        </Series.Sequence>

        {/* 13. Collab Demo (ThreeCanvas — CollabDemoScene) */}
        <Series.Sequence durationInFrames={460}>
          <CollabDemoScene />
          {/* <Audio src={staticFile("audio/collab_narration.mp3")} volume={0.9} /> */}
        </Series.Sequence>

        {/* 14. Real-World CRDTs (ThreeCanvas — RealWorldCrdtScene) */}
        <Series.Sequence durationInFrames={410}>
          <RealWorldCrdtScene />
          {/* <Audio src={staticFile("audio/realworld_narration.mp3")} volume={0.9} /> */}
        </Series.Sequence>

        {/* 15. Conclusion (ThreeCanvas — EndScene) */}
        <Series.Sequence durationInFrames={400}>
          <EndScene />
          {/* <Audio src={staticFile("audio/conclusion_narration.mp3")} volume={0.9} /> */}
        </Series.Sequence>
      </Series>

      {/* Subtitle overlay — each scene gets a Sequence with SubtitleOverlay */}
      {SUBTITLES.map((segments, i) => (
        <Sequence key={i} from={offsets[i]} durationInFrames={SCENE_DURATIONS[i]}>
          <SubtitleOverlay segments={segments} sceneDuration={SCENE_DURATIONS[i]} />
        </Sequence>
      ))}
    </AbsoluteFill>
  );
};
