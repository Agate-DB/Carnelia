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
import { Phase1CutScene } from "./scenes/Phase1CutScene";
import { Phase2CutScene } from "./scenes/Phase2CutScene";
import { Phase3CutScene } from "./scenes/Phase3CutScene";
import { FONT_PRIMARY } from "./fonts";
import { PresentedByScene } from "./scenes/PresentedByScene";

/**
 * CrdtExplainer — main composition (v4: BirdWatch narrative + Phase cutscenes)
 *
 * 19 scenes at 20 fps — hybrid: ThreeCanvas + remotion-bits
 *   0. Presented By (prologue)              220 frames  (11s)
 *   1. Coordination Bottleneck              700 frames  (35s)
 *   2. CRDT Breakthrough                    620 frames  (31s)
 *   3. Scaling Problem                      510 frames  (25.5s)
 *  ── Phase 1: Understanding CRDTs ──
 *   4. Phase 1 Cutscene                     120 frames  (10s)
 *   5. CRDT Solution                        500 frames  (25s)
 *   6. Join Semi-Lattice                    600 frames  (30s)
 *   7. G-Counter                            500 frames  (25s)
 *   8. Limits of Basic CRDTs               400 frames  (20s)
 *  ── Phase 2: Our Solution — The MDCS ──
 *   9. Phase 2 Cutscene                     120 frames  (6s)
 *  10. Enter MDCS                           300 frames  (15s)
 *  11. Merkle-Clock                         500 frames  (25s)
 *  12. Tombstone-Free                       400 frames  (20s)
 *  13. Delta Propagation                    430 frames  (21.5s)
 *  ── Phase 3: Real-World Applications ──
 *  14. Phase 3 Cutscene                     120 frames  (6s)
 *  15. Offline Sync                         590 frames  (29.5s)
 *  16. Collab Demo                          460 frames  (23s)
 *  17. Real-World CRDTs                     410 frames  (20.5s)
 *  18. Conclusion                           400 frames  (20s)
 *                                         ──────
 *                                  total: 7900 frames (395s / ~6:35)
 */

/* ── Scene durations ────────────────────────────────────── */
const SCENE_DURATIONS = [
  /* 0*/  220,
  /* 1*/  700,
  /* 2*/  620,
  /* 3*/  510,
  /* 4*/  200,
  /* 5*/  500,
  /* 6*/  580,
  /* 7*/  580,
  /* 8*/  460,
  /* 9*/  280,
  /* 10*/ 340,
  /* 11*/ 500,
  /* 12*/ 480,
  /* 13*/ 520,
  /* 14*/ 150,
  /* 15*/ 590,
  /* 16*/ 460,
  /* 17*/ 410,
  /* 18*/ 400,
] as const;

/* ── Subtitle segments: each mapped to a scene ────────── */
/*  pos: "bottom" (default) | "top" | "topLeft" | "topRight"       */
type SubSeg = { text: string; fadeIn: number; fadeOut: number; pos?: "top" | "bottom" | "topLeft" | "topRight" };
const SUBTITLES: SubSeg[][] = [
  /* 0 intro */ [
    { text: "Presented by Carnelia — the Merkle-Delta CRDT Store.", fadeIn: 30, fadeOut: 150 }
  ],
  /* 1  Coordination Bottleneck */    [
    { text: "Meet BirdWatch, the future of social media. Watcher 302 posts a photo of a falcon, and it goes viral.", fadeIn: 10, fadeOut: 180 },
    { text: "To handle this traffic, we scale out, adding dozens of servers to our cluster.", fadeIn: 200, fadeOut: 370 },
    { text: "But the click count is now split across all these nodes. Your server doesn't know the total.", fadeIn: 370, fadeOut: 520 },
    { text: "This is coordination. It is slow, it is fragile, and latency gets exponentially worse.", fadeIn: 520, fadeOut: 680 },
  ],
  /* 2  CRDT Breakthrough */    [
    { text: "Users don't need the perfect 'global total' instantly , they just need immediate feedback.", fadeIn: 10, fadeOut: 150 },
    { text: "This is where Conflict-free Replicated Data Types break the deadlock.", fadeIn: 155, fadeOut: 300},
    { text: "Every node accepts updates locally and instantly. They would  gossip in the background.", fadeIn: 300, fadeOut: 420 },
    { text: "Even with delays, duplication, or reordering, the CRDT's guarantee eventual convergence.", fadeIn: 420, fadeOut: 600 },
  ],
  /* 3  Scaling */      [
    { text: "Soon We Would need to scale. We add more servers so clients can connect to any node.", fadeIn: 10, fadeOut: 150, pos: "top" },
    { text: "Each node maintains a local view of that click count. When a user clicks, the local node updates.", fadeIn: 150, fadeOut: 310, pos: "top" },
    { text: "In a traditional system, we stop everything. We coordinate. This coordination is slow and kills performance.", fadeIn: 310, fadeOut: 480, pos: "top" },
  ],
  // phase 1 cutscene
  /* 4  Phase 1 */      [
    { text: "Phase 1 — Understanding CRDTs: how conflict-free replicated data types enable coordination-free systems.", fadeIn: 10, fadeOut: 190 },
  ],
  /* 5  CRDT Solution */[
    { text: "In BirdWatch, users don't need the exact global truth instantly. They just need immediate feedback.", fadeIn: 10, fadeOut: 200 },
    { text: "In such scenarios, CRDT's can change the game.", fadeIn: 200, fadeOut: 280, pos: "top" },
    { text: "Nodes update locally and gossip in the background. Even with delays, duplication, or reordering, all nodes converge.", fadeIn: 280, fadeOut: 470, pos: "top" },
  ],
  /* 6  SemiLattice */  [
    { text: "How does this magic work? It relies on Join Semi-Lattices.", fadeIn: 10, fadeOut: 100 },
    { text: "Imagine a one-way street always moving upward. Whether we merge A then B, or B then A, we eventually reach the same Least Upper Bound.", fadeIn: 100, fadeOut: 370, pos: "top" },
  { text: "Merging two counters always results in a higher, unified number and never loses a single click.", fadeIn: 370, fadeOut: 580, pos: "top" },
  ],
  /* 7  GCounter */     [
    { text: "The G-Counter: a Grow-Only Counter. We store a single vector per replica.", fadeIn: 10, fadeOut: 130 },
    { text: "When Replica A receives a click, It increments its own slot.", fadeIn: 130, fadeOut: 260 },
    { text: "While gossiping, they merge by taking the maximum of each slot. The total is the sum of all slots.", fadeIn: 260, fadeOut: 410 },
    { text: "Every replica writes independently. The final total is mathematically correct.", fadeIn: 410, fadeOut: 560 },
  ],
  /* 8  Limits */       [
    { text: "However, basic CRDTs have flaws. Sending the entire vector every sync wastes bandwidth, ie. state bloat.", fadeIn: 10, fadeOut: 240, pos: "top" },
    { text: "Deleting data requires tombstones - markers that accumulate forever, cluttering storage.", fadeIn: 240, fadeOut: 340, pos: "top" },
    { text: "This is where we need a more advanced algorithm.", fadeIn: 340, fadeOut: 460, pos: "top" },
  ],
  // phase 2 cutscene
  /* 9  Phase 2 */      [
    { text: "Phase 2 Our Solution: the Merkle-Delta CRDT Store: lock-free, offline-first, open-membership database that systematically addresses the gaps in current CRDT systems.", fadeIn: 10, fadeOut: 260 },
  ],
  /* 10 Enter MDCS */   [
    { text: "This brings us to using Delta Stores.", fadeIn: 10, fadeOut: 100 },
    { text: "Instead of shipping full states, we generate tiny incremental updates called deltas.", fadeIn: 100, fadeOut: 220, pos: "top" },
    { text: "These Lightweight mutations dramatically reduces the cost of synchronization.", fadeIn: 220, fadeOut: 320, pos: "top" },
  ],
  /* 11 MerkleClock */  [
    { text: "Traditional G-Counters use Vector Clocks which can be fragile in open networks.", fadeIn: 10, fadeOut: 130 },
    { text: "We replace them with a Merkle-Clock: an immutable DAG (Directed Acyclic Graph) of hashed updates.", fadeIn: 130, fadeOut: 290, pos: "top" },
    { text: "The same hash at the head equates identical history. We only need to sync the missing blocks.", fadeIn: 290, fadeOut: 470, pos: "top" },
  ],
  /* 12 Tombstone */    [
    { text: "We also solve the 'trash' problem. Instead of using tombstones, we use a Dot Store and Causal Context.", fadeIn: 10, fadeOut: 150 },
    { text: "If a data point is missing from the active store, it is deleted. Old metadata is cleaned up automatically.", fadeIn: 150, fadeOut: 340, pos: "top" },
    { text: "Our storage stays small even after millions of updates.", fadeIn: 340, fadeOut: 460, pos: "top" },
  ],
  /* 13 Delta Propagation */[
    { text: "Here we don't ship the full state we only send the tiny delta mutations.", fadeIn: 10, fadeOut: 100 },
    { text: "A delta-mutator produces only the change: m(X) = X ⊔ mδ(X).", fadeIn: 100, fadeOut: 370, pos: "top" },
    { text: "We dramatically lower bandwidth since the deltas are idempotent, commutative, and associative.", fadeIn: 370, fadeOut: 520, pos: "top" },
  ],
  // phase 3 cutscene
  /* 14 Phase 3 */      [
    { text: "Phase 3 — Real-World Applications: scenarios where Carnelia solves distributed challenges.", fadeIn: 10, fadeOut: 140 },
  ],
  /* 15 Offline Sync */   [
    { text: "What happens when a device goes offline? Both replicas keep editing independently.", fadeIn: 10, fadeOut: 150 },
    { text: "States can diverge, Data can fall out of syncronization. But we handle this gracefully.", fadeIn: 150, fadeOut: 290, pos: "top" },
    { text: "On reconnection, the DAG-Syncer performs bidirectional gap repair.", fadeIn: 290, fadeOut: 400, pos: "top" },
    { text: "Missing deltas are fetched by hash and applied in topological order leading to zero data loss.", fadeIn: 400, fadeOut: 570, pos: "top" },
  ],
  /* 16 Collab Demo */    [
    { text: "Traditional collaborative editing relies on central servers which can be a single point of failure.", fadeIn: 10, fadeOut: 150 },
    { text: "Carnelia uses peer-to-peer δ-CRDTs: no server needed with full offline support.", fadeIn: 150, fadeOut: 290 },
    { text: "Multiple editors modify JSON documents simultaneously — all changes merge conflict-free.", fadeIn: 290, fadeOut: 440 },
  ],
  /* 17 Real-World CRDTs */[
    { text: "CRDTs already power the tools you use every day.", fadeIn: 10, fadeOut: 130 },
    { text: "Figma, Google Docs, Apple Notes, Linear — all use convergence-based replication.", fadeIn: 130, fadeOut: 260, pos: "top" },
    { text: "The pattern: local-first writes + automatic convergence. Carnelia goes fully peer-to-peer.", fadeIn: 260, fadeOut: 390, pos: "top" },
  ],
  /* 18 Conclusion */     [
    { text: "By combining optimistic updates with the efficiency of MDCS, we get the best of both worlds.", fadeIn: 10, fadeOut: 140 },
    { text: "Partition-tolerant, offline-first, rigorously consistent — without the bloat.", fadeIn: 140, fadeOut: 270, pos: "top" },
    { text: "Your data always converges, no matter how chaotic the distribution gets.", fadeIn: 270, fadeOut: 380, pos: "top" },
  ],
];

export const crdtExplainerSchema = z.object({});

export const CRDT_EXPLAINER_DURATION = SCENE_DURATIONS.reduce((a, b) => a + b, 0);
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
        volume={0.05}
        startFrom={0}
      />

      <Series>
        <Series.Sequence durationInFrames={SCENE_DURATIONS[0]}>
          <PresentedByScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={SCENE_DURATIONS[1]}>
          <BirdWatchScene />
          <Audio src={staticFile("audio/problem_narration.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 2. CRDT Breakthrough (remotion-bits) */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[2]}>
          <CrdtBreakthroughScene />
          <Audio src={staticFile("audio/solution_intro_narration.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 3. Scaling Problem (ThreeCanvas — ReplicaScene) */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[3]}>
          <ReplicaScene />
          <Audio src={staticFile("audio/scaling_narration.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 4. Phase 1 — Understanding CRDTs */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[4]}>
          <Phase1CutScene />
          <Audio src={staticFile("audio/phase1.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 5. CRDT Solution (ThreeCanvas — MergeScene) */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[5]}>
          <MergeScene />
          <Audio src={staticFile("audio/crdt_solution_narration.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 6. Join Semi-Lattice (ThreeCanvas) */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[6]}>
          <SemilatticeScene />
          <Audio src={staticFile("audio/semilattice_narration.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 7. G-Counter (remotion-bits) */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[7]}>
          <GCounterScene />
          <Audio src={staticFile("audio/gcounter_narration.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 8. Limits of Basic CRDTs (ThreeCanvas) */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[8]}>
          <LimitationsScene />
          <Audio src={staticFile("audio/limitations_narration.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 9. Phase 2 — Our Solution: The MDCS */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[9]}>
          <Phase2CutScene />
          <Audio src={staticFile("audio/phase2.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 10. Enter MDCS (ThreeCanvas — CarneliaSolutionScene) */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[10]}>
          <CarneliaSolutionScene />
          <Audio src={staticFile("audio/mdcs_narration.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 11. Merkle-Clock (ThreeCanvas) */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[11]}>
          <MerkleScene />
          <Audio src={staticFile("audio/merkle_narration.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 12. Tombstone-Free (ThreeCanvas — DotStoreScene) */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[12]}>
          <DotStoreScene />
          <Audio src={staticFile("audio/tombstone_narration.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 13. Delta Propagation (ThreeCanvas — DeltaScene) */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[13]}>
          <DeltaScene />
          <Audio src={staticFile("audio/delta_narration.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 14. Phase 3 — Real-World Applications */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[14]}>
          <Phase3CutScene />
          <Audio src={staticFile("audio/phase3.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 15. Offline Sync (ThreeCanvas — CarneliaSyncScene) */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[15]}>
          <CarneliaSyncScene />
          <Audio src={staticFile("audio/sync_narration.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 16. Collab Demo (ThreeCanvas — CollabDemoScene) */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[16]}>
          <CollabDemoScene />
          <Audio src={staticFile("audio/collab_narration.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 17. Real-World CRDTs (ThreeCanvas — RealWorldCrdtScene) */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[17]}>
          <RealWorldCrdtScene />
          <Audio src={staticFile("audio/realworld_narration.mp3")} volume={0.9} />
        </Series.Sequence>

        {/* 18. Conclusion (ThreeCanvas — EndScene) */}
        <Series.Sequence durationInFrames={SCENE_DURATIONS[18]}>
          <EndScene />
          <Audio src={staticFile("audio/conclusion_narration.mp3")} volume={0.9} />
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