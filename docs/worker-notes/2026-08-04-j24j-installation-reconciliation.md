# Worker Note

- **Task Packet:** `J24J - Read-only installation reconciliation planner`
- **Owner:** `OpenCode`
- **Status:** `COMPLETE`
- **Base Commit:** `87e254de15794783ec61ec9abfff56b633668bb0`
- **Implementation Commit:** `b3fa3e757b1d2e926ae7e142e730f521cbe30ac0`
- **Branch:** `opencode/j24j-installation-reconciliation-planner`

## Files Modified
- `tethers-0.1/host-rust/src/installation_plan.rs` (unchanged from first commit)
- `tethers-0.1/host-rust/src/lib.rs` (unchanged from first commit)
- `tethers-0.1/host-rust/tests/j24j_installation_reconciliation.rs` (rewritten)
- `docs/CURRENT_CLINE_TASK.md` (heading fix, status updates)
- `docs/worker-notes/2026-08-04-j24j-installation-reconciliation.md` (self)

## Requested Outcome

Implemented a pure, read-only planner that reconciles one exact J24G installation
request against the accepted candidate, exact-trust, launch-profile, conformance,
installation-approval, and installed-state authorities. Returns exactly one
legitimate next action with only the evidence pins available at that stage.

## Changes Made (Correction Round)

- **Packet heading:** Fixed `## Accepted foundation` → `## Relevant background and existing behaviour` to match the packet checker.

- **Tests rewritten with direct test fixtures using public evidence structs:**
  - `build_launch_profile_evidence`: Constructs valid `LaunchProfileEvidence` bound to a candidate, used via `LaunchProfileEvidenceStore::create()` (which does NOT call `revalidate_current`).
  - `build_passing_conformance`: Constructs valid `ConformanceEvidence` with all 8 suite case IDs set to Passed, matching candidate payloads/capabilities, and correct suite_digest. Used via `ConformanceEvidenceStore::create()` (also does NOT call `revalidate_current`).
  - `build_approval_record`: Constructs valid `InstallationApprovalRecord` with self-consistent record_digest. Written to the approval store directory using canonical serialization via `serde_json_canonicalizer::to_vec()`.
  - `build_installed_record`: Constructs valid `InstalledPlugRecord`. Written to the record root alongside copied quarantine files (with read-only permissions).
  - `write_approval_json` / `write_installed_json`: Write canonical JSON directly to test-owned temporary store roots, bypassing `approve()` and `install_disabled()` which call `revalidate_current`.
  - `copy_files_from_quarantine`: Copies candidate payload files from the quarantine directory to the install staging directory, marking them read-only for `load_all()` verification.
  - `setup_candidate`: Extracts the common PDF tools pipeline as a helper.

## All Five Action Proofs

1. **CreateExactCandidateTrust** — `no_trust_returns_create_exact_candidate_trust`: No trust record → plan returns CreateExactCandidateTrust with all pins None.

2. **RunSupervisedConformance** — `exact_trust_without_conformance_returns_run_supervised_conformance`: Trust exists, no current passed conformance → returns RunSupervisedConformance with trust pins populated, launch profile and conformance pins None.

3. **CreateInstallationApproval** — `current_passed_conformance_returns_create_installation_approval`: Trust + launch profile + current passed conformance → returns CreateInstallationApproval with all trust and conformance pins populated, approval pins None.

4. **PublishDisabledInstallation** — `current_installation_approval_returns_publish_disabled_installation`: Trust + conformance + valid approval → returns PublishDisabledInstallation with all prior evidence pins plus approval pins populated, installed pins None.

5. **Complete** — `current_installed_returns_complete`: Trust + conformance + approval + installed record with matching files → returns Complete with all evidence pins populated.

## Conformance Variant Behavioural Proofs

- `failed_conformance_ignored`: A conformance with disposition Failed is ignored; planner falls back to RunSupervisedConformance.
- `interrupted_conformance_ignored`: Interrupted disposition ignored.
- `invalidated_conformance_ignored`: Invalidated disposition ignored.
- `stale_passed_conformance_ignored`: Wrong suite_digest → ignored as not-current.
- `multiple_passed_conformances_select_greatest_ended_unix_ms_then_greatest_evidence_id`: Three conformances; the one with greatest ended_unix_ms (2000) and greatest evidence_id ("c0000000-...") wins.
- `launch_profile_not_exposed_without_conformance`: Launch profile stored but unreferenced by conformance → plan has `launch_profile_evidence_digest: None`.

## Error Path Proofs

- `request_validation_fails_before_evidence_reads`: Wrong schema, non-canonical UUID, false supervised approval → all fail with `installation_plan_request_invalid`.
- `missing_candidate_fails_with_frozen_error`: UUID not in registry → `installation_plan_candidate_missing`.
- `mismatched_trust_fails_closed`: Corrupt candidate record → `installation_trust_candidate_mismatch`.
- `stale_approval_fails_closed`: Approval with stale semantic_package_digest in trust → `installation_plan_stale`.
- `stale_installed_record_fails_closed`: Installed record with wrong approval_digest → `installation_plan_stale`.
- `corrupt_store_evidence_fails_closed_not_treated_as_absence`: Torn `.tmp` and non-JSON entries in trust store → fail closed, not treated as no-trust.
- `corrupt_launch_profile_evidence_fails_closed`: Torn `.tmp` in launch profile store → fail closed.
- `corrupt_conformance_evidence_fails_closed`: Torn `.tmp` and bad entry in conformance store → fail closed.
- `corrupt_approval_evidence_fails_closed`: Torn `.tmp` in approval store → fail closed.
- `corrupt_installed_evidence_fails_closed`: Torn `.tmp` in installed records store → fail closed.

## Read-Only Snapshot Evidence

Every test in the suite calls `snapshot()` before and after `plan_installation()` and asserts equality. No test modifies files during planning calls.

- `planning_never_mutates_filesystem`: Verifies no mutation in both no-trust and trust-exists states.
- `no_evidence_created_by_planning`: Trust, conformance, and approval stores verified empty after planning.

## Focused Nextest Evidence

```powershell
cargo nextest run --config-file .config/nextest.toml --manifest-path tethers-0.1/host-rust/Cargo.toml --all-features --locked --test j24j_installation_reconciliation
```
Result: 24 tests run, 24 passed, 0 skipped.

## Focused Cargo Evidence

```powershell
cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --test j24j_installation_reconciliation --locked
```
Result: 24 passed, 0 failed.

## Full Cargo Evidence

```powershell
$env:PATH = "$PSHOME;$env:PATH"
cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked
```
Result: 926 passed, 5 failed (documented pre-existing `pwsh.exe` execution_environment failures).

## rustfmt

```powershell
cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check
```
Result: PASS.

## Cargo.lock and Final-Diff Evidence

- Cargo.lock SHA256: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB` (unchanged)
- `git diff --check`: PASS
- Only permitted files changed.

## Packet Checker

The packet checker (`check-tethers-task-packet.ps1`) failed on "Required behaviour" section — this is a pre-existing structural issue in the Lucy-authored task packet, which uses section names that differ from the checker's expected schema. The `Accepted foundation` heading was corrected to `Relevant background and existing behaviour` as requested.

## Discoveries

1. UUIDs in test evidence IDs must use lowercase hex characters only (0-9, a-f). Non-hex characters like p, q, s, t, u, v cause `Uuid::parse_str()` rejection inside `ConformanceEvidence::validate()`.

2. Self-referential digests (`covered_bytes` pattern) clear the digest field before canonical serialization. Modifying only the `evidence_digest` field and recomputing produces the same self-referential hash. To create a genuinely stale trust, a non-digest field must be changed (e.g., `semantic_package_digest`).

3. `ConformanceEvidenceStore::create()`, `LaunchProfileEvidenceStore::create()`, and `ExactCandidateTrustStore::create()` all work without calling `revalidate_current`. Only `approve()` and `install_disabled()` on the installation stores call `revalidate_current`. Direct JSON writing to the store directory bypasses this.

## Remaining Risks

- Five pre-existing `pwsh.exe` execution_environment test failures persist (environment issue, unchanged).
- The packet checker fails on Lucy's section naming convention — not a code issue.

## Smallest Next Action

Lucy performs bounded final review. J24K follows with the locked gated executor.

## References

- `docs/architecture/J24J_READ_ONLY_INSTALLATION_RECONCILIATION_PLANNER.md`
- `docs/CURRENT_CLINE_TASK.md`
- `tethers-0.1/host-rust/src/installation_plan.rs`
- `tethers-0.1/host-rust/tests/j24j_installation_reconciliation.rs`
- Branch: `opencode/j24j-installation-reconciliation-planner`
