# cargo-async-doctor

`cargo-async-doctor` is a Cargo subcommand for auditing common async Rust footguns and explaining how to fix them.

The goal is not to replace Clippy, rustc diagnostics, or runtime docs. The goal is to sit one layer above them:

- detect high-signal async misuse patterns
- explain why the pattern is dangerous
- point to a minimal fix
- link to deeper guidance and examples

## Status

Phase 2: First Trustworthy Checks is now implemented.

The repository currently provides:

- a runnable `cargo-async-doctor` binary crate
- a stable CLI argument model for scanning a selected manifest
- a diagnostics model with stable check IDs and shipped explanation text
- three real Phase 2 diagnostics backed by positive and negative fixtures
- separate human-readable and JSON renderers
- structured output tests, unit tests, fixture tests, and CI

What it does **not** provide yet:

- `explain` mode
- full workspace and span fidelity
- the reserved `guard-across-await` check, which stays out of Phase 2 because a syntax-only version would be too noisy

Those remain tracked in [`BUILD.md`](BUILD.md).

## Available Today

Current command surface:

```bash
cargo async-doctor --help
cargo async-doctor --message-format human
cargo async-doctor --message-format json
cargo async-doctor --workspace
cargo async-doctor --manifest-path ./Cargo.toml
```

Current behavior is intentionally narrow: the scan analyzes Rust files under the selected manifest's `src/` tree, emits only shipped Phase 2 diagnostics, and keeps rendering separate from detection.

## Shipped Phase 2 Checks

- `blocking-sleep-in-async`
  Detects direct `std::thread::sleep(...)` calls, plus `thread::sleep(...)` when `thread` is imported from `std::thread`, inside `async fn` bodies or nested `async { ... }` blocks.
- `blocking-std-api-in-async`
  Detects direct blocking `std::fs` calls inside async contexts for this allowlist:
  `canonicalize`, `copy`, `create_dir`, `create_dir_all`, `metadata`, `read`, `read_dir`, `read_link`, `read_to_string`, `remove_dir`, `remove_dir_all`, `remove_file`, `rename`, `symlink_metadata`, and `write`.
  The same calls are also detected as `fs::...` when `fs` is imported from `std::fs`.
- `sync-async-bridge-hazard`
  Detects `Handle::current().block_on(...)` and `Runtime::new().block_on(...)` style Tokio bridges inside async contexts when the receiver is clearly `tokio::runtime::Handle` or `tokio::runtime::Runtime`.

## Current Limits

- Phase 2 scans only the selected manifest's `src/` tree.
- `--workspace` is accepted, but it does not yet expand workspace members or improve package selection.
- Detection is syntax-driven. It does not resolve macros, re-exports, wildcard imports, or local `use` statements inside blocks.
- Messages include the relative file path, but Phase 2 does not yet report spans or line numbers.
- `guard-across-await` remains reserved but intentionally unshipped until the project has stronger type/context handling.

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
- if a check cannot be made trustworthy yet, leave it out

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
- [`fixtures/README.md`](fixtures/README.md) documents the shipped Phase 2 fixture set.

## License

MIT. See [`LICENSE`](LICENSE).
