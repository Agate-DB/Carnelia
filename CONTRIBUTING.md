# Contributing to Carnelia

Thanks for contributing to Carnelia.

This guide is inspired by the practical style of Redis contributing guidelines: discuss changes early, keep patches focused, include tests, and make review easy.

## Ground Rules

- Be respectful and constructive. See CODE_OF_CONDUCT.md.
- Keep changes small and focused on one problem.
- Prefer incremental pull requests over large rewrites.
- Fix root causes, not only symptoms.
- Update docs when behavior or APIs change.

## Before You Start

For non-trivial changes, open an issue first and describe:

- The problem statement
- Why the current behavior is insufficient
- A proposed solution
- Trade-offs and alternatives considered

This avoids duplicate work and helps maintain architectural consistency across `mdcs-core`, `mdcs-delta`, `mdcs-merkle`, `mdcs-compaction`, `mdcs-db`, `mdcs-sdk`, and `mdcs-wasm`.

## Good First Issues

If you are new to the project, good starter contributions include:

- Documentation clarifications and example fixes
- Small test improvements for existing modules
- Bug fixes with clear reproduction steps
- WASM binding parity improvements between Rust APIs and `mdcs-wasm`

When picking an issue, prefer:

- Small and well-scoped tasks
- Low coupling across crates
- A clear definition of done (tests + docs)

## Development Setup

1. Install stable Rust toolchain.
2. Clone the repository.
3. Run:

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --lib --bins --tests --benches -- -D warnings
```

For wasm changes:

```bash
cargo install wasm-pack
cargo test -p mdcs-wasm
```

## Coding Guidelines

- Follow idiomatic Rust and existing crate/module patterns.
- Keep public APIs stable unless a breaking change is explicitly intended.
- Avoid unrelated refactors in the same PR.
- Do not introduce unnecessary dependencies.
- Prefer clear naming over clever implementations.

## Testing Expectations

Every behavior change should include tests.

- Bug fix: add a regression test.
- New feature: add unit/integration coverage.
- Performance-sensitive change: include benchmark notes/results when relevant.

At minimum, contributors should run:

```bash
cargo fmt --all --check
cargo clippy --workspace --lib --bins --tests --benches -- -D warnings
cargo test --workspace
```

## Commit and PR Guidelines

### Commits

- Keep commits logically scoped.
- Use clear messages describing what changed and why.
- Avoid "WIP" or vague commit titles in final history.

### Pull Requests

Use the PR template at `.github/PULL_REQUEST_TEMPLATE.md`.

Each PR should include:

- Problem summary
- Proposed solution
- Testing performed (commands + outcomes)
- Any API, behavior, or performance impact
- Docs updates (if applicable)

PRs that are easier to review are merged faster:

- One logical change per PR
- Minimal diff noise
- Clear migration notes for breaking changes

## Documentation

If you change behavior, update relevant docs/examples:

- `README.md`
- crate-level READMEs under `crates/*`
- examples under `examples/*`
- performance docs under `docs/*` when performance characteristics change

## Security

Do not open public issues for critical vulnerabilities.

Follow SECURITY.md and report sensitive issues privately to the maintainer email listed there.

## Review and Merge Process

- Maintainers may request design, testing, or scope adjustments.
- PRs must pass CI (`fmt`, `clippy`, `test`).
- Maintainers may ask to split large PRs before review.

Thanks again for helping improve Carnelia.
