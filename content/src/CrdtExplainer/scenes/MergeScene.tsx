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
 * Scene 5 — Strong Eventual Consistency: Merge & Convergence
 *
 * Three replica spheres with different internal states converge into
 * one final merged state. All three slide together and their colors
 * blend, showing deterministic convergence regardless of delivery
 * order.
 *
 * Concept grounding (architecture doc §1.1 & analysis §5):
 * - "Strong Eventual Consistency (SEC)"
 * - "all replicas reach an equivalent state once same updates delivered"
 * - "any two replicas that agree on DAG heads → identical history"
 *
 * AUDIO CUE: merge_narration.mp3
 */

/** Replica sphere that can split/converge */
const ConvergingNode: React.FC<{
  basePosition: [number, number, number];
  mergePosition: [number, number, number];
  convergence: number;
  color: string;
  mergedColor: string;
  entrance: number;
}> = ({ basePosition, mergePosition, convergence, color, mergedColor, entrance }) => {
  const frame = useCurrentFrame();

  const x = interpolate(convergence, [0, 1], [basePosition[0], mergePosition[0]]);
  const y = interpolate(convergence, [0, 1], [basePosition[1], mergePosition[1]]) + Math.sin(frame * 0.025) * 0.04;
  const z = interpolate(convergence, [0, 1], [basePosition[2], mergePosition[2]]);

  // Blend colors
  const matColor = convergence > 0.8 ? mergedColor : color;
  const glow = interpolate(convergence, [0.6, 1], [0.2, 0.8], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const s = entrance * interpolate(convergence, [0, 0.5, 1], [0.32, 0.28, 0.38], {
    extrapolateLeft: "clamp", extrapolateRight: "clamp",
  });

  return (
    <group position={[x, y, z]}>
      <mesh scale={[s, s, s]}>
        <sphereGeometry args={[1, 32, 32]} />
        <meshStandardMaterial
          color={matColor}
          roughness={0.15}
          metalness={0.8}
          emissive={matColor}
          emissiveIntensity={glow}
          transparent
          opacity={entrance}
        />
      </mesh>
      <mesh rotation={[Math.PI / 2, 0, frame * 0.008]} scale={[s * 2.4, s * 2.4, s * 2.4]}>
        <torusGeometry args={[0.45, 0.008, 8, 64]} />
        <meshBasicMaterial color={matColor} transparent opacity={entrance * 0.2} />
      </mesh>
    </group>
  );
};

/** Merge flash effect */
const MergeFlash: React.FC<{ active: boolean }> = ({ active }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  if (!active) return null;
  const pulse = spring({ frame: frame - 140, fps, config: { damping: 6, stiffness: 200 } });
  const scale = pulse * 1.2;
  const opacity = Math.max(0, 0.4 - pulse * 0.4);

  return (
    <mesh position={[0, 0, 0]} scale={[scale, scale, scale]}>
      <sphereGeometry args={[1, 32, 32]} />
      <meshBasicMaterial color="#c9a0ff" transparent opacity={opacity} />
    </mesh>
  );
};

/** "Equal" sign group — shows ≡ once merge is complete */
const EqualitySign: React.FC<{
  visible: boolean;
  position: [number, number, number];
}> = ({ visible, position }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  if (!visible) return null;

  const ent = spring({ frame: frame - 170, fps, config: { damping: 14 } });
  const s = ent * 0.15;

  return (
    <group position={position}>
      <mesh position={[0, 0.035, 0]} scale={[s, s * 0.1, s * 0.1]}>
        <boxGeometry args={[2, 1, 1]} />
        <meshStandardMaterial color="#6eff9e" emissive="#6eff9e" emissiveIntensity={0.6} />
      </mesh>
      <mesh position={[0, -0.035, 0]} scale={[s, s * 0.1, s * 0.1]}>
        <boxGeometry args={[2, 1, 1]} />
        <meshStandardMaterial color="#6eff9e" emissive="#6eff9e" emissiveIntensity={0.6} />
      </mesh>
    </group>
  );
};

/** Orbit trail ring that follows each converging node */
const OrbitTrail: React.FC<{
  basePosition: [number, number, number];
  mergePosition: [number, number, number];
  convergence: number;
  color: string;
  entrance: number;
}> = ({ basePosition, mergePosition, convergence, color, entrance }) => {
  const frame = useCurrentFrame();
  const x = interpolate(convergence, [0, 1], [basePosition[0], mergePosition[0]]);
  const y = interpolate(convergence, [0, 1], [basePosition[1], mergePosition[1]]);
  const opacity = entrance * interpolate(convergence, [0, 0.7, 1], [0.2, 0.15, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const s = entrance * 0.5;

  return (
    <mesh position={[x, y, 0]} rotation={[Math.PI / 3, frame * 0.01, 0]} scale={[s, s, s]}>
      <torusGeometry args={[1, 0.008, 8, 48]} />
      <meshBasicMaterial color={color} transparent opacity={opacity} />
    </mesh>
  );
};

/** Convergence radial burst at merge point */
const ConvergenceBurst: React.FC<{ active: boolean }> = ({ active }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  if (!active) return null;

  const burstCount = 8;
  const progress = spring({ frame: frame - 145, fps, config: { damping: 12 } });
  return (
    <group>
      {Array.from({ length: burstCount }).map((_, i) => {
        const angle = (i / burstCount) * Math.PI * 2;
        const r = progress * 1.5;
        return (
          <mesh key={i} position={[Math.cos(angle) * r, Math.sin(angle) * r, 0]} scale={[0.02, 0.02, 0.02]}>
            <sphereGeometry args={[1, 8, 8]} />
            <meshBasicMaterial color="#c9a0ff" transparent opacity={Math.max(0, 0.6 - progress * 0.6)} />
          </mesh>
        );
      })}
    </group>
  );
};

export const MergeScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();

  const convergence = interpolate(frame, [80, 145], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const entranceA = spring({ frame, fps, delay: 5, config: { damping: 14 } });
  const entranceB = spring({ frame, fps, delay: 12, config: { damping: 14 } });
  const entranceC = spring({ frame, fps, delay: 19, config: { damping: 14 } });

  const titleOpacity = interpolate(frame, [5, 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Equation text
  const eqProgress = spring({ frame, fps, delay: 40, config: { damping: 200 } });
  const eqOpacity = interpolate(eqProgress, [0, 1], [0, 1]);
  const eqY = interpolate(eqProgress, [0, 1], [10, 0]);

  // "SEC guaranteed" callout
  const secOpacity = interpolate(frame, [150, 170], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const secY = interpolate(spring({ frame, fps, delay: 150, config: { damping: 200 } }), [0, 1], [10, 0]);

  const fadeOut = interpolate(frame, [470, 500], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ backgroundColor: "#1e1e1e", opacity: fadeOut }}>
      <ThreeCanvas linear width={width} height={height}>
        <color attach="background" args={["#1e1e1e"]} />
        <ambientLight intensity={0.4} />
        <pointLight position={[5, 5, 5]} intensity={1} color="#6ea0ff" />
        <pointLight position={[-5, -3, 3]} intensity={0.5} color="#a06eff" />
        <directionalLight position={[0, 5, 5]} intensity={0.3} />

        <ConvergingNode
          basePosition={[-2.5, 0.5, 0]}
          mergePosition={[0, 0, 0]}
          convergence={convergence}
          color="#4a9eff"
          mergedColor="#c9a0ff"
          entrance={entranceA}
        />
        <ConvergingNode
          basePosition={[2.5, 0.5, 0]}
          mergePosition={[0, 0, 0]}
          convergence={convergence}
          color="#ff6a9e"
          mergedColor="#c9a0ff"
          entrance={entranceB}
        />
        <ConvergingNode
          basePosition={[0, -2, 0]}
          mergePosition={[0, 0, 0]}
          convergence={convergence}
          color="#6eff9e"
          mergedColor="#c9a0ff"
          entrance={entranceC}
        />

        <MergeFlash active={frame >= 140 && convergence >= 0.95} />
        <ConvergenceBurst active={frame >= 145 && convergence >= 0.95} />
        <EqualitySign visible={frame >= 170} position={[1.5, 0, 0.5]} />

        {/* Orbit trails */}
        <OrbitTrail basePosition={[-2.5, 0.5, 0]} mergePosition={[0, 0, 0]} convergence={convergence} color="#4a9eff" entrance={entranceA} />
        <OrbitTrail basePosition={[2.5, 0.5, 0]} mergePosition={[0, 0, 0]} convergence={convergence} color="#ff6a9e" entrance={entranceB} />
        <OrbitTrail basePosition={[0, -2, 0]} mergePosition={[0, 0, 0]} convergence={convergence} color="#6eff9e" entrance={entranceC} />
      </ThreeCanvas>

      <AbsoluteFill style={{ pointerEvents: "none" }}>
        {/* Title */}
        <div style={{ position: "absolute", top: 35, left: 60, opacity: titleOpacity }}>
          <span style={{ fontFamily: FONT_DISPLAY, fontSize: 26, color: "rgba(255,255,255,0.9)" }}>
            Strong Eventual Consistency
          </span>
        </div>

        {/* Delivery order badge */}
        <div style={{
          position: "absolute",
          top: 70,
          right: 50,
          opacity: eqOpacity,
          background: "rgba(255,255,255,0.03)",
          border: "1px solid rgba(255,255,255,0.08)",
          borderRadius: 10,
          padding: "10px 16px",
          maxWidth: 240,
        }}>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.45)", margin: 0, marginBottom: 4 }}>delivery order</p>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 16, color: "#c9a0ff", margin: 0, lineHeight: 1.5 }}>
            ABC = BCA = CAB
          </p>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.3)", margin: 0, marginTop: 4 }}>
            any permutation → same result
          </p>
        </div>

        {/* Equation */}
        <div style={{ position: "absolute", top: "18%", right: 60, opacity: eqOpacity, transform: `translateY(${eqY}px)` }}>
          <div style={{
            fontFamily: FONT_PRIMARY,
            fontSize: 20,
            color: "rgba(255,255,255,0.7)",
            background: "rgba(255,255,255,0.03)",
            border: "1px solid rgba(255,255,255,0.08)",
            borderRadius: 8,
            padding: "10px 18px",
          }}>
            S₁ ⊔ S₂ ⊔ S₃ → S<sub>final</sub>
          </div>
        </div>

        {/* Node labels (fade as they converge) */}
        <div style={{ position: "absolute", left: "14%", top: "38%", opacity: titleOpacity * (1 - convergence) }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 16, color: "#4a9eff" }}>A: {"{ x:1, y:3 }"}</span>
        </div>
        <div style={{ position: "absolute", right: "14%", top: "38%", opacity: titleOpacity * (1 - convergence) }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 16, color: "#ff6a9e" }}>B: {"{ x:1, y:5 }"}</span>
        </div>
        <div style={{ position: "absolute", left: "43%", bottom: "18%", opacity: titleOpacity * (1 - convergence) }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 16, color: "#6eff9e" }}>C: {"{ x:2, y:3 }"}</span>
        </div>

        {/* Merged state label */}
        {convergence > 0.9 && (
          <div style={{ position: "absolute", left: "42%", top: "52%", opacity: secOpacity }}>
            <span style={{ fontFamily: FONT_PRIMARY, fontSize: 17, color: "#c9a0ff" }}>
              {"{ x:2, y:5 }"}
            </span>
          </div>
        )}

        {/* SEC callout */}
        <div style={{
          position: "absolute",
          bottom: 50,
          left: 0,
          right: 0,
          textAlign: "center",
          opacity: secOpacity,
          transform: `translateY(${secY}px)`,
        }}>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 22, color: "white", margin: 0 }}>
            Same updates delivered → identical state
          </p>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 16, color: "rgba(255,255,255,0.4)", marginTop: 6 }}>
            No coordination, no consensus protocol required
          </p>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
