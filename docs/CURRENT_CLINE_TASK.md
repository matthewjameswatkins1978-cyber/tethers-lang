# Current Implementation Task

Control contract: `1`

Task: `J05 exact one-shot Ask approval and resume`

Status: `READY`

Task colour: `Red`

Owner: `Codex`

Route: `Codex — fresh reconciliation branch from current origin/main`

Worker note: `docs/worker-notes/2026-07-25-j05-authoritative-implementation.md`

Base branch: `main`

Base commit: `d7962642fc85a433a7d4257de73a9f2417f4418f`

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
- The packet checker matches field and section names case-insensitively.
- The current demonstration host deliberately returns
  `ScopeAssessment::ScopeNotEstablished` for its structured manifest. J03b must
  therefore deny that Action before Ask. J05 must not weaken this boundary or
  invent a `project`/`task` argument convention.
- J05 production-path evidence may instead use a dedicated, test-only capability
  and binding fixture whose scope can already be established honestly, or an
  `Unrestricted` fixture whose mandatory per-call confirmation produces Ask.
  This fixture exists to exercise the real host orchestration seam, not to alter
  production scope semantics or make the existing demo dispatchable.

## Preflight self-repair authority

Codex must fix bounded control-plane or preflight defects itself, rerun the gate,
and continue without returning to Lucy when the correction does not alter product
semantics, trust boundaries, implementation scope, or accepted architecture.

Codex is authorised to:

- restore this packet and its worker note after a previous preflight-only
  `BLOCKED` attempt;
- update `Base commit` to the current clean `origin/main` HEAD when no product
  implementation has started and intervening commits have already been accepted
  on remote `main`;
- correct packet formatting, heading capitalisation, stale status, worker-note
  metadata, expected-clean-state declarations, ignored local evidence files, and
  equivalent mechanical checker-contract faults;
- make the smallest necessary change to the packet checker when the checker is
  clearly rejecting semantically equivalent packet formatting rather than
  detecting a real safety problem;
- record every self-repair in the final worker note and include before/after gate
  evidence.

Codex must not use self-repair authority to change J05 behaviour, acceptance
meaning, permissions, security boundaries, frozen decisions, implementation
scope, or forbidden changes. Stop only when repair would require one of those
changes, risk loss of work, conceal unexpected source drift, or weaken a safety
control.

A preflight-only correction is not counted as one of the two failed
implementation attempts.

## Required behaviour

1. Add the exact approval proof, binding digest, state model, and one-shot atomic
   consume boundary from the authoritative design.
2. Connect J05 to the real production host orchestration path rather than
   test-only helper functions. Exercise that path with a dedicated authorised
   test capability/binding fixture that can honestly reach Ask under J03b. Do not
   change the existing structured-scope demo's fail-closed behaviour.
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
- a dedicated test capability/binding fixture and production-path integration
  harness if needed
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
- The existing structured-scope demo remains `ScopeNotEstablished` and denied.
- A J05 test fixture must use declared, trusted data and an honest host-owned
  assessment. It may not infer scope from convenient argument names.
- A test-only fixture proves reachability of the production orchestration seam;
  it does not create a new production capability or change manifest semantics.

## Acceptance criteria

1. The exact approval proof, constituent-field checks, binding digest, state
   vocabulary, and atomic consume primitive match the authoritative design and
   are proved by focused tests.
2. A production-path integration test proves real host processing reaches the
   J05 request and resume seam through the dedicated authorised fixture rather
   than otherwise-unused helpers. The existing structured-scope demo remains
   safely denied before Ask.
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
- Do not make the existing structured-scope demo reach Ask by inventing a scope
  mapping or returning `WithinScope` without a trusted assessor.
- Do not turn a test fixture into a general production scope assessor.
- Do not implement J06, deadlines, monotonic clocks, J07, or uncertain outcomes.
- Do not add durable cross-restart approval persistence, GUI, remote approval
  endpoint, standing approval, retry, or compensation.
- Do not change Tethers language, OCaml planner, manifest format, or MCP protocol.
- Do not push, merge, tag, or publish unless Matthew explicitly authorises it.

## Stop conditions

Stop with exact evidence and one smallest unresolved question when:

- the authoritative design conflicts with accepted J03/J04a behaviour;
- the real host orchestration seam cannot be exercised even with a dedicated
  honest test capability/binding fixture without changing an unapproved trust
  boundary;
- two materially similar implementation attempts fail;
- the safety reference and current code cannot be reconciled without importing
  unrelated or J07 work;
- an unrelated repository or environment failure prevents trustworthy
  verification;
- a preflight repair would alter semantics, weaken a safety control, conceal
  unexpected source changes, or risk losing work.

Do not stop merely because the existing structured-scope demo correctly denies
before Ask. Use the authorised fixture route, preserve the demo's fail-closed
behaviour, prove both paths, and continue.

Do not stop merely because a packet heading, base SHA, worker-note field, ignored
evidence file, or equivalent mechanical control-plane detail is stale or
malformed. Repair it, prove the gate, and continue.

## Expected pre-existing changes

None. Start from a clean fresh branch created from current `origin/main` at or
after `d7962642fc85a433a7d4257de73a9f2417f4418f`.