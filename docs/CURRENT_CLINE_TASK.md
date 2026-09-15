# TETHERS x RESOLVE01 - P2 Guard Admission Boundary

Task: `TETHERS x RESOLVE01 / P2 - Guard Admission Boundary`

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Red`

Owner: `Codex`

Route: `Implement the bounded Rust-host guard-admission seam over the accepted P1 preparation proof. Preserve Tethers as the sole authority for resolution, scope, policy, approval, replay, durable intent, execution and outcomes. Resolve may only admit or refuse a request and may never grant permission. P2 ends before any live Resolve transport or outcome delivery.`

Base commit: `fab258663af363b6e5902ea3bbe5d3dd8de61678`

Implementation checkpoint: `fab258663af363b6e5902ea3bbe5d3dd8de61678`

Worker note: `docs/worker-notes/2026-09-15-tethers-resolve-p2.md`

Suggested branch:

`codex/tethers-resolve-p2-guard-admission`

## Objective

Add one narrow, non-transport guard-admission boundary to the Rust host. The
route must rerun current Tethers preparation authorities, record durable intent,
ask Resolve one closed admission question, then continue through the existing
deadline, replay G1, provider, outcome, Result Anchor and Trail path only when
Resolve returns `Admitted`. Preparation and admission are evidence/coordination;
neither grants Tethers permission.

## Relevant background and existing behaviour

P0 and P1 are accepted. `GuardPreparationProof`, `ResolveGuardRequired`,
opaque `ScopeKey`, exact fresh reconstruction, current capability/schema/scope/
policy/approval checks, replay authority, `DispatchReadyAction`, existing
provider execution, outcomes, Result Anchors and FileTrail already exist in the
Rust host. The P1 proof is not authority and must be rebuilt before each guarded
request.

## Required behaviour

1. Add an opaque validated `ResolveGuardRef`; raw values never enter Trail,
   logs, diagnostics, anchors or Resolve-facing evidence.
2. Add exactly one narrow adapter trait with the closed result set:
   `Admitted`, `Rejected`, `Indeterminate`. Adapter errors map to
   `Indeterminate` unless a host configuration error is detected before the
   adapter boundary.
3. The request contains only the opaque guard reference, P1 proof identity/
   `ResolveGuardRequired`, opaque `ScopeKey`s, and the existing Tethers action
   identity/reference. No raw arguments, paths, manifests, policy internals or
   Resolve state cross the seam.
4. Guarded execution is explicitly host-selected and disabled by default.
   Provider/Plug/manifest/Tether data cannot enable it. Do not mutate the frozen
   `tethers.project/1` schema unless a separately authorised compatibility
   decision makes that safe.
5. Preserve this order: current Tethers gates, replay admission, durable intent,
   guard admission, deadline, replay G1, existing executor/provider and outcome.
   Durable-intent failure means zero adapter/provider calls.
6. Denied, unavailable, unresolved Ask, invalid action/schema, scope refusal,
   stale P1 proof, replay refusal and missing adapter never call the adapter.
7. `Rejected` and `Indeterminate` stop before deadline/G1/provider, produce
   bounded typed host/Trail evidence, and are not relabelled as Tethers DENY or
   provider FAILED/UNCERTAIN. No automatic retry.
8. Together preparation and durable intent remain serial and deterministic. All
   members must be admitted before any group provider invocation; any refusal or
   indeterminate result yields zero group provider calls.
9. Preserve existing replay, provider execution, outcome, Result Anchor, Plan,
   Core and syntax semantics. No live Resolve transport/database or public Plug.

## Frozen decisions and invariants

- Tethers remains the authority for capability, manifest, provider, argument,
  scope, binding, trust, policy, approval, replay, durable intent and outcome.
- Resolve coordinates only the external guard question. It cannot grant
  permission, alter Tethers policy, invoke a provider or deliver outcomes.
- P1 preparation is rebuilt from current Tethers state before each guard call;
  stale evidence is refused exactly and never repaired or widened.
- The existing execution seam, replay ledger, FileTrail and Result Anchor
  taxonomy remain the only accepted authorities.
- P2 has no live Resolve transport, database, public Plug, retry, recovery
  controller, Core change, syntax change or new product semantics.

## Acceptance criteria

1. Controlled opaque ref and one closed adapter seam exist.
2. P1 freshness is mandatory and current policy/approval cannot be bypassed.
3. Durable intent precedes every adapter call; adapter/provider zero-call refusal
  and failure paths are directly tested.
4. Admitted serial and Together paths reach existing execution exactly once;
  Rejected/Indeterminate paths reach no provider and no G1.
5. Invalid refs, secret leakage, disabled mode, missing adapter, stale proof,
  crash/recovery boundaries, Trail compatibility and config compatibility are
  tested.
6. No HTTP/MCP/Socket/SQLite Resolve implementation, outcome delivery, retry,
  recovery controller, Core change, syntax change or new Result Anchor class.
7. Focused P2, relevant Rust/host, formatting/checks, task checker, secret,
  documentation, whitespace, and repository-authoritative verification pass.
8. Architecture note and worker note record the actual implementation and
  explicit P2 exclusions.
9. The final branch is reviewed, normally published to the task remote branch,
   and leaves the canonical main checkout unaffected until acceptance.

## Relevant components

- `tethers-0.1/host-rust/src/resolve_guard.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/host-rust/src/plan_execution.rs`
- `tethers-0.1/host-rust/src/dispatch.rs`
- `tethers-0.1/host-rust/src/runtime_config.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P2_GUARD_ADMISSION.md`
- `docs/worker-notes/2026-09-15-tethers-resolve-p2.md`

## Required verification

Run the repository-owned diagnostics, task checker, focused P2 tests, relevant
serial/Together/recovery/Trail/config tests, `cargo fmt --all -- --check`,
`cargo check --all-targets --all-features --locked`, relevant Rust/host suites,
MCP and compatibility checks, warning ratchet, secret scan, documentation/link
checks, `just verify`, `git diff --check`, complete diff review and final clean
status. Record commands not run and exact first failures.

## Forbidden changes

- No OCaml/Core/parser/AST/evaluator/Plan/syntax/policy vocabulary changes.
- No Resolve HTTP/MCP/Socket protocol, SQLite/database, scheduler, retry,
  renewal, redelivery, outcome-delivery or recovery-controller implementation.
- No public Resolve capability/Plug, new scope form, replay semantic change,
  provider ordering change, provider outcome taxonomy change or Result Anchor
  taxonomy change.
- No raw guard refs, credentials, paths or arguments in evidence/diagnostics.
- No force push, history rewrite, direct main update or unrelated cleanup.

## Stop conditions

Stop and report if the binding/intent authority is ambiguous, Trail cannot
accept bounded guard evidence without a schema decision, guarded mode cannot be
represented without silently changing `tethers.project/1`, the current host
would auto-call Resolve/provider after a crash without explicit fresh admission,
or implementation begins requiring transport, database, provider execution, Core
changes or a generic coordination framework. After two materially similar failed
approaches to the same design issue, stop with exact evidence and one smallest
unresolved question.

## Implementation scope

The intended bounded scope is the Rust host guard/application/execution seams,
their focused tests, one architecture note, this task packet, and the named
worker note. Do not broaden the packet checker to repository-wide inference.

## Expected pre-existing changes

None.
