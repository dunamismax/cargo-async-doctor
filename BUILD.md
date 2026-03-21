# BUILD

## Purpose

`cargo-async-doctor` should become a high-signal async Rust audit tool:

- narrow enough to be trusted
- practical enough to use in real repos
- educational enough to improve user judgment, not just surface warnings

## Current Phase

Phase 0: repo setup and planning

What exists:

- repo created
- project positioning chosen
- README and build plan in place

What does not exist yet:

- Cargo workspace
- CLI
- check engine
- fixtures
- CI
- published crate

## Product Shape

Expected near-term shape:

```text
cargo-async-doctor/
├── Cargo.toml
├── README.md
├── BUILD.md
├── LICENSE
├── src/
├── tests/
├── fixtures/
└── docs/
```

Likely implementation split after bootstrap:

- binary crate for CLI entrypoint
- internal analysis modules for syntax/config/project scanning
- diagnostic model for human and JSON output
- fixture-backed regression tests

## Canonical Scope

Build the tool around a small set of checks that satisfy all of these:

1. The pattern is common enough to matter.
2. The warning can be defended from official docs or established ecosystem guidance.
3. The fix can be explained clearly.
4. The false-positive rate can be kept acceptable.

If a proposed check fails one of those tests, it should not ship yet.

## Phase Plan

### Phase 1: CLI Skeleton

Goal:

- create the Cargo subcommand
- support `--help`
- support plain text and JSON output
- establish fixture-driven testing

Definition of done:

- `cargo run -- --help` works
- output format model exists
- test harness exists
- CI runs fmt, clippy, tests

### Phase 2: First Trustworthy Checks

Target checks:

- blocking sleep in async contexts
- obvious blocking std APIs in async functions
- dangerous sync/async bridge patterns with documented support

Definition of done:

- each check has fixtures
- each check has explanation text
- each check links to deeper guidance
- README documents exact MVP scope

### Phase 3: Explain Mode

Goal:

- add `cargo async-doctor explain <check-id>`
- print a compact explainer plus suggested fix direction

Definition of done:

- stable check IDs exist
- human-readable explain output exists
- JSON schema includes machine-readable IDs

### Phase 4: Workspace Readiness

Goal:

- support real multi-crate workspaces
- improve path reporting
- document performance expectations and limitations

Definition of done:

- workspace fixtures exist
- output names the affected package/file/span where feasible
- false positives are reviewed against real repos

## Agent Rules

Agents working in this repo should follow these rules:

- keep `BUILD.md` current when phases or priorities change
- prefer fewer checks with stronger explanations
- do not add heuristics without fixture coverage
- do not claim a warning is authoritative unless the repo can justify it with docs or tests
- keep human output concise and machine output stable
- avoid silently broadening scope into a general lint runner

## Research Inputs

Primary source categories:

- official Rust documentation
- official Tokio documentation
- rust-lang issue trackers where behavior or diagnostics are discussed
- tokio-rs issue trackers where async footguns are documented
- existing Clippy lint behavior and configuration

Prefer primary sources over blogs for normative guidance.

## Quality Gates

Before the first public release, the repo should have:

- clear crate description and scope boundaries
- fixture-backed regression tests
- structured diagnostics model
- CI for fmt, clippy, and tests
- explicit non-goals in README
- release checklist

## Progress Log

### 2026-03-21

- Phase 0 setup completed
- repo name confirmed
- README, LICENSE, and BUILD plan added
- dual-push git remote setup planned to match other repos
