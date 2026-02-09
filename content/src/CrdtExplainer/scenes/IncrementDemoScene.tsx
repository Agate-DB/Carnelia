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
 * Scene — Step-by-step PNCounter increment demo
 *
 * Walks through the alice / bob counter example:
 *   Phase 1 (0–120):   alice increments page_views +=5, +=3
 *   Phase 2 (120–220): bob increments page_views +=10, likes +=2
 *   Phase 3 (220–330): bidirectional sync → merge
 *   Phase 4 (330–420): converged view — both replicas identical
 *
 * AUDIO CUE: increment_demo_narration.mp3
 */

/* Floating 3D replica sphere for each peer */
const ReplicaSphere: React.FC<{
  position: [number, number, number];
  color: string;
  entrance: number;
  pulse?: boolean;
}> = ({ position, color, entrance, pulse }) => {
  const frame = useCurrentFrame();
  const s = entrance * 0.22;
  const p = pulse ? 1 + Math.sin(frame * 0.08) * 0.06 : 1;

  return (
    <group position={position}>
      <mesh scale={[s * p, s * p, s * p]} rotation={[0, frame * 0.005, 0.15]}>
        <icosahedronGeometry args={[1, 1]} />
        <meshStandardMaterial
          color={color}
          roughness={0.15}
          metalness={0.75}
          emissive={color}
          emissiveIntensity={0.35}
          transparent
          opacity={entrance * 0.8}
        />
      </mesh>
      {/* orbit ring */}
      <mesh rotation={[Math.PI / 2, 0, frame * 0.01]} scale={[s * 2.2, s * 2.2, s * 2.2]}>
        <torusGeometry args={[0.65, 0.005, 8, 40]} />
        <meshBasicMaterial color={color} transparent opacity={entrance * 0.12} />
      </mesh>
    </group>
  );
};

/* Sync beam between two 3D positions */
const SyncBeamLine: React.FC<{
  from: [number, number, number];
  to: [number, number, number];
  opacity: number;
  color: string;
}> = ({ from, to, opacity: beamOpacity, color }) => {
  const dx = to[0] - from[0];
  const dy = to[1] - from[1];
  const len = Math.sqrt(dx * dx + dy * dy);
  return (
    <mesh
      position={[(from[0] + to[0]) / 2, (from[1] + to[1]) / 2, 0]}
      rotation={[0, 0, Math.atan2(dy, dx)]}
    >
      <boxGeometry args={[len, 0.02, 0.02]} />
      <meshBasicMaterial color={color} transparent opacity={beamOpacity * 0.4} />
    </mesh>
  );
};

/* Counter row — displayed in the 2D overlay */
const CounterRow: React.FC<{
  label: string;
  value: number;
  highlight: boolean;
  animValue?: number;
  attrib?: string;
}> = ({ label, value, highlight, animValue, attrib }) => {
  const displayVal = animValue !== undefined ? Math.round(animValue) : value;
  return (
    <div style={{
      display: "flex", justifyContent: "space-between", alignItems: "center",
      padding: "3px 0", borderBottom: "1px solid rgba(255,255,255,0.04)",
    }}>
      <span style={{
        fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.5)",
      }}>
        {label}
      </span>
      <div style={{ display: "flex", alignItems: "baseline", gap: 6 }}>
        <span style={{
          fontFamily: FONT_DISPLAY, fontSize: 16,
          color: highlight ? "#6eff9e" : "rgba(255,255,255,0.7)",
          transition: "color 0.3s",
        }}>
          {displayVal}
        </span>
        {attrib && (
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 9, color: "rgba(255,255,255,0.28)" }}>
            {attrib}
          </span>
        )}
      </div>
    </div>
  );
};

export const IncrementDemoScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();

  /* ---- phase gates ---- */
  const ph1 = frame >= 0;
  const ph2 = frame >= 120;
  const ph3 = frame >= 220;
  const ph4 = frame >= 330;

  /* ---- entrances ---- */
  const aliceEnt = spring({ frame, fps, delay: 5, config: { damping: 14 } });
  const bobEnt = spring({ frame, fps, delay: 20, config: { damping: 14 } });

  /* ---- alice increments ---- */
  const alicePv1 = interpolate(frame, [40, 60], [0, 5], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const alicePv2 = interpolate(frame, [80, 100], [5, 8], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const alicePv = ph1 ? (frame < 80 ? alicePv1 : alicePv2) : 0;

  /* ---- bob increments ---- */
  const bobPv = ph2 ? interpolate(frame, [140, 165], [0, 10], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }) : 0;
  const bobLikes = ph2 ? interpolate(frame, [175, 195], [0, 2], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }) : 0;

  /* ---- sync & merge ---- */
  const syncProgress = ph3 ? interpolate(frame, [230, 280], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }) : 0;
  const syncBeamOpacity = ph3 ? interpolate(frame, [225, 240], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }) * interpolate(frame, [290, 310], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }) : 0;

  /* ---- merged values ---- */
  const mergedLikes = ph3 ? interpolate(syncProgress, [0, 1], [0, 2]) : 0;

  /* ---- phase labels ---- */
  const phaseLabel = ph4 ? "Phase 4 — Converged" : ph3 ? "Phase 3 — Bidirectional Sync" : ph2 ? "Phase 2 — Bob increments" : "Phase 1 — Alice increments";
  const phaseColor = ph4 ? "#6eff9e" : ph3 ? "#c9a0ff" : ph2 ? "#4ab5ff" : "#ffaa44";
  const phaseOpacity = spring({ frame, fps, delay: ph4 ? 332 : ph3 ? 222 : ph2 ? 122 : 5, config: { damping: 20 } });

  /* ---- final result ---- */
  const resultOpacity = ph4 ? interpolate(frame, [340, 360], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }) : 0;

  /* ---- layout helpers ---- */
  const alicePos: [number, number, number] = [-2.0, 0, 0];
  const bobPos: [number, number, number] = [2.0, 0, 0];

  const fadeOut = interpolate(frame, [400, 420], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  /* ---- stepped highlights ---- */
  const aliceHighlight = (ph1 && !ph2) || (ph3 && !ph4) || ph4;
  const bobHighlight = (ph2 && !ph3) || (ph3 && !ph4) || ph4;

  /* ---- alice panel values ---- */
  const alicePanelPv = ph4 ? 10 : ph3 ? interpolate(syncProgress, [0, 1], [8, 10]) : alicePv;
  const alicePanelLikes = ph4 ? 2 : ph3 ? mergedLikes : 0;

  /* ---- bob panel values ---- */
  const bobPanelPv = ph4 ? 10 : ph3 ? 10 : bobPv;
  const bobPanelLikes = ph4 ? 2 : ph3 ? 2 : bobLikes;

  /* ---- per-replica attribution ---- */
  const alicePvAttrib = (ph4 || (ph3 && syncProgress > 0.5)) ? "(bob:+10)" : (ph1 && !ph2 && frame >= 40) ? `(alice:+${Math.round(alicePv)})` : (ph2 && !ph3) ? "(alice:+8)" : "";
  const aliceLikesAttrib = (ph3 || ph4) ? "(bob:+2)" : "";
  const bobPvAttrib = (ph3 || ph4) ? "(bob:+10)" : (ph2 && frame >= 140) ? `(bob:+${Math.round(bobPv)})` : "";
  const bobLikesAttrib = (ph3 || ph4) ? "(bob:+2)" : (ph2 && frame >= 175) ? `(bob:+${Math.round(bobLikes)})` : "";

  return (
    <AbsoluteFill style={{ backgroundColor: "#1e1e1e", opacity: fadeOut }}>
      <ThreeCanvas linear width={width} height={height}>
        <color attach="background" args={["#1e1e1e"]} />
        <ambientLight intensity={0.35} />
        <pointLight position={[4, 4, 4]} intensity={0.7} color="#7ecfff" />
        <pointLight position={[-4, -2, 3]} intensity={0.5} color="#ff9e7e" />

        <ReplicaSphere position={alicePos} color="#ffaa44" entrance={aliceEnt} pulse={aliceHighlight} />
        <ReplicaSphere position={bobPos} color="#4ab5ff" entrance={bobEnt} pulse={bobHighlight} />

        {/* Sync beams */}
        {ph3 && (
          <>
            <SyncBeamLine from={alicePos} to={bobPos} opacity={syncBeamOpacity} color="#c9a0ff" />
            {/* Delta cubes traveling */}
            {[0.3, 0.55, 0.8].map((t) => {
              const tVal = (syncProgress + t) % 1;
              const x = interpolate(tVal, [0, 1], [alicePos[0], bobPos[0]]);
              const s = 0.04 * syncBeamOpacity;
              return (
                <mesh key={`d${t}`} position={[x, Math.sin(tVal * Math.PI) * 0.15, 0.1]} scale={[s, s, s]}>
                  <boxGeometry args={[1, 1, 1]} />
                  <meshBasicMaterial color="#c9a0ff" transparent opacity={syncBeamOpacity * 0.6} />
                </mesh>
              );
            })}
          </>
        )}

        {/* Convergence flash */}
        {ph4 && (
          <mesh position={[0, 0, -0.5]} scale={[interpolate(frame, [330, 345], [0, 3], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }), interpolate(frame, [330, 345], [0, 3], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }), 1]}>
            <ringGeometry args={[0.8, 1.0, 40]} />
            <meshBasicMaterial color="#6eff9e" transparent opacity={interpolate(frame, [330, 355], [0.35, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" })} />
          </mesh>
        )}
      </ThreeCanvas>

      <AbsoluteFill style={{ pointerEvents: "none" }}>
        {/* Phase indicator */}
        <div style={{
          position: "absolute", top: 30, left: 0, right: 0, textAlign: "center",
          opacity: phaseOpacity,
        }}>
          <span style={{ fontFamily: FONT_DISPLAY, fontSize: 18, color: phaseColor }}>
            {phaseLabel}
          </span>
        </div>

        {/* Title */}
        <div style={{ position: "absolute", top: 60, left: 60, opacity: aliceEnt }}>
          <span style={{ fontFamily: FONT_DISPLAY, fontSize: 14, color: "rgba(255,255,255,0.3)" }}>
            PNCounter: page_views + likes
          </span>
        </div>

        {/* Alice replica panel */}
        <div style={{
          position: "absolute", left: "6%", top: "30%",
          background: "rgba(255,170,68,0.04)", border: "1px solid rgba(255,170,68,0.15)",
          borderRadius: 10, padding: "14px 20px", width: 180, opacity: aliceEnt,
        }}>
          <div style={{ fontFamily: FONT_PRIMARY, fontSize: 14, color: "#ffaa44", marginBottom: 10 }}>
            Alice&apos;s Replica
          </div>
          <CounterRow label="page_views" value={10} highlight={aliceHighlight} animValue={alicePanelPv} attrib={alicePvAttrib} />
          <CounterRow label="likes" value={2} highlight={ph3 || ph4} animValue={alicePanelLikes} attrib={aliceLikesAttrib} />

          {/* Step annotations */}
          {ph1 && !ph2 && (
            <div style={{ marginTop: 8, fontFamily: FONT_PRIMARY, fontSize: 10, color: "rgba(255,170,68,0.5)", lineHeight: 1.5 }}>
              {frame >= 40 && <div>page_views += 5</div>}
              {frame >= 80 && <div>page_views += 3  (total: 8)</div>}
            </div>
          )}
        </div>

        {/* Bob replica panel */}
        <div style={{
          position: "absolute", right: "6%", top: "30%",
          background: "rgba(74,181,255,0.04)", border: "1px solid rgba(74,181,255,0.15)",
          borderRadius: 10, padding: "14px 20px", width: 180, opacity: bobEnt,
        }}>
          <div style={{ fontFamily: FONT_PRIMARY, fontSize: 14, color: "#4ab5ff", marginBottom: 10 }}>
            Bob&apos;s Replica
          </div>
          <CounterRow label="page_views" value={10} highlight={bobHighlight} animValue={bobPanelPv} attrib={bobPvAttrib} />
          <CounterRow label="likes" value={2} highlight={ph2 || ph3 || ph4} animValue={bobPanelLikes} attrib={bobLikesAttrib} />

          {/* Step annotations */}
          {ph2 && !ph3 && (
            <div style={{ marginTop: 8, fontFamily: FONT_PRIMARY, fontSize: 10, color: "rgba(74,181,255,0.5)", lineHeight: 1.5 }}>
              {frame >= 140 && <div>page_views += 10</div>}
              {frame >= 175 && <div>likes += 2</div>}
            </div>
          )}
        </div>

        {/* Sync annotation */}
        {ph3 && !ph4 && (
          <div style={{
            position: "absolute", left: "50%", top: "46%", transform: "translate(-50%, -50%)",
            background: "rgba(201,160,255,0.06)", border: "1px solid rgba(201,160,255,0.12)",
            borderRadius: 8, padding: "10px 18px", maxWidth: 200, textAlign: "center",
            opacity: syncBeamOpacity,
          }}>
            <p style={{ fontFamily: FONT_PRIMARY, fontSize: 11, color: "#c9a0ff", margin: 0, lineHeight: 1.6 }}>
              bob ──sync──▶ alice ✓<br />
              alice ──sync──▶ bob ✓<br />
              <span style={{ color: "rgba(255,255,255,0.3)", fontSize: 10 }}>Bidirectional CRDT merge</span>
            </p>
          </div>
        )}

        {/* Converged result */}
        <div style={{
          position: "absolute", bottom: 50, left: 0, right: 0, textAlign: "center",
          opacity: resultOpacity,
        }}>
          <div style={{
            display: "inline-block", background: "rgba(110,255,158,0.05)",
            border: "1px solid rgba(110,255,158,0.15)", borderRadius: 10,
            padding: "12px 28px", textAlign: "center",
          }}>
            <div style={{ fontFamily: FONT_DISPLAY, fontSize: 16, color: "#6eff9e", marginBottom: 4 }}>
              ✓ ALL REPLICAS CONVERGED
            </div>
            <span style={{ fontFamily: FONT_PRIMARY, fontSize: 14, color: "rgba(255,255,255,0.7)" }}>
              page_views = <span style={{ color: "#ffaa44" }}>10</span>{" "}
              <span style={{ fontSize: 10, color: "rgba(255,255,255,0.3)" }}>(alice:8 + bob:10)</span>
              {" · "}
              likes = <span style={{ color: "#4ab5ff" }}>2</span>{" "}
              <span style={{ fontSize: 10, color: "rgba(255,255,255,0.3)" }}>(bob:2)</span>
            </span>
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
