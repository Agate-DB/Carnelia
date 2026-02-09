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
 * Scene 4 — δ-CRDT: Delta Propagation
 *
 * Two replica cubes connected by a channel. Instead of shipping the
 * full state (big cube), only a tiny delta (small cube/particle) is
 * sent. Shows the bandwidth advantage: Δ ≪ S.
 *
 * Concept grounding (architecture doc §2.1):
 * - "delta-mutator function generates a delta"
 * - "deltas are idempotent, commutative, and associative"
 * - "minimizes network traffic by disseminating small, incremental changes"
 *
 * AUDIO CUE: delta_narration.mp3
 */

/** State block — big translucent cube representing full state */
const StateBlock: React.FC<{
  position: [number, number, number];
  entrance: number;
  color: string;
  label: string;
}> = ({ position, entrance, color }) => {
  const frame = useCurrentFrame();
  const s = entrance * 0.55;

  return (
    <group position={position}>
      {/* Solid state cube */}
      <mesh scale={[s, s, s]} rotation={[0.35, frame * 0.004, 0.15]}>
        <boxGeometry args={[1, 1, 1]} />
        <meshStandardMaterial
          color={color}
          roughness={0.2}
          metalness={0.7}
          transparent
          opacity={entrance * 0.45}
          emissive={color}
          emissiveIntensity={0.15}
        />
      </mesh>
      <mesh scale={[s * 1.01, s * 1.01, s * 1.01]} rotation={[0.35, frame * 0.004, 0.15]}>
        <boxGeometry args={[1, 1, 1]} />
        <meshBasicMaterial color={color} wireframe transparent opacity={entrance * 0.25} />
      </mesh>
    </group>
  );
};

/** Delta particle — small bright cube that flies across */
const DeltaParticle: React.FC<{
  fromX: number;
  toX: number;
  startFrame: number;
  duration: number;
  color: string;
  y: number;
}> = ({ fromX, toX, startFrame, duration, color, y }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const localFrame = frame - startFrame;
  if (localFrame < 0 || localFrame > duration) return null;

  const t = spring({ frame: localFrame, fps, config: { damping: 18, mass: 0.6 } });
  const x = interpolate(t, [0, 1], [fromX, toX]);
  const arcY = y + Math.sin(t * Math.PI) * 0.4;
  const pulse = 1 + Math.sin(localFrame * 0.3) * 0.1;

  return (
    <group position={[x, arcY, 0.5]}>
      <mesh scale={[0.08 * pulse, 0.08 * pulse, 0.08 * pulse]}>
        <octahedronGeometry args={[1, 0]} />
        <meshStandardMaterial
          color={color}
          emissive={color}
          emissiveIntensity={2}
          roughness={0}
          metalness={0.5}
        />
      </mesh>
      {/* Glow halo */}
      <mesh scale={[0.25 * pulse, 0.25 * pulse, 0.01]}>
        <circleGeometry args={[1, 16]} />
        <meshBasicMaterial color={color} transparent opacity={0.15} />
      </mesh>
    </group>
  );
};

/** "Full state" ghost cube — shows what we're NOT sending */
const FullStateGhost: React.FC<{
  position: [number, number, number];
  visible: boolean;
}> = ({ position, visible }) => {
  const frame = useCurrentFrame();
  const opacity = visible ? interpolate(frame, [60, 75], [0, 0.15], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }) : 0;

  if (opacity < 0.01) return null;
  return (
    <mesh position={position} rotation={[0.2, 0.3, 0]} scale={[0.55, 0.55, 0.55]}>
      <boxGeometry args={[1, 1, 1]} />
      <meshBasicMaterial color="#ff4444" wireframe transparent opacity={opacity} />
    </mesh>
  );
};

/** Channel line between nodes — enhanced with animated pulse dots */
const Channel: React.FC<{
  fromX: number;
  toX: number;
  y: number;
  opacity: number;
}> = ({ fromX, toX, y, opacity }) => {
  const frame = useCurrentFrame();
  const length = toX - fromX;
  const count = 12;
  const pulseCount = 5;

  return (
    <group>
      {/* Static dash segments */}
      {Array.from({ length: count }).map((_, i) => {
        const t = (i + 0.5) / count;
        return (
          <mesh key={`d${i}`} position={[fromX + length * t, y, 0]}>
            <boxGeometry args={[length / count * 0.4, 0.008, 0.008]} />
            <meshBasicMaterial color="#ffffff" transparent opacity={opacity * 0.15} />
          </mesh>
        );
      })}
      {/* Animated pulse dots traveling along the channel */}
      {Array.from({ length: pulseCount }).map((_, i) => {
        const t = ((frame * 0.012 + i / pulseCount) % 1);
        const x = fromX + length * t;
        const blink = Math.sin(frame * 0.1 + i * 2) * 0.3 + 0.5;
        return (
          <mesh key={`p${i}`} position={[x, y, 0.05]} scale={[0.015, 0.015, 0.015]}>
            <sphereGeometry args={[1, 8, 8]} />
            <meshBasicMaterial color="#6eff9e" transparent opacity={opacity * blink} />
          </mesh>
        );
      })}
    </group>
  );
};

/** Bandwidth comparison bars (full state vs delta) */
const BandwidthBars: React.FC<{ entrance: number; frame: number }> = ({ entrance, frame }) => {
  const fullWidth = entrance * 1.2;
  const deltaWidth = entrance * 0.18;
  const pulse = Math.sin(frame * 0.05) * 0.02;

  return (
    <group position={[0, -1.6, 0.5]}>
      {/* Full state bar (red, big) */}
      <mesh position={[0, 0.15, 0]} scale={[fullWidth, 0.08, 0.04]}>
        <boxGeometry args={[1, 1, 1]} />
        <meshStandardMaterial color="#ff4444" emissive="#ff4444" emissiveIntensity={0.3} transparent opacity={entrance * 0.5} />
      </mesh>
      {/* Delta bar (green, small) */}
      <mesh position={[-fullWidth * 0.5 + deltaWidth * 0.5 + pulse, -0.15, 0]} scale={[deltaWidth, 0.08, 0.04]}>
        <boxGeometry args={[1, 1, 1]} />
        <meshStandardMaterial color="#6eff9e" emissive="#6eff9e" emissiveIntensity={0.5} transparent opacity={entrance * 0.7} />
      </mesh>
    </group>
  );
};

export const DeltaScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();

  const nodeAX = -2.5;
  const nodeBX = 2.5;

  const entranceA = spring({ frame, fps, delay: 5, config: { damping: 14 } });
  const entranceB = spring({ frame, fps, delay: 15, config: { damping: 14 } });
  const channelOpacity = interpolate(frame, [25, 45], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Title + description
  const titleOpacity = interpolate(frame, [5, 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // "Not this" cross-out for full state
  const crossOpacity = interpolate(frame, [80, 95], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Delta label
  const deltaLblOpacity = interpolate(frame, [55, 70], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Bottom callout
  const calloutopacity = interpolate(frame, [110, 130], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const calloutY = interpolate(spring({ frame, fps, delay: 110, config: { damping: 200 } }), [0, 1], [15, 0]);

  const fadeOut = interpolate(frame, [210, 240], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ backgroundColor: "#0a0a1a", opacity: fadeOut }}>
      <ThreeCanvas linear width={width} height={height}>
        <color attach="background" args={["#0a0a1a"]} />
        <ambientLight intensity={0.4} />
        <pointLight position={[5, 5, 5]} intensity={1} color="#6ea0ff" />
        <pointLight position={[-5, -3, 3]} intensity={0.5} color="#a06eff" />

        <StateBlock position={[nodeAX, 0, 0]} entrance={entranceA} color="#4a9eff" label="Replica A" />
        <StateBlock position={[nodeBX, 0, 0]} entrance={entranceB} color="#ff6a9e" label="Replica B" />

        <Channel fromX={nodeAX + 0.8} toX={nodeBX - 0.8} y={0} opacity={channelOpacity} />

        {/* Delta particles flying across (multiple waves) */}
        <DeltaParticle fromX={nodeAX + 0.6} toX={nodeBX - 0.6} startFrame={50} duration={40} color="#6eff9e" y={0.15} />
        <DeltaParticle fromX={nodeAX + 0.6} toX={nodeBX - 0.6} startFrame={60} duration={40} color="#ffc46a" y={-0.1} />
        <DeltaParticle fromX={nodeBX - 0.6} toX={nodeAX + 0.6} startFrame={100} duration={40} color="#c9a0ff" y={0.05} />
        <DeltaParticle fromX={nodeBX - 0.6} toX={nodeAX + 0.6} startFrame={110} duration={40} color="#6affea" y={-0.2} />

        {/* Full state ghost — shows what traditional CvRDTs ship */}
        <FullStateGhost position={[(nodeAX + nodeBX) / 2, 0.8, 0.3]} visible={frame >= 60 && frame < 130} />

        {/* Bandwidth comparison bars */}
        <BandwidthBars entrance={calloutopacity} frame={frame} />
      </ThreeCanvas>

      <AbsoluteFill style={{ pointerEvents: "none" }}>
        {/* Title */}
        <div style={{ position: "absolute", top: 35, left: 60, opacity: titleOpacity }}>
          <span style={{ fontFamily: FONT_DISPLAY, fontSize: 22, color: "rgba(255,255,255,0.9)" }}>
            δ-CRDT: Delta Mutations
          </span>
        </div>

        {/* Node labels */}
        <div style={{ position: "absolute", left: "15%", top: "54%", opacity: titleOpacity }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 14, color: "#4a9eff" }}>Replica A</span>
        </div>
        <div style={{ position: "absolute", right: "15%", top: "54%", opacity: titleOpacity }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 14, color: "#ff6a9e" }}>Replica B</span>
        </div>

        {/* Delta label on the particle path */}
        <div style={{ position: "absolute", left: "46%", top: "36%", opacity: deltaLblOpacity }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 15, color: "#6eff9e" }}>Δ (delta)</span>
        </div>

        {/* Full state "not this" with cross */}
        <div style={{ position: "absolute", left: "43%", top: "22%", opacity: crossOpacity }}>
          <span style={{
            fontFamily: FONT_PRIMARY,
            fontSize: 13,
            color: "#ff4444",
            textDecoration: "line-through",
          }}>full state S</span>
        </div>

        {/* Mutator formula */}
        <div style={{
          position: "absolute",
          top: 70,
          right: 50,
          opacity: deltaLblOpacity,
          background: "rgba(255,255,255,0.03)",
          border: "1px solid rgba(255,255,255,0.08)",
          borderRadius: 10,
          padding: "10px 16px",
        }}>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 11, color: "rgba(255,255,255,0.45)", margin: 0, marginBottom: 4 }}>delta-mutator</p>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 14, color: "#6eff9e", margin: 0 }}>
            m(X) = X ⊔ mδ(X)
          </p>
        </div>

        {/* Bottom callout: bandwidth comparison */}
        <div style={{
          position: "absolute",
          bottom: 45,
          left: 0,
          right: 0,
          textAlign: "center",
          opacity: calloutopacity,
          transform: `translateY(${calloutY}px)`,
        }}>
          <div style={{
            fontFamily: FONT_PRIMARY,
            fontSize: 22,
            color: "white",
            display: "flex",
            justifyContent: "center",
            alignItems: "center",
            gap: 16,
          }}>
            <span style={{ color: "#6eff9e" }}>Δ</span>
            <span style={{ color: "rgba(255,255,255,0.3)", fontSize: 18 }}>≪</span>
            <span style={{ color: "#ff4444", textDecoration: "line-through", opacity: 0.5, fontSize: 18 }}>S</span>
          </div>
          <div style={{ display: "flex", justifyContent: "center", gap: 24, marginTop: 6 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <div style={{ width: 40, height: 6, background: "#ff4444", borderRadius: 3, opacity: 0.5 }} />
              <span style={{ fontFamily: FONT_PRIMARY, fontSize: 11, color: "rgba(255,255,255,0.35)" }}>full state</span>
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <div style={{ width: 10, height: 6, background: "#6eff9e", borderRadius: 3 }} />
              <span style={{ fontFamily: FONT_PRIMARY, fontSize: 11, color: "rgba(255,255,255,0.35)" }}>delta Δ</span>
            </div>
          </div>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 14, color: "rgba(255,255,255,0.45)", marginTop: 8 }}>
            Ship only what changed — idempotent, commutative, associative
          </p>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
