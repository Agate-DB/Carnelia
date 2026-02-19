#!/usr/bin/env bash
set -euo pipefail

# ── Carnelia CRDT Explainer — 4K Render Script ──
# All quality/scale/GL settings are in remotion.config.ts
# Usage:
#   ./render.sh              # Full 4K render

COMP="CrdtExplainer"
OUT_DIR="out"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$OUT_DIR"


echo "🎬 Full 4K render: ${COMP}"
echo "   Output: ${OUT_DIR}/crdt-explainer-4k-${TIMESTAMP}.mp4"
echo "   Settings from remotion.config.ts (4K, CRF 1, PNG, yuv444p)"
echo ""
pnpm exec remotion render "$COMP" "$OUT_DIR/crdt-explainer-4k-${TIMESTAMP}.mp4" \
    --log=verbose

echo ""
echo "✅ Done! Output in ${OUT_DIR}/"
ls -lh "$OUT_DIR"/*.mp4 2>/dev/null | tail -3
