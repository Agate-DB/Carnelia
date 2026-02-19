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
 * Phase 1 Cutscene — "Understanding CRDTs"
 *
 * Brief transition card indicating we are entering Phase 1:
 * explaining the core concepts of traditional CRDTs.
 *
 * Timeline (120 frames @ 20fps = 6s):
 *   0–20:   Background wipe + phase number
 *   15–40:  Title text entrance
 *   35–60:  Subtitle entrance
 *   90–120: Fade out
 */

const BRAND = "#e06040";
const ACCENT_BLUE = "#4a9eff";
const BG = "#1e1e1e";

export const Phase1CutScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  /* ── Animations ── */
  const wipeProgress = interpolate(frame, [0, 18], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const phaseNumEnt = spring({ frame, fps, delay: 5, config: { damping: 14 } });
  const titleEnt = spring({ frame, fps, delay: 15, config: { damping: 16 } });
  const subtitleEnt = spring({ frame, fps, delay: 35, config: { damping: 18 } });

  const fadeOut = interpolate(frame, [95, 200], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  /* Decorative bar width */
  const barWidth = interpolate(frame, [10, 50], [0, 320], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  /* Floating dots animation */
  const dots = Array.from({ length: 8 }, (_, i) => {
    const angle = (i / 8) * Math.PI * 2 + frame * 0.01;
    const radius = 180 + Math.sin(i * 1.3 + frame * 0.02) * 30;
    return {
      x: Math.cos(angle) * radius,
      y: Math.sin(angle) * radius,
      opacity: interpolate(frame, [10 + i * 3, 25 + i * 3], [0, 0.4], {
        extrapolateLeft: "clamp",
        extrapolateRight: "clamp",
      }),
      size: 4 + (i % 3) * 2,
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
      {/* Background accent wipe */}
      <div
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          width: `${wipeProgress * 100}%`,
          height: "100%",
          background: `linear-gradient(90deg, rgba(74, 154, 255, 0.06) 0%, transparent 100%)`,
        }}
      />

      {/* Vertical accent line */}
      <div
        style={{
          position: "absolute",
          left: 80,
          top: "50%",
          transform: "translateY(-50%)",
          width: 3,
          height: interpolate(phaseNumEnt, [0, 1], [0, 140]),
          background: ACCENT_BLUE,
          borderRadius: 2,
          opacity: 0.7,
        }}
      />

      {/* Floating dots */}
      {dots.map((dot, i) => (
        <div
          key={i}
          style={{
            position: "absolute",
            left: `calc(50% + ${dot.x}px)`,
            top: `calc(50% + ${dot.y}px)`,
            width: dot.size,
            height: dot.size,
            borderRadius: "50%",
            backgroundColor: i % 2 === 0 ? ACCENT_BLUE : BRAND,
            opacity: dot.opacity * fadeOut,
          }}
        />
      ))}

      {/* Central content */}
      <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 12, zIndex: 1 }}>
        {/* Phase number */}
        <div
          style={{
            fontFamily: FONT_DISPLAY,
            fontSize: 14,
            letterSpacing: 6,
            textTransform: "uppercase",
            color: ACCENT_BLUE,
            opacity: phaseNumEnt,
            transform: `translateY(${(1 - phaseNumEnt) * -15}px)`,
          }}
        >
          Phase 1
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
          Understanding CRDTs
        </div>

        {/* Decorative bar */}
        <div
          style={{
            width: barWidth,
            height: 2,
            background: `linear-gradient(90deg, transparent, ${ACCENT_BLUE}, transparent)`,
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
          }}
        >
          How conflict-free data types enable coordination-free distributed systems
        </div>
      </div>
    </AbsoluteFill>
  );
};
