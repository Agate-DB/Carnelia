import React from "react";
import { AbsoluteFill, Series } from "remotion";
import { z } from "zod";
import { PresentedByScene } from "./scenes/PresentedByScene";
import { IntroScene } from "./scenes/IntroScene";
import { ReplicaScene } from "./scenes/ReplicaScene";
import { SemilatticeScene } from "./scenes/SemilatticeScene";
import { DeltaScene } from "./scenes/DeltaScene";
import { MergeScene } from "./scenes/MergeScene";
import { LimitationsScene } from "./scenes/LimitationsScene";
import { CarneliaSolutionScene } from "./scenes/CarneliaSolutionScene";
import { DotStoreScene } from "./scenes/DotStoreScene";
import { MerkleScene } from "./scenes/MerkleScene";

/**
 * CrdtExplainer — main composition
 *
 * 10 scenes at 30 fps ≈ 83.5 seconds
 *   1. Presented by Carnelia            170 frames  (5.7s)
 *   2. Intro & problem statement        210 frames  (7s)
 *   3. Optimistic replication           240 frames  (8s)
 *   4. Join-semilattice                 225 frames  (7.5s)
 *   5. δ-CRDT delta mutations           240 frames  (8s)
 *   6. Strong eventual consistency      240 frames  (8s)
 *   7. CRDT Limitations                 320 frames  (10.7s)
 *   8. Carnelia's Solutions             320 frames  (10.7s)
 *   9. Tombstone-free (dot store)       240 frames  (8s)
 *   10. Merkle-Clock DAG               300 frames  (10s)
 *                                     ──────
 *                              total: 2505 frames (~83.5s)
 */

export const crdtExplainerSchema = z.object({});

export const CRDT_EXPLAINER_DURATION = 2505;
export const CRDT_EXPLAINER_FPS = 30;

export const CrdtExplainer: React.FC<z.infer<typeof crdtExplainerSchema>> = () => {
  return (
    <AbsoluteFill style={{ backgroundColor: "#0a0a1a" }}>
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

        <Series.Sequence durationInFrames={320}>
          <LimitationsScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={320}>
          <CarneliaSolutionScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={240}>
          <DotStoreScene />
        </Series.Sequence>

        <Series.Sequence durationInFrames={300}>
          <MerkleScene />
        </Series.Sequence>
      </Series>
    </AbsoluteFill>
  );
};
