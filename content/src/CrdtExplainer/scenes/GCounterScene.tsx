import React, { useMemo } from "react";
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import {
  AnimatedText,
  AnimatedCounter,
  Particles,
  Spawner,
  Behavior,
  StaggeredMotion,
  GradientTransition,
  useViewportRect,
} from "remotion-bits";
import { FONT_PRIMARY, FONT_DISPLAY } from "../fonts";

/**
 * Scene 5 — Implementing the G-Counter (500 frames / 25s at 20fps)
 *
 * Visualises a Grow-Only Counter with per-server vector slots.
 * Server A receives a click → increments its slot.
 * Merge = max per slot. Total = sum of all slots.
 *
 * Uses remotion-bits: AnimatedCounter, AnimatedText, StaggeredMotion,
 *   Particles, GradientTransition
 */

const BRAND = "#e06040";
const ACCENT_BLUE = "#4a9eff";
const ACCENT_GREEN = "#6eff9e";
const ACCENT_GOLD = "#ffc46a";
const ACCENT_PURPLE = "#c9a0ff";
const BG = "#1e1e1e";
const CARD_BG = "rgba(26, 28, 38, 0.88)";

/* ── Server colors ─────────────────────────────────── */
const SERVERS = [
  { id: "A", color: ACCENT_BLUE, localVal: 42, mergedVal: 42 },
  { id: "B", color: ACCENT_GREEN, localVal: 37, mergedVal: 42 },
  { id: "C", color: ACCENT_PURPLE, localVal: 28, mergedVal: 42 },
] as const;

/* ── Arrow SVG ─────────────────────────────────────── */
const ArrowDown: React.FC<{ size: number; color: string; opacity?: number }> = ({ size, color, opacity = 1 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth={2.5} opacity={opacity}>
    <path d="M12 5v14M5 12l7 7 7-7" />
  </svg>
);

export const GCounterScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const rect = useViewportRect();
  const { vmin } = rect;

  /* ── Fade in / out ─────────────────────────────────── */
  const fadeIn = interpolate(frame, [0, 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const fadeOut = interpolate(frame, [470, 500], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  /* ── Phase timings (frames) ────────────────────────── */
  // Phase 1: Show the vector concept (0–120)
  // Phase 2: Server A gets a click → increments slot (120–220)
  // Phase 3: Gossip / merge animation (220–350)
  // Phase 4: Sum = total count (350–500)

  const titleSpring = spring({ frame: frame - 5, fps, config: { damping: 14 } });
  const vectorAppear = spring({ frame: frame - 40, fps, config: { damping: 12, stiffness: 120 } });

  /* ── Server card springs ───────────────────────────── */
  const serverSprings = SERVERS.map((_, i) =>
    spring({ frame: frame - (55 + i * 18), fps, config: { damping: 13, stiffness: 110 } })
  );

  /* ── Click event on Server A ───────────────────────── */
  const clickPulse = spring({ frame: frame - 140, fps, config: { damping: 8, stiffness: 200 } });
  const clickFade = interpolate(frame, [140, 155], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  /* ── Merge phase ───────────────────────────────────── */
  const mergeProgress = interpolate(frame, [230, 340], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const mergeSpring = spring({ frame: frame - 230, fps, config: { damping: 16, stiffness: 80 } });

  /* ── Gossip arrows between servers ─────────────────── */
  const gossipOpacity = interpolate(frame, [230, 250, 320, 340], [0, 0.8, 0.8, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  /* ── Sum reveal ────────────────────────────────────── */
  const sumSpring = spring({ frame: frame - 360, fps, config: { damping: 12 } });

  /* ── Formula appear ────────────────────────────────── */
  const formulaSpring = spring({ frame: frame - 400, fps, config: { damping: 14 } });

  /* ── Slot values over time ─────────────────────────── */
  const slotAValue = frame < 140 ? 42 : 43;         // Click increments A's slot
  const slotBDisplay = frame < 230 ? 37 : 42;       // After merge, takes max
  const slotCDisplay = frame < 230 ? 28 : 42;       // After merge, takes max
  const totalBefore = 42 + 37 + 28; // 107
  const totalAfter = 42 + 43; // Actually would be 43 + 42 + 42 = 127 after merge. Let's simplify:
  // Before click: [42, 37, 28] → sum = 107
  // After click on A: [43, 37, 28] → sum = 108
  // After merge (max): [43, 42, 42] → sum = 127

  return (
    <AbsoluteFill style={{ backgroundColor: BG, opacity: fadeIn * fadeOut }}>
      {/* Background gradient */}
      <GradientTransition
        gradient={[
          "radial-gradient(ellipse at 20% 30%, rgba(74,158,255,0.1) 0%, transparent 50%)",
          "radial-gradient(ellipse at 80% 70%, rgba(110,255,158,0.08) 0%, transparent 50%)",
        ]}
        duration={500}
        easing="easeInOut"
      />

      {/* Ambient particles */}
      <Particles style={{ position: "absolute", inset: 0, opacity: 0.3 }}>
        <Spawner
          rate={0.3}
          max={25}
          lifespan={140}
          position={{ x: rect.width / 2, y: rect.height / 2 }}
          area={{ width: rect.width, height: rect.height }}
          velocity={{ x: 0, y: -0.2, varianceX: 0.2, varianceY: 0.1 }}
        >
          <div style={{ width: vmin * 0.5, height: vmin * 0.5, borderRadius: "50%", background: "rgba(255,255,255,0.3)" }} />
        </Spawner>
        <Behavior drag={0.98} opacity={[1, 0]} scale={{ start: 1, end: 0.2 }} />
      </Particles>

      {/* ── Title ────────────────────────────────────── */}
      <div style={{ position: "absolute", top: vmin * 3, left: 0, right: 0, textAlign: "center", zIndex: 10 }}>
        <AnimatedText
          style={{ fontFamily: FONT_DISPLAY, fontSize: vmin * 4.5, fontWeight: 700, color: "#fff" }}
          transition={{
            split: "word",
            splitStagger: 4,
            opacity: [0, 1],
            y: [15, 0],
            blur: [3, 0],
            duration: 22,
            delay: 5,
            easing: "easeOutCubic",
          }}
        >
          The G-Counter: Grow-Only Counter
        </AnimatedText>
      </div>

      {/* ── "Not a single integer" callout ───────────── */}
      <div
        style={{
          position: "absolute",
          top: vmin * 12,
          left: "50%",
          transform: `translateX(-50%) scale(${vectorAppear})`,
          opacity: vectorAppear,
          zIndex: 10,
        }}
      >
        <div
          style={{
            fontFamily: FONT_PRIMARY,
            fontSize: vmin * 2,
            color: "rgba(255,255,255,0.7)",
            background: "rgba(255,255,255,0.05)",
            border: "1px solid rgba(255,255,255,0.1)",
            borderRadius: vmin * 0.8,
            padding: `${vmin * 0.8}px ${vmin * 2}px`,
          }}
        >
          Not a single integer — a <span style={{ color: ACCENT_GOLD, fontWeight: 700 }}>vector</span>, one slot per server
        </div>
      </div>

      {/* ── Server vector cards ──────────────────────── */}
      <div
        style={{
          position: "absolute",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -45%)",
          display: "flex",
          gap: vmin * 4,
          zIndex: 5,
        }}
      >
        {SERVERS.map((srv, i) => {
          const s = serverSprings[i];
          const isA = srv.id === "A";
          const isPulsing = isA && frame >= 140 && frame < 180;
          const currentVal = isA
            ? (frame < 140 ? srv.localVal : slotAValue)
            : (frame < 230 ? srv.localVal : slotBDisplay);
          const displayVal = srv.id === "C" ? (frame < 230 ? srv.localVal : slotCDisplay) : currentVal;

          return (
            <div
              key={srv.id}
              style={{
                transform: `scale(${s}) ${isPulsing ? `scale(${1 + clickPulse * 0.08})` : ""}`,
                opacity: s,
              }}
            >
              {/* Server card */}
              <div
                style={{
                  width: vmin * 22,
                  background: CARD_BG,
                  borderRadius: vmin * 1.5,
                  border: `2px solid ${isPulsing ? srv.color : "rgba(255,255,255,0.1)"}`,
                  boxShadow: isPulsing
                    ? `0 0 ${vmin * 3}px ${srv.color}40`
                    : `0 ${vmin * 0.5}px ${vmin * 2}px rgba(0,0,0,0.3)`,
                  overflow: "hidden",
                  transition: "border-color 0.3s, box-shadow 0.3s",
                }}
              >
                {/* Server header */}
                <div
                  style={{
                    background: `linear-gradient(135deg, ${srv.color}20, transparent)`,
                    padding: `${vmin * 1.2}px ${vmin * 2}px`,
                    borderBottom: `1px solid rgba(255,255,255,0.08)`,
                    display: "flex",
                    alignItems: "center",
                    gap: vmin * 1,
                  }}
                >
                  <div
                    style={{
                      width: vmin * 1.5,
                      height: vmin * 1.5,
                      borderRadius: "50%",
                      background: srv.color,
                      boxShadow: `0 0 ${vmin * 0.8}px ${srv.color}`,
                    }}
                  />
                  <span style={{ fontFamily: FONT_DISPLAY, fontSize: vmin * 2.2, fontWeight: 700, color: "#fff" }}>
                    Server {srv.id}
                  </span>
                </div>

                {/* Vector slots */}
                <div style={{ padding: `${vmin * 1.5}px ${vmin * 2}px` }}>
                  <div style={{ fontFamily: FONT_PRIMARY, fontSize: vmin * 1.3, color: "rgba(255,255,255,0.5)", marginBottom: vmin * 0.8 }}>
                    Vector Slot
                  </div>
                  <div
                    style={{
                      fontFamily: FONT_PRIMARY,
                      fontSize: vmin * 5,
                      fontWeight: 700,
                      color: srv.color,
                      textAlign: "center",
                      lineHeight: 1.2,
                    }}
                  >
                    {displayVal}
                  </div>

                  {/* Click indicator on Server A */}
                  {isA && frame >= 130 && (
                    <div
                      style={{
                        marginTop: vmin * 0.8,
                        fontFamily: FONT_PRIMARY,
                        fontSize: vmin * 1.4,
                        color: ACCENT_GREEN,
                        textAlign: "center",
                        opacity: clickFade,
                      }}
                    >
                      +1 click! → {slotAValue}
                    </div>
                  )}
                </div>
              </div>

              {/* "max()" label during merge */}
              {frame >= 230 && frame < 360 && srv.id !== "A" && (
                <div
                  style={{
                    textAlign: "center",
                    marginTop: vmin * 1,
                    fontFamily: FONT_PRIMARY,
                    fontSize: vmin * 1.4,
                    color: ACCENT_GOLD,
                    opacity: gossipOpacity,
                  }}
                >
                  max({srv.localVal}, {slotAValue}) = {slotAValue > srv.localVal ? slotAValue : srv.localVal}
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* ── Gossip arrows between servers ────────────── */}
      {frame >= 230 && frame < 350 && (
        <div
          style={{
            position: "absolute",
            top: "42%",
            left: "50%",
            transform: "translate(-50%, 0)",
            display: "flex",
            gap: vmin * 16,
            opacity: gossipOpacity,
            zIndex: 6,
          }}
        >
          {/* Arrow A → B */}
          <div style={{ display: "flex", flexDirection: "column", alignItems: "center" }}>
            <svg width={vmin * 8} height={vmin * 2} viewBox="0 0 80 20">
              <path d="M5 10 H65" stroke={ACCENT_GOLD} strokeWidth={2} strokeDasharray="6 4">
                <animate attributeName="stroke-dashoffset" from="0" to="-20" dur="0.8s" repeatCount="indefinite" />
              </path>
              <polygon points="65,5 75,10 65,15" fill={ACCENT_GOLD} />
            </svg>
            <span style={{ fontFamily: FONT_PRIMARY, fontSize: vmin * 1.1, color: ACCENT_GOLD, marginTop: vmin * 0.3 }}>
              gossip
            </span>
          </div>
        </div>
      )}

      {/* ── Sum total ────────────────────────────────── */}
      <div
        style={{
          position: "absolute",
          bottom: vmin * 10,
          left: "50%",
          transform: `translateX(-50%) translateY(${(1 - sumSpring) * 25}px)`,
          opacity: sumSpring,
          zIndex: 10,
        }}
      >
        <div
          style={{
            background: "rgba(255,196,106,0.08)",
            border: `2px solid ${ACCENT_GOLD}`,
            borderRadius: vmin * 1.2,
            padding: `${vmin * 1.5}px ${vmin * 4}px`,
            display: "flex",
            alignItems: "center",
            gap: vmin * 2,
          }}
        >
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: vmin * 2.2, color: "rgba(255,255,255,0.8)" }}>
            Total Count =
          </span>
          <AnimatedCounter
            transition={{
              values: [0, 127],
              duration: 40,
              delay: 365,
              color: [ACCENT_GOLD, ACCENT_GOLD],
            }}
            toFixed={0}
            style={{
              fontFamily: FONT_DISPLAY,
              fontSize: vmin * 4,
              fontWeight: 700,
              color: ACCENT_GOLD,
            }}
          />
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: vmin * 1.6, color: "rgba(255,255,255,0.5)" }}>
            (sum of all slots)
          </span>
        </div>
      </div>

      {/* ── Formula ──────────────────────────────────── */}
      <div
        style={{
          position: "absolute",
          bottom: vmin * 3,
          left: "50%",
          transform: `translateX(-50%) scale(${formulaSpring})`,
          opacity: formulaSpring,
          zIndex: 10,
        }}
      >
        <div
          style={{
            fontFamily: FONT_PRIMARY,
            fontSize: vmin * 1.8,
            color: "rgba(255,255,255,0.6)",
            background: "rgba(255,255,255,0.04)",
            border: "1px solid rgba(255,255,255,0.08)",
            borderRadius: vmin * 0.6,
            padding: `${vmin * 0.6}px ${vmin * 2}px`,
          }}
        >
          merge(A, B) = [max(A₁, B₁), max(A₂, B₂), …, max(Aₙ, Bₙ)]
        </div>
      </div>
    </AbsoluteFill>
  );
};
