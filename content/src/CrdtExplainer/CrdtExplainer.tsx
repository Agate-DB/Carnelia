import React from "react";
import { AbsoluteFill, Audio, Sequence, Series, staticFile, useCurrentFrame, useVideoConfig, interpolate } from "remotion";
import { z } from "zod";
import { PresentedByScene } from "./scenes/PresentedByScene";
import { IntroScene } from "./scenes/IntroScene";
import { ReplicaScene } from "./scenes/ReplicaScene";
import { SemilatticeScene } from "./scenes/SemilatticeScene";
import { DeltaScene } from "./scenes/DeltaScene";
import { MergeScene } from "./scenes/MergeScene";
import { RealWorldCrdtScene } from "./scenes/RealWorldCrdtScene";
import { LimitationsScene } from "./scenes/LimitationsScene";
import { CarneliaSolutionScene } from "./scenes/CarneliaSolutionScene";
import { DotStoreScene } from "./scenes/DotStoreScene";
import { MerkleScene } from "./scenes/MerkleScene";
import { CarneliaSyncScene } from "./scenes/CarneliaSyncScene";
import { CollabDemoScene } from "./scenes/CollabDemoScene";
import { IncrementDemoScene } from "./scenes/IncrementDemoScene";
import { EndScene } from "./scenes/EndScene";
import { FONT_PRIMARY } from "./fonts";

/**
 * CrdtExplainer — main composition
 *
 * 15 scenes at 30 fps ≈ 149.8 seconds
 *   1. Presented by Carnelia            170 frames  (5.7s)
 *   2. Intro & problem statement        210 frames  (7s)
 *   3. Optimistic replication           240 frames  (8s)
 *   4. Join-semilattice                 225 frames  (7.5s)
 *   5. δ-CRDT delta mutations           240 frames  (8s)
 *   6. Strong eventual consistency      240 frames  (8s)
 *   7. Real-world CRDT examples         300 frames  (10s)
 *   8. CRDT Limitations                 380 frames  (12.7s)
 *   9. Carnelia's Solutions             360 frames  (12s)
 *  10. Tombstone-free (dot store)       240 frames  (8s)
 *  11. Merkle-Clock DAG                 300 frames  (10s)
 *  12. Carnelia offline sync            400 frames  (13.3s)
 *  13. Collab demo (JSON + text)        410 frames  (13.7s)
 *  14. PNCounter increment demo         420 frames  (14s)
 *  15. End screen / summary             360 frames  (12s)
 *                                     ──────
 *                              total: 4495 frames (~149.8s)
 */

/* ── Scene durations ────────────────────────────────────── */
const SCENE_DURATIONS = [170, 210, 240, 225, 240, 240, 300, 380, 360, 240, 300, 400, 410, 420, 360] as const;

/* ── Subtitle segments: each mapped to a scene ────────── */
/*  pos: "bottom" (default) | "top" | "topLeft" | "topRight"       */
type SubSeg = { text: string; fadeIn: number; fadeOut: number; pos?: "top" | "bottom" | "topLeft" | "topRight" };
const SUBTITLES: SubSeg[][] = [
  /* 1  PresentedBy */  [{ text: "Presented by Carnelia — the Merkle-Delta CRDT Store.", fadeIn: 30, fadeOut: 150 }],
  /* 2  Intro */        [{ text: "How can distributed replicas update data independently, without any coordination, and still converge to exactly the same state?", fadeIn: 15, fadeOut: 130 }, { text: "This is the problem that CRDTs solve.", fadeIn: 130, fadeOut: 195 }],
  /* 3  Replicas */     [{ text: "In optimistic replication, every replica accepts writes locally — no locks, no consensus, no waiting.", fadeIn: 10, fadeOut: 120, pos: "top" }, { text: "Each node maintains a full copy and applies updates immediately. High availability, partition tolerance.", fadeIn: 120, fadeOut: 225, pos: "top" }],
  /* 4  Semilattice */  [{ text: "The mathematical foundation is a join-semilattice. Every state has a merge operation that computes the least upper bound.", fadeIn: 10, fadeOut: 120 }, { text: "This merge is commutative, associative, and idempotent — apply it in any order, any number of times.", fadeIn: 120, fadeOut: 210, pos: "top" }],
  /* 5  Delta */        [{ text: "Traditional state-based CRDTs ship the entire object every sync — expensive.", fadeIn: 10, fadeOut: 90 }, { text: "Delta CRDTs generate tiny incremental mutations. Far smaller than full state, dramatically reducing bandwidth.", fadeIn: 90, fadeOut: 170, pos: "top" }, { text: "Idempotent and commutative — they tolerate loss, duplication, and reordering.", fadeIn: 170, fadeOut: 225, pos: "top" }],
  /* 6  Merge/SEC */    [{ text: "The payoff: Strong Eventual Consistency.", fadeIn: 10, fadeOut: 80 }, { text: "Once all replicas receive the same updates — regardless of delivery order — they converge to an identical state.", fadeIn: 80, fadeOut: 170, pos: "top" }, { text: "No coordination, no consensus protocol, no conflict resolution needed.", fadeIn: 170, fadeOut: 225, pos: "top" }],
  /* 7  RealWorld */    [{ text: "CRDTs power some of the most popular collaboration tools today.", fadeIn: 10, fadeOut: 80 }, { text: "Figma, Google Docs, Apple Notes, Linear, VS Code Live Share — local-first writes with automatic convergence.", fadeIn: 80, fadeOut: 200, pos: "top" }, { text: "But most rely on a central server. Carnelia goes fully peer-to-peer.", fadeIn: 200, fadeOut: 285, pos: "top" }],
  /* 8  Limitations */  [{ text: "But traditional CRDTs have real-world problems.", fadeIn: 10, fadeOut: 60, pos: "top" }, { text: "State bloat, tombstone accumulation, vector clock fragility, transport assumptions…", fadeIn: 60, fadeOut: 180, pos: "top" }, { text: "…and network partition fragility — poor connectivity splits replica groups with no automatic repair.", fadeIn: 180, fadeOut: 310, pos: "top" }, { text: "These gaps prevent production deployment in open-membership networks.", fadeIn: 310, fadeOut: 365, pos: "top" }],
  /* 9  Solution */     [{ text: "Carnelia's Merkle-Delta architecture addresses each problem directly.", fadeIn: 10, fadeOut: 70, pos: "top" }, { text: "δ-CRDT deltas instead of full state. Dot store instead of tombstones. Merkle-Clock instead of vector clocks.", fadeIn: 70, fadeOut: 180, pos: "top" }, { text: "DAG-Syncer for reliable transport. Anti-entropy gossip for partition recovery.", fadeIn: 180, fadeOut: 280, pos: "top" }, { text: "A complete architectural synthesis.", fadeIn: 280, fadeOut: 345, pos: "top" }],
  /* 10 DotStore */     [{ text: "Many systems use tombstones — markers that say 'this was deleted' — and they accumulate forever.", fadeIn: 10, fadeOut: 100 }, { text: "MDCS uses a causal context and a dot store. Absence in the store means deleted. No tombstones, bounded metadata.", fadeIn: 100, fadeOut: 225, pos: "top" }],
  /* 11 Merkle */       [{ text: "Instead of vector clocks, the MDCS uses a Merkle-Clock — an immutable DAG of hashed updates.", fadeIn: 10, fadeOut: 110 }, { text: "Concurrent updates fork the DAG; merges rejoin it. The DAG-Syncer fetches missing blocks by content ID.", fadeIn: 110, fadeOut: 210, pos: "top" }, { text: "Same head hashes = identical causal histories. Guaranteed.", fadeIn: 210, fadeOut: 285, pos: "top" }],
  /* 12 Sync */         [{ text: "Two devices start connected, editing the same document. Then the mobile goes offline.", fadeIn: 10, fadeOut: 100 }, { text: "Both continue editing independently — the document diverges.", fadeIn: 100, fadeOut: 200, pos: "topRight" }, { text: "When connectivity returns, the anti-entropy protocol kicks in: gossip head CIDs, fetch missing blocks, apply deltas in topological order.", fadeIn: 200, fadeOut: 320, pos: "top" }, { text: "Both replicas converge — identical state, zero conflicts, no server required.", fadeIn: 320, fadeOut: 385, pos: "top" }],
  /* 13 Collab */       [{ text: "In Figma or Google Docs, every edit routes through a central server.", fadeIn: 10, fadeOut: 100 }, { text: "In Carnelia, there is no server. Three team members edit JSON config concurrently.", fadeIn: 100, fadeOut: 210, pos: "top" }, { text: "After CRDT merge: a single consistent document with zero conflicts.", fadeIn: 210, fadeOut: 300, pos: "top" }, { text: "Rich text works the same way — concurrent insertions resolve via unique position IDs.", fadeIn: 300, fadeOut: 395, pos: "top" }],
  /* 14 Increment */    [{ text: "Alice increments page_views by 5, then 3 — her local counter reads 8.", fadeIn: 10, fadeOut: 100, pos: "topLeft" }, { text: "Bob independently increments page_views by 10 and likes by 2. Neither knows about the other.", fadeIn: 100, fadeOut: 200, pos: "topRight" }, { text: "They sync — bob to alice, alice to bob. Bidirectional CRDT merge via delta exchange.", fadeIn: 200, fadeOut: 320, pos: "top" }, { text: "Both replicas converge: page_views = 10, likes = 2. Identical state, no coordination.", fadeIn: 320, fadeOut: 405, pos: "top" }],
  /* 15 End */           [{ text: "CRDTs guarantee convergence without consensus — and Carnelia makes it practical.", fadeIn: 30, fadeOut: 180 }, { text: "Open-membership, offline-first, peer-to-peer, Byzantine-tolerant.", fadeIn: 180, fadeOut: 280, pos: "topRight" }, { text: "github.com/Agate-DB/Carnelia", fadeIn: 280, fadeOut: 345, pos: "topRight" }],
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

export const CRDT_EXPLAINER_DURATION = 4495;
export const CRDT_EXPLAINER_FPS = 30;

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
        volume={0.18}
        startFrom={0}
      />

      <Series>
        <Series.Sequence durationInFrames={170}>
          <PresentedByScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={210}>
          <IntroScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={240}>
          <ReplicaScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={225}>
          <SemilatticeScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={240}>
          <DeltaScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={240}>
          <MergeScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={300}>
          <RealWorldCrdtScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={380}>
          <LimitationsScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={360}>
          <CarneliaSolutionScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={240}>
          <DotStoreScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={300}>
          <MerkleScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={400}>
          <CarneliaSyncScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={410}>
          <CollabDemoScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={420}>
          <IncrementDemoScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={360}>
          <EndScene />
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
