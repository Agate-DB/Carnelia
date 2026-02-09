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
 * Scene 0 — Presented By Carnelia
 *
 * Cinematic opening title card. A central glowing crystal (icosahedron)
 * rotates slowly while concentric rings expand outward. The "Carnelia"
 * name fades in with a subtitle describing the project.
 *
 * AUDIO CUE: (silence or ambient drone)
 */

/** Central crystal — slowly rotating icosahedron with inner glow */
const CarnelianCrystal: React.FC<{ entrance: number }> = ({ entrance }) => {
  const frame = useCurrentFrame();
  const s = entrance * 0.55;
  const breathe = 1 + Math.sin(frame * 0.025) * 0.04;

  return (
    <group>
      {/* Core crystal */}
      <mesh
        scale={[s * breathe, s * breathe, s * breathe]}
        rotation={[frame * 0.005, frame * 0.007, frame * 0.003]}
      >
        <icosahedronGeometry args={[1, 1]} />
        <meshStandardMaterial
          color="#e06040"
          roughness={0.08}
          metalness={0.9}
          emissive="#e06040"
          emissiveIntensity={0.6 * entrance}
          transparent
          opacity={entrance * 0.85}
        />
      </mesh>
      {/* Wireframe shell */}
      <mesh
        scale={[s * breathe * 1.15, s * breathe * 1.15, s * breathe * 1.15]}
        rotation={[frame * 0.005, frame * 0.007, frame * 0.003]}
      >
        <icosahedronGeometry args={[1, 1]} />
        <meshBasicMaterial color="#e06040" wireframe transparent opacity={entrance * 0.12} />
      </mesh>
      {/* Inner glow sphere */}
      <mesh scale={[s * 0.35, s * 0.35, s * 0.35]}>
        <sphereGeometry args={[1, 16, 16]} />
        <meshBasicMaterial color="#ff9070" transparent opacity={entrance * 0.3} />
      </mesh>
    </group>
  );
};

/** Expanding ring pulse */
const PulseRing: React.FC<{
  delay: number;
  entrance: number;
  color: string;
}> = ({ delay, entrance, color }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const expand = spring({ frame, fps, delay, config: { damping: 30, mass: 2 } });
  const radius = 0.6 + expand * 2.5;
  const opacity = Math.max(0, (1 - expand * 0.7) * entrance * 0.25);

  if (opacity < 0.005) return null;

  return (
    <mesh rotation={[Math.PI / 2, 0, 0]}>
      <torusGeometry args={[radius, 0.008, 8, 64]} />
      <meshStandardMaterial
        color={color}
        emissive={color}
        emissiveIntensity={0.5}
        transparent
        opacity={opacity}
      />
    </mesh>
  );
};

/** Orbiting particle */
const OrbitParticle: React.FC<{
  radius: number;
  speed: number;
  offset: number;
  color: string;
  entrance: number;
}> = ({ radius, speed, offset, color, entrance }) => {
  const frame = useCurrentFrame();
  const angle = frame * speed + offset;
  const x = Math.cos(angle) * radius * entrance;
  const z = Math.sin(angle) * radius * entrance;
  const y = Math.sin(angle * 0.7 + offset) * 0.3 * entrance;

  return (
    <mesh position={[x, y, z]}>
      <sphereGeometry args={[0.025 * entrance, 8, 8]} />
      <meshStandardMaterial
        color={color}
        emissive={color}
        emissiveIntensity={1.5}
        transparent
        opacity={entrance * 0.7}
      />
    </mesh>
  );
};

/** Grid floor for depth */
const GridFloor: React.FC<{ opacity: number }> = ({ opacity }) => (
  <mesh rotation={[-Math.PI / 2, 0, 0]} position={[0, -2.5, -1]}>
    <planeGeometry args={[24, 24, 24, 24]} />
    <meshBasicMaterial color="#e06040" wireframe transparent opacity={opacity * 0.03} />
  </mesh>
);

export const PresentedByScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();

  const crystalEnt = spring({ frame, fps, delay: 10, config: { damping: 18 } });
  const gridOpacity = interpolate(frame, [0, 30], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Text animations
  const presentedByOpacity = interpolate(frame, [20, 45], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const presentedByY = interpolate(spring({ frame, fps, delay: 20, config: { damping: 200 } }), [0, 1], [20, 0]);

  const nameProgress = spring({ frame, fps, delay: 40, config: { damping: 14 } });
  const nameOpacity = interpolate(nameProgress, [0, 1], [0, 1]);
  const nameScale = interpolate(nameProgress, [0, 1], [0.85, 1]);

  const taglineOpacity = interpolate(frame, [70, 95], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const taglineY = interpolate(spring({ frame, fps, delay: 70, config: { damping: 200 } }), [0, 1], [15, 0]);

  const techOpacity = interpolate(frame, [100, 120], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Fade out
  const fadeOut = interpolate(frame, [195, 220], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ backgroundColor: "#1e1e1e", opacity: fadeOut }}>
      <ThreeCanvas linear width={width} height={height}>
        <color attach="background" args={["#1e1e1e"]} />
        <ambientLight intensity={0.3} />
        <pointLight position={[5, 5, 5]} intensity={1} color="#e08060" />
        <pointLight position={[-5, -3, 3]} intensity={0.5} color="#a06eff" />
        <pointLight position={[0, 0, 4]} intensity={0.6} color="#ff7050" />
        <directionalLight position={[0, 5, 5]} intensity={0.2} />

        <GridFloor opacity={gridOpacity} />
        <CarnelianCrystal entrance={crystalEnt} />

        {/* Pulse rings */}
        <PulseRing delay={30} entrance={crystalEnt} color="#e06040" />
        <PulseRing delay={50} entrance={crystalEnt} color="#ff9070" />
        <PulseRing delay={70} entrance={crystalEnt} color="#e06040" />

        {/* Orbiting particles */}
        <OrbitParticle radius={1.5} speed={0.015} offset={0} color="#e06040" entrance={crystalEnt} />
        <OrbitParticle radius={1.8} speed={0.012} offset={2.1} color="#ff9070" entrance={crystalEnt} />
        <OrbitParticle radius={1.3} speed={0.018} offset={4.2} color="#ffc46a" entrance={crystalEnt} />
        <OrbitParticle radius={2.0} speed={0.01} offset={1.0} color="#a06eff" entrance={crystalEnt} />
        <OrbitParticle radius={1.6} speed={0.014} offset={3.5} color="#4a9eff" entrance={crystalEnt} />
        <OrbitParticle radius={2.2} speed={0.008} offset={5.0} color="#6eff9e" entrance={crystalEnt} />
      </ThreeCanvas>

      <AbsoluteFill style={{ justifyContent: "center", alignItems: "center", zIndex: 1, pointerEvents: "none" }}>
        <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 0 }}>
          {/* "Presented by" */}
          <p
            style={{
              fontFamily: FONT_PRIMARY,
              fontSize: 17,
              color: "rgba(255,255,255,0.4)",
              opacity: presentedByOpacity,
              transform: `translateY(${presentedByY}px)`,
              letterSpacing: "6px",
              textTransform: "uppercase",
              margin: 0,
              marginBottom: 16,
            }}
          >
            Presented by Agate.
          </p>

          {/* "Carnelia" */}
          <h1
            style={{
              fontFamily: FONT_DISPLAY,
              fontSize: 115,
              fontWeight: 400,
              color: "#e06040",
              opacity: nameOpacity,
              transform: `scale(${nameScale})`,
              textAlign: "center",
              letterSpacing: "2px",
              margin: 0,
              textShadow: "0 0 80px rgba(224,96,64,0.5), 0 0 160px rgba(224,96,64,0.2)",
            }}
          >
            Carnelia
          </h1>

          {/* Tagline */}
          <p
            style={{
              fontFamily: FONT_PRIMARY,
              fontSize: 20,
              color: "rgba(255,255,255,0.55)",
              opacity: taglineOpacity,
              transform: `translateY(${taglineY}px)`,
              textAlign: "center",
              margin: 0,
              marginTop: 20,
              letterSpacing: "1px",
            }}
          >
            Merkle-Delta CRDT Store
          </p>

          {/* Tech pills */}
          <div
            style={{
              display: "flex",
              gap: 12,
              marginTop: 28,
              opacity: techOpacity,
            }}
          >
            {["δ-CRDT", "Merkle-Clock", "Dot Store", "DAG-Syncer"].map((label) => (
              <span
                key={label}
                style={{
                  fontFamily: FONT_PRIMARY,
                  fontSize: 13,
                  color: "rgba(224,96,64,0.85)",
                  padding: "5px 14px",
                  border: "1px solid rgba(224,96,64,0.2)",
                  borderRadius: 20,
                  background: "rgba(224,96,64,0.04)",
                  letterSpacing: "0.5px",
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
