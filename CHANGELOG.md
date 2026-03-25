# Changelog

All notable user-facing changes to `cargo-async-doctor` should be recorded here.

This project follows a lightweight Keep a Changelog style. Add entries to `Unreleased` as work lands, then promote that section into a dated versioned release when publishing.

## [Unreleased]

## [1.0.0] - 2026-03-25

### Changed

- Promoted to v1.0.0 production release
- Updated status and documentation to reflect stable release

## [0.1.3] - 2026-03-24

### Fixed

- scan reachability now respects active `#[cfg(...)]` items and modules, using the current target cfgs plus each package's default features so disabled code stays quiet
- nested inline `#[path = ...]` modules now resolve the same source file rustc would load, avoiding false negatives when decoy sibling files exist

## [0.1.2] - 2026-03-22

### Fixed

- scan package reachability now follows Cargo target roots and reachable module trees, so explicit target paths outside `src/` are included and stray uncompiled Rust files under `src/` stay quiet
- nested inline modules no longer inherit parent `use` aliases for `std::thread`, `std::fs`, or Tokio runtime types, which removes false positives from outer-scope alias leakage

## [0.1.1] - 2026-03-21

### Docs

- finalized the post-release repository docs and removed the stale `BUILD.md` phase tracker

## [0.1.0] - 2026-03-21

### Added

- first public release of `cargo-async-doctor`
- human-readable and JSON scan output plus `explain` mode for stable check IDs
- three shipped checks: `blocking-sleep-in-async`, `blocking-std-api-in-async`, and `sync-async-bridge-hazard`
- Cargo metadata-driven package selection for package manifests, root-package workspaces, and virtual workspace manifests
- package-aware diagnostics with workspace-relative file paths plus optional line and column reporting when a syntax span is available

### Changed

- tightened warning wording so the current diagnostics are more precise about blocking behavior and Tokio-specific fix paths
- simplified the scan JSON contract by dropping the old internal `placeholder` field before the first public release

### Docs

- documented the release checklist, versioning policy, and changelog process in `docs/release.md`
- clarified release readiness and JSON stability expectations in the repository docs
