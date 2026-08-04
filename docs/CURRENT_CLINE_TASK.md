# Current Implementation Task

Control contract: `1`
Task: `J24K2 - Non-inheritable RAII lock and single-step executor`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Red`
Route: `OpenCode using DeepSeek Pro V4 for bounded security-sensitive Rust implementation; Lucy performs independent review and routine safe merge`
Base branch: `main`
Base commit: `9dc4498b644317e99851879cd40f2874eb611298`
Implementation branch: `opencode/j24k2-locked-single-step-executor`
Worker note: `docs/worker-notes/2026-08-04-j24k2-locked-single-step-executor.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Implementation checkpoint: `PENDING`

## Objective

Implement J24K2: the Windows non-inheritable RAII installation lock and the bounded single-step installation executor.

One invocation must:

```text
acquire lock
  -> plan with J24J inside the lock
  -> execute zero or one supported action
  -> re-plan
  -> validate the action-specific postcondition
  -> release the lock last
```

J24K2 executes:

- `CreateExactCandidateTrust`;
- `RunSupervisedConformance`;
- `CreateInstallationApproval`;
- `Complete`.

`PublishDisabledInstallation` is recognised but must fail closed without mutation until J24K3 adds the publication intent and recovery authority.

J24K2 does not add the J24L four-call driver, publication intent, installed-root recovery, CLI, prompts, output styling, or enablement.

## Relevant background and existing behaviour

J24K1 is accepted on `main` and provides:

- crate-private `CurrentTrustAuthority`;
- `ExactCandidateTrustAuthority` backed only by `ExactCandidateTrustStore`;
- explicit authority-aware conformance, approval, and installation internals;
- behavioural proof that supplied authority reaches every downstream trust check.

J24J provides the pure planner:

```rust
pub fn plan_installation(
    request: &InstallationRequest,
    candidates: &CandidateRegistry,
    exact_trust: &ExactCandidateTrustStore,
    launch_profiles: &LaunchProfileEvidenceStore,
    conformance: &ConformanceEvidenceStore,
    approvals: &InstallationApprovalStore,
    installed: &InstalledPlugRegistry,
) -> Result<InstallationPlan>;
```

Accepted action progression:

```text
CreateExactCandidateTrust
  -> RunSupervisedConformance
  -> CreateInstallationApproval
  -> PublishDisabledInstallation
  -> Complete
```

Accepted baseline:

```text
Rust             1.97.1
Cargo tests      940 passing minimum before J24K2
Nextest retries  0
Cargo.lock       D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

The frozen J24K architecture is authoritative. Do not edit it.

## Required behaviour

### 1. New execution module

Add:

```text
tethers-0.1/host-rust/src/installation_execution.rs
```

Expose it from `lib.rs` as:

```rust
pub mod installation_execution;
```

The public seam must be structurally equivalent to:

```rust
pub struct InstallationExecutionContext<'a> {
    pub lock_path: &'a Path,
    pub quarantine_root: &'a Path,
    pub conformance_scratch_root: &'a Path,
    pub candidates: &'a CandidateRegistry,
    pub exact_trust: &'a ExactCandidateTrustStore,
    pub launch_profiles: &'a LaunchProfileEvidenceStore,
    pub conformance: &'a ConformanceEvidenceStore,
    pub approvals: &'a InstallationApprovalStore,
    pub installed: &'a InstalledPlugRegistry,
}

pub struct InstallationExecutionOptions<'a> {
    pub approving_authority: &'a str,
    pub host_build_identity: &'a str,
    pub conformance_wall_time: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationStepResult {
    pub before: InstallationPlan,
    pub after: InstallationPlan,
    pub outcome: InstallationStepOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallationStepOutcome {
    AlreadyComplete,
    Advanced {
        executed: InstallationPlanAction,
    },
    ConformanceRecordedWithoutAdvance {
        evidence_id: String,
        disposition: ConformanceDisposition,
    },
}

pub fn execute_next_installation_action(
    request: &InstallationRequest,
    context: &InstallationExecutionContext<'_>,
    options: &InstallationExecutionOptions<'_>,
) -> Result<InstallationStepResult>;
```

A narrowly cleaner field order is acceptable. The semantic inputs and outputs are frozen.

Validate non-empty `approving_authority`, non-empty `host_build_identity`, and a positive conformance wall-time before mutation. Invalid options fail with:

```text
code: installation_execution_options_invalid
message: installation execution options are invalid
```

### 2. Outer lock scope

The public function must contain the outer lifetime boundary:

```rust
pub fn execute_next_installation_action(...) -> Result<InstallationStepResult> {
    let _lock = InstallationLockGuard::acquire(context.lock_path)?;
    execute_installation_action_while_locked(request, context, options)
}
```

All planner, candidate, trust, launch, conformance, approval, scratch, and postcondition values live in the inner function. The lock therefore drops after every other local on normal return and panic unwind.

The function must never accept a precomputed plan.

### 3. Windows lock abstraction

Add a private owned guard:

```rust
struct InstallationLockGuard {
    file: std::fs::File,
}
```

On Windows:

- `lock_path` must be absolute;
- its parent must already exist, be a directory, and be reparse-safe;
- any existing lock anchor must be an ordinary, empty, non-reparse file;
- acquire with `OpenOptionsExt::share_mode(0)` and no polling;
- Windows sharing or lock violations (`ERROR_SHARING_VIOLATION` / `ERROR_LOCK_VIOLATION`) map to `installation_busy`;
- explicitly clear `HANDLE_FLAG_INHERIT` using `SetHandleInformation`;
- verify the opened path again after acquisition;
- write no PID, timestamp, owner, or other bytes;
- dropping the owned `File` releases the lock;
- the empty anchor may remain permanently.

On non-Windows, acquisition fails before planning with `installation_lock_invalid`; do not invent a stale-file pseudo-lock.

Stable lock errors:

```text
installation_busy
installation_lock_invalid
installation_lock_io
```

Stable messages:

```text
installation_busy: another installation action is already running
installation_lock_invalid: installation lock path is invalid
installation_lock_io: installation lock could not be acquired
```

### 4. Locked planning and exact candidate loading

After lock acquisition, call J24J.

Load exactly one candidate matching `before.candidate_id` from the accepted registry. Validate it and require its package ID, package version, and semantic digest to equal the plan.

Missing, duplicate, invalid, or mismatched evidence fails closed using existing planner/candidate errors or:

```text
code: installation_execution_plan_stale
message: installation plan no longer matches current evidence
```

Every record used for an action must be reloaded from its current store and checked against the exact IDs and digests pinned in `before`. Do not trust detached caller objects or merely match by package name.

### 5. CreateExactCandidateTrust

For `CreateExactCandidateTrust`:

- call `ExactCandidateTrustStore::create(candidate, request, options.approving_authority)`;
- perform no other mutation;
- re-plan inside the same lock;
- require exactly:

```text
CreateExactCandidateTrust -> RunSupervisedConformance
```

The `after` plan must pin the created exact-trust record digest and reconstructed package-trust evidence digest.

Return:

```rust
InstallationStepOutcome::Advanced {
    executed: InstallationPlanAction::CreateExactCandidateTrust,
}
```

### 6. RunSupervisedConformance

For `RunSupervisedConformance`:

1. reload and validate the exact-candidate trust record pinned by `before`;
2. reconstruct `PackageTrustEvidence::exact_candidate` and require its digest matches `before`;
3. construct `ExactCandidateTrustAuthority` explicitly;
4. call `PreparedSupervisedLaunch::prepare` with the exact candidate, quarantine root, scratch root, and bounded wall-time;
5. persist the prepared `LaunchProfileEvidence` through `LaunchProfileEvidenceStore::create`;
6. call `run_host_conformance_with_authority` with the exact authority;
7. persist the returned `ConformanceEvidence` through `ConformanceEvidenceStore::create`;
8. explicitly clean the prepared scratch directory;
9. re-plan and validate the outcome.

No legacy publisher/developer authority may be constructed or consulted.

A private scratch guard must own the prepared launch and perform best-effort cleanup on unwind. Ordinary success performs explicit cleanup.

If launch-profile or conformance evidence has been durably published and explicit scratch cleanup then fails, return:

```text
code: installation_scratch_cleanup_failed
message: conformance scratch cleanup failed after evidence publication
```

Do not remove durable evidence to conceal cleanup failure.

Passed conformance requires:

```text
RunSupervisedConformance -> CreateInstallationApproval
```

The `after` plan must pin the exact newly persisted launch-profile and conformance IDs/digests.

Failed or interrupted conformance may legitimately produce:

```text
RunSupervisedConformance -> RunSupervisedConformance
```

Return:

```rust
InstallationStepOutcome::ConformanceRecordedWithoutAdvance {
    evidence_id,
    disposition,
}
```

J24K2 must stop. It must not retry conformance or perform another action.

Any other same-action result is stagnant.

### 7. CreateInstallationApproval

For `CreateInstallationApproval`:

- reload the exact trust, package-trust evidence, launch profile, and passed conformance exactly pinned by `before`;
- construct `ExactCandidateTrustAuthority` explicitly;
- call `InstallationApprovalStore::approve_with_authority`;
- perform no other mutation;
- re-plan;
- require exactly:

```text
CreateInstallationApproval -> PublishDisabledInstallation
```

The `after` plan must pin the newly created approval ID and digest while retaining every prior pin.

Return `Advanced` for `CreateInstallationApproval`.

### 8. Deferred publication boundary

For `PublishDisabledInstallation`, J24K2 must perform no mutation and return:

```text
code: installation_publication_deferred
message: disabled installation publication requires J24K3
```

Do not call `InstalledPlugRegistry::install_disabled_with_authority` in J24K2.

Do not create staging directories, final destinations, installed records, executor-state roots, or recovery intent.

J24K3 will replace this temporary fail-closed boundary with crash-safe publication.

### 9. Complete

For `Complete`:

- perform no mutation;
- call J24J again while still locked;
- require `before == after`;
- return `AlreadyComplete`.

### 10. Post-plan and transition validation

Initial planning errors and action-seam errors retain their existing safe codes.

If the action mutates durable state but the immediate second J24J call fails, return:

```text
code: installation_execution_postcondition_failed
message: installation state could not be reconciled after mutation: <safe underlying code>
```

Do not roll back immutable evidence. The next invocation must be able to resume from durable state after the underlying inconsistency is corrected.

Add a private transition validator. Rank actions in accepted order and fail as follows:

Same action, except recorded failed/interrupted conformance:

```text
installation_execution_stagnant
```

Backward action:

```text
installation_execution_regressed
```

Skipped or contradictory action:

```text
installation_execution_invalid_transition
```

Expected action but missing, changed, or contradictory pins:

```text
installation_execution_postcondition_failed
```

Stable messages may include the safe before/after action names but no paths, untrusted stderr, or package-controlled text.

### 11. One-mutation invariant

No action handler may call another action handler.

No loop over planner actions is permitted.

One invocation creates at most one of these logical state transitions:

- one exact-candidate trust record;
- one launch-profile plus one conformance record as the single conformance action;
- one installation approval record;
- no mutation for complete or deferred publication.

## Relevant components

- `tethers-0.1/host-rust/src/installation_execution.rs`
- `tethers-0.1/host-rust/src/installation_execution_tests.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/tests/j24k2_locked_single_step_executor.rs`
- `installation_plan::plan_installation`
- `installation_plan::{InstallationPlan, InstallationPlanAction}`
- `current_trust::ExactCandidateTrustAuthority`
- `PreparedSupervisedLaunch`
- `run_host_conformance_with_authority`
- `InstallationApprovalStore::approve_with_authority`
- accepted candidate and evidence stores
- `m3_store::{verify_chain, reject_reparse, M3Error, Result}`
- Windows `OpenOptionsExt`, raw handles, and `SetHandleInformation`

## Frozen decisions and invariants

- J24K2 is Windows-lock authoritative; non-Windows execution fails closed.
- The lock is the exclusively held non-inheritable file handle, not file existence.
- Lock acquisition is immediate and never polls.
- Planning occurs only after lock acquisition.
- The lock outer scope drops last.
- Every action reloads and validates currently pinned evidence.
- Exact-candidate actions use only `ExactCandidateTrustAuthority`.
- Existing J24K1 legacy wrappers remain unchanged.
- One invocation performs zero or one logical mutation.
- Failed and interrupted conformance are recorded but never automatically retried.
- `PublishDisabledInstallation` is fail-closed and mutation-free until J24K3.
- No executor-state root, publication intent, recovery, installed-root audit, adoption, deletion, or repair belongs to J24K2.
- No J24L multi-call driver, CLI, prompt, terminal output, enablement, operational-scope, schema, dependency, Cargo configuration, OCaml, or Cargo.lock change is permitted.

## Acceptance criteria

1. A non-inheritable exclusive Windows lock guard exists and is private.
2. A second acquisition fails immediately with `installation_busy`.
3. Ordinary return, error, and panic unwind release the lock.
4. A supervised child cannot retain the lock handle after the parent guard drops.
5. Busy lock refusal occurs before request validation or planning.
6. The public execution seam matches the frozen semantic boundary.
7. No precomputed plan can be supplied.
8. `CreateExactCandidateTrust` performs only that stage and advances exactly once.
9. Passed conformance persists current launch/conformance evidence and advances exactly once.
10. Failed and interrupted conformance persist evidence, return `ConformanceRecordedWithoutAdvance`, and do not retry.
11. Candidate tampering after launch preparation is refused by existing revalidation before provider execution.
12. Approval creation uses exact authority and advances exactly once.
13. `PublishDisabledInstallation` performs no mutation and returns the frozen deferred error.
14. `Complete` performs no mutation and returns equal before/after plans.
15. Pure transition tests cover stagnant, regressed, skipped, and pin-mismatch refusal.
16. A post-plan failure after durable mutation releases the lock and leaves state resumable.
17. Existing J24K1 authority tests and representative M3/J24J suites remain green.
18. Focused Nextest runs with zero retries.
19. Full verification passes with at least the accepted 940-test baseline plus new tests.
20. Cargo.lock remains byte-identical and only permitted files change.

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
  -E 'test(j24k2)'

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k2 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test j24k2_locked_single_step_executor `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k1 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test j24j_installation_reconciliation `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test m3_lifecycle `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test j23c2_pdf_conformance `
  --locked

$env:PATH = "$PSHOME;$env:PATH"
just verify

Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
git diff --check
git status --short
```

The focused Nextest expression may be adjusted once if discovery reports a different exact test name. Record exact executed and skipped counts. Do not repeat ineffective filters blindly.

Use the existing `m3_fixture_provider` binary and accepted M3 fixture patterns for real conformance tests. Do not replace behavioural execution with mocks where the packet requires provider-process evidence.

OpenCode LSP is not a gate. Do not spend task time diagnosing empty LSP results.

Cargo.lock must remain:

`D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`

No full-verification failure is acceptable after `$PSHOME` is prepended to PATH.

## Forbidden changes

- No edit to `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`.
- No change to J24K1 trust semantics or legacy public wrappers.
- No optional, default, global, static, or thread-local trust authority.
- No caller-supplied plan.
- No internal action loop or second mutation.
- No automatic conformance retry.
- No call to installed publication from the deferred action.
- No publication intent, recovery matrix, installed-root audit, adoption, deletion, rollback, or repair.
- No J24L driver, CLI, prompt, terminal styling, enablement, operational-scope, packaging, release, or OCaml work.
- No schema, dependency, Cargo configuration, tool configuration, or Cargo.lock changes.
- No production test-only constructor or bypass.
- No files outside the permitted set.

Permitted files:

- `tethers-0.1/host-rust/src/installation_execution.rs`;
- `tethers-0.1/host-rust/src/installation_execution_tests.rs`;
- `tethers-0.1/host-rust/src/lib.rs`;
- `tethers-0.1/host-rust/tests/j24k2_locked_single_step_executor.rs`;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-04-j24k2-locked-single-step-executor.md`.

## Stop conditions

Stop as `BLOCKED` only if:

- Windows exclusive file-handle locking cannot be implemented using existing dependencies and standard/windows-sys APIs;
- the lock cannot be made explicitly non-inheritable;
- existing store visibility prevents the single-step executor without public evidence-bypass APIs;
- exact-candidate conformance or approval cannot use the accepted J24K1 authority seams;
- safe implementation requires publication intent, recovery, schema, dependency, CLI, enablement, or out-of-scope files;
- required verification still fails after one evidence-led correction.

Do not stop for failed LSP, one ineffective Nextest filter, a stale local branch ref, or a failed broad text replacement. Reread the current file and make one smaller evidence-led correction.

## Expected pre-existing changes

The branch already contains the documentation-only preparation commit for:

- `docs/worker-notes/2026-08-04-j24k2-locked-single-step-executor.md`;
- this `docs/CURRENT_CLINE_TASK.md` packet.

The frozen J24K architecture and accepted J24K1 implementation are inherited from `main`. Do not revert or edit them.
