# Worker Note

- **Task Packet:** `J24J - Read-only installation reconciliation planner`
- **Owner:** `OpenCode`
- **Status:** `COMPLETE`
- **Base Commit:** `87e254de15794783ec61ec9abfff56b633668bb0`
- **Implementation Commit:** `e1a35e7a56e0ee916ac06fc948d64b27ac30750a`
- **Branch / Worktree:** `opencode/j24j-installation-reconciliation-planner`

## Files Modified
- `tethers-0.1/host-rust/src/installation_plan.rs` (new)
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/tests/j24j_installation_reconciliation.rs` (new)
- `docs/CURRENT_CLINE_TASK.md`

## Requested Outcome

Implemented a pure, read-only planner that reconciles one exact J24G installation
request against the accepted candidate, exact-trust, launch-profile, conformance,
installation-approval, and installed-state authorities. Returns exactly one
legitimate next action with only the evidence pins available at that stage.

## Changes Made

- `src/installation_plan.rs`: Added `InstallationPlanAction` enum (5 variants),
  `InstallationPlan` struct with frozen fields, and `plan_installation` public
  function. Internal functions: `validate_request`, `select_candidate`,
  `find_exact_trust`, `select_current_conformance`, `check_approval`,
  `check_installed`. All errors use existing `M3Error` and `Result` types.

- `src/lib.rs`: Added `pub mod installation_plan`.

- `tests/j24j_installation_reconciliation.rs`: 11 focused integration tests.
  Tests are `#[cfg(windows)]` and use the PDF tools pipeline for candidate
  creation.

## Decisions and Assumptions

- Single-variant enums (`InstallationTrustScope::ExactCandidate`,
  `InstallationTargetState::Disabled`) are compile-time guarantees. No negative
  enum fixtures added.

- The `candidate_invalid` error propagation from `CandidateRecord::validate()`
  through `CandidateRegistry::load_all()` maps through existing `PackageError`
  wrapping with code `candidate_invalid` in `select_candidate`.

- **Conformance/approval/installed stage tests are not executable with
  ExactCandidate trust** because `InstallationApprovalStore::approve()` and
  `InstalledPlugRegistry::install_disabled()` internally call
  `PackageTrustEvidence::revalidate_current()` which deliberately fails closed
  for ExactCandidate mode with `trust_exact_candidate_authority_required`. The
  J24K locked executor is required to supply current trust authority. Tests
  cover stages (no trust, trust without conformance) fully and verify the
  remaining errors through structural coverage of the code paths.

- Launch profile authority is only selected when pinned by reusable current
  conformance. No unpinned launch profile is exposed.

## Evidence-Chain Algorithm

```text
1. validate_request  → schema, candidate UUID, trust scope, supervised approval, target state
2. select_candidate  → exact candidate_id match in registry
3. find_exact_trust  → ExactCandidateTrustStore::find → require_for_candidate → PackageTrustEvidence::exact_candidate
4. select_current_conformance → load_all conformance, filter Passed + matching candidate_id, find launch profile by digest, require_for_candidate, require_current, sort by ended_unix_ms desc → evidence_id desc
5. check_approval    → load_all approvals, filter by candidate_id, validate() + full pin comparison
6. check_installed   → load_all installed, filter by source_candidate_id, validate() + full pin comparison
```

Earliest missing legitimate action returned with only evidence pins proven at
that stage. Future pins are None.

## Read-Only Snapshot Evidence

Every test that calls `plan_installation` takes a filesystem snapshot before and
after and asserts equality. No test modifies files during planning calls.

- `no_trust_returns_create_exact_candidate_trust` — snapshot unchanged
- `exact_trust_without_conformance_returns_run_supervised_conformance` — snapshot unchanged
- `request_validation_fails_before_evidence_reads` — sentinel directory unchanged
- `missing_candidate_fails_with_frozen_error` — store roots unchanged
- `mismatched_trust_fails_closed` — store roots unchanged
- `corrupt_store_evidence_fails_closed_not_treated_as_absence` — snapshot unchanged after each corruption
- `planning_never_mutates_filesystem` — snapshots verified for no-trust and trust paths
- `no_evidence_created_by_planning` — trust/conformance/approval stores verified empty after planning

## Focused Nextest and Cargo Evidence

Focused Nextest (unit tests):
```powershell
cargo nextest run --config-file .config/nextest.toml --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked -E 'test(j24j_installation_reconciliation) | test(installation_plan)'
```
Result: 5 passed (unit tests), 1144 skipped

Focused Cargo integration tests:
```powershell
cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --test j24j_installation_reconciliation --locked
```
Result: 11 passed, 0 failed

## Full Cargo Evidence

```powershell
cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked
```
Result: 926 passed, 5 failed (documented pre-existing `pwsh.exe` not found environment failures)

## Tool Usefulness and Fallbacks

- rg: used extensively for seam discovery and code navigation. Reliable.
- Cargo compiler diagnostics: primary feedback loop. Reliable.
- rustfmt: auto-applied formatting, --check passes.
- LSP: not used. Not needed for this task.
- Nextest integration test filter: integration tests not matched by filter;
  packet explicitly permits one adjustment. Used `cargo test --test`
  directly for integration tests. Recorded as the single adjustment.

## Cargo.lock and Final-Diff Evidence

- Cargo.lock SHA256: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB` (unchanged)
- `git diff --check`: PASS
- `git diff --stat`: 4 files, +1170 -2
- Only permitted files changed.

## Discoveries

1. The packet checker (`check-tethers-task-packet.ps1`) requires section
   "Relevant background and existing behaviour" but Lucy's packet uses
   "Accepted foundation". This is a packet structure deviation, not a blocker.
   The packet was compiled by Lucy and is READY.

2. `InstallationApprovalStore::approve()` and `InstalledPlugRegistry::install_disabled()`
   both call `PackageTrustEvidence::revalidate_current()` internally, which
   deliberately fails for `ExactCandidate` trust mode. This blocks end-to-end
   testing of approval/installed stages with ExactCandidate trust. The planner
   itself correctly does not call `revalidate_current`. J24K is required to
   supply current trust authority before these stages can be exercised in
   integration tests.

3. `run_host_conformance()` also calls `revalidate_current()` via
   `PreparedSupervisedLaunch::revalidate_current_trust()`, blocking conformance
   runs with ExactCandidate trust.

## Remaining Risks

- Stages requiring current conformance, approval, or installed state are
  structurally covered in the planner code but cannot be integration-tested
  until J24K provides the required current trust authority.

- The 5 `pwsh.exe` failures in the full Cargo suite remain a documented
  environment limitation.

## Smallest Next Action

Lucy performs bounded final review of the pushed J24J branch. J24K follows with
the locked gated executor that consumes this plan and supplies the missing
current installation-trust authority.

## References

- `docs/architecture/J24J_READ_ONLY_INSTALLATION_RECONCILIATION_PLANNER.md`
- `docs/architecture/J24I_EXACT_CANDIDATE_INSTALLATION_TRUST.md`
- `docs/CURRENT_CLINE_TASK.md`
- `tethers-0.1/host-rust/src/installation_plan.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/tests/j24j_installation_reconciliation.rs`
- Branch: `opencode/j24j-installation-reconciliation-planner`
