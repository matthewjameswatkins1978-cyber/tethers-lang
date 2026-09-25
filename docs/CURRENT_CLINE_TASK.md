# TETHERS 0.8.0 RELEASE HARDENING AND READINESS

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Amber`

Owner: `Codex`

Route: `Codex on native Windows; local verification and external-consumer smoke`

Worker note: `docs/worker-notes/2026-09-25-tethers-readiness-cleanup.md`

Base branch: `main`

Base commit: `7e29110319c554a6586865ec6c47a45498696d16`

The Base commit is the starting baseline only, not the final 0.8.0 release SHA.

OCaml switch path: `N/A`

Rust toolchain: `1.97.1`; plain Cargo resolved by root pin; `--locked` mandatory

Toolchain preflight: `required`

Rust change class: `NON_RUST`

## Objective

Harden Tethers into a reusable, versioned authority component. Complete cleanup,
freeze and identify the final 0.8.0 release candidate, package it, tag that
exact commit, build the full runtime from the tag, rerun every required proof
against those artifacts, install that runtime locally, and record provenance.
The starting baseline is not the final release SHA.

## Relevant background and existing behaviour

- Canonical repository is `matthewjameswatkins1978-cyber/tethers-lang`, branch
  `main`, handoff SHA `7e29110319c554a6586865ec6c47a45498696d16`, including merged
  R0/R1/R2 architecture.
- Tethers Core deterministically plans. `tethers.authority/1` evaluates current
  host authority. The external Host owns physical effects and outcome reporting.
- Product version is `0.8.0` on the release candidate; latest released Windows
  full-runtime package at task start is `0.7.1`. Linux package proof PR #45 is open and its
  required package workflow currently fails. PR #29 remains an open draft with
  a unique `AGENTS.md` change.
- The 0.8.0 Rust crate is package-ready under `MIT OR Apache-2.0`; crates.io
  publication is deferred. The Windows x64 runtime is a separate process/binary
  consumption route. The OCaml engine remains executable-only.
- Initial local inventory found 59 linked worktrees across three Git stores,
  eleven dirty worktrees, and 21 unreachable commits in the J10 store, plus
  unreachable commits in two other stores. No checkout may be bulk-deleted.
- The intended local layout is one clean `main` checkout under
  `Projects/Tethers/tethers-lang` and only genuinely active task checkouts.
  Preserve outer project repositories that merely contain a Tethers checkout.

## Required behaviour

1. Record the canonical source, exact SHA, live package/version/support state,
   install route, and authority/consumer boundaries without changing product
   semantics or making unsupported promises.
2. Provide one tiny standard-library-only external consumer example using only
   the public `tethers` process interface and versioned JSON contracts.
3. Prove ALLOW, ASK/approval, DENY, useful receipt/evidence, malformed schema,
   and caller-supplied authority rejection from a fresh temporary consumer
   context; no effect may occur on ASK before approval or on DENY/error.
4. Classify each discovered Tethers checkout/worktree as canonical, active,
   merged/redundant, unique/recover, or obsolete experiment. Preserve unmerged
   commits, dirty content, useful unique work, credentials, host state, and
   personal data before removing any exact worktree directory.
5. Assess PR #45 and PR #29 against the exact starting baseline. Leave both remote
   PRs untouched and retain their unique work locally or remotely.
6. Count unreachable commits before and after cleanup; preserve every commit
   that is not proven represented by canonical `main` or another retained
   branch. Prune stale worktree metadata only after verifying it is stale.
7. Produce a readiness report with exact commands/results, worktree and dirty
   counts, recovery/retention/deletion decisions, open PR status, package and
   platform limits, external-smoke evidence, and known limitations.

## Relevant components

- `docs/PROJECT_CONTROL.md`
- `docs/CURRENT_CLINE_TASK.md`
- `README.md`
- `QUICKSTART.md`
- `docs/AI_INTEGRATION.md`
- `docs/VERSIONING.md`
- `docs/SUPPORT_MATRIX.md`
- `docs/tethers.authority.1.md`
- `docs/architecture/TETHERS_R2_EXTERNAL_AUTHORITY_GATE.md`
- `examples/external-consumer/`
- `scripts/verify-tethers.py`
- Local Tethers Git worktree stores identified in the cleanup inventory

## Frozen decisions and invariants

- `main` at the specified SHA is canonical; historical checkouts are evidence,
  not competing implementations.
- Keep one semantic authority. Core plans; host policy decides; Tethers Gate
  records authority and durable intent; external consumers execute effects and
  report outcomes. Never treat a Plan, approval, or commit response as proof an
  external effect happened.
- Reject malformed, unsupported, ambiguous, or caller-forged authority input.
  Fail closed and preserve ALLOW/ASK/DENY/unavailable distinctions.
- Do not add Tethers syntax, a second evaluator, an embedded LLM, a new runtime
  framework, dependencies, new platform promises, or product semantics.
- Use the approved `MIT OR Apache-2.0` license after checking incorporated
  source and dependency notices.
- Prove locked crate packaging and publish dry-run; do not publish to crates.io.
- Freeze a clean final candidate, tag that exact SHA `v0.8.0`, build release
  artifacts from the tag, rerun all proofs against those artifacts, and install
  and smoke-test the exact runtime.
- Never bulk-delete. Remove only individually inventoried worktree paths after
  their exact contents and recovery state have been checked. Never delete
  parent project directories, credentials, host databases/state, personal data,
  or unique assets.
- Retain PR #45 and PR #29 pending their owners' decisions.

## Acceptance criteria

1. The readiness docs state current product/release versions, supported target,
   crate and process/binary integration routes, and
   Tethers/external-Host responsibility split consistently.
2. An external example defines a Tether/Capability fixture, starts Tethers by
   executable path, negotiates the exact protocol, parses strict structured
   responses, and imports no internal Tethers modules.
3. The fresh-context smoke proves allow, ask, explicit approval, deny, receipt,
   malformed/unsupported protocol refusal, forbidden authority refusal, and
   zero effects on any non-approved path.
4. `cargo package --locked` and, where supported, `cargo publish --dry-run
   --locked` succeed from the frozen candidate; an external consumer builds
   from the package artifact.
5. Tag `v0.8.0` points to the frozen final SHA; artifact hashes and all proof
   results identify that exact SHA. The exact runtime is installed and smoked.
6. The worktree ledger covers all discovered Git stores and worktrees; each
   checkout has one classification and every dirty/unreachable item has a
   recorded preservation or retention outcome.
7. Before/after worktree counts and unreachable-commit counts are recorded;
   only redundant or obsolete exact paths are removed, and the intended
   canonical checkout is clean on `main` at current remote `main`.
8. PR #45 and PR #29 are assessed against the starting baseline and remain open;
   their unique content remains available.
9. The final task-packet checker, verifier, focused consumer tests/smoke,
   formatter check, whitespace check, and final status are reported truthfully.

## Required verification

1. Inspect current worktree inventory, status, branch ancestry, dirty/untracked
   and ignored paths, remote branch preservation, and unreachable commits.
2. `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`
3. `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`
4. Run standard-library external consumer unit tests and actual CLI/Gate smoke
   against the current built host and verified current OCaml engine.
5. `just verify`
6. Re-run the task-packet checker, `git diff --check`, inspect the complete
   diff, final worktree inventory, and final Git status.

## Formatting and checkpoint sequence

This is `NON_RUST`. Run `cargo fmt --manifest-path
tethers-0.1/host-rust/Cargo.toml --all -- --check` only. Do not modify Rust
source or run a mutating formatter.

## Completion and publication

Publish the reviewed 0.8.0 release commit, exact tag, and runtime artifacts after
all required verification passes. Do not publish the crate to crates.io. Mark
`COMPLETE` only if exact-tag evidence and the final packet checker support
completion; otherwise record the precise remaining blocker.

## Forbidden changes

- No Core/language/protocol/authority/replay/provider/packaging semantics or
  product-version changes.
- No speculative features, new product semantics, or crates.io publication.
- No deletion of any remote ref or PR.
- No worktree removal before review of tracked, untracked, ignored, and
  unreachable state plus a recorded recovery decision.
- No unrelated change to other projects nested beside or inside the Tethers
  worktree roots.

## Stop conditions

Stop on a lost/unclear unique change, any unreviewed credentials or user data,
unavailable required verification, an unsafe worktree relation, or conflicting
authority evidence. Preserve it and report the exact path/SHA and the smallest
unresolved decision.

## Expected pre-existing changes

None in this fresh task worktree at base `7e29110319c554a6586865ec6c47a45498696d16`.
