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
 * Scene — End Screen / Conclusion
 *
 * Three-phase recap + Carnelia value prop + CTA.
 * Phases: 1) Understanding CRDTs, 2) The MDCS Solution, 3) Real-World Impact
 *
 * Timeline (400 frames @ 20fps = 20s):
 *   0–30:    Title fade in — "The Journey So Far"
 *   15–60:   Phase recap cards appear
 *   60–120:  Key takeaway pills
 *   120–170: Carnelia value prop block
 *   155–210: CTA / links
 *   370–400: Fade out
 *
 * AUDIO CUE: conclusion_narration.mp3
 */

const BRAND = "#e06040";
const BG = "#1e1e1e";

const PHASES = [
  { num: "1", title: "Understanding CRDTs", desc: "Semilattices, G-Counters, conflict-free merge", color: "#4a9eff" },
  { num: "2", title: "The MDCS Solution", desc: "Delta CRDTs · Merkle-Clock · Dot Store", color: BRAND },
  { num: "3", title: "Real-World Impact", desc: "Offline sync · Collab editing · P2P apps", color: "#6affea" },
];

const TAKEAWAYS = [
  { icon: "⊔", label: "Join-Semilattice", desc: "Commutative, associative, idempotent merge" },
  { icon: "δ", label: "Delta CRDTs", desc: "Bandwidth-efficient incremental mutations" },
  { icon: "◆", label: "Dot Store", desc: "Tombstone-free deletion via causal context" },
  { icon: "#", label: "Merkle-Clock", desc: "Immutable, verifiable causal DAG" },
  { icon: "↔", label: "Anti-Entropy", desc: "Partition-tolerant gossip sync" },
];

/* Ambient floating particle */
const Particle: React.FC<{
  seed: number;
  color: string;
}> = ({ seed, color }) => {
  const frame = useCurrentFrame();
  const x = Math.sin(seed * 1.7 + frame * 0.006) * 3.5;
  const y = Math.cos(seed * 2.3 + frame * 0.008) * 2;
  const z = Math.sin(seed * 0.9 + frame * 0.004) * 2 - 1;
  const s = 0.015 + (seed % 3) * 0.008;

  return (
    <mesh position={[x, y, z]}>
      <sphereGeometry args={[s, 6, 6]} />
      <meshBasicMaterial color={color} transparent opacity={0.15} />
    </mesh>
  );
};

export const EndScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();

  /* ── entrances ── */
  const titleEnt = spring({ frame, fps, delay: 5, config: { damping: 16 } });
  const fadeOut = interpolate(frame, [370, 400], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  /* Carnelia block entrance */
  const carneliEnt = spring({ frame, fps, delay: 120, config: { damping: 14 } });
  const ctaEnt = spring({ frame, fps, delay: 155, config: { damping: 14 } });

  return (
    <AbsoluteFill style={{ backgroundColor: BG, opacity: fadeOut }}>
      <ThreeCanvas linear width={width} height={height}>
        <color attach="background" args={[BG]} />
        <ambientLight intensity={0.25} />
        <pointLight position={[3, 3, 4]} intensity={0.5} color="#7ecfff" />
        <pointLight position={[-3, -2, 3]} intensity={0.4} color={BRAND} />

        {/* Ambient particles */}
        {Array.from({ length: 20 }, (_, i) => (
          <Particle key={i} seed={i * 3.14} color={i % 2 === 0 ? BRAND : "#7ecfff"} />
        ))}

        {/* Central Carnelia emblem — slow rotating icosahedron */}
        <mesh
          rotation={[frame * 0.003, frame * 0.005, 0.2]}
          scale={[carneliEnt * 0.6, carneliEnt * 0.6, carneliEnt * 0.6]}
          position={[0, -0.3, 0]}
        >
          <icosahedronGeometry args={[1, 1]} />
          <meshStandardMaterial
            color={BRAND}
            roughness={0.2}
            metalness={0.7}
            emissive={BRAND}
            emissiveIntensity={0.3}
            transparent
            opacity={carneliEnt * 0.25}
          />
        </mesh>

        {/* Orbit ring */}
        <mesh
          rotation={[Math.PI / 2.5, 0, frame * 0.004]}
          scale={[carneliEnt * 1.6, carneliEnt * 1.6, carneliEnt * 1.6]}
          position={[0, -0.3, 0]}
        >
          <torusGeometry args={[0.8, 0.006, 8, 50]} />
          <meshBasicMaterial color={BRAND} transparent opacity={carneliEnt * 0.12} />
        </mesh>
      </ThreeCanvas>

      {/* ── 2D overlay ── */}
      <AbsoluteFill style={{ pointerEvents: "none" }}>
        {/* Title */}
        <div
          style={{
            position: "absolute",
            top: 24,
            left: 0,
            right: 0,
            textAlign: "center",
            opacity: titleEnt,
            transform: `translateY(${(1 - titleEnt) * 12}px)`,
          }}
        >
          <span style={{ fontFamily: FONT_DISPLAY, fontSize: 26, color: "rgba(255,255,255,0.9)" }}>
            The Journey So Far
          </span>
        </div>

        {/* Phase recap — three columns */}
        <div
          style={{
            position: "absolute",
            top: 64,
            left: "50%",
            transform: "translateX(-50%)",
            display: "flex",
            gap: 18,
          }}
        >
          {PHASES.map((phase, i) => {
            const ent = spring({ frame, fps, delay: 15 + i * 14, config: { damping: 14 } });
            return (
              <div
                key={i}
                style={{
                  opacity: ent,
                  transform: `translateY(${(1 - ent) * 20}px)`,
                  background: "rgba(255,255,255,0.025)",
                  border: `1px solid ${phase.color}22`,
                  borderRadius: 10,
                  padding: "10px 16px",
                  width: 166,
                  textAlign: "center",
                }}
              >
                <div style={{ fontFamily: FONT_DISPLAY, fontSize: 11, letterSpacing: 3, color: phase.color, marginBottom: 4, textTransform: "uppercase" }}>
                  Phase {phase.num}
                </div>
                <div style={{ fontFamily: FONT_DISPLAY, fontSize: 14, color: "rgba(255,255,255,0.85)", marginBottom: 4 }}>
                  {phase.title}
                </div>
                <div style={{ fontFamily: FONT_PRIMARY, fontSize: 10, color: "rgba(255,255,255,0.35)", lineHeight: 1.5 }}>
                  {phase.desc}
                </div>
              </div>
            );
          })}
        </div>

        {/* Key takeaways — compact row below phases */}
        <div
          style={{
            position: "absolute",
            top: 192,
            left: "50%",
            transform: "translateX(-50%)",
            display: "flex",
            flexWrap: "wrap",
            gap: 6,
            justifyContent: "center",
            maxWidth: 560,
          }}
        >
          {TAKEAWAYS.map((t, i) => {
            const ent = spring({ frame, fps, delay: 60 + i * 10, config: { damping: 14 } });
            return (
              <div
                key={i}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                  opacity: ent,
                  transform: `translateX(${(1 - ent) * 20}px)`,
                  background: "rgba(255,255,255,0.03)",
                  border: "1px solid rgba(255,255,255,0.06)",
                  borderRadius: 16,
                  padding: "4px 12px",
                }}
              >
                <span style={{ fontFamily: FONT_DISPLAY, fontSize: 16, color: BRAND, width: 18, textAlign: "center" }}>
                  {t.icon}
                </span>
                <span style={{ fontFamily: FONT_PRIMARY, fontSize: 11, color: "rgba(255,255,255,0.55)" }}>
                  {t.label}
                </span>
              </div>
            );
          })}
        </div>

        {/* Carnelia value prop */}
        <div
          style={{
            position: "absolute",
            bottom: 88,
            left: 0,
            right: 0,
            textAlign: "center",
            opacity: carneliEnt,
            transform: `translateY(${(1 - carneliEnt) * 15}px)`,
          }}
        >
          <div
            style={{
              display: "inline-block",
              background: `rgba(224, 96, 64, 0.06)`,
              border: `1px solid rgba(224, 96, 64, 0.18)`,
              borderRadius: 12,
              padding: "12px 28px",
              maxWidth: 600,
            }}
          >
            <div style={{ fontFamily: FONT_DISPLAY, fontSize: 20, color: BRAND, marginBottom: 4 }}>
              Carnelia — Merkle-Delta CRDT Store
            </div>
            <div style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.55)", lineHeight: 1.6 }}>
              Open-membership · Offline-first · Peer-to-peer · Byzantine-tolerant
            </div>
            <div style={{ fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.3)", marginTop: 4 }}>
              From BirdWatch's coordination bottleneck to strong eventual consistency — without consensus.
            </div>
          </div>
        </div>

        {/* CTA / repo link */}
        <div
          style={{
            position: "absolute",
            bottom: 36,
            left: 0,
            right: 0,
            textAlign: "center",
            opacity: ctaEnt,
          }}
        >
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 16, color: "rgba(255,255,255,0.4)" }}>
            github.com/Agate-DB/Carnelia
          </span>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
