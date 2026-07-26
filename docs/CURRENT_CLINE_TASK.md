# Current Implementation Task

Control contract: `1`

Task: `J09 durable replay protection`

Status: `PROPOSED`

Task colour: `Red`

Owner: `Codex`

Route: `Codex — later Red implementation branch from current origin/main after Lucy authorisation`

Worker note: `docs/worker-notes/2026-07-26-j09-durable-replay-implementation.md`

Base branch: `main`

Base commit: `15e6f8a4dab7fe6c1c4a21266f9fe826e69a51b7`

Base rationale: this is the amended J09 host-data-root and native Windows
substrate design checkpoint on the review branch. It supersedes the earlier
native-substrate checkpoint `d67771ff2d93e7fe0909835e13c0988fa10a0c18`; the
current packet commit is its planning-only descendant. Neither is a runtime
baseline. The accepted runtime implementation base remains `main` at
`e679338e2887510d907d3b1c77eaf7a922dfad37`, as recorded by the authoritative
J09 design. A later implementation branch starts from that accepted `main`,
with this corrected design checkpoint as its reviewed authority.

## Objective

Implement the frozen J09 host-owned durable replay ledger so an execution identity can never repeat an external effect after restart. This is design-ready only; `PROPOSED` does not authorise implementation.

## Relevant background and existing behaviour

- `docs/J09_DURABLE_REPLAY_DESIGN.md` is the sole J09 authority.
- J05 approval consumption and J06 deadline, truthful outcome, redaction, and Result Anchor semantics are accepted on `main`.
- Historical J07/J08 outcomes are absorbed into J06; do not create separate J07/J08 work.
- Existing intent/outcome Trail is evidence, not replay-admission authority.
- The host-local `--host-data-root` option and separate `provision-replay`
  operation are the sole J09 storage authority; `TRAIL_PATH` remains independent
  audit storage and is never a replay-root fallback.
- The preserved safety branch is out of scope and must not be inspected for implementation material.

## Required behaviour

1. Create one host-owned canonical stable execution identity per attempt.
2. Bind identity to the exact non-secret dispatch proof fields.
3. Persist replay state in the host-owned versioned ledger specified by J09.
4. Fail closed for missing, corrupt, partial, unreadable, or unprovable persistence.
5. Publish `intent_recorded` before Trail intent and provider invocation.
6. Publish `invocation_armed` before the provider invocation boundary.
7. Block every duplicate identity before and after possible provider invocation.
8. Persist J06 known success, known failure, and uncertainty as final replay states.
9. Require manual resolution for incomplete and uncertain states.
10. Create no duplicate standard Result Anchor on any replay block.
11. Preserve J05 consumption and J06 truth; add no retry or compensation.
12. Provide deterministic persistence and clock-related test seams.
13. Use only the frozen native Windows NTFS substrate and fail closed before
    approval consumption or dispatch when its storage proof cannot be made.

## Relevant components

- `docs/J09_DURABLE_REPLAY_DESIGN.md`
- `docs/J05_EXACT_ASK_APPROVAL_DESIGN.md`
- `docs/J06_DEADLINE_OUTCOME_DESIGN.md`
- `tethers-0.1/host-rust/src/dispatch.rs`
- `tethers-0.1/host-rust/src/main.rs`
- `tethers-0.1/host-rust/src/outcome.rs`
- `tethers-0.1/host-rust/src/result_anchor.rs`
- focused new host replay-ledger and target-specific Windows persistence modules
- `tethers-0.1/host-rust/Cargo.toml` and `Cargo.lock` for the only authorised
  target-specific dependencies

## Frozen decisions and invariants

- The J09 design controls replay semantics.
- Completed success and known failure are permanently replay-blocked.
- Incomplete and uncertain identities are never automatically retried.
- A new attempt needs a new execution identity; no consumed J05 approval is restored.
- Replay persistence failure fails closed at startup and lookup.
- Runtime persistence supports only local fixed NTFS after handle-bound,
  reparse-safe verification; all other storage is unavailable before approval
  consumption or dispatch.
- An Allow or approved Ask requires an absolute, already-provisioned host data
  root; missing, incomplete, unproven, or relative roots fail before J05
  consumption and dispatch. Normal execution never provisions replay storage.
- Provisioning alone may create the exact v1 replay subtree after handle-bound
  local-NTFS, owner, and DACL validation; it is narrowly idempotent and never
  repairs or rewrites operator storage or ACLs.
- `windows-sys` with the frozen Foundation, Storage_FileSystem, System_IO,
  Security, and System_Threading features, plus `uuid` v4, are authorised only
  as specified by the frozen J09 design; no other dependency is authorised.
- Result Anchor queueing is J10 work and remains absent.
- No planner, manifest, protocol, provider-contract, or safety-branch change is permitted.

## Acceptance criteria

1. Tests prove canonical host-created identities and binding mismatch rejection.
2. Tests prove a host-owned ledger stores only redacted bound data.
3. Tests prove record publication is complete, durable, and non-replacing.
4. Tests prove every persistence read/validation/write/flush/publish failure fails closed.
5. Tests prove intent ledger state precedes Trail intent and zero calls follow failure.
6. Tests prove armed state precedes every possible provider call.
7. Tests prove duplicate attempts make zero calls in each ledger state.
8. Tests prove final state ordering for J06 success, failure, and uncertainty.
9. Tests prove restart reconstruction blocks incomplete/uncertain states for manual resolution.
10. Tests prove replay blocks create no duplicate standard Result Anchor.
11. Tests prove J05 approval remains consumed and no retry/compensation exists.
12. Tests prove deterministic fault and clock seams cover the numbered J09 matrix.
13. Native Windows tests prove storage admission, reparse rejection,
    cross-process exclusion, no-replace publication, flush/reopen fault closure,
    and documented containment of unsafe Win32 calls.
14. Tests prove missing or relative host-data-root fails before J05 consumption
    and dispatch, while unmatched, denied, and pending-Ask paths remain
    provider-free without replay storage.
15. Tests prove only explicit provisioning creates the exact complete v1
    hierarchy, exact reprovisioning is non-mutating `AlreadyProvisioned`, and
    partial or unknown storage is never repaired.
16. Tests prove handle-bound root owner and DACL validation accepts only the
    frozen authority set and rejects reparse substitution or unprovable ACLs.
17. The worker note maps every numbered J09 verification case, including the
    host-data-root and provisioning cases, to focused evidence or an existing
    regression.

## Required verification

- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`
- `Set-Location tethers-0.1/host-rust; cargo fmt --check; cargo test`
- relevant native PowerShell host integration checks named by implementation evidence
- `git diff --check`
- inspect complete diff and final `git status --short`

## Forbidden changes

- No automatic retry, compensation, recovery execution, approval restoration, or event queueing.
- No J10/J11 work, planner/OCaml, manifest, MCP protocol, provider contract,
  install, merge, or safety-branch change.
- No dependency except the design-authorised target-specific `windows-sys` 0.61
  feature set and `uuid` v4 feature; no other dependency is authorised.
- No public Tethers language configuration change. The only permitted
  reference-host configuration addition is `--host-data-root <ABSOLUTE_PATH>`
  and the only permitted storage establishment operation is
  `provision-replay <ABSOLUTE_HOST_DATA_ROOT>`; neither may infer a root from
  Trail, request, environment, provider, temporary, executable, or working
  directory paths.
- No raw secret, argument, payload, path, stderr, or stack diagnostic in durable replay/audit/Anchor data.

## Stop conditions

Stop for a semantic, trust, security, atomic-durability, or platform-primitive conflict; do not substitute best-effort persistence. Stop after two materially similar failures with exact evidence and one smallest unresolved question.

## Expected pre-existing changes

None.
