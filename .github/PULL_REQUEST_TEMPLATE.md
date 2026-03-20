## Summary

Describe what this PR changes and why.

## Problem

What problem does this solve?

## Solution

Explain the approach and key design decisions.

## Scope

- [ ] This PR addresses a single logical change
- [ ] No unrelated refactors are included

## Testing

List commands you ran and outcomes.

```bash
cargo fmt --all --check
cargo clippy --workspace --lib --bins --tests --benches -- -D warnings
cargo test --workspace
```

For wasm-specific changes (if applicable):

```bash
cargo test -p mdcs-wasm
```

## Compatibility / Risk

- [ ] No breaking API changes
- [ ] Breaking API changes (explain below)

If breaking, include migration notes:

## Documentation

- [ ] README updated (if needed)
- [ ] Crate docs/examples updated (if needed)
- [ ] No docs update needed

## Checklist

- [ ] I followed `CONTRIBUTING.md`
- [ ] I added/updated tests for behavior changes
- [ ] CI should pass for this PR
