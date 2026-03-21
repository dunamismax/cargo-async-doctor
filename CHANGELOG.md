# Changelog

All notable user-facing changes to `cargo-async-doctor` should be recorded here.

This project follows a lightweight Keep a Changelog style. Add entries to `Unreleased` as work lands, then promote that section into a dated versioned release when publishing.

## [Unreleased]

### Changed

- tightened warning wording so the current diagnostics are more precise about blocking behavior and Tokio-specific fix paths
- simplified the scan JSON contract by dropping the old internal `placeholder` field before the first public release
- documented the release checklist, versioning policy, and changelog process in `docs/release.md`

### Docs

- clarified release readiness and JSON stability expectations in the repository docs
- recorded that in-repo Phase 5 hardening can complete before the external `0.1.0` publish step
