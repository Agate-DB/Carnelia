#!/usr/bin/env bash
set -euo pipefail

# ── Carnelia CRDT Explainer — Chunked 4K Render Script ──
# Renders in sequential 1000-frame chunks, then concatenates with ffmpeg.
# This avoids R3F/memory exhaustion on long renders.
#
# Usage:
#   ./render.sh                    # Full chunked 4K render
#   CHUNK_SIZE=500 ./render.sh     # Override chunk size

COMP="CrdtExplainer"
OUT_DIR="out"
CHUNKS_DIR="out/chunks"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
CHUNK_SIZE="${CHUNK_SIZE:-1000}"
FINAL_OUTPUT="${OUT_DIR}/crdt-explainer-4k-${TIMESTAMP}.mp4"

mkdir -p "$OUT_DIR" "$CHUNKS_DIR"

# ── Detect total frames ───────────────────────────────────
echo "🔍 Querying composition duration..."
TOTAL_FRAMES=$(pnpm exec remotion compositions \
  --json 2>/dev/null \
  | grep -A4 "\"id\":\"${COMP}\"" \
  | grep '"durationInFrames"' \
  | grep -oP '\d+' \
  | head -1 \
  || echo "")

if [[ -z "$TOTAL_FRAMES" ]]; then
  # Fallback: sum of SCENE_DURATIONS from CrdtExplainer.tsx
  TOTAL_FRAMES=8500
  echo "⚠️  Could not query compositions, using fallback: ${TOTAL_FRAMES} frames"
else
  echo "   Total frames: ${TOTAL_FRAMES}"
fi

# ── Chunk render loop ─────────────────────────────────────
CHUNK_FILES=()
START=1000

echo ""
echo "🎬 Chunked 4K render: ${COMP}"
echo "   Chunk size : ${CHUNK_SIZE} frames"
echo "   Total      : ${TOTAL_FRAMES} frames"
echo "   Final out  : ${FINAL_OUTPUT}"
echo ""

while [[ $START -lt $TOTAL_FRAMES ]]; do
  END=$(( START + CHUNK_SIZE - 1 ))
  if [[ $END -ge $TOTAL_FRAMES ]]; then
    END=$(( TOTAL_FRAMES - 1 ))
  fi

  CHUNK_NUM=$(( START / CHUNK_SIZE + 1 ))
  CHUNK_FILE="${CHUNKS_DIR}/chunk-$(printf '%04d' $CHUNK_NUM).mp4"
  CHUNK_FILES+=("$CHUNK_FILE")

  echo "▶  Chunk ${CHUNK_NUM}: frames ${START}-${END} → ${CHUNK_FILE}"

  pnpm exec remotion render "$COMP" "$CHUNK_FILE" \
    --frames="${START}-${END}" \
    --chromium-flags="--timeout=60" \
    --timeout=120000 \
    --log=verbose

  echo "✅ Chunk ${CHUNK_NUM} done"
  echo ""

  START=$(( END + 1 ))
done

# ── Concatenate with ffmpeg ───────────────────────────────
echo "🔗 Concatenating ${#CHUNK_FILES[@]} chunks..."
CONCAT_LIST="${CHUNKS_DIR}/concat-${TIMESTAMP}.txt"

for f in "${CHUNK_FILES[@]}"; do
  echo "file '$(realpath "$f")'" >> "$CONCAT_LIST"
done

ffmpeg -y -f concat -safe 0 -i "$CONCAT_LIST" -c copy "$FINAL_OUTPUT"

echo ""
echo "✅ Done! Final output: ${FINAL_OUTPUT}"
ls -lh "$FINAL_OUTPUT"

# ── Cleanup chunks ────────────────────────────────────────
read -rp "🗑  Delete chunk files? [y/N] " CLEAN
if [[ "${CLEAN,,}" == "y" ]]; then
  rm -rf "$CHUNKS_DIR"
  echo "   Chunks deleted."
fi
