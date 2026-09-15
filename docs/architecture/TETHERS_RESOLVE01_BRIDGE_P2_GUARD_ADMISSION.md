# Tethers x Resolve01 P2 — Guard Admission Boundary

Status: accepted on main at PR #30 merge

P2 adds one Rust-host seam for a future Resolve guard decision. It does not add
a Resolve transport, database, public capability/Plug, outcome delivery,
retry, recovery controller, or any Core/Tether-language change.

## Boundary

The existing Tethers order remains authoritative:

```text
current resolution, schema, scope, policy and approval
  -> replay admission
  -> durable DispatchReadyAction intent
  -> admit_guard
  -> deadline and replay G1
  -> existing provider executor and outcome path
```

The ordinary execution path remains the default. A caller must explicitly use
the guarded host seam; provider, manifest, Plug and Tether data cannot enable
it. The strict `tethers.project/1` configuration is unchanged.

## Types and authority

`ResolveGuardRef`, `ResolveGuardAdmission`, `ResolveGuardAdapter`, and
`GuardAdmissionContext` live in `tethers-0.1/host-rust/src/resolve_guard.rs`.
The ref is bounded opaque input and is represented in diagnostics and durable
evidence only by the existing `approval::digest` JCS/SHA-256 authority. Its
value has no P2 semantics.

The adapter receives one private-field `ResolveGuardAdmissionRequest` only
after replay admission has produced the real host `ExecutionId`. Its semantic
projection is exactly the Resolve R0 boundary: opaque guard ID, opaque action
reference, and opaque sorted duplicate-free ScopeKeys. The preparation digest
and planner `ActionId` remain host evidence used to build and validate the
request; they are not Resolve concepts and do not cross this adapter boundary.
Adapter errors map to `Indeterminate`; there is no fourth adapter state.

`ActionId` names the planned logical action. `ExecutionId` names the
host-admitted execution. Resolve sees `ExecutionId` as its opaque `action_ref`
at guard admission; P4 outcome delivery uses that same identity.

P1 preparation is rebuilt from current Tethers-owned authorities before a
guarded request is formed. Explicit resume uses P1 exact reconstruction and
refuses any material drift. The old proof is comparison input only.

## Execution and evidence

The guarded shared boundary asks the adapter only after replay admission and
durable intent. `Admitted` continues into the existing deadline/G1/provider
boundary. `Rejected` and `Indeterminate` stop before G1 and provider invocation
and are represented by distinct host outcomes, not Tethers policy denial or
provider failure/uncertainty. Existing `ExecutionOutcome` and Result Anchor
taxonomies remain unchanged.

Because P2 freezes the existing replay state machine, a crash or durable
failure after replay intent but before G1 remains the existing
`IntentRecorded`/manual-resolution recovery state. P2 does not retry the guard,
invent a terminal provider outcome, or silently clear that state. This is
deliberate fail-closed recovery, not a claim that a provider ran.

Guard decisions are durably recorded through the existing `Trail`/`FileTrail`
authority as a bounded `GuardAdmissionEntry` containing execution/action IDs,
preparation digest, guard-reference digest, and the closed decision. No raw
guard reference, argument, path, credential, manifest or Resolve state is
recorded. A Trail failure stops the request as an audit failure before provider
invocation.

The same ordering is available to the Together preparation seam: admission is
performed during serial preparation, before the coordinator can arm G1 or
launch any worker. P2 does not introduce batch or transaction semantics.

## Explicit exclusions and handoff

P2 does not implement live `admit_guard` transport, Resolve Claim/Commitment/
epoch/lease state, provider outcome delivery, automatic retries, crash recovery
or a Resolve-side policy. After P2, the conceptual handoff is:

```text
ResolveGuardRequired
  -> future S3-frozen adapter transport
  -> Admitted | Rejected | Indeterminate
```
