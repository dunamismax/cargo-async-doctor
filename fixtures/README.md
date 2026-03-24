# Fixtures

**Small Cargo projects used to keep `cargo-async-doctor` checks honest.**

This directory holds focused fixture projects that exercise shipped diagnostics, workspace-selection behavior, reachability rules, and false-positive control.

## Phase Coverage

Phase 2 ships focused fixtures for the first trustworthy checks:

- `placeholder/minimal-bin` is a tiny fixture project that proves manifest-path handling and fixture discovery work.
- `phase2/blocking-sleep-positive` and `phase2/blocking-sleep-negative` cover blocking sleep detection and false-positive control.
- `phase2/blocking-std-api-positive` and `phase2/blocking-std-api-negative` cover the narrow blocking `std::fs` allowlist and lookalike filtering.
- `phase2/sync-async-bridge-positive` and `phase2/sync-async-bridge-negative` cover clearly identifiable Tokio `block_on` bridge hazards and local lookalikes.

Phase 4 adds workspace-focused fixtures:

- `phase4/workspace-root-package` is a multi-crate workspace whose root is also a package. It covers root-package scans, workspace-member scans, and `--workspace` expansion across the root package plus members.
- `phase4/virtual-workspace` is a virtual workspace with explicit `default-members`. It proves that scanning the workspace manifest without `--workspace` stays on default members, while the same manifest can expand to all members when `--workspace` is used.

Phase 5 adds target-reachability and alias-scope regressions:

- `phase5/uncompiled-src-file` keeps a stray Rust file under `src/` that is never compiled. It proves scans follow reachable crate/module trees instead of warning on every `src/**/*.rs` file.
- `phase5/custom-target-path` uses an explicit Cargo target path under `bin/` and a sibling module file. It proves scans include real crate targets outside `src/`.
- `phase5/nested-inline-module-alias-leakage` keeps parent `use` aliases next to a nested inline module with local lookalikes. It proves nested modules do not inherit alias environments that Rust scope would not expose.

Phase 6 adds cfg-aware reachability and nested `#[path = ...]` regressions:

- `phase6/cfg-reachability` mixes cfg-disabled functions and modules with one cfg-enabled module behind a default feature. It proves scans skip inactive code while still following the active reachability set.
- `phase6/nested-inline-path-attribute` keeps both a decoy `src/child.rs` and the real `src/outer/child.rs`. It proves nested inline `#[path = ...]` resolution follows the same file rustc would load.

## Fixture Rules

- keep fixtures small, explicit, and check-specific
- add positive and negative cases together so false-positive control evolves with each shipped diagnostic
- prefer the smallest project shape that proves the rule under test
