# cargo-async-doctor

`cargo-async-doctor` is a Cargo subcommand for spotting common async Rust hazards and explaining the fix.

It is intentionally narrow. The goal is not to replace rustc, Clippy, or runtime docs. The goal is to catch a small set of high-signal async mistakes, explain why they matter, and point to a practical fix.

## Install

```bash
cargo install cargo-async-doctor
```

## Status

Current published release: `0.1.2`

What exists today:
- a runnable Cargo subcommand
- human and JSON output
- stable shipped check IDs
- `explain` output for every shipped check
- package/workspace selection through `cargo metadata`
- package-aware diagnostics with workspace-relative paths
- line/column data when a direct syntax span is available
- fixture-backed tests, unit tests, structured-output checks, and CI

What does not exist yet:
- deep name resolution for macros, re-exports, wildcard imports, stored runtime handles, or block-local `use` items
- the reserved `guard-across-await` check
- any claim that every future location will have a source span

## Quick start

```bash
cargo async-doctor --help
cargo async-doctor
cargo async-doctor --workspace
cargo async-doctor --message-format json
cargo async-doctor explain blocking-sleep-in-async
cargo async-doctor --message-format json explain blocking-sleep-in-async
```

## Shipped checks

### `blocking-sleep-in-async`

Detects direct `std::thread::sleep(...)` calls, plus module aliases imported from `std::thread`, inside async functions, async impl methods, nested `async { ... }` blocks, and async closures.

### `blocking-std-api-in-async`

Detects direct blocking `std::fs` calls inside the same async contexts for this allowlist:

`canonicalize`, `copy`, `create_dir`, `create_dir_all`, `metadata`, `read`, `read_dir`, `read_link`, `read_to_string`, `remove_dir`, `remove_dir_all`, `remove_file`, `rename`, `symlink_metadata`, and `write`.

The same calls are also detected through module aliases imported from `std::fs`.

### `sync-async-bridge-hazard`

Detects `Handle::current().block_on(...)` and `Runtime::new().block_on(...)` style Tokio bridges inside async contexts when the receiver is clearly `tokio::runtime::Handle` or `tokio::runtime::Runtime`, including imported type aliases and simple receiver wrappers such as `unwrap`, `expect`, `clone`, and references.

## Current behavior

- scan mode resolves package selection through `cargo metadata`
- package manifests scan that package unless `--workspace` is set
- virtual workspace manifests scan default members by default and all members with `--workspace`
- diagnostics include package context and workspace-relative paths when available
- explain mode serves canonical shipped-check content by stable check ID
- rendering stays separate from detection and explain-content selection

## Current limits

- detection is still syntax-driven
- it does not resolve macros, re-exports, wildcard imports, function imports, or block-local `use` items
- sync/async bridge detection still does not follow stored handles or runtimes
- location data is best-effort and tied to direct syntax matches
- explain mode covers shipped checks only

## Local verification

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

## Repo docs

- [`BUILD.md`](BUILD.md) — repo handoff, current status, release posture, and next moves
- [`CHANGELOG.md`](CHANGELOG.md) — user-facing release history
- [`docs/release.md`](docs/release.md) — release checklist and versioning policy
- [`fixtures/README.md`](fixtures/README.md) — fixture notes

## Relationship to `rust-async-field-guide`

`cargo-async-doctor` is the tool companion to [`rust-async-field-guide`](https://github.com/dunamismax/rust-async-field-guide).

The guide owns the longer teaching surface. This repo owns fast diagnosis and actionable warnings.

## License

MIT. See [`LICENSE`](LICENSE).
