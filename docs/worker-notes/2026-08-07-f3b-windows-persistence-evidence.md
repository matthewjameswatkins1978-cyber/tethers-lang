# Worker Note

Task: `F3b - Windows persistence primitive evidence`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `145a791ceb3f5e3b8855aeadbac83671d9a2b363`

Implementation checkpoint: `bedf96af3988a93d531c69accae4523f3b43bd8d`

## Requested outcome

Establish direct Windows evidence for five persistence primitive clusters
identified by F3a. Correction pass (r2) splits every durability label
into three separate properties: flush accepted, close/reopen survival,
and power-loss survival. Atomic-visibility labels split into pre-rename
absence, post-rename completeness, and concurrent rename observation.
Every PROVEN label corresponds to a hard assertion that fails if false.

## Changes made

### Corrections applied (r2)

1. **Durability labels split**: "file-data durability PROVEN" replaced with
   three distinct properties. Flush accepted and close/reopen survival are
   PROVEN (F3b). Power-loss survival is UNVERIFIED (F3b).

2. **Atomic visibility corrected**: "atomic visibility PROVEN" replaced with
   pre-rename final-name absence (PROVEN), post-rename complete bytes (PROVEN),
   and atomic visibility during rename (UNVERIFIED, no concurrent observer).

3. **Parent-directory feasibility hardened**: `parent_directory_flush_feasibility`
   now directly asserts both `CreateFileW` succeeds AND `FlushFileBuffers`
   succeeds. If either fails, the test FAILS — F3b-2 becomes UNVERIFIED for
   that target rather than silently passing.

4. **Trail truncated-line evidence strengthened**:
   `trail_truncated_final_line_present_and_non_parseable` directly asserts
   exactly 2 lines, truncated line non-empty, and `serde_json::from_str` Err.
   No conditional branches that allow silent pass. Production Trail reader
   classification of truncated entries marked UNVERIFIED.

5. **Evidence matrix recalculated**: Every PROVEN item in
   `PERSISTENCE_INVENTORY.md` now corresponds to a hard assertion.

### F3b-1: `sync_all()` + `fs::rename`

Tests in `tests/f3b_windows_persistence_evidence.rs`:

| Test | Property | Status |
|---|---|---|
| `sync_all_rename_flush_accepted` | flush accepted | PROVEN |
| `sync_all_rename_bytes_survive_close_and_reopen` | bytes survive close/reopen | PROVEN |
| `sync_all_rename_final_absent_before_rename` | final absent before rename | PROVEN |
| `sync_all_rename_final_absent_before_rename` | final complete after rename | PROVEN |
| `sync_all_rename_temporary_disappears_after_rename` | tmp disappears | PROVEN |
| `sync_all_rename_multiple_records_independent` | records independent | PROVEN |
| `sync_all_stale_tmp_visible_after_failure` | stale tmp visible | PROVEN |
| — | atomic visibility during rename (concurrent) | UNVERIFIED |
| — | file data survives power loss | UNVERIFIED |
| — | directory entry survives power loss | UNVERIFIED |

### F3b-2: Parent-directory durability

| Property | Status |
|---|---|
| CreateFileW opens dir with FILE_GENERIC_WRITE | PROVEN (hard assert) |
| FlushFileBuffers on dir handle accepted | PROVEN (hard assert) |
| Production performs parent-dir flush | DISPROVEN |
| Dir entry survives power loss after flush | UNVERIFIED |

### F3b-3: Replay Windows publish primitive

Tests in `src/replay_windows.rs` inline tests:

| Property | Status |
|---|---|
| CreateFileW(CREATE_NEW \| FILE_FLAG_WRITE_THROUGH) accepted | PROVEN |
| WriteFile writes complete bytes | PROVEN |
| FlushFileBuffers before rename accepted | PROVEN |
| SetFileInformationByHandle rename accepted | PROVEN |
| FlushFileBuffers on renamed handle accepted | PROVEN |
| Exact bytes survive close/reopen | PROVEN |
| CREATE_NEW rejects existing file | PROVEN |
| ReplaceIfExists:false blocks replacement | PROVEN |
| Atomic visibility during rename (concurrent) | UNVERIFIED |
| File data survives power loss | UNVERIFIED |
| Dir entry survives power loss | UNVERIFIED |

### F3b-4: Trail JSONL interruption

Tests in `src/dispatch.rs` inline tests:

| Property | Status |
|---|---|
| Complete line survives close/reopen | PROVEN |
| Multiple lines ordered and parseable | PROVEN |
| Truncated final line present and non-parseable (raw serde_json) | PROVEN |
| Incomplete-line raw bytes present in file | PROVEN |
| Production Trail reader classification of truncated entry | UNVERIFIED |

### F3b-5: Local Anchor root reparse-point safety

| Property | Status |
|---|---|
| Junction redirects writes (admin required) | UNVERIFIED (mklink unavailable) |
| SHA-256 filenames prevent traversal in names | PROVEN |
| Root has verify_chain/reject_reparse | DISPROVEN |

## Decisions and assumptions

- Every PROVEN label corresponds to a hard assertion in a characterization test.
  No PROVEN label is inferred from API names or documentation terminology alone.
- Flush accepted ≠ power-loss durability proved. Close/reopen ≠ atomic
  visibility proved. Three separate properties per write primitive.
- The production Trail reader is NOT exercised in F3b-4. The truncated-line
  test uses raw `serde_json::from_str`. Production reader classification
  of truncated entries is UNVERIFIED unless tested separately.
- F3b-5 remains UNVERIFIED due to mklink privilege limitation.

## Evidence

### Focused characterization tests

| Command | Result |
|---|---|
| `cargo test --test f3b_windows_persistence_evidence` | PASS (8/8) |
| `cargo test --lib -- f3b3` | PASS (4/4) |
| `cargo test --lib -- dispatch::tests::trail_` | PASS (6/6, including 4 new) |

### Full test suite

`cargo test --all-targets --all-features --locked`: 1273+ tests PASS.

## Verification matrix

| Command | Result |
|---|---|
| `git fetch origin --prune` | PASS |
| `git rev-parse origin/main` | PASS (`145a791`) |
| `git status --short --branch` | PASS (clean) |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --all-targets --all-features --locked` | PASS |
| `cargo test --all-targets --all-features --locked` | PASS |
| `cargo clippy --all-targets --all-features --locked -- -W clippy::all` | PASS |
| `just verify` | PASS |
| `just verify-agent` | PASS (15/15) |
| `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` | PASS |
| `git diff --exit-code origin/main...HEAD -- docs/foundation-pass/fixtures` | PASS |
| `git diff --check origin/main...HEAD` | PASS |

## Discoveries

- `FlushFileBuffers` on a Windows directory handle is technically feasible
  (hard-asserted in `parent_directory_flush_feasibility`).
- The Replay post-rename `FlushFileBuffers` flushes the renamed file handle only.
- Trail FileTrail has no per-line digest, checksum, or integrity footer.
- `mklink /J` requires admin/dev-mode on contemporary Windows.

## Remaining risks

1. Power-loss durability: UNVERIFIED for all 14 stores.
2. Concurrent atomic visibility: UNVERIFIED for all rename-based stores.
3. Local Anchor root reparse-point: UNVERIFIED (privilege limitation).
4. Production Trail reader classification of truncated entries: UNVERIFIED.

## Smallest next action

Push the correction commit, then route to Lucy for independent review.
Do not begin F3c. If Lucy identifies a correction, compile it as one
bounded task.

## References

- Accepted main: `145a791ceb3f5e3b8855aeadbac83671d9a2b363`
- Task packet: `docs/CURRENT_CLINE_TASK.md`
- Foundation Pass plan: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- F3a worker note: `docs/worker-notes/2026-08-07-f3a-persistence-vocabulary.md`
