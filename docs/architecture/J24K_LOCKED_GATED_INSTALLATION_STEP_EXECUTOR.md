# J24K Locked Gated Installation Step Executor

Status: FROZEN
Owner: Lucy
Implementation owner: OpenCode
Planning base: `db84c71dc92381921cdc05c62029a1899c13d7f2`

## 1. Purpose

J24K is the host-owned execution boundary between the accepted J24J read-only installation planner and the future J24L public Plug-install CLI.

J24K performs at most one legitimate durable installation mutation per invocation.

Its invariant is:

```text
acquire lock
  -> recover or reject private executor state
  -> reconcile current durable evidence with J24J
  -> execute zero or one exact action
  -> reconcile again
  -> prove the action-specific postcondition
  -> release the lock last
```

J24K does not contain an internal four-step installation loop. J24L will later call the single-step primitive at most four times, using a fresh plan and fresh lock for each step.

## 2. Accepted foundations

J24K builds on accepted authority only:

- J24G typed installation request and disabled target state;
- J24H read-only evidence-store access and launch-profile persistence;
- J24I immutable exact-candidate installation trust;
- J24J pure installation reconciliation planning;
- existing candidate revalidation, supervised launch, conformance, installation approval, immutable installed-state publication, and M3 store primitives.

The logical installation progression remains:

```text
CreateExactCandidateTrust
  -> RunSupervisedConformance
  -> CreateInstallationApproval
  -> PublishDisabledInstallation
  -> Complete
```

## 3. Delivery decomposition

J24K is implemented as three bounded packages.

### J24K1: explicit current-trust authority foundation

Introduce a crate-private authority abstraction that allows the existing conformance, approval, and installation seams to require an explicit current-trust authority.

The existing signed-publisher and unsigned-developer paths retain their public signatures and behaviour. Exact-candidate authority is added without global state, fallback, or policy leakage.

### J24K2: non-inheritable RAII lock and single-step executor

Add the Windows host installation lock, open authorities inside its lifetime, run J24J, and execute the trust, conformance, approval, or complete actions with action-specific postconditions.

### J24K3: crash-safe disabled installation publication

Add private publication intent, recovery, installed-root consistency audit, and the crash-safe `PublishDisabledInstallation` action.

J24L remains separate.

## 4. Final public execution seam

The final J24K public seam will be structurally equivalent to:

```rust
pub struct InstallationExecutionContext<'a> {
    pub lock_path: &'a Path,
    pub executor_state_root: &'a Path,
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

pub struct InstallationStepResult {
    pub before: InstallationPlan,
    pub after: InstallationPlan,
    pub outcome: InstallationStepOutcome,
}

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

The exact field arrangement may be narrowed during J24K2 if compilation proves an already-accepted store bundle is cleaner. The semantic boundary is frozen.

## 5. Single-step invariant

One invocation performs zero or one durable mutation.

- `Complete` performs no mutation and returns `AlreadyComplete`.
- every other successful ordinary action must advance exactly one stage;
- passed conformance must advance to `CreateInstallationApproval`;
- failed or interrupted conformance may legitimately remain at `RunSupervisedConformance` and returns `ConformanceRecordedWithoutAdvance`;
- J24K never automatically retries conformance;
- J24K never executes a second mutation in the same invocation.

J24K never accepts a caller-supplied precomputed plan. It plans only after acquiring the lock.

## 6. Explicit current-trust authority

J24K1 introduces a crate-private abstraction equivalent to:

```rust
pub(crate) trait CurrentTrustAuthority {
    fn revalidate_current(
        &self,
        candidate: &CandidateRecord,
        evidence: &PackageTrustEvidence,
        now_unix_ms: u64,
    ) -> Result<()>;
}
```

Implementations:

```rust
PublisherDeveloperTrustAuthority<'a>
ExactCandidateTrustAuthority<'a>
```

Every internal authority-aware seam requires an explicit authority argument.

Forbidden forms:

- `Option<&dyn CurrentTrustAuthority>`;
- implicit default authority;
- fallback from exact-candidate mode to publisher or developer trust;
- global, static, or thread-local authority state;
- mutation of publisher/developer stores to simulate exact-candidate trust.

The legacy implementation delegates to the existing `PackageTrustEvidence::revalidate_current` behaviour.

The exact-candidate implementation must:

1. validate the candidate and package-trust evidence;
2. require `TrustModeEvidence::ExactCandidate`;
3. load the current exact-candidate record by candidate ID;
4. call `record.require_for_candidate(candidate)`;
5. compare candidate ID, candidate-record digest, installation-trust record digest, and approving authority;
6. reconstruct `PackageTrustEvidence::exact_candidate(&record)`;
7. require exact equality with the supplied evidence.

Missing or changed exact-candidate authority fails closed.

Current trust authority validates authority only. It does not absorb candidate-byte integrity. Existing candidate and launch revalidation boundaries remain mandatory and must not be weakened.

## 7. Authority-aware internal seams

The implementation will add crate-private authority-aware forms equivalent to:

```rust
PreparedSupervisedLaunch::revalidate_current_trust_with(...)
PreparedSupervisedLaunch::launch_for_candidate_with(...)
run_host_conformance_with_authority(...)
InstallationApprovalStore::approve_with_authority(...)
InstalledPlugRegistry::install_disabled_with_authority(...)
```

Existing public methods retain their signatures and construct `PublisherDeveloperTrustAuthority` internally.

J24K constructs `ExactCandidateTrustAuthority` explicitly.

## 8. Lock contract

J24K2 adds one Windows host installation lock held by an RAII guard.

The lock is the exclusively held file handle, not the existence of the lock file.

Requirements:

- lock path is absolute and reparse-safe;
- acquisition fails immediately when already held;
- no polling or indefinite waiting;
- lock handle is explicitly non-inheritable;
- supervised provider children cannot retain lock ownership;
- stack unwind and process termination release the handle;
- no PID, timestamp, or mutable owner metadata is written;
- the empty lock anchor may remain permanently.

The public entry point must use an outer lock scope:

```rust
pub fn execute_next_installation_action(...) -> Result<InstallationStepResult> {
    let _lock = InstallationLockGuard::acquire(...)?;
    execute_installation_action_while_locked(...)
}
```

All planner, launch, scratch, and recovery values live in the inner function so the lock drops last.

Stable lock errors:

```text
installation_busy
installation_lock_invalid
installation_lock_io
```

## 9. Action postconditions

### CreateExactCandidateTrust

Expected transition:

```text
CreateExactCandidateTrust -> RunSupervisedConformance
```

### RunSupervisedConformance

J24K prepares the supervised launch, persists launch-profile evidence, runs conformance with exact-candidate authority, persists conformance evidence, and cleans scratch state.

Expected passed transition:

```text
RunSupervisedConformance -> CreateInstallationApproval
```

A failed or interrupted conformance record may leave:

```text
RunSupervisedConformance -> RunSupervisedConformance
```

That is the sole permitted same-action result.

### CreateInstallationApproval

Expected transition:

```text
CreateInstallationApproval -> PublishDisabledInstallation
```

### PublishDisabledInstallation

Expected transition:

```text
PublishDisabledInstallation -> Complete
```

### Complete

No mutation. Before and after plans remain equal.

Ordinary same-action, backward, skipped, or contradictory transitions fail closed.

Stable execution errors:

```text
installation_execution_stagnant
installation_execution_regressed
installation_execution_invalid_transition
installation_execution_postcondition_failed
```

## 10. Crash-safe installed publication

The existing installed publication can rename a verified staging directory to its final `plug-<installed_id>` directory before the immutable installed record is written.

J24K3 adds one private atomic intent outside installed payload directories:

```text
<executor-state-root>/installation-intent/current.json
```

The intent contains the complete precomputed installed record and digest plus the candidate and destination identity needed for recovery.

Intent creation uses canonical JSON, a temporary file, flush, sync, and atomic rename.

The installed record is published unchanged during recovery. Its `created_unix_ms` records when the publication transaction was created. Recovery does not refresh timestamps or recompute record identity.

Publication sequence:

```text
write durable intent
  -> build and verify staging directory
  -> rename staging to final destination
  -> publish exact precomputed installed record
  -> remove intent
```

## 11. Recovery rules

Recovery happens only while holding the installation lock.

Before publishing a destination without a record, J24K must revalidate:

- request and exact candidate;
- current exact-candidate trust record;
- reconstructed package-trust evidence;
- pinned launch profile;
- pinned conformance against the current suite;
- complete installation approval chain;
- precomputed installed record;
- exact destination file set, lengths, hashes, read-only permissions, and path safety.

Recovery never adopts or deletes unexplained final state.

| Intent | Staging | Destination | Record | Result |
|---|---:|---:|---:|---|
| present | absent | absent | absent | remove intent; mutation never began |
| present | present | absent | absent | remove staging; remove intent only after successful cleanup |
| present | absent | present | absent | fully revalidate; publish exact record; remove intent |
| present | absent | present | matching | verify both; remove completed intent |
| present | any | absent | present | fail closed |
| present | present | present | any | fail closed |
| present | absent | present | mismatched | fail closed |
| absent | absent | untracked final directory | absent | fail closed |
| malformed or torn intent | any | any | any | fail closed |

Failed staging cleanup retains the intent.

A malformed intent, temporary intent file, unknown intent-root entry, duplicate intent, stale evidence, mismatched destination, or contradictory state fails closed.

Stable recovery errors include:

```text
installation_intent_invalid
installation_intent_evidence_stale
installation_destination_untracked
installation_recovery_conflict
```

## 12. Installed-root consistency audit

Because final directories use generated installed IDs, orphan detection cannot be limited to a deterministic candidate path.

While holding the global installation lock, J24K3 reconciles every final-form `plug-*` directory against either:

- one validated installed record; or
- the single validated current publication intent.

Any untracked final directory is a global integrity failure. J24K does not adopt it, delete it, or continue with another candidate.

## 13. Panic and cleanup behaviour

- lock release is guaranteed by owned-handle RAII;
- lock scope outlives scratch and recovery guards;
- provider children cannot inherit the lock;
- conformance scratch receives explicit cleanup on ordinary return and best-effort cleanup on unwind;
- cleanup failure is reported without pretending the durable conformance result did not occur;
- generic torn immutable-store temporary files remain fail-closed evidence and are not silently deleted by J24K.

## 14. J24L boundary

J24L later drives J24K:

```text
for at most four calls:
    AlreadyComplete -> success
    Advanced -> continue
    ConformanceRecordedWithoutAdvance -> stop and report
```

A fifth mutation attempt fails with:

```text
installation_iteration_limit
```

J24L contains no action-specific mutation code.

## 15. Required architecture evidence

Across J24K packages, tests must prove:

- exact authority has no legacy fallback;
- signed and unsigned legacy paths retain existing behaviour;
- candidate tampering after launch preparation is refused by existing revalidation;
- a second lock acquisition fails immediately;
- lock releases on ordinary error and unwind;
- provider children do not inherit lock ownership;
- planning occurs inside the lock;
- one invocation performs at most one mutation;
- failed and interrupted conformance record without automatic retry;
- passed conformance advances exactly one stage;
- ordinary same-action and backward transitions fail;
- torn intent fails closed;
- failed staging cleanup retains intent;
- destination digest, file-set, permission, and reparse mismatches refuse recovery;
- valid destination recovery publishes exactly once;
- matching record and destination clear the completed intent;
- record without destination fails closed;
- untracked final destination fails closed;
- post-plan failure releases the lock and leaves durable state resumable.

## 16. Non-goals

J24K does not:

- parse CLI arguments;
- prompt a human;
- print terminal styling or progress UI;
- enable a Plug;
- alter operational scope;
- change installation request JSON;
- change accepted evidence schemas;
- merge exact-candidate trust into publisher policy;
- add an internal multi-mutation loop;
- automatically retry failed conformance;
- roll back immutable evidence;
- silently repair unexplained installed state;
- change OCaml, language semantics, packaging, or release behaviour;
- add dependencies or change Cargo.lock.
