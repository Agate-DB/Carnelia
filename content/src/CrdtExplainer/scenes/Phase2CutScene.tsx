import React from "react";
import {
  AbsoluteFill,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { FONT_PRIMARY, FONT_DISPLAY } from "../fonts";

/**
 * Phase 2 Cutscene — "Our Solution: The MDCS"
 *
 * Transition card for Phase 2: introducing the Merkle-Delta CRDT Store
 * as the solution to the limitations discussed in Phase 1.
 *
 * Timeline (120 frames @ 20fps = 6s):
 *   0–18:   Accent wipe from right
 *   5–40:   Phase number + title entrance
 *   35–70:  Three pillars animate in (Delta, Merkle, DotStore)
 *   90–120: Fade out
 */

const BRAND = "#e06040";
const ACCENT_GREEN = "#6eff9e";
const BG = "#1e1e1e";

const PILLARS = [
  { icon: "δ", label: "Delta CRDTs", color: "#4a9eff" },
  { icon: "#", label: "Merkle-Clock", color: BRAND },
  { icon: "◆", label: "Dot Store", color: ACCENT_GREEN },
];

export const Phase2CutScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  /* ── Animations ── */
  const wipeProgress = interpolate(frame, [0, 18], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const phaseNumEnt = spring({ frame, fps, delay: 5, config: { damping: 14 } });
  const titleEnt = spring({ frame, fps, delay: 15, config: { damping: 16 } });
  const subtitleEnt = spring({ frame, fps, delay: 30, config: { damping: 18 } });

  const fadeOut = interpolate(frame, [95, 280], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  /* Decorative bar */
  const barWidth = interpolate(frame, [10, 50], [0, 280], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  /* Hex grid decoration */
  const hexes = Array.from({ length: 12 }, (_, i) => {
    const col = i % 4;
    const row = Math.floor(i / 4);
    const stagger = row % 2 === 0 ? 0 : 35;
    return {
      x: col * 70 + stagger - 260,
      y: row * 60 - 80,
      opacity: interpolate(frame, [8 + i * 2, 22 + i * 2], [0, 0.08], {
        extrapolateLeft: "clamp",
        extrapolateRight: "clamp",
      }),
    };
  });

  return (
    <AbsoluteFill
      style={{
        backgroundColor: BG,
        opacity: fadeOut,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      {/* Background accent wipe — from right */}
      <div
        style={{
          position: "absolute",
          top: 0,
          right: 0,
          width: `${wipeProgress * 100}%`,
          height: "100%",
          background: `linear-gradient(270deg, rgba(224, 96, 64, 0.06) 0%, transparent 100%)`,
        }}
      />

      {/* Hex grid decoration */}
      {hexes.map((hex, i) => (
        <div
          key={i}
          style={{
            position: "absolute",
            left: `calc(50% + ${hex.x}px)`,
            top: `calc(50% + ${hex.y}px)`,
            width: 40,
            height: 40,
            border: `1px solid ${BRAND}`,
            borderRadius: 6,
            opacity: hex.opacity * fadeOut,
            transform: "rotate(45deg)",
          }}
        />
      ))}

      {/* Vertical accent line — right side */}
      <div
        style={{
          position: "absolute",
          right: 80,
          top: "50%",
          transform: "translateY(-50%)",
          width: 3,
          height: interpolate(phaseNumEnt, [0, 1], [0, 140]),
          background: BRAND,
          borderRadius: 2,
          opacity: 0.7,
        }}
      />

      {/* Central content */}
      <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 12, zIndex: 1 }}>
        {/* Phase number */}
        <div
          style={{
            fontFamily: FONT_DISPLAY,
            fontSize: 14,
            letterSpacing: 6,
            textTransform: "uppercase",
            color: BRAND,
            opacity: phaseNumEnt,
            transform: `translateY(${(1 - phaseNumEnt) * -15}px)`,
          }}
        >
          Phase 2
        </div>

        {/* Title */}
        <div
          style={{
            fontFamily: FONT_DISPLAY,
            fontSize: 36,
            color: "rgba(255, 255, 255, 0.95)",
            opacity: titleEnt,
            transform: `translateY(${(1 - titleEnt) * 20}px)`,
            textAlign: "center",
          }}
        >
          Our Solution: The MDCS
        </div>

        {/* Decorative bar */}
        <div
          style={{
            width: barWidth,
            height: 2,
            background: `linear-gradient(90deg, transparent, ${BRAND}, transparent)`,
            borderRadius: 1,
            opacity: 0.5,
          }}
        />

        {/* Subtitle */}
        <div
          style={{
            fontFamily: FONT_PRIMARY,
            fontSize: 15,
            color: "rgba(255, 255, 255, 0.5)",
            opacity: subtitleEnt,
            transform: `translateY(${(1 - subtitleEnt) * 10}px)`,
            textAlign: "center",
            maxWidth: 450,
            lineHeight: 1.6,
            marginBottom: 16,
          }}
        >
          Merkle-Delta CRDT Store — efficient, verifiable, tombstone-free
        </div>

        {/* Three pillars */}
        <div style={{ display: "flex", gap: 24, marginTop: 8 }}>
          {PILLARS.map((p, i) => {
            const ent = spring({ frame, fps, delay: 40 + i * 10, config: { damping: 14 } });
            return (
              <div
                key={i}
                style={{
                  display: "flex",
                  flexDirection: "column",
                  alignItems: "center",
                  gap: 6,
                  opacity: ent,
                  transform: `translateY(${(1 - ent) * 20}px) scale(${0.8 + ent * 0.2})`,
                }}
              >
                <div
                  style={{
                    width: 48,
                    height: 48,
                    borderRadius: 12,
                    border: `1.5px solid ${p.color}`,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    background: `${p.color}0a`,
                  }}
                >
                  <span style={{ fontFamily: FONT_DISPLAY, fontSize: 22, color: p.color }}>{p.icon}</span>
                </div>
                <span
                  style={{
                    fontFamily: FONT_PRIMARY,
                    fontSize: 11,
                    color: "rgba(255,255,255,0.5)",
                    textAlign: "center",
                    whiteSpace: "nowrap",
                  }}
                >
                  {p.label}
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </AbsoluteFill>
  );
};
