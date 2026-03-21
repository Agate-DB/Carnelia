#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Publish mdcs-wasm browser artifact (.wasm) to GHCR as OCI.

Usage:
  ./scripts/publish-wasm-oci.sh --owner <ghcr-namespace> [--username <github-username>] [--tag <tag>] [--crate-dir <path>] [--no-build]

Options:
  --owner <owner>        GHCR namespace path for ghcr.io/<owner>/mdcs-wasm (required)
  --username <username>  GitHub username used for GHCR auth (optional)
  --tag <tag>            OCI tag (default: latest)
  --crate-dir <path>     Path to mdcs-wasm crate (default: crates/mdcs-wasm)
  --no-build             Skip wasm-pack build and use existing pkg/mdcs_wasm_bg.wasm
  -h, --help             Show this help

Auth:
  Uses GHCR_TOKEN env var if set; otherwise falls back to GITHUB_TOKEN.
  Username resolution order: --username, GHCR_USER, GITHUB_USER.
  Token must include write:packages.
EOF
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: required command not found: $1" >&2
    exit 1
  fi
}

OWNER=""
USERNAME=""
TAG="latest"
CRATE_DIR="crates/mdcs-wasm"
NO_BUILD="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --owner)
      OWNER="${2:-}"
      shift 2
      ;;
    --tag)
      TAG="${2:-}"
      shift 2
      ;;
    --username)
      USERNAME="${2:-}"
      shift 2
      ;;
    --crate-dir)
      CRATE_DIR="${2:-}"
      shift 2
      ;;
    --no-build)
      NO_BUILD="true"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$OWNER" ]]; then
  echo "error: --owner is required" >&2
  usage
  exit 1
fi

TOKEN="${GHCR_TOKEN:-${GITHUB_TOKEN:-}}"
if [[ -z "$TOKEN" ]]; then
  echo "error: set GHCR_TOKEN or GITHUB_TOKEN with write:packages scope" >&2
  exit 1
fi

if [[ -z "$USERNAME" ]]; then
  USERNAME="${GHCR_USER:-${GITHUB_USER:-}}"
fi

if [[ -z "$USERNAME" ]]; then
  echo "error: GHCR username required; pass --username or set GHCR_USER/GITHUB_USER" >&2
  exit 1
fi

require_cmd wasm-pack
require_cmd wkg

if [[ ! -d "$CRATE_DIR" ]]; then
  echo "error: crate dir not found: $CRATE_DIR" >&2
  exit 1
fi

WASM_PATH="$CRATE_DIR/pkg/mdcs_wasm_bg.wasm"
OCI_REF="ghcr.io/$OWNER/mdcs-wasm:$TAG"
OCI_SOURCE="org.opencontainers.image.source=https://github.com/Agate-DB/Carnelia"

if [[ "$NO_BUILD" != "true" ]]; then
  echo "==> Building mdcs-wasm via wasm-pack"
  (
    cd "$CRATE_DIR"
    wasm-pack build --target web --release --out-dir pkg
  )
fi

if [[ ! -f "$WASM_PATH" ]]; then
  echo "error: wasm artifact not found: $WASM_PATH" >&2
  exit 1
fi

echo "==> Pushing OCI artifact: $OCI_REF"
wkg oci push -u "$USERNAME" -p "$TOKEN" --annotation "$OCI_SOURCE" "$OCI_REF" "$WASM_PATH"

echo "==> Pulling back for verification"
TMP_PULL="$(mktemp -t mdcs-wasm-oci-XXXXXX.wasm)"
wkg oci pull -u "$USERNAME" -p "$TOKEN" "$OCI_REF" -o "$TMP_PULL"

if command -v regctl >/dev/null 2>&1; then
  echo "==> Inspecting manifest with regctl"
  regctl manifest get "$OCI_REF" | sed -n '1,80p'
else
  echo "==> regctl not found, skipping manifest inspection"
fi

echo "==> Done"
echo "OCI reference: $OCI_REF"
echo "Local pulled copy: $TMP_PULL"
