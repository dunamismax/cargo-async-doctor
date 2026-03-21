# Fixtures

This directory holds small Cargo projects used by `cargo-async-doctor` tests.

Phase 2 ships focused fixtures for the first trustworthy checks:

- `placeholder/minimal-bin` is a tiny fixture project that proves manifest-path handling and fixture discovery work.
- `phase2/blocking-sleep-positive` and `phase2/blocking-sleep-negative` cover blocking sleep detection and false-positive control.
- `phase2/blocking-std-api-positive` and `phase2/blocking-std-api-negative` cover the narrow blocking `std::fs` allowlist and lookalike filtering.
- `phase2/sync-async-bridge-positive` and `phase2/sync-async-bridge-negative` cover clearly identifiable Tokio `block_on` bridge hazards and local lookalikes.

Keep fixtures small, explicit, and check-specific. Add positive and negative cases together so false-positive control evolves with each shipped diagnostic.
