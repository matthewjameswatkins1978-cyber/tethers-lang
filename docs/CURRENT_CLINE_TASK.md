# Current Implementation Task

Control contract: `1`
Task: `J24J - Read-only installation reconciliation planner`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode using DeepSeek Pro V4 for bounded semantic Rust planning logic; Lucy performs independent review and routine safe merge`
Base branch: `main`
Base commit: `87e254de15794783ec61ec9abfff56b633668bb0`
Implementation branch: `opencode/j24j-installation-reconciliation-planner`
Worker note: `docs/worker-notes/2026-08-04-j24j-installation-reconciliation.md`
Implementation blueprint: `docs/architecture/J24J_READ_ONLY_INSTALLATION_RECONCILIATION_PLANNER.md`
Rust toolchain: `1.97.1`
Implementation checkpoint: `b3fa3e757b1d2e926ae7e142e730f521cbe30ac0`

## Objective

Implement a pure, read-only planner that reconciles one exact J24G installation request against the accepted candidate, exact-trust, launch-profile, conformance, installation-approval, and installed-state authorities.

The planner returns exactly one legitimate next action:

- create exact-candidate trust;
- run supervised conformance;
- create installation approval;
- publish disabled installation;
- complete.

## Relevant background and existing behaviour

J24G, J24H, and J24I are accepted on `main`.

The installation sequence is:

```text
J24G request contract
  -> J24H read-only evidence access
  -> J24I exact-candidate trust
  -> J24J read-only reconciliation planner
  -> J24K locked gated executor
  -> J24L thin public plug install CLI
```

Accepted baseline:

```text
Rust             1.97.1
Cargo tests      926 passing minimum
Nextest retries  0
Cargo.lock       D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

## Required behaviour

1. Revalidate all public `InstallationRequest` fields before reading evidence.
2. Load the validated candidate registry and select only the exact requested candidate.
3. Find and validate exact-candidate installation trust, treating present mismatch as an error rather than absence.
4. Construct deterministic `PackageTrustEvidence::exact_candidate` without calling `PackageTrustEvidence::revalidate_current`.
5. Select reusable current passed conformance only when candidate, trust, launch profile, and current suite pins all match.
6. Select multiple reusable conformances by greatest `ended_unix_ms`, then lexicographically greatest `evidence_id`.
7. Validate any existing installation approval against the complete selected evidence chain and fail closed on stale pins.
8. Validate any existing present-disabled installed record against the complete selected evidence chain and fail closed on stale pins.
9. Return the earliest missing legitimate action with only genuine evidence pins populated and perform no mutation.

Historical failed, interrupted, invalidated, or stale conformance is not reusable authority and may be ignored when planning a fresh supervised run. Malformed or corrupt store evidence must fail closed.

## Relevant components

- `tethers-0.1/host-rust/src/installation_plan.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/tests/j24j_installation_reconciliation.rs`
- `InstallationRequest`
- `CandidateRegistry`
- `ExactCandidateTrustStore`
- `PackageTrustEvidence`
- `LaunchProfileEvidenceStore`
- `ConformanceEvidenceStore`
- `InstallationApprovalStore`
- `InstalledPlugRegistry`
- `current_suite_digest`
- existing `M3Error` and `Result`

## Frozen decisions and invariants

- The public action enum has exactly five variants: `CreateExactCandidateTrust`, `RunSupervisedConformance`, `CreateInstallationApproval`, `PublishDisabledInstallation`, and `Complete`.
- The public plan record and `plan_installation` signature are frozen by the J24J architecture blueprint.
- The planner accepts already-opened authorities and does not define host-data-root layout.
- Launch-profile authority exists only through a reusable current conformance record that pins it.
- A present stale approval or installed record is an error, not permission to create a replacement.
- Future-stage evidence fields remain `None`; the planner never invents IDs or digests.
- The planner is read-only and does not acquire a lock, generate time or identity, launch a process, create authority, publish state, or inspect enablement.
- Accepted evidence schemas, dependencies, Cargo configuration, OCaml, and Cargo.lock remain unchanged.

## Acceptance criteria

1. The J24J module and export match the frozen public seam.
2. All five plan actions are reached through valid evidence states.
3. Every later action carries the complete valid earlier evidence chain.
4. Corrupt evidence is never treated as absence.
5. Historical non-current conformance does not block a fresh conformance action.
6. Deterministic conformance selection is independent of enumeration order.
7. Approval and installed state fail closed when their pins drift.
8. Every successful and failed planning route preserves recursive byte/path snapshots.
9. Focused Nextest executes the J24J integration suite with zero retries.
10. Focused ordinary Cargo integration tests pass.
11. Cargo.lock remains byte-identical and the final diff contains only permitted files.
12. Final Cargo verification retains the accepted 926-test baseline, with only the five documented pre-existing `pwsh.exe` environment failures.

## Required verification

Required commands:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1

cargo fmt `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all -- --check

cargo nextest run `
  --config-file .config/nextest.toml `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-features --locked `
  --test j24j_installation_reconciliation

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test j24j_installation_reconciliation `
  --locked

$env:PATH = "$PSHOME;$env:PATH"
just verify

Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
git diff --check
git status --short
```

Recorded implementation evidence:

- focused Nextest: 24 passed, 0 failed, 0 skipped;
- focused Cargo integration: 24 passed, 0 failed;
- Cargo baseline: 926 passed with five unchanged pre-existing `pwsh.exe` environment failures;
- rustfmt: pass;
- Cargo.lock: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`.

The packet and worker-note headings were normalized during Lucy's acceptance review after OpenCode reported that the earlier Lucy-authored packet did not match the control-v1 checker schema. This connector-side documentation correction did not rerun local commands.

## Forbidden changes

- No trust creation, conformance execution, approval creation, installation publication, enablement work, lock acquisition, timestamp or UUID generation inside the planner.
- No provider preparation, process launch, protocol communication, payload copying, or CLI command.
- No call to `PackageTrustEvidence::revalidate_current` from J24J.
- No accepted evidence-schema, dependency, Cargo.lock, tool-configuration, OCaml, packaging, or release changes.
- No production test-only constructors.
- No files outside the five permitted paths.

## Stop conditions

Stop as `BLOCKED` only if:

- an accepted authority lacks a required read-only load or validation seam;
- a required exact pin is absent from accepted records;
- safe reconciliation would require weakening validation;
- implementation requires mutation, process launch, lock, CLI, dependency, schema, or out-of-scope changes;
- required verification still fails after one evidence-led correction.

Do not stop for failed LSP, an unavailable optional tool, or one failed exact replacement.

## Expected pre-existing changes

None.
