# Current Implementation Task

Control contract: `1`
Task: `J24K3e2 - Exact durable disabled installation publication mutation`
Owner: `OpenCode`
Model: `HY3`
Status: `READY`
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
  -> freshly revalidate the complete prepared intent and current recovery state
  -> create the exact durable publication intent
  -> build and verify the exact staging directory
  -> rename staging to the exact final destination
  -> publish the exact precomputed installed record unchanged
  -> verify completed publication
  -> remove the exact intent
  -> prove recovery is idle
```

This package performs the durable publication transaction only. It does not acquire the installation lock, change the public execution context, replace the deferred public executor route, add CLI wiring, or execute any other J24J action.

## Relevant accepted behaviour

Accepted main is exactly:

```text
45f78e47a09638d4070bf4479e4f1dcbe39c8cb1
```

J24K3e1 now provides:

- `PreparedInstallationPublication`, sealed and crate-private;
- one exact validated `InstallationPublicationIntent`;
- one exact precomputed `InstalledPlugRecord`;
- one frozen transaction UUID, destination, timestamp, record digest and intent digest;
- fresh exact J24J planning, full evidence revalidation and idle recovery proof before preparation;
- no durable mutation.

Accepted J24K3 recovery support already provides:

- authoritative intent persistence;
- exact transaction-state observation;
- exact destination verification;
- complete current-evidence revalidation;
- global installed-root audit;
- sealed recovery planning;
- exact recovery mutation back to idle.

The frozen publication order remains:

```text
write durable intent
  -> build and verify staging
  -> rename staging to final destination
  -> publish exact precomputed installed record
  -> remove intent
```

The record identity, creation timestamp and digest must never be regenerated during mutation or recovery.

## Required behaviour

The following numbered index is checker-facing and restates the ten frozen required-behaviour subsections without changing their meaning or scope.

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
- do not regenerate any prepared identity or content;
- consume or otherwise prevent accidental reuse of the prepared transaction after successful mutation.

### 2. Freshly revalidate before the first durable write

Immediately before creating durable intent state:

- call the accepted complete prepared-intent evidence revalidator against current authoritative stores;
- call `plan_installation_recovery(request, context)` and require idle recovery state;
- require the global installed-root audit to succeed;
- require no current intent, staging, destination, record or contradictory installed state;
- require the prepared intent and installed record both validate exactly;
- require transaction ID, candidate ID, destination and installed-record digest remain internally exact.

This fresh revalidation is mandatory even though J24K3e1 already validated the value.

Any stale evidence, changed authority, candidate drift, path-safety failure or non-idle recovery state must fail before durable intent creation.

### 3. Persist the exact precomputed publication intent atomically

Create the durable intent only through the accepted `InstallationPublicationIntentStore` creation boundary.

Requirements:

- persist the exact prepared intent unchanged;
- use the accepted canonical JSON, temporary-file, flush, sync and atomic-rename behaviour;
- refuse any existing, torn, malformed or contradictory intent state;
- do not overwrite or replace an existing transaction;
- after successful creation, the durable intent must load as exact equality with the prepared intent.

Once the exact intent exists, ordinary failures must leave state recoverable rather than pretending mutation never began.

### 4. Build and verify one exact staging directory

Create exactly one staging directory for the prepared transaction using the accepted private naming convention.

Copy only the exact candidate file set justified by the prepared record:

- `plug.json`;
- payload files;
- signature files.

Requirements:

- revalidate the candidate and quarantine path chain before copying;
- reject reparse points and unsafe path components;
- create parent directories only within the staging root;
- create new files without overwriting existing entries;
- flush and sync written bytes using accepted installed-state behaviour;
- mark installed files read-only using accepted platform behaviour;
- verify the exact file set, lengths, hashes, read-only permissions and path safety through the accepted destination-verification boundary before rename;
- do not write the installed record into the payload directory.

If staging construction or verification fails after intent creation, retain the intent. Cleanup of a wholly staging-only state may use only the already accepted exact recovery mutation authority, not ad hoc broad deletion.

### 5. Rename staging to the exact final destination

The final destination must be exactly the prepared intent destination and the prepared record installation path.

Requirements:

- verify the install root and destination path chain immediately before rename;
- require the destination to be absent;
- rename the verified staging directory atomically within the same install root;
- do not copy into, merge with, replace or adopt an existing destination;
- do not choose a new destination if the prepared one conflicts;
- after rename, verify the exact destination again through the accepted verifier.

A rename failure retains the intent and any observable state needed for accepted recovery classification.

### 6. Publish the exact precomputed installed record unchanged

Publish the installed record only through a minimal private exact-record publication seam.

Requirements:

- use the exact record already contained in the prepared intent;
- do not regenerate UUID, timestamp, bindings, fields or digest;
- require `InstalledPlugRecord::validate` immediately before publication;
- require the final destination to verify exactly against the record;
- create the immutable registry record under the exact installed ID;
- refuse overwrite, duplicate release, duplicate candidate, mismatched identity or contradictory registry state;
- after publication, load and require exact equality with the prepared record.

Do not route this through the legacy `install_disabled` mutation path, because that path generates its own identity and performs its own staging transaction.

### 7. Remove the intent only after completed publication is proven

Before intent removal:

- create one fresh authoritative recovery plan;
- require the exact completed-publication disposition for the current prepared transaction;
- execute only the accepted exact completed-intent removal route;
- require exact intent identity and digest match before deletion;
- do not directly unlink the intent through a new ad hoc path.

After removal:

- require the intent store to be empty;
- require a fresh recovery plan to be idle;
- require the installed record and destination to remain exact;
- return the exact installed record or an equally narrow success value.

### 8. Preserve crash-resumable state at every durable boundary

Direct tests must demonstrate accepted recovery can resume or finish from each durable prefix:

```text
intent only
intent + staging
intent + destination
intent + destination + exact record
completed publication with intent awaiting removal
```

The mutation boundary must not introduce a state outside the accepted J24K3 recovery table.

Do not attempt broad rollback after final destination publication.
Do not delete unexplained final state.
Do not remove the intent when staging cleanup fails.
Do not claim success while recovery remains non-idle.

### 9. Preserve exact recovery and path-safety classifications

Use existing stable families, including:

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

Preserve earlier authoritative classifications when a lower layer detects the failure first.

Do not add a broad generic publication error.
Do not collapse unsafe paths into evidence staleness.
Do not convert malformed state into success-on-absence cleanup.

### 10. Preserve excluded behaviour

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
- add public fault-injection hooks, caller clocks or deterministic UUID injection;
- refactor unrelated recovery code.

## Relevant components

Expected files are bounded to the minimum needed among:

- `tethers-0.1/host-rust/src/installation_publication_preparation.rs` for sealed prepared-value access or narrow ownership adjustment;
- a new private publication-mutation module and direct test module;
- `tethers-0.1/host-rust/src/installation_publication_intent.rs` only for a minimum exact accepted-store seam if absent;
- `tethers-0.1/host-rust/src/installation_recovery*.rs` only to call existing accepted planning/execution boundaries, not redesign them;
- `tethers-0.1/host-rust/src/installed.rs` only for a minimum exact precomputed-record publication seam;
- `tethers-0.1/host-rust/src/lib.rs` for private module registrations;
- this task packet and its worker note.

Changing any other production file requires a clear compile-proven necessity recorded in the worker note. Stop rather than widening architecture casually.

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

1. A valid sealed prepared publication with fresh current evidence completes one exact durable transaction.
2. Fresh revalidation occurs before the first durable write and stale preparation is refused without intent creation.
3. The persisted intent is exactly equal to the prepared intent.
4. Staging contains exactly the prepared candidate file set with accepted lengths, hashes, permissions and path safety.
5. The staging directory is verified before exact atomic rename.
6. The final destination exactly equals `plug-<installed_id>` from the prepared record.
7. The published installed record is byte-semantically equal to the prepared record.
8. UUID, creation timestamp, destination, record digest and intent digest are never regenerated.
9. Intent removal happens only after fresh recovery classifies the transaction as completed publication.
10. Successful completion leaves idle recovery, no intent or staging, and one exact destination plus record.
11. Every durable prefix is accepted by existing recovery planning and can be completed or cleaned by accepted recovery execution.
12. Staging cleanup failure retains the intent.
13. Existing, mismatched, unsafe, malformed, torn or contradictory state fails closed without adoption or overwrite.
14. No lock, public executor, context or CLI wiring is introduced.
15. All named regressions and full serial verification pass.
16. Cargo.lock remains unchanged.

## Direct test acceptance

Add direct tests whose names begin `j24k3e2`.

At minimum prove:

1. valid prepared publication completes exactly once;
2. returned/published record exactly equals the prepared record;
3. persisted intent exactly equals the prepared intent before later steps;
4. stale evidence before mutation creates no intent;
5. non-idle recovery before mutation creates no new state;
6. candidate or quarantine drift before mutation is refused;
7. exact staging file set, lengths, hashes and read-only permissions are verified;
8. unsafe or reparse staging paths fail closed;
9. existing staging or destination conflicts are not overwritten or adopted;
10. rename uses the exact prepared destination;
11. final destination is reverified before record publication;
12. exact record publication refuses mismatched identity, digest, duplicate release and duplicate candidate;
13. record publication uses the original prepared UUID and timestamp;
14. completed publication is freshly recovery-planned before intent removal;
15. successful completion leaves no intent, no staging and idle recovery;
16. intent-only crash prefix is recoverable;
17. intent-plus-staging crash prefix is recoverable;
18. destination-without-record crash prefix publishes the exact record once;
19. destination-plus-matching-record crash prefix removes only the completed intent;
20. record-without-destination remains fail-closed;
21. mismatched destination or record remains fail-closed;
22. staging cleanup failure retains intent;
23. path-safety failures preserve `unsafe_store_path`;
24. a second attempt with the same prepared value cannot duplicate publication;
25. existing J24K3d2 recovery regression remains green;
26. existing legacy installed-state mutation behaviour remains green.

Use real stores and filesystem fixtures. Production fault-injection hooks, caller clocks, deterministic UUID injection and arbitrary prepared-value constructors remain forbidden.

## Regression acceptance

Preserve all accepted suites, including:

- J24K3e1;
- J24K3d2;
- J24K3d1;
- J24K3c4;
- J24K3c3;
- J24K3c2;
- J24K3c1;
- J24K3b;
- J24K3a;
- J24K2;
- J24J;
- installed-state and M3 lifecycle suites.

Run full serial `just verify` with `RUST_TEST_THREADS=1`.

## Checkpoint procedure

1. Change task and worker-note status `READY` to `IN_PROGRESS`.
2. Implement production code and direct tests.
3. Commit production code and tests.
4. Capture the exact implementation SHA using `git rev-parse HEAD`.
5. Verify it resolves using `git cat-file -e "<sha>^{commit}"`.
6. Record that SHA as `Implementation checkpoint` in both documents.
7. Run direct tests, focused Nextest, named regressions and full serial verification at that exact checkpoint.
8. Add the required `## Changes made` worker-note section before changing status to `COMPLETE`.
9. Record evidence honestly, including any test not run.
10. Commit verification documentation only.
11. Capture and verify the exact verification SHA.
12. Record it as `Verification checkpoint` in both documents through a final documentation-only commit.
13. Run the task packet checker and require PASS.
14. Require `cargo fmt --all -- --check`, `git diff --check` and clean `git status`.
15. Push the branch and report the exact final remote tip.

Do not put a self-referential final remote-tip field into either document.

## Required handoff

Report:

- branch name;
- final remote tip;
- implementation checkpoint;
- verification checkpoint;
- exact production and test files changed;
- direct and focused test counts;
- named regression results;
- full serial verification result;
- Cargo.lock SHA-256 and unchanged status;
- task-packet checker result;
- `cargo fmt --check`, `git diff --check` and clean-status results;
- any earlier authoritative error classification that differs from a packet prediction;
- remaining risks and the smallest next action.

Stop on any failed verification, changed Cargo.lock, unexplained file, branch mismatch, non-fast-forward history, or scope expansion.
