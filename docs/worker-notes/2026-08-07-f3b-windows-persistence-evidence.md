# Worker Note

Task: `F3b - Windows persistence primitive evidence`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `145a791ceb3f5e3b8855aeadbac83671d9a2b363`

Implementation checkpoint: `f8e5442f5a0f0d18a9f4f6a1f9cd7bdc4c4c525f`

## Requested outcome

Establish direct Windows evidence for five persistence primitive clusters
identified by F3a, using isolated characterization tests and platform
investigation. No production repair or persistence redesign.

## Changes made

### 1. F3b-1: `sync_all()` + `fs::rename` characterization
- **File:** `tests/f3b_windows_persistence_evidence.rs`
- Tests: `sync_all_rename_full_cycle_observed`, `sync_all_rename_survives_close_and_reopen`, `sync_all_rename_no_partial_file_visible`, `sync_all_rename_multiple_records_independent`, `sync_all_stale_tmp_visible_after_failure`
- All 7 named observable properties PROVEN during ordinary execution
- Power-loss directory durability: UNVERIFIED

### 2. F3b-2: Parent-directory durability feasibility
- **File:** `tests/f3b_windows_persistence_evidence.rs`
- Test: `parent_directory_flush_feasibility`
- Directory handle open with FILE_FLAG_BACKUP_SEMANTICS: feasible
- FlushFileBuffers on directory handle: OS accepts the call
- Current implementation: no parent-directory flush performed
- Full durability after power loss: UNVERIFIED

### 3. F3b-3: Replay Windows publish primitive
- **File:** `src/replay_windows.rs` inline tests
- Tests: `f3b3_create_write_through_open_and_write`, `f3b3_flush_before_rename_file_data_durability`, `f3b3_create_new_prevents_overwrite`, `f3b3_rename_without_replacement_defence`
- All 6 stages characterized individually
- FILE_FLAG_WRITE_THROUGH: file created
- WriteFile: complete write verified
- FlushFileBuffers before rename: file-data durability PROVEN
- SetFileInformationByHandle rename: non-replacing rename PROVEN
- FlushFileBuffers on renamed handle: flushes file handle only, NOT parent directory
- Reopen/re-read: PROVEN file-data persistence, NOT directory-entry durability
- CREATE_NEW exclusion: PROVEN against existing files

### 4. F3b-4: Trail JSONL interruption behaviour
- **File:** `src/dispatch.rs` inline tests
- Tests: `trail_complete_line_survives_close_and_reopen`, `trail_multiple_complete_lines_ordered_and_parseable`, `trail_truncated_final_line_detected`, `trail_incomplete_line_no_newline_present_in_raw_bytes`
- Complete lines survive close/reopen: PROVEN
- Multiple lines ordered and parseable: PROVEN
- Truncated final line detected as non-parseable JSON: PROVEN
- Incomplete line without newline present in raw bytes: PROVEN
- Current behaviour: reader sees incomplete bytes; JSON parse fails

### 5. F3b-5: Local Anchor root reparse-point safety
- **File:** `tests/f3b_windows_persistence_evidence.rs`
- Test: `local_anchor_reparse_point_can_redirect_writes`
- Junction creation requires administrator/developer-mode privileges
- Could not complete direct redirect test on this machine
- Verdict: UNVERIFIED. Root `verify_chain()`/`reject_reparse()` absent
- SHA-256 filenames prevent traversal in individual names but not root

### 6. Documentation
- `docs/CURRENT_CLINE_TASK.md`: F3b task packet with 5 numbered behaviours
- `docs/CURRENT_GOAL.md`: Updated to F3b active increment
- `docs/PROJECT_DASHBOARD.md`: Updated to F3b active task
- `docs/foundation-pass/PERSISTENCE_INVENTORY.md`: Added F3b Findings section with per-property PROVEN/UNVERIFIED tags and summary matrix
- `docs/foundation-pass/DEBT_LEDGER.md`: Updated A1 with F3b investigation results

## Decisions and assumptions

- Characterized what the OS and Rust std actually do on the primary target,
  without inferring guarantees from API names or documentation terminology.
- Distinguished file-data durability (flush on file handle), atomic visibility
  (rename of complete file, create_new exclusion), and directory-entry
  durability (flush on parent directory) as separate properties.
- F3b-5 junction test limitation (mklink requires admin privileges) is recorded
  honestly as UNVERIFIED; the exposure hypothesis remains unproven.
- No production seams widened. All tests are isolated characterization harnesses
  using temp directories and direct Windows API calls.

## Evidence

### F3b characterization tests

| Test | Package | Result |
|---|---|---|
| `sync_all_rename_full_cycle_observed` | F3b-1 | PASS |
| `sync_all_rename_survives_close_and_reopen` | F3b-1 | PASS |
| `sync_all_rename_no_partial_file_visible` | F3b-1 | PASS |
| `sync_all_rename_multiple_records_independent` | F3b-1 | PASS |
| `sync_all_stale_tmp_visible_after_failure` | F3b-1 | PASS |
| `parent_directory_flush_feasibility` | F3b-2 | PASS |
| `f3b3_create_write_through_open_and_write` | F3b-3 | PASS |
| `f3b3_flush_before_rename_file_data_durability` | F3b-3 | PASS |
| `f3b3_create_new_prevents_overwrite` | F3b-3 | PASS |
| `f3b3_rename_without_replacement_defence` | F3b-3 | PASS |
| `trail_complete_line_survives_close_and_reopen` | F3b-4 | PASS |
| `trail_multiple_complete_lines_ordered_and_parseable` | F3b-4 | PASS |
| `trail_truncated_final_line_detected` | F3b-4 | PASS |
| `trail_incomplete_line_no_newline_present_in_raw_bytes` | F3b-4 | PASS |
| `local_anchor_reparse_point_can_redirect_writes` | F3b-5 | PASS (mklink unavailable) |

### Full test suite

All 1273+ existing tests pass. No regressions.

## Verification matrix

| Command | Result |
|---|---|
| `git fetch origin --prune` | PASS |
| `git rev-parse origin/main` | PASS (`145a791`) |
| `git rev-parse HEAD` | PASS (`f8e5442`) |
| `git status --short --branch` | PASS (clean) |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --all-targets --all-features --locked` | PASS |
| `cargo test --all-targets --all-features --locked` | PASS (1273+ tests) |
| `cargo clippy --all-targets --all-features --locked -- -W clippy::all` | PASS (all warnings pre-existing) |
| `just verify` | PASS |
| `just verify-agent` | PASS (15/15 checks) |
| `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` | PASS |
| `git diff --exit-code origin/main...HEAD -- docs/foundation-pass/fixtures` | PASS (byte-identical) |
| `git diff --check origin/main...HEAD` | PASS |
| `git diff --name-only origin/main...HEAD` | PASS (8 files, no production repair) |

### Focused F3b characterization tests (explicit)

| Command | Result |
|---|---|
| `cargo test --lib -- f3b3` | PASS (4/4) |
| `cargo test --lib -- trail_complete\|trail_multiple\|trail_truncated\|trail_incomplete` | PASS (4/4) |
| `cargo test --test f3b_windows_persistence_evidence` | PASS (7/7) |

## Discoveries

- `FlushFileBuffers` on a Windows directory handle is technically feasible
  (the OS accepts the call), but full directory-entry durability depends on
  volume write-cache behaviour that cannot be controlled from user-mode Rust.
- The Replay post-rename `FlushFileBuffers` flushes the renamed file handle
  metadata/data, not the parent directory. The post-rename re-read proves
  file content integrity, not directory-entry durability.
- `SetFileInformationByHandle` with `ReplaceIfExists: false` correctly blocks
  replacement in the tested configuration, but the exclusion may be bypassed
  on certain volume types.
- Trail FileTrail has no per-line digest, checksum, or integrity footer.
  A truncated final line produces a JSON parse error. No automated recovery
  mechanism exists.
- `mklink /J` requires administrator or developer-mode privileges on
  contemporary Windows, limiting Local Anchor root reparse-point
  characterization without privilege elevation.

## Remaining risks

1. **Directory-entry durability after power loss**: UNVERIFIED for all 14
   stores. File-data flush is provably issued; the directory entry itself
   may not survive power loss. A later package (F3c/d) may add optional
   parent-directory flush where the platform supports it.
2. **Local Anchor root reparse-point exposure**: UNVERIFIED. The root
   has no `verify_chain()` or `reject_reparse()`. An elevated-privilege
   characterization would complete this evidence. If exposure is confirmed,
   repair belongs to a later package.
3. **Trail truncation recovery**: No automated recovery. A partial final
   line is present in the file and causes JSON parse failure. The reader
   has no integrity footer to distinguish corruption from truncation.
4. **Replay `ReplaceIfExists` bypass**: The `ReplaceIfExists: false` flag
   prevents replacement in standard configurations, but its enforcement may
   vary by volume type. The Replay's CREATE_NEW exclusion provides a
   stronger overlapping defence.

## Smallest next action

Push the branch to GitHub, then route to Lucy for independent review before
F3c. If Lucy's review identifies a correction, compile it as one bounded
task rather than beginning F3c.

## References

- Accepted main: `145a791ceb3f5e3b8855aeadbac83671d9a2b363`
- Task packet: `docs/CURRENT_CLINE_TASK.md`
- Foundation Pass plan: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- F3a worker note: `docs/worker-notes/2026-08-07-f3a-persistence-vocabulary.md`
- Persistence inventory: `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- Debt ledger: `docs/foundation-pass/DEBT_LEDGER.md`
