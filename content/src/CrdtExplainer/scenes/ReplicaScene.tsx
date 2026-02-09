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
 * Scene 2 — Optimistic Replication: Independent Replicas
 *
 * Shows 3 replica nodes (spheres with orbit rings) in a triangular layout.
 * Each node independently applies a local update (counter-style increment
 * visualized as a growing bar). Demonstrates that replicas never coordinate
 * before accepting writes — "optimistic replication."
 *
 * Concept grounding (architecture doc):
 * - "Replicas: independent nodes, each maintains a full local copy"
 * - "Optimistic Replication: users modify data on any replica independently"
 *
 * AUDIO CUE: replicas_narration.mp3
 */

/** A single replica sphere with orbit ring and local counter bar */
const ReplicaNode: React.FC<{
  position: [number, number, number];
  color: string;
  delay: number;
  counterValue: number; // 0..1 for bar height
  pulsing: boolean;
}> = ({ position, color, delay, counterValue, pulsing }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const entrance = spring({ frame, fps, delay, config: { damping: 14 } });
  const yBob = Math.sin(frame * 0.025 + delay) * 0.07 * entrance;
  const pulseScale = pulsing
    ? 1 + spring({ frame: frame - delay, fps, config: { damping: 8, stiffness: 150 } }) * 0.15
    : 1;
  const s = entrance * pulseScale;

  return (
    <group position={[position[0], position[1] + yBob, position[2]]}>
      {/* Main sphere */}
      <mesh scale={[s * 0.38, s * 0.38, s * 0.38]}>
        <sphereGeometry args={[1, 32, 32]} />
        <meshStandardMaterial
          color={color}
          roughness={0.2}
          metalness={0.7}
          emissive={color}
          emissiveIntensity={0.3 + (pulsing ? 0.4 : 0)}
          transparent
          opacity={entrance}
        />
      </mesh>

      {/* Orbit ring */}
      <mesh rotation={[Math.PI / 2, 0, frame * 0.01]} scale={[entrance * 1.8, entrance * 1.8, entrance * 1.8]}>
        <torusGeometry args={[0.38, 0.012, 8, 64]} />
        <meshStandardMaterial color={color} transparent opacity={entrance * 0.45} emissive={color} emissiveIntensity={0.3} />
      </mesh>

      {/* Local state bar — grows with counterValue */}
      {counterValue > 0 && (
        <group position={[0, -0.65, 0.3]}>
          {/* Bar background */}
          <mesh scale={[entrance, entrance, entrance]}>
            <boxGeometry args={[0.5, 0.08, 0.08]} />
            <meshStandardMaterial color={color} transparent opacity={0.15 * entrance} roughness={0.5} />
          </mesh>
          {/* Bar fill */}
          <mesh
            position={[-(0.5 / 2) * (1 - counterValue), 0, 0.005]}
            scale={[entrance, entrance, entrance]}
          >
            <boxGeometry args={[0.5 * counterValue, 0.08, 0.09]} />
            <meshStandardMaterial
              color={color}
              emissive={color}
              emissiveIntensity={0.6}
              transparent
              opacity={entrance * 0.9}
              roughness={0.2}
            />
          </mesh>
        </group>
      )}

      {/* Pulse ring on update */}
      {pulsing && (
        <mesh rotation={[Math.PI / 2, 0, 0]}>
          <ringGeometry args={[0.4 + (pulseScale - 1) * 3, 0.42 + (pulseScale - 1) * 3, 32]} />
          <meshBasicMaterial color={color} transparent opacity={Math.max(0, 1 - (pulseScale - 1) * 8) * 0.5} side={2} />
        </mesh>
      )}
    </group>
  );
};

/** Dashed connection line between nodes */
const ConnectionLine: React.FC<{
  from: [number, number, number];
  to: [number, number, number];
  opacity: number;
}> = ({ from, to, opacity }) => {
  const dx = to[0] - from[0];
  const dy = to[1] - from[1];
  const midZ = (from[2] + to[2]) / 2;

  // Render 6 dash segments
  const dashCount = 6;
  return (
    <group>
      {Array.from({ length: dashCount }).map((_, i) => {
        const t = (i + 0.3) / dashCount;
        const dashLen = (Math.sqrt(dx * dx + dy * dy) / dashCount) * 0.5;
        const px = from[0] + dx * t;
        const py = from[1] + dy * t;
        return (
          <mesh key={i} position={[px, py, midZ]} rotation={[0, 0, Math.atan2(dy, dx)]}>
            <boxGeometry args={[dashLen, 0.012, 0.012]} />
            <meshBasicMaterial color="#ffffff" transparent opacity={opacity} />
          </mesh>
        );
      })}
    </group>
  );
};

/** Network cloud — wireframe sphere centered between nodes */
const NetworkCloud: React.FC<{ entrance: number }> = ({ entrance }) => {
  const frame = useCurrentFrame();
  const s = entrance * 0.8;
  return (
    <group position={[0, 0.1, -0.5]}>
      <mesh scale={[s * 1.4, s * 0.8, s * 0.6]} rotation={[0, frame * 0.002, 0]}>
        <icosahedronGeometry args={[1, 1]} />
        <meshBasicMaterial color="#4a9eff" wireframe transparent opacity={entrance * 0.04} />
      </mesh>
    </group>
  );
};

/** Data sync particle — travels between two replicas after updates */
const SyncParticle: React.FC<{
  from: [number, number, number];
  to: [number, number, number];
  startFrame: number;
  color: string;
}> = ({ from, to, startFrame, color }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const localFrame = frame - startFrame;
  if (localFrame < 0 || localFrame > 50) return null;

  const t = spring({ frame: localFrame, fps, config: { damping: 18, mass: 0.6 } });
  const x = interpolate(t, [0, 1], [from[0], to[0]]);
  const y = interpolate(t, [0, 1], [from[1], to[1]]) + Math.sin(t * Math.PI) * 0.5;
  const s = 0.035 * (1 + Math.sin(localFrame * 0.4) * 0.15);

  return (
    <mesh position={[x, y, 0.3]}>
      <octahedronGeometry args={[s, 0]} />
      <meshStandardMaterial color={color} emissive={color} emissiveIntensity={1.5} transparent opacity={0.8} />
    </mesh>
  );
};

export const ReplicaScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();

  const nodePositions: [number, number, number][] = [
    [-2.4, 0.9, 0],
    [2.4, 0.9, 0],
    [0, -1.5, 0],
  ];
  const colors = ["#4a9eff", "#ff6a9e", "#6eff9e"];
  const labels = ["Replica A", "Replica B", "Replica C"];

  // Counter animations — each replica updates at different times
  const counterA = interpolate(frame, [60, 90], [0, 0.4], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const counterB = interpolate(frame, [90, 120], [0, 0.6], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const counterC = interpolate(frame, [120, 150], [0, 0.3], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const counters = [counterA, counterB, counterC];

  const lineOpacity = interpolate(frame, [35, 55], [0, 0.2], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const labelEntrance = spring({ frame, fps, delay: 40, config: { damping: 200 } });
  const labelOpacity = interpolate(frame, [40, 60], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // "No coordination" annotation
  const noCoordOpacity = interpolate(frame, [140, 160], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Description
  const descProgress = spring({ frame, fps, delay: 50, config: { damping: 200 } });
  const descOpacity = interpolate(descProgress, [0, 1], [0, 1]);
  const descY = interpolate(descProgress, [0, 1], [15, 0]);

  // Fade out
  const fadeOut = interpolate(frame, [210, 240], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ backgroundColor: "#0a0a1a", opacity: fadeOut }}>
      <ThreeCanvas linear width={width} height={height}>
        <color attach="background" args={["#0a0a1a"]} />
        <ambientLight intensity={0.4} />
        <pointLight position={[5, 5, 5]} intensity={1} color="#6ea0ff" />
        <pointLight position={[-5, -3, 3]} intensity={0.5} color="#a06eff" />
        <directionalLight position={[0, 4, 4]} intensity={0.2} />

        {nodePositions.map((pos, i) => (
          <ReplicaNode
            key={i}
            position={pos}
            color={colors[i]}
            delay={i * 10}
            counterValue={counters[i]}
            pulsing={
              (i === 0 && frame >= 60 && frame <= 95) ||
              (i === 1 && frame >= 90 && frame <= 125) ||
              (i === 2 && frame >= 120 && frame <= 155)
            }
          />
        ))}

        <ConnectionLine from={nodePositions[0]} to={nodePositions[1]} opacity={lineOpacity} />
        <ConnectionLine from={nodePositions[1]} to={nodePositions[2]} opacity={lineOpacity} />
        <ConnectionLine from={nodePositions[0]} to={nodePositions[2]} opacity={lineOpacity} />

        <NetworkCloud entrance={lineOpacity} />

        {/* Sync particles after counters update */}
        <SyncParticle from={nodePositions[0]} to={nodePositions[1]} startFrame={155} color="#4a9eff" />
        <SyncParticle from={nodePositions[1]} to={nodePositions[2]} startFrame={165} color="#ff6a9e" />
        <SyncParticle from={nodePositions[2]} to={nodePositions[0]} startFrame={175} color="#6eff9e" />
      </ThreeCanvas>

      {/* 2D overlay labels */}
      <AbsoluteFill style={{ pointerEvents: "none" }}>
        {/* Node labels */}
        {[
          { left: "14%", top: "20%" },
          { right: "14%", top: "20%" },
          { left: "44%", bottom: "14%" },
        ].map((style, i) => (
          <div
            key={i}
            style={{
              position: "absolute",
              ...style,
              opacity: labelOpacity,
              transform: `translateY(${(1 - labelEntrance) * 10}px)`,
            } as React.CSSProperties}
          >
            <span style={{ fontFamily: FONT_PRIMARY, fontSize: 15, color: colors[i], fontWeight: 400 }}>
              {labels[i]}
            </span>
          </div>
        ))}

        {/* Counter labels next to nodes */}
        {frame >= 60 && (
          <div style={{ position: "absolute", left: "14%", top: "34%", opacity: interpolate(frame, [60, 75], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }) }}>
            <span style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.5)" }}>
              counter: {Math.round(counterA * 10)}
            </span>
          </div>
        )}
        {frame >= 90 && (
          <div style={{ position: "absolute", right: "14%", top: "34%", opacity: interpolate(frame, [90, 105], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }) }}>
            <span style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.5)" }}>
              counter: {Math.round(counterB * 10)}
            </span>
          </div>
        )}
        {frame >= 120 && (
          <div style={{ position: "absolute", left: "44%", bottom: "22%", opacity: interpolate(frame, [120, 135], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }) }}>
            <span style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.5)" }}>
              counter: {Math.round(counterC * 10)}
            </span>
          </div>
        )}

        {/* "No coordination needed" callout */}
        <div style={{ position: "absolute", top: 50, right: 60, opacity: noCoordOpacity }}>
          <div style={{
            fontFamily: FONT_PRIMARY,
            fontSize: 13,
            color: "#ffc46a",
            padding: "8px 16px",
            border: "1px solid rgba(255,196,106,0.3)",
            borderRadius: 6,
            background: "rgba(255,196,106,0.06)",
          }}>
            No coordination before writes
          </div>
        </div>

        {/* Title */}
        <div style={{ position: "absolute", top: 35, left: 60, opacity: descOpacity, transform: `translateY(${descY}px)` }}>
          <span style={{ fontFamily: FONT_DISPLAY, fontSize: 22, color: "rgba(255,255,255,0.9)" }}>
            Optimistic Replication
          </span>
        </div>

        {/* Bottom description */}
        <div style={{ position: "absolute", bottom: 35, left: 0, right: 0, textAlign: "center", opacity: descOpacity, transform: `translateY(${descY}px)` }}>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 16, color: "rgba(255,255,255,0.6)", margin: 0 }}>
            Each replica accepts writes locally — high availability, partition tolerant
          </p>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.3)", marginTop: 6 }}>
            CAP theorem: CRDTs choose Availability + Partition tolerance
          </p>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
