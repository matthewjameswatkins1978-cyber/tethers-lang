# Current Implementation Task

Control contract: `1`
Task: `J24K3e2 - Exact durable disabled installation publication mutation`
Owner: `OpenCode`
Model: `HY3`
Status: `IN_PROGRESS`
Task colour: `Red`
Route: `OpenCode using HY3 for one bounded Rust publication-mutation package; Lucy performs independent review and routine safe merge`
Base branch: `main`
Base commit: `45f78e47a09638d4070bf4479e4f1dcbe39c8cb1`
Implementation branch: `opencode/j24k3e2-exact-publication-mutation`
Worker note: `docs/worker-notes/2026-08-05-j24k3e2-exact-publication-mutation.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Accepted main: `45f78e47a09638d4070bf4479e4f1dcbe39c8cb1`
Implementation checkpoint: `WORKTREE`
Verification checkpoint: `WORKTREE`

## Objective

Implement only J24K3e2: one crate-private mutation boundary that consumes the sealed J24K3e1 prepared publication, freshly revalidates it immediately before mutation, and performs the exact crash-safe disabled-installation publication transaction.

Required sequence:

```text
receive sealed prepared publication
  -> freshly revalidate complete prepared intent and current recovery state
  -> create exact durable publication intent
  -> build and verify exact staging directory
  -> rename staging to exact final destination
  -> publish exact precomputed installed record unchanged
  -> verify completed publication
  -> remove exact intent through accepted recovery execution
  -> prove recovery is idle
```

This package performs the durable publication transaction only. It does not acquire the installation lock, change the public execution context, replace the deferred public executor route, add CLI wiring, or execute another J24J action.

## Relevant background and existing behaviour

Accepted main is exactly:

```text
45f78e47a09638d4070bf4479e4f1dcbe39c8cb1
```

J24K3e1 provides a sealed crate-private `PreparedInstallationPublication` containing one exact validated `InstallationPublicationIntent` and one exact precomputed `InstalledPlugRecord`. The transaction UUID, destination, timestamp, record digest and intent digest are already frozen. J24K3e1 performs no durable mutation.

Accepted J24K3 recovery support already provides authoritative intent persistence, exact transaction-state observation, exact destination verification, complete current-evidence revalidation, global installed-root audit, sealed recovery planning, and exact recovery mutation back to idle.

The frozen publication order is:

```text
write durable intent
  -> build and verify staging
  -> rename staging to final destination
  -> publish exact precomputed installed record
  -> remove intent
```

Record identity, creation timestamp, destination and digest must never be regenerated during mutation or recovery.

## Required behaviour

The following numbered index is checker-facing and defines the complete bounded implementation scope.

1. Add one sealed prepared-publication mutation function.
2. Freshly revalidate before the first durable write.
3. Persist the exact precomputed publication intent atomically.
4. Build and verify one exact staging directory.
5. Rename staging to the exact final destination.
6. Publish the exact precomputed installed record unchanged.
7. Remove the intent only after completed publication is proven.
8. Preserve crash-resumable state at every durable boundary.
9. Preserve exact recovery and path-safety classifications.
10. Preserve all excluded lock, executor and legacy behaviour.

### 1. Add one sealed prepared-publication mutation function

Add one crate-private production seam structurally equivalent to:

```rust
pub(crate) fn execute_prepared_disabled_installation_publication(
    request: &InstallationRequest,
    context: &InstallationRecoveryPlanningContext<'_>,
    prepared: PreparedInstallationPublication,
) -> Result<InstalledPlugRecord>;
```

The exact ownership and return arrangement may be narrowed for Rust clarity, but the semantic boundary is frozen.

Requirements:

- accept only the sealed J24K3e1 prepared value;
- do not accept arbitrary intent or record values;
- do not accept caller-supplied roots, UUIDs, timestamps, clocks or callbacks;
- do not expose a public mutation API;
- do not regenerate prepared identity or content;
- consume or otherwise prevent accidental reuse after successful mutation.

### 2. Freshly revalidate before the first durable write

Immediately before creating durable intent state:

- call the accepted complete prepared-intent evidence revalidator against current authoritative stores;
- call `plan_installation_recovery(request, context)` and require idle recovery state;
- require the global installed-root audit to succeed;
- require no current intent, staging, destination, record or contradictory installed state;
- require the prepared intent and installed record both validate exactly;
- require transaction ID, candidate ID, destination and installed-record digest remain internally exact.

Any stale evidence, changed authority, candidate drift, path-safety failure or non-idle recovery state must fail before durable intent creation.

### 3. Persist the exact precomputed publication intent atomically

Create durable intent only through the accepted `InstallationPublicationIntentStore` creation boundary.

Requirements:

- persist the exact prepared intent unchanged;
- retain accepted canonical JSON, temporary-file, flush, sync and atomic-rename behaviour;
- refuse existing, torn, malformed or contradictory intent state;
- never overwrite or replace an existing transaction;
- load after creation and require exact equality with the prepared intent.

Once the exact intent exists, later ordinary failures must leave recoverable state.

### 4. Build and verify one exact staging directory

Create exactly one staging directory for the prepared transaction using the accepted private naming convention. Copy only the exact candidate file set justified by the prepared record: `plug.json`, payload files and signature files.

Requirements:

- revalidate candidate and quarantine path chain before copying;
- reject reparse points and unsafe path components;
- create parent directories only inside the staging root;
- create new files without overwrite;
- flush and sync bytes using accepted installed-state behaviour;
- mark installed files read-only using accepted platform behaviour;
- verify exact file set, lengths, hashes, read-only permissions and path safety through the accepted destination verifier before rename;
- do not write the installed record into the payload directory.

If staging construction or verification fails after intent creation, retain the intent. Any staging-only cleanup must use accepted exact recovery authority rather than ad hoc broad deletion.

### 5. Rename staging to the exact final destination

Requirements:

- final destination exactly matches the prepared intent and record path;
- verify install-root and destination path chain immediately before rename;
- require destination absent;
- rename verified staging atomically within the same install root;
- never copy into, merge with, replace or adopt an existing destination;
- never choose a replacement destination;
- verify the exact destination again after rename.

A rename failure retains intent and all observable state required for accepted recovery classification.

### 6. Publish the exact precomputed installed record unchanged

Publish only through a minimal crate-private exact-record publication seam.

Requirements:

- use the exact record contained in the prepared intent;
- never regenerate UUID, timestamp, bindings, fields or digest;
- validate immediately before publication;
- require final destination to verify exactly against the record;
- create the immutable registry record under the exact installed ID;
- refuse overwrite, duplicate release, duplicate candidate, mismatched identity and contradictory registry state;
- reload and require exact equality after publication.

Do not route through legacy `install_disabled`, which generates its own identity and transaction.

### 7. Remove the intent only after completed publication is proven

Before intent removal:

- create one fresh authoritative recovery plan;
- require the exact completed-publication disposition for this transaction;
- execute only the accepted completed-intent removal route;
- require exact identity and digest match;
- do not add a direct unlink path.

After removal, require an empty intent store, idle fresh recovery, and unchanged exact destination plus record.

### 8. Preserve crash-resumable state at every durable boundary

Tests must demonstrate accepted recovery can resume or finish from:

```text
intent only
intent + staging
intent + destination
intent + destination + exact record
completed publication with intent awaiting removal
```

Do not introduce a state outside the accepted recovery table. Do not broadly roll back after final destination publication, delete unexplained final state, remove intent after staging-cleanup failure, or claim success while recovery remains non-idle.

### 9. Preserve exact recovery and path-safety classifications

Preserve accepted stable families, including:

```text
installation_intent_invalid
installation_intent_evidence_stale
installation_destination_untracked
installation_recovery_conflict
installation_recovery_io
installed_conflict
installed_record_invalid
unsafe_store_path
```

Preserve an earlier authoritative classification when a lower layer detects the failure first. Do not add a broad publication error, collapse unsafe paths into evidence staleness, or convert malformed state into success-on-absence cleanup.

### 10. Preserve excluded behaviour

All exclusions are normative and repeated under `## Forbidden changes` below.

## Relevant components

Expected changes are bounded to the minimum needed among:

- `tethers-0.1/host-rust/src/installation_publication_preparation.rs` for sealed prepared-value access or narrow ownership adjustment;
- one new private publication-mutation module and direct test module;
- `tethers-0.1/host-rust/src/installation_publication_intent.rs` only for a minimum exact accepted-store seam if absent;
- `tethers-0.1/host-rust/src/installation_recovery*.rs` only to call existing accepted planning or execution boundaries, not redesign them;
- `tethers-0.1/host-rust/src/installed.rs` only for a minimum exact precomputed-record publication seam;
- `tethers-0.1/host-rust/src/lib.rs` for private module registrations;
- this task packet and its worker note.

Changing another production file requires a compile-proven necessity recorded in the worker note. Stop rather than widening architecture casually.

## Frozen decisions and invariants

- J24K3e1 owns transaction identity and immutable publication content.
- J24K3e2 performs only the exact prepared transaction.
- Fresh complete revalidation occurs immediately before intent creation.
- Durable mutation starts only when the exact intent is atomically created.
- Intent precedes staging; staging precedes destination; destination precedes record; record precedes intent removal.
- Installed record identity, timestamp and digest never change.
- Existing J24K3 recovery planning and execution remain the sole recovery authority.
- Every durable prefix is recoverable or fail-closed under accepted rules.
- No installation lock is acquired in this package.
- Later composition must hold one installation lock across preparation and mutation.

## Acceptance criteria

1. Valid sealed preparation with fresh evidence completes one exact transaction.
2. Fresh revalidation occurs before first durable write; stale preparation creates no intent.
3. Persisted intent exactly equals prepared intent.
4. Staging contains exactly the justified files with accepted hashes, lengths, permissions and path safety.
5. Staging is verified before exact atomic rename.
6. Final destination exactly equals `plug-<installed_id>` from the prepared record.
7. Published installed record is byte-semantically equal to the prepared record.
8. UUID, timestamp, destination, record digest and intent digest are never regenerated.
9. Intent removal follows fresh completed-publication recovery classification.
10. Success leaves idle recovery, no intent or staging, and one exact destination plus record.
11. Every durable prefix is accepted by existing recovery planning and can be completed or cleaned by existing recovery execution.
12. Staging-cleanup failure retains intent.
13. Existing, mismatched, unsafe, malformed, torn or contradictory state fails closed without adoption or overwrite.
14. No lock, public executor, context or CLI wiring is introduced.
15. All named regressions and full serial verification pass.
16. Cargo.lock remains unchanged.

## Required verification

Add direct tests whose names begin `j24k3e2`. At minimum prove:

1. valid prepared publication completes exactly once;
2. returned and published record equal the prepared record;
3. persisted intent equals the prepared intent before later steps;
4. stale evidence before mutation creates no intent;
5. non-idle recovery creates no new state;
6. candidate or quarantine drift is refused;
7. exact staging files, lengths, hashes and read-only permissions are verified;
8. unsafe or reparse staging paths fail closed;
9. existing staging or destination is not overwritten or adopted;
10. rename uses exact prepared destination;
11. final destination is reverified before record publication;
12. exact record publication refuses mismatched identity, digest, duplicate release and duplicate candidate;
13. publication retains original UUID and timestamp;
14. completed publication is freshly recovery-planned before intent removal;
15. success leaves no intent, no staging and idle recovery;
16. intent-only and intent-plus-staging prefixes are recoverable;
17. destination-without-record publishes the exact record once;
18. destination-plus-matching-record removes only completed intent;
19. record-without-destination, mismatched destination and mismatched record fail closed;
20. staging-cleanup failure retains intent;
21. path-safety failures preserve `unsafe_store_path`;
22. the same prepared transaction cannot duplicate publication;
23. J24K3d2 recovery regressions remain green;
24. legacy installed-state mutation remains green.

Use real stores and filesystem fixtures. Run focused direct tests, named J24K3e1/J24K3d2/J24K3d1/J24K3c4/J24K3c3/J24K3c2/J24K3c1/J24K3b/J24K3a/J24K2/J24J and installed-state/M3 regressions, then full serial:

```text
RUST_TEST_THREADS=1 just verify
```

Also require:

```text
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
cargo fmt --all -- --check
git diff --check
git status --short
```

Cargo.lock SHA-256 must match the starting value and Cargo.lock must be unchanged.

## Forbidden changes

J24K3e2 must not:

- acquire or alter `InstallationLockGuard`;
- change `InstallationExecutionContext` or public execution signatures;
- replace `installation_publication_deferred` in `installation_execution.rs`;
- wire `execute_next_installation_action`;
- implement J24L or any CLI;
- execute trust, conformance or approval actions;
- change J24J planning;
- change intent, installed-record or request schemas;
- change legacy `install_disabled` signatures or mutation order;
- add dependencies or change Cargo.lock;
- add production fault-injection hooks, caller clocks or deterministic UUID injection;
- add arbitrary prepared-value constructors;
- refactor unrelated recovery architecture;
- author new policy, adoption, cleanup or repair behaviour.

## Stop conditions

Stop immediately and report without widening scope on any of:

- task-packet checker failure;
- shell mismatch;
- branch, remote-tip, base or ancestry mismatch;
- non-fast-forward or unexpected history;
- unclear authority boundary or need for new policy;
- required change outside the bounded relevant components without compile-proven necessity;
- failed direct test, named regression or full verification;
- changed Cargo.lock or dependency graph;
- unexplained modified or untracked file;
- inability to preserve an accepted recovery or path-safety classification;
- requirement to acquire the lock, alter public execution context, wire the executor or implement CLI behaviour.

Do not repair or rewrite this Red task's normative scope. Return the blocker to Lucy.

## Expected pre-existing changes

None. Before changing status to `IN_PROGRESS`, require a clean worktree at the exact remote branch tip, with no modified or untracked files.

## Checkpoint procedure

1. Change task and worker-note status `READY` to `IN_PROGRESS`.
2. Implement production code and direct tests.
3. Commit production code and tests.
4. Capture and resolve the exact implementation SHA.
5. Record it as `Implementation checkpoint` in both documents.
6. Run direct tests, focused regressions, named regressions and full serial verification at that exact checkpoint.
7. Add and complete the worker note's `## Changes made` section before status `COMPLETE`.
8. Record evidence honestly, including anything not run.
9. Commit verification documentation only.
10. Capture and resolve the exact verification SHA.
11. Record it as `Verification checkpoint` in both documents through a final documentation-only commit.
12. Run the task-packet checker and require PASS.
13. Require formatting, diff and clean-status gates.
14. Push branch and report the exact final remote tip.

Do not add a self-referential final-tip field to either document.

## Required handoff

Report:

- branch and final remote tip;
- implementation and verification checkpoints;
- exact production and test files changed;
- direct and focused test counts;
- named regression and full serial verification results;
- Cargo.lock SHA-256 and unchanged status;
- task-packet checker, formatting, diff and clean-status results;
- any earlier authoritative error classification differing from a packet prediction;
- remaining risks and smallest next action.
