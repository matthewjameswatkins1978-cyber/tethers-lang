# Current Implementation Task

Control contract: `1`
Task: `J24K3d1 - Validated read-only installation recovery plan`
Owner: `OpenCode`
Status: `READY`
Task colour: `Red`
Route: `OpenCode using DeepSeek Pro for one bounded Rust composition and recovery-planning package; Lucy performs independent review and routine safe merge`
Base branch: `opencode/j24k3d1-validated-recovery-plan`
Base commit: `e2cffcb93fdd457cadf2091b8657e7e6a4e8a5a2`
Implementation branch: `opencode/j24k3d1-validated-recovery-plan`
Worker note: `docs/worker-notes/2026-08-05-j24k3d1-validated-recovery-plan.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Accepted main: `20cd25f328568aa2726505580689d67b6219449c`

## Objective

Implement only J24K3d1: one crate-private, read-only recovery-planning boundary that composes the accepted J24K3a through J24K3c4 primitives.

Given the typed installation request and host-owned recovery stores, the planner must return either:

- no pending publication intent; or
- one exact validated recovery disposition paired with the exact loaded publication intent.

Before returning a mutation-bearing disposition, it must complete every read-only proof required for that disposition. It must not perform the mutation itself.

The governing sequence is:

```text
load current intent from the authoritative store
  -> audit the global installed-root namespace
  -> if no intent: return no pending recovery
  -> observe the exact transaction state
  -> classify with the accepted pure classifier
  -> conditionally revalidate evidence and destination
  -> return one sealed validated recovery plan
```

This package must not delete staging, publish a record, remove an intent, acquire a lock, call J24J, or wire `installation_execution.rs`.

## Relevant background and existing behaviour

Accepted `main` is exactly:

```text
20cd25f328568aa2726505580689d67b6219449c
```

The accepted recovery foundation now provides:

- J24K3a: `InstallationPublicationIntent` and authoritative single-current-intent store;
- J24K3b: pure `classify_installation_recovery` with four accepted dispositions;
- J24K3c1: exact staging, destination, and installed-record observation;
- J24K3c2: exact intent-destination file-set, hash, length, permission, and path verification;
- J24K3c3: complete request, candidate, exact-trust, launch, conformance, approval, and installed-record evidence revalidation;
- J24K3c4: global direct `plug-*` namespace accounting against installed records and the optional current intent.

These seams are individually accepted but are not yet composed. A later mutation package must not independently choose which proofs to run or accept a caller-supplied intent that can omit the authoritative current transaction.

`InstallationExecutionContext` still has no executor-state root or intent store and `PublishDisabledInstallation` remains deliberately deferred. Do not change that boundary in this package.

## Required behaviour

### 1. Add one narrow recovery-planning module

Add a private module structurally equivalent to:

```rust
pub(crate) struct InstallationRecoveryPlanningContext<'a> {
    pub intents: &'a InstallationPublicationIntentStore,
    pub installed: &'a InstalledPlugRegistry,
    pub evidence: InstallationRecoveryEvidenceContext<'a>,
}

pub(crate) struct ValidatedInstallationRecoveryPlan {
    // private fields
}

pub(crate) fn plan_installation_recovery(
    request: &InstallationRequest,
    context: &InstallationRecoveryPlanningContext<'_>,
) -> Result<ValidatedInstallationRecoveryPlan>;
```

The exact field arrangement may be narrowed for borrowing clarity, but the semantic boundary is frozen.

The caller must not supply:

- an optional intent;
- staging, destination, or record-presence booleans;
- a preclassified disposition;
- arbitrary paths or roots outside accepted store objects;
- callbacks, mutation capabilities, allow-lists, or repair policy.

The planner loads the current intent itself from `InstallationPublicationIntentStore`.

### 2. Return one sealed plan with no caller-constructible inconsistent state

`ValidatedInstallationRecoveryPlan` must have private fields and no public or crate-visible arbitrary constructor.

It may expose narrow crate-private read access structurally equivalent to:

```rust
pub(crate) fn intent(&self) -> Option<&InstallationPublicationIntent>;
pub(crate) fn disposition(&self) -> Option<InstallationRecoveryDisposition>;
pub(crate) fn is_idle(&self) -> bool;
```

Its invariant is exact:

- idle plan: no intent and no disposition;
- pending plan: one validated intent and one disposition;
- mixed states are unrepresentable outside the module.

Do not carry mutable stores, filesystem handles, arbitrary paths, snapshots, caller strings, or callbacks in the returned plan.

### 3. Load the authoritative current intent first

Call `InstallationPublicationIntentStore::load()` before inspecting the installed root or transaction paths.

- malformed, torn, duplicate, unknown, or unsafe intent state retains the accepted intent-store error;
- no caller may suppress a pending intent by passing `None`;
- do not create, repair, rewrite, or remove the intent.

### 4. Always run the global installed-root audit

After loading the optional current intent, call:

```rust
installed.audit_installation_recovery_destinations(intent.as_ref())
```

This happens for both `Some` and `None`.

When there is no intent, the planner must still reject an orphan or malformed direct `plug-*` final destination. Absence of a transaction is not permission to ignore global installed-state corruption.

Preserve accepted recovery-facing errors unchanged. Do not remap them to a new planning error.

### 5. Return idle only after a successful no-intent audit

When the authoritative intent store returns `None` and the global audit succeeds, return one idle validated plan.

Do not observe a synthetic transaction, inspect evidence stores, create a placeholder intent, or validate an unrelated request against a nonexistent transaction.

### 6. Observe and classify one current transaction

When an intent exists:

1. call `InstalledPlugRegistry::observe_installation_recovery(&intent)`;
2. call `classify_installation_recovery(snapshot.as_observation(&intent))`;
3. preserve the exact accepted disposition.

Do not reproduce the state table, compare booleans independently, or add a second classifier.

Contradictory state remains `installation_recovery_conflict`.

### 7. Apply disposition-specific read-only proofs

For these dispositions:

```text
RemoveIntentOnly
RemoveStagingThenIntent
```

return the pending validated plan after successful load, global audit, observation, and classification.

Do not require current package evidence for cleanup-only recovery. Stale or missing candidate/trust/conformance/approval state must not prevent removal of an intent whose durable publication never began, or removal of its exact private staging directory by a later package.

For these dispositions:

```text
RevalidateDestinationThenPublishRecord
VerifyCompletedPublicationThenRemoveIntent
```

before returning the plan, call in this order:

1. `revalidate_installation_recovery_evidence(request, &intent, &context.evidence)`;
2. `installed.verify_installation_recovery_destination(&intent)`.

Both publication-ready and completed-publication recovery must prove current evidence and exact destination bytes. A matching installed record alone is not enough to clear the intent.

Preserve accepted errors unchanged:

```text
installation_intent_invalid
installation_intent_evidence_stale
installation_destination_untracked
installation_recovery_conflict
installation_recovery_io
unsafe_store_path
```

No lower-layer path, JSON, record string, or operating-system diagnostic may be introduced by this module.

### 8. Remain strictly read-only

The planner must not change:

- intent entries or metadata;
- staging, destination, or unrelated install-root entries;
- installed records;
- candidate, trust, launch-profile, conformance, or approval stores;
- timestamps or permissions.

It must not call:

- `InstallationPublicationIntentStore::create`;
- `InstallationPublicationIntentStore::remove_if_matches`;
- `fs::remove_file`, `fs::remove_dir`, or `fs::remove_dir_all`;
- installed record creation or ordinary installation publication;
- lock acquisition;
- J24J planning or J24K execution.

### 9. Add direct production-entry-point tests

Add one private test module whose test names begin `j24k3d1`.

Directly prove at minimum:

- empty intent, install, and record roots return an idle plan;
- no intent plus one untracked canonical final destination fails as `installation_destination_untracked`;
- malformed or torn authoritative intent fails before installed-root auditing can influence the result;
- intent only returns `RemoveIntentOnly` without requiring candidate or evidence stores to contain current evidence;
- exact staging only returns `RemoveStagingThenIntent` without requiring current package evidence;
- destination only plus complete current evidence returns `RevalidateDestinationThenPublishRecord`;
- matching destination and exact installed record plus complete current evidence returns `VerifyCompletedPublicationThenRemoveIntent`;
- staging plus destination fails as `installation_recovery_conflict`;
- record without destination fails closed;
- an authorised intent destination does not excuse a second untracked final destination;
- destination-only recovery with stale request, candidate, exact trust, launch, conformance, approval, or installed-record pins fails as `installation_intent_evidence_stale`;
- destination file-set, digest, length, permission, or reparse drift fails through the accepted recovery error;
- completed-publication recovery still revalidates evidence and destination before returning a plan;
- invalid or unsafe roots retain the accepted stable errors;
- every successful planning route leaves exact entry sets, bytes, modification timestamps, and read-only permissions unchanged across the intent root, install root, record root, candidate/quarantine, exact-trust, launch-profile, conformance, and approval roots.

Exercise `plan_installation_recovery`. Do not test only helper methods, source strings, or the accepted lower-level seams in isolation.

Platform-gated reparse fixtures are allowed. Do not use unsafe representation tricks to invent impossible enum states.

## Relevant components

- `tethers-0.1/host-rust/src/installation_recovery_plan.rs`
- `tethers-0.1/host-rust/src/installation_recovery_plan_tests.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_request.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `InstallationPublicationIntentStore`
- `InstallationRecoveryEvidenceContext`
- `InstallationRecoveryDisposition`
- `classify_installation_recovery`
- `InstalledPlugRegistry::observe_installation_recovery`
- `InstalledPlugRegistry::verify_installation_recovery_destination`
- `InstalledPlugRegistry::audit_installation_recovery_destinations`
- `revalidate_installation_recovery_evidence`

The accepted lower-level production modules are reference-only unless one minimal visibility change is required to compose an already accepted crate-private seam. Do not change their semantics.

## Frozen decisions and invariants

- The intent is loaded from the authoritative store, never supplied by the caller.
- Intent loading precedes installed-root and transaction observation.
- Global installed-root audit always runs, including when no intent exists.
- The accepted pure classifier remains the only state-table authority.
- Cleanup-only dispositions do not require current package evidence.
- Destination publication and completed-intent removal require both current evidence and exact destination verification.
- The returned plan contains no mutation capability and cannot represent intent/disposition mismatch.
- No mutation, lock, J24J planning, executor wiring, or public API is added.
- No dependency, Cargo configuration, Cargo.lock, request schema, evidence schema, CLI, packaging, release, enablement, operational-scope, or OCaml change is permitted.
- Workers record implementation and verification checkpoints only. They must not place a supposed final branch SHA inside a file committed on that branch. Lucy records the reviewed remote tip externally after review.

## Acceptance criteria

1. One crate-private planner loads the authoritative current intent itself.
2. The global installed-root audit runs for both intent-present and intent-absent state.
3. Idle is returned only after a successful no-intent global audit.
4. Intent-present state is observed and classified only through accepted production seams.
5. Cleanup-only dispositions do not require package evidence freshness.
6. Destination publication and completed-publication dispositions require both evidence and destination revalidation.
7. Accepted stable errors are preserved without lower-layer detail leakage.
8. The returned plan has private invariant-preserving state and no mutation capability.
9. The planner performs no durable or metadata mutation.
10. Direct tests exercise every disposition and the idle route through the production planner.
11. Focused Nextest passes with zero retries and all `j24k3d1` tests pass.
12. J24K3c4, J24K3c3, J24K3c2, J24K3c1, J24K3b, J24K3a, J24K2, J24J, and M3 lifecycle regressions remain green.
13. Full serial `just verify` and the task packet checker pass.
14. Cargo.lock remains byte-identical and only permitted files change.
15. The task packet and worker note record exact commands, counts, implementation checkpoint, verification checkpoint, discoveries, and risks without self-referential final-tip fields.

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
  -E 'test(j24k3d1)'

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

$env:PATH = "$PSHOME;$env:PATH"
$env:RUST_TEST_THREADS = "1"
just verify

Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
git diff --check
git status --short
git log --oneline --decorate -14
```

Cargo.lock must remain:

```text
D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

The completed packet must record the commit on which the packet checker and full verification were actually run. Do not update that commit field afterward merely to make it equal the branch tip; doing so creates a new unverified commit and restarts the SHA chase.

## Forbidden changes

- No edit to the frozen architecture.
- No mutation of intent, staging, destination, installed records, or evidence stores.
- No intent creation or removal.
- No staging cleanup.
- No installed-record publication.
- No lock acquisition or lock-context changes.
- No J24J planner call or `installation_execution.rs` wiring.
- No public API, schema, dependency, Cargo configuration, Cargo.lock, CLI, packaging, release, enablement, operational-scope, or OCaml change.
- No second recovery state table or duplicated evidence/destination validator.
- No caller-supplied intent, booleans, snapshot, disposition, paths, allow-list, callback, or repair policy.
- No self-referential `Final remote tip` field in committed task or worker documentation.
- No unrelated refactor or broad test framework.

Permitted files:

- `tethers-0.1/host-rust/src/installation_recovery_plan.rs`;
- `tethers-0.1/host-rust/src/installation_recovery_plan_tests.rs`;
- `tethers-0.1/host-rust/src/lib.rs` only for private module and test registration;
- one minimal visibility-only edit in an accepted lower-level module if compilation proves it essential;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-05-j24k3d1-validated-recovery-plan.md`.

## Stop conditions

Stop as `BLOCKED` only if composition requires changing a public API, accepted recovery semantics, request or evidence schema, dependency, Cargo.lock, or performing mutation; or if full verification still fails after one evidence-led correction.

Do not stop for failed LSP, a stale local ref, building complete private fixtures, adding platform-gated tests, serialising the known Windows handle-contention suite, or making one narrow crate-private visibility adjustment.

## Expected pre-existing changes

The branch begins from accepted main and contains only the J24K3d1 worker-note scaffold at the Base commit. No J24K3d1 production code or tests exist yet.
