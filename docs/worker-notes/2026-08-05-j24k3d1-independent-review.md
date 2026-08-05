# Independent Review Note

Task: `J24K3d1 - Validated read-only installation recovery plan and correction`
Reviewer: `Lucy`
Status: `ACCEPTED`
Accepted main before merge: `20cd25f328568aa2726505580689d67b6219449c`
Reviewed branch tip before this note: `e58effa857f5958b8db2035687579c370e69beb6`
Original planner implementation checkpoint: `351a2782b59d1b08c5529bd18caf8a7fa29cde6b`
Planner path-safety correction checkpoint: `9a48be42cdedad116c6a0f7df00e927d5abccd86`
Bounded teardown correction checkpoint: `208ef6f0f05dd8cf21b41afc39227423cc9b7e04`
Verified completion candidate: `aa8720b5758296ea550cb119354b6073908664a6`

## Review outcome

Accepted for routine fast-forward merge.

The crate-private recovery planner:

- loads the authoritative current publication intent itself;
- always audits the global installed-root namespace, including no-intent state;
- returns idle only after a successful no-intent audit;
- observes transaction state through the accepted registry seam;
- classifies only through the accepted pure recovery classifier;
- avoids package-evidence requirements for cleanup-only dispositions;
- revalidates current evidence and then exact destination bytes for both publication-bearing dispositions;
- returns a sealed read-only intent/disposition plan;
- performs no mutation, lock acquisition, reconciliation planning, or executor wiring.

The independent-review correction adds direct planner-entry regressions for destination junction or symlink state, a removed already-opened install root, and a record-root junction or symlink. The production planner remained unchanged.

The authorised `m3_lifecycle` correction is test-only. Its bounded helper retries only `PermissionDenied` and `DirectoryNotEmpty`, treats `NotFound` as success, waits 25 milliseconds between attempts, stops after two seconds, and still fails with the full path and final error when contention persists. It replaces only the final fixture-tree teardown in `m3_malformed_and_interrupted_conformance_fail_without_retry_or_install`; assertions, process shutdown, conformance behaviour, production code, dependencies, and Cargo.lock are unchanged.

## Verification evidence

Luna reported:

- ten exact serial repetitions of the formerly failing lifecycle test: 10/10 passed;
- focused J24K3d1 Nextest: 28 passed, 0 failed, 0 retries, with 2 platform-gated Unix tests skipped on Windows;
- all named J24K3d1, J24K3a through J24K3c4, J24K3b, J24K2, J24J, and M3 lifecycle regressions passed;
- full serial `just verify`: passed with zero failures;
- task packet checker: PASS;
- `cargo fmt --check`: passed;
- `git diff --check`: passed;
- Cargo.lock SHA-256 unchanged: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`;
- final working tree: clean.

The reviewer inspected the actual planner implementation, direct regression tests, teardown helper, changed-file boundary, and commit ancestry. The Rust suite was not independently rerun by the reviewer.

## SHA clarification

The final packet and correction worker note record nonexistent teardown checkpoint `208ef6f0cbf29c5933cc72a8c93ca87973a3f733`. GitHub proves the actual commit is `208ef6f0f05dd8cf21b41afc39227423cc9b7e04`.

The handoff also described the earlier planner path-safety implementation as `9a48be4e08d06e636cb53e21c9686ef65fbca8c8`; GitHub proves the actual commit is `9a48be42cdedad116c6a0f7df00e927d5abccd86`.

The verification checkpoint `aa8720b5758296ea550cb119354b6073908664a6` and reviewed remote tip `e58effa857f5958b8db2035687579c370e69beb6` are real. GitHub shows the latter changed only task and worker-note evidence after the tested completion candidate.

This independent note is the external fixed point for the audit trail and deliberately does not attempt to name its own resulting commit.

## Remaining boundary

A later package must recheck the authoritative intent immediately before mutation. Staging cleanup, exact installed-record publication, intent removal, lock integration, reconciliation execution, and public CLI wiring remain outside J24K3d1.