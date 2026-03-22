# BUILD.md

## Purpose

This file is the execution manual for `cargo-async-doctor`.

It keeps the repo honest while the tool evolves from its initial release into a mature, stable diagnostic. At any point it should answer:

- what the tool actually ships today
- which public surfaces must stay stable
- which checks are real versus reserved
- what the next safe improvement path is
- what must be proven before stronger claims are made

This is a living document. When code and docs disagree, fix them together in the same change.

---

## Mission

Build and maintain a small, trustworthy Cargo subcommand for diagnosing common async Rust hazards.

The emphasis is on:

- **High-signal checks** --- a few reliable detections beat many noisy ones
- **Clear explanations** --- every shipped check has actionable explanation content
- **Stable public surfaces** --- shipped check IDs and JSON output are part of the contract
- **Conservative scope** --- the tool is intentionally narrow
- **Low false-positive pressure** --- trust is the product

---

## Repo snapshot

**Current phase: Phase 3 --- correctness hardening / Phase 5 --- maintenance discipline**
**Published version: `0.1.2`**

What exists today:

- Published crate `cargo-async-doctor` `0.1.2` on crates.io
- Three shipped checks: `blocking-sleep-in-async`, `blocking-std-api-in-async`, `sync-async-bridge-hazard`
- Human and JSON scan output
- Human and JSON `explain` output
- Stable shipped check IDs
- Package/workspace selection through `cargo metadata`
- Package-aware diagnostics with workspace-relative paths where possible
- Line/column data when a direct syntax span exists
- Fixture-backed tests, unit tests, structured-output checks, and CI
- Release notes and versioning policy documented in-repo

What does **not** exist today:

- Deep semantic resolution for macros, re-exports, wildcard imports, stored runtime handles, or block-local `use` items
- The reserved `guard-across-await` check
- Any promise that every future diagnostic can carry exact line/column information
- A broad lint surface; this repo is intentionally narrow

---

## Source-of-truth mapping

| File | Owns |
|------|------|
| `README.md` | Public framing, install, quick start, current limitations |
| `BUILD.md` | Implementation map, phase tracking, decisions, working rules |
| `CHANGELOG.md` | User-facing release history |
| `Cargo.toml` | Published crate metadata, dependency surface |
| `docs/release.md` | Release checklist and versioning policy |
| `src/main.rs` | Binary entry point |
| `src/lib.rs` | Library root |
| `src/cli.rs` | Clap command definitions |
| `src/scan.rs` | Cargo metadata package resolution |
| `src/analysis.rs` | Syn AST visitor and hazard detection |
| `src/diagnostics.rs` | Diagnostic types and check IDs |
| `src/explain.rs` | Explain content by check ID |
| `src/render.rs` | Human and JSON output rendering |
| `fixtures/` | Positive/negative fixture truth for shipped checks |
| `tests/` | Fixture-backed integration tests and helpers |

**Invariant:** If docs, code, and CLI output ever disagree, the next change must reconcile all three.

---

## Working rules

1. **Stable check IDs matter more than check count.** Shipping a new ID is a public commitment. Don't do it casually.
2. **Explanation quality >= detection quality.** A warning without a good explanation is worse than no warning.
3. **JSON stability is contractual.** Once a field ships in structured output, breaking it costs real downstream users.
4. **If a check cannot be made trustworthy, do not ship it.** No "experimental" flags as an excuse for noise.
5. **Package/workspace targeting is correctness, not convenience.** Bugs here quietly damage confidence even when individual checks work.
6. **Syntax-driven but honest about it.** The tool says what it can and cannot see. No false claims of semantic analysis.
7. **Fixture-driven regression protection.** Every shipped check has positive and negative fixture coverage.
8. **Docs are product surface.** Stale docs are bugs. README/BUILD drift can overstate what the tool actually does.
9. **Conservative dependency posture.** Every dependency must justify itself. The current set (`syn`, `clap`, `serde`, `serde_json`, `anyhow`, `proc-macro2`, `toml`) is deliberate.
10. **Release process is part of the product.** For a cargo plugin, packaging discipline is not optional.

---

## Tracking conventions

Use this language consistently in docs, commits, and issues:

| Term | Meaning |
|------|---------|
| **done** | Implemented and verified |
| **checked** | Verified by command or test output |
| **planned** | Intentional, not started |
| **in-progress** | Actively being worked on |
| **blocked** | Cannot proceed without a decision or dependency |
| **risk** | Plausible failure mode that could distort the design |
| **decision** | A durable call with consequences |

When new work lands, update: repo snapshot, phase dashboard, decisions (if architecture changed), and progress log with date and what was verified.

---

## Quality gates

### Minimum local gate (all phases)

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

### Useful command smoke set

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

For docs-only changes, verify wording consistency and that repo state matches documented commands.

If a gate is temporarily unavailable, document why. Never silently skip.

---

## Dependency strategy

### Current dependencies

| Crate | Purpose |
|-------|---------|
| `syn` (2.0, full + visit) | Rust source parsing and AST traversal |
| `proc-macro2` (span-locations) | Source span data for diagnostics |
| `clap` (4.5, derive) | CLI argument parsing |
| `serde` + `serde_json` | Structured JSON output |
| `anyhow` | Error handling |
| `toml` | Cargo manifest parsing |

### Dependency posture

The current set is deliberate and justified. `syn` is non-negotiable for Rust AST analysis. `clap` earns its weight for a cargo subcommand. `serde`/`serde_json` are required for JSON output stability. Future additions must clear the same bar: what does it replace, what does it cost, is there a simpler alternative?

---

## Phase dashboard

### Phase 0 --- Repo framing and public-surface definition
**Status: done**

Goals:
- [x] Repository structure, README, BUILD, LICENSE
- [x] Define shipped check surface and stability guarantees
- [x] Establish quality gates and release process

Exit criteria: Repo structure supports real implementation work. Public docs do not overclaim.

---

### Phase 1 --- First shipped checks and explain flow
**Status: done**

Goals:
- [x] `blocking-sleep-in-async` detection
- [x] `blocking-std-api-in-async` detection
- [x] `sync-async-bridge-hazard` detection
- [x] Explain flow with canonical content per check ID
- [x] Human and JSON output rendering
- [x] Fixture-backed tests for all shipped checks

Exit criteria: Three checks ship with explanation content, human/JSON output, and fixture coverage.

---

### Phase 2 --- Package/workspace targeting fidelity
**Status: done**

Goals:
- [x] Package selection through `cargo metadata`
- [x] Workspace-relative paths in diagnostics
- [x] Virtual workspace support (default members and `--workspace`)
- [x] Package manifest targeting

Exit criteria: Package/workspace selection matches `cargo check` behavior. Diagnostics carry correct package context.

---

### Phase 3 --- Correctness hardening and structured output stability
**Status: in-progress**

Goals:
- [ ] Package/workspace targeting correctness edge cases
- [ ] Nested module and `#[cfg(...)]` reachability fidelity
- [ ] Warning wording precision
- [ ] JSON stability discipline
- [ ] Explain coverage staying aligned with shipped behavior

Exit criteria: Current shipped surface is tightened without widening recklessly. JSON output is stable enough for downstream tooling.

Risks:
- **risk:** adding checks to meet a count target damages trust faster than it increases usefulness
- **risk:** JSON drift without explicit schema discipline is expensive for downstream users

---

### Phase 4 --- Future check expansion with strict false-positive control
**Status: not started**

Goals:
- [ ] Evaluate `guard-across-await` shippability
- [ ] Determine deeper name resolution investment vs. tool size
- [ ] Decide runtime-agnostic vs. Tokio-shaped check scope

Exit criteria: Any new check has a false-positive story strong enough to defend before merging.

Risks:
- **risk:** adding weak checks damages the trust that makes this tool useful
- **risk:** scope creep toward a general async linter

---

### Phase 5 --- Release polish and long-term maintenance discipline
**Status: in-progress**

Goals:
- [ ] Changelog stays current as fixes land
- [ ] Release process stays explicit and documented
- [ ] Packaging/docs.rs/crates.io sanity stays cheap to verify
- [ ] Docs stay aligned with the real shipped surface

Exit criteria: The repo is easy to defend as a small, stable, published tool.

Risks:
- **risk:** README/BUILD drift can overstate what the tool actually does
- **risk:** package selection correctness bugs can quietly damage confidence

---

## Decisions

### decision-0001: Syntax-driven analysis over semantic resolution
**Phase:** 1

The tool uses `syn` for AST-level detection rather than attempting full name resolution. This means it catches direct calls and simple aliases but misses re-exports, macros, and stored handles. The tradeoff is deliberate: trustworthy results on the common case beat ambitious results with high false-positive rates.

### decision-0002: Stable check IDs as public contract
**Phase:** 1

Shipped check IDs (`blocking-sleep-in-async`, etc.) are part of the public surface. Renaming or removing them is a breaking change. New IDs require the same commitment.

### decision-0003: `guard-across-await` stays reserved
**Phase:** 1 (revisited Phase 4)

The check is reserved but not shipped. It stays that way until the repo can defend it with acceptable noise levels.

### decision-0004: Separate rendering from detection
**Phase:** 1

Detection produces diagnostics. Rendering formats them. This separation keeps JSON stability independent of detection improvements and makes human output a presentation concern, not a logic concern.

### decision-0005: Release/process docs are product
**Phase:** 2

For a cargo plugin, release discipline is not optional extra prose. `docs/release.md` and `CHANGELOG.md` are part of the tool's product surface.

---

## Risks

### Active risks

- **risk:** adding checks to meet a count target damages trust faster than it increases usefulness
- **risk:** public JSON drift without explicit schema discipline would be expensive for downstream users
- **risk:** README/BUILD drift can overstate what the tool actually does
- **risk:** package selection correctness bugs can quietly damage confidence even when individual checks are fine
- **risk:** deeper name resolution investment may not be worth the complexity for an intentionally small tool

### Mitigated risks

- **risk (mitigated):** overclaiming in docs --- README and BUILD explicitly state current limits
- **risk (mitigated):** unstable check IDs --- stable ID contract established from Phase 1

---

## Open questions

| Question | Phase | Impact |
|----------|-------|--------|
| Should `guard-across-await` ever ship? | 4 | Check surface expansion |
| How much deeper name resolution is worth adding? | 4 | Tool complexity vs. intentional narrowness |
| Should future checks be Tokio-shaped only or runtime-agnostic? | 4 | Scope definition |
| How to handle macros that generate async code? | 4 | Detection fidelity |

---

## Immediate next moves

- Keep README, BUILD, and release docs aligned with `0.1.2` behavior
- Continue correctness hardening inside the current shipped check set
- Only add new checks when the semantics and false-positive story are strong enough to defend
- Keep `CHANGELOG.md` current as fixes land

---

## Progress log

### 2026-03-22

- Revised README and BUILD to match cross-repo documentation standards
- Kept all existing content, restructured for consistency

### 2026-03-22

- Restored `README.md` as a real repo overview after the docs-tightening regression
- Reintroduced `BUILD.md` as the operational handoff / status document
- Kept the current published `0.1.2` surface and release docs as the source of truth

### 2026-03-22

- `0.1.2` released with package-target reachability and nested inline-module fixes

### 2026-03-21

- `0.1.1` released
- Initial public release sequence and release-process docs established

---

## Decision log

- Stable check IDs matter more than shipping more checks quickly
- Syntax-driven but trustworthy beats ambitious but noisy
- `guard-across-await` stays reserved until the repo can defend it
- Release/process docs are part of the product for a cargo plugin, not optional extra prose

---

*Update this log only with things that actually happened.*
