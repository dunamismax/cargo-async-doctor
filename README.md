# cargo-async-doctor

[![CI](https://github.com/dunamismax/cargo-async-doctor/actions/workflows/ci.yml/badge.svg)](https://github.com/dunamismax/cargo-async-doctor/actions/workflows/ci.yml) [![crates.io](https://img.shields.io/crates/v/cargo-async-doctor.svg)](https://crates.io/crates/cargo-async-doctor) [![docs.rs](https://docs.rs/cargo-async-doctor/badge.svg)](https://docs.rs/cargo-async-doctor) [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Spot common async Rust hazards. Get the fix.**

`cargo-async-doctor` is a Cargo subcommand that detects high-signal async mistakes in Rust code, explains why they matter, and points to a practical fix. It is intentionally narrow: a small set of trustworthy checks, not a replacement for rustc, Clippy, or runtime documentation.

> **Status:** v1.0.0 production release. Three shipped checks, human and JSON output, package/workspace targeting, cfg-aware reachability, fixture-backed tests, and CI are in place. See [CHANGELOG.md](CHANGELOG.md) for release history.

## Why cargo-async-doctor?

Async Rust has a class of bugs that compile fine, pass Clippy, and then deadlock or starve your runtime at 2 AM. The standard tooling doesn't catch them because they're semantically valid code doing the wrong thing in an async context. `cargo-async-doctor` exists to catch exactly those.

| Approach | Catches async hazards | Explains the fix | Machine-readable | Zero config |
|----------|:---:|:---:|:---:|:---:|
| `rustc` | No | N/A | Yes | Yes |
| Clippy | Partial | Partial | Yes | Yes |
| Runtime docs | Sometimes | Yes | No | No |
| **cargo-async-doctor** | **Yes** | **Yes** | **Yes** | **Yes** |

## Install

```bash
cargo install cargo-async-doctor
```

## Quick start

```bash
cargo async-doctor                                        # scan current package
cargo async-doctor --workspace                            # scan all workspace members
cargo async-doctor --message-format json                  # machine-readable output
cargo async-doctor explain blocking-sleep-in-async        # explain a specific check
cargo async-doctor --message-format json explain blocking-sleep-in-async
```

## Shipped checks

### `blocking-sleep-in-async`

Detects direct `std::thread::sleep(...)` calls, plus module aliases imported from `std::thread`, inside async functions, async impl methods, nested `async { ... }` blocks, and async closures.

### `blocking-std-api-in-async`

Detects direct blocking `std::fs` calls inside the same async contexts for this allowlist: `canonicalize`, `copy`, `create_dir`, `create_dir_all`, `metadata`, `read`, `read_dir`, `read_link`, `read_to_string`, `remove_dir`, `remove_dir_all`, `remove_file`, `rename`, `symlink_metadata`, and `write`. The same calls are also detected through module aliases imported from `std::fs`.

### `sync-async-bridge-hazard`

Detects `Handle::current().block_on(...)` and `Runtime::new().block_on(...)` style Tokio bridges inside async contexts when the receiver is clearly `tokio::runtime::Handle` or `tokio::runtime::Runtime`, including imported type aliases and simple receiver wrappers such as `unwrap`, `expect`, `clone`, and references.

## Architecture

```text
┌──────────────────────────────────────────────┐
│               cargo-async-doctor CLI          │
│         (clap: scan / explain / flags)        │
└──────┬──────────────┬──────────────┬─────────┘
       │              │              │
  ┌────▼────┐   ┌─────▼─────┐  ┌────▼──────┐
  │  Scan   │   │  Analysis  │  │  Explain   │
  │         │   │            │  │            │
  │ cargo   │   │ syn AST    │  │ check ID → │
  │ metadata│   │ visitor    │  │ content    │
  │ package │   │ detection  │  │            │
  │ resolve │   │ diagnostics│  │            │
  └────┬────┘   └─────┬──────┘  └────┬──────┘
       │              │              │
       └──────────────▼──────────────┘
                 ┌──────────┐
                 │  Render   │
                 │           │
                 │ human /   │
                 │ JSON      │
                 └──────────┘
```

- **Scan** --- resolves package selection through `cargo metadata`, discovers source files, walks target packages
- **Analysis** --- parses Rust source with `syn`, visits AST nodes, detects hazard patterns inside async contexts
- **Explain** --- serves canonical explanation content by stable check ID
- **Render** --- formats diagnostics as human-readable text or structured JSON, separate from detection logic

## What Ships Today

- Scan mode resolves package selection through `cargo metadata`
- Package manifests scan that package unless `--workspace` is set
- Virtual workspace manifests scan default members by default and all members with `--workspace`
- Diagnostics include package context and workspace-relative paths when available
- Line/column data when a direct syntax span is available
- Explain mode serves canonical shipped-check content by stable check ID
- Rendering stays separate from detection and explain-content selection

## Current limits

- Detection is syntax-driven, not semantic
- Does not resolve macros, re-exports, wildcard imports, function imports, or block-local `use` items
- Sync/async bridge detection does not follow stored handles or runtimes
- Location data is best-effort, tied to direct syntax matches
- Explain mode covers shipped checks only

## Development And Verification

From the repository root:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Full smoke test:

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
cargo package --allow-dirty --list
```

## Repository Layout

```text
.
├── BUILD.md              # execution manual -- status, decisions, progress
├── README.md             # public-facing project description
├── CHANGELOG.md          # user-facing release history
├── LICENSE               # MIT
├── Cargo.toml            # published crate metadata and dependencies
├── .github/
│   └── workflows/
│       └── ci.yml        # CI pipeline
├── docs/
│   └── release.md        # release checklist and versioning policy
├── fixtures/
│   └── ...               # positive/negative fixture truth for shipped checks
├── src/
│   ├── main.rs           # binary entry point
│   ├── lib.rs            # library root
│   ├── cli.rs            # clap command definitions
│   ├── scan.rs           # cargo metadata package resolution
│   ├── analysis.rs       # syn AST visitor and hazard detection
│   ├── diagnostics.rs    # diagnostic types and check IDs
│   ├── explain.rs        # explain content by check ID
│   └── render.rs         # human and JSON output rendering
└── tests/
    ├── fixtures.rs       # fixture-backed integration tests
    └── support/
        └── mod.rs        # test helpers
```

## Roadmap

| Phase | Name | Status |
|-------|------|--------|
| 0 | Repo framing and public-surface definition | **Done** |
| 1 | First shipped checks and explain flow | **Done** |
| 2 | Package/workspace targeting fidelity | **Done** |
| 3 | Correctness hardening and structured output stability | **Done** |
| 4 | Future check expansion with strict false-positive control | Not started |
| 5 | Release polish and long-term maintenance discipline | **In progress** |

See [BUILD.md](BUILD.md) for the full phase breakdown with goals, exit criteria, risks, and decisions.

## Design principles

1. **Trust over breadth.** A small set of reliable checks beats a large set of noisy ones. If a check cannot be made trustworthy, do not ship it.
2. **Explain what you find.** Every shipped check has explanation content at least as strong as the detection itself. A warning without context is just noise.
3. **Stable public surfaces.** Shipped check IDs and JSON output schemas are part of the contract. Breaking them costs downstream users.
4. **Syntax-driven but honest.** The tool is explicit about what it can and cannot see. No false claims of semantic analysis.
5. **Conservative scope.** This is a narrow tool on purpose. Scope creep toward a general linter is a bug, not a feature.

## Relationship to `rust-async-field-guide`

`cargo-async-doctor` is the tool companion to [`rust-async-field-guide`](https://github.com/dunamismax/rust-async-field-guide). The guide owns the longer teaching surface. This repo owns fast diagnosis and actionable warnings.

## Contributing

The tool is published and in active use. Contributions are welcome for correctness improvements and false-positive fixes within the current shipped check set. New checks require a strong false-positive story before merging. Open an issue first.

## License

MIT --- see [LICENSE](LICENSE).
