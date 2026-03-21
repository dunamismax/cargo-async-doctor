# cargo-async-doctor

`cargo-async-doctor` is a Cargo subcommand for auditing common async Rust footguns and explaining how to fix them.

The goal is not to replace Clippy, rustc diagnostics, or runtime docs. The goal is to sit one layer above them:

- detect high-signal async misuse patterns
- explain why the pattern is dangerous
- point to a minimal fix
- link to deeper guidance and examples

## Status

Phase 1: CLI Bootstrap is now implemented.

The repository currently provides:

- a runnable `cargo-async-doctor` binary crate
- a stable CLI argument model for placeholder scans
- a diagnostics model with stable check IDs reserved in code
- separate human-readable and JSON renderers
- baseline unit tests, fixture scaffolding, and CI

What it does **not** provide yet:

- real async misuse detection
- per-check explanation content
- `explain` mode

Those land in Phase 2 and Phase 3, as tracked in [`BUILD.md`](BUILD.md).

## Available Today

Current command surface:

```bash
cargo async-doctor --help
cargo async-doctor --message-format human
cargo async-doctor --message-format json
cargo async-doctor --workspace
cargo async-doctor --manifest-path ./Cargo.toml
```

Current behavior is intentionally small: the scan flow succeeds, emits no diagnostics, and clearly marks itself as a Phase 1 placeholder.

## Planned Focus Areas

The first real checks should stay narrow and trustworthy:

- blocking work inside async contexts
- risky sync/async bridging patterns
- guard-across-`.await` hazards
- Tokio-specific runtime footguns where clear guidance exists

The first release should favor a small number of trustworthy checks over broad but noisy coverage.

## Non-Goals

- replacing `cargo clippy`
- inventing a new async runtime
- guessing about behavior without a documented basis
- shipping dozens of weak heuristics in the first release

## Design Principles

- explanations must be at least as strong as the detection
- every warning should map to a documented fix path
- false-positive control matters more than check count
- runtime-specific guidance should be clearly labeled
- output rendering stays separate from detection logic
- examples and docs should be developed in lockstep with checks

## Relationship To `rust-async-field-guide`

This crate is the tool companion to [`rust-async-field-guide`](https://github.com/dunamismax/rust-async-field-guide).

The guide should own long-form teaching material:

- broken example
- symptom
- root cause
- corrected example
- test

This crate should own fast diagnosis and actionable warnings.

## Local Verification

From the repository root:

```bash
cargo run -- async-doctor --help
cargo run -- --message-format human
cargo run -- --message-format json
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Notes:

- `cargo run -- async-doctor --help` verifies the Cargo-subcommand-style invocation path.
- The two `cargo run` scan commands exercise both output renderers.
- The lint and test commands match the repository CI workflow.

## Project Docs

- [`BUILD.md`](BUILD.md) is the canonical build plan and status tracker.
- [`fixtures/README.md`](fixtures/README.md) documents the initial fixture test scaffolding.

## License

MIT. See [`LICENSE`](LICENSE).
