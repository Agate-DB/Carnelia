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
 * Scene 1 — Intro & Problem Statement
 *
 * Floating geometric primitives representing distributed data fragments.
 * Introduces the core question CRDTs answer: how can independent copies
 * stay consistent without coordination?
 *
 * AUDIO CUE: intro_narration.mp3
 */

const FloatingShape: React.FC<{
  position: [number, number, number];
  color: string;
  delay: number;
  geometry: "sphere" | "box" | "octahedron" | "torus" | "icosahedron";
  scale?: number;
}> = ({ position, color, delay, geometry, scale = 1 }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const entrance = spring({ frame, fps, delay, config: { damping: 12 } });
  const yFloat = Math.sin((frame - delay) * 0.03) * 0.15;
  const rotY = (frame - delay) * 0.012;
  const rotX = (frame - delay) * 0.008;
  const s = entrance * scale;

  return (
    <mesh
      position={[position[0], position[1] + yFloat * entrance, position[2]]}
      rotation={[rotX * entrance, rotY * entrance, 0]}
      scale={[s, s, s]}
    >
      {geometry === "sphere" && <sphereGeometry args={[0.4, 32, 32]} />}
      {geometry === "box" && <boxGeometry args={[0.55, 0.55, 0.55]} />}
      {geometry === "octahedron" && <octahedronGeometry args={[0.45]} />}
      {geometry === "torus" && <torusGeometry args={[0.35, 0.12, 16, 32]} />}
      {geometry === "icosahedron" && <icosahedronGeometry args={[0.4, 0]} />}
      <meshStandardMaterial
        color={color}
        roughness={0.25}
        metalness={0.7}
        transparent
        opacity={entrance}
        emissive={color}
        emissiveIntensity={0.15}
      />
    </mesh>
  );
};

/** Wireframe grid for depth perception */
const GridPlane: React.FC<{ opacity: number }> = ({ opacity }) => (
  <mesh rotation={[-Math.PI / 2, 0, 0]} position={[0, -2.2, -2]}>
    <planeGeometry args={[20, 20, 20, 20]} />
    <meshBasicMaterial color="#4a9eff" wireframe transparent opacity={opacity * 0.06} />
  </mesh>
);

/** Ambient particle field — small dots drifting slowly for depth */
const ParticleField: React.FC<{ entrance: number }> = ({ entrance }) => {
  const frame = useCurrentFrame();
  const particles = React.useMemo(() =>
    Array.from({ length: 30 }).map((_, i) => ({
      x: (Math.sin(i * 7.3) * 6) - 3,
      y: (Math.cos(i * 4.1) * 4) - 2,
      z: -2 - (i % 5) * 1.5,
      speed: 0.003 + (i % 3) * 0.002,
      size: 0.012 + (i % 4) * 0.005,
    })), []);

  return (
    <group>
      {particles.map((p, i) => (
        <mesh
          key={i}
          position={[
            p.x + Math.sin(frame * p.speed + i) * 0.3,
            p.y + Math.cos(frame * p.speed * 0.7 + i * 2) * 0.2,
            p.z,
          ]}
        >
          <sphereGeometry args={[p.size * entrance, 6, 6]} />
          <meshBasicMaterial color="#ffffff" transparent opacity={entrance * 0.12} />
        </mesh>
      ))}
    </group>
  );
};

export const IntroScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();

  const titleProgress = spring({ frame, fps, delay: 8, config: { damping: 200 } });
  const titleOpacity = interpolate(titleProgress, [0, 1], [0, 1]);
  const titleY = interpolate(titleProgress, [0, 1], [50, 0]);

  const subtitleProgress = spring({ frame, fps, delay: 30, config: { damping: 200 } });
  const subtitleOpacity = interpolate(subtitleProgress, [0, 1], [0, 1]);
  const subtitleY = interpolate(subtitleProgress, [0, 1], [25, 0]);

  const problemProgress = spring({ frame, fps, delay: 75, config: { damping: 200 } });
  const problemOpacity = interpolate(problemProgress, [0, 1], [0, 1]);
  const problemY = interpolate(problemProgress, [0, 1], [20, 0]);

  const gridOpacity = interpolate(frame, [0, 40], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const fadeOut = interpolate(frame, [185, 210], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill style={{ backgroundColor: "#1e1e1e", opacity: fadeOut }}>
      <ThreeCanvas linear width={width} height={height}>
        <color attach="background" args={["#1e1e1e"]} />
        <ambientLight intensity={0.5} color={0xffffff} />
        <pointLight position={[5, 5, 5]} intensity={1.2} color="#6ea0ff" />
        <pointLight position={[-5, -3, 3]} intensity={0.6} color="#a06eff" />
        <directionalLight position={[0, 5, 5]} intensity={0.3} />

        <GridPlane opacity={gridOpacity} />
        <ParticleField entrance={gridOpacity} />

        <FloatingShape position={[-3.0, 1.0, -1.5]} color="#4a9eff" delay={0} geometry="octahedron" scale={0.7} />
        <FloatingShape position={[3.2, 0.6, -2.0]} color="#ff6a9e" delay={4} geometry="icosahedron" scale={0.6} />
        <FloatingShape position={[-1.5, -1.0, -0.8]} color="#6eff9e" delay={8} geometry="box" scale={0.5} />
        <FloatingShape position={[2.0, -0.8, -1.2]} color="#ffc46a" delay={12} geometry="torus" scale={0.8} />
        <FloatingShape position={[0.3, 1.8, -2.5]} color="#a06eff" delay={6} geometry="sphere" scale={0.5} />
        <FloatingShape position={[-2.5, -0.3, -2.0]} color="#6affea" delay={16} geometry="icosahedron" scale={0.5} />
        <FloatingShape position={[3.5, 1.5, -3.0]} color="#ff9e6a" delay={10} geometry="box" scale={0.4} />
        <FloatingShape position={[0, -1.5, -1.0]} color="#4a9eff" delay={20} geometry="octahedron" scale={0.4} />
        <FloatingShape position={[-3.5, 0.5, -3.0]} color="#c9a0ff" delay={14} geometry="sphere" scale={0.35} />
      </ThreeCanvas>

      <AbsoluteFill style={{ justifyContent: "center", alignItems: "center", zIndex: 1 }}>
        <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 20, maxWidth: 1100, padding: "0 60px" }}>
          <h1
            style={{
              fontFamily: FONT_DISPLAY,
              fontSize: 72,
              fontWeight: 400,
              color: "white",
              opacity: titleOpacity,
              transform: `translateY(${titleY}px)`,
              textAlign: "center",
              letterSpacing: "-1px",
              margin: 0,
              textShadow: "0 0 60px rgba(74,158,255,0.35)",
            }}
          >
            How CRDTs Work
          </h1>
          <p
            style={{
              fontFamily: FONT_PRIMARY,
              fontSize: 18,
              color: "rgba(255,255,255,0.5)",
              opacity: subtitleOpacity,
              transform: `translateY(${subtitleY}px)`,
              textAlign: "center",
              margin: 0,
              letterSpacing: "5px",
              textTransform: "uppercase",
            }}
          >
            Conflict-Free Replicated Data Types
          </p>
          <p
            style={{
              fontFamily: FONT_PRIMARY,
              fontSize: 16,
              color: "rgba(255,255,255,0.4)",
              opacity: problemOpacity,
              transform: `translateY(${problemY}px)`,
              textAlign: "center",
              margin: 0,
              marginTop: 24,
              lineHeight: 1.8,
              maxWidth: 680,
            }}
          >
            How can distributed replicas update independently
            <br />
            and still converge to the same state — without coordination?
          </p>

          {/* Use-case pills */}
          <div
            style={{
              display: "flex",
              gap: 10,
              marginTop: 20,
              opacity: problemOpacity * 0.7,
              transform: `translateY(${problemY}px)`,
            }}
          >
            {["Collaborative Editors", "Offline-First Apps", "Distributed Databases", "P2P Networks"].map((label) => (
              <span
                key={label}
                style={{
                  fontFamily: FONT_PRIMARY,
                  fontSize: 11,
                  color: "rgba(255,255,255,0.35)",
                  padding: "4px 12px",
                  border: "1px solid rgba(255,255,255,0.08)",
                  borderRadius: 16,
                  background: "rgba(255,255,255,0.02)",
                }}
              >
                {label}
              </span>
            ))}
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
