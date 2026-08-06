# Current Implementation Task

Control contract: `1`
Task: `J24K3f - Lock-composed disabled installation publication`
Owner: `OpenCode`
Model: `DeepSeek Pro`
Status: `COMPLETE`
Task colour: `Red`
Route: `OpenCode using DeepSeek Pro for one bounded Rust lock-composition package; Lucy performs independent review and routine safe merge`
Base branch: `main`
Base commit: `13cae687dc59c0dae74363b24d0ab57547702c53`
Implementation branch: `opencode/j24k3f-lock-composed-publication`
Worker note: `docs/worker-notes/2026-08-05-j24k3f-lock-composed-publication.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Accepted main: `13cae687dc59c0dae74363b24d0ab57547702c53`
Implementation checkpoint: `b64b5d46cd152bd71b993ee634a14c3c05799912`
Verification checkpoint: `TBD`

## Objective

Implement only J24K3f: compose the accepted J24K3e1 preparation boundary and J24K3e2 exact mutation boundary into the existing single-step installation executor while the existing `InstallationLockGuard` remains held.

For `InstallationPlanAction::PublishDisabledInstallation`, the locked executor must:

```text
use the current before-plan already created inside the lock
  -> open the accepted publication-intent store from executor_state_root
  -> prepare one sealed exact publication
  -> execute that exact prepared publication
  -> freshly run J24J after mutation
  -> require PublishDisabledInstallation -> Complete
  -> return Advanced { executed: PublishDisabledInstallation }
```

This package finishes J24K publication execution. It does not add J24L, a CLI, a multi-step loop, or any second mutation per invocation.

The package is explicitly authorised to restore the already frozen blueprint field:

```rust
pub executor_state_root: &'a Path,
```

on `InstallationExecutionContext`. No other context field or public API change is authorised.

## Relevant background and existing behaviour

Accepted main is exactly `13cae687dc59c0dae74363b24d0ab57547702c53`.

Accepted foundations now provide:

- J24K2: the non-inheritable RAII installation lock, planning inside the lock, one-action execution and transition checking;
- J24K3d: exact crash recovery planning and mutation while locked;
- J24K3e1: sealed read-only publication preparation from an exact current `PublishDisabledInstallation` plan;
- J24K3e2: exact durable mutation consuming that sealed prepared value;
- `installation_execution.rs` still returns `installation_publication_deferred` for `PublishDisabledInstallation`.

The frozen architecture blueprint already defines `executor_state_root: &'a Path` on `InstallationExecutionContext`, but the current implementation omitted it. J24K3f requires that exact root to open `InstallationPublicationIntentStore` and construct `InstallationRecoveryPlanningContext` without deriving authority from an unrelated path.

The outer public entry point already acquires the lock and delegates to an inner function whose recovery, planner, action and post-plan values remain inside the lock lifetime. Preserve that shape.

## Required behaviour

1. Restore exactly one accepted `InstallationExecutionContext` field: `executor_state_root: &'a Path`.
2. Update every legitimate construction site and fixture for that field without adding defaults or derived fallback paths.
3. Replace only the deferred `PublishDisabledInstallation` action arm.
4. Reuse the exact current locked `before` plan as the preparation comparison value.
5. Open `InstallationPublicationIntentStore` from `context.executor_state_root` while the lock is held.
6. Build the accepted `InstallationRecoveryPlanningContext` from the intent store and existing executor authorities.
7. Call J24K3e1 preparation and J24K3e2 mutation while the same lock remains held.
8. Run one fresh authoritative J24J plan after mutation and require `PublishDisabledInstallation -> Complete`.
9. Return the existing `Advanced { executed: PublishDisabledInstallation }` outcome shape.
10. Preserve recovery-before-ordinary-action ordering, earlier error classifications, and the one-invocation/one-mutation invariant.

## Relevant components

Expected changes are bounded to the minimum among:

- `tethers-0.1/host-rust/src/installation_execution.rs`;
- existing execution-context construction sites and direct fixtures that must supply the newly restored accepted field;
- its direct test module or one new narrowly named J24K3f test module;
- `tethers-0.1/host-rust/src/lib.rs` only if a new private test module is added;
- this packet and its worker note.

J24K3e1, J24K3e2, recovery, lock, intent, installed-state and planner modules should be called, not redesigned. Any other production file requires a compile-proven necessity recorded before editing.

## Frozen decisions and invariants

- `executor_state_root` is an independent accepted authority root, not a subdirectory inferred from lock, quarantine, scratch, install or record paths.
- `InstallationPublicationIntentStore` is opened from exactly `context.executor_state_root`.
- One outer `InstallationLockGuard` spans recovery, before-plan, preparation, mutation, after-plan and postcondition checks.
- Preparation and mutation never occur outside that lock in this executor route.
- J24J remains the sole ordinary installation reconciliation authority.
- The caller never supplies a precomputed plan.
- One invocation performs zero or one durable ordinary mutation.
- Publication preparation does not count as a mutation.
- Publication mutation is the sole ordinary mutation in this action arm.
- Recovery, if needed, occurs before ordinary planning and may consume the invocation according to accepted J24K2/J24K3 behaviour.
- J24K3f adds no retry and no second action.
- J24L remains separate.

## Acceptance criteria

1. `InstallationExecutionContext` contains exactly the restored `executor_state_root: &'a Path` field in the frozen blueprint position or an equivalent Rust ordering.
2. All context construction sites compile only by supplying an explicit executor-state root.
3. No fallback derives executor state from another root.
4. A locked publication-ready request completes exact disabled installation publication.
5. J24K3e1 receives the exact `before` plan produced inside the lock.
6. J24K3e2 consumes the resulting sealed prepared value inside the same lock lifetime.
7. Successful publication yields a fresh after-plan with action `Complete`.
8. The returned outcome is `Advanced { executed: PublishDisabledInstallation }`.
9. The returned before-plan is the original exact publication-ready plan and the returned after-plan is the fresh exact complete plan.
10. A stale evidence change between planning and preparation fails closed without publication.
11. A mutation or recovery failure releases the lock and leaves accepted resumable state.
12. A concurrent second invocation still fails immediately with `installation_busy`.
13. No invocation executes any second ordinary mutation.
14. Existing trust, conformance, approval, complete and failed-conformance action behaviour remains unchanged.
15. `installation_publication_deferred` is removed only from the now-implemented action route and is not repurposed.
16. No other public context/API, CLI, J24L, schema, dependency or Cargo.lock change occurs.
17. Named J24K3e2, J24K3e1, J24K3d2, J24K2 and J24J regressions pass.
18. Full serial verification passes.

## Required verification

Add direct tests whose names begin `j24k3f` and use real stores/filesystem fixtures. At minimum prove:

- valid publication-ready state advances to `Complete` through the public locked executor;
- the publication intent store is rooted under the explicit executor-state root;
- no intent state is created under lock, quarantine, scratch, install or record roots;
- exact destination and installed record are created and recovery is idle;
- returned before/after plans and executed action are exact;
- preparation or evidence failure before intent creation produces no publication;
- failure after intent creation remains recoverable and the lock is released;
- a second lock acquisition/invocation remains immediately busy;
- no second action is executed;
- existing `Complete` still returns `AlreadyComplete` without mutation.

Run direct tests, focused Nextest where available, named regressions for `j24k3e2`, `j24k3e1`, `j24k3d2`, `j24k2`, `j24j`, installed-state regressions, then `RUST_TEST_THREADS=1 just verify`, packet checker, fmt, diff check and clean status.

## Forbidden changes

Do not:

- add any `InstallationExecutionContext` field other than exactly `executor_state_root: &'a Path`;
- derive executor state from `lock_path`, `quarantine_root`, `conformance_scratch_root`, install root, record root, current directory, environment variables or process-global state;
- make `executor_state_root` optional or provide an implicit default;
- alter lock acquisition, lock path rules, handle inheritance or RAII behaviour;
- add another lock or shorten the existing lock lifetime;
- change any other public function signature or result enum;
- add an internal loop, retry, fifth-call logic or J24L;
- parse CLI arguments or print UI/progress output;
- execute trust, conformance, approval or publication twice;
- regenerate prepared publication identity;
- redesign preparation, mutation, recovery, intent or installed-state modules;
- change schemas, dependencies or Cargo.lock;
- add production fault injection, caller clocks or arbitrary constructors.

## Stop conditions

Stop before further edits on any packet-checker failure, branch/base mismatch, dirty unexplained file, need for any public context/API change beyond the exact accepted `executor_state_root` field, need to redesign lock/recovery/publication boundaries, failed direct test or regression, changed Cargo.lock, non-fast-forward history or scope expansion.

Do not repair or rewrite this Red task's normative scope. Return any further blocker to Lucy.

## Expected pre-existing changes

None.

## Checkpoint procedure

1. Require the amended READY packet checker passes.
2. Change packet and worker-note status to `IN_PROGRESS`.
3. Implement production code and direct tests.
4. Commit implementation and capture one full implementation SHA.
5. Record that SHA in both documents.
6. Run all required verification at that exact checkpoint.
7. Complete the worker note honestly, including `## Changes made`.
8. Change both statuses to `COMPLETE`.
9. Commit verification documentation only.
10. Capture and record the verification checkpoint through a final documentation-only commit if required.
11. Require packet checker, fmt, diff check and clean status.
12. Push the branch and report exact SHAs and evidence. Do not merge.