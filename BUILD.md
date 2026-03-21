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

- Current phase: `Phase 2 - First Trustworthy Checks`
- Repository maturity: `Phase 1 CLI skeleton implemented; real diagnostics not started`
- Public promise today: runnable CLI skeleton with stable output model, test harness, and CI scaffold
- Public promise after Phase 2: first real async diagnostics with fixture coverage and actionable wording

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

The tool should eventually support:

- concise human-readable terminal output
- structured JSON output
- stable check IDs
- `explain` mode for a specific check ID

The first implementation should keep the output contract intentionally small and easy to evolve.

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

Status: `[ ] Not started`

Goals:

- ship the first real diagnostics
- prove the repo can produce low-noise warnings

Checklist:

- [ ] Implement blocking sleep detection
- [ ] Implement obvious blocking std API detection in async contexts
- [ ] Implement at least one documented sync/async bridge hazard check
- [ ] Assign stable IDs to each check
- [ ] Write explanation text for each check
- [ ] Add positive fixtures for each check
- [ ] Add negative fixtures for false-positive control
- [ ] Add snapshot or structured output tests
- [ ] Update `README.md` with exact MVP check list
- [ ] Record known limitations for each shipped check

Exit criteria:

- at least 2-3 real checks are shippable
- checks are backed by fixtures
- warnings are actionable and documented

### Phase 3 - Explain Mode

Status: `[ ] Not started`

Goals:

- let users ask for deeper context on a specific warning

Checklist:

- [ ] Add `cargo async-doctor explain <check-id>`
- [ ] Define canonical explain content format
- [ ] Ensure every shipped check has explain content
- [ ] Support both human and machine-readable references to check IDs
- [ ] Link explanations to deeper guide material where appropriate
- [ ] Add tests for unknown and known check IDs

Exit criteria:

- `explain` works for all shipped checks
- check IDs are stable and useful in docs/issues

### Phase 4 - Workspace And Path Fidelity

Status: `[ ] Not started`

Goals:

- support real-world Cargo workspaces cleanly
- improve package/file/span reporting quality

Checklist:

- [ ] Add multi-crate workspace fixtures
- [ ] Resolve package context accurately
- [ ] Improve file path reporting
- [ ] Add optional span or line reporting where feasible
- [ ] Test behavior in root package and workspace member scenarios
- [ ] Document current parsing/analysis limits

Exit criteria:

- workspace output is understandable and dependable
- results point users to the right package and file

### Phase 5 - Release Hardening

Status: `[ ] Not started`

Goals:

- make the tool fit for first public crate release

Checklist:

- [ ] Review warning wording for clarity and precision
- [ ] Audit JSON format for stability and naming consistency
- [ ] Add release checklist
- [ ] Add versioning policy note
- [ ] Add changelog or release notes process
- [ ] Test against at least a few real repositories
- [ ] Review false-positive and false-negative behavior
- [ ] Finalize crate metadata for publication
- [ ] Publish `0.1.0`

Exit criteria:

- maintainers can defend the initial public scope
- release process is documented
- crate is ready for external users

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

## Next Recommended Work

When starting a fresh engineering session, begin with `Phase 2 - First Trustworthy Checks`.

Suggested order:

1. implement blocking sleep detection with positive and negative fixtures
2. add at least one more high-confidence async hazard check
3. tighten warning wording and explanation text alongside each check
4. add structured output assertions for real diagnostics
5. update this BUILD file as tasks complete
