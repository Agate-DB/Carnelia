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
 * Scene 15 — End Screen / Summary
 *
 * Summarizes what was covered, reiterates Carnelia's value, and closes.
 *
 * Timeline (230 frames @ 20fps = 11.5s):
 *   0–30:    Title fade in — "What We Covered"
 *   20–120:  Key takeaways appear one by one
 *   120–170: Carnelia value prop block
 *   155–210: CTA / links
 *   200–230: Fade out
 *
 * AUDIO CUE: end_narration.mp3
 */

const BRAND = "#e06040";
const BG = "#1e1e1e";

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
  const fadeOut = interpolate(frame, [250, 270], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

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
            top: 36,
            left: 0,
            right: 0,
            textAlign: "center",
            opacity: titleEnt,
            transform: `translateY(${(1 - titleEnt) * 12}px)`,
          }}
        >
          <span style={{ fontFamily: FONT_DISPLAY, fontSize: 26, color: "rgba(255,255,255,0.9)" }}>
            What We Covered
          </span>
        </div>

        {/* Key takeaways — staggered entrance */}
        <div
          style={{
            position: "absolute",
            top: 80,
            left: "50%",
            transform: "translateX(-50%)",
            display: "flex",
            flexDirection: "column",
            gap: 6,
            width: 520,
          }}
        >
          {TAKEAWAYS.map((t, i) => {
            const ent = spring({ frame, fps, delay: 25 + i * 16, config: { damping: 14 } });
            return (
              <div
                key={i}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 12,
                  opacity: ent,
                  transform: `translateX(${(1 - ent) * 30}px)`,
                  background: "rgba(255,255,255,0.03)",
                  border: "1px solid rgba(255,255,255,0.06)",
                  borderRadius: 8,
                  padding: "6px 14px",
                }}
              >
                <span style={{ fontFamily: FONT_DISPLAY, fontSize: 22, color: BRAND, width: 24, textAlign: "center" }}>
                  {t.icon}
                </span>
                <div>
                  <span style={{ fontFamily: FONT_DISPLAY, fontSize: 16, color: "rgba(255,255,255,0.8)" }}>
                    {t.label}
                  </span>
                  <span style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.35)", marginLeft: 8 }}>
                    {t.desc}
                  </span>
                </div>
              </div>
            );
          })}
        </div>

        {/* Carnelia value prop */}
        <div
          style={{
            position: "absolute",
            bottom: 100,
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
              padding: "14px 32px",
              maxWidth: 620,
            }}
          >
            <div style={{ fontFamily: FONT_DISPLAY, fontSize: 22, color: BRAND, marginBottom: 6 }}>
              Carnelia — Merkle-Delta CRDT Store
            </div>
            <div style={{ fontFamily: FONT_PRIMARY, fontSize: 14, color: "rgba(255,255,255,0.55)", lineHeight: 1.6 }}>
              Open-membership · Offline-first · Peer-to-peer · Byzantine-tolerant
            </div>
            <div style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.3)", marginTop: 6 }}>
              Strong eventual consistency without consensus.
            </div>
          </div>
        </div>

        {/* CTA / repo link */}
        <div
          style={{
            position: "absolute",
            bottom: 40,
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
