import React from "react";
import { ThreeCanvas } from "@remotion/three";
import {
  AbsoluteFill,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { FONT_PRIMARY, FONT_DISPLAY } from "../fonts";

/**
 * Scene — Carnelia Offline Sync & Anti-Entropy
 *
 * Three-phase visualization showing how Carnelia handles:
 *   Phase 1: Two replicas editing independently (one goes offline)
 *   Phase 2: Both replicas diverge with local edits
 *   Phase 3: Reconnection → DAG-Syncer gap repair → convergence
 *
 * References the offline_sync.rs example and network_simulation.rs
 *
 * Concept grounding (architecture doc §4.0):
 * - "DAG-Syncer: pull-based reconciliation"
 * - "Broadcaster gossips head CIDs"
 * - "traverses backward via hash-based requests"
 * - "common ancestor → fetch missing → apply in topological order"
 *
 * AUDIO CUE: carnelia_sync_narration.mp3
 */

/** A replica device node: mobile or desktop */
const DeviceNode: React.FC<{
  position: [number, number, number];
  entrance: number;
  color: string;
  type: "mobile" | "desktop";
  isOffline?: boolean;
}> = ({ position, entrance, color, type, isOffline = false }) => {
  const frame = useCurrentFrame();
  const s = entrance * (type === "mobile" ? 0.2 : 0.28);
  const yBob = Math.sin(frame * 0.02 + position[0] * 3) * 0.03;
  const offlinePulse = isOffline ? Math.sin(frame * 0.08) * 0.15 + 0.4 : 0.5;

  return (
    <group position={[position[0], position[1] + yBob * entrance, position[2]]}>
      {/* Device body */}
      <mesh scale={type === "mobile" ? [s * 0.5, s * 1, s * 0.1] : [s * 1.2, s * 0.8, s * 0.1]} rotation={[0.1, frame * 0.003, 0]}>
        <boxGeometry args={[10, 10, 10]} />
        <meshStandardMaterial
          color={color}
          roughness={0.2}
          metalness={0.7}
          emissive={color}
          emissiveIntensity={isOffline ? offlinePulse : 0.5}
          transparent
          opacity={entrance * (isOffline ? 0.5 : 0.85)}
        />
      </mesh>
      {/* Screen area */}
      <mesh scale={type === "mobile" ? [s * 0.4, s * 0.7, s * 0.105] : [s * 1.0, s * 0.6, s * 0.105]} rotation={[0.1, frame * 0.003, 0]}>
        <boxGeometry args={[10, 10, 10]} />
        <meshStandardMaterial
          color={isOffline ? "#333333" : "#1a1a3a"}
          emissive={isOffline ? "#331111" : "#1a1a4a"}
          emissiveIntensity={isOffline ? 0.2 : 0.5}
          transparent
          opacity={entrance * 0.9}
        />
      </mesh>
    </group>
  );
};

/** Sync Beam — animated beam connecting two nodes during sync */
const SyncBeam: React.FC<{
  fromX: number;
  toX: number;
  y: number;
  progress: number;
  color: string;
}> = ({ fromX, toX, y, progress, color }) => {
  const frame = useCurrentFrame();
  if (progress < 0.01) return null;

  const len = (toX - fromX) * progress;
  const midX = fromX + len * 0.5;
  const particleCount = 4;

  return (
    <group>
      {/* Main beam */}
      <mesh position={[midX, y, 0.1]}>
        <boxGeometry args={[Math.abs(len), 0.015, 0.015]} />
        <meshStandardMaterial color={color} emissive={color} emissiveIntensity={0.8} transparent opacity={progress * 0.5} />
      </mesh>
      {/* Traveling particles */}
      {Array.from({ length: particleCount }).map((_, i) => {
        const t = ((frame * 0.02 + i / particleCount) % 1);
        const x = fromX + (toX - fromX) * t;
        return (
          <mesh key={i} position={[x, y, 0.15]} scale={[0.02 * progress, 0.02 * progress, 0.02 * progress]}>
            <sphereGeometry args={[1, 8, 8]} />
            <meshBasicMaterial color={color} transparent opacity={progress * 0.7} />
          </mesh>
        );
      })}
    </group>
  );
};

/** Delta blocks — small cubes representing delta payloads */
const DeltaBlocks: React.FC<{
  position: [number, number, number];
  count: number;
  entrance: number;
  color: string;
}> = ({ position, count, entrance, color }) => {
  const frame = useCurrentFrame();
  return (
    <group position={position}>
      {Array.from({ length: count }).map((_, i) => {
        const x = (i - (count - 1) / 2) * 0.12;
        const yBob = Math.sin(frame * 0.03 + i * 1.5) * 0.03;
        const s = entrance * 0.05;
        return (
          <mesh key={i} position={[x, yBob, 0]} scale={[s, s, s]} rotation={[0, frame * 0.008 + i, 0]}>
            <octahedronGeometry args={[1, 0]} />
            <meshStandardMaterial
              color={color}
              emissive={color}
              emissiveIntensity={0.6}
              transparent
              opacity={entrance * 0.8}
            />
          </mesh>
        );
      })}
    </group>
  );
};

export const CarneliaSyncScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();

  // Phase 1: Both online (frames 0–80)
  const mobileEnt = spring({ frame, fps, delay: 5, config: { damping: 14 } });
  const desktopEnt = spring({ frame, fps, delay: 15, config: { damping: 14 } });

  // Phase 2: Mobile goes offline (frames 80–200)
  const isOffline = frame >= 80 && frame < 250;
  const mobileEdits = interpolate(frame, [90, 150], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const desktopEdits = interpolate(frame, [110, 170], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Phase 3: Reconnection and sync (frames 250–380)
  const syncLeft = spring({ frame, fps, delay: 270, config: { damping: 200 } });
  const syncRight = spring({ frame, fps, delay: 285, config: { damping: 200 } });
  const convergenceFlash = frame >= 310 ? spring({ frame: frame - 310, fps, config: { damping: 6, stiffness: 200 } }) : 0;

  const titleOpacity = interpolate(frame, [5, 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Phase labels
  const phase1 = interpolate(frame, [5, 20, 75, 85], [0, 1, 1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const phase2 = interpolate(frame, [80, 95, 240, 250], [0, 1, 1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const phase3 = interpolate(frame, [250, 265], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Result callout
  const resultOpacity = interpolate(frame, [325, 345], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const resultY = interpolate(spring({ frame, fps, delay: 325, config: { damping: 200 } }), [0, 1], [12, 0]);

  const fadeOut = interpolate(frame, [560, 590], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ backgroundColor: "#1e1e1e", opacity: fadeOut }}>
      <ThreeCanvas linear width={width} height={height}>
        <color attach="background" args={["#1e1e1e"]} />
        <ambientLight intensity={0.4} />
        <pointLight position={[5, 5, 5]} intensity={0.8} color="#6ea0ff" />
        <pointLight position={[-5, -3, 3]} intensity={0.5} color="#a06eff" />

        {/* Mobile device */}
        <DeviceNode position={[-2.5, 0.3, 0]} entrance={mobileEnt} color="#4a9eff" type="mobile" isOffline={isOffline} />
        {/* Desktop device */}
        <DeviceNode position={[2.5, 0.3, 0]} entrance={desktopEnt} color="#ff6a9e" type="desktop" />

        {/* Mobile's local edits (phase 2) */}
        <DeltaBlocks position={[-2.5, -0.6, 0]} count={3} entrance={mobileEdits} color="#4a9eff" />
        {/* Desktop's local edits (phase 2) */}
        <DeltaBlocks position={[2.5, -0.6, 0]} count={2} entrance={desktopEdits} color="#ff6a9e" />

        {/* Sync beams (phase 3) */}
        <SyncBeam fromX={-2.0} toX={2.0} y={0.3} progress={syncLeft} color="#6eff9e" />
        <SyncBeam fromX={2.0} toX={-2.0} y={0.0} progress={syncRight} color="#6affea" />

      </ThreeCanvas>

      <AbsoluteFill style={{ pointerEvents: "none" }}>
        {/* Title */}
        <div style={{ position: "absolute", top: 35, left: 60, opacity: titleOpacity }}>
          <span style={{ fontFamily: FONT_DISPLAY, fontSize: 26, color: "#e06040" }}>
            Carnelia: Offline Sync & Anti-Entropy
          </span>
        </div>

        {/* Device labels */}
        <div style={{ position: "absolute", left: "12%", top: "54%", opacity: mobileEnt }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 16, color: "#4a9eff" }}>
            Mobile {isOffline ? "📵" : "📶"}
          </span>
        </div>
        <div style={{ position: "absolute", right: "12%", top: "54%", opacity: desktopEnt }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 16, color: "#ff6a9e" }}>
            Desktop 📶
          </span>
        </div>

        {/* Phase 1 indicator */}
        <div style={{ position: "absolute", top: 80, right: 50, opacity: phase1 }}>
          <div style={{
            fontFamily: FONT_PRIMARY, fontSize: 17, color: "#6eff9e",
            padding: "8px 16px", border: "1px solid rgba(110,255,158,0.2)",
            borderRadius: 8, background: "rgba(110,255,158,0.04)",
          }}>
            Phase 1: Both Online
          </div>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.4)", marginTop: 6, maxWidth: 240, lineHeight: 1.5 }}>
            Initial state synced via CRDT merge — both replicas identical
          </p>
        </div>

        {/* Phase 2 indicator */}
        <div style={{ position: "absolute", top: 80, right: 50, opacity: phase2 }}>
          <div style={{
            fontFamily: FONT_PRIMARY, fontSize: 17, color: "#ff6a6a",
            padding: "8px 16px", border: "1px solid rgba(255,106,106,0.2)",
            borderRadius: 8, background: "rgba(255,106,106,0.04)",
          }}>
            Phase 2: Network Partition
          </div>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.4)", marginTop: 6, maxWidth: 240, lineHeight: 1.5 }}>
            Mobile goes offline — both devices keep editing locally. States diverge.
          </p>
          <div style={{
            fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.3)",
            marginTop: 8, padding: "6px 10px", border: "1px solid rgba(255,255,255,0.06)",
            borderRadius: 6, background: "rgba(255,255,255,0.02)",
          }}>
            <div style={{ color: "#4a9eff" }}>Mobile: +3 items (offline)</div>
            <div style={{ color: "#ff6a9e", marginTop: 3 }}>Desktop: +2 items (online)</div>
          </div>
        </div>

        {/* Phase 3 indicator */}
        <div style={{ position: "absolute", top: 80, right: 50, opacity: phase3 }}>
          <div style={{
            fontFamily: FONT_PRIMARY, fontSize: 17, color: "#6affea",
            padding: "8px 16px", border: "1px solid rgba(106,255,234,0.2)",
            borderRadius: 8, background: "rgba(106,255,234,0.04)",
          }}>
            Phase 3: Reconnection & Merge
          </div>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.4)", marginTop: 6, maxWidth: 240, lineHeight: 1.5 }}>
            DAG-Syncer performs bidirectional gap repair — replicas exchange missing deltas
          </p>
          <div style={{
            fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.3)",
            marginTop: 8, padding: "6px 10px", border: "1px solid rgba(255,255,255,0.06)",
            borderRadius: 6, background: "rgba(255,255,255,0.02)", lineHeight: 1.6,
          }}>
            <div>→ Mobile sends state to Desktop</div>
            <div>← Desktop sends state to Mobile</div>
            <div style={{ color: "#6eff9e", marginTop: 4 }}>✓ CRDT merge: commutative, idempotent</div>
          </div>
        </div>

        {/* Anti-entropy protocol box */}
        <div style={{
          position: "absolute", left: 50, bottom: 120, opacity: phase3,
          background: "rgba(255,255,255,0.03)", border: "1px solid rgba(255,255,255,0.08)",
          borderRadius: 10, padding: "10px 16px", maxWidth: 300,
        }}>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.5)", margin: 0, marginBottom: 4 }}>Anti-Entropy Protocol</p>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 14, color: "#6affea", margin: 0, lineHeight: 1.6 }}>
            1. Gossip head CIDs to peers<br />
            2. Compare against local DAG<br />
            3. Fetch missing blocks by hash<br />
            4. Apply deltas in topological order
          </p>
        </div>

        {/* Result callout */}
        <div style={{
          position: "absolute", bottom: 35, left: 0, right: 0, textAlign: "center",
          opacity: resultOpacity, transform: `translateY(${resultY}px)`,
        }}>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 22, color: "#6eff9e", margin: 0 }}>
            ✓ All replicas converged — zero data loss
          </p>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 14, color: "rgba(255,255,255,0.35)", marginTop: 6 }}>
            Both devices' edits preserved automatically via CRDT merge semantics
          </p>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
