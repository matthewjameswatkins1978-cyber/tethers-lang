# Tethers 0.8.2 — Agent D RED-phase reproduction evidence

Status: RED-phase record. This branch intentionally contains regression
suites that FAIL against the v0.8.1 baseline. It must not be merged into the
integration branch before the corresponding Lane A/B/C repairs are integrated;
after integration the same suites are the GREEN acceptance evidence.

- Baseline (v0.8.1 tag == origin/main at packet time):
  `2710e867768f33be3e6bd1e722004c1c50e3f53b`
- Suites: `tests/audit_082_replay.rs`, `tests/audit_082_git_status.rs`,
  `tests/audit_082_process.rs` (copies of the Lane A/B regression suites,
  unmodified).
- Local platform: native Windows x86-64, Rust 1.97.1, `--locked`,
  `--test-threads=1`, debug profile.

## Windows x86-64 local RED results (against baseline)

Command:

```
cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --locked \
  --no-fail-fast --test audit_082_replay --test audit_082_git_status \
  --test audit_082_process -- --test-threads=1
```

### audit_082_git_status — 5 FAILED / 2 passed (DEF-05 reproduced)

| Test | Baseline result | Defect proved |
| --- | --- | --- |
| `audit_082_clean_repository_reports_clean` | FAILED | pre-0.8.2 output lacks the `renamed_paths`/`conflicted_paths` contract |
| `audit_082_modified_staged_untracked_classification_matches_native_git` | ok | basic XY classification already worked |
| `audit_082_staged_rename_reports_source_and_destination` | FAILED | rename source token misparsed as an independent entry |
| `audit_082_rename_then_worktree_modification_is_both_staged_and_unstaged` | FAILED | same rename defect (`RM` records) |
| `audit_082_merge_conflict_is_reported_as_conflict` | FAILED | `conflict` hard-coded `false`; `UU` leaked into staged+unstaged |
| `audit_082_unicode_paths_round_trip_without_corruption` | FAILED | byte-slicing/lossy handling corrupts multibyte identities |
| `audit_082_detached_head_reports_native_header` | ok | header pass-through already worked |

### audit_082_process — 1 FAILED (RSK-04 truthfulness reproduced)

| Test | Baseline result | Defect proved |
| --- | --- | --- |
| `audit_082_exec_timeout_reports_truthful_supervision_on_windows` | FAILED | baseline self-reports no `resource_limits` field; the POSIX variant (CI lanes) additionally proves descendant survival after timeout |

### audit_082_replay — 2 FAILED / 2 passed

| Test | Baseline result | Meaning |
| --- | --- | --- |
| `audit_082_abandoned_admission_is_resolved_via_cli_and_key_released` | FAILED | DEF-03: no operator route existed; `replay-resolve-claim` is an unknown command on the baseline |
| `audit_082_resolve_refuses_states_beyond_claimed_no_state` | FAILED | DEF-03: same |
| `audit_082_provision_failure_emits_structured_diagnostic_object` | ok on Windows | DEF-04 is a POSIX-only defect (Windows already emitted structured diagnostics); the Unix RED proof is the CI Linux/macOS run of this same suite against the baseline |
| `audit_082_staging_shaped_debris_fails_closed_on_windows` | ok on Windows | documents the frozen J09 Windows fail-closed debris contract (invariant, not a Windows defect) |

## Unix (Linux x86-64 / macOS ARM64+Intel) RED results

Produced by CI on this evidence branch (baseline code + suites): see the PR
checks attached to this branch. Expected baseline failures:

- `audit_082_recognised_residue_never_blocks_ledger_reopen` (DEF-01: the
  deterministic `.{stem}.tmp` residue makes `ReplayLedger::open` and any
  republish impossible);
- `audit_082_provision_failure_emits_structured_diagnostic_object` (DEF-04:
  `data.diagnostic` is `null` on POSIX);
- `audit_082_group_writable_replay_storage_fails_closed_with_security_diagnostic`
  (RSK-01: no ownership/permission contract existed);
- `audit_082_exec_timeout_terminates_whole_posix_tree` (RSK-04: only the
  immediate child was killed; `supervision` self-reported
  `child_process_only`);
- `unix_abrupt_host_death_orphans_no_descendants` (DEF-02: descendants
  survive abrupt host death; no watchdog existed on the baseline).

CI run references are recorded in the Lane D worker note and final report.

## Verdict rules used by Agent D

- A repair is accepted only when the exact suite that failed here passes,
  unmodified, against the accepted candidate SHA.
- Any recovery logic that deletes, rewrites, or reclassifies durable evidence
  is challenged independently (Lane A's resolve route quarantines and audits;
  it never deletes the claim record).
