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
 * Scene 7 — Merkle-Clock DAG: Verifiable Causal History
 *
 * Visualizes a Merkle-DAG where each node is a content-addressed update.
 * Shows the "fork and merge" pattern: H(A) → H(B) and H(C) concurrently,
 * then H(D) merging both. Demonstrates gap repair: a new replica discovers
 * missing blocks by traversing predecessor hashes.
 *
 * Concept grounding (architecture doc §2.2):
 * - "Each node is uniquely identified by cryptographic hash (CID)"
 * - "Directed edges formed by including predecessor hashes"
 * - "DAG-Syncer traverses predecessor hashes, fetches missing blocks"
 * - "Two replicas that agree on head CIDs → identical causal history"
 *
 * AUDIO CUE: merkle_narration.mp3
 */

/** DAG Node — dodecahedron with label */
const DAGNode: React.FC<{
  position: [number, number, number];
  color: string;
  entrance: number;
  isHead?: boolean;
  glow?: number;
}> = ({ position, color, entrance, isHead = false, glow = 0.3 }) => {
  const frame = useCurrentFrame();
  const yBob = Math.sin(frame * 0.02 + position[0] * 3) * 0.03;
  const s = entrance * (isHead ? 0.28 : 0.22);
  const headPulse = isHead ? 1 + Math.sin(frame * 0.06) * 0.05 : 1;

  return (
    <group position={[position[0], position[1] + yBob * entrance, position[2]]}>
      <mesh scale={[s * headPulse, s * headPulse, s * headPulse]} rotation={[frame * 0.006, frame * 0.008, 0]}>
        <dodecahedronGeometry args={[1, 0]} />
        <meshStandardMaterial
          color={color}
          roughness={0.15}
          metalness={0.8}
          emissive={color}
          emissiveIntensity={glow}
          transparent
          opacity={entrance}
        />
      </mesh>
      {/* Wireframe overlay */}
      <mesh scale={[s * headPulse * 1.02, s * headPulse * 1.02, s * headPulse * 1.02]} rotation={[frame * 0.006, frame * 0.008, 0]}>
        <dodecahedronGeometry args={[1, 0]} />
        <meshBasicMaterial color={color} wireframe transparent opacity={entrance * 0.12} />
      </mesh>
      {/* Head indicator ring */}
      {isHead && (
        <mesh rotation={[Math.PI / 2, 0, frame * 0.01]} scale={[s * 3, s * 3, s * 3]}>
          <torusGeometry args={[0.5, 0.008, 8, 32]} />
          <meshBasicMaterial color={color} transparent opacity={entrance * 0.2} />
        </mesh>
      )}
    </group>
  );
};

/** DAG Edge — animated line flowing from child to parent */
const DAGEdge: React.FC<{
  from: [number, number, number];
  to: [number, number, number];
  color: string;
  progress: number;
}> = ({ from, to, color, progress }) => {
  if (progress < 0.01) return null;

  const dx = to[0] - from[0];
  const dy = to[1] - from[1];
  const dz = to[2] - from[2];
  const length = Math.sqrt(dx * dx + dy * dy + dz * dz) * progress;

  // Direction
  const midX = from[0] + dx * progress * 0.5;
  const midY = from[1] + dy * progress * 0.5;
  const midZ = from[2] + dz * progress * 0.5;

  const rotZ = Math.atan2(dy, dx);

  return (
    <group>
      {/* Main line */}
      <mesh position={[midX, midY, midZ]} rotation={[0, 0, rotZ]}>
        <boxGeometry args={[length, 0.018, 0.018]} />
        <meshStandardMaterial color={color} emissive={color} emissiveIntensity={0.4} transparent opacity={progress * 0.5} />
      </mesh>
      {/* Arrow dot at target end */}
      <mesh position={[from[0] + dx * progress * 0.92, from[1] + dy * progress * 0.92, from[2] + dz * progress * 0.92]}>
        <sphereGeometry args={[0.025 * progress, 8, 8]} />
        <meshStandardMaterial color={color} emissive={color} emissiveIntensity={0.8} />
      </mesh>
    </group>
  );
};

/** Gap-repair pulse — travels along an edge */
const GapRepairPulse: React.FC<{
  from: [number, number, number];
  to: [number, number, number];
  startFrame: number;
  duration: number;
}> = ({ from, to, startFrame, duration }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const localFrame = frame - startFrame;
  if (localFrame < 0 || localFrame > duration) return null;

  const t = spring({ frame: localFrame, fps, config: { damping: 20 } });
  const x = interpolate(t, [0, 1], [from[0], to[0]]);
  const y = interpolate(t, [0, 1], [from[1], to[1]]);
  const z = interpolate(t, [0, 1], [from[2], to[2]]);

  return (
    <mesh position={[x, y, z]}>
      <sphereGeometry args={[0.04, 8, 8]} />
      <meshStandardMaterial color="#6affea" emissive="#6affea" emissiveIntensity={2} />
    </mesh>
  );
};

export const MerkleScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();

  /*
   * DAG Layout:
   *        H(D)  [merge head]
   *       /    \
   *    H(B)    H(C)  [concurrent forks]
   *       \    /
   *        H(A)  [genesis]
   */
  const positions = {
    hA: [0, -1.4, 0] as [number, number, number],
    hB: [-1.8, 0, 0] as [number, number, number],
    hC: [1.8, 0, 0] as [number, number, number],
    hD: [0, 1.4, 0] as [number, number, number],
  };

  // Sequential node entrances
  const entA = spring({ frame, fps, delay: 5, config: { damping: 14 } });
  const entB = spring({ frame, fps, delay: 30, config: { damping: 14 } });
  const entC = spring({ frame, fps, delay: 40, config: { damping: 14 } });
  const entD = spring({ frame, fps, delay: 80, config: { damping: 14 } });

  // Edge animations
  const edgeAB = spring({ frame, fps, delay: 25, config: { damping: 200 } });
  const edgeAC = spring({ frame, fps, delay: 35, config: { damping: 200 } });
  const edgeBD = spring({ frame, fps, delay: 70, config: { damping: 200 } });
  const edgeCD = spring({ frame, fps, delay: 75, config: { damping: 200 } });

  // Head glow for D
  const dGlow = frame > 80
    ? interpolate(spring({ frame: frame - 80, fps, config: { damping: 12 } }), [0, 1], [0.3, 1.2])
    : 0.3;

  // Text
  const titleOpacity = interpolate(frame, [5, 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const labelOpacity = interpolate(frame, [15, 30], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const forkLabel = interpolate(frame, [50, 65], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const mergeLabel = interpolate(frame, [95, 110], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Gap repair annotation
  const gapRepairOpacity = interpolate(frame, [130, 150], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const grY = interpolate(spring({ frame, fps, delay: 130, config: { damping: 200 } }), [0, 1], [10, 0]);

  // Callout
  const calloutOpacity = interpolate(frame, [170, 190], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const calloutY = interpolate(spring({ frame, fps, delay: 170, config: { damping: 200 } }), [0, 1], [12, 0]);

  const fadeOut = interpolate(frame, [240, 270], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ backgroundColor: "#1e1e1e", opacity: fadeOut }}>
      <ThreeCanvas linear width={width} height={height}>
        <color attach="background" args={["#1e1e1e"]} />
        <ambientLight intensity={0.4} />
        <pointLight position={[5, 5, 5]} intensity={1} color="#6ea0ff" />
        <pointLight position={[-5, -3, 3]} intensity={0.5} color="#a06eff" />
        <directionalLight position={[0, 5, 5]} intensity={0.3} />

        {/* Nodes */}
        <DAGNode position={positions.hA} color="#51a877" entrance={entA} />
        <DAGNode position={positions.hB} color="#4a9eff" entrance={entB} />
        <DAGNode position={positions.hC} color="#ff6a9e" entrance={entC} />
        <DAGNode position={positions.hD} color="#c9a0ff" entrance={entD} isHead glow={dGlow} />

        {/* Edges (arrows point from child to parent = causality direction) */}
        <DAGEdge from={positions.hB} to={positions.hA} color="#4a9eff" progress={edgeAB} />
        <DAGEdge from={positions.hC} to={positions.hA} color="#ff6a9e" progress={edgeAC} />
        <DAGEdge from={positions.hD} to={positions.hB} color="#c9a0ff" progress={edgeBD} />
        <DAGEdge from={positions.hD} to={positions.hC} color="#c9a0ff" progress={edgeCD} />

        {/* Gap repair pulses — new replica fetching missing blocks */}
        <GapRepairPulse from={positions.hD} to={positions.hB} startFrame={135} duration={30} />
        <GapRepairPulse from={positions.hD} to={positions.hC} startFrame={140} duration={30} />
        <GapRepairPulse from={positions.hB} to={positions.hA} startFrame={150} duration={30} />
        <GapRepairPulse from={positions.hC} to={positions.hA} startFrame={155} duration={30} />
      </ThreeCanvas>

      <AbsoluteFill style={{ pointerEvents: "none" }}>
        {/* Title */}
        <div style={{ position: "absolute", top: 35, left: 60, opacity: titleOpacity }}>
          <span style={{ fontFamily: FONT_DISPLAY, fontSize: 22, color: "rgba(255,255,255,0.9)" }}>
            Merkle-Clock DAG
          </span>
        </div>

        {/* Node Hash Labels */}
        <div style={{ position: "absolute", left: "47%", bottom: "16%", opacity: labelOpacity }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "#51a877" }}>H(A)</span>
        </div>
        <div style={{ position: "absolute", left: "22%", top: "43%", opacity: interpolate(frame, [28, 40], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }) }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "#4a9eff" }}>H(B)</span>
        </div>
        <div style={{ position: "absolute", right: "22%", top: "43%", opacity: interpolate(frame, [38, 50], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }) }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "#ff6a9e" }}>H(C)</span>
        </div>
        <div style={{ position: "absolute", left: "46%", top: "17%", opacity: mergeLabel }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 14, color: "#c9a0ff" }}>H(D) — head</span>
        </div>

        {/* "Concurrent" fork annotation */}
        <div style={{ position: "absolute", right: 60, top: "30%", opacity: forkLabel }}>
          <div style={{
            fontFamily: FONT_PRIMARY,
            fontSize: 12,
            color: "rgba(255,255,255,0.5)",
            padding: "6px 12px",
            border: "1px solid rgba(255,255,255,0.08)",
            borderRadius: 6,
            background: "rgba(255,255,255,0.02)",
          }}>
            concurrent forks
          </div>
        </div>

        {/* Gap repair annotation */}
        <div style={{ position: "absolute", left: 60, top: "30%", opacity: gapRepairOpacity, transform: `translateY(${grY}px)` }}>
          <div style={{
            fontFamily: FONT_PRIMARY,
            fontSize: 13,
            color: "#6affea",
            padding: "8px 14px",
            border: "1px solid rgba(106,255,234,0.2)",
            borderRadius: 6,
            background: "rgba(106,255,234,0.04)",
          }}>
            DAG-Syncer: gap repair
            <br />
            <span style={{ fontSize: 11, opacity: 0.6 }}>
              fetch missing blocks by CID
            </span>
          </div>
        </div>

        {/* Bottom callout */}
        <div style={{
          position: "absolute",
          bottom: 45,
          left: 0,
          right: 0,
          textAlign: "center",
          opacity: calloutOpacity,
          transform: `translateY(${calloutY}px)`,
        }}>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 18, color: "white", margin: 0 }}>
            Content-addressed, immutable, verifiable history
          </p>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.4)", marginTop: 6 }}>
            Same head CIDs → guaranteed identical causal history
          </p>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
