# Current Implementation Task

Control contract: `1`
Task: `J24K3d2 - Exact installation recovery executor`
Owner: `OpenCode`
Model: `Luna`
Status: `COMPLETE`
Task colour: `Red`
Route: `OpenCode using Luna for one bounded Rust recovery-mutation package; Lucy performs independent review and routine safe merge`
Base branch: `main`
Base commit: `ea4076085ed246a95eb2c0edab462b8c69d461fc`
Implementation branch: `opencode/j24k3d2-exact-recovery-executor`
Worker note: `docs/worker-notes/2026-08-05-j24k3d2-exact-recovery-executor.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Accepted main: `ea4076085ed246a95eb2c0edab462b8c69d461fc`
Implementation checkpoint: `371136913c99a67c08eb61484d6a69e3576ea5ad`
Verification checkpoint: `WORKTREE`

## Objective

Implement only J24K3d2: one crate-private exact installation-recovery executor built on the accepted J24K3d1 sealed read-only plan.

The executor must:

```text
receive one sealed validated recovery plan
  -> create a fresh authoritative recovery plan
  -> require exact plan equality before the first mutation
  -> perform only the accepted disposition sequence
  -> replan between dependent recovery mutations
  -> remove the exact authoritative intent only when the required post-state is proven
  -> create one fresh final plan
  -> require idle recovery state
```

J24K3d2 completes an already-started or already-completed private publication transaction. It does not begin a new publication transaction and does not execute an ordinary J24J installation action.

The package remains crate-private and unwired from the public installation executor. Lock integration is deliberately deferred to the next composition package, where planning and execution will occur inside one held installation-lock lifetime.

## Relevant background and existing behaviour

The accepted J24K3d1 foundation is the sole read-only planning and proof-selection boundary for private installation recovery.

## Accepted foundation

Accepted main is exactly:

```text
ea4076085ed246a95eb2c0edab462b8c69d461fc
```

The accepted recovery foundation provides:

- J24K3a: one validated authoritative `InstallationPublicationIntentStore`, including exact `remove_if_matches`;
- J24K3b: the four frozen recovery dispositions;
- J24K3c1: exact staging, destination and installed-record observation;
- J24K3c2: exact destination file-set, digest, length, permission and path verification;
- J24K3c3: complete current-evidence revalidation;
- J24K3c4: global installed-root namespace audit;
- J24K3d1: one sealed read-only plan that composes those proofs and performs no mutation.

J24K3d1 is authoritative for planning and proof selection. J24K3d2 must not reproduce its state table, evidence rules or classifier.

The frozen recovery dispositions are:

```text
RemoveIntentOnly
RemoveStagingThenIntent
RevalidateDestinationThenPublishRecord
VerifyCompletedPublicationThenRemoveIntent
```

## Required behaviour

### 1. Add one narrow crate-private recovery executor

Add a private module structurally equivalent to:

```rust
pub(crate) enum InstallationRecoveryExecutionOutcome {
    Idle,
    Recovered {
        disposition: InstallationRecoveryDisposition,
    },
}

pub(crate) fn execute_validated_installation_recovery(
    request: &InstallationRequest,
    context: &InstallationRecoveryPlanningContext<'_>,
    plan: ValidatedInstallationRecoveryPlan,
) -> Result<InstallationRecoveryExecutionOutcome>;
```

The exact naming and ownership arrangement may be narrowed for Rust borrowing clarity, but the semantic boundary is frozen.

The executor accepts only the sealed `ValidatedInstallationRecoveryPlan` produced by J24K3d1.

It must not accept:

- a caller-supplied intent;
- a caller-supplied disposition;
- staging, destination or record booleans;
- arbitrary roots or paths;
- callbacks, mutation functions or repair policy;
- a precomputed installed-record replacement;
- an allow-list or adoption policy.

Make the minimum internal trait derives or accessors needed to compare one sealed plan with a freshly generated plan. Do not expose arbitrary constructors or mutable fields.

### 2. Require a fresh exact plan before the first mutation

Immediately before any mutation, call `plan_installation_recovery(request, context)` again.

Require exact equality between:

- idle or pending state;
- the complete validated intent;
- the recovery disposition.

If the fresh plan differs, return the frozen recovery conflict classification and perform no mutation.

Do not treat a new but superficially equivalent intent as the same transaction. Exact value equality is required.

The future lock-integration package will guarantee that no concurrent installer can change state between this recheck and mutation. J24K3d2 must still perform the recheck itself.

### 3. Idle route

When both the supplied and fresh plans are idle:

- perform no mutation;
- return `InstallationRecoveryExecutionOutcome::Idle`;
- preserve every relevant root byte-for-byte.

### 4. RemoveIntentOnly

For `RemoveIntentOnly`:

1. require the fresh plan to equal the sealed supplied plan;
2. call only `InstallationPublicationIntentStore::remove_if_matches` with the exact planned intent;
3. require it to return `true`;
4. create one fresh final recovery plan;
5. require the final plan to be idle;
6. return `Recovered { disposition: RemoveIntentOnly }`.

A missing or changed intent is a conflict, not success.

### 5. RemoveStagingThenIntent

Add one narrow host-owned registry operation structurally equivalent to:

```rust
InstalledPlugRegistry::remove_installation_recovery_staging(
    &self,
    intent: &InstallationPublicationIntent,
) -> Result<()>;
```

It must:

- validate the complete intent;
- revalidate the already-opened install and record roots;
- derive only the exact `.staging-<transaction_id>` path;
- reject symlink, junction or reparse staging state;
- require staging present, destination absent and record absent;
- remove only that exact staging directory;
- verify the exact staging path is absent afterward;
- map ordinary filesystem failure to `installation_recovery_io`;
- preserve `unsafe_store_path`;
- leave the intent untouched on failure;
- never inspect or delete arbitrary sibling staging paths.

The executor sequence is exact:

1. fresh supplied-plan equality check;
2. remove exact staging;
3. call J24K3d1 again;
4. require the same exact intent with `RemoveIntentOnly`;
5. remove that intent through `remove_if_matches` and require `true`;
6. replan and require idle;
7. return `Recovered { disposition: RemoveStagingThenIntent }`.

Failed staging cleanup retains the intent.

### 6. RevalidateDestinationThenPublishRecord

Add one narrow exact-record publication operation structurally equivalent to:

```rust
InstalledPlugRegistry::publish_installation_recovery_record(
    &self,
    intent: &InstallationPublicationIntent,
) -> Result<()>;
```

It must:

- validate the complete intent and embedded installed record;
- revalidate the already-opened install and record roots;
- derive only the exact destination and exact `<installed_id>.json` record path;
- require staging absent, destination present and installed record absent;
- call the accepted destination verifier again immediately before publication;
- publish the exact precomputed `intent.installed_record` through the accepted immutable record-store boundary;
- preserve `created_unix_ms`, installed ID, record digest and every record byte-semantic field;
- never recompute the record, refresh timestamps or generate a new identity;
- verify the newly observable record exactly equals the intent record;
- map ordinary filesystem failure to `installation_recovery_io`;
- preserve `unsafe_store_path`;
- leave the intent present on failure.

The executor sequence is exact:

1. fresh supplied-plan equality check, which already reruns complete evidence and destination proofs;
2. publish the exact precomputed installed record;
3. call J24K3d1 again;
4. require the same exact intent with `VerifyCompletedPublicationThenRemoveIntent`;
5. remove that intent through `remove_if_matches` and require `true`;
6. replan and require idle;
7. return `Recovered { disposition: RevalidateDestinationThenPublishRecord }`.

No destination rename or staging construction occurs in this package.

### 7. VerifyCompletedPublicationThenRemoveIntent

For `VerifyCompletedPublicationThenRemoveIntent`:

1. require the fresh plan to equal the supplied plan;
2. rely on that fresh J24K3d1 plan to rerun current evidence and exact destination verification;
3. remove only the exact matching authoritative intent;
4. require removal to return `true`;
5. replan and require idle;
6. return `Recovered { disposition: VerifyCompletedPublicationThenRemoveIntent }`.

Do not rewrite or republish an already matching installed record.

### 8. Error preservation and resumability

Use existing stable recovery-facing errors only:

```text
installation_intent_invalid
installation_intent_conflict
installation_intent_io
installation_intent_evidence_stale
installation_destination_untracked
installation_recovery_conflict
installation_recovery_io
unsafe_store_path
```

Do not add a generic success-on-absence path.

After any error:

- never remove a mismatched intent;
- failed staging cleanup leaves intent and staging recoverable;
- failed exact record publication leaves intent and destination recoverable;
- successful record publication followed by intent-removal failure leaves matching record, destination and intent recoverable through the completed-publication disposition;
- unexplained final state remains fail-closed.

Do not add rollback of immutable records or deletion of final destinations.

### 9. No public or lock integration yet

J24K3d2 must not:

- modify `InstallationExecutionContext`;
- acquire or expose `InstallationLockGuard`;
- call the recovery executor from `execute_next_installation_action`;
- call J24J or ordinary installation execution;
- create a publication intent;
- build or rename a staging directory;
- implement `PublishDisabledInstallation`;
- add a public API;
- change dependencies or Cargo.lock.

The new executor is crate-private and directly tested. The next package will wire it inside the existing outer lock scope.

## Relevant components

- `tethers-0.1/host-rust/src/installation_recovery_execution.rs`: crate-private recovery executor.
- `tethers-0.1/host-rust/src/installation_recovery_execution_tests.rs`: direct executor-entry tests.
- `tethers-0.1/host-rust/src/installation_recovery_plan.rs`: sealed-plan equality support only.
- `tethers-0.1/host-rust/src/installed.rs`: exact staging cleanup and exact record publication.
- `tethers-0.1/host-rust/src/lib.rs`: private module registrations only.

## Frozen decisions and invariants

- J24K3d1 remains the sole planner, classifier, evidence, and proof-selection boundary.
- The executor accepts only `ValidatedInstallationRecoveryPlan` and uses exact value equality for every replan check.
- Intent removal is performed only through `InstallationPublicationIntentStore::remove_if_matches` with the exact authoritative intent.
- Recovery mutations are limited to exact staging cleanup and publication of the exact precomputed installed record.
- Every successful route ends with a fresh idle recovery plan; failures retain resumable authoritative state.
- Lock ownership and public installation-executor composition remain deferred to the next package.

## Acceptance criteria

1. The executor accepts only a sealed validated recovery plan and performs an exact fresh-plan equality check before mutation.
2. Idle, intent-only, staging, destination-only, and completed-publication routes perform only their accepted mutations and require a final idle plan.
3. Staging cleanup and exact record publication preserve authoritative state and fail closed on path, root, evidence, record, and filesystem errors.
4. Direct `j24k3d2` tests prove resumability, exact identity preservation, no unrelated mutations, and no ordinary installation execution.
5. All named regression suites, focused Nextest, full serial verification, formatting, lockfile hash, and task-packet checks pass.
6. A fresh idle plan equal to the supplied idle plan performs no mutation.
7. A fresh pending plan must equal the complete supplied intent and disposition before mutation.
8. A changed authoritative intent returns `installation_recovery_conflict` without mutation.
9. A changed disposition returns `installation_recovery_conflict` without mutation.
10. Intent-only recovery removes only the exact matching authoritative intent.
11. Missing or mismatched intent removal is not treated as success.
12. Staging recovery removes only the exact transaction staging directory.
13. Staging recovery replans to the same intent-only disposition before intent removal.
14. Failed staging cleanup retains both the intent and staging state.
15. Reparse staging state is rejected without deleting its target.
16. Destination-only recovery uses only the exact verified destination and precomputed record.
17. Exact record publication preserves every installed-record field and digest.
18. Record publication rejects staging, destination, root, and record conflicts.
19. Record publication replans to completed-publication state before intent removal.
20. Failed record publication retains the intent and destination.
21. Completed-publication recovery removes only the exact matching intent.
22. No recovery route rewrites or republishes an already matching record.
23. Every successful mutation route ends with a fresh idle plan.
24. Unsafe paths preserve `unsafe_store_path`.
25. Ordinary filesystem failures map to the existing recovery I/O family.
26. No recovery route creates staging, renames a destination, adopts final state, or invokes ordinary installation execution.

## Direct test acceptance

Add direct production-entry tests whose names begin `j24k3d2`.

At minimum prove:

1. idle plan performs no mutation;
2. intent-only recovery removes exactly the matching intent and returns idle;
3. staging recovery removes only exact staging, then exact intent, and returns idle;
4. destination-only recovery publishes exactly the precomputed installed record, removes intent, and returns idle;
5. matching completed publication removes only intent and returns idle;
6. a changed authoritative intent after the supplied plan was created causes conflict with no mutation;
7. a changed disposition after the supplied plan was created causes conflict with no mutation;
8. stale evidence or destination drift before execution prevents publication and retains intent;
9. staging cleanup failure retains intent and staging;
10. staging symlink, junction or reparse state is refused without deleting its target;
11. unsafe or missing install/record roots preserve frozen errors;
12. record conflict or record-root write failure retains intent;
13. exact record publication preserves installed ID, `created_unix_ms`, record digest and complete record equality;
14. after exact record publication, completed-state replan is required before intent removal;
15. after staging cleanup, intent-only replan is required before intent removal;
16. a second call using a newly created idle plan performs no mutation;
17. unrelated executor, quarantine, candidate, trust, launch, conformance and approval roots remain unchanged;
18. no final destination is adopted or deleted;
19. no ordinary installation action or J24J mutation occurs.

Use real filesystem and platform path fixtures. Do not add callbacks, fault-injection production hooks or arbitrary constructors.

For Windows cleanup contention, use a real non-shareable file handle fixture. For Unix, use a real platform-appropriate permission or directory-state fixture. Record platform skips honestly.

## Regression acceptance

Preserve all accepted suites, including:

- J24K3d1;
- J24K3c4;
- J24K3c3;
- J24K3c2;
- J24K3c1;
- J24K3b;
- J24K3a;
- J24K2;
- J24J;
- M3 lifecycle.

The bounded Windows lifecycle teardown helper accepted in J24K3d1 remains unchanged.

## Checkpoint procedure

Avoid both transcription errors and self-referential final-tip fields.

1. Change task and worker-note status `READY` to `IN_PROGRESS`.
2. Implement production code and direct tests.
3. Commit production code and tests.
4. Capture the exact implementation SHA using `git rev-parse HEAD`.
5. Verify it resolves using `git cat-file -e "<sha>^{commit}"`.
6. Record that exact SHA as `Implementation checkpoint` in the task and worker note.
7. Update task and worker note to `COMPLETE`, leaving `Verification checkpoint: WORKTREE`.
8. Commit that completion candidate.
9. Capture the exact completion-candidate SHA using `git rev-parse HEAD`.
10. Run every required verification command at that exact commit.
11. After every command is green, verify the candidate SHA still resolves and record it as `Verification checkpoint`.
12. Commit and push only the final evidence-document update.
13. Run the task-packet checker once more at the final documentation tip.
14. Return the final remote tip externally.

Do not add a `Final remote tip` field to any committed document.

Do not manually invent, shorten or reconstruct checkpoint SHAs. Copy exact command output.

## Expected pre-existing changes

None.

## Required verification

Run from repository root:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1

cargo fmt `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all -- --check

cargo nextest run `
  --config-file .config/nextest.toml `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-features --locked `
  -E 'test(j24k3d2)'

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3d2 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3d1 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3c4 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3c3 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3c2 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3c1 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3b `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3a `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k2 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test j24j_installation_reconciliation `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test m3_lifecycle `
  --locked

$env:PATH = "$HOME\.cargo\bin;$PSHOME;$env:PATH"
$env:RUST_TEST_THREADS = "1"
just verify

Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
git diff --check
git status --short
```

Focused Nextest must finish with zero failures and zero retries.

Full serial `just verify` must finish with zero failures. Do not exclude a failed test from totals.

Cargo.lock must remain exactly:

```text
D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

## Permitted files

- `tethers-0.1/host-rust/src/installation_recovery_execution.rs` or one equivalently narrow new module;
- `tethers-0.1/host-rust/src/installation_recovery_execution_tests.rs` or one equivalently narrow direct-test module;
- `tethers-0.1/host-rust/src/installation_recovery_plan.rs` only for minimum sealed-plan equality/access support;
- `tethers-0.1/host-rust/src/installed.rs` only for exact staging cleanup and exact record-publication methods;
- `tethers-0.1/host-rust/src/lib.rs` only for private module registrations;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-05-j24k3d2-exact-recovery-executor.md`.

## Forbidden changes

- No installation-lock integration.
- No `InstallationExecutionContext` change.
- No call from `execute_next_installation_action`.
- No J24J planning or ordinary action execution.
- No new publication-intent creation.
- No staging construction or staging-to-destination rename.
- No `PublishDisabledInstallation` implementation.
- No destination deletion, adoption or rollback.
- No public API, schema or error-code expansion.
- No dependency, Cargo configuration or Cargo.lock change.
- No CLI, packaging, release, enablement, operational-scope or OCaml change.
- No automatic tool installation.
- No unrelated refactor.

## Stop conditions

Stop as `BLOCKED` if:

- the accepted J24K3d1 plan cannot remain the sole proof-selection boundary;
- safe exact mutation would require a public seam, arbitrary caller paths or production fault-injection hooks;
- a recovery error cannot be represented by the existing frozen error families;
- focused Nextest is unavailable;
- any required regression remains red;
- full serial `just verify` has any failure;
- Cargo.lock changes.

Do not merge.
