# BUILD.md

## Purpose

This is the live execution manual for `cargo-async-doctor`.

Use it to answer, at a glance:

- what the tool actually ships today
- which public contracts must stay stable
- what the next release is trying to prove
- which work is still open, and why it matters
- which expansion ideas are intentionally deferred until trust is earned

If code, README, CHANGELOG, and this file disagree, fix them together in the same change.

---

## Mission

Build and maintain a small, trustworthy Cargo subcommand for diagnosing async Rust hazards that are easy to write, painful to debug, and often invisible to standard tooling.

The project wins by being:

- **narrow** enough to stay believable
- **strict** enough to keep false positives low
- **clear** enough that each diagnostic points to a practical fix
- **stable** enough that humans and tooling can rely on it

This repo is not trying to become a general async linter. Trust is the product.

---

## Repo snapshot

**Published crate:** `0.1.3`
**Current branch posture:** clean post-`0.1.3` release
**Active phases:** Phase 5 (release discipline); Phase 3 (correctness hardening) is complete

### Shipped today

- Crate `cargo-async-doctor` published on crates.io at `0.1.2`
- Three shipped checks:
  - `blocking-sleep-in-async`
  - `blocking-std-api-in-async`
  - `sync-async-bridge-hazard`
- Human and JSON scan output
- Human and JSON `explain` output
- Stable shipped check IDs
- Package/workspace selection through `cargo metadata`
- Package-aware diagnostics with workspace-relative paths where available
- Best-effort line/column data when a direct syntax span exists
- Fixture-backed tests, unit tests, structured-output coverage, CI
- Release checklist and versioning policy in `docs/release.md`

### Not true yet

- no deep semantic resolution for macros, re-exports, wildcard imports, function imports, stored handles, or block-local `use` items
- no shipped `guard-across-await` check
- no claim that every diagnostic will always carry exact line/column data
- no broad lint surface beyond the intentionally small shipped check set

---

## Current release train

### Next release focus

The `0.1.3` correctness release has shipped. The next release should focus on either a fourth check (`guard-across-await`) if the false-positive story is defensible, or additional correctness/hardening work that earns further trust.

---

## Public contracts to protect

These are the surfaces that downstream users will feel immediately if they drift:

1. **Stable shipped check IDs**
2. **JSON field names and schema discipline**
3. **Warning wording meaning**
4. **Explain content matching actual detection behavior**
5. **Package/workspace targeting correctness**

The code can change aggressively behind those boundaries. The boundaries themselves change slowly and deliberately.

---

## Source-of-truth mapping

| File | Owns |
|------|------|
| `README.md` | public framing, install, quick start, current behavior, limits |
| `BUILD.md` | execution plan, active phases, decisions, risks, next moves |
| `CHANGELOG.md` | user-visible change history and `Unreleased` state |
| `docs/release.md` | release checklist and SemVer / JSON stability policy |
| `Cargo.toml` | published crate metadata and dependency surface |
| `src/cli.rs` | CLI surface and command definitions |
| `src/scan.rs` | package resolution and source discovery |
| `src/analysis.rs` | AST traversal and hazard detection |
| `src/diagnostics.rs` | diagnostic structures and stable check IDs |
| `src/explain.rs` | explain content keyed by check ID |
| `src/render.rs` | human and JSON presentation |
| `fixtures/` | positive/negative fixture truth |
| `tests/` | integration coverage and regression protection |

**Invariant:** if README says something is shipped, BUILD and the crate behavior must back it up.

---

## Working rules

1. **Trust beats breadth.** A smaller tool with believable output is better than a larger tool users learn to ignore.
2. **Stable IDs matter more than fast iteration.** Shipping a new ID is a public commitment.
3. **Explain quality must match detection quality.** Diagnostics without practical guidance are just noise with extra steps.
4. **Package/workspace targeting is correctness work.** If the wrong files are scanned, otherwise-good checks become untrustworthy.
5. **Syntax-driven is fine; pretending otherwise is not.** Be explicit about what the tool cannot see.
6. **Fixtures are the regression boundary.** Every shipped rule needs positive and negative examples that defend the intended behavior.
7. **Docs are product surface.** README drift, BUILD drift, and CHANGELOG drift are bugs.
8. **Release discipline is part of the tool.** For a Cargo plugin, crates.io and docs.rs behavior are part of the user experience.
9. **Dependencies stay justified.** New crates need a better reason than convenience.
10. **Expansion is earned.** New checks only ship after the existing surface is boringly reliable.

---

## Quality gates

### Minimum local gate

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

### Useful smoke set

```bash
cargo run -- async-doctor --help
cargo run -- --message-format human
cargo run -- --message-format json
cargo run -- explain blocking-sleep-in-async
cargo run -- --message-format json explain blocking-sleep-in-async
cargo run -- --manifest-path fixtures/phase4/workspace-root-package/Cargo.toml
cargo run -- --workspace --manifest-path fixtures/phase4/workspace-root-package/Cargo.toml
cargo run -- --manifest-path fixtures/phase4/workspace-root-package/member-bin/Cargo.toml
cargo run -- --manifest-path fixtures/phase4/virtual-workspace/Cargo.toml
cargo package --allow-dirty --list
```

For docs-only changes, run the smallest checks that prove the docs still describe the same repo state.

---

## Dependency strategy

### Current dependencies

| Crate | Purpose |
|-------|---------|
| `syn` (2.0, `full` + `visit`) | Rust parsing and AST traversal |
| `proc-macro2` (`span-locations`) | best-effort source span data |
| `clap` (4.5, `derive`) | CLI argument parsing |
| `serde` + `serde_json` | structured output |
| `anyhow` | error handling |
| `toml` | Cargo manifest parsing |

### Rule

The current dependency set is deliberate. Additions must answer three questions clearly:

1. what concrete limitation does this remove?
2. why is the complexity worth it for this small tool?
3. what contract or maintenance burden does it create?

---

## Phase dashboard

### Phase 0 — Repo framing and public-surface definition
**Status:** done

Goals:
- [x] repository structure, README, BUILD, LICENSE
- [x] define the current public surface and stability language
- [x] establish quality gates and release-process docs

Exit criteria: the repo can support real implementation and release work without public-surface ambiguity.

---

### Phase 1 — First shipped checks and explain flow
**Status:** done

Goals:
- [x] `blocking-sleep-in-async`
- [x] `blocking-std-api-in-async`
- [x] `sync-async-bridge-hazard`
- [x] explain flow keyed by stable check ID
- [x] human and JSON output
- [x] fixture-backed coverage for the shipped checks

Exit criteria: the initial tool is useful, explainable, and publishable.

---

### Phase 2 — Package/workspace targeting fidelity
**Status:** done

Goals:
- [x] package selection through `cargo metadata`
- [x] workspace-relative paths in diagnostics where possible
- [x] virtual workspace handling for default members and `--workspace`
- [x] package-manifest targeting
- [x] reachable module-tree scanning instead of naive file walking

Exit criteria: the scan target set matches Cargo intent closely enough that users can trust what was and was not analyzed.

---

### Phase 3 — Correctness hardening inside the shipped surface
**Status:** done

This is the current engineering center of gravity. The right move is to make the existing checks harder to fool before inventing new ones.

Completed or landed work:
- [x] target-root reachability avoids scanning stray uncompiled files
- [x] nested inline modules no longer leak outer alias assumptions
- [x] active `#[cfg(...)]` reachability keeps disabled code quiet
- [x] nested inline `#[path = ...]` modules resolve like rustc instead of matching the wrong sibling file

Still-open work:
- [x] run the latest scan behavior against several real async Rust repos and record whether the warnings still feel high-signal
- [x] audit the wording for each shipped diagnostic against the current detection boundaries and fix anything that now overstates certainty
- [x] review remaining reachability corner cases around unusual target layouts and module trees found during real-repo testing
- [x] confirm the current JSON output still expresses enough context for downstream tooling without adding unstable detail too early
- [x] keep `explain` content aligned with any wording or scope refinements from the hardening work

Exit criteria:
- shipped checks behave more predictably on real repositories
- warning wording matches what the syntax-driven engine can actually prove
- JSON output remains stable while the detection logic gets sharper

All exit criteria verified. Phase 3 is complete.

Active risks:
- ~~**risk:** correctness work that is not tested on real repos can still feel good locally and be noisy in practice~~ — mitigated: spot-checked against mini-redis, hyper, reqwest, axum, sqlx, and tokio; all findings were legitimate true positives
- ~~**risk:** wording drift can make a syntactic heuristic sound stronger than it really is~~ — mitigated: all diagnostic and explain wording audited and confirmed accurate
- ~~**risk:** silent JSON drift would punish integrators for internal refactors~~ — mitigated: JSON output verified stable against schema_version 1 contract

---

### Phase 4 — Check expansion with a strict false-positive bar
**Status:** planned

This phase starts only after Phase 3 has earned it.

Goals:
- [ ] write down the exact success/failure story for `guard-across-await` before any implementation push
- [ ] decide whether deeper name resolution would improve signal enough to justify the complexity cost
- [ ] choose the check-shaping policy for future rules: Tokio-specific when necessary vs. runtime-agnostic when credible
- [ ] require a fixture story and explanation story before any new check is considered shippable

Exit criteria: any new check can be defended on trust grounds before it is defended on ambition grounds.

Active risks:
- **risk:** pressure to grow the check list can damage the thing users actually value: believable output
- **risk:** deeper resolution logic may cost more complexity than this intentionally small tool should carry
- **risk:** expansion can blur the repo's identity if the scope stops being obviously narrow

---

### Phase 5 — Release polish and long-term maintenance discipline
**Status:** in-progress

Goals:
- [ ] keep `CHANGELOG.md` current as fixes land instead of reconstructing history later
- [ ] keep README/BUILD/release docs synchronized before each release, not after
- [ ] make patch-release readiness cheap to verify with the existing gate + smoke set
- [ ] keep crates.io/docs.rs/package sanity part of normal release review
- [ ] preserve the narrow-tool positioning as the repo grows so the docs never imply a general lint suite

Exit criteria: releasing a small correctness update feels routine instead of ceremonial, and the docs still tell the truth.

Active risks:
- **risk:** docs can freeze around the last release while `Unreleased` keeps moving
- **risk:** public-facing phrasing can slowly overstate maturity if maintenance discipline gets sloppy

---

## Milestone map

### Milestone A — Next patch release is defensible

**Status:** achieved — `0.1.3` released 2026-03-24

Definition:
- unreleased correctness fixes are verified locally and spot-checked on real repos
- docs and changelog agree on what changed
- release notes can explain the value in one short paragraph

### Milestone B — Shipped checks feel boringly reliable

Definition:
- obvious async hazards in the current scope are caught consistently
- false positives are rare enough that users do not learn to tune the tool out
- wording is precise enough that a warning reads like a diagnosis, not a guess

### Milestone C — Expansion decision made on evidence, not itchiness

Definition:
- the repo has enough confidence in the current surface to justify either a fourth check or a conscious decision to stay narrow longer
- `guard-across-await` is either scoped rigorously or explicitly deferred again

---

## Decisions

### decision-0001 — Syntax-driven analysis over semantic resolution
**Phase:** 1

The tool uses `syn`-level analysis instead of full compiler-style name resolution. That keeps implementation size and false-positive pressure under control, at the cost of missing re-exports, macros, stored handles, wildcard imports, and other deeper cases.

### decision-0002 — Stable shipped check IDs are a public contract
**Phase:** 1

Renaming or removing a shipped check ID is a breaking change. New IDs must be treated like API surface.

### decision-0003 — `guard-across-await` remains reserved
**Phase:** 1, revisited in Phase 4

The idea is valuable, but it does not ship until the repo can defend the false-positive story and explainability story.

### decision-0004 — Rendering stays separate from detection
**Phase:** 1

Detection produces diagnostics; rendering formats them. This keeps presentation changes from bleeding into analysis logic and helps preserve JSON contract stability.

### decision-0005 — Release/process docs are part of the product
**Phase:** 2

For a published Cargo plugin, crates.io metadata, docs.rs rendering, changelog clarity, and release repeatability are part of what users consume.

### decision-0006 — Grow by hardening first, not by collecting checks
**Phase:** 3

The next meaningful credibility gains come from better targeting, reachability fidelity, wording precision, and real-repo validation. New checks can wait.

---

## Open questions

| Question | Why it matters |
|----------|----------------|
| Should the next release be cut as soon as the current hardening work is verified, or should it batch one more correctness pass? | determines patch cadence and release scope |
| How many real-repo spot checks are enough before calling the latest reachability fixes ready? | prevents local-only confidence |
| Is `guard-across-await` actually shippable without semantic overreach? | decides whether Phase 4 is real or still aspirational |
| How much deeper name resolution is worth paying for in an intentionally narrow tool? | determines long-term complexity ceiling |
| Should future check growth stay Tokio-shaped when needed or push harder for runtime-agnostic wording and detection? | sets scope expectations for users |

---

## Immediate next moves

1. ~~verify the current unreleased correctness fixes with the existing smoke set~~ — done
2. ~~run the tool against a few real async Rust repos and note any surprising noise or misses~~ — done: tested mini-redis, hyper, reqwest, axum, sqlx, tokio; zero false positives found
3. ~~tighten diagnostic wording anywhere the implementation is more heuristic than the prose currently implies~~ — done: all wording audited and confirmed accurate
4. ~~keep README, BUILD, CHANGELOG, and release docs aligned before deciding to publish~~ — done
5. ~~decide whether the next cut is a small patch release now or after one more hardening pass~~ — done: released `0.1.3`

---

## Progress log

### 2026-03-24

- released `0.1.3` to crates.io: cfg-aware reachability and `#[path = ...]` resolution fixes
- updated BUILD.md, CHANGELOG.md, and Cargo.toml for the `0.1.3` release
- Milestone A achieved
- completed Phase 3 correctness hardening: all open work items verified
- spot-checked the tool against six real async Rust repos (mini-redis, hyper, reqwest, axum, sqlx, tokio) — zero false positives; all findings were legitimate true positives
- audited all diagnostic wording and explain content against current detection boundaries — no overstated claims found
- verified JSON output stability against schema_version 1 contract
- full smoke set and all 44 tests pass cleanly after clean rebuild
- Milestone A criteria met: next patch release is defensible
- rewrote `BUILD.md` as a live execution manual instead of a mostly static status handoff
- tied active planning to the real post-`0.1.2` `Unreleased` work already captured in `CHANGELOG.md`
- made the next release gate explicit so the repo has meaningful unchecked work instead of vague future intent

### 2026-03-22

- revised README and BUILD for structural consistency across the portfolio
- restored `README.md` as a real repo overview after the earlier docs-tightening regression
- reintroduced `BUILD.md` as the operational status document
- released `0.1.2` with package-target reachability and nested inline-module fixes

### 2026-03-21

- released `0.1.1`
- established the initial public release sequence and release-process docs

---

*Only log things that actually happened. Aspirations belong in phases, milestones, and next moves — not in the history.*
