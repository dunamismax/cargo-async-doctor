# Changelog

All notable user-facing changes to `cargo-async-doctor` should be recorded here.

This project follows a lightweight Keep a Changelog style. Add entries to `Unreleased` as work lands, then promote that section into a dated versioned release when publishing.

## [Unreleased]

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
