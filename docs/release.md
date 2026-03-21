# Release Process

This document covers release hardening and publication steps for `cargo-async-doctor`.

The project is intentionally narrow. A release is ready when the shipped checks, wording, docs, and machine-readable output can all be defended.

## Release Checklist

### In-repo hardening

- [ ] `Cargo.toml` metadata is current: description, authors, license, repository, homepage, documentation, keywords, and categories
- [ ] `README.md`, `BUILD.md`, and `CHANGELOG.md` match the actual shipped surface
- [ ] warning wording is reviewed for clarity, scope, and runtime-specific precision
- [ ] every shipped check still has a stable check ID and `explain` coverage
- [ ] scan JSON output still uses `schema_version` and documented field names
- [ ] any JSON-breaking change is paired with a schema version bump and release-note callout
- [ ] fixture-backed false-positive and false-negative coverage still reflects the shipped rules
- [ ] the tool has been run against at least a few real repositories and the results were spot-checked
- [ ] `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` pass
- [ ] `cargo package --allow-dirty --list` looks correct

### Publication steps

These are intentionally separate from in-repo hardening.

- [ ] review the final `CHANGELOG.md` entry for the release
- [ ] create the release commit and tag
- [ ] run `cargo publish --dry-run`
- [ ] publish to crates.io
- [ ] cut the corresponding GitHub release notes

## Versioning Policy

`cargo-async-doctor` follows SemVer, with a stricter project-level policy for public surfaces:

- shipped check IDs are stable once released
- JSON field names are stable within a given `schema_version`
- additive JSON changes may stay on the same `schema_version` when old consumers continue to work
- any JSON-breaking change requires a `schema_version` bump and an explicit changelog note
- warning wording may be clarified in patch releases when the meaning stays the same
- if a warning's meaning or detection scope changes materially, call that out in the changelog even when the check ID stays the same
- new checks should land in minor releases, not patch releases, unless they only fix a clearly broken shipped path

## Changelog And Release Notes Process

Use `CHANGELOG.md` as the source of truth for user-facing change history.

Process:

1. add user-visible changes to the `Unreleased` section as they land
2. group entries under `Added`, `Changed`, `Fixed`, or `Docs` where that keeps the release readable
3. when cutting a release, rename `Unreleased` to the version number and add the release date
4. keep the entries focused on behavior, output, diagnostics, and documentation that users or integrators will notice
5. if JSON or check behavior changed, name it explicitly instead of hiding it under generic wording

## JSON Stability Notes

Current public JSON expectations:

- scan output is versioned by `schema_version`
- scan output includes `target`, `summary`, `diagnostics`, and `notes`
- explain output is versioned separately by its own `schema_version`
- `placeholder` is not part of the public scan report shape and should not return
- location fields are best-effort, but their field names are stable

If the project needs richer machine-readable output later, prefer additive fields first and reserve breaking reshapes for a schema-version increment.
