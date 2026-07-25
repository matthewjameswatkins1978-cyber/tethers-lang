# Current Implementation Task

Control contract: `1`

Task: `J05 exact one-shot Ask approval and resume`

Status: `READY`

Task colour: `Red`

Owner: `Codex`

Route: `Codex — fresh reconciliation branch from current origin/main`

Worker note: `docs/worker-notes/2026-07-25-j05-authoritative-implementation.md`

Base branch: `main`

Base commit: `2c9b161579a7bdf016754dc6814bde9c4f1b79b7`

## Objective

Implement the authoritative one-shot Ask approval and resume boundary defined in
`docs/J05_EXACT_ASK_APPROVAL_DESIGN.md`, using the preserved safety branch only
as reference and rebuilding the production orchestration seam from current
`main`.

## Relevant background and existing behaviour

- J04a is the accepted implementation baseline.
- The immutable safety branch
  `safety/preserve-local-main-20260725` at
  `f74999aba9135f0493cf28693ba6444c22388294` contains useful draft types and
  tests, but its J05 design was not frozen before implementation and its
  production orchestration is defective.
- The safety snapshot also contains obsolete workflow files and incomplete J07
  code. Neither may be imported wholesale.
- `docs/J05_EXACT_ASK_APPROVAL_DESIGN.md` is the sole J05 authority.

## Required behaviour

1. Add the exact approval proof, binding digest, state model, and one-shot atomic
   consume boundary from the authoritative design.
2. Connect J05 to the real production host path rather than test-only helper
   functions.
3. The resume seam must perform fresh current resolution, schema validation,
   host-owned scope assessment, and effective policy evaluation itself. It must
   not trust a caller-supplied final policy result.
4. Changed proof or fresh non-approval gate failure invalidates the matching
   pending or approved record and prevents dispatch.
5. Store transitions occur before matching Trail claims. Failed transitions
   produce no false `approval_granted`, `approval_denied`,
   `approval_cancelled`, `approval_invalidated`, or `approval_consumed` entry.
6. Terminal records are not reused as pending requests. Missing, denied,
   cancelled, invalidated, and consumed states retain distinct errors.
7. A matching approved record is consumed exactly once before durable intent.
   It is never restored after intent or execution failure.
8. Every unattempted branch creates no durable intent, provider call, execution
   outcome, or standard result Anchor.
9. Implement the complete 28-case focused verification matrix from the design,
   plus proportionate integration and regression coverage.

## Relevant components

- `docs/J05_EXACT_ASK_APPROVAL_DESIGN.md`
- `docs/DECISIONS.md` (J03, J03a, J03b, J04a)
- `docs/CAPABILITY_BRIDGE.md`
- `tethers-0.1/host-rust/src/policy.rs`
- `tethers-0.1/host-rust/src/dispatch.rs`
- `tethers-0.1/host-rust/src/main.rs`
- `tethers-0.1/host-rust/src/result_anchor.rs`
- new focused approval module if justified
- relevant existing Rust and host integration tests
- safety commit `f74999aba9135f0493cf28693ba6444c22388294`
  for selective reference only

## Frozen decisions and invariants

- The authoritative J05 design controls all approval semantics.
- Approval confirms one exact Action; it never becomes standing permission.
- Fresh ordinary policy and trust checks precede approval use on every resume.
- Only a host-recognised human decision boundary can approve, deny, or cancel.
- No automatic retry.
- No approval restoration after consumption.
- No standard Result Anchor for an unattempted Action.
- Tethers Core and OCaml protocol semantics remain unchanged.
- J06 documentation and all J07 implementation are out of scope.
- Gorilla Coding workflow files from current `main` remain authoritative.

## Acceptance criteria

1. The exact approval proof, constituent-field checks, binding digest, state
   vocabulary, and atomic consume primitive match the authoritative design and
   are proved by focused tests.
2. A production-path integration test proves real host processing reaches the
   J05 request and resume seam rather than otherwise-unused helpers.
3. Resume tests prove fresh resolution, schema, scope, binding, and effective
   policy evaluation occur inside the seam and caller-supplied policy cannot
   authorise dispatch.
4. Changed proof fields and fresh Deny, Unavailable, schema, binding, and scope
   failures invalidate the matching live record and make zero executor calls.
5. Tests prove every authorisation Trail claim follows a successful state or
   policy transition, and failed transitions produce no false claim.
6. Missing, denied, cancelled, invalidated, consumed, and existing terminal
   records produce distinct state-correct outcomes and are not reused as pending.
7. Concurrency and replay tests prove one matching approval is consumed exactly
   once before intent and is never restored after later failure.
8. Every unattempted branch is proved to create zero durable intents, provider
   calls, execution outcomes, and standard result Anchors.
9. All 28 design cases have individually identifiable tests or a documented
   one-to-one mapping, and the full required verification passes or is reported
   precisely.

## Required verification

Run sequentially from `tethers-0.1`:

```powershell
Set-Location host-rust
cargo fmt --check
cargo test
Set-Location ..
pwsh -NoProfile -File scripts/check-fixtures.ps1
pwsh -NoProfile -File scripts/test-engine.ps1
pwsh -NoProfile -File scripts/test-mcp-transcripts.ps1
pwsh -NoProfile -File scripts/test-host-denial.ps1
pwsh -NoProfile -File scripts/test-host-execution-failure.ps1
pwsh -NoProfile -File scripts/demo.ps1
Set-Location engine-ocaml
opam exec -- dune build
Set-Location ..
```

Then from repository root:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
git diff --check
git status --short --branch
git diff --stat
git diff
```

Record every command and exact result. Never claim an unrun check passed.

## Forbidden changes

- Do not modify or delete the safety branch or preservation commit.
- Do not merge or cherry-pick the preservation commit.
- Do not import old workflow, Copilot, Cline-control, or task-control files from
  the safety branch.
- Do not transplant preserved `main.rs` wholesale.
- Do not implement J06, deadlines, monotonic clocks, J07, or uncertain outcomes.
- Do not add durable cross-restart approval persistence, GUI, remote approval
  endpoint, standing approval, retry, or compensation.
- Do not change Tethers language, OCaml planner, manifest format, or MCP protocol.
- Do not push, merge, tag, or publish unless Matthew explicitly authorises it.

## Stop conditions

Stop with exact evidence and one smallest unresolved question when:

- the authoritative design conflicts with accepted J03/J04a behaviour;
- a fresh-policy resume cannot be connected without changing an unapproved
  trust boundary;
- two materially similar implementation attempts fail;
- the safety reference and current code cannot be reconciled without importing
  unrelated or J07 work;
- an unrelated repository or environment failure prevents trustworthy
  verification.

## Expected pre-existing changes

None. Start from a clean fresh branch created from current `origin/main` at or
after `2c9b161579a7bdf016754dc6814bde9c4f1b79b7`.