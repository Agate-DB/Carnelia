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
 * Scene 6 — Tombstone-Free Removal: Dot Store & Causal Context
 *
 * Visualizes how the MDCS achieves remove semantics without tombstones.
 * The causal context (large ring) tracks all dots ever created.
 * The dot store (inner cluster) contains only "live" dots.
 * Removing an element purges its dot from the store but keeps it
 * in the context — the absence IS the deletion record.
 *
 * Concept grounding (architecture doc §2.3):
 * - "causal context: set of all dots that have been created"
 * - "dot store: contains only dots corresponding to currently live data"
 * - "An item is removed if dot in context but absent from dot store"
 * - "prevents unbounded metadata growth"
 *
 * AUDIO CUE: dotstore_narration.mp3
 */

/** A single dot sphere */
const Dot: React.FC<{
  position: [number, number, number];
  color: string;
  size: number;
  alive: boolean;
  entrance: number;
  fadeOutProgress: number; // 0..1, 1 = fully removed from store
}> = ({ position, color, size, alive, entrance, fadeOutProgress }) => {
  const frame = useCurrentFrame();
  const s = entrance * size * (alive ? 1 : Math.max(0, 1 - fadeOutProgress));
  const yBob = Math.sin(frame * 0.03 + position[0] * 5) * 0.04;
  const opacity = alive ? entrance : entrance * Math.max(0.05, 1 - fadeOutProgress);

  if (s < 0.001) return null;

  return (
    <mesh position={[position[0], position[1] + yBob * entrance, position[2]]} scale={[s, s, s]}>
      <sphereGeometry args={[1, 16, 16]} />
      <meshStandardMaterial
        color={alive ? color : "#555555"}
        emissive={alive ? color : "#333333"}
        emissiveIntensity={alive ? 0.6 : 0.1}
        roughness={0.3}
        metalness={0.6}
        transparent
        opacity={opacity}
      />
    </mesh>
  );
};

/** Causal context ring */
const CausalContextRing: React.FC<{
  entrance: number;
  highlight: boolean;
}> = ({ entrance, highlight }) => {
  const frame = useCurrentFrame();
  const s = entrance;
  const glowIntensity = highlight ? 0.5 : 0.15;

  return (
    <group>
      {/* Main ring */}
      <mesh rotation={[Math.PI / 2, 0, frame * 0.003]} scale={[s * 1.8, s * 1.8, s * 1.8]}>
        <torusGeometry args={[1, 0.015, 8, 64]} />
        <meshStandardMaterial
          color="#ffc46a"
          emissive="#ffc46a"
          emissiveIntensity={glowIntensity}
          transparent
          opacity={entrance * 0.5}
        />
      </mesh>
      {/* Secondary ring — slightly larger */}
      <mesh rotation={[Math.PI / 2.2, 0.3, frame * 0.002]} scale={[s * 2.0, s * 2.0, s * 2.0]}>
        <torusGeometry args={[1, 0.008, 8, 64]} />
        <meshStandardMaterial
          color="#ffc46a"
          emissive="#ffc46a"
          emissiveIntensity={0.1}
          transparent
          opacity={entrance * 0.2}
        />
      </mesh>
    </group>
  );
};

/** Dot-store cluster boundary */
const DotStoreBoundary: React.FC<{
  entrance: number;
}> = ({ entrance }) => {
  const frame = useCurrentFrame();
  return (
    <mesh rotation={[Math.PI / 2, 0, -frame * 0.005]} scale={[entrance * 0.9, entrance * 0.9, entrance * 0.9]}>
      <torusGeometry args={[1, 0.01, 8, 64]} />
      <meshStandardMaterial
        color="#6eff9e"
        emissive="#6eff9e"
        emissiveIntensity={0.3}
        transparent
        opacity={entrance * 0.35}
      />
    </mesh>
  );
};

export const DotStoreScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();

  // Phase 1: Show dots (all alive)
  // Phase 2: Remove one dot (frame 100-140): dot fades from store but ring keeps it
  // Phase 3: Callout — no tombstone needed

  const mainEnt = spring({ frame, fps, delay: 5, config: { damping: 14 } });
  const ringEnt = spring({ frame, fps, delay: 15, config: { damping: 14 } });
  const storeEnt = spring({ frame, fps, delay: 25, config: { damping: 14 } });

  // Dot removal animation (dot #2 gets removed)
  const removeFade = interpolate(frame, [100, 135], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const ringHighlight = frame >= 95 && frame <= 160;

  // Dots arrangement in cluster
  const dots: { pos: [number, number, number]; color: string; alive: boolean }[] = [
    { pos: [-0.35, 0.25, 0.1], color: "#4a9eff", alive: true },
    { pos: [0.3, 0.3, -0.1], color: "#ff6a9e", alive: true },
    { pos: [-0.05, -0.35, 0.15], color: "#6eff9e", alive: frame < 100 }, // this one gets removed
    { pos: [0.25, -0.20, -0.05], color: "#c9a0ff", alive: true },
    { pos: [-0.40, -0.10, -0.1], color: "#6affea", alive: true },
  ];

  // Text
  const titleOpacity = interpolate(frame, [5, 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const labelDelay = 35;
  const contextLabelOpacity = interpolate(frame, [labelDelay, labelDelay + 15], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const storeLabelOpacity = interpolate(frame, [labelDelay + 15, labelDelay + 30], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // "Remove" indicator
  const removeIndicator = interpolate(frame, [90, 105], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // "No tombstone" callout
  const noTombstone = interpolate(frame, [145, 165], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const ntY = interpolate(spring({ frame, fps, delay: 145, config: { damping: 200 } }), [0, 1], [12, 0]);

  const fadeOut = interpolate(frame, [210, 240], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ backgroundColor: "#1e1e1e", opacity: fadeOut }}>
      <ThreeCanvas linear width={width} height={height}>
        <color attach="background" args={["#1e1e1e"]} />
        <ambientLight intensity={0.4} />
        <pointLight position={[5, 5, 5]} intensity={1} color="#6ea0ff" />
        <pointLight position={[-5, -3, 3]} intensity={0.5} color="#a06eff" />
        <directionalLight position={[0, 3, 4]} intensity={0.3} />

        {/* Causal context (outer ring) */}
        <CausalContextRing entrance={ringEnt} highlight={ringHighlight} />

        {/* Dot store (inner ring boundary) */}
        <DotStoreBoundary entrance={storeEnt} />

        {/* Dots */}
        {dots.map((d, i) => (
          <Dot
            key={i}
            position={d.pos}
            color={d.color}
            size={0.12}
            alive={d.alive}
            entrance={mainEnt}
            fadeOutProgress={i === 2 ? removeFade : 0}
          />
        ))}
      </ThreeCanvas>

      <AbsoluteFill style={{ pointerEvents: "none" }}>
        {/* Title */}
        <div style={{ position: "absolute", top: 35, left: 60, opacity: titleOpacity }}>
          <span style={{ fontFamily: FONT_DISPLAY, fontSize: 22, color: "rgba(255,255,255,0.9)" }}>
            Tombstone-Free Removal
          </span>
        </div>

        {/* Causal context label */}
        <div style={{ position: "absolute", right: "14%", top: "25%", opacity: contextLabelOpacity }}>
          <div style={{
            fontFamily: FONT_PRIMARY,
            fontSize: 14,
            color: "#ffc46a",
            padding: "6px 14px",
            border: "1px solid rgba(255,196,106,0.2)",
            borderRadius: 6,
            background: "rgba(255,196,106,0.04)",
          }}>
            Causal Context <span style={{ fontSize: 11, opacity: 0.5 }}>all dots ever</span>
          </div>
        </div>

        {/* Dot store label */}
        <div style={{ position: "absolute", left: "14%", bottom: "28%", opacity: storeLabelOpacity }}>
          <div style={{
            fontFamily: FONT_PRIMARY,
            fontSize: 14,
            color: "#6eff9e",
            padding: "6px 14px",
            border: "1px solid rgba(110,255,158,0.2)",
            borderRadius: 6,
            background: "rgba(110,255,158,0.04)",
          }}>
            Dot Store <span style={{ fontSize: 11, opacity: 0.5 }}>live data</span>
          </div>
        </div>

        {/* Remove indicator */}
        {frame >= 90 && (
          <div style={{ position: "absolute", left: "47%", top: "62%", opacity: removeIndicator }}>
            <span style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "#ff6a9e" }}>
              remove( )
            </span>
          </div>
        )}

        {/* Explanation */}
        {frame >= 130 && (
          <div style={{ position: "absolute", right: "10%", bottom: "28%", opacity: removeIndicator, maxWidth: 260 }}>
            <p style={{ fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.5)", lineHeight: 1.6, margin: 0 }}>
              Dot in context, absent from store = deleted
            </p>
          </div>
        )}

        {/* "No tombstone" callout */}
        <div style={{
          position: "absolute",
          bottom: 45,
          left: 0,
          right: 0,
          textAlign: "center",
          opacity: noTombstone,
          transform: `translateY(${ntY}px)`,
        }}>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 18, color: "white", margin: 0 }}>
            No tombstones — bounded metadata growth
          </p>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.4)", marginTop: 6 }}>
            Absence in the dot store IS the deletion record
          </p>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
