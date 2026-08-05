# Current Implementation Task

Control contract: `1`
Task: `J24K3e1 - Read-only disabled installation publication preparation`
Owner: `OpenCode`
Model: `HY3`
Status: `IN_PROGRESS`
Task colour: `Red`
Route: `OpenCode using HY3 for one bounded Rust publication-preparation package; Lucy performs independent review and routine safe merge`
Base branch: `main`
Base commit: `fe4f0e84569e793be3c0e8818799ac36e895da1a`
Implementation branch: `opencode/j24k3e1-publication-preparation`
Worker note: `docs/worker-notes/2026-08-05-j24k3e1-publication-preparation.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Accepted main: `fe4f0e84569e793be3c0e8818799ac36e895da1a`
Implementation checkpoint: `WORKTREE`
Verification checkpoint: `WORKTREE`

## Objective

Implement only J24K3e1: one crate-private, read-only preparation boundary for a future crash-safe `PublishDisabledInstallation` transaction.

The preparation sequence is:

```text
receive the current ordinary J24J before-plan
  -> create one fresh authoritative J24J plan
  -> require exact plan equality and PublishDisabledInstallation
  -> create one fresh authoritative J24K3d1 recovery plan
  -> require idle recovery state after the global installed-root audit
  -> load and revalidate the exact plan-pinned evidence chain
  -> precompute one immutable disabled installed record
  -> construct one matching publication intent
  -> revalidate the complete prepared intent evidence chain
  -> prove recovery remains idle
  -> return one sealed prepared publication value
```

J24K3e1 generates transaction identity and immutable publication content, but performs no durable mutation.

It must not create the intent file, create or copy staging files, rename staging into the final destination, publish an installed record, remove an intent, acquire a lock, execute an ordinary J24J action, or wire the public executor.

## Relevant background and existing behaviour

Accepted main is exactly:

```text
fe4f0e84569e793be3c0e8818799ac36e895da1a
```

The accepted ordinary installation progression remains:

```text
CreateExactCandidateTrust
  -> RunSupervisedConformance
  -> CreateInstallationApproval
  -> PublishDisabledInstallation
  -> Complete
```

`installation_execution.rs` still returns `installation_publication_deferred` for `PublishDisabledInstallation`.

The accepted crash-safe recovery foundation now provides:

- J24K3a: authoritative private publication intent storage;
- J24K3b: pure recovery classification;
- J24K3c1: exact transaction-state observation;
- J24K3c2: exact destination verification;
- J24K3c3: complete current-evidence revalidation;
- J24K3c4: global installed-root audit;
- J24K3d1: sealed read-only recovery planning;
- J24K3d2: exact recovery mutation execution returning to idle.

The missing ordinary publication path must eventually:

```text
precompute record and intent
  -> create durable intent
  -> build and verify staging
  -> rename staging to final destination
  -> publish exact record
  -> remove intent
```

This package implements only the first line.

## Required behaviour

The following numbered index is checker-facing and restates the ten frozen
required-behaviour subsections without changing their meaning or scope.

1. Add one sealed crate-private prepared publication value.
2. Add one read-only preparation function.
3. Require one fresh exact ordinary plan.
4. Require idle private recovery before preparation.
5. Load and revalidate the exact plan-pinned evidence chain.
6. Precompute one exact immutable disabled installed record.
7. Construct and validate one exact publication intent.
8. Prove preparation remained read-only.
9. Preserve existing recovery and path-safety error classifications.
10. Preserve existing public installation behaviour.

### 1. Add one sealed crate-private prepared publication value

Add a private module structurally equivalent to:

```rust
pub(crate) struct PreparedInstallationPublication {
    intent: InstallationPublicationIntent,
}

impl PreparedInstallationPublication {
    pub(crate) fn intent(&self) -> &InstallationPublicationIntent;
    pub(crate) fn installed_record(&self) -> &InstalledPlugRecord;
}
```

The exact naming and accessor arrangement may be narrowed for Rust clarity, but the semantic boundary is frozen.

Requirements:

- all fields are private;
- no arbitrary constructor;
- no mutable accessor;
- the value owns exactly one validated publication intent containing exactly one precomputed installed record;
- `Debug`, `PartialEq` or `Eq` may be derived only where useful for direct tests and later exact-value checks;
- the prepared value performs no mutation on drop.

It must not expose:

- arbitrary roots or paths;
- a caller-supplied installed record;
- replacement intent data;
- callbacks or mutation functions;
- adoption or repair policy.

### 2. Add one read-only preparation function

Add a crate-private function structurally equivalent to:

```rust
pub(crate) fn prepare_disabled_installation_publication(
    request: &InstallationRequest,
    context: &InstallationRecoveryPlanningContext<'_>,
    before: &InstallationPlan,
) -> Result<PreparedInstallationPublication>;
```

The exact context arrangement may be narrowed if compilation proves a small dedicated borrowed context is cleaner. Do not add a public context or duplicate mutable store ownership.

The function accepts the existing `before` plan only as an exact value to compare against fresh authority. It must never trust caller-supplied plan pins without reloading them from the authoritative stores.

### 3. Require one fresh exact ordinary plan

At function entry, call `plan_installation` using the authoritative stores already reachable through the preparation/recovery context.

Require:

- fresh plan exactly equals `before`;
- action is exactly `InstallationPlanAction::PublishDisabledInstallation`;
- candidate identity and every existing trust, launch, conformance and approval pin remain exact;
- `installed_id` and `installed_record_digest` are absent.

A stale or forged before-plan performs no mutation and fails through the existing execution plan-stale or invalid-transition family.

Do not accept a superficially equivalent plan with changed pins.

### 4. Require idle private recovery before preparation

Call `plan_installation_recovery(request, context)` before generating transaction identity.

Require:

- the recovery plan is idle;
- it contains no intent and no disposition;
- the global installed-root audit has succeeded.

Any present, malformed, torn or contradictory intent blocks preparation.
Any staging, destination, record or global installed-root conflict blocks preparation.
An untracked final destination remains fail-closed.

Do not clean, adopt or alter recovery state.

A non-idle valid recovery plan returns `installation_recovery_conflict` without mutation.
Errors discovered by J24K3d1 retain their accepted recovery-facing classification.

### 5. Load and revalidate the exact plan-pinned evidence chain

Use authoritative stores only.

Load exactly one candidate matching `before.candidate_id` and require exact equality with the before-plan identity fields.

Revalidate the quarantined candidate and path chain.

Load the exact-candidate trust record and require:

- exact candidate match;
- record digest equals `before.exact_candidate_trust_record_digest`;
- reconstructed exact-candidate trust evidence digest equals `before.trust_evidence_digest`;
- current exact-candidate trust authority still accepts it;
- no publisher/developer fallback.

Load the launch profile by exact `before.launch_profile_evidence_digest` and require it for the candidate.

Load conformance by exact ID and digest from the before-plan and require it remains current against:

- candidate;
- exact trust evidence;
- exact launch profile;
- current conformance suite digest.

Load installation approval by exact ID and digest from the before-plan and require the complete approval chain, including reviewed capabilities, remains current.

Reject missing, duplicate, stale, drifted or contradictory evidence.

Do not select a newer or merely compatible replacement record for any plan pin.

### 6. Precompute one exact immutable disabled installed record

Add the minimum crate-private installed-state preparation seam needed to construct one record without filesystem mutation.

The prepared record must contain:

- one newly generated canonical lowercase UUID installed ID;
- `installation_relative_path` exactly `plug-<installed_id>`;
- state exactly `present_disabled`;
- package, candidate, archive, payload, signature and capability evidence copied exactly from the validated candidate;
- exact current trust evidence;
- exact approval ID and digest;
- exact conformance ID and digest;
- exact provider, launch, platform and architecture fields;
- socket and MCP protocol pins matching accepted installed-state behaviour;
- disabled bindings derived deterministically from candidate capabilities, preserving accepted order;
- one `created_unix_ms` generated exactly once for this prepared transaction;
- one canonical record digest calculated after all other fields are frozen.

Call `InstalledPlugRecord::validate` before returning.

The preparation seam must also refuse a duplicate installed package release or contradictory installed registry state.

Prefer one small private pure record-construction helper that can preserve schema consistency. Do not refactor the legacy installation mutation order or error mapping merely for elegance. If sharing with the legacy path would change behaviour, keep the preparation path separate and document why.

### 7. Construct and validate one exact publication intent

Construct the intent only through the accepted `InstallationPublicationIntent::from_precomputed_record` boundary or an exactly equivalent accepted constructor.

Require:

- transaction ID equals installed ID;
- candidate ID equals source candidate ID;
- destination equals the record installation path;
- installed-record digest equals the record digest;
- intent digest covers the frozen intent exactly;
- complete intent validation succeeds.

Then call the accepted complete recovery evidence revalidator against the newly prepared intent.

This second proof must demonstrate that the new precomputed record is fully justified by current candidate, exact trust, launch, conformance and approval evidence.

Do not persist the intent.

### 8. Prove preparation remained read-only

After record and intent preparation, call `plan_installation_recovery(request, context)` again and require idle recovery state.

Successful preparation must leave byte-semantic snapshots unchanged for:

- intent root;
- install root;
- installed-record root;
- quarantine root;
- candidate registry;
- exact trust store;
- launch-profile store;
- conformance store;
- approval store;
- unrelated executor state.

The only newly created values exist in memory inside the sealed prepared value.

### 9. Error preservation

Use existing stable families wherever possible:

```text
installation_execution_plan_stale
installation_execution_invalid_transition
installation_recovery_conflict
installation_recovery_io
installation_intent_invalid
installation_intent_evidence_stale
installation_destination_untracked
unsafe_store_path
```

Do not collapse path-safety failures into generic evidence staleness.
Do not add success-on-absence behaviour.
Do not add a broad generic `publication_failed` code.

### 10. Preserve existing public installation behaviour

The existing public `InstalledPlugRegistry::install_disabled` and crate-private `install_disabled_with_authority` signatures and externally observable behaviour remain accepted.

J24K3e1 must not:

- wire the new prepared value into those methods;
- change their staging, rename, record publication or cleanup order;
- change their timestamps, UUID generation, errors or mutation semantics;
- remove or weaken any existing revalidation.

Any private helper extraction must be proven by existing installed-state regressions and full verification.

## Relevant components

- `tethers-0.1/host-rust/src/installation_publication_preparation.rs`: new read-only preparation boundary;
- `tethers-0.1/host-rust/src/installation_publication_preparation_tests.rs`: direct production-entry tests;
- `tethers-0.1/host-rust/src/installation_plan.rs`: fresh authoritative ordinary plan;
- `tethers-0.1/host-rust/src/installation_recovery_plan.rs`: required idle recovery proof;
- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs`: complete prepared-intent evidence proof;
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`: accepted intent construction and validation;
- `tethers-0.1/host-rust/src/installed.rs`: minimum read-only record-preparation seam;
- `tethers-0.1/host-rust/src/lib.rs`: private module registrations only.

## Frozen decisions and invariants

- J24J is the sole ordinary installation reconciliation authority.
- J24K3d1 is the sole private recovery planning and global audit authority.
- Preparation requires both a fresh exact ordinary plan and idle recovery state.
- Plan pins are identities, not hints for selecting replacement evidence.
- Installed ID and creation time are generated once and frozen before intent construction.
- The complete record and intent are immutable values after preparation.
- The prepared value is sealed and crate-private.
- J24K3e1 performs no durable mutation.
- Later mutation must freshly revalidate before creating durable state.
- Later lock composition must keep preparation and mutation inside one held lock lifetime.

## Acceptance criteria

1. A fresh exact `PublishDisabledInstallation` plan with idle recovery produces one sealed prepared publication.
2. The prepared record validates and exactly pins the current candidate, trust, launch, conformance and approval chain.
3. The prepared intent validates and exactly contains the prepared record.
4. Installed ID, destination, creation time, record digest and intent digest are generated and frozen consistently.
5. Successful preparation changes no durable root.
6. A stale or forged ordinary plan is refused without mutation.
7. Any non-`PublishDisabledInstallation` action is refused without mutation.
8. Pending, malformed, torn or contradictory recovery state blocks preparation without cleanup.
9. Global untracked installed destinations block preparation.
10. Missing, duplicate, stale or drifted candidate/trust/launch/conformance/approval evidence is refused.
11. Candidate or evidence reparse/path-safety failures preserve `unsafe_store_path`.
12. Duplicate installed package release or contradictory installed state is refused.
13. No publication intent file, staging directory, final destination or installed record is created.
14. No lock, public executor or ordinary action wiring is introduced.
15. Existing legacy installed-state behaviour and all named regressions remain green.
16. Cargo.lock remains unchanged.

## Direct test acceptance

Add direct tests whose names begin `j24k3e1`.

At minimum prove:

1. valid publication-ready evidence produces a sealed prepared value;
2. prepared intent and record both validate;
3. installed ID is a canonical lowercase UUID;
4. destination is exactly `plug-<installed_id>`;
5. intent transaction ID, record installed ID and destination identity agree;
6. record fields exactly match candidate, trust, launch, conformance and approval evidence;
7. disabled bindings exactly match candidate capabilities in accepted order;
8. `created_unix_ms` is nonzero and remains unchanged through intent construction and validation;
9. record digest and intent digest remain stable under repeated validation;
10. successful preparation leaves all durable roots unchanged;
11. stale before-plan identity or pins are refused without mutation;
12. wrong before-plan action is refused without mutation;
13. an authoritative plan changed after the supplied before-plan is refused;
14. a valid pending recovery intent blocks preparation and is retained;
15. malformed or torn intent state blocks preparation and is retained;
16. staging, destination, record or global untracked-final recovery conflicts are not cleaned or adopted;
17. missing or changed exact trust is refused;
18. missing or drifted launch, conformance or approval evidence is refused;
19. candidate-byte or quarantine path drift is refused;
20. duplicate installed release is refused;
21. unsafe install, record, intent or candidate path state preserves `unsafe_store_path`;
22. no test observes an intent file, staging directory, final destination or installed record created by preparation;
23. two successful independent preparations may have different generated transaction IDs, while each remains internally exact and both leave durable state unchanged;
24. existing public `install_disabled` behaviour remains covered by regression tests.

Use real stores and filesystem fixtures. Do not add callbacks, clocks supplied by callers, deterministic UUID injection, fault-injection production hooks or arbitrary constructors.

## Regression acceptance

Preserve all accepted suites, including:

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

The bounded Windows lifecycle teardown helper remains unchanged.

## Checkpoint procedure

Avoid transcription errors and self-referential final-tip fields.

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
Do not manually invent, shorten or reconstruct checkpoint SHAs.

## Expected pre-existing changes

- This task packet and its READY worker-note scaffold are present on the implementation branch.
- Accepted J24K3d2 production and independent-review code are already on main.
- No Rust production change is expected before implementation begins.

## Required verification

Run from repository root:

```powershell
$env:PATH = "$HOME\.cargo\bin;$PSHOME;$env:PATH"

pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1

cargo fmt `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all -- --check

cargo nextest run `
  --config-file .config/nextest.toml `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-features --locked `
  -E 'test(j24k3e1)'

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3e1 `
  --locked

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

$env:RUST_TEST_THREADS = "1"
just verify

Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
git diff --check
git status --short
git log --oneline --decorate -16
```

Focused Nextest must finish with zero failures and zero retries.
Full serial `just verify` must finish with zero failures.
Do not exclude a failed test from totals.

If the known Windows lifecycle teardown test fails during a non-serial named regression command:

1. record the exact test name and error;
2. rerun that exact test serially and require it to pass;
3. still require the final full serial `just verify` to pass with zero failures.

Cargo.lock SHA-256 must remain:

```text
D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

## Permitted files

- `tethers-0.1/host-rust/src/installation_publication_preparation.rs`;
- `tethers-0.1/host-rust/src/installation_publication_preparation_tests.rs`;
- `tethers-0.1/host-rust/src/installed.rs` only for the minimum crate-private read-only record-preparation seam or pure shared constructor;
- `tethers-0.1/host-rust/src/lib.rs` only for private module registrations;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-05-j24k3e1-publication-preparation.md`.

If compilation proves one tiny accessor or visibility change is essential in an already accepted crate-private module, stop and document it before broadening scope. Do not silently edit `installation_execution.rs`, recovery classification, intent persistence or public APIs.

## Forbidden changes

- No publication intent persistence.
- No staging directory creation or file copying.
- No staging verification mutation seam.
- No staging-to-destination rename.
- No installed-record publication.
- No intent removal.
- No recovery execution.
- No lock acquisition or lock visibility change.
- No `InstallationExecutionContext` change.
- No replacement of `handle_deferred_publication`.
- No public executor or CLI wiring.
- No ordinary installation action execution.
- No public API, schema or interchange change.
- No dependency, Cargo configuration or Cargo.lock change.
- No OCaml, language, packaging, enablement, operational-scope or release change.
- No caller-supplied clock, UUID, record, intent, path, callback or repair policy.
- No unrelated refactor.

## Stop conditions

Stop as `BLOCKED` if:

- focused Nextest is unavailable;
- the task requires changing accepted J24J or J24K3 recovery semantics;
- exact record preparation cannot be implemented without changing the existing public installation mutation order;
- a required error must be collapsed into a weaker generic class;
- full serial `just verify` remains red;
- Cargo.lock changes.

Do not stop for adding crate-private sealed values, read-only evidence selection, exact record construction, intent construction, direct tests or private module registration within this packet.

## Return format

```text
J24K3e1 complete

Branch:
Final remote tip:
Implementation checkpoint:
Verification checkpoint:

Fresh exact J24J plan:
Idle recovery proof:
Exact candidate and quarantine proof:
Exact current trust proof:
Exact launch proof:
Exact conformance proof:
Exact approval proof:
Installed record precomputation:
Publication intent construction:
Complete prepared-intent revalidation:
Final idle proof:
Durable mutation performed:

Direct J24K3e1 tests:
Focused Nextest:
J24K3d2 regression:
J24K3d1 regression:
J24K3c4 regression:
J24K3c3 regression:
J24K3c2 regression:
J24K3c1 regression:
J24K3b regression:
J24K3a regression:
J24K2 regression:
J24J regression:
M3 lifecycle regression:
Full just verify:

Cargo.lock SHA-256:
Final task packet checker:
cargo fmt --check:
git diff --check:
git status:

Files changed:
Discoveries:
Remaining risks:
```
