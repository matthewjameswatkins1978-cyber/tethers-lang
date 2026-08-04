# J24J Worker Note

Task: `J24J - Read-only installation reconciliation planner`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `87e254de15794783ec61ec9abfff56b633668bb0`
Implementation checkpoint: `b3fa3e757b1d2e926ae7e142e730f521cbe30ac0`
Branch: `opencode/j24j-installation-reconciliation-planner`

## Requested outcome

Implement a pure, read-only planner that reconciles one exact J24G installation request against candidate, exact-trust, launch-profile, conformance, installation-approval, and installed-state authorities. Return exactly one legitimate next action with only the evidence pins available at that stage.

## Changes made

Production implementation:

- added `tethers-0.1/host-rust/src/installation_plan.rs` with the frozen five-action enum, plan record, and `plan_installation` seam;
- exported the module from `tethers-0.1/host-rust/src/lib.rs`;
- retained existing `M3Error` and `Result` and did not call `PackageTrustEvidence::revalidate_current`.

Correction round:

- rewrote `tethers-0.1/host-rust/tests/j24j_installation_reconciliation.rs` with direct valid evidence fixtures;
- used public launch-profile and conformance store creation seams;
- wrote canonical approval and installed fixtures directly into test-owned temporary stores because the production execution seams deliberately require J24K's future current-trust authority;
- proved every planner stage behaviorally rather than merely proving enum variants exist;
- normalized `docs/CURRENT_CLINE_TASK.md` and this worker note to the control-v1 checker schema during Lucy's acceptance review.

Changed paths are limited to:

- `tethers-0.1/host-rust/src/installation_plan.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/tests/j24j_installation_reconciliation.rs`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-04-j24j-installation-reconciliation.md`

## Decisions and assumptions

- Single-variant request enums remain compile-time guarantees; no impossible negative enum fixture was introduced.
- Launch-profile evidence becomes planning authority only when pinned by reusable current passed conformance.
- Failed, interrupted, invalidated, or stale conformance is historical rather than current authority and is ignored when selecting a reusable run.
- A present mismatched exact-trust record, stale approval, stale installed record, or corrupt store entry fails closed.
- Multiple reusable conformances are ordered by greatest `ended_unix_ms`, then lexicographically greatest `evidence_id`.
- Direct test fixture publication does not weaken production validation: every constructed record computes its canonical digest and calls its public `validate()` method before use.
- The planner itself performs no mutation, process launch, lock acquisition, authority creation, or enablement inspection.

## Evidence

All five actions are exercised through real planner calls:

1. `no_trust_returns_create_exact_candidate_trust`
2. `exact_trust_without_conformance_returns_run_supervised_conformance`
3. `current_passed_conformance_returns_create_installation_approval`
4. `current_installation_approval_returns_publish_disabled_installation`
5. `current_installed_returns_complete`

Additional behavioral evidence covers:

- greatest end time and evidence-ID tie-breaking;
- launch profiles not exposed without reusable conformance;
- failed, interrupted, invalidated, and stale passed conformance ignored;
- invalid request rejection before evidence reads;
- missing candidate and mismatched trust failure;
- stale approval and stale installed state failure;
- corrupt trust, launch-profile, conformance, approval, and installed stores failing closed;
- exact stage pins with future-stage pins left empty;
- recursive path-and-byte snapshots before and after successful and failed planning calls;
- no evidence or provider process created by planning.

Verification recorded by OpenCode:

```text
Focused Nextest integration   24 passed, 0 failed, 0 skipped
Focused Cargo integration     24 passed, 0 failed
Cargo baseline                926 passed
Pre-existing environment      5 pwsh.exe execution_environment failures
rustfmt                       PASS
Cargo.lock SHA256             D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

The five `pwsh.exe` failures are the same documented environmental failures present before J24J. The packet and worker-note schema corrections were made through the GitHub connector during Lucy's acceptance review, so local commands were not rerun after those documentation-only commits.

## Discoveries

- Valid evidence IDs must be canonical lowercase UUIDs; non-hex characters are rejected by existing validation.
- Self-covered record digests are computed with the digest field cleared, so a genuinely stale fixture must change a covered non-digest field and recompute the record digest.
- `LaunchProfileEvidenceStore::create`, `ConformanceEvidenceStore::create`, and `ExactCandidateTrustStore::create` do not require current-trust revalidation.
- Production approval and installation publication correctly require current trust authority; J24K will own that locked execution boundary.
- The original Lucy-authored task packet used descriptive headings that did not match the newer control-v1 checker names. The content was retained while the headings and worker-note fields were normalized.

## Remaining risks

- Five unrelated Windows `pwsh.exe` environment tests remain red in the recorded full Cargo run; they are unchanged from the accepted baseline.
- J24J proves planning over valid durable evidence. J24K must still prove the locked executor supplies current exact installation-trust authority while creating those durable states.
- The connector-side documentation normalization was reviewed against the checker source but was not followed by a local checker execution.

## Smallest next action

Lucy accepts and fast-forwards J24J to `main`. Then freeze J24K around the installation lock and gated executor that consumes the J24J plan.

## References

- `docs/architecture/J24J_READ_ONLY_INSTALLATION_RECONCILIATION_PLANNER.md`
- `docs/architecture/J24I_EXACT_CANDIDATE_INSTALLATION_TRUST.md`
- `docs/CURRENT_CLINE_TASK.md`
- `tethers-0.1/host-rust/src/installation_plan.rs`
- `tethers-0.1/host-rust/tests/j24j_installation_reconciliation.rs`
- branch `opencode/j24j-installation-reconciliation-planner`
