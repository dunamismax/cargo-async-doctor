# cargo-async-doctor

`cargo-async-doctor` will be a Cargo subcommand for auditing common async Rust footguns and explaining how to fix them.

The goal is not to replace Clippy, rustc diagnostics, or runtime docs. The goal is to sit one layer above them:

- detect high-signal async misuse patterns
- explain why the pattern is dangerous
- point to a minimal fix
- link to deeper guidance and examples

Planned focus areas:

- blocking work inside async contexts
- risky sync/async bridging patterns
- cancellation and shutdown hazards
- shared-state and guard-across-`.await` mistakes
- tracing and observability setup gaps
- Tokio-specific runtime footguns where clear guidance exists

## Status

This repository is in Phase 0. The repo, positioning, and build plan exist; the crate implementation does not yet.

## Intended UX

The initial target is a familiar Cargo subcommand interface:

```bash
cargo async-doctor
cargo async-doctor --workspace
cargo async-doctor --message-format json
cargo async-doctor explain blocking-in-async
```

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

## Initial Check Candidates

- `std::thread::sleep` or equivalent blocking calls inside async code
- `std::fs` and other obvious blocking std APIs used directly from async contexts
- suspicious `block_in_place` / `block_on` combinations
- likely guard-held-across-`.await` cases not already covered well enough by current tooling
- missing or weak tracing setup in async binaries where instrumentation is clearly intended

## Project Docs

- [`BUILD.md`](BUILD.md) is the canonical build plan and agent handoff document.

## License

MIT. See [`LICENSE`](LICENSE).
