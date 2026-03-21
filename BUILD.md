# BUILD

This is the canonical build plan and status tracker for `cargo-async-doctor`.

Agents and engineers should treat this file as a living document:

- read it before starting work
- update it when phase scope changes
- check off completed items as they land
- add dated notes to the progress log after meaningful milestones
- avoid drifting into work that is not represented here

If README and BUILD disagree, update one of them immediately so the repo has one coherent story.

## Mission

`cargo-async-doctor` should become a high-signal Cargo subcommand for detecting common async Rust hazards and explaining how to fix them.

The product bar is not “many warnings.” The product bar is:

- trustworthy checks
- low-noise output
- explanations that improve user judgment
- guidance grounded in primary sources and verified examples

## Current Status

- Current phase: `Phase 5 - Release Hardening`
- Repository maturity: `Phase 5 in-repo release hardening complete; local packaging and publish dry-run verification passed; external publish still pending`
- Public promise today: first-release-ready async diagnostics with stable IDs, clearer warning wording, fixture-backed false-positive control, explain mode for every shipped check, workspace-aware package/file/line reporting, and a documented release/versioning process
- External step still pending: publish `0.1.0` without changing the already-hardened in-repo surface

## Operating Rules

All contributors and agents should follow these rules:

- Prefer a smaller, better MVP over broad speculative coverage.
- Do not add a check unless its warning text can be defended.
- Every shipped check must have fixture coverage.
- Every shipped check must have a stable check ID.
- Every shipped check must have a user-facing explanation path.
- Runtime-specific guidance must be labeled as such.
- Do not silently expand the scope into a generic linter platform.
- Keep machine-readable output stable once introduced.

## Source-Of-Truth Inputs

Normative guidance should come from primary sources whenever possible:

- official Rust documentation
- official Tokio documentation
- rust-lang issue trackers
- tokio-rs issue trackers
- existing Clippy lint behavior and configuration

Secondary sources can help shape examples, but they should not be the basis for strong warnings by themselves.

## Product Shape

Near-term repository shape:

```text
cargo-async-doctor/
├── Cargo.toml
├── README.md
├── BUILD.md
├── LICENSE
├── src/
├── tests/
├── fixtures/
├── docs/
└── .github/
```

Expected near-term code shape:

- one binary crate for the Cargo subcommand
- internal modules for project loading, scanning, diagnostics, and rendering
- fixture-driven tests for checks and output

## Output Contract

The tool currently supports:

- concise human-readable terminal output
- structured JSON output
- stable check IDs
- `explain` mode for a specific check ID
- schema-versioned scan output with stable field names

The output contract should stay intentionally small. Additive JSON changes are preferred; breaking JSON changes require a schema version bump and release-note callout.

## MVP Check Themes

The first release should focus on a narrow set of checks such as:

- blocking sleep in async code
- obvious blocking std APIs inside async contexts
- dangerous sync/async bridge patterns with clear documented support
- guard-across-`.await` hazards not already covered well enough by current defaults

Anything beyond that should be treated as post-MVP unless it is clearly low risk and high value.

## Phase Tracker

Use this section as the active execution checklist.

### Phase 0 - Positioning And Planning

Status: `[x] Complete`

Goals:

- establish the repo’s mission
- define scope boundaries
- create starting docs and repo hygiene

Checklist:

- [x] Choose repository and crate name
- [x] Write initial `README.md`
- [x] Write initial `BUILD.md`
- [x] Add MIT `LICENSE`
- [x] Add baseline `.gitignore`
- [x] Align repo positioning with profile/public narrative
- [x] Publish initial docs to remote

Exit criteria:

- public repo clearly states what it is and is not
- a new contributor can identify the intended MVP

### Phase 1 - CLI Bootstrap

Status: `[x] Complete`

Goals:

- create the initial Cargo project
- establish command surface and output model
- wire basic test and CI structure

Checklist:

- [x] Create `Cargo.toml` and `src/main.rs`
- [x] Implement `cargo async-doctor --help`
- [x] Define CLI argument model
- [x] Define diagnostics model with stable IDs
- [x] Add a placeholder scan flow that exits successfully
- [x] Add human-readable output renderer
- [x] Add JSON output renderer skeleton
- [x] Add baseline unit tests
- [x] Add fixture test harness scaffolding
- [x] Add CI for `cargo fmt`, `cargo clippy`, and `cargo test`
- [x] Document local verification commands in `README.md`

Exit criteria:

- command runs locally
- help text is coherent
- output model exists in code
- repo has test and CI scaffolding

### Phase 2 - First Trustworthy Checks

Status: `[x] Complete`

Goals:

- ship the first real diagnostics
- prove the repo can produce low-noise warnings

Checklist:

- [x] Implement blocking sleep detection
- [x] Implement obvious blocking std API detection in async contexts
- [x] Implement at least one documented sync/async bridge hazard check
- [x] Assign stable IDs to each check
- [x] Write explanation text for each check
- [x] Add positive fixtures for each check
- [x] Add negative fixtures for false-positive control
- [x] Add snapshot or structured output tests
- [x] Update `README.md` with exact MVP check list
- [x] Record known limitations for each shipped check

Exit criteria:

- at least 2-3 real checks are shippable
- checks are backed by fixtures
- warnings are actionable and documented

Shipped scope:

- `blocking-sleep-in-async` for direct `std::thread::sleep(...)` calls and module aliases imported from `std::thread` inside async functions, async impl methods, nested async blocks, and async closures
- `blocking-std-api-in-async` for a narrow allowlist of direct `std::fs` calls and module aliases imported from `std::fs` inside the same async contexts
- `sync-async-bridge-hazard` for clearly identifiable Tokio `Handle::current().block_on(...)` and `Runtime::new().block_on(...)` patterns inside async contexts, including imported type aliases and simple receiver wrappers such as `unwrap`, `expect`, `clone`, and references

Deliberately not shipped in Phase 2:

- `guard-across-await` remains reserved only; a syntax-only implementation would be too noisy without stronger type/context handling

### Phase 3 - Explain Mode

Status: `[x] Complete`

Goals:

- let users ask for deeper context on a specific warning

Checklist:

- [x] Add `cargo async-doctor explain <check-id>`
- [x] Define canonical explain content format
- [x] Ensure every shipped check has explain content
- [x] Support both human and machine-readable references to check IDs
- [x] Link explanations to deeper guide material where appropriate
- [x] Add tests for unknown and known check IDs

Exit criteria:

- `explain` works for all shipped checks
- check IDs are stable and useful in docs/issues

### Phase 4 - Workspace And Path Fidelity

Status: `[x] Complete`

Goals:

- support real-world Cargo workspaces cleanly
- improve package/file/span reporting quality

Checklist:

- [x] Add multi-crate workspace fixtures
- [x] Resolve package context accurately
- [x] Improve file path reporting
- [x] Add optional span or line reporting where feasible
- [x] Test behavior in root package and workspace member scenarios
- [x] Document current parsing/analysis limits

Exit criteria:

- workspace output is understandable and dependable
- results point users to the right package and file

### Phase 5 - Release Hardening

Status: `[~] In-repo hardening complete; external publish pending`

Goals:

- make the tool fit for first public crate release

Checklist:

- [x] Review warning wording for clarity and precision
- [x] Audit JSON format for stability and naming consistency
- [x] Add release checklist
- [x] Add versioning policy note
- [x] Add changelog or release notes process
- [x] Test against at least a few real repositories
- [x] Review false-positive and false-negative behavior
- [x] Finalize crate metadata for publication
- [ ] Publish `0.1.0` externally

Exit criteria:

- maintainers can defend the initial public scope
- release process is documented
- crate is ready for external users before the final external publish step

### Phase 6 - Post-MVP Expansion

Status: `[ ] Not started`

Goals:

- expand carefully without sacrificing trust

Possible work:

- [ ] Add more async misuse checks
- [ ] Add config support for enabling/disabling checks
- [ ] Add severity levels
- [ ] Add per-check docs pages
- [ ] Add integration paths into editor or CI workflows
- [ ] Add links into `rust-async-field-guide`

Exit criteria:

- scope expansion remains disciplined
- the tool still feels focused rather than generic

## Implementation Notes

These are constraints to keep in mind during design and execution:

- Start with the simplest analysis approach that can support the first checks.
- Avoid a heavyweight architecture before the first real warnings exist.
- Keep check implementations separable and testable.
- Keep output rendering separate from detection logic.
- Prefer explicit fixtures over clever test generation at the start.

## Definition Of Done

A task is not done unless:

- code is implemented
- tests or fixtures cover it
- docs are updated if behavior changed
- the phase checklist is updated
- the progress log records the milestone if it materially changes repo status

## Progress Log

### 2026-03-21

- Phase 0 completed
- Initial repo docs, license, and git hygiene established
- Dual-push remote setup aligned with other repositories
- Phase 1 completed with the initial Cargo project, CLI bootstrap, stable diagnostics model, and placeholder scan flow
- Added separate human-readable and JSON renderers plus baseline unit and fixture-scaffolding tests
- Added GitHub Actions CI for `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`
- Updated `README.md` and this build tracker to match the implemented Phase 1 state
- Phase 2 completed with three shipped checks: blocking sleep in async contexts, a narrow allowlist of blocking `std::fs` calls in async contexts, and clearly identifiable Tokio `block_on` bridge hazards
- Replaced the placeholder scan with a syntax-driven analyzer that scans the selected manifest's `src/` tree while keeping detection logic separate from rendering
- Added positive and negative fixtures for each shipped check plus structured JSON assertions for the Phase 2 output contract
- Documented shipped scope limits, including that `--workspace` does not yet expand workspace members and that `guard-across-await` remains intentionally unshipped in this phase
- Phase 3 completed with `cargo async-doctor explain <check-id>` plus separate human and JSON explain renderers
- Defined a canonical explain shape with `title`, `summary`, `detects`, `does_not_detect`, `suggested_fixes`, and `references`
- Added explain coverage for every shipped Phase 2 check plus known/unknown check ID tests
- Updated the shipped check descriptions to match the actual alias and async-context detection scope
- Phase 4 completed with Cargo metadata-driven package selection for package manifests, root-package workspaces, and virtual workspace manifests
- Added multi-crate Phase 4 workspace fixtures covering a root package workspace, workspace-member scans, `--workspace` expansion, and virtual-workspace default-member behavior
- Added package-aware diagnostics with workspace-relative file paths plus optional line and column reporting sourced from syntax spans
- Updated human and JSON scan output to surface selected packages, package context, location fields, and current workspace/parsing limits
- Phase 5 in-repo release hardening completed: warning/help wording tightened, the legacy `placeholder` JSON field was removed before first release, and crate metadata plus release docs were finalized
- Added `docs/release.md` with the release checklist, versioning policy, changelog process, and JSON stability notes
- Added `CHANGELOG.md` and updated `README.md` to reflect release-ready-in-repo status with external publish still pending
- Ran `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `cargo package --allow-dirty --list`, and read-only scans against several real local repositories to review false positives and false negatives
- Real-repo validation found two likely true positives in `gitpulse` (`std::fs::create_dir_all` and `std::fs::metadata` inside async functions), no false positives in the checked `tauri::async_runtime::block_on` sync setup path, and one expected false negative in `rust-async-field-guide/examples/blocking-in-async-code` because the current shipped scope does not follow function imports such as `use std::thread::sleep; sleep(...)`
- Completed a final publish-prep pass: trimmed package contents to exclude repo-only `.github/` and `.gitignore`, confirmed `cargo package --allow-dirty --list` looked sane, and verified a publish dry-run on the prepared tree succeeds without further code or doc fixes

## Next Recommended Work

When starting a fresh engineering session, finish the external `0.1.0` release steps documented in `docs/release.md`.

Suggested order:

1. review and finalize the `CHANGELOG.md` release entry
2. create the release commit and tag
3. publish `0.1.0`
4. cut the matching GitHub release notes
5. move to `Phase 6 - Post-MVP Expansion` only after the first release is out
