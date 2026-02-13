import React, { useMemo } from "react";
import { AbsoluteFill, Img, interpolate, spring, staticFile, useCurrentFrame, useVideoConfig } from "remotion";
import {
  AnimatedText,
  AnimatedCounter,
  Particles,
  Spawner,
  Behavior,
  GradientTransition,
  useViewportRect,
} from "remotion-bits";
import { FONT_PRIMARY, FONT_DISPLAY } from "../fonts";

/**
 * Scene 1 — The Coordination Bottleneck (700 frames / 35s at 20fps)
 *
 * Phase A (0–200): BirdWatch intro — "Watcher 302" posts falcon photo, goes viral.
 * Phase B (200–400): Scale-out — server cluster grows, click count splits.
 * Phase C (400–560): Coordination trap — servers calling each other, latency spikes.
 * Phase D (560–700): Loading screen — user staring, waiting for global consensus.
 *
 * Uses remotion-bits: AnimatedCounter, AnimatedText, Particles, GradientTransition
 * AUDIO CUE: problem_narration.mp3
 */

const BRAND = "#e06040";
const ACCENT_BLUE = "#4a9eff";
const ACCENT_GREEN = "#6eff9e";
const BG = "#1e1e1e";
const CARD_BG = "rgba(30, 30, 40, 0.85)";
const SERVER_COLOR = "#c9a0ff";
const DANGER = "#ff4444";

/* ── Small bird icon (SVG) ─────────────────────────── */
const BirdIcon: React.FC<{ size: number; color?: string }> = ({ size, color = BRAND }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill={color}>
    <path d="M23 2a8.4 8.4 0 0 1-2.36.33A4.13 4.13 0 0 0 22.46.64a8.23 8.23 0 0 1-2.61 1A4.1 4.1 0 0 0 12 5.07a11.63 11.63 0 0 1-8.46-4.3 4.1 4.1 0 0 0 1.27 5.48A4.07 4.07 0 0 1 3 5.8v.05a4.1 4.1 0 0 0 3.29 4.02 4.09 4.09 0 0 1-1.85.07A4.11 4.11 0 0 0 8.28 13 8.23 8.23 0 0 1 1 14.61 11.6 11.6 0 0 0 7.29 16.5c7.55 0 11.67-6.25 11.67-11.67 0-.18 0-.35-.01-.53A8.35 8.35 0 0 0 23 2z" />
  </svg>
);

/* ── Server icon (SVG) ─────────────────────────────── */
const ServerIcon: React.FC<{ size: number; color?: string; pulse?: boolean }> = ({ size, color = SERVER_COLOR, pulse }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth={1.8} opacity={pulse ? 0.8 : 1}>
    <rect x="2" y="2" width="20" height="8" rx="2" ry="2" />
    <rect x="2" y="14" width="20" height="8" rx="2" ry="2" />
    <line x1="6" y1="6" x2="6.01" y2="6" />
    <line x1="6" y1="18" x2="6.01" y2="18" />
  </svg>
);

/* ── Spinner icon (loading) ────────────────────────── */
const Spinner: React.FC<{ size: number; color?: string; angle: number }> = ({ size, color = DANGER, angle }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" style={{ transform: `rotate(${angle}deg)` }}>
    <circle cx="12" cy="12" r="10" stroke="rgba(255,255,255,0.1)" strokeWidth="3" />
    <path d="M22 12A10 10 0 0 0 12 2" stroke={color} strokeWidth="3" strokeLinecap="round" />
  </svg>
);

export const BirdWatchScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const rect = useViewportRect();
  const { vmin } = rect;

  /* ── Global fade in / out ──────────────────────────── */
  const fadeIn = interpolate(frame, [0, 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const fadeOut = interpolate(frame, [670, 700], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  /* ── Phase A: BirdWatch card (0–200) ───────────────── */
  const cardSpring = spring({ frame: frame - 15, fps, config: { damping: 14, stiffness: 120 } });
  const userAppear = spring({ frame: frame - 30, fps, config: { damping: 12 } });
  const counterSpring = spring({ frame: frame - 60, fps, config: { damping: 16 } });
  const shimmerAngle = interpolate(frame, [0, 700], [0, 360]);

  // Card shrinks/moves left during phase B
  const cardScale = interpolate(frame, [200, 280], [1, 0.45], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const cardX = interpolate(frame, [200, 280], [0, -34], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const cardY = interpolate(frame, [200, 280], [0, -20], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  /* ── Phase B: Server cluster (200–400) ─────────────── */
  const serversAppear = interpolate(frame, [220, 300], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const serverNodes = useMemo(() => [
    { x: 0, y: -16, delay: 230, label: "US-East" },
    { x: 22, y: -6, delay: 245, label: "US-West" },
    { x: 14, y: 12, delay: 260, label: "EU-Central" },
    { x: -14, y: 10, delay: 275, label: "Asia-Pacific" },
    { x: -20, y: -4, delay: 290, label: "SA-South" },
  ], []);

  /* ── Phase C: Coordination lines (400–560) ─────────── */
  const coordAppear = interpolate(frame, [400, 440], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const latencyPulse = Math.sin(frame * 0.06) * 0.3 + 0.7;

  /* ── Phase D: Loading screen overlay (560–700) ─────── */
  const loadingAppear = spring({ frame: frame - 560, fps, config: { damping: 14, stiffness: 80 } });
  const spinnerAngle = interpolate(frame, [560, 700], [0, 720]);

  /* ── Notification dots that fly in (Phase A) ───────── */
  const notifs = useMemo(() => {
    const items: { x: number; y: number; delay: number; color: string }[] = [];
    for (let i = 0; i < 8; i++) {
      items.push({
        x: Math.cos(i * 0.8) * vmin * 18 + (Math.sin(i * 2.3) * vmin * 6),
        y: Math.sin(i * 1.2) * vmin * 12 + (Math.cos(i * 1.7) * vmin * 4),
        delay: 80 + i * 12,
        color: [BRAND, ACCENT_BLUE, "#ff6a9e", ACCENT_GREEN][i % 4],
      });
    }
    return items;
  }, [vmin]);

  return (
    <AbsoluteFill style={{ backgroundColor: BG, opacity: fadeIn * fadeOut }}>
      {/* Gradient background */}
      <GradientTransition
        gradient={[
          "radial-gradient(ellipse at 30% 40%, rgba(224,96,64,0.15) 0%, transparent 60%)",
          "radial-gradient(ellipse at 70% 60%, rgba(74,158,255,0.12) 0%, transparent 60%)",
        ]}
        duration={700}
        easing="easeInOut"
      />

      {/* Ambient particles */}
      <Particles style={{ position: "absolute", inset: 0, opacity: 0.35 }}>
        <Spawner
          rate={0.4}
          max={35}
          lifespan={140}
          position={{ x: rect.width / 2, y: rect.height / 2 }}
          area={{ width: rect.width, height: rect.height }}
          velocity={{ x: 0, y: -0.25, varianceX: 0.25, varianceY: 0.12 }}
        >
          <div style={{ width: vmin * 0.5, height: vmin * 0.5, borderRadius: "50%", background: "rgba(255,255,255,0.35)" }} />
          <div style={{ width: vmin * 0.35, height: vmin * 0.35, borderRadius: "50%", background: BRAND, opacity: 0.4 }} />
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
          The Coordination Bottleneck
        </AnimatedText>
      </div>

      {/* ── Social media card mockup ─────────────────── */}
      <div
        style={{
          position: "absolute",
          left: `calc(50% + ${cardX}%)`,
          top: `calc(50% + ${cardY}%)`,
          transform: `translate(-50%, -50%) scale(${cardSpring * cardScale})`,
          width: vmin * 55,
          borderRadius: vmin * 2,
          background: CARD_BG,
          backdropFilter: "blur(12px)",
          border: "1px solid rgba(255,255,255,0.12)",
          boxShadow: `0 0 ${vmin * 4}px rgba(224,96,64,0.15), 0 ${vmin * 2}px ${vmin * 6}px rgba(0,0,0,0.4)`,
          overflow: "hidden",
          zIndex: 5,
        }}
      >
        {/* User header */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: vmin * 1.5,
            padding: `${vmin * 2}px ${vmin * 2.5}px`,
            borderBottom: "1px solid rgba(255,255,255,0.08)",
            opacity: userAppear,
          }}
        >
          <div
            style={{
              width: vmin * 5,
              height: vmin * 5,
              borderRadius: "50%",
              background: `linear-gradient(135deg, ${BRAND}, ${ACCENT_BLUE})`,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <img
              src="https://randomuser.me/api/portraits/men/32.jpg"
              alt="Watcher 302 avatar"
              style={{
                width: "100%",
                height: "100%",
                borderRadius: "50%",
                objectFit: "cover",
                border: `${vmin * 0.18}px solid rgba(255,255,255,0.7)`,
                boxShadow: `0 0 ${vmin * 0.7}px ${BRAND}`,
              }}
            />
          </div>
          <div>
            <div style={{ fontFamily: FONT_DISPLAY, fontSize: vmin * 2, fontWeight: 700, color: "#fff" }}>
              Watcher 302
            </div>
            <div style={{ fontFamily: FONT_PRIMARY, fontSize: vmin * 1.2, color: "rgba(255,255,255,0.5)" }}>
              @watcher302 · 2h
            </div>
          </div>
        </div>

        {/* "Photo" placeholder — gradient with shimmer */}
        <div
          style={{
            width: "100%",
            height: vmin * 28,
            position: "relative",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            overflow: "hidden",
            borderRadius: vmin * 1.2,
            boxShadow: `0 0 ${vmin * 2}px rgba(0,0,0,0.25)`
          }}
        >
          <Img
            src={staticFile("falcon.jpg")}
            alt="Falcon"
            style={{
              width: "100%",
              height: "100%",
              objectFit: "cover",
              borderRadius: vmin * 1.2,
              boxShadow: `0 0 ${vmin * 2}px rgba(0,0,0,0.25)`
            }} 
          />
          {/* Optional shimmer overlay for effect */}
          <div
            style={{
              position: "absolute",
              inset: 0,
              pointerEvents: "none",
              background: `linear-gradient(${shimmerAngle}deg, #2a3a4a88, #1e2e3e44, #3a2a3a33, #2a3a4a88)`,
              backgroundSize: "200% 200%",
              mixBlendMode: "soft-light",
              borderRadius: vmin * 1.2,
            }}
          />
          <div
            style={{
              position: "absolute",
              bottom: vmin * 1.5,
              right: vmin * 2,
              fontFamily: FONT_PRIMARY,
              fontSize: vmin * 1.3,
              color: "rgba(255,255,255,0.6)",
              background: "rgba(0,0,0,0.5)",
              padding: `${vmin * 0.4}px ${vmin * 1}px`,
              borderRadius: vmin * 0.5,
            }}
          >
            Photo by Vincent van Zalinge on Unsplash
          </div>
        </div>

        {/* Engagement row */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: vmin * 4,
            padding: `${vmin * 2}px ${vmin * 2.5}px`,
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: vmin * 1, opacity: counterSpring }}>
            <BirdIcon size={vmin * 3} color={BRAND} />
            <AnimatedCounter
              transition={{
                values: [0, 983742],
                duration: 130,
                delay: 70,
                color: [BRAND, BRAND],
              }}
              toFixed={0}
              style={{
                fontFamily: FONT_PRIMARY,
                fontSize: vmin * 2.5,
                fontWeight: 700,
                color: BRAND,
              }}
            />
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: vmin * 0.8, opacity: counterSpring }}>
            <svg width={vmin * 2.2} height={vmin * 2.2} viewBox="0 0 24 24" fill="none" stroke={ACCENT_GREEN} strokeWidth={2}>
              <path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8M16 6l-4-4-4 4M12 2v13" />
            </svg>
            <span style={{ fontFamily: FONT_PRIMARY, fontSize: vmin * 1.8, color: ACCENT_GREEN, fontWeight: 600 }}>
              12.4k
            </span>
          </div>
        </div>
      </div>

      {/* ── Notification pulse dots (Phase A) ─────────── */}
      {frame < 250 && notifs.map((n, i) => {
        const s = spring({ frame: frame - n.delay, fps, config: { damping: 10, stiffness: 180 } });
        const fade = interpolate(frame, [n.delay, n.delay + 15, n.delay + 50, n.delay + 70], [0, 0.9, 0.9, 0], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
        });
        return (
          <div
            key={i}
            style={{
              position: "absolute",
              left: `calc(50% + ${n.x}px)`,
              top: `calc(50% + ${n.y}px)`,
              width: vmin * 1.2,
              height: vmin * 1.2,
              borderRadius: "50%",
              background: n.color,
              opacity: fade,
              transform: `scale(${s})`,
              boxShadow: `0 0 ${vmin * 1.5}px ${n.color}`,
              zIndex: 4,
            }}
          />
        );
      })}

      {/* ── Phase B: Server cluster ──────────────────── */}
      {frame >= 210 && (
        <div style={{
          position: "absolute",
          left: "50%",
          top: "38%",
          transform: "translate(-50%, -50%)",
          width: vmin * 56,
          height: vmin * 44,
          opacity: serversAppear,
          zIndex: 6,
        }}>
          

          {/* Server nodes */}
          {serverNodes.map((srv, i) => {
            const ent = spring({ frame: frame - srv.delay, fps, config: { damping: 12, stiffness: 140 } });
            const isActive = frame >= 400;
            const activePulse = isActive ? Math.sin(frame * 0.08 + i * 1.5) * 0.2 + 0.8 : 1;
            return (
              <div
                key={i}
                style={{
                  position: "absolute",
                  left: `calc(50% + ${vmin * srv.x}px)`,
                  top: `calc(50% + ${vmin * srv.y}px)`,
                  transform: `translate(-50%, -50%) scale(${ent})`,
                  textAlign: "center",
                  opacity: activePulse,
                }}
              >
                <ServerIcon size={vmin * 5} color={isActive ? DANGER : SERVER_COLOR} pulse={isActive} />
                <div style={{
                  fontFamily: FONT_PRIMARY,
                  fontSize: vmin * 1.6,
                  textWrap: "nowrap",
                  color: "rgba(255,255,255,0.5)",
                  marginTop: vmin * 0.3,
                }}>
                  {srv.label}
                </div>
                {/* Per-server counter fragment */}
                {frame >= 300 && (
                  <div style={{
                    fontFamily: FONT_PRIMARY,
                    fontSize: vmin * 1.2,
                    color: BRAND,
                    marginTop: vmin * 0.2,
                    textWrap: "nowrap",
                    opacity: interpolate(frame, [300 + i * 10, 320 + i * 10], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }),
                  }}>
                    count: {Math.floor(983742 / 5 * (i + 1) / 5).toLocaleString()}
                  </div>
                )}
              </div>
            );
          })}

          {/* Counter split label */}
          {frame >= 340 && (
            <div style={{
              position: "absolute",
              left: "50%",
              bottom: 0,
              transform: "translateX(-50%)",
              whiteSpace: "nowrap",
              opacity: interpolate(frame, [340, 370], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }),
              background: "rgba(224,96,64,0.12)",
              border: `1px solid ${BRAND}`,
              borderRadius: vmin * 0.8,
              padding: `${vmin * 0.8}px ${vmin * 1.5}px`,
            }}>
              <span style={{ fontFamily: FONT_PRIMARY, fontSize: vmin * 1.4, color: "rgba(255,255,255,0.85)" }}>
                ⚠ Click count split across nodes
              </span>
            </div>
          )}
        </div>
      )}

      {/* ── Phase C: Coordination lines between servers ── */}
      {frame >= 400 && (
        <svg
          style={{
            position: "absolute",
            inset: 0,
            width: "100%",
            height: "100%",
            zIndex: 3,
            opacity: coordAppear * latencyPulse,
          }}
        >
          {/* Coordination lines between server positions */}
          {[
            [0, 1], [0, 2], [1, 2], [1, 3], [2, 3], [2, 4], [3, 4], [0, 4],
          ].map(([a, b], i) => {
            const sA = serverNodes[a];
            const sB = serverNodes[b];
            const clusterCX = rect.width * 0.50;
            const clusterCY = rect.height * 0.38;
            const x1 = clusterCX + vmin * sA.x;
            const y1 = clusterCY + vmin * sA.y;
            const x2 = clusterCX + vmin * sB.x;
            const y2 = clusterCY + vmin * sB.y;
            const lineDelay = 400 + i * 6;
            const lineOpacity = interpolate(frame, [lineDelay, lineDelay + 20], [0, 0.4], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
            return (
              <line
                key={i}
                x1={x1} y1={y1} x2={x2} y2={y2}
                stroke={DANGER}
                strokeWidth={2}
                strokeDasharray="8 5"
                opacity={lineOpacity}
              />
            );
          })}
        </svg>
      )}

      {/* ── Phase C: Latency callout ─────────────────── */}
      {frame >= 430 && (
        <div style={{
          position: "absolute",
          left: "50%",
          bottom: vmin * 8,
          transform: `translateX(-50%) translateY(${(1 - interpolate(frame, [430, 460], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" })) * 20}px)`,
          opacity: interpolate(frame, [430, 460, 555, 565], [0, 1, 1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }),
          zIndex: 10,
          textAlign: "center",
        }}>
          <div style={{
            background: "rgba(255,68,68,0.1)",
            border: `1px solid ${DANGER}`,
            borderRadius: vmin * 1,
            padding: `${vmin * 1.5}px ${vmin * 3}px`,
          }}>
            <div style={{ fontFamily: FONT_DISPLAY, fontSize: vmin * 2.5, color: DANGER, marginBottom: vmin * 0.5 }}>
              ⏱ Coordination Overhead
            </div>
            <div style={{ fontFamily: FONT_PRIMARY, fontSize: vmin * 1.6, color: "rgba(255,255,255,0.7)", lineHeight: 1.6 }}>
              Every server must stop, call every other server,<br />
              and wait for replies before responding.
            </div>
          </div>

          {/* Latency bar visualization */}
          <div style={{
            display: "flex",
            gap: vmin * 1,
            marginTop: vmin * 3,
            justifyContent: "center",
            alignItems: "flex-end",
          }}>
            {[1, 2, 3, 4, 5].map((n) => {
              const barH = interpolate(frame, [460, 520], [vmin * 1, vmin * (1 + n * 1.5)], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
              return (
                <div key={n} style={{
                  width: vmin * 2,
                  height: barH,
                  background: `linear-gradient(to top, ${DANGER}, ${BRAND})`,
                  borderRadius: vmin * 0.3,
                  opacity: 0.7,
                }}>
                  <div style={{
                    fontFamily: FONT_PRIMARY,
                    fontSize: vmin * 1.2,
                    color: "rgba(255,255,255,0.5)",
                    textAlign: "center",
                    marginTop: -vmin * 2,
                  }}>
                    {n * 50}ms
                  </div>
                </div>
              );
            })}
            <div style={{ fontFamily: FONT_PRIMARY, fontSize: vmin * 1.8, color: "rgba(255,255,255,0.4)", marginLeft: vmin * 0.5, alignSelf: "center" }}>
              more servers → more latency
            </div>
          </div>
        </div>
      )}

      {/* ── Phase D: Loading screen overlay ──────────── */}
      {frame >= 555 && (
        <div style={{
          position: "absolute",
          left: "50%",
          top: "50%",
          transform: `translate(-50%, -50%) scale(${loadingAppear})`,
          zIndex: 20,
          textAlign: "center",
        }}>
          <div style={{
            background: "rgba(15,15,20,0.92)",
            border: "1px solid rgba(255,255,255,0.1)",
            borderRadius: vmin * 2,
            padding: `${vmin * 5}px ${vmin * 8}px`,
            boxShadow: `0 0 ${vmin * 8}px rgba(0,0,0,0.6)`,
          }}>
            <Spinner size={vmin * 8} color={DANGER} angle={spinnerAngle} />
            <div style={{
              fontFamily: FONT_DISPLAY,
              fontSize: vmin * 2.8,
              color: "rgba(255,255,255,0.7)",
              marginTop: vmin * 2,
            }}>
              Fetching global count...
            </div>
            <div style={{
              fontFamily: FONT_PRIMARY,
              fontSize: vmin * 1.5,
              color: "rgba(255,255,255,0.35)",
              marginTop: vmin * 1,
            }}>
              Waiting for {serverNodes.length} servers to respond
            </div>

            {/* Latency timer counting up */}
            <div style={{
              fontFamily: FONT_PRIMARY,
              fontSize: vmin * 3.5,
              color: DANGER,
              marginTop: vmin * 2,
              fontVariantNumeric: "tabular-nums",
            }}>
              {((frame - 560) / fps).toFixed(1)}s
            </div>
          </div>
        </div>
      )}

      {/* ── Bottom problem statement ─────────────────── */}
      {frame >= 600 && (
        <div style={{
          position: "absolute",
          bottom: vmin * 4,
          left: "50%",
          transform: `translateX(-50%) translateY(${(1 - interpolate(frame, [600, 630], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" })) * 20}px)`,
          opacity: interpolate(frame, [600, 630], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }),
          zIndex: 21,
        }}>
          <div style={{
            background: "rgba(224,96,64,0.12)",
            border: `1px solid ${BRAND}`,
            borderRadius: vmin * 1,
            padding: `${vmin * 1.2}px ${vmin * 3}px`,
          }}>
            <AnimatedText
              style={{
                fontFamily: FONT_DISPLAY,
                fontSize: vmin * 2,
                fontWeight: 600,
                color: "rgba(255,255,255,0.9)",
              }}
              transition={{
                split: "word",
                splitStagger: 2,
                opacity: [0, 1],
                y: [8, 0],
                duration: 18,
                delay: 605,
                easing: "easeOutCubic",
              }}
            >
              Global consensus that doesn't actually matter yet.
            </AnimatedText>
          </div>
        </div>
      )}
    </AbsoluteFill>
  );
};
