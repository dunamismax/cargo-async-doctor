# cargo-async-doctor

cargo-async-doctor is a Cargo subcommand for spotting common async Rust hazards and explaining the fix. It is intentionally narrow.

## Install

```bash
cargo install cargo-async-doctor
```

## Quick start

```bash
cargo async-doctor
cargo async-doctor --workspace
cargo async-doctor --message-format json
cargo async-doctor explain blocking-sleep-in-async
```

## Scope

- scans the selected package or workspace through `cargo metadata`
- emits human or JSON output
- ships explain text for known check IDs
- complements rustc and Clippy instead of replacing them
