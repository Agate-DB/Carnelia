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
 * Phase 3 Cutscene — "Real-World Applications"
 *
 * Transition card for Phase 3: discussing real-world scenarios
 * where the MDCS / Carnelia can solve actual problems.
 *
 * Timeline (120 frames @ 20fps = 6s):
 *   0–18:   Dual accent wipe
 *   5–40:   Phase number + title entrance
 *   35–70:  Use-case tags animate in
 *   90–120: Fade out
 */

const BRAND = "#e06040";
const ACCENT_TEAL = "#6affea";
const ACCENT_GOLD = "#ffc46a";
const BG = "#1e1e1e";

const USE_CASES = [
  { label: "Offline-First Sync", icon: "📱" },
  { label: "Collaborative Editing", icon: "✏️" },
  { label: "Partition Recovery", icon: "🔄" },
  { label: "Peer-to-Peer Apps", icon: "🌐" },
];

export const Phase3CutScene: React.FC = () => {
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

  const fadeOut = interpolate(frame, [95, 150], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  /* Decorative bar */
  const barWidth = interpolate(frame, [10, 50], [0, 300], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  /* Connection lines animation */
  const connectionOpacity = interpolate(frame, [20, 40], [0, 0.12], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  /* Radial dots */
  const dots = Array.from({ length: 16 }, (_, i) => {
    const angle = (i / 16) * Math.PI * 2 + frame * 0.008;
    const radius = 200 + Math.sin(i * 2.1 + frame * 0.015) * 40;
    return {
      x: Math.cos(angle) * radius,
      y: Math.sin(angle) * radius,
      opacity: interpolate(frame, [5 + i * 2, 20 + i * 2], [0, 0.3], {
        extrapolateLeft: "clamp",
        extrapolateRight: "clamp",
      }),
      size: 3 + (i % 4),
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
      {/* Dual accent wipes */}
      <div
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          width: `${wipeProgress * 45}%`,
          height: "100%",
          background: `linear-gradient(90deg, rgba(106, 255, 234, 0.04) 0%, transparent 100%)`,
        }}
      />
      <div
        style={{
          position: "absolute",
          top: 0,
          right: 0,
          width: `${wipeProgress * 45}%`,
          height: "100%",
          background: `linear-gradient(270deg, rgba(255, 196, 106, 0.04) 0%, transparent 100%)`,
        }}
      />

      {/* Radial dots */}
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
            backgroundColor: i % 3 === 0 ? ACCENT_TEAL : i % 3 === 1 ? ACCENT_GOLD : BRAND,
            opacity: dot.opacity * fadeOut,
          }}
        />
      ))}

      {/* Cross connection lines */}
      <svg
        style={{ position: "absolute", width: "100%", height: "100%", pointerEvents: "none" }}
        viewBox="0 0 640 360"
      >
        <line x1="200" y1="180" x2="440" y2="180" stroke={ACCENT_TEAL} strokeWidth={0.5} opacity={connectionOpacity} />
        <line x1="320" y1="100" x2="320" y2="260" stroke={ACCENT_GOLD} strokeWidth={0.5} opacity={connectionOpacity} />
        <line x1="230" y1="120" x2="410" y2="240" stroke={BRAND} strokeWidth={0.5} opacity={connectionOpacity * 0.7} />
        <line x1="410" y1="120" x2="230" y2="240" stroke={BRAND} strokeWidth={0.5} opacity={connectionOpacity * 0.7} />
      </svg>

      {/* Central content */}
      <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 12, zIndex: 1 }}>
        {/* Phase number */}
        <div
          style={{
            fontFamily: FONT_DISPLAY,
            fontSize: 14,
            letterSpacing: 6,
            textTransform: "uppercase",
            color: ACCENT_TEAL,
            opacity: phaseNumEnt,
            transform: `translateY(${(1 - phaseNumEnt) * -15}px)`,
          }}
        >
          Phase 3
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
          Real-World Applications
        </div>

        {/* Decorative bar */}
        <div
          style={{
            width: barWidth,
            height: 2,
            background: `linear-gradient(90deg, ${ACCENT_TEAL}, transparent, ${ACCENT_GOLD})`,
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
          Scenarios where Carnelia solves real distributed systems challenges
        </div>

        {/* Use-case tags */}
        <div style={{ display: "flex", gap: 14, flexWrap: "wrap", justifyContent: "center" }}>
          {USE_CASES.map((uc, i) => {
            const ent = spring({ frame, fps, delay: 40 + i * 8, config: { damping: 14 } });
            return (
              <div
                key={i}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                  opacity: ent,
                  transform: `translateY(${(1 - ent) * 15}px)`,
                  background: "rgba(255,255,255,0.03)",
                  border: "1px solid rgba(255,255,255,0.08)",
                  borderRadius: 20,
                  padding: "6px 16px",
                }}
              >
                <span style={{ fontSize: 16 }}>{uc.icon}</span>
                <span
                  style={{
                    fontFamily: FONT_PRIMARY,
                    fontSize: 12,
                    color: "rgba(255,255,255,0.6)",
                    whiteSpace: "nowrap",
                  }}
                >
                  {uc.label}
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </AbsoluteFill>
  );
};
