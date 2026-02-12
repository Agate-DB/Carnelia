import React, { useMemo } from "react";
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import {
  AnimatedText,
  Particles,
  Spawner,
  Behavior,
  GradientTransition,
  useViewportRect,
} from "remotion-bits";
import { FONT_PRIMARY, FONT_DISPLAY } from "../fonts";

/**
 * Scene 2 — The CRDT Breakthrough (500 frames / 25s at 20fps)
 *
 * Phase A (0–160):  "Users don't need the perfect global total" — immediate feedback
 * Phase B (160–320): CRDT intro — nodes accept locally + gossip in background
 * Phase C (320–500): Convergence demo — messages reorder/duplicate, still converge
 *
 * Uses remotion-bits: AnimatedText, Particles, GradientTransition
 * AUDIO CUE: solution_intro_narration.mp3
 */

const ACCENT_BLUE = "#4a9eff";
const ACCENT_GREEN = "#6eff9e";
const ACCENT_PURPLE = "#c9a0ff";
const ACCENT_TEAL = "#6affea";
const BG = "#1e1e1e";

/* ── Node component — a server/replica circle ──────── */
const ReplicaNode: React.FC<{
  x: number;
  y: number;
  color: string;
  label: string;
  entrance: number;
  localCount?: number;
  showCount?: boolean;
  pulseGlow?: boolean;
  frame: number;
}> = ({ x, y, color, label, entrance, localCount, showCount, pulseGlow, frame }) => {
  const glow = pulseGlow ? Math.sin(frame * 0.06) * 0.3 + 0.7 : 0;
  return (
    <div style={{
      position: "absolute",
      left: `${x}%`,
      top: `${y}%`,
      transform: `translate(-50%, -50%) scale(${entrance})`,
      textAlign: "center",
    }}>
      {/* Glow ring */}
      {pulseGlow && (
        <div style={{
          position: "absolute",
          left: "50%",
          top: "50%",
          transform: "translate(-50%, -50%)",
          width: 72,
          height: 72,
          borderRadius: "50%",
          border: `2px solid ${color}`,
          opacity: glow * 0.4,
          boxShadow: `0 0 20px ${color}`,
        }} />
      )}
      {/* Node circle */}
      <div style={{
        width: 52,
        height: 52,
        borderRadius: "50%",
        background: `radial-gradient(circle at 35% 35%, ${color}, ${color}88)`,
        border: `2px solid ${color}`,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        boxShadow: `0 0 16px ${color}44`,
      }}>
        <span style={{ fontFamily: FONT_DISPLAY, fontSize: 14, fontWeight: 700, color: "#fff" }}>
          {label}
        </span>
      </div>
      {/* Local counter */}
      {showCount && localCount !== undefined && (
        <div style={{
          fontFamily: FONT_PRIMARY,
          fontSize: 13,
          color,
          marginTop: 6,
          fontVariantNumeric: "tabular-nums",
        }}>
          {localCount.toLocaleString()}
        </div>
      )}
    </div>
  );
};

/* ── Gossip arrow — animated dashed line btw nodes ─── */
const GossipArrow: React.FC<{
  fromX: number;
  fromY: number;
  toX: number;
  toY: number;
  progress: number;
  color: string;
  frame: number;
}> = ({ fromX, fromY, toX, toY, progress, color, frame }) => {
  if (progress < 0.01) return null;
  const dashOffset = -frame * 0.8;
  return (
    <line
      x1={`${fromX}%`} y1={`${fromY}%`}
      x2={`${fromX + (toX - fromX) * progress}%`} y2={`${fromY + (toY - fromY) * progress}%`}
      stroke={color}
      strokeWidth={2}
      strokeDasharray="8 5"
      strokeDashoffset={dashOffset}
      opacity={progress * 0.6}
      strokeLinecap="round"
    />
  );
};

export const CrdtBreakthroughScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const rect = useViewportRect();
  const { vmin } = rect;

  /* ── Fade in / out ─────────────────────────────────── */
  const fadeIn = interpolate(frame, [0, 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const fadeOut = interpolate(frame, [470, 500], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  /* ── Phase A: Immediate feedback concept (0–160) ──── */
  const feedbackAppear = interpolate(frame, [20, 50], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const checkScale = spring({ frame: frame - 80, fps, config: { damping: 10, stiffness: 200 } });
  const vsOld = interpolate(frame, [100, 130], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  /* ── Phase B: CRDT nodes appear (160–320) ──────────── */
  const nodes = useMemo(() => [
    { x: 30, y: 42, color: ACCENT_BLUE, label: "A", delay: 170 },
    { x: 55, y: 30, color: ACCENT_GREEN, label: "B", delay: 185 },
    { x: 70, y: 50, color: ACCENT_PURPLE, label: "C", delay: 200 },
    { x: 45, y: 62, color: ACCENT_TEAL, label: "D", delay: 215 },
  ], []);

  const crdtTitleAppear = interpolate(frame, [155, 180], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  /* Gossip arrows appear in phase B */
  const gossipPairs = useMemo(() => [
    { from: 0, to: 1, delay: 235, color: ACCENT_BLUE },
    { from: 1, to: 2, delay: 250, color: ACCENT_GREEN },
    { from: 2, to: 3, delay: 265, color: ACCENT_PURPLE },
    { from: 3, to: 0, delay: 280, color: ACCENT_TEAL },
    { from: 0, to: 2, delay: 295, color: ACCENT_BLUE },
    { from: 1, to: 3, delay: 305, color: ACCENT_GREEN },
  ], []);

  /* ── Phase C: Convergence (320–500) ────────────────── */
  const convergePulse = frame >= 370 ? spring({ frame: frame - 370, fps, config: { damping: 6, stiffness: 180 } }) : 0;
  const allSame = frame >= 390;

  // Count evolution per node
  const getNodeCount = (nodeIdx: number) => {
    if (frame < 230) return 0;
    const base = [196748, 196749, 196748, 196747][nodeIdx];
    const drift = frame < 350 ? Math.floor(Math.sin(frame * 0.02 + nodeIdx * 1.5) * 3) : 0;
    if (allSame) return 196750;
    return base + drift;
  };

  // "No blocking" badge
  const noBadge = interpolate(frame, [230, 260], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Result text
  const resultAppear = interpolate(frame, [400, 430], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ backgroundColor: BG, opacity: fadeIn * fadeOut }}>
      {/* Gradient background */}
      <GradientTransition
        gradient={[
          "radial-gradient(ellipse at 60% 40%, rgba(110,255,158,0.12) 0%, transparent 60%)",
          "radial-gradient(ellipse at 40% 60%, rgba(74,158,255,0.1) 0%, transparent 60%)",
        ]}
        duration={500}
        easing="easeInOut"
      />

      {/* Ambient particles */}
      <Particles style={{ position: "absolute", inset: 0, opacity: 0.3 }}>
        <Spawner
          rate={0.3}
          max={30}
          lifespan={150}
          position={{ x: rect.width / 2, y: rect.height / 2 }}
          area={{ width: rect.width, height: rect.height }}
          velocity={{ x: 0, y: -0.2, varianceX: 0.2, varianceY: 0.1 }}
        >
          <div style={{ width: vmin * 0.5, height: vmin * 0.5, borderRadius: "50%", background: "rgba(255,255,255,0.3)" }} />
          <div style={{ width: vmin * 0.35, height: vmin * 0.35, borderRadius: "50%", background: ACCENT_GREEN, opacity: 0.35 }} />
        </Spawner>
        <Behavior drag={0.98} opacity={[1, 0]} scale={{ start: 1, end: 0.3 }} />
      </Particles>

      {/* ── Scene title ──────────────────────────────── */}
      <div style={{ position: "absolute", top: vmin * 3, left: 0, right: 0, textAlign: "center", zIndex: 10 }}>
        <AnimatedText
          style={{ fontFamily: FONT_DISPLAY, fontSize: vmin * 4.5, fontWeight: 700, color: "#fff" }}
          transition={{
            split: "word",
            splitStagger: 4,
            opacity: [0, 1],
            y: [20, 0],
            blur: [4, 0],
            duration: 25,
            delay: 5,
            easing: "easeOutCubic",
          }}
        >
          The CRDT Breakthrough
        </AnimatedText>
      </div>

      {/* ── Phase A: Immediate feedback concept ──────── */}
      {frame < 200 && (
        <div style={{
          position: "absolute",
          left: "50%",
          top: "40%",
          transform: "translate(-50%, -50%)",
          opacity: feedbackAppear,
          textAlign: "center",
          zIndex: 6,
        }}>
          {/* User click mockup */}
          <div style={{
            background: "rgba(30,30,40,0.85)",
            border: "1px solid rgba(255,255,255,0.12)",
            borderRadius: vmin * 2,
            padding: `${vmin * 3}px ${vmin * 6}px`,
            display: "inline-block",
          }}>
            <div style={{
              fontFamily: FONT_PRIMARY,
              fontSize: vmin * 2,
              color: "rgba(255,255,255,0.5)",
              marginBottom: vmin * 1.5,
            }}>
              User taps "like"
            </div>
            <div style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              gap: vmin * 2,
            }}>
              <div style={{
                fontFamily: FONT_DISPLAY,
                fontSize: vmin * 4,
                color: ACCENT_GREEN,
                transform: `scale(${checkScale})`,
              }}>
                ✓
              </div>
              <div style={{
                fontFamily: FONT_PRIMARY,
                fontSize: vmin * 2.2,
                color: ACCENT_GREEN,
              }}>
                Instantly registered
              </div>
            </div>
          </div>

          {/* vs comparison */}
          {frame >= 100 && (
            <div style={{
              marginTop: vmin * 3,
              display: "flex",
              gap: vmin * 4,
              justifyContent: "center",
              opacity: vsOld,
            }}>
              <div style={{
                padding: `${vmin * 1.5}px ${vmin * 2.5}px`,
                borderRadius: vmin * 0.8,
                border: "1px solid rgba(255,68,68,0.3)",
                background: "rgba(255,68,68,0.06)",
              }}>
                <div style={{ fontFamily: FONT_PRIMARY, fontSize: vmin * 1.5, color: "#ff4444", textDecoration: "line-through", opacity: 0.6 }}>
                  Wait for global consensus
                </div>
              </div>
              <div style={{
                fontFamily: FONT_PRIMARY,
                fontSize: vmin * 2,
                color: "rgba(255,255,255,0.3)",
                alignSelf: "center",
              }}>→</div>
              <div style={{
                padding: `${vmin * 1.5}px ${vmin * 2.5}px`,
                borderRadius: vmin * 0.8,
                border: `1px solid ${ACCENT_GREEN}44`,
                background: `${ACCENT_GREEN}0a`,
              }}>
                <div style={{ fontFamily: FONT_PRIMARY, fontSize: vmin * 1.5, color: ACCENT_GREEN }}>
                  Accept locally, converge later
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* ── Phase B: CRDT — nodes + gossip ────────────── */}
      {frame >= 155 && (
        <>
          {/* CRDT title badge */}
          <div style={{
            position: "absolute",
            top: vmin * 14,
            left: "50%",
            transform: "translateX(-50%)",
            opacity: crdtTitleAppear,
            zIndex: 8,
            textAlign: "center",
          }}>
            <div style={{
              background: `${ACCENT_GREEN}12`,
              border: `1px solid ${ACCENT_GREEN}44`,
              borderRadius: vmin * 1,
              padding: `${vmin * 1}px ${vmin * 3}px`,
              display: "inline-block",
            }}>
              <span style={{ fontFamily: FONT_DISPLAY, fontSize: vmin * 2.5, color: ACCENT_GREEN }}>
                CRDT
              </span>
              <span style={{ fontFamily: FONT_PRIMARY, fontSize: vmin * 1.6, color: "rgba(255,255,255,0.5)", marginLeft: vmin * 1 }}>
                Conflict-free Replicated Data Type
              </span>
            </div>
          </div>

          {/* Replica nodes */}
          {nodes.map((node, i) => {
            const ent = spring({ frame: frame - node.delay, fps, config: { damping: 12, stiffness: 140 } });
            return (
              <ReplicaNode
                key={i}
                x={node.x}
                y={node.y}
                color={node.color}
                label={node.label}
                entrance={ent}
                localCount={getNodeCount(i)}
                showCount={frame >= 230}
                pulseGlow={allSame}
                frame={frame}
              />
            );
          })}

          {/* Gossip arrows (SVG overlay) */}
          <svg style={{ position: "absolute", inset: 0, width: "100%", height: "100%", zIndex: 2, pointerEvents: "none" }}>
            {gossipPairs.map((gp, i) => {
              const prog = spring({ frame: frame - gp.delay, fps, config: { damping: 18, stiffness: 100 } });
              return (
                <GossipArrow
                  key={i}
                  fromX={nodes[gp.from].x}
                  fromY={nodes[gp.from].y}
                  toX={nodes[gp.to].x}
                  toY={nodes[gp.to].y}
                  progress={prog}
                  color={gp.color}
                  frame={frame}
                />
              );
            })}
          </svg>

          {/* "No blocking" badge */}
          {frame >= 230 && (
            <div style={{
              position: "absolute",
              right: vmin * 5,
              top: vmin * 18,
              opacity: noBadge,
              zIndex: 8,
            }}>
              <div style={{
                background: "rgba(30,30,40,0.85)",
                border: "1px solid rgba(255,255,255,0.1)",
                borderRadius: vmin * 1,
                padding: `${vmin * 1.5}px ${vmin * 2}px`,
                maxWidth: vmin * 28,
              }}>
                <div style={{ fontFamily: FONT_DISPLAY, fontSize: vmin * 1.8, color: ACCENT_GREEN, marginBottom: vmin * 0.5 }}>
                  ⚡ No Blocking
                </div>
                <div style={{ fontFamily: FONT_PRIMARY, fontSize: vmin * 1.3, color: "rgba(255,255,255,0.55)", lineHeight: 1.6 }}>
                  Every node accepts updates locally.<br />
                  Gossip happens in the background.
                </div>
              </div>
            </div>
          )}

          {/* Message resilience badges (Phase C) */}
          {frame >= 340 && (
            <div style={{
              position: "absolute",
              left: vmin * 4,
              bottom: vmin * 18,
              zIndex: 8,
            }}>
              {[
                { icon: "🔀", label: "Reordered", delay: 340 },
                { icon: "📋", label: "Duplicated", delay: 355 },
                { icon: "⏳", label: "Delayed", delay: 370 },
              ].map((item, i) => {
                const badgeOpacity = interpolate(frame, [item.delay, item.delay + 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
                return (
                  <div key={i} style={{
                    display: "flex",
                    alignItems: "center",
                    gap: vmin * 1,
                    marginBottom: vmin * 1,
                    opacity: badgeOpacity,
                  }}>
                    <span style={{ fontSize: vmin * 2 }}>{item.icon}</span>
                    <span style={{
                      fontFamily: FONT_PRIMARY,
                      fontSize: vmin * 1.4,
                      color: "rgba(255,255,255,0.5)",
                      padding: `${vmin * 0.4}px ${vmin * 1}px`,
                      border: "1px solid rgba(255,255,255,0.1)",
                      borderRadius: vmin * 0.5,
                      background: "rgba(255,255,255,0.03)",
                    }}>
                      {item.label}
                    </span>
                    <span style={{ fontFamily: FONT_PRIMARY, fontSize: vmin * 1.4, color: ACCENT_GREEN }}>
                      → still converges ✓
                    </span>
                  </div>
                );
              })}
            </div>
          )}
        </>
      )}

      {/* ── Convergence flash ────────────────────────── */}
      {convergePulse > 0 && (
        <div style={{
          position: "absolute",
          left: "48%",
          top: "46%",
          transform: "translate(-50%, -50%)",
          width: vmin * convergePulse * 35,
          height: vmin * convergePulse * 35,
          borderRadius: "50%",
          border: `2px solid ${ACCENT_GREEN}`,
          opacity: Math.max(0, 0.5 - convergePulse * 0.5),
          zIndex: 1,
        }} />
      )}

      {/* ── Result text (bottom) ─────────────────────── */}
      {frame >= 400 && (
        <div style={{
          position: "absolute",
          bottom: vmin * 5,
          left: "50%",
          transform: `translateX(-50%) translateY(${(1 - resultAppear) * 20}px)`,
          opacity: resultAppear,
          zIndex: 10,
          textAlign: "center",
        }}>
          <div style={{
            background: `${ACCENT_GREEN}10`,
            border: `1px solid ${ACCENT_GREEN}44`,
            borderRadius: vmin * 1,
            padding: `${vmin * 1.5}px ${vmin * 4}px`,
          }}>
            <div style={{ fontFamily: FONT_DISPLAY, fontSize: vmin * 2.5, color: ACCENT_GREEN, marginBottom: vmin * 0.5 }}>
              ✓ Everyone converges on the same truth
            </div>
            <div style={{ fontFamily: FONT_PRIMARY, fontSize: vmin * 1.5, color: "rgba(255,255,255,0.5)" }}>
              No blocking · No waiting · Mathematically guaranteed
            </div>
          </div>
        </div>
      )}
    </AbsoluteFill>
  );
};
