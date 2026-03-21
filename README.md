# cargo-async-doctor

`cargo-async-doctor` is a Cargo subcommand for auditing common async Rust footguns and explaining how to fix them.

The goal is not to replace Clippy, rustc diagnostics, or runtime docs. The goal is to sit one layer above them:

- detect high-signal async misuse patterns
- explain why the pattern is dangerous
- point to a minimal fix
- link to deeper guidance and examples

## Package

- crates.io: [`cargo-async-doctor`](https://crates.io/crates/cargo-async-doctor)
- docs.rs: [`cargo-async-doctor`](https://docs.rs/cargo-async-doctor)
- install: `cargo install cargo-async-doctor`

## Status

`cargo-async-doctor` `0.1.0` is published on crates.io. The shipped surface is intentionally narrow, stable, and ready for real use.

The repository currently provides:

- a runnable `cargo-async-doctor` binary crate
- a stable CLI argument model for scanning a selected manifest or explaining a shipped check ID
- a diagnostics model with stable check IDs
- canonical explain content for every shipped check
- three real diagnostics backed by positive and negative fixtures
- Cargo metadata-driven package selection for package manifests, root-package workspaces, and virtual workspace manifests
- separate human-readable and JSON renderers for both scan and explain output
- package-aware diagnostics with workspace-relative file paths plus optional line and column reporting when a syntax span is available
- a documented release checklist, versioning policy, and changelog process for future releases
- structured output tests, unit tests, fixture tests, and CI

What it does **not** provide yet:

- the reserved `guard-across-await` check, which stays out of the shipped surface because a syntax-only version would be too noisy
- deep name resolution for macros, re-exports, wildcard imports, stored runtime handles, or block-local `use` items
- guarantees that every future location will have a span; line and column data are best-effort and tied to direct syntax matches

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

- scan mode resolves package selection through `cargo metadata`
- package manifests scan that package only unless `--workspace` is set
- virtual workspace manifests scan default members by default and all workspace members with `--workspace`
- diagnostics include package context plus workspace-relative file paths when available
- line and column fields are included when the syntax pass can attach a direct source span
- explain mode serves canonical shipped-check content by stable check ID
- rendering stays separate from both detection and explain-content selection

## Shipped Checks in v0.1.0

- `blocking-sleep-in-async`
  Detects direct `std::thread::sleep(...)` calls, plus module aliases imported from `std::thread`, inside `async fn` bodies, async impl methods, nested `async { ... }` blocks, and async closures.
- `blocking-std-api-in-async`
  Detects direct blocking `std::fs` calls inside the same async contexts for this allowlist:
  `canonicalize`, `copy`, `create_dir`, `create_dir_all`, `metadata`, `read`, `read_dir`, `read_link`, `read_to_string`, `remove_dir`, `remove_dir_all`, `remove_file`, `rename`, `symlink_metadata`, and `write`.
  The same calls are also detected through module aliases imported from `std::fs`.
- `sync-async-bridge-hazard`
  Detects `Handle::current().block_on(...)` and `Runtime::new().block_on(...)` style Tokio bridges inside async contexts when the receiver is clearly `tokio::runtime::Handle` or `tokio::runtime::Runtime`, including imported type aliases and simple receiver wrappers such as `unwrap`, `expect`, `clone`, and references.

## Scan Output Shape

Scan output is versioned by `schema_version` and currently carries:

- stable check IDs
- package name and package manifest path
- a display file path, relative to the workspace root when possible
- a package-relative path
- optional line and column fields
- human-readable messages and help text
- top-level `target`, `summary`, `diagnostics`, and `notes` fields in JSON mode

The human renderer also lists the selected packages so workspace scans stay understandable.

The old internal `placeholder` field is not part of the public JSON contract and is intentionally absent from the release-ready report shape.

## Explain Output Format

For a known shipped check ID, `cargo async-doctor explain <check-id>` returns the same canonical sections in both human and JSON renderers:

- `title`
- `summary`
- `detects`
- `does_not_detect`
- `suggested_fixes`
- `references`

Unknown IDs return `found: false`, `error: "unknown-check-id"`, and the list of known shipped check IDs so tooling can recover without scraping human text.

## Versioning And Compatibility

The crate follows SemVer, with a stricter compatibility policy for the public machine-readable surface:

- shipped check IDs stay stable once released
- JSON field names stay stable within a given `schema_version`
- additive JSON changes should preserve old consumers
- JSON-breaking changes require a `schema_version` bump and an explicit changelog note
- materially different warning behavior should be called out in release notes even when the check ID does not change

The detailed release checklist and changelog process live in [`docs/release.md`](docs/release.md).

## Current Limits

- Detection is still syntax-driven. It does not resolve macros, re-exports, wildcard imports, local `use` statements inside blocks, or function imports such as `use std::thread::sleep; sleep(...)` and `use std::fs::read_to_string; read_to_string(...)`.
- The shipped checks still match only the narrow documented `0.1.0` patterns; the workspace and path improvements broaden targeting fidelity, not lint surface area.
- Sync/async bridge detection still does not follow stored handles or runtimes such as `let handle = Handle::current(); handle.block_on(...)`.
- Location data is best-effort: line and column fields are present for direct syntax matches, but not for future patterns that may need deeper analysis.
- Explain mode covers the shipped `0.1.0` checks only; `guard-across-await` remains reserved and intentionally unshipped until the project has stronger type/context handling.

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
cargo run -- --manifest-path fixtures/phase4/workspace-root-package/Cargo.toml
cargo run -- --workspace --manifest-path fixtures/phase4/workspace-root-package/Cargo.toml
cargo run -- --manifest-path fixtures/phase4/workspace-root-package/member-bin/Cargo.toml
cargo run -- --manifest-path fixtures/phase4/virtual-workspace/Cargo.toml
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo package --allow-dirty --list
```

Notes:

- `cargo run -- async-doctor --help` verifies the Cargo-subcommand-style invocation path.
- The scan commands exercise both the default manifest selection and the Phase 4 workspace/package selection paths.
- The explain commands exercise the human and JSON explain renderers.
- The lint and test commands match the repository CI workflow.
- `cargo package --allow-dirty --list` is a local packaging sanity check before publishing a follow-on release.

## Project Docs

- [`CHANGELOG.md`](CHANGELOG.md) tracks user-facing release history.
- [`docs/release.md`](docs/release.md) documents the release checklist, versioning policy, publication flow, and registry/docs.rs constraints.
- [`fixtures/README.md`](fixtures/README.md) documents the fixture set.

## License

MIT. See [`LICENSE`](LICENSE).
