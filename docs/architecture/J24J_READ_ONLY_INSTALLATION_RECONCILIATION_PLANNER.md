# J24J Read-Only Installation Reconciliation Planner

Status: FROZEN
Owner: Lucy
Implementation owner: OpenCode
Task colour: Amber
Route: DeepSeek Pro V4
Planning base: `958d56a8582ea4e8c8a9ccc898e54d19f24bad5c`

## 1. Purpose

J24J resumes the Plug-installation sequence after the bounded maintenance programme.

It adds a pure, read-only planner that reconciles one validated J24G installation request against the immutable authorities established by J24H and J24I.

The planner answers one question:

> What is the next legitimate installation action for this exact candidate, given the evidence that already exists?

It must return exactly one next action and the immutable evidence pins that justify it.

J24J does not create trust, prepare or launch a provider, run conformance, create installation approval, copy payloads, publish installed state, acquire a lock, enable a Plug, or add a CLI command.

## 2. Accepted foundations

J24J builds only on accepted authority:

- J24G: typed request schema `tethers.plug-install/1`, exact candidate trust, explicit supervised-execution approval, disabled target state;
- J24H: read-only `open_existing` seams and immutable launch-profile evidence storage;
- J24I: exact-candidate installation trust and `PackageTrustEvidence::exact_candidate`;
- existing M3 conformance, installation-approval, and installed-record validation.

The planner may use:

- `InstallationRequest`;
- `CandidateRegistry::load_all`;
- `ExactCandidateTrustStore::find`;
- `ExactCandidateTrustRecord::require_for_candidate`;
- `PackageTrustEvidence::exact_candidate`;
- `PackageTrustEvidence::require_for_candidate`;
- `LaunchProfileEvidenceStore::load_all`;
- `ConformanceEvidenceStore::load_all`;
- `ConformanceEvidence::require_current`;
- `InstallationApprovalStore::load_all`;
- `InstalledPlugRegistry::load_all`;
- `current_suite_digest`.

It must not call `PackageTrustEvidence::revalidate_current` because J24I deliberately requires the future locked executor to supply current exact installation-trust authority.

## 3. Module boundary

Add:

```text
tethers-0.1/host-rust/src/installation_plan.rs
```

Export it from `lib.rs`.

Add focused integration evidence:

```text
tethers-0.1/host-rust/tests/j24j_installation_reconciliation.rs
```

Do not change existing runtime modules unless compilation proves one narrowly missing read-only accessor. If that occurs, stop and report the exact missing seam before widening scope.

## 4. Public planning seam

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationPlanAction {
    CreateExactCandidateTrust,
    RunSupervisedConformance,
    CreateInstallationApproval,
    PublishDisabledInstallation,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationPlan {
    pub candidate_id: String,
    pub package_id: String,
    pub package_version: String,
    pub semantic_package_digest: String,
    pub action: InstallationPlanAction,
    pub exact_candidate_trust_record_digest: Option<String>,
    pub trust_evidence_digest: Option<String>,
    pub launch_profile_evidence_digest: Option<String>,
    pub conformance_evidence_id: Option<String>,
    pub conformance_evidence_digest: Option<String>,
    pub installation_approval_id: Option<String>,
    pub installation_approval_digest: Option<String>,
    pub installed_id: Option<String>,
    pub installed_record_digest: Option<String>,
}

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

Use existing `M3Error` and `Result`.

The function accepts already-opened read-only authorities. J24J does not define host-data-root layout, create missing roots, or open stores itself.

## 5. Request validation

`InstallationRequest` fields are public and may be manually constructed.

Before reading evidence, require:

- schema exactly `tethers.plug-install/1`;
- canonical lowercase hyphenated candidate UUID;
- `InstallationTrustScope::ExactCandidate`;
- `allow_non_isolated_supervised_execution == true`;
- `InstallationTargetState::Disabled`.

Failure:

- code: `installation_plan_request_invalid`
- message: `installation request is not valid for reconciliation`

The single-variant enums remain compile-time guarantees. Do not add unsafe or impossible enum fixtures.

## 6. Candidate selection

Load the complete validated candidate registry view.

Require exactly one candidate whose `candidate_id` equals the request candidate ID.

No match:

- code: `installation_plan_candidate_missing`
- message: `installation candidate is not present`

More than one match is defensive only because the registry should already reject duplicate identities:

- code: `installation_plan_conflict`
- message: `installation evidence is ambiguous`

Candidate validation failures map to existing safe candidate errors or to:

- code: `candidate_invalid`
- message containing the existing safe candidate error text.

## 7. Reconciliation order

Reconcile from the most advanced durable state backwards, but never accept a later record without validating its complete chain.

Logical progression:

```text
exact candidate
  -> exact-candidate trust
  -> current passed conformance pinned to one launch profile
  -> installation approval pinned to that conformance and trust
  -> installed present-disabled record pinned to that approval
```

Return the earliest missing legitimate action:

1. no exact trust: `CreateExactCandidateTrust`;
2. exact trust but no current passed conformance chain: `RunSupervisedConformance`;
3. current passed conformance but no approval: `CreateInstallationApproval`;
4. current approval but no installed record: `PublishDisabledInstallation`;
5. exact installed present-disabled record: `Complete`.

Historical failed, interrupted, invalidated, or stale conformance evidence is not corruption and may be ignored when deciding that a new supervised conformance run is required.

Malformed, torn, filename-mismatched, duplicate, or otherwise invalid store evidence must fail closed through the existing store validation. It must never be treated as absence.

## 8. Exact trust reconciliation

Use `ExactCandidateTrustStore::find(candidate_id)`.

When absent, return `CreateExactCandidateTrust` with all later evidence fields `None`.

When present:

1. call `record.require_for_candidate(candidate)`;
2. construct `PackageTrustEvidence::exact_candidate(&record)`;
3. call `trust.require_for_candidate(candidate)`;
4. retain both record and package-trust digests in every later plan.

A present but mismatched exact trust record is an error, not a reason to create another record.

## 9. Current conformance selection

Load and validate all launch-profile and conformance evidence.

A reusable conformance chain requires:

- disposition `Passed`;
- one launch-profile record with digest equal to `launch_profile_evidence_digest`;
- launch profile `require_for_candidate(candidate)` succeeds;
- conformance `require_current(candidate, trust, launch, current_suite_digest())` succeeds.

A passed record whose pins are stale is historical, not current. It may be ignored and the planner returns `RunSupervisedConformance` when no reusable chain remains.

If multiple current passed chains remain, select deterministically by:

1. greatest `ended_unix_ms`;
2. then lexicographically greatest `evidence_id`.

This chooses the newest valid completed run without depending on filesystem enumeration order.

Do not select or expose an unpinned launch profile merely because it matches the candidate. A launch profile becomes planning authority only through a reusable current conformance record.

## 10. Installation approval reconciliation

Load all validated approvals for the candidate.

There must be at most one, matching existing approval-store conflict rules.

If more than one exists:

- code: `installation_plan_conflict`
- message: `installation evidence is ambiguous`

A candidate approval is current only when all of these equal the selected chain:

- candidate ID;
- package ID and version;
- semantic package digest;
- raw archive digest;
- trust evidence digest;
- launch-profile evidence digest;
- conformance evidence ID and digest;
- provider ID and version.

Call `approval.validate()` before comparing pins.

A present candidate approval with any stale pin must fail closed:

- code: `installation_plan_stale`
- message: `installation approval does not match current evidence`

Do not ignore a stale approval and attempt to create a second one.

When no approval exists, return `CreateInstallationApproval` with trust, launch-profile, and conformance pins populated.

## 11. Installed-state reconciliation

Load all validated installed records.

A record for the request candidate is complete only when:

- `source_candidate_id` equals the candidate ID;
- state is `present_disabled`;
- package ID and version match;
- semantic package digest and raw archive digest match;
- trust evidence digest matches;
- installation approval ID and digest match;
- conformance evidence ID and digest match;
- provider ID and version match;
- launch-profile label remains `supervised`.

Call `record.validate()` before comparing pins.

A present exact-candidate installed record with any stale pin fails:

- code: `installation_plan_stale`
- message: `installed state does not match current evidence`

More than one installed record for the exact candidate fails with the standard ambiguity error.

When no exact installed record exists but a current approval does, return `PublishDisabledInstallation`.

When one exact current record exists, return `Complete` with every evidence pin populated.

Do not inspect or alter enablement state. J24G requires installation disabled; enablement remains a later separate human decision.

## 12. Read-only guarantee

`plan_installation` must perform no mutation:

- no directory or file creation;
- no deletion or rename;
- no lock acquisition;
- no timestamps generated;
- no trust creation;
- no launch-profile preparation;
- no process launch;
- no conformance execution;
- no approval creation;
- no payload copy;
- no installed publication;
- no enablement change.

Tests must snapshot every supplied store and candidate/quarantine root recursively before and after each planning path.

## 13. Stable planner errors

New planner-owned errors:

| Condition | Code | Message |
|---|---|---|
| invalid manually constructed request | `installation_plan_request_invalid` | `installation request is not valid for reconciliation` |
| requested candidate absent | `installation_plan_candidate_missing` | `installation candidate is not present` |
| duplicate/ambiguous matching evidence | `installation_plan_conflict` | `installation evidence is ambiguous` |
| existing approval pins are stale | `installation_plan_stale` | `installation approval does not match current evidence` |
| existing installed pins are stale | `installation_plan_stale` | `installed state does not match current evidence` |

Existing candidate, store, trust, launch-profile, conformance, approval, and installed validation errors remain unchanged.

Do not expose filesystem paths, request JSON, quarantine paths, or raw platform I/O details in new planner errors.

## 14. Required evidence

Focused tests must prove:

1. valid request plus candidate and no trust returns `CreateExactCandidateTrust`;
2. matching exact trust returns `RunSupervisedConformance`;
3. failed, interrupted, invalidated, or stale conformance does not become reusable authority;
4. current passed conformance plus its exact launch profile returns `CreateInstallationApproval`;
5. multiple current passed conformances select greatest end time, then greatest evidence ID;
6. an unrelated matching launch profile without current conformance is not exposed as authority;
7. current approval returns `PublishDisabledInstallation`;
8. current installed present-disabled record returns `Complete`;
9. every plan contains exactly the evidence pins available at that stage and no invented future pin;
10. missing candidate fails with the frozen error;
11. manually altered schema, candidate ID, and false supervised approval fail before evidence reads;
12. present mismatched exact trust fails rather than planning a replacement;
13. corrupt evidence in any loaded store fails closed rather than being treated as absent;
14. a stale candidate approval fails and is not ignored;
15. a stale exact-candidate installed record fails and is not ignored;
16. duplicate candidate approvals or installed records fail as ambiguous where a direct valid fixture is structurally possible;
17. recursive byte/path snapshots remain unchanged for every successful and failed planning route;
18. no provider process is launched and no new evidence file appears;
19. J24G, J24H, and J24I focused suites remain green;
20. existing M3 lifecycle, approval, installation, and installed-state tests remain green.

Use direct Rust fixtures and existing store publication APIs. Do not add production test-only constructors.

## 15. Permitted files

Only:

- `tethers-0.1/host-rust/src/installation_plan.rs`;
- `tethers-0.1/host-rust/src/lib.rs`;
- `tethers-0.1/host-rust/tests/j24j_installation_reconciliation.rs`;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-04-j24j-installation-reconciliation.md`.

Stop before changing another path.

## 16. Verification

Use:

```powershell
cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check

cargo nextest run `
  --config-file .config/nextest.toml `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-targets --all-features --locked `
  -E 'test(j24j_installation_reconciliation) | test(installation_plan)'

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test j24j_installation_reconciliation `
  --locked

just verify

Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
git diff --check
git status --short
```

The focused Nextest filter may be adjusted once if Nextest reports the exact integration test name differently. Do not repeat a failing filter blindly.

Do not run OpenCode LSP as a gate. If tried once and useful, record its evidence. If empty, null, unavailable, or slow, record that fact and continue with `rg`, compiler diagnostics, and tests.

Do not run full Nextest, cargo-deny, cargo-machete, `just verify-agent`, OCaml tests, packaging, or release work. Dependencies, Cargo.lock, tool configuration, OCaml, and public CLI are forbidden from changing.

Cargo.lock must remain:

`D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`

## 17. Stop conditions

Stop as BLOCKED only when:

- an accepted store lacks a required read-only `load_all` or validation seam;
- existing evidence cannot be reconciled without weakening an accepted validation contract;
- a required exact pin is not represented in existing records;
- implementation would require process launch, mutation, lock acquisition, CLI work, dependency changes, or out-of-scope module changes;
- focused or final required verification still fails after one evidence-led correction.

Do not stop for ineffective LSP, an unavailable optional tool, or a first failed exact text replacement. Reread, make one smaller fresh patch, and continue.

## 18. Non-goals

J24J does not:

- create any authority or evidence;
- define or acquire the installation lock;
- define host-data-root layout;
- prepare, launch, or communicate with a provider;
- run conformance;
- create installation approval;
- copy or install payloads;
- publish installed state;
- enable or disable a Plug;
- add `plug install` or another CLI command;
- alter J24G request JSON;
- change candidate, trust, launch-profile, conformance, approval, installed, enablement, package, or protocol schemas;
- change Tethers language semantics or OCaml;
- add dependencies or change Cargo.lock.

J24K will own the installation lock and gated executor that consumes this plan.
