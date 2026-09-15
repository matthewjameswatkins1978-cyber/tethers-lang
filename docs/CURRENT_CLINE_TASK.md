# TETHERS x RESOLVE01 - P1 Preparation Proof and ScopeKey Foundation

Task: `TETHERS x RESOLVE01 / P1`

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Codex`

Route: `Implement the Rust-host preparation evidence substrate frozen by P0. Add controlled GuardPreparationProof and opaque ScopeKey types, deterministic current-source evidence, fresh reconstruction, exact comparison, bounded mismatch reasons, and the ResolveGuardRequired projection. Do not contact Resolve or change Tethers product semantics.`

Base commit: `66e8360a7247b60ccc8fb6cb447f796b922afb42`

Implementation checkpoint: `6be53ea`

Worker note: `docs/worker-notes/2026-09-15-tethers-resolve-p1.md`

Suggested branch:

`codex/tethers-resolve-p1-preparation-proof`

## Objective

Produce deterministic, machine-verifiable evidence of the Tethers capability,
manifest, provider, validated arguments, resolved scope, binding, and current
policy/approval state needed by a future Resolve guard request. Preparation is
evidence only; it grants no permission, performs no provider work, and does not
implement guard admission.

## Relevant background and existing behaviour

P0 is accepted at `66e8360a7247b60ccc8fb6cb447f796b922afb42`. The Rust host
already owns capability resolution, verified manifest/provider binding,
validated arguments, binding-owned scope assessment, effective policy, fresh
approval checks, replay, durable intent, provider invocation, outcomes, Trail,
and Result Anchors. Existing `approval::digest` is the repository JCS/SHA-256
authority. Current safe scope forms are PathPrefix and Unrestricted; other
manifest scope forms are not established for this integration. P1 adds only
preparation evidence over those existing authorities.

## Required behaviour

1. Add a controlled `GuardPreparationProof` containing every P0-required field:
   format version, evaluation/plan/action identity, capability identity,
   argument/manifest/provider identity, resolved scope digest, opaque ScopeKeys,
   and binding digest.
2. Add an opaque, validated `ScopeKey` whose deterministic projection is
   derived only from already resolved binding-owned Tethers scope.
3. Support the current safe `PathPrefix` and explicit `Unrestricted` forms;
   refuse `Repository`, `Calendar`, and `ScopeNotEstablished` without hashing
   raw input or downgrading to unrestricted.
4. Reuse the existing JCS and SHA-256 digest authority. Do not create a second
   canonicalisation or hashing stack.
5. Add `ResolveGuardRequired` containing only preparation identity, Tethers
   action identity, and opaque ScopeKeys.
6. Expose a bounded host-owned preparation route after exact capability,
   manifest/provider, scope, policy, and approval checks. It must not dispatch,
   touch replay, record a provider outcome, or write a Result Anchor.
7. Reconstruct fresh evidence from current host state and compare every
   material proof field exactly. Any drift returns a typed refusal reason.
8. Keep raw arguments, credentials, raw scope values, Resolve state, and policy
   internals out of proof and Resolve-facing projections and diagnostics.
9. Add focused deterministic, adversarial, drift, unsupported-version, and
   zero-provider-invocation tests.
10. Document the actual implementation contract for P2.

## Frozen decisions and invariants

- Tethers remains the authority for capability, manifest, provider, argument,
  scope, binding, trust, policy, and approval truth.
- Preparation evidence is never permission and never invokes a provider.
- ScopeKeys are opaque equality dimensions; Resolve must not interpret them.
- Fresh reconstruction reruns current Tethers authorities and exact comparison
  refuses any material drift; old evidence is never repaired or widened.
- P1 has no live Resolve boundary, database, wire protocol, Core change, syntax
  change, replay change, provider execution change, or outcome change.

## Acceptance criteria

1. Rust host only: no OCaml/Core, Tether syntax, Plan schema, replay,
   provider-execution, outcome, or Resolve transport/database changes.
2. Proof construction is controlled and cannot be freely fabricated from raw
   strings by external callers.
3. ScopeKeys are versioned, opaque, sorted, duplicate-free, deterministic, and
   based only on resolved PathPrefix/Unrestricted scope.
4. Missing/unsupported scope and malformed/future proof versions fail closed.
5. Argument, manifest, provider, binding, scope, capability, action, plan, and
   evaluation drift are individually refused; unchanged reconstruction matches.
6. Existing JCS/SHA-256 authorities are reused and field authorities are
   recorded in the architecture note.
7. Preparation and reconstruction demonstrably invoke zero target providers.
8. Focused P1 tests, relevant Rust/host tests, repository verification,
   task-packet checker, secret scan, documentation checks, and `git diff --check`
   pass from the task branch.
9. The final worker note records the starting SHA, checkpoint, files, tests,
   decisions, unresolved questions, and stop conditions.
10. The architecture note accurately records the P1 implementation and the
    exact bounded input contract handed to P2.

## Relevant components

- `tethers-0.1/host-rust/src/resolve_guard.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/src/configured_runtime.rs`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P1_PREPARATION_PROOF.md`
- `docs/worker-notes/2026-09-15-tethers-resolve-p1.md`

## Required verification

Run the repository-owned tool diagnostic, task-packet checker, `cargo fmt
--all -- --check`, focused P1 tests, relevant host/Rust tests, `just verify`,
secret scan, documentation/link checks, `git diff --check`, complete diff/path
inspection, and final clean-status checks. Do not inherit P0 evidence.

## Forbidden changes

- No OCaml/Core/parser/AST/evaluator/Plan or syntax changes.
- No Resolve network, wire protocol, database, guard-admission trait, or live
  adapter.
- No provider invocation, replay mutation, Trail guard event, Result Anchor
  taxonomy change, policy redesign, approval bypass, or new scope form.
- No raw secrets or arbitrary raw argument/scope values in evidence.
- No force-push, history rewrite, direct main update, or unrelated cleanup.

## Stop conditions

Stop and report if binding digest cannot be defined from one existing semantic
authority, if safe scope projection is unavailable, if Core changes or a new
semantic authority are required, if fresh policy/approval cannot be rerun, or if
P1 starts becoming a generic coordination framework. After two materially
similar failed implementation attempts against one design issue, stop with the
exact evidence and smallest unresolved question.

## Expected pre-existing changes

None

## Implementation scope

- `tethers-0.1/host-rust/src/resolve_guard.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/src/configured_runtime.rs`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P1_PREPARATION_PROOF.md`
- `docs/worker-notes/2026-09-15-tethers-resolve-p1.md`
