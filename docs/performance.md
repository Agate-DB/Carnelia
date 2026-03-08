# Performance Tooling Guide

This project uses four tools for optimization loops:

- `rustfmt` for deterministic formatting
- `clippy` for lint-driven cleanup
- `criterion` for repeatable micro/meso benchmarks
- `perf` + flamegraph for hotspot profiling

## Quick commands

```bash
cargo fmt --all --check
cargo clippy --workspace --lib --bins --tests --benches -- -D warnings
cargo bench -p carnelia --bench stress_core
cargo bench -p carnelia --bench stress_db
cargo run --release -- full
```

## Profiling full stress suite (Linux/WSL)

```bash
./scripts/profile/perf.sh
./scripts/profile/flamegraph.sh
```


## Suggested optimization loop

1. Run `cargo run --release -- full` and note slow phases.
2. Run Criterion benches and compare regressions before/after changes.
3. Run `perf` and flamegraph, optimize dominant call paths only.
4. Re-run convergence/stress checks to avoid semantic regressions.
5. Keep `fmt` + `clippy` green before commit.
