# Current Implementation Task

Control contract: `1`

Task: `J06 monotonic deadline and truthful outcome classification`

Status: `READY`

Task colour: `Red`

Owner: `Codex`

Route: `Codex — fresh implementation branch from current origin/main`

Worker note: `docs/worker-notes/2026-07-25-j06-deadline-outcome-implementation.md`

Base branch: `main`

Base commit: `1f984ff3c89c66b5580e8b6e7936b8e41d9db93d`

## Objective

Implement the authoritative J06 deadline and outcome boundary defined in
`docs/J06_DEADLINE_OUTCOME_DESIGN.md`.

J06 must use monotonic timing, distinguish unattempted, known success, known
failure, and uncertain execution truthfully, preserve J05 one-shot approval
consumption, redact durable reasons, and introduce no retry or J07 behaviour.

## Relevant background and existing behaviour

- J05 is accepted and merged on `main` at
  `1f984ff3c89c66b5580e8b6e7936b8e41d9db93d`.
- Durable intent already precedes provider execution.
- Existing Result Anchors represent known success and known failure.
- Existing unattempted paths create no standard Result Anchor.
- `docs/J06_DEADLINE_OUTCOME_DESIGN.md` is the sole J06 authority.
- `docs/J06_DEADLINE_OUTCOME_DESIGN_CANDIDATE.md` is provenance only.
- The immutable safety branch
  `safety/preserve-local-main-20260725` at
  `f74999aba9135f0493cf28693ba6444c22388294` may be inspected selectively, but
  its partial J07-style runtime code is rejected and must not be transplanted.

## Preflight self-repair authority

Codex must repair bounded mechanical control-plane defects itself and continue
when the correction does not change product semantics, trust boundaries,
security, implementation scope, or accepted architecture.

Codex may:

- update the Base commit to current clean `origin/main` before implementation
  starts when intervening remote commits are already accepted;
- correct packet formatting, status, headings, worker-note metadata, ignored
  local evidence files, and equivalent checker-contract defects;
- make the smallest checker correction when it rejects semantically equivalent
  packet formatting;
- record every self-repair in the worker note.

Codex must stop when repair would alter J06 semantics, weaken a safety boundary,
conceal source drift, risk losing work, or import unrelated code.

## Required behaviour

1. Add a host-owned monotonic clock abstraction for execution deadlines.
2. Add a deterministic controllable clock for tests.
3. Start the deadline only after durable intent has succeeded.
4. Keep planning, approval waiting, approval consumption, policy evaluation, and
   failed intent persistence outside the execution deadline.
5. Mark the provider invocation boundary immediately before a valid
   `DispatchReadyAction` may cause external effects.
6. Classify failures before that boundary as `Unattempted`.
7. Classify trusted provider success with valid output as `Succeeded`.
8. Classify explicit provider-declared errors as known `Failed`.
9. Classify trusted provider success with schema-invalid output as known
   `Failed` with `result_validation_failed`.
10. Classify deadline expiry or transport ambiguity after invocation may have
    begun as `Uncertain`, never guessed `Failed`.
11. Cover process loss, malformed/truncated responses, protocol interruption,
    and absence of trustworthy final evidence.
12. Use the deterministic rule that a response first observed after the
    monotonic deadline is `Uncertain`.
13. Durably persist attempted outcomes before creating their standard Result
    Anchor.
14. Add `capability.uncertain` without weakening the existing success/failure
    Anchor contracts.
15. Produce no standard Result Anchor for `Unattempted`.
16. Preserve a known or uncertain in-memory classification after outcome-audit
    failure, report audit failure separately, create no standard Result Anchor,
    and authorise no retry.
17. Add a pure explicit redaction boundary for durable outcome and Result Anchor
    reasons.
18. Prevent raw stderr, transport payloads, paths, credentials, tokens,
    arguments, stack traces, and provider-private messages from crossing durable
    boundaries.
19. Preserve J05 approval consumption after every later outcome or audit path.
20. Add no automatic retry, implicit compensation, restart replay, or J07
    recovery behaviour.
21. Implement all 48 design cases with individually identifiable tests or an
    explicit one-to-one mapping.

## Relevant components

- `docs/J06_DEADLINE_OUTCOME_DESIGN.md`
- `docs/J05_EXACT_ASK_APPROVAL_DESIGN.md`
- `docs/DECISIONS.md`
- `docs/CAPABILITY_BRIDGE.md`
- `tethers-0.1/host-rust/src/dispatch.rs`
- `tethers-0.1/host-rust/src/main.rs`
- `tethers-0.1/host-rust/src/result_anchor.rs`
- `tethers-0.1/host-rust/src/provider.rs`
- `tethers-0.1/host-rust/src/stdio_provider.rs`
- focused new clock/outcome/redaction modules when justified
- existing Rust and integration tests
- safety commit `f74999aba9135f0493cf28693ba6444c22388294`
  for selective design reference only

## Frozen decisions and invariants

- The authoritative J06 design controls all deadline and outcome semantics.
- Deadline decisions use monotonic time only.
- Wall-clock timestamps may label events but never decide classification.
- Deadline starts after durable intent and before provider invocation.
- Before invocation is unattempted.
- Ambiguity after invocation may have begun is uncertain.
- Explicit provider failure is known failed.
- Schema-invalid successful output is known failed.
- Known outcomes remain known after later audit failure.
- Uncertain outcomes remain uncertain after later audit failure.
- No outcome-write failure authorises retry.
- No automatic retry or compensation.
- No consumed J05 approval is restored.
- No standard Result Anchor for unattempted Actions.
- Durable reasons must be redacted and stable.
- Tethers Core, OCaml planner semantics, manifest format, and MCP protocol remain
  unchanged.
- The existing structured-scope demo remains fail-closed.
- J07 is entirely out of scope.

## Acceptance criteria

1. Focused tests prove the deadline starts only after durable intent.
2. Production deadline decisions use a monotonic clock abstraction.
3. Tests inject and control a deterministic clock.
4. Tests prove the exact invocation boundary and all unattempted cases.
5. Tests prove known success, explicit provider failure, and schema-invalid
   provider success classifications.
6. Tests prove each post-invocation ambiguity becomes `Uncertain`.
7. Tests prove a response observed after deadline remains uncertain.
8. Tests prove durable outcome precedes standard Result Anchor for success,
   failure, and uncertainty.
9. Tests prove outcome persistence failure preserves in-memory truth, creates no
   standard Result Anchor, and authorises no retry.
10. Tests prove durable and Result Anchor reasons are redacted.
11. Tests prove J05 approval remains consumed after every later path.
12. Tests prove no automatic retry or compensation occurs.
13. All 48 design cases have one-to-one evidence.
14. Existing J03-J05 trust and regression tests remain green.
15. The complete required verification passes and the worker note records exact
    commands and results.

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

- Do not modify, delete, merge, cherry-pick, or push the safety branch.
- Do not transplant preserved `main.rs`, J07 code, workflow files, or corrupted
  safety-branch runtime code.
- Do not use wall-clock `SystemTime` for deadline decisions.
- Do not add automatic retry, compensation, or recovery replay.
- Do not restore consumed J05 approvals.
- Do not classify post-invocation ambiguity as known failure.
- Do not create standard Result Anchors for unattempted Actions.
- Do not persist raw uncertain reasons, credentials, arguments, stack traces,
  transport payloads, or provider-private diagnostics.
- Do not change Tethers language, OCaml planner, manifest format, or MCP protocol.
- Do not implement J07.
- Do not push, merge, tag, or publish unless Matthew explicitly authorises it.

## Stop conditions

Stop with exact evidence and one smallest unresolved question only when:

- the authoritative design conflicts with accepted J03-J05 behaviour;
- a monotonic deadline cannot be introduced without changing an unapproved
  protocol or trust boundary;
- truthful uncertainty cannot be represented without an unavoidable protocol
  change;
- two materially similar implementation attempts fail;
- unrelated repository or environment failure prevents trustworthy verification;
- preflight repair would alter semantics, weaken safety, conceal drift, or risk
  losing work.

Do not stop for progress reports, safe increments, a passing focused test, one
completed module, or ordinary compiler/test failures. Continue until the task is
`COMPLETE` or a genuine stop condition exists.

## Expected pre-existing changes

None. Start from a clean fresh branch created from current `origin/main` at or
after `1f984ff3c89c66b5580e8b6e7936b8e41d9db93d`.
