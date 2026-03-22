# cargo-async-doctor Build Manual

Last updated: 2026-03-22
Status: released narrow-scope tool, currently in maintenance / correctness hardening mode
Current published version: `0.1.2`

## Purpose

This file is the operational handoff for `cargo-async-doctor`.

Use it to answer four things quickly:
- what the tool actually ships today
- which public surfaces must stay stable
- which checks are real versus reserved
- what the next safe improvement path is

## Mission

Build and maintain a small, trustworthy Cargo subcommand for diagnosing common async Rust hazards.

The emphasis is on:
- high-signal checks
- clear explanations
- stable check IDs and machine-readable output
- conservative scope
- low false-positive pressure

## Current repo truth

What exists today:
- published crate `cargo-async-doctor` `0.1.2`
- human and JSON scan output
- human and JSON `explain` output
- stable shipped check IDs
- package/workspace selection through `cargo metadata`
- package-aware diagnostics with workspace-relative paths where possible
- line/column data when a direct syntax span exists
- fixture-backed tests, unit tests, structured-output checks, and CI
- release notes and versioning policy documented in-repo

What does not exist today:
- deep semantic resolution for macros, re-exports, wildcard imports, stored runtime handles, or block-local `use` items
- the reserved `guard-across-await` check
- any promise that every future diagnostic can carry exact line/column information
- a broad lint surface; this repo is intentionally narrow

## Shipped checks

- `blocking-sleep-in-async`
- `blocking-std-api-in-async`
- `sync-async-bridge-hazard`

Anything else should be treated as future work until it is implemented, tested, documented, and released.

## Source of truth by concern

- `README.md`
  - public framing
  - install / quick start
  - current limitations
- `BUILD.md`
  - current status
  - release posture
  - next moves
  - public-surface guardrails
- `Cargo.toml`
  - published crate metadata
  - dependency surface
- `src/`
  - detection logic, CLI, rendering, and explain content wiring
- `fixtures/`
  - positive / negative fixture truth for shipped checks
- `CHANGELOG.md`
  - user-facing release history
- `docs/release.md`
  - release checklist and versioning policy

## Architecture and operating model

Current shape is intentionally simple:
- one published binary crate
- syntax-driven analysis
- separate scan and explain flows
- separate renderers for human and JSON output
- fixture-driven regression protection around shipped checks

Boundary rules worth protecting:
- stable shipped check IDs matter more than check count
- explanation quality must be at least as strong as detection quality
- JSON stability matters once a surface is released
- if a check cannot be made trustworthy, do not ship it
- package/workspace targeting fidelity is part of tool correctness, not a side concern

## Verification and quality gates

### Minimum local gate

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

### Useful command smoke set

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

## Phase dashboard

- Phase 0 - repo framing and public-surface definition: **done**
- Phase 1 - first shipped checks and explain flow: **done**
- Phase 2 - package/workspace targeting fidelity: **done**
- Phase 3 - correctness hardening and structured output stability: **in progress**
- Phase 4 - future check expansion with strict false-positive control: **not started**
- Phase 5 - release polish and long-term maintenance discipline: **in progress**

## Active focus

### Phase 3 - correctness hardening and structured output stability

Keep tightening the current shipped surface without widening it recklessly.

Current high-value work:
- package/workspace targeting correctness
- nested module and `#[cfg(...)]` reachability fidelity
- warning wording precision
- JSON stability discipline
- explain coverage staying aligned with shipped behavior

### Phase 5 - maintenance discipline

Keep the repo easy to defend as a small stable tool:
- changelog stays current
- release process stays explicit
- packaging/docs.rs/crates.io sanity stays cheap to verify
- docs stay aligned with the real shipped surface

## Open decisions and risks

### Open decisions

- whether `guard-across-await` ever becomes shippable with acceptable noise
- how much deeper name resolution is worth adding before the tool stops being intentionally small
- whether future checks should remain Tokio-shaped only or broaden to more runtime-agnostic patterns

### Risks

- adding weak checks would damage trust faster than it increases usefulness
- public JSON drift without explicit schema discipline would be expensive for downstream users
- README/BUILD drift can overstate what the tool actually does
- package selection correctness bugs can quietly damage confidence even when individual checks are fine

## Immediate next moves

- keep README, BUILD, and release docs aligned with `0.1.2` behavior
- continue correctness hardening inside the current shipped check set
- only add new checks when the semantics and false-positive story are strong enough to defend
- keep `CHANGELOG.md` current as fixes land

## Progress log

### 2026-03-22
- restored `README.md` as a real repo overview after the docs-tightening regression
- reintroduced `BUILD.md` as the operational handoff / status document
- kept the current published `0.1.2` surface and release docs as the source of truth

### 2026-03-22
- `0.1.2` released with package-target reachability and nested inline-module fixes

### 2026-03-21
- `0.1.1` released
- initial public release sequence and release-process docs established

## Decision log

- stable check IDs matter more than shipping more checks quickly
- syntax-driven but trustworthy beats ambitious but noisy
- `guard-across-await` stays reserved until the repo can defend it
- release/process docs are part of the product for a cargo plugin, not optional extra prose
