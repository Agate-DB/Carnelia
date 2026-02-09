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
 * Scene — How Carnelia Fixes Each CRDT Limitation
 *
 * Maps each limitation to Carnelia's architectural answer:
 * 1. State Bloat      → δ-CRDT (delta mutations)
 * 2. Tombstones       → Dot Store + Causal Context
 * 3. Vector Clocks    → Merkle-Clock DAG
 * 4. Transport        → DAG-Syncer anti-entropy
 *
 * Visual: four "problem → solution" transitions. The broken 3D object
 * from LimitationsScene transforms into the corresponding fix.
 *
 * Concept grounding (architecture doc §4):
 * - "δ-CRDT core generates bandwidth-efficient incremental state changes"
 * - "causal context + dot store: tombstone-free removal"
 * - "Merkle-Clock: content-addressed, open-membership, Byzantine-resistant"
 * - "DAG-Syncer: pull-based, idempotent, robust to failure"
 *
 * AUDIO CUE: solution_narration.mp3
 */

/** Fix arrow — animated arrow from left to right */
const FixArrow: React.FC<{
  y: number;
  progress: number;
}> = ({ y, progress }) => {
  if (progress < 0.01) return null;
  const length = 0.8 * progress;

  return (
    <group position={[0, y, 0.2]}>
      <mesh>
        <boxGeometry args={[length, 0.012, 0.012]} />
        <meshStandardMaterial color="#6eff9e" emissive="#6eff9e" emissiveIntensity={0.8} transparent opacity={progress * 0.6} />
      </mesh>
      {/* Arrowhead */}
      <mesh position={[length / 2 + 0.03, 0, 0]} rotation={[0, 0, -Math.PI / 4]}>
        <boxGeometry args={[0.06 * progress, 0.012, 0.012]} />
        <meshStandardMaterial color="#6eff9e" emissive="#6eff9e" emissiveIntensity={0.8} transparent opacity={progress * 0.6} />
      </mesh>
    </group>
  );
};

/** Problem icon — small red shape */
const ProblemIcon: React.FC<{
  position: [number, number, number];
  entrance: number;
  geometry: "cube" | "stack" | "ring" | "channel";
}> = ({ position, entrance, geometry }) => {
  const frame = useCurrentFrame();
  const s = entrance * 0.16;

  return (
    <group position={position}>
      {geometry === "cube" && (
        <mesh scale={[s, s, s]} rotation={[0.3, frame * 0.004, 0.1]}>
          <boxGeometry args={[1, 1, 1]} />
          <meshStandardMaterial color="#ff4444" emissive="#ff4444" emissiveIntensity={0.4} transparent opacity={entrance * 0.5} roughness={0.3} />
        </mesh>
      )}
      {geometry === "stack" && (
        <group>
          {[0, 0.08, 0.16, 0.24].map((y, i) => (
            <mesh key={i} position={[0, y - 0.12, 0]} scale={[s * 0.7, s * 0.3, s * 0.3]}>
              <boxGeometry args={[1, 1, 1]} />
              <meshStandardMaterial color="#888888" emissive="#666666" emissiveIntensity={0.2} transparent opacity={entrance * 0.5} />
            </mesh>
          ))}
        </group>
      )}
      {geometry === "ring" && (
        <mesh rotation={[Math.PI / 2, 0, frame * 0.005]} scale={[s, s, s]}>
          <torusGeometry args={[0.8, 0.06, 8, 16]} />
          <meshStandardMaterial color="#ffc46a" emissive="#ffc46a" emissiveIntensity={0.3} transparent opacity={entrance * 0.4} />
        </mesh>
      )}
      {geometry === "channel" && (
        <group>
          <mesh scale={[s * 2, 0.008, 0.008]}>
            <boxGeometry args={[1, 1, 1]} />
            <meshBasicMaterial color="#4a9eff" transparent opacity={entrance * 0.3} />
          </mesh>
          <mesh position={[0, 0.06, 0]} scale={[s * 0.3, s * 0.3, s * 0.3]}>
            <octahedronGeometry args={[1, 0]} />
            <meshStandardMaterial color="#ff4444" transparent opacity={entrance * 0.4 * (0.5 + Math.sin(frame * 0.1) * 0.5)} />
          </mesh>
        </group>
      )}
    </group>
  );
};

/** Solution icon — glowing green shape */
const SolutionIcon: React.FC<{
  position: [number, number, number];
  entrance: number;
  geometry: "delta" | "dotstore" | "merkle" | "syncer";
}> = ({ position, entrance, geometry }) => {
  const frame = useCurrentFrame();
  const s = entrance * 0.18;
  const breathe = 1 + Math.sin(frame * 0.03) * 0.03;

  return (
    <group position={position}>
      {geometry === "delta" && (
        <mesh scale={[s * 0.5 * breathe, s * 0.5 * breathe, s * 0.5 * breathe]} rotation={[frame * 0.008, frame * 0.005, 0]}>
          <octahedronGeometry args={[1, 0]} />
          <meshStandardMaterial color="#6eff9e" emissive="#6eff9e" emissiveIntensity={0.8} transparent opacity={entrance * 0.85} roughness={0.1} metalness={0.8} />
        </mesh>
      )}
      {geometry === "dotstore" && (
        <group>
          {/* Outer context ring */}
          <mesh rotation={[Math.PI / 2, 0, frame * 0.004]} scale={[s * 1.5, s * 1.5, s * 1.5]}>
            <torusGeometry args={[0.6, 0.015, 8, 32]} />
            <meshStandardMaterial color="#ffc46a" emissive="#ffc46a" emissiveIntensity={0.4} transparent opacity={entrance * 0.4} />
          </mesh>
          {/* Inner dots */}
          {[[-0.06, 0, 0], [0.06, 0.04, 0], [0, -0.06, 0]].map((pos, i) => (
            <mesh key={i} position={pos as [number, number, number]} scale={[s * 0.3, s * 0.3, s * 0.3]}>
              <sphereGeometry args={[1, 8, 8]} />
              <meshStandardMaterial color="#6eff9e" emissive="#6eff9e" emissiveIntensity={0.6} transparent opacity={entrance * 0.8} />
            </mesh>
          ))}
        </group>
      )}
      {geometry === "merkle" && (
        <group>
          {/* Small DAG: 3 nodes */}
          {[[0, 0.12, 0], [-0.1, -0.06, 0], [0.1, -0.06, 0]].map((pos, i) => (
            <mesh key={i} position={pos as [number, number, number]} scale={[s * 0.25 * breathe, s * 0.25 * breathe, s * 0.25 * breathe]} rotation={[frame * 0.006, frame * 0.008, 0]}>
              <dodecahedronGeometry args={[1, 0]} />
              <meshStandardMaterial color="#c9a0ff" emissive="#c9a0ff" emissiveIntensity={0.6} transparent opacity={entrance * 0.8} roughness={0.1} metalness={0.8} />
            </mesh>
          ))}
          {/* Edges */}
          {[
            { from: [0, 0.12, 0], to: [-0.1, -0.06, 0] },
            { from: [0, 0.12, 0], to: [0.1, -0.06, 0] },
          ].map((edge, i) => {
            const dx = (edge.to[0] - edge.from[0]);
            const dy = (edge.to[1] - edge.from[1]);
            const len = Math.sqrt(dx * dx + dy * dy);
            return (
              <mesh key={i} position={[(edge.from[0] + edge.to[0]) / 2, (edge.from[1] + edge.to[1]) / 2, 0]} rotation={[0, 0, Math.atan2(dy, dx)]}>
                <boxGeometry args={[len, 0.008, 0.008]} />
                <meshStandardMaterial color="#c9a0ff" emissive="#c9a0ff" emissiveIntensity={0.4} transparent opacity={entrance * 0.4} />
              </mesh>
            );
          })}
        </group>
      )}
      {geometry === "syncer" && (
        <group>
          {/* Bidirectional arrows */}
          <mesh scale={[s * 2.2, 0.012, 0.012]}>
            <boxGeometry args={[1, 1, 1]} />
            <meshStandardMaterial color="#6affea" emissive="#6affea" emissiveIntensity={0.6} transparent opacity={entrance * 0.5} />
          </mesh>
          {/* Pulse traveling along */}
          <mesh position={[Math.sin(frame * 0.06) * s * 0.8, 0, 0]} scale={[s * 0.25, s * 0.25, s * 0.25]}>
            <sphereGeometry args={[1, 8, 8]} />
            <meshStandardMaterial color="#6affea" emissive="#6affea" emissiveIntensity={1.5} transparent opacity={entrance * 0.8} />
          </mesh>
        </group>
      )}
    </group>
  );
};

export const CarneliaSolutionScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();

  // Four rows, each with problem → arrow → solution
  const rows = [
    { y: 1.2, delay: 5, arrowDelay: 30, solDelay: 45, probGeo: "cube", solGeo: "delta", label: "State Bloat", fix: "δ-CRDT Deltas", fixDesc: "Ship only what changed — Δ ≪ S", fixColor: "#6eff9e" },
    { y: 0.4, delay: 55, arrowDelay: 80, solDelay: 95, probGeo: "stack", solGeo: "dotstore", label: "Tombstones", fix: "Dot Store + Context", fixDesc: "Absence in store = deleted, no markers", fixColor: "#ffc46a" },
    { y: -0.4, delay: 105, arrowDelay: 130, solDelay: 145, probGeo: "ring", solGeo: "merkle", label: "Vector Clocks", fix: "Merkle-Clock DAG", fixDesc: "Content-addressed, Byzantine-resistant", fixColor: "#c9a0ff" },
    { y: -1.2, delay: 155, arrowDelay: 180, solDelay: 195, probGeo: "channel", solGeo: "syncer", label: "Transport", fix: "DAG-Syncer", fixDesc: "Pull-based, idempotent, tolerates loss", fixColor: "#6affea" },
  ] as const;

  const titleOpacity = interpolate(frame, [5, 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Carnelia branding callout
  const brandOpacity = interpolate(frame, [220, 245], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const brandY = interpolate(spring({ frame, fps, delay: 220, config: { damping: 200 } }), [0, 1], [12, 0]);

  const fadeOut = interpolate(frame, [290, 320], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ backgroundColor: "#0a0a1a", opacity: fadeOut }}>
      <ThreeCanvas linear width={width} height={height}>
        <color attach="background" args={["#0a0a1a"]} />
        <ambientLight intensity={0.4} />
        <pointLight position={[5, 5, 5]} intensity={0.8} color="#6ea0ff" />
        <pointLight position={[-5, -3, 3]} intensity={0.5} color="#a06eff" />
        <directionalLight position={[0, 5, 5]} intensity={0.2} />

        {rows.map((row, i) => {
          const probEnt = spring({ frame, fps, delay: row.delay, config: { damping: 14 } });
          const arrowProg = spring({ frame, fps, delay: row.arrowDelay, config: { damping: 200 } });
          const solEnt = spring({ frame, fps, delay: row.solDelay, config: { damping: 14 } });

          return (
            <group key={i}>
              <ProblemIcon position={[-1.5, row.y, 0]} entrance={probEnt} geometry={row.probGeo} />
              <FixArrow y={row.y} progress={arrowProg} />
              <SolutionIcon position={[1.5, row.y, 0]} entrance={solEnt} geometry={row.solGeo} />
            </group>
          );
        })}
      </ThreeCanvas>

      <AbsoluteFill style={{ pointerEvents: "none" }}>
        {/* Title */}
        <div style={{ position: "absolute", top: 35, left: 60, opacity: titleOpacity }}>
          <span style={{ fontFamily: FONT_DISPLAY, fontSize: 22, color: "#e06040" }}>
            How Carnelia Fixes This
          </span>
        </div>

        {/* Row labels */}
        {rows.map((row, i) => {
          const rowOpacity = interpolate(frame, [row.delay + 5, row.delay + 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
          const solOpacity = interpolate(frame, [row.solDelay + 5, row.solDelay + 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

          // Map y to screen percentage (approximate)
          const topPct = interpolate(row.y, [-1.2, 1.2], [72, 18]);

          return (
            <React.Fragment key={i}>
              {/* Problem label */}
              <div style={{ position: "absolute", left: "3%", top: `${topPct}%`, opacity: rowOpacity }}>
                <span style={{
                  fontFamily: FONT_PRIMARY, fontSize: 13, color: "#ff6a6a",
                  textDecoration: "line-through", textDecorationColor: "rgba(255,106,106,0.5)",
                }}>
                  {row.label}
                </span>
              </div>
              {/* Solution label */}
              <div style={{ position: "absolute", right: "3%", top: `${topPct}%`, opacity: solOpacity, maxWidth: 260 }}>
                <span style={{ fontFamily: FONT_PRIMARY, fontSize: 14, color: row.fixColor, fontWeight: 400 }}>
                  {row.fix}
                </span>
                <p style={{ fontFamily: FONT_PRIMARY, fontSize: 11, color: "rgba(255,255,255,0.35)", margin: 0, marginTop: 3, lineHeight: 1.5 }}>
                  {row.fixDesc}
                </p>
              </div>
            </React.Fragment>
          );
        })}

        {/* Carnelia branding callout */}
        <div style={{
          position: "absolute", bottom: 35, left: 0, right: 0, textAlign: "center",
          opacity: brandOpacity, transform: `translateY(${brandY}px)`,
        }}>
          <p style={{ fontFamily: FONT_DISPLAY, fontSize: 20, color: "#e06040", margin: 0 }}>
            Carnelia — a complete architectural synthesis
          </p>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.4)", marginTop: 6 }}>
            Open-membership · Byzantine-tolerant · Bandwidth-efficient
          </p>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
