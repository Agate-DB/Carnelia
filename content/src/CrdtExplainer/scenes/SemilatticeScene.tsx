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
 * Scene 3 — Join-Semilattice: the mathematical foundation
 *
 * Visualizes 3D lattice structure. Two divergent state cubes move
 * upward along separate paths and merge at a "least upper bound" (LUB).
 * Shows the join operation is commutative, associative, and idempotent.
 *
 * Concept grounding (analysis report §3):
 * - "States form a join-semilattice"
 * - "merge function computes the least upper bound"
 * - "commutative, associative, idempotent"
 *
 * AUDIO CUE: semilattice_narration.mp3
 */

/** A 3D lattice node (rounded cube) */
const LatticeNode: React.FC<{
  position: [number, number, number];
  color: string;
  label: string;
  entrance: number;
  glow?: number;
  scale?: number;
}> = ({ position, color, entrance, glow = 0.3, scale = 0.28 }) => {
  const frame = useCurrentFrame();
  const yBob = Math.sin(frame * 0.02) * 0.03 * entrance;
  const s = entrance * scale;

  return (
    <group position={[position[0], position[1] + yBob, position[2]]}>
      <mesh scale={[s, s, s]} rotation={[0.3, frame * 0.005, 0.2]}>
        <boxGeometry args={[1, 1, 1]} />
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
      <mesh scale={[s * 1.02, s * 1.02, s * 1.02]} rotation={[0.3, frame * 0.005, 0.2]}>
        <boxGeometry args={[1, 1, 1]} />
        <meshBasicMaterial color={color} wireframe transparent opacity={entrance * 0.15} />
      </mesh>
    </group>
  );
};

/** Ambient particle field for scene depth */
const LatticeParticleField: React.FC<{ count: number; entrance: number }> = ({ count, entrance }) => {
  const frame = useCurrentFrame();
  const particles = React.useMemo(() =>
    Array.from({ length: count }, (_, i) => ({
      x: (Math.sin(i * 1.7 + 3.1) * 4),
      y: (Math.cos(i * 2.3 + 1.2) * 3),
      z: (Math.sin(i * 0.9 + 5.7) * 2 - 1.5),
      speed: 0.01 + (i % 7) * 0.003,
      size: 0.015 + (i % 5) * 0.005,
    })), [count]);

  return (
    <group>
      {particles.map((p, i) => {
        const yOff = Math.sin(frame * p.speed + i) * 0.3;
        return (
          <mesh key={i} position={[p.x, p.y + yOff, p.z]} scale={[p.size * entrance, p.size * entrance, p.size * entrance]}>
            <sphereGeometry args={[1, 8, 8]} />
            <meshBasicMaterial color="#6ea0ff" transparent opacity={entrance * 0.2} />
          </mesh>
        );
      })}
    </group>
  );
};

/** Pulsing ring around the LUB node */
const MergeRing: React.FC<{ position: [number, number, number]; entrance: number; index: number }> = ({ position, entrance, index }) => {
  const frame = useCurrentFrame();
  const pulse = Math.sin(frame * 0.04 + index * 2.1) * 0.15 + 1;
  const s = entrance * 0.5 * pulse;
  return (
    <mesh position={position} rotation={[Math.PI / 2, 0, frame * 0.01 + index * 1.05]} scale={[s, s, s]}>
      <torusGeometry args={[1, 0.01, 8, 48]} />
      <meshBasicMaterial color="#c9a0ff" transparent opacity={entrance * 0.25} />
    </mesh>
  );
};

/** Animated edge between lattice nodes */
const LatticeEdge: React.FC<{
  from: [number, number, number];
  to: [number, number, number];
  color: string;
  progress: number;
}> = ({ from, to, color, progress }) => {
  if (progress < 0.01) return null;
  const dx = to[0] - from[0];
  const dy = to[1] - from[1];
  const length = Math.sqrt(dx * dx + dy * dy) * progress;
  const midX = from[0] + dx * progress * 0.5;
  const midY = from[1] + dy * progress * 0.5;

  return (
    <mesh position={[midX, midY, 0]} rotation={[0, 0, Math.atan2(dy, dx)]}>
      <boxGeometry args={[length, 0.02, 0.02]} />
      <meshStandardMaterial color={color} emissive={color} emissiveIntensity={0.5} transparent opacity={progress * 0.6} />
    </mesh>
  );
};

export const SemilatticeScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();

  /*
   * Lattice structure (Hasse diagram):
   *          ⊤ (LUB - merge result)
   *         / \
   *        S₁  S₂     ← two divergent states
   *         \ /
   *          ⊥ (bottom / initial state)
   */
  const positions = {
    bottom: [0, -1.6, 0] as [number, number, number],
    s1: [-1.5, 0, 0] as [number, number, number],
    s2: [1.5, 0, 0] as [number, number, number],
    top: [0, 1.6, 0] as [number, number, number],
  };

  // Sequential entrances
  const bottomEnt = spring({ frame, fps, delay: 5, config: { damping: 14 } });
  const s1Ent = spring({ frame, fps, delay: 30, config: { damping: 14 } });
  const s2Ent = spring({ frame, fps, delay: 45, config: { damping: 14 } });
  const topEnt = spring({ frame, fps, delay: 90, config: { damping: 14 } });

  // Edge animations
  const edgeBottom1 = spring({ frame, fps, delay: 25, config: { damping: 200 } });
  const edgeBottom2 = spring({ frame, fps, delay: 40, config: { damping: 200 } });
  const edge1Top = spring({ frame, fps, delay: 80, config: { damping: 200 } });
  const edge2Top = spring({ frame, fps, delay: 85, config: { damping: 200 } });

  // Top node merge glow
  const mergeGlow = frame > 90
    ? interpolate(spring({ frame: frame - 90, fps, config: { damping: 10 } }), [0, 1], [0.3, 1.5])
    : 0.3;

  // Text phases
  const titleOpacity = interpolate(frame, [5, 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const propOpacity = interpolate(frame, [120, 140], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const propY = interpolate(spring({ frame, fps, delay: 120, config: { damping: 200 } }), [0, 1], [15, 0]);

  // Labels
  const labelOpacity = interpolate(frame, [55, 70], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const lubLabelOpacity = interpolate(frame, [100, 115], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Fade out
  const fadeOut = interpolate(frame, [310, 335], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ backgroundColor: "#1e1e1e", opacity: fadeOut }}>
      <ThreeCanvas linear width={width} height={height}>
        <color attach="background" args={["#1e1e1e"]} />
        <ambientLight intensity={0.4} />
        <pointLight position={[5, 5, 5]} intensity={1} color="#6ea0ff" />
        <pointLight position={[-5, -3, 3]} intensity={0.5} color="#a06eff" />
        <directionalLight position={[0, 5, 5]} intensity={0.3} />

        {/* Ambient particles */}
        <LatticeParticleField count={20} entrance={bottomEnt} />

        {/* Bottom node (⊥) */}
        <LatticeNode position={positions.bottom} color="#c9a0ff" label="⊥" entrance={bottomEnt} scale={0.22} glow={0.35} />
        {/* S₁ */}
        <LatticeNode position={positions.s1} color="#4a9eff" label="S₁" entrance={s1Ent} />
        {/* S₂ */}
        <LatticeNode position={positions.s2} color="#ff6a9e" label="S₂" entrance={s2Ent} />
        {/* ⊤ (LUB) */}
        <LatticeNode position={positions.top} color="#c9a0ff" label="⊤" entrance={topEnt} glow={mergeGlow} scale={0.35} />

        {/* Merge rings around LUB */}
        <MergeRing position={positions.top} entrance={topEnt} index={0} />
        <MergeRing position={positions.top} entrance={topEnt} index={1} />

        {/* Edges */}
        <LatticeEdge from={positions.bottom} to={positions.s1} color="#4a9eff" progress={edgeBottom1} />
        <LatticeEdge from={positions.bottom} to={positions.s2} color="#ff6a9e" progress={edgeBottom2} />
        <LatticeEdge from={positions.s1} to={positions.top} color="#c9a0ff" progress={edge1Top} />
        <LatticeEdge from={positions.s2} to={positions.top} color="#c9a0ff" progress={edge2Top} />
      </ThreeCanvas>

      <AbsoluteFill style={{ pointerEvents: "none" }}>
        {/* Title */}
        <div style={{ position: "absolute", top: 35, left: 60, opacity: titleOpacity }}>
          <span style={{ fontFamily: FONT_DISPLAY, fontSize: 26, color: "rgba(255,255,255,0.9)" }}>
            Join-Semilattice
          </span>
        </div>

        {/* Node labels */}
        <div style={{ position: "absolute", left: "47%", bottom: "14%", opacity: labelOpacity }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 17, color: "#51a877" }}>⊥ bottom</span>
        </div>
        <div style={{ position: "absolute", left: "23%", top: "42%", opacity: labelOpacity }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 17, color: "#4a9eff" }}>State S₁</span>
        </div>
        <div style={{ position: "absolute", right: "23%", top: "42%", opacity: labelOpacity }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 17, color: "#ff6a9e" }}>State S₂</span>
        </div>
        <div style={{ position: "absolute", left: "45%", top: "14%", opacity: lubLabelOpacity }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 18, color: "#c9a0ff" }}>S₁ ⊔ S₂ = LUB</span>
        </div>

        {/* Formal definition box */}
        <div style={{
          position: "absolute",
          top: 70,
          right: 50,
          opacity: propOpacity,
          transform: `translateY(${propY}px)`,
          background: "rgba(255,255,255,0.03)",
          border: "1px solid rgba(255,255,255,0.08)",
          borderRadius: 10,
          padding: "12px 18px",
          maxWidth: 260,
        }}>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 14, color: "rgba(255,255,255,0.5)", margin: 0, marginBottom: 6 }}>Formal definition</p>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 16, color: "#4a9eff", margin: 0, lineHeight: 1.6 }}>
            a ⊔ b = b ⊔ a
          </p>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 16, color: "#ff6a9e", margin: 0, lineHeight: 1.6 }}>
            (a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)
          </p>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 16, color: "#6eff9e", margin: 0, lineHeight: 1.6 }}>
            a ⊔ a = a
          </p>
        </div>

        {/* Properties */}
        <div style={{ position: "absolute", bottom: 60, left: 0, right: 0, textAlign: "center", opacity: propOpacity, transform: `translateY(${propY}px)` }}>
          <div style={{ fontFamily: FONT_PRIMARY, fontSize: 20, color: "white", display: "flex", justifyContent: "center", gap: 40 }}>
            <span style={{ color: "#4a9eff" }}>commutative</span>
            <span style={{ color: "rgba(255,255,255,0.3)" }}>·</span>
            <span style={{ color: "#ff6a9e" }}>associative</span>
            <span style={{ color: "rgba(255,255,255,0.3)" }}>·</span>
            <span style={{ color: "#6eff9e" }}>idempotent</span>
          </div>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 16, color: "rgba(255,255,255,0.35)", marginTop: 10 }}>
            Merge in any order, any number of times — always converges
          </p>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
