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
 * Scene — Real-World CRDTs: Who Uses Them?
 *
 * Shows 5 real products that rely on CRDT-like technology:
 *   1. Figma — multiplayer cursors, real-time design collab
 *   2. Google Docs — OT + CRDT hybrid for collaborative editing
 *   3. Apple Notes — cross-device sync via CloudKit CRDTs
 *   4. Linear — real-time issue tracking with offline support
 *   5. VS Code Live Share — collaborative coding sessions
 *
 * Each product appears as a floating "card" with a 3D icon
 * and a brief description of how it uses CRDTs / convergence.
 *
 * AUDIO CUE: realworld_narration.mp3
 */

/** Floating 3D icon for each product */
const ProductIcon: React.FC<{
  position: [number, number, number];
  entrance: number;
  color: string;
  geometry: "box" | "sphere" | "torus" | "octahedron" | "dodecahedron";
  spin?: number;
}> = ({ position, entrance, color, geometry, spin = 0.005 }) => {
  const frame = useCurrentFrame();
  const s = entrance * 0.22;
  const yBob = Math.sin(frame * 0.025 + position[0] * 2) * 0.06;

  return (
    <group position={[position[0], position[1] + yBob * entrance, position[2]]}>
      <mesh
        scale={[s, s, s]}
        rotation={[0.3, frame * spin, 0.15]}
      >
        {geometry === "box" && <boxGeometry args={[1, 1, 1]} />}
        {geometry === "sphere" && <sphereGeometry args={[1, 24, 24]} />}
        {geometry === "torus" && <torusGeometry args={[0.7, 0.25, 12, 32]} />}
        {geometry === "octahedron" && <octahedronGeometry args={[1, 0]} />}
        {geometry === "dodecahedron" && <dodecahedronGeometry args={[1, 0]} />}
        <meshStandardMaterial
          color={color}
          roughness={0.15}
          metalness={0.8}
          emissive={color}
          emissiveIntensity={0.4}
          transparent
          opacity={entrance * 0.85}
        />
      </mesh>
      {/* Wireframe overlay */}
      <mesh
        scale={[s * 1.08, s * 1.08, s * 1.08]}
        rotation={[0.3, frame * spin, 0.15]}
      >
        {geometry === "box" && <boxGeometry args={[1, 1, 1]} />}
        {geometry === "sphere" && <sphereGeometry args={[1, 24, 24]} />}
        {geometry === "torus" && <torusGeometry args={[0.7, 0.25, 12, 32]} />}
        {geometry === "octahedron" && <octahedronGeometry args={[1, 0]} />}
        {geometry === "dodecahedron" && <dodecahedronGeometry args={[1, 0]} />}
        <meshBasicMaterial color={color} wireframe transparent opacity={entrance * 0.1} />
      </mesh>
    </group>
  );
};

/** Ambient connecting lines between product nodes */
const ConnectionWeb: React.FC<{ entrance: number }> = ({ entrance }) => {
  const positions: [number, number, number][] = [
    [-2.8, 0.8, 0], [2.8, 0.8, 0], [-1.8, -0.8, 0], [1.8, -0.8, 0], [0, -2.0, 0],
  ];

  const pairs = [
    [0, 1], [0, 2], [1, 3], [2, 3], [2, 4], [3, 4],
  ];

  return (
    <group>
      {pairs.map(([a, b], i) => {
        const from = positions[a];
        const to = positions[b];
        const dx = to[0] - from[0];
        const dy = to[1] - from[1];
        const len = Math.sqrt(dx * dx + dy * dy);
        return (
          <mesh
            key={i}
            position={[(from[0] + to[0]) / 2, (from[1] + to[1]) / 2, -0.2]}
            rotation={[0, 0, Math.atan2(dy, dx)]}
          >
            <boxGeometry args={[len, 0.005, 0.005]} />
            <meshBasicMaterial color="#ffffff" transparent opacity={entrance * 0.06} />
          </mesh>
        );
      })}
    </group>
  );
};

const products = [
  {
    name: "Figma",
    desc: "Multiplayer cursors & real-time design via custom CRDTs",
    approach: "Server-mediated CRDT",
    color: "#a259ff",
    geometry: "box" as const,
    pos: [-2.8, 0.8, 0] as [number, number, number],
    delay: 10,
    labelPos: { left: "4%", top: "18%" },
  },
  {
    name: "Google Docs",
    desc: "Operational Transformation + CRDT hybrid for text editing",
    approach: "OT → CRDT migration",
    color: "#4285f4",
    geometry: "sphere" as const,
    pos: [2.8, 0.8, 0] as [number, number, number],
    delay: 45,
    labelPos: { right: "4%", top: "18%" },
  },
  {
    name: "Apple Notes",
    desc: "Cross-device sync via CloudKit CRDTs for offline-first",
    approach: "OS-level CRDT sync",
    color: "#ffcc02",
    geometry: "torus" as const,
    pos: [-1.8, -0.8, 0] as [number, number, number],
    delay: 80,
    labelPos: { left: "8%", top: "55%" },
  },
  {
    name: "Linear",
    desc: "Real-time issue tracking with instant offline support",
    approach: "Offline-first CRDT",
    color: "#5e6ad2",
    geometry: "octahedron" as const,
    pos: [1.8, -0.8, 0] as [number, number, number],
    delay: 115,
    labelPos: { right: "8%", top: "55%" },
  },
  {
    name: "VS Code Live Share",
    desc: "Collaborative coding sessions with concurrent editing",
    approach: "Server + OT/CRDT",
    color: "#0078d4",
    geometry: "dodecahedron" as const,
    pos: [0, -2.0, 0] as [number, number, number],
    delay: 150,
    labelPos: { left: "50%", bottom: "8%", transform: "translateX(-50%)" },
  },
];

export const RealWorldCrdtScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();

  const titleOpacity = interpolate(frame, [5, 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Callout: common pattern
  const patternOpacity = interpolate(frame, [185, 205], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const patternY = interpolate(spring({ frame, fps, delay: 185, config: { damping: 200 } }), [0, 1], [12, 0]);

  // Web entrance
  const webEntrance = interpolate(frame, [30, 80], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  const fadeOut = interpolate(frame, [270, 300], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ backgroundColor: "#1e1e1e", opacity: fadeOut }}>
      <ThreeCanvas linear width={width} height={height}>
        <color attach="background" args={["#1e1e1e"]} />
        <ambientLight intensity={0.4} />
        <pointLight position={[5, 5, 5]} intensity={0.8} color="#6ea0ff" />
        <pointLight position={[-5, -3, 3]} intensity={0.5} color="#a06eff" />
        <directionalLight position={[0, 5, 5]} intensity={0.2} />

        <ConnectionWeb entrance={webEntrance} />

        {products.map((p, i) => {
          const ent = spring({ frame, fps, delay: p.delay, config: { damping: 14 } });
          return (
            <ProductIcon
              key={i}
              position={p.pos}
              entrance={ent}
              color={p.color}
              geometry={p.geometry}
            />
          );
        })}
      </ThreeCanvas>

      <AbsoluteFill style={{ pointerEvents: "none" }}>
        {/* Title */}
        <div style={{ position: "absolute", top: 35, left: 60, opacity: titleOpacity }}>
          <span style={{ fontFamily: FONT_DISPLAY, fontSize: 22, color: "rgba(255,255,255,0.9)" }}>
            CRDTs in the Real World
          </span>
        </div>

        {/* Product labels */}
        {products.map((p, i) => {
          const cardOpacity = interpolate(frame, [p.delay + 5, p.delay + 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
          const posStyle: React.CSSProperties = { position: "absolute", opacity: cardOpacity, maxWidth: 280 };

          // Apply position
          if ("left" in p.labelPos) posStyle.left = p.labelPos.left;
          if ("right" in p.labelPos) posStyle.right = p.labelPos.right;
          if ("top" in p.labelPos) posStyle.top = p.labelPos.top;
          if ("bottom" in p.labelPos) posStyle.bottom = p.labelPos.bottom;
          if ("transform" in p.labelPos) posStyle.transform = p.labelPos.transform;

          return (
            <div key={i} style={posStyle}>
              <div style={{
                fontFamily: FONT_PRIMARY, fontSize: 15, color: p.color, fontWeight: 500,
                marginBottom: 3,
              }}>
                {p.name}
              </div>
              <p style={{ fontFamily: FONT_PRIMARY, fontSize: 11, color: "rgba(255,255,255,0.5)", margin: 0, lineHeight: 1.5 }}>
                {p.desc}
              </p>
              <span style={{
                fontFamily: FONT_PRIMARY, fontSize: 10, color: "rgba(255,255,255,0.25)",
                marginTop: 3, display: "inline-block",
                padding: "2px 8px", border: "1px solid rgba(255,255,255,0.08)",
                borderRadius: 4,
              }}>
                {p.approach}
              </span>
            </div>
          );
        })}

        {/* Common pattern callout */}
        <div style={{
          position: "absolute", bottom: 45, left: 0, right: 0, textAlign: "center",
          opacity: patternOpacity, transform: `translateY(${patternY}px)`,
        }}>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 17, color: "white", margin: 0 }}>
            The pattern: <span style={{ color: "#6eff9e" }}>local-first</span> writes + <span style={{ color: "#c9a0ff" }}>automatic convergence</span>
          </p>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.35)", marginTop: 6 }}>
            But most rely on central servers — Carnelia goes fully peer-to-peer
          </p>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
