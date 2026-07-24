# Current Implementation Task

Control contract: `1`

Task: `J04a effective-policy fail-closed correction`

Status: `ACCEPTED`

Task colour: `Amber`

Owner: `Codex`

Route: `Codex — one bounded correction in the current checkout`

Worker note: `docs/worker-notes/2026-07-24-j04a-effective-policy-correction.md`

Base branch: `main`

Base commit: `643c6ed40e3e8a167afd53eca2c98597c0aa8f24`

## Objective

Correct the two demonstrated J04 fail-open paths without changing the frozen
J03/J03a/J03b policy contract: reject a Plan whose non-empty manifest digest
does not match the current verified manifest, and prevent the reference host
from asserting `within_scope` when no host/binding assessor exists.

## Relevant background and existing behaviour

- J04's new `evaluate_effective_policy()` checks only that `manifest_digest`
  is present and non-empty, then resolves by name/version/provider. It never
  compares the Plan pin with `ResolvedCapability::manifest_digest()`.
- J03 requires a revoked, stale, or mismatched manifest digest to yield
  `unavailable` before dispatch.
- The reference host demo manifest declares structured `path_prefix` scope but
  its Action has `project` and `task` arguments. `main.rs` explicitly states
  that no binding-specific assessor exists, yet passes
  `ScopeAssessment::WithinScope`, allowing dispatch.
- J03b defines that exact situation as `scope_not_established`, which must
  deny before any local Allow or Ask. It deliberately defers any
  argument-to-scope mapping to a later binding/adapter task.

## Required behaviour

1. After successful live resolution, compare the Action's required
   `manifest_digest` pin byte-for-byte with the resolved verified manifest
   digest. A mismatch returns `Unavailable` with a distinct inspectable reason
   before schema, scope, local-policy, dispatch, Trail, or executor work.
2. Replace the demo's unsupported `ScopeAssessment::WithinScope` assertion
   with `ScopeAssessment::ScopeNotEstablished` while no concrete
   host/binding-specific assessor exists. The structured-scope demo must
   therefore deny and make no dispatch attempt; adjust only its direct test
   expectation to reflect this fail-closed outcome.
3. Add focused regressions for both paths, including a non-empty stale digest
   under an exact local Allow and a structured-scope host execution with no
   assessor. Existing declared, verified, schema-valid `WithinScope` unit
   cases must continue to reach their expected Allow/Ask outcomes.

## Relevant components

- `tethers-0.1/host-rust/src/policy.rs`
- `tethers-0.1/host-rust/src/main.rs`
- `tethers-0.1/host-rust/src/resolver.rs`
- `tethers-0.1/scripts/demo.ps1`
- `tethers-0.1/scripts/test-host-execution-failure.ps1`
- `docs/DECISIONS.md` (J03/J03a/J03b)
- `docs/CAPABILITY_BRIDGE.md` (manifest pin and scope enforcement)

## Frozen decisions and invariants

- The four outcomes and J03/J03a precedence remain unchanged. A stale or
  mismatched manifest digest is `Unavailable`; it is not a local-policy Deny.
- A structured scope without a host/binding-owned assessment is
  `ScopeNotEstablished` and denies. Do not infer `path`, `repository`,
  `calendar`, `project`, or any other argument convention.
- Do not change manifest format, Tethers/OCaml protocol, dispatch proof
  boundary, executor/provider behaviour, approval/resume logic, or Trail
  semantics.
- J05 remains unauthorised.

## Acceptance criteria

1. A focused policy test changes only a valid Action's non-empty digest and
   proves `Unavailable` with the new digest-mismatch reason despite exact
   local Allow; the existing valid matching-digest path still succeeds.
2. A focused host-level check proves the reference host's structured-scope
   demo with no assessor is denied before `prepare_and_record()` and makes
   zero executor calls; the test does not introduce argument mapping.
3. `cargo fmt --check`, the complete Rust suite, fixtures, engine, MCP
   transcripts, host denial/execution-failure coverage, the corrected demo
   expectation, OCaml build, packet checker, whitespace check, complete diff,
   and final Git status all pass or are reported precisely.

## Required verification

Run sequentially from `tethers-0.1`:

```powershell
Set-Location host-rust; cargo fmt --check; cargo test; Set-Location ..
pwsh -NoProfile -File scripts/check-fixtures.ps1
pwsh -NoProfile -File scripts/test-engine.ps1
pwsh -NoProfile -File scripts/test-mcp-transcripts.ps1
pwsh -NoProfile -File scripts/test-host-denial.ps1
pwsh -NoProfile -File scripts/test-host-execution-failure.ps1
pwsh -NoProfile -File scripts/demo.ps1
Set-Location engine-ocaml; opam exec -- dune build; Set-Location ..
```

Then from the repository root:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
git diff --check
git status --short
```

## Forbidden changes

- No argument-to-resource mapping or real scope-assessor implementation.
- No Ask approval-token creation, consumption, or resume behaviour.
- No dispatch, executor, provider, protocol, fixture, OCaml-engine, manifest,
  or Trail redesign.
- No commit, push, merge, amend, tag, or publication.

## Stop conditions

Stop if changing the reference demo from completion to safe denial would
require an unapproved scope-mapping semantic decision, if digest comparison
cannot reuse the verified resolver result without changing dispatch, or if any
unrelated failure prevents trustworthy verification. Record exact evidence and
one smallest unresolved question.

## Expected pre-existing changes

- `docs/CAPABILITY_BRIDGE.md`
- `docs/DECISIONS.md`
- `docs/TASK_QUEUE.md`
- `docs/worker-notes/2026-07-24-j03-four-outcome-policy-contract.md`
- `docs/worker-notes/2026-07-24-j03a-one-shot-approval-correction.md`
- `docs/worker-notes/2026-07-24-j03b-scope-assessment-boundary.md`
- `docs/worker-notes/2026-07-24-j04-codex-review.md`
- `docs/worker-notes/2026-07-24-j04-effective-policy-resolution.md`
- `tethers-0.1/host-rust/src/main.rs`
- `tethers-0.1/host-rust/src/policy.rs`
- `tethers-0.1/host-rust/src/validation.rs`
- `tethers-0.1/scripts/demo.ps1`
- `tethers-0.1/scripts/test-host-execution-failure.ps1`
