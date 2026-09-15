# TETHERS x RESOLVE01 - P0 Contract and Seam Audit

Task: `TETHERS x RESOLVE01 / P0`

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Red`

Owner: `Codex`

Route: `Freeze the Resolve Guard Adapter contract against current Tethers host machinery, document exact insertion points and open dependencies, and make no implementation-semantics change.`

Base commit: `ddc5d0fccfc0a38ff0af3013c5e9ff6ba357e0b2`

Worker note: `docs/worker-notes/2026-09-15-tethers-resolve-p0.md`

Suggested branch:

`codex/tethers-resolve-p0-contract`

## Objective

Produce the repository-owned P0 architecture note that freezes the Resolve
Guard Adapter seam without adding Resolve concepts to Tethers Core or creating
parallel policy, scope, replay, Trail, provider, packaging, or recovery
machinery.

## Relevant background and existing behaviour

The Tethers host already owns exact capability resolution, manifest and
provider binding, input validation, binding-owned scope assessment, effective
policy, one-shot approval, replay admission, durable intent, provider
invocation, outcome classification, Trail recording, and Result Anchors. The
shared execution boundary is used by both ordinary sequential Actions and the
accepted Together-group prepare/invoke split. Tethers Core remains the
Resolve-agnostic OCaml planner and must not learn Goal, Commitment, Claim,
lease, worker, or Resolve recovery concepts.

The supplied Resolve01 plan defines two future components: a trusted Rust-host
Resolve Guard Adapter and a normal user-facing Resolve01 TetherPlug. P0 covers
only the contract/seam audit and preparation/resume handshake freeze. The live
Resolve wire protocol is deferred until Resolve S3 freezes it.

## Required behaviour

1. Record the constitutional ownership boundary: Resolve coordinates, Tethers
   authorises and executes, and Resolve receives authoritative Tethers outcomes.
2. Identify the exact current Tethers host seams for resolution, scope,
   policy, approval, replay, durable intent, provider invocation, Trail,
   outcomes, Result Anchors, configuration, and Plug lifecycle.
3. Freeze a preparation/resume handshake in which Tethers-derived scope keys
   are opaque to Resolve, preparation is evidence rather than authority, and
   resume reconstructs and exactly compares fresh Tethers evidence.
4. Freeze the execution ordering: Tethers permission and replay admission,
   durable intent, Resolve guard admission, existing invocation arming, and
   existing provider execution; DENY, UNAVAILABLE, unresolved ASK, rejected
   guard, and indeterminate guard admission never invoke the target provider.
5. Record the current supported version and package surfaces and explicitly
   defer Resolve01 public capability and live-wire choices that are not yet
   frozen by Resolve01.
6. Define P0 exclusions, acceptance evidence, and stop conditions so later
   implementation packets cannot silently widen the integration.

## Relevant components

- `tethers-0.1/host-rust/src/configured_runtime.rs`
- `tethers-0.1/host-rust/src/policy.rs`
- `tethers-0.1/host-rust/src/approval.rs`
- `tethers-0.1/host-rust/src/replay_runtime.rs`
- `tethers-0.1/host-rust/src/replay.rs`
- `tethers-0.1/host-rust/src/dispatch.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/host-rust/src/executor.rs`
- `tethers-0.1/host-rust/src/installed_provider_executor.rs`
- `tethers-0.1/host-rust/src/runtime_config.rs`
- `tethers-0.1/host-rust/src/package.rs`
- `tethers-0.1/host-rust/src/plug_pack.rs`
- `tethers-0.1/host-rust/src/plug_conform.rs`
- `docs/CONSTITUTION.md`
- `tethers-0.1/SPEC.md`
- `docs/DECISIONS.md`
- `docs/CAPABILITY_BRIDGE.md`
- `docs/VERSIONING.md`
- `docs/COMPATIBILITY.md`

## Frozen decisions and invariants

- No Tethers Core or OCaml changes are authorised.
- No Tether syntax, Plan meaning, policy vocabulary, manifest dialect, or
  Result Anchor taxonomy changes are authorised.
- Resolve receives only an opaque guard reference and opaque, Tethers-derived
  ScopeKeys; Tethers does not inspect Resolve Goal, Commitment, Claim, lease,
  epoch, DAG, worker, or recovery state.
- Resolve guard admission is coordination, not permission. A valid guard can
  never turn Tethers DENY, UNAVAILABLE, or unresolved ASK into ALLOW.
- ScopeKeys are derived only from the existing reviewed Tethers scope
  representation and are deterministic, versioned, sorted, duplicate-free,
  and compared by exact equality. No new scope form is added in P0.
- Preparation evidence never permits execution. Resume reconstructs the fresh
  Tethers proof and refuses any identity, argument, manifest, provider,
  binding, scope, or trust drift.
- Durable Tethers intent remains mandatory before any Resolve admission or
  provider effect. The existing replay and invocation boundaries remain the
  only replay/execution machinery.
- Provider outcomes remain exactly SUCCEEDED, FAILED, or UNCERTAIN. Resolve
  notification failure cannot rewrite a durable Tethers outcome.
- The Resolve01 TetherPlug remains ordinary user-facing capability exposure;
  guard admission is never a normal Tether capability.

## Acceptance criteria

1. The architecture note names both components and preserves their ownership
   boundaries without introducing a third authority.
2. The note identifies current code/document seams for scope, policy,
   approval, replay, intent, invocation, outcomes, Trail, anchors, config, and
   Plug lifecycle.
3. The preparation proof fields, opaque ScopeKey rule, resume exact-match
   rule, and deferred Resolve wire seam are explicit and versionable.
4. The serial and Together-group execution paths have the same documented
   post-intent/pre-provider guard insertion boundary.
5. The note records current package/Socket/MCP versions and all unresolved
   dependencies without inventing Resolve01 API details.
6. The branch contains only the authorised packet, architecture note, and
   worker note; no Rust, OCaml, configuration, package, fixture, or test
   semantics change is present.

## Required verification

Run the repository-owned tool diagnostic, the task-packet checker in both
IN_PROGRESS and final state, `cargo fmt --all -- --check`, `git diff --check`,
and complete diff/path inspection. Product Rust/OCaml tests are not required
for this documentation-only P0, but must not be represented as run.

## Forbidden changes

- No Rust or OCaml production-code changes.
- No Resolve network calls, SQLite access, live adapter, fake protocol, or
  generic plugin framework.
- No new policy state, scope assessor, replay store, Trail file, executor,
  outcome taxonomy, retry path, scheduler, or recovery controller.
- No Resolve01 public capability names, schemas, effects, or provider contract
  before Resolve01 freezes them.
- No merge, direct `main` update, force-push, history rewrite, installation,
  dependency update, or unrelated cleanup.

## Stop conditions

Stop and report if the current code cannot expose the required seam without
duplicating Tethers authority, if the preparation/resume contract requires a
new Tethers semantic decision, if Resolve concepts must enter Core, if the
current scope machinery must be broadened solely for Resolve, or if the live
Resolve protocol is needed before Resolve S3 freezes it. After two materially
similar failed attempts against the same underlying problem, stop with exact
evidence and the smallest unresolved question.

## Expected pre-existing changes

None

## Implementation scope

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P0_SEAM_AUDIT.md`
- `docs/worker-notes/2026-09-15-tethers-resolve-p0.md`
