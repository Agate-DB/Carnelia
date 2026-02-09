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
 * Scene — CRDT Limitations: Why Traditional CRDTs Fall Short
 *
 * Four animated "problem cards" appear sequentially in 3D space,
 * each paired with a visual metaphor:
 *
 * 1. State Bloat — a growing cube that gets unreasonably large
 * 2. Tombstone Accumulation — graveyard of dead markers stacking up
 * 3. Vector Clock Fragility — a clock-like ring that fragments
 * 4. Transport Assumptions — messages lost/duplicated on a channel
 *
 * Concept grounding (analysis report §2):
 * - "State-based CRDTs require replicas to periodically exchange their entire state payload"
 * - "Tombstones (deletion markers) are never removed and lead to performance degradation"
 * - "Vector Clocks metadata overhead grows linearly with participants"
 * - "Op-based CRDTs depend on reliable messaging layer: exactly-once, causally-ordered"
 *
 * AUDIO CUE: limitations_narration.mp3
 */

/** Bloating cube — starts small, grows uncontrollably */
const BloatingCube: React.FC<{
  entrance: number;
  bloatProgress: number;
  position: [number, number, number];
}> = ({ entrance, bloatProgress, position }) => {
  const frame = useCurrentFrame();
  const baseScale = 0.18 * entrance;
  const bloat = 1 + bloatProgress * 2.5;
  const s = baseScale * bloat;
  const dangerGlow = bloatProgress * 0.8;

  return (
    <group position={position}>
      <mesh scale={[s, s, s]} rotation={[0.3, frame * 0.004, 0.1]}>
        <boxGeometry args={[1, 1, 1]} />
        <meshStandardMaterial
          color={bloatProgress > 0.5 ? "#ff4444" : "#4a9eff"}
          roughness={0.2}
          metalness={0.6}
          emissive={bloatProgress > 0.5 ? "#ff4444" : "#4a9eff"}
          emissiveIntensity={dangerGlow}
          transparent
          opacity={entrance * 0.6}
        />
      </mesh>
      <mesh scale={[s * 1.02, s * 1.02, s * 1.02]} rotation={[0.3, frame * 0.004, 0.1]}>
        <boxGeometry args={[1, 1, 1]} />
        <meshBasicMaterial
          color={bloatProgress > 0.5 ? "#ff4444" : "#4a9eff"}
          wireframe
          transparent
          opacity={entrance * 0.15}
        />
      </mesh>
    </group>
  );
};

/** Tombstone markers stacking up */
const TombstoneStack: React.FC<{
  entrance: number;
  count: number;
  position: [number, number, number];
}> = ({ entrance, count, position }) => {
  return (
    <group position={position}>
      {Array.from({ length: count }).map((_, i) => {
        const y = i * 0.12 - (count * 0.06);
        const s = entrance * 0.06;
        return (
          <mesh key={i} position={[0, y, 0]} scale={[s, s * 1.4, s * 0.3]}>
            <boxGeometry args={[1, 1, 1]} />
            <meshStandardMaterial
              color="#e0d0b8"
              roughness={0.4}
              metalness={0.3}
              emissive="#e0d0b8"
              emissiveIntensity={0.45}
              transparent
              opacity={entrance * 0.85}
            />
          </mesh>
        );
      })}
    </group>
  );
};

/** Fragmenting clock ring — a torus that breaks apart */
const FragmentingClock: React.FC<{
  entrance: number;
  fragmentProgress: number;
  position: [number, number, number];
}> = ({ entrance, fragmentProgress, position }) => {
  const frame = useCurrentFrame();
  const s = entrance * 0.45;

  return (
    <group position={position}>
      {/* Main ring — fading as it fragments */}
      <mesh rotation={[Math.PI / 2, 0, frame * 0.005]} scale={[s, s, s]}>
        <torusGeometry args={[1, 0.06, 8, 32]} />
        <meshStandardMaterial
          color="#ffc46a"
          emissive="#ffc46a"
          emissiveIntensity={0.3}
          transparent
          opacity={entrance * (1 - fragmentProgress * 0.7)}
        />
      </mesh>
      {/* Fragments flying out */}
      {fragmentProgress > 0.1 && Array.from({ length: 5 }).map((_, i) => {
        const angle = (i / 5) * Math.PI * 2 + frame * 0.01;
        const dist = fragmentProgress * 0.6;
        const fx = Math.cos(angle) * (0.45 + dist) * s;
        const fy = Math.sin(angle) * (0.45 + dist) * s;
        const fs = 0.04 * entrance;
        return (
          <mesh key={i} position={[fx, fy, 0]}>
            <boxGeometry args={[fs, fs * 0.5, fs * 0.3]} />
            <meshStandardMaterial
              color="#ffc46a"
              emissive="#ffc46a"
              emissiveIntensity={0.4}
              transparent
              opacity={entrance * Math.max(0, 1 - fragmentProgress)}
            />
          </mesh>
        );
      })}
    </group>
  );
};

/** Partitioned replicas — two node groups drifting apart with broken link */
const PartitionedNodes: React.FC<{
  entrance: number;
  partitionProgress: number;
  position: [number, number, number];
}> = ({ entrance, partitionProgress, position }) => {
  const frame = useCurrentFrame();
  const drift = partitionProgress * 0.4;
  const s = entrance * 0.12;
  const signalFade = Math.max(0, 1 - partitionProgress * 1.5);

  return (
    <group position={position}>
      {/* Left node cluster */}
      <mesh position={[-0.35 - drift, 0.05, 0]} scale={[s, s, s]} rotation={[0.3, frame * 0.005, 0.1]}>
        <sphereGeometry args={[1, 16, 16]} />
        <meshStandardMaterial color="#4a9eff" emissive="#4a9eff" emissiveIntensity={0.4} transparent opacity={entrance * 0.7} />
      </mesh>
      <mesh position={[-0.55 - drift, -0.1, 0]} scale={[s * 0.7, s * 0.7, s * 0.7]}>
        <sphereGeometry args={[1, 12, 12]} />
        <meshStandardMaterial color="#4a9eff" emissive="#4a9eff" emissiveIntensity={0.2} transparent opacity={entrance * 0.5} />
      </mesh>
      {/* Right node cluster */}
      <mesh position={[0.35 + drift, -0.05, 0]} scale={[s, s, s]} rotation={[0.2, frame * 0.004, -0.1]}>
        <sphereGeometry args={[1, 16, 16]} />
        <meshStandardMaterial color="#ff6a9e" emissive="#ff6a9e" emissiveIntensity={0.4} transparent opacity={entrance * 0.7} />
      </mesh>
      <mesh position={[0.5 + drift, 0.12, 0]} scale={[s * 0.7, s * 0.7, s * 0.7]}>
        <sphereGeometry args={[1, 12, 12]} />
        <meshStandardMaterial color="#ff6a9e" emissive="#ff6a9e" emissiveIntensity={0.2} transparent opacity={entrance * 0.5} />
      </mesh>
      {/* Dashed connection line — fading */}
      {Array.from({ length: 6 }).map((_, i) => {
        const t = (i + 0.5) / 6;
        const x = -0.3 + t * 0.6;
        return (
          <mesh key={i} position={[x, 0, 0]}>
            <boxGeometry args={[0.04 * entrance, 0.006, 0.006]} />
            <meshBasicMaterial color="#ff4444" transparent opacity={entrance * signalFade * 0.3} />
          </mesh>
        );
      })}
      {/* Lightning / break mark */}
      {partitionProgress > 0.3 && (
        <group>
          <mesh position={[0, 0, 0.02]} rotation={[0, 0, 0.3]} scale={[entrance * 0.006, entrance * 0.08, entrance * 0.003]}>
            <boxGeometry args={[1, 1, 1]} />
            <meshBasicMaterial color="#ff4444" transparent opacity={entrance * 0.7} />
          </mesh>
          <mesh position={[0.015, -0.02, 0.02]} rotation={[0, 0, -0.4]} scale={[entrance * 0.006, entrance * 0.06, entrance * 0.003]}>
            <boxGeometry args={[1, 1, 1]} />
            <meshBasicMaterial color="#ff4444" transparent opacity={entrance * 0.6} />
          </mesh>
        </group>
      )}
    </group>
  );
};

/** Lost/duplicated message particles on a channel */
const BrokenChannel: React.FC<{
  entrance: number;
  position: [number, number, number];
}> = ({ entrance, position }) => {
  const frame = useCurrentFrame();

  // Three message particles — one lost (fades), one duplicated (splits)
  const msgs = [
    { x: -0.5, lost: false, dup: false, color: "#6eff9e" },
    { x: 0, lost: true, dup: false, color: "#ff4444" },
    { x: 0.5, lost: false, dup: true, color: "#4a9eff" },
  ];

  return (
    <group position={position}>
      {/* Channel line */}
      <mesh>
        <boxGeometry args={[1.6 * entrance, 0.008, 0.008]} />
        <meshBasicMaterial color="#ffffff" transparent opacity={entrance * 0.12} />
      </mesh>
      {msgs.map((m, i) => {
        const moveT = Math.sin(frame * 0.04 + i * 2) * 0.3;
        const particleOpacity = m.lost
          ? entrance * Math.max(0, 0.6 - Math.abs(Math.sin(frame * 0.05)) * 0.8)
          : entrance * 0.7;
        const s = entrance * 0.04;
        return (
          <group key={i}>
            <mesh position={[m.x + moveT, 0.05, 0]} scale={[s, s, s]}>
              <octahedronGeometry args={[1, 0]} />
              <meshStandardMaterial
                color={m.color}
                emissive={m.color}
                emissiveIntensity={0.8}
                transparent
                opacity={particleOpacity}
              />
            </mesh>
            {/* Duplicate ghost */}
            {m.dup && (
              <mesh position={[m.x + moveT + 0.1, -0.08, 0]} scale={[s * 0.7, s * 0.7, s * 0.7]}>
                <octahedronGeometry args={[1, 0]} />
                <meshStandardMaterial
                  color={m.color}
                  emissive={m.color}
                  emissiveIntensity={0.4}
                  transparent
                  opacity={particleOpacity * 0.4}
                />
              </mesh>
            )}
          </group>
        );
      })}
      {/* X mark for lost message */}
      <mesh position={[0, -0.12, 0]} rotation={[0, 0, Math.PI / 4]} scale={[entrance * 0.005, entrance * 0.06, entrance * 0.005]}>
        <boxGeometry args={[1, 1, 1]} />
        <meshBasicMaterial color="#ff4444" transparent opacity={entrance * 0.6} />
      </mesh>
      <mesh position={[0, -0.12, 0]} rotation={[0, 0, -Math.PI / 4]} scale={[entrance * 0.005, entrance * 0.06, entrance * 0.005]}>
        <boxGeometry args={[1, 1, 1]} />
        <meshBasicMaterial color="#ff4444" transparent opacity={entrance * 0.6} />
      </mesh>
    </group>
  );
};

export const LimitationsScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();

  // Five problems appear sequentially
  // Problem 1: State Bloat (frames 0–80)
  const bloatEnt = spring({ frame, fps, delay: 5, config: { damping: 14 } });
  const bloatProgress = interpolate(frame, [20, 75], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Problem 2: Tombstone Accumulation (frames 50–130)
  const tombEnt = spring({ frame, fps, delay: 50, config: { damping: 14 } });
  const tombCount = Math.min(12, Math.floor(interpolate(frame, [55, 120], [0, 12], { extrapolateLeft: "clamp", extrapolateRight: "clamp" })));

  // Problem 3: Vector Clock Fragility (frames 100–180)
  const clockEnt = spring({ frame, fps, delay: 100, config: { damping: 14 } });
  const fragmentProgress = interpolate(frame, [115, 170], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Problem 4: Transport Assumptions (frames 150–230)
  const channelEnt = spring({ frame, fps, delay: 150, config: { damping: 14 } });

  // Problem 5: Network Partition / Poor Connectivity (frames 200–280)
  const partitionEnt = spring({ frame, fps, delay: 200, config: { damping: 14 } });
  const partitionProgress = interpolate(frame, [215, 270], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Labels
  const label1 = interpolate(frame, [10, 25], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const label2 = interpolate(frame, [55, 70], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const label3 = interpolate(frame, [105, 120], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const label4 = interpolate(frame, [155, 170], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const label5 = interpolate(frame, [205, 220], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Summary callout
  const summaryOpacity = interpolate(frame, [290, 310], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const summaryY = interpolate(spring({ frame, fps, delay: 290, config: { damping: 200 } }), [0, 1], [12, 0]);

  const fadeOut = interpolate(frame, [410, 440], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ backgroundColor: "#1e1e1e", opacity: fadeOut }}>
      <ThreeCanvas linear width={width} height={height}>
        <color attach="background" args={["#1e1e1e"]} />
        <ambientLight intensity={0.4} />
        <pointLight position={[5, 5, 5]} intensity={0.8} color="#6ea0ff" />
        <pointLight position={[-5, -3, 3]} intensity={0.5} color="#a06eff" />

        {/* 5-panel layout: 2 top, 2 middle, 1 bottom center */}
        <BloatingCube entrance={bloatEnt} bloatProgress={bloatProgress} position={[-2.2, 1.4, 0]} />
        <TombstoneStack entrance={tombEnt} count={tombCount} position={[2.2, 1.4, 0]} />
        <FragmentingClock entrance={clockEnt} fragmentProgress={fragmentProgress} position={[-2.2, -0.2, 0]} />
        <BrokenChannel entrance={channelEnt} position={[2.2, -0.2, 0]} />
        <PartitionedNodes entrance={partitionEnt} partitionProgress={partitionProgress} position={[0, -1.8, 0]} />
      </ThreeCanvas>

      <AbsoluteFill style={{ pointerEvents: "none" }}>
        {/* Title */}
        <div style={{ position: "absolute", top: 35, left: 60, opacity: label1 }}>
          <span style={{ fontFamily: FONT_DISPLAY, fontSize: 26, color: "rgba(255,255,255,0.9)" }}>
            Where Traditional CRDTs Fall Short
          </span>
        </div>

        {/* Problem 1 label */}
        <div style={{ position: "absolute", left: "8%", top: "16%", opacity: label1, maxWidth: 260 }}>
          <div style={{
            fontFamily: FONT_PRIMARY, fontSize: 17, color: "#ff6a6a",
            padding: "6px 12px", border: "1px solid rgba(255,106,106,0.2)",
            borderRadius: 6, background: "rgba(255,106,106,0.04)",
          }}>
            1. State Bloat
          </div>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.4)", marginTop: 4, lineHeight: 1.5 }}>
            CvRDTs ship the entire state every sync — grows linearly with data
          </p>
        </div>

        {/* Problem 2 label */}
        <div style={{ position: "absolute", right: "8%", top: "16%", opacity: label2, maxWidth: 260 }}>
          <div style={{
            fontFamily: FONT_PRIMARY, fontSize: 17, color: "#e0d0b8",
            padding: "6px 12px", border: "1px solid rgba(200,184,160,0.25)",
            borderRadius: 6, background: "rgba(200,184,160,0.06)",
          }}>
            2. Tombstone Accumulation
          </div>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.4)", marginTop: 4, lineHeight: 1.5 }}>
            Deletion markers never removed — unbounded metadata growth
          </p>
        </div>

        {/* Problem 3 label */}
        <div style={{ position: "absolute", left: "8%", top: "46%", opacity: label3, maxWidth: 260 }}>
          <div style={{
            fontFamily: FONT_PRIMARY, fontSize: 17, color: "#ffc46a",
            padding: "6px 12px", border: "1px solid rgba(255,196,106,0.2)",
            borderRadius: 6, background: "rgba(255,196,106,0.04)",
          }}>
            3. Vector Clock Fragility
          </div>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.4)", marginTop: 4, lineHeight: 1.5 }}>
            Metadata O(n) in replicas — Byzantine nodes can forge causal links
          </p>
        </div>

        {/* Problem 4 label */}
        <div style={{ position: "absolute", right: "8%", top: "46%", opacity: label4, maxWidth: 260 }}>
          <div style={{
            fontFamily: FONT_PRIMARY, fontSize: 17, color: "#4a9eff",
            padding: "6px 12px", border: "1px solid rgba(74,158,255,0.2)",
            borderRadius: 6, background: "rgba(74,158,255,0.04)",
          }}>
            4. Transport Assumptions
          </div>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.4)", marginTop: 4, lineHeight: 1.5 }}>
            CmRDTs require exactly-once, causally-ordered delivery
          </p>
        </div>

        {/* Problem 5 label */}
        <div style={{ position: "absolute", left: "50%", bottom: "8%", opacity: label5, maxWidth: 320, transform: "translateX(-50%)" }}>
          <div style={{
            fontFamily: FONT_PRIMARY, fontSize: 17, color: "#ff9e6a",
            padding: "6px 12px", border: "1px solid rgba(255,158,106,0.2)",
            borderRadius: 6, background: "rgba(255,158,106,0.04)",
            textAlign: "center",
          }}>
            5. Network Partition Fragility
          </div>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.4)", marginTop: 4, lineHeight: 1.5, textAlign: "center" }}>
            Poor connectivity splits replica groups — state diverges with no automatic repair
          </p>
        </div>

        {/* Summary */}
        <div style={{
          position: "absolute", bottom: 35, left: 0, right: 0, textAlign: "center",
          opacity: summaryOpacity, transform: `translateY(${summaryY}px)`,
        }}>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 22, color: "#ff6a6a", margin: 0 }}>
            These gaps prevent production deployment in open-membership networks
          </p>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
