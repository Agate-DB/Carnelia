perf report
perf record -F 999 -- cargo run --release -- full
# Capture perf samples from the full stress suite and open report.

set -euo pipefail
