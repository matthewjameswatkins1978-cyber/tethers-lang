# Tethers x Resolve01 - P0 Contract and Seam Audit

Status: P0 working architecture note

Base: `ddc5d0fccfc0a38ff0af3013c5e9ff6ba357e0b2`

This note freezes the Tethers-side boundary for the Resolve01 integration. It
does not authorise runtime implementation, Resolve API assumptions, or a
Resolve01 public capability contract.

## 1. Ownership boundary

The integration has two deliberately separate components:

1. **Resolve Guard Adapter** - trusted Rust-host coordination seam. It carries
   an opaque Resolve guard reference, supplies Tethers-derived opaque ScopeKeys,
   admits the guard immediately before existing provider execution, reports
   durable Tethers outcomes, and records coordination evidence through the
   existing Tethers machinery.
2. **Resolve01 TetherPlug** - ordinary future `.tetherplug` package for public
   Resolve capabilities only after Resolve01 freezes that provider contract.

The guard-admission operation is internal coordination and is never exposed as
an ordinary Tether capability.

The ownership rule is therefore:

```text
Resolve coordinates live commitment ownership
        |
        | opaque execution-guard admission
        v
Tethers resolves, authorises, fences, executes, and classifies
        |
        | authoritative SUCCEEDED / FAILED / UNCERTAIN outcome
        v
Resolve updates its own commitment state
```

Tethers does not interpret Resolve Goal, Commitment, Claim, lease expiry,
Claim epoch, DAG topology, dependency state, worker ownership, attention state,
or recovery policy. Resolve never supplies Tethers permission and never calls
the target provider directly.

## 2. Verified current Tethers seams

| Concern | Existing authority | P0 integration consequence |
| --- | --- | --- |
| Core planning | OCaml Core and its current wire/Plan path | No Resolve syntax, Goal, or guard field enters Core. |
| Exact capability resolution | `host-rust/src/resolver.rs`, consumed by `HostExecutionService::resolve_exact_capability` | Resolve mode uses the same exact name/version/provider/manifest resolution. |
| Scope | `PreparedRuntime::assess_action_scope` in `configured_runtime.rs` | ScopeKeys must be derived from this reviewed binding-owned result, never raw argument names. Current supported safe forms remain bounded. |
| Effective policy | `policy::evaluate_effective_policy` in `policy.rs` | Existing `ALLOW`, `ASK`, `DENY`, `UNAVAILABLE` remain authoritative. |
| Ask approval | `approval.rs` and `request_exact_approval` | Resolve reservation begins only after an existing Ask has been approved and consumed. |
| Replay | `replay_runtime::ReplayAuthority` and `ReplayAdmissionGuard` | A new Resolve guard never resets or bypasses Tethers replay state. |
| Durable intent | `dispatch::prepare_and_record` and `DispatchReadyAction` | The returned readiness proof remains the sole route toward provider execution. |
| Shared execution | `application::execute_shared_boundary` | The guard seam belongs after durable intent and before invocation arming/provider effect. |
| Together groups | `execute_boundary_prepare` plus `execute_boundary_invoke_only` | The same proof boundary applies after group preparation and before coordinator G1/provider invocation. |
| Provider invocation | `CapabilityExecutor`, `InstalledProviderExecutor`, and retained Socket session | No Resolve executor is introduced; the existing provider call is reused unchanged. |
| Provider outcomes | `outcome::ExecutionOutcome` | Preserve SUCCEEDED, FAILED, and UNCERTAIN exactly; guard-admission ambiguity is not provider UNCERTAIN. |
| Trail | `dispatch::Trail` / `FileTrail` | Add bounded guard coordination entries to the existing Trail only; never create a Resolve Trail file. |
| Result Anchors | `result_anchor.rs` and `ResponseResultAnchorWriter` | Keep `capability.succeeded`, `capability.failed`, and `capability.uncertain`. |
| Configuration | `runtime_config.rs` and `run_command.rs` | A later host-owned `resolve_guard_mode` is explicit and defaults disabled; providers cannot enable it. |
| Plug lifecycle | `plug_pack.rs`, `package.rs`, `plug_conform.rs`, installation/enablement modules | Future Resolve01 packages use normal pack/inspect/conform/install/enablement. |

## 3. Existing execution order and insertion point

The current shared boundary performs the following important order:

```text
action/pin/executor validation
  -> non-dispatchable policy stop
  -> replay admission and fresh-state check
  -> replay G0 intent
  -> dispatch::prepare_and_record (durable Tethers intent)
  -> deadline check
  -> replay G1 invocation arming
  -> existing CapabilityExecutor provider call
  -> output/failure/uncertainty classification
  -> durable Tethers outcome and replay G2
  -> Result Anchor
```

For Resolve-managed execution, the future guarded order is:

```text
existing Tethers gates and replay
  -> durable Tethers intent / DispatchReadyAction
  -> Resolve guard admission
  -> existing deadline + G1 invocation arming
  -> existing provider invocation
```

Guard admission must be inserted at the shared post-intent/pre-provider seam.
For serial Actions that seam is in `execute_shared_boundary`. For Together
groups it is between `execute_boundary_prepare` and the existing coordinator
`publish_armed`/`execute_boundary_invoke_only` path. This is one Tethers
boundary expressed through two accepted scheduling paths, not a new executor.

`DENY`, `UNAVAILABLE`, unresolved `ASK`, replay refusal, invalid preparation,
rejected guard admission, and indeterminate guard admission all produce zero
target-provider calls. Guard rejection is coordination-precondition evidence,
not Tethers permission denial. Guard-admission indeterminacy is not provider
UNCERTAIN because the target provider has not been invoked.

## 4. Preparation/resume handshake

### Preparation

After exact capability resolution, action/schema validation, binding-owned scope
assessment, effective policy, and any existing Ask approval have succeeded, the
host creates a versioned `GuardPreparationProof` as evidence. The proof binds:

```text
format_version
evaluation_id
plan_id
action_id
capability_name
capability_version
argument_digest
manifest_digest
provider_identity
resolved_scope_digest
scope_keys
binding_digest
```

The exact Rust representation must use validated newtypes/private
construction where the current idiom supports it, the existing JCS/SHA-256
canonicalisation authority, and field-by-field exact validation. It contains
digests and safe opaque identities, never secret argument values or arbitrary
free text. It does not permit execution.

The caller receives a host-level `ResolveGuardRequired` projection containing
the preparation identity/proof digest, action identity, and opaque ScopeKeys.
The OCaml Plan is not changed.

### ScopeKeys

Tethers owns ScopeKey production. The adapter consumes the already resolved
Tethers scope/target representation and applies a versioned canonical selected
projection followed by the existing JCS/SHA-256 authority. Resolve treats each
result only as an opaque equality key. Multiple independently conflicting
dimensions are represented as a deterministic sorted duplicate-free vector.

The current runtime's safe reviewed scope path is `PathPrefix` with one
configured JSON-pointer binding, plus `Unrestricted`; repository and calendar
forms currently return `ScopeNotEstablished`. P0 does not broaden that support.
If a future scope representation cannot be safely projected, the integration
fails closed rather than deriving a key from argument names, descriptions,
filenames, provider claims, or AI judgement.

### Resume

On resume with an opaque guard reference, the host reconstructs fresh Tethers
resolution, validation, scope, trust, policy, and approval state, then creates
a fresh preparation proof. It compares every proof field and the derived
binding digest exactly. Any change in action/evaluation/plan identity,
arguments, capability/version, manifest, provider, binding, scope, ScopeKeys,
or relevant trust/approval state refuses dispatch. It never repairs or widens
the old proof.

Resolve validates its own Commitment/Claim/worker/epoch/boot/lock state and
returns an opaque `ExecutionGuard`. Those Resolve facts are not carried into
Tethers semantics.

## 5. Guard adapter contract seam

The live wire protocol is intentionally deferred until Resolve S3 freezes it.
P2 may introduce one narrow Rust trait for the genuine external-system/test
boundary, conceptually:

```text
admit_guard(guard_ref, scope_keys, tethers_action_ref)
```

The closed result must distinguish `Admitted`, `Rejected`, and `Indeterminate`.
Only `Admitted` may reach the existing provider boundary. Rejected and
indeterminate results record typed coordination evidence and make zero target
provider calls. No generic transport, plugin, scheduler, retry, or recovery
framework is authorised by this seam.

After the provider outcome is durable, a later bounded delivery seam may call:

```text
record_tethers_outcome(tethers_action_ref, SUCCEEDED | FAILED | UNCERTAIN)
```

Delivery failure is separate coordination evidence and cannot mutate the
already durable Tethers outcome. Automatic redelivery remains disabled until
Resolve proves this operation idempotent for the exact action reference and
outcome.

## 6. Version and lifecycle surface

The current checked-out Tethers contract is:

- Plug package format `1`;
- Socket major `1`;
- MCP binding `2025-11-25` over local stdio;
- capability manifest format `1.0`;
- host project configuration `tethers.project/1` with runtime format `0.1`;
- Trail/receipt `tethers.trail/1`;
- policy vocabulary `allow`, `ask`, `deny`, `unavailable`.

The future Resolve01 Plug must use normal author -> pack -> inspect -> conform
-> host installation/approval/enablement. Pack-generated payload indexes,
manifest digests, semantic package digest, raw archive digest, and hashes are
never hand-authored. Conformance remains evidence only.

No public Resolve capability operation, schema, effect, scope, idempotency,
timeout, confirmation policy, or provider version is selected in P0. The
safety-critical guard adapter is not part of that package.

## 7. P0 exclusions and later gates

P0 does not implement `GuardPreparationProof`, ScopeKey types, guard refs,
admission traits, Trail entries, delivery persistence, resolve mode, or the
Resolve01 Plug. Those belong to separately reviewed P1-P5 packets. P6 alone
may claim cross-project tests, and P7 alone may update release-facing support
or compatibility material.

Stop rather than invent if implementation would require Core changes, syntax
changes, delegated permission, Resolve-side provider execution, Claim/epoch
understanding, Resolve SQLite access, worker-side scope duplication, a second
Trail/replay/executor, automatic provider retry, a competing outcome taxonomy,
or a new scope form solely for Resolve.

## 8. P0 acceptance evidence

This note is accepted as P0 evidence only when the branch diff contains the
packet, this note, and its worker note, with no Rust/OCaml production changes;
the task checker and `cargo fmt --all -- --check` pass; `git diff --check`
passes; and the exact branch/base/status are recorded. This is an architecture
freeze input for later implementation review, not proof that Resolve01
execution or cross-project conformance exists.
