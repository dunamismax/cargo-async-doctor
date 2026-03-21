# cargo-async-doctor

`cargo-async-doctor` is a Cargo subcommand for auditing common async Rust footguns and explaining how to fix them.

The goal is not to replace Clippy, rustc diagnostics, or runtime docs. The goal is to sit one layer above them:

- detect high-signal async misuse patterns
- explain why the pattern is dangerous
- point to a minimal fix
- link to deeper guidance and examples

## Status

Phase 3: Explain Mode is now implemented.

The repository currently provides:

- a runnable `cargo-async-doctor` binary crate
- a stable CLI argument model for scanning a selected manifest or explaining a shipped check ID
- a diagnostics model with stable check IDs
- canonical explain content for every shipped Phase 2 check
- three real Phase 2 diagnostics backed by positive and negative fixtures
- separate human-readable and JSON renderers for both scan and explain output
- structured output tests, unit tests, fixture tests, and CI

What it does **not** provide yet:

- full workspace and span fidelity
- the reserved `guard-across-await` check, which stays out of the shipped surface because a syntax-only version would be too noisy

Those remain tracked in [`BUILD.md`](BUILD.md).

## Available Today

Current command surface:

```bash
cargo async-doctor --help
cargo async-doctor --message-format human
cargo async-doctor --message-format json
cargo async-doctor --workspace
cargo async-doctor --manifest-path ./Cargo.toml
cargo async-doctor explain blocking-sleep-in-async
cargo async-doctor --message-format json explain blocking-sleep-in-async
```

Current behavior is intentionally narrow:

- scan mode analyzes Rust files under the selected manifest's `src/` tree and emits only shipped diagnostics
- explain mode serves canonical shipped-check content by stable check ID
- rendering stays separate from both detection and explain-content selection

## Shipped Phase 2 Checks

- `blocking-sleep-in-async`
  Detects direct `std::thread::sleep(...)` calls, plus module aliases imported from `std::thread`, inside `async fn` bodies, async impl methods, nested `async { ... }` blocks, and async closures.
- `blocking-std-api-in-async`
  Detects direct blocking `std::fs` calls inside the same async contexts for this allowlist:
  `canonicalize`, `copy`, `create_dir`, `create_dir_all`, `metadata`, `read`, `read_dir`, `read_link`, `read_to_string`, `remove_dir`, `remove_dir_all`, `remove_file`, `rename`, `symlink_metadata`, and `write`.
  The same calls are also detected through module aliases imported from `std::fs`.
- `sync-async-bridge-hazard`
  Detects `Handle::current().block_on(...)` and `Runtime::new().block_on(...)` style Tokio bridges inside async contexts when the receiver is clearly `tokio::runtime::Handle` or `tokio::runtime::Runtime`, including imported type aliases and simple receiver wrappers such as `unwrap`, `expect`, `clone`, and references.

## Explain Output Format

For a known shipped check ID, `cargo async-doctor explain <check-id>` returns the same canonical sections in both human and JSON renderers:

- `title`
- `summary`
- `detects`
- `does_not_detect`
- `suggested_fixes`
- `references`

Unknown IDs return `found: false`, `error: "unknown-check-id"`, and the list of known shipped check IDs so tooling can recover without scraping human text.

## Current Limits

- Scan mode still analyzes only the selected manifest's `src/` tree.
- `--workspace` is accepted, but it does not yet expand workspace members or improve package selection.
- Detection is syntax-driven. It does not resolve macros, re-exports, wildcard imports, or local `use` statements inside blocks.
- Explain mode covers the shipped Phase 2 checks only; `guard-across-await` remains reserved and intentionally unshipped until the project has stronger type/context handling.
- Messages include the relative file path, but the project does not yet report spans or line numbers.

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
cargo run -- explain blocking-sleep-in-async
cargo run -- --message-format json explain blocking-sleep-in-async
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Notes:

- `cargo run -- async-doctor --help` verifies the Cargo-subcommand-style invocation path.
- The two scan commands exercise both scan renderers.
- The two explain commands exercise the human and JSON explain renderers.
- The lint and test commands match the repository CI workflow.

## Project Docs

- [`BUILD.md`](BUILD.md) is the canonical build plan and status tracker.
- [`fixtures/README.md`](fixtures/README.md) documents the shipped Phase 2 fixture set.

## License

MIT. See [`LICENSE`](LICENSE).
