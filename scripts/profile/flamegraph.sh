#!/usr/bin/env bash
set -euo pipefail

# Build and profile the full stress suite using cargo-flamegraph.
# Requires: cargo install flamegraph (Linux/WSL).
cargo flamegraph -p carnelia --bin carnelia --release -- full

