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
 * Scene — Carnelia Collaborative Editing: JSON & Rich Text
 *
 * Two-panel scene comparing:
 *   Left: Traditional approach (Figma/Google Docs) — server-mediated OT
 *   Right: Carnelia approach — peer-to-peer CRDTs
 *
 * Then shows concrete examples:
 *   - JSON collab (3 team members editing project config simultaneously)
 *   - Rich text collab (concurrent insertions auto-merge)
 *
 * References examples/mdcs-sdk/json_collab.rs and collaborative_text.rs
 *
 * AUDIO CUE: collab_demo_narration.mp3
 */

/** Server node — central mediator for traditional approach */
const ServerNode: React.FC<{
  position: [number, number, number];
  entrance: number;
}> = ({ position, entrance }) => {
  const frame = useCurrentFrame();
  const s = entrance * 0.25;
  const pulse = 1 + Math.sin(frame * 0.04) * 0.04;

  return (
    <group position={position}>
      <mesh scale={[s * pulse, s * 1.3 * pulse, s * 0.4 * pulse]} rotation={[0.1, frame * 0.003, 0]}>
        <boxGeometry args={[1, 1, 1]} />
        <meshStandardMaterial
          color="#ff4444"
          roughness={0.3}
          metalness={0.6}
          emissive="#ff4444"
          emissiveIntensity={0.3}
          transparent
          opacity={entrance * 0.7}
        />
      </mesh>
      {/* "Server" status LED */}
      <mesh position={[0, s * 0.8, s * 0.22]} scale={[0.02, 0.02, 0.02]}>
        <sphereGeometry args={[1, 8, 8]} />
        <meshBasicMaterial color="#ff4444" transparent opacity={entrance * (0.5 + Math.sin(frame * 0.1) * 0.3)} />
      </mesh>
    </group>
  );
};

/** P2P node for Carnelia approach */
const P2PNode: React.FC<{
  position: [number, number, number];
  entrance: number;
  color: string;
}> = ({ position, entrance, color }) => {
  const frame = useCurrentFrame();
  const s = entrance * 0.16;

  return (
    <group position={[position[0], position[1] + Math.sin(frame * 0.02 + position[0] * 3) * 0.03 * entrance, position[2]]}>
      <mesh scale={[s, s, s]} rotation={[0.2, frame * 0.006, 0.1]}>
        <dodecahedronGeometry args={[1, 0]} />
        <meshStandardMaterial
          color={color}
          roughness={0.1}
          metalness={0.8}
          emissive={color}
          emissiveIntensity={0.5}
          transparent
          opacity={entrance * 0.85}
        />
      </mesh>
      <mesh rotation={[Math.PI / 2, 0, frame * 0.008]} scale={[s * 1.8, s * 1.8, s * 1.8]}>
        <torusGeometry args={[0.6, 0.008, 8, 32]} />
        <meshBasicMaterial color={color} transparent opacity={entrance * 0.15} />
      </mesh>
    </group>
  );
};

/** Connection spokes from clients to server */
const ServerSpokes: React.FC<{
  serverPos: [number, number, number];
  clientPositions: [number, number, number][];
  entrance: number;
}> = ({ serverPos, clientPositions, entrance }) => {
  return (
    <group>
      {clientPositions.map((cp, i) => {
        const dx = cp[0] - serverPos[0];
        const dy = cp[1] - serverPos[1];
        const len = Math.sqrt(dx * dx + dy * dy);
        return (
          <mesh
            key={i}
            position={[(serverPos[0] + cp[0]) / 2, (serverPos[1] + cp[1]) / 2, 0]}
            rotation={[0, 0, Math.atan2(dy, dx)]}
          >
            <boxGeometry args={[len, 0.008, 0.008]} />
            <meshBasicMaterial color="#ff4444" transparent opacity={entrance * 0.15} />
          </mesh>
        );
      })}
    </group>
  );
};

/** P2P mesh connections */
const P2PMesh: React.FC<{
  positions: [number, number, number][];
  entrance: number;
  color: string;
}> = ({ positions, entrance, color }) => {
  const pairs: [number, number][] = [];
  for (let i = 0; i < positions.length; i++) {
    for (let j = i + 1; j < positions.length; j++) {
      pairs.push([i, j]);
    }
  }
  return (
    <group>
      {pairs.map(([a, b], idx) => {
        const from = positions[a];
        const to = positions[b];
        const dx = to[0] - from[0];
        const dy = to[1] - from[1];
        const len = Math.sqrt(dx * dx + dy * dy);
        return (
          <mesh
            key={idx}
            position={[(from[0] + to[0]) / 2, (from[1] + to[1]) / 2, -0.1]}
            rotation={[0, 0, Math.atan2(dy, dx)]}
          >
            <boxGeometry args={[len, 0.006, 0.006]} />
            <meshBasicMaterial color={color} transparent opacity={entrance * 0.12} />
          </mesh>
        );
      })}
    </group>
  );
};

export const CollabDemoScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();

  // Architecture comparison (frames 0–200)
  const tradEnt = spring({ frame, fps, delay: 10, config: { damping: 14 } });
  const p2pEnt = spring({ frame, fps, delay: 40, config: { damping: 14 } });

  // JSON collab example (frames 180–320)
  const jsonPhase = interpolate(frame, [180, 200], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const jsonStep1 = interpolate(frame, [200, 220], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const jsonStep2 = interpolate(frame, [240, 260], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const jsonStep3 = interpolate(frame, [280, 300], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Contrast badge
  const contrastOpacity = interpolate(frame, [100, 120], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  // Result
  const resultOpacity = interpolate(frame, [340, 360], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const resultY = interpolate(spring({ frame, fps, delay: 340, config: { damping: 200 } }), [0, 1], [12, 0]);

  const titleOpacity = interpolate(frame, [5, 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const fadeOut = interpolate(frame, [430, 460], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  const tradClients: [number, number, number][] = [[-4.0, 1.5, 0], [-4.0, -0.5, 0], [-3.0, -1.5, 0]];
  const p2pNodes: [number, number, number][] = [[2.5, 1.2, 0], [4.0, 0, 0], [2.5, -1.2, 0]];

  return (
    <AbsoluteFill style={{ backgroundColor: "#1e1e1e", opacity: fadeOut }}>
      <ThreeCanvas linear width={width} height={height}>
        <color attach="background" args={["#1e1e1e"]} />
        <ambientLight intensity={0.4} />
        <pointLight position={[5, 5, 5]} intensity={0.8} color="#6ea0ff" />
        <pointLight position={[-5, -3, 3]} intensity={0.5} color="#a06eff" />

        {/* Traditional: Server + 3 clients */}
        <ServerNode position={[-2.5, 0.3, 0]} entrance={tradEnt} />
        <ServerSpokes serverPos={[-2.5, 0.3, 0]} clientPositions={tradClients} entrance={tradEnt} />
        {tradClients.map((pos, i) => (
          <P2PNode key={`t${i}`} position={pos} entrance={tradEnt} color="#b0c8e8" />
        ))}

        {/* Carnelia: P2P mesh */}
        <P2PMesh positions={p2pNodes} entrance={p2pEnt} color="#6eff9e" />
        {p2pNodes.map((pos, i) => (
          <P2PNode key={`p${i}`} position={pos} entrance={p2pEnt} color={["#4a9eff", "#ff6a9e", "#6eff9e"][i]} />
        ))}
      </ThreeCanvas>

      <AbsoluteFill style={{ pointerEvents: "none" }}>
        {/* Title */}
        <div style={{ position: "absolute", top: 35, left: 60, opacity: titleOpacity }}>
          <span style={{ fontFamily: FONT_DISPLAY, fontSize: 26, color: "#e06040" }}>
            Collaborative Editing: Carnelia vs Traditional
          </span>
        </div>

        {/* Traditional label */}
        <div style={{ position: "absolute", left: "5%", top: "14%", opacity: tradEnt, maxWidth: 240 }}>
          <div style={{ fontFamily: FONT_PRIMARY, fontSize: 18, color: "#ff4444", marginBottom: 4 }}>
            Figma / Google Docs
          </div>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.4)", margin: 0, lineHeight: 1.5 }}>
            Server-mediated OT/CRDT<br />
            Single point of failure<br />
            Requires internet connection
          </p>
        </div>

        {/* Carnelia label */}
        <div style={{ position: "absolute", right: "5%", top: "14%", opacity: p2pEnt, maxWidth: 240 }}>
          <div style={{ fontFamily: FONT_PRIMARY, fontSize: 18, color: "#6eff9e", marginBottom: 4 }}>
            Carnelia (MDCS)
          </div>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.4)", margin: 0, lineHeight: 1.5 }}>
            Peer-to-peer δ-CRDTs<br />
            No single point of failure<br />
            Full offline support
          </p>
        </div>

        {/* Contrast badges */}
        <div style={{
          position: "absolute", left: "50%", top: "15%", opacity: contrastOpacity,
          transform: "translateX(-50%)",
        }}>
          <span style={{ fontFamily: FONT_PRIMARY, fontSize: 19, color: "rgba(255,255,255,0.6)" }}>vs</span>
        </div>

        {/* JSON Collab Example */}
        <div style={{
          position: "absolute", left: "50%", top: "50%", transform: "translate(-50%, -50%)",
          opacity: jsonPhase, maxWidth: 500, width: "100%",
        }}>
          <div style={{
            background: "rgba(255,255,255,0.03)", border: "1px solid rgba(255,255,255,0.08)",
            borderRadius: 12, padding: "16px 24px",
          }}>
            <div style={{ fontFamily: FONT_PRIMARY, fontSize: 17, color: "#e06040", marginBottom: 12 }}>
              JSON Collab Demo — 3 concurrent editors
            </div>

            {/* Step 1 */}
            <div style={{ opacity: jsonStep1, marginBottom: 8, transition: "opacity 0.3s" }}>
              <span style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "#4a9eff" }}>ProjectManager</span>
              <span style={{ fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.3)" }}> → </span>
              <span style={{ fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.5)" }}>
                set("name", "Project Alpha"), set("version", "1.0.0")
              </span>
            </div>

            {/* Step 2 */}
            <div style={{ opacity: jsonStep2, marginBottom: 8 }}>
              <span style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "#ff6a9e" }}>Developer</span>
              <span style={{ fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.3)" }}> → </span>
              <span style={{ fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.5)" }}>
                set("tech.language", "Rust"), set("tech.framework", "MDCS")
              </span>
            </div>

            {/* Step 3 */}
            <div style={{ opacity: jsonStep3, marginBottom: 12 }}>
              <span style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "#6eff9e" }}>Designer</span>
              <span style={{ fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.3)" }}> → </span>
              <span style={{ fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.5)" }}>
                set("ui.theme", "dark"), set("ui.primary_color", "#3498db")
              </span>
            </div>

            {/* Result */}
            <div style={{ opacity: jsonStep3, borderTop: "1px solid rgba(255,255,255,0.06)", paddingTop: 10 }}>
              <div style={{ fontFamily: FONT_PRIMARY, fontSize: 12, color: "rgba(255,255,255,0.3)", marginBottom: 6 }}>
                After CRDT merge — all 3 clients identical:
              </div>
              <div style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "#6eff9e", lineHeight: 1.5 }}>
                {"{ name, version, status, tech.*, ui.* }"} — 0 conflicts
              </div>
            </div>
          </div>
        </div>

        {/* Rich Text note */}
        <div style={{
          position: "absolute", left: 50, bottom: 100, opacity: jsonStep3,
          background: "rgba(255,255,255,0.03)", border: "1px solid rgba(255,255,255,0.08)",
          borderRadius: 10, padding: "10px 16px", maxWidth: 320,
        }}>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "rgba(255,255,255,0.5)", margin: 0, marginBottom: 4 }}>Rich Text (RGA)</p>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 13, color: "#c9a0ff", margin: 0, lineHeight: 1.6 }}>
            Concurrent character insertions resolve via<br />
            unique position IDs — no server arbitration
          </p>
        </div>

        {/* Result callout */}
        <div style={{
          position: "absolute", bottom: 35, left: 0, right: 0, textAlign: "center",
          opacity: resultOpacity, transform: `translateY(${resultY}px)`,
        }}>
          <p style={{ fontFamily: FONT_PRIMARY, fontSize: 20, color: "white", margin: 0 }}>
            <span style={{ color: "#ff4444", textDecoration: "line-through", opacity: 0.5 }}>Central server</span>
            {" → "}
            <span style={{ color: "#6eff9e" }}>Peer-to-peer</span>
            {" · "}
            <span style={{ color: "#6affea" }}>Offline-first</span>
            {" · "}
            <span style={{ color: "#c9a0ff" }}>Conflict-free</span>
          </p>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
