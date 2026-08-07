# F3a Persistence Inventory

Baseline: `83eec98a0f33f964623f4cbbf4548a76bbdf5255` (`origin/main`, accepted F2)
Source: `tethers-0.1/host-rust/src/`

Every claim is evidence-backed from the accepted-main source and direct tests.
Claims not proved by accepted-main evidence are marked `UNVERIFIED (F3b)`.
This inventory does not repair any persistence behaviour.

## Durable Classes

### 1. Immutable Atomic Record

Once written via write-then-rename, never modified. Corrupt or missing record is invalid state.

| Store | Module | Write Primitive | Atomic Visibility | File Durability | Dir-Entry Durability | Recovery Reader | Corruption Classification | Unsafe-Path Protection | Tests |
|---|---|---|---|---|---|---|---|---|---|
| Candidate Registry | `candidate.rs:439-506` | `create()`: `write_new()` writes `.{id}.tmp` with `sync_all()` on tmp, then `fs::rename` to `{id}.json` | Atomic rename (NTFS) | `sync_all()` on tmp file before rename | UNVERIFIED (F3b) | `load_all()` (`candidate.rs:507-554`) rejects `.tmp`, validates `record_digest`, checks `candidate_id` matches file stem | `.tmp` = torn write; digest mismatch = corrupt; path/filename divergence = tamper; identity mismatch = tamper | `verify_existing_chain()` (`candidate.rs:71-100`) checks all ancestors for reparse points | `torn_temporary_record_fails_closed`, `filename_disagreement_and_duplicate_identity_evidence_fail_closed`, `preexisting_staging_target_and_escape_destination_fail_closed_without_write` |
| Publisher Trust Store | `trust.rs:262-414` via `m3_store.rs` | `StoreRoot::create_json()` (`trust.rs:414`) writes `.{id}.tmp`, `sync_all()` on tmp, `fs::rename` | Atomic rename | `sync_all()` | UNVERIFIED (F3b) | `current()` (`trust.rs:284`) rejects `.tmp`, validates record digest, predecessor chain continuity | `.tmp` = torn write; digest mismatch = corrupt; predecessor gap = chain break | `verify_chain()` on StoreRoot checks every ancestor for reparse | `trust_transitions_restart_and_revocation_fail_closed` |
| Developer Approval Store | `trust.rs:500-772` via `m3_store.rs` | `StoreRoot::create_json()` | Atomic rename | `sync_all()` | UNVERIFIED (F3b) | `find()` (`trust.rs:776`) rejects `.tmp`, validates record digest | Same as Publisher Trust | Same as Publisher Trust | `developer_approval_is_exact_digest_only` |
| Launch Profile Evidence | `launch_profile.rs` via `m3_store.rs` | `StoreRoot::create_json()` | Atomic rename | `sync_all()` | UNVERIFIED (F3b) | `load_all()` rejects `.tmp`, validates `profile_evidence_digest`, checks filename is digest | `.tmp` = torn write; digest mismatch = corrupt; filename divergence = tamper | StoreRoot `verify_chain()` | No inline tests; exercised through `tests/j24e_candidate_preparation.rs`, `tests/j24h_installation_evidence_access.rs`, `tests/j24j_installation_reconciliation.rs` |
| Conformance Evidence | `conformance.rs` via `m3_store.rs` | `StoreRoot::create_json()` | Atomic rename | `sync_all()` | UNVERIFIED (F3b) | `load_all()` rejects `.tmp`, validates evidence digest, checks `evidence_id` matches file stem | `.tmp` = torn write; digest mismatch = corrupt; identity mismatch = tamper | StoreRoot `verify_chain()` | No inline tests; exercised through `tests/m3_lifecycle.rs`, `tests/j23c2_pdf_conformance.rs`, `tests/j24k2_locked_single_step_executor.rs` |
| Installation Approval | `installed.rs:188-320` via `m3_store.rs` | `StoreRoot::create_json()` via the `InstallationApprovalStore::approve()` method | Atomic rename | `sync_all()` | UNVERIFIED (F3b) | `load_all()` (`installed.rs:290-319`) rejects `.tmp`, validates record digest, checks `approval_id` matches file stem | `.tmp` = torn write; digest mismatch = corrupt; identity mismatch = tamper; duplicate approval = conflict | StoreRoot `verify_chain()` | No inline tests; exercised through `installation_execution_tests.rs`, `tests/j24k2_locked_single_step_executor.rs`, `tests/m3_lifecycle.rs` |
| Installed Plug Registry | `installed.rs:662-1288` | Two-part: (1) `install_disabled()` creates staging dir `.staging-{id}`, copies files with per-file `sync_all()`, `fs::rename` staging to final `plug-{id}`; (2) `create_json` on record StoreRoot writes JSON record | Atomic rename of staging dir; atomic rename of record via StoreRoot | Per-file `sync_all()`; record `sync_all()` on tmp before rename | UNVERIFIED (F3b) | `load_all()` (`installed.rs:868-943`) rejects `.tmp` records, validates `record_digest`, checks `installed_id` matches file stem, cross-checks installed file set against records | `.tmp` = torn write; digest mismatch = corrupt; missing/excess/drifted payload = tamper; escaped path = tamper; duplicate identity or release = conflict | `verify_chain()` via StoreRoot; `reject_reparse()` on each entry; `recovery_expected_path()` destination validation | `installation_recovery_destination_tests.rs`, `installation_recovery_audit_tests.rs`, `installation_publication_mutation_tests.rs`, `tests/j24l2_plug_install_cli.rs` |
| Enablement Records | `enablement.rs:148-365` via `m3_store.rs` | `StoreRoot::create_json()` | Atomic rename | `sync_all()` | UNVERIFIED (F3b) | `load_all()` (`enablement.rs:291`) rejects `.tmp`, validates record digest, checks predecessor chain per installed identity | `.tmp` = torn write; digest mismatch = corrupt; predecessor gap = chain break | StoreRoot `verify_chain()` | `enablement_is_explicit_and_disable_removes_availability` |
| Replay Claim (identity) | `replay_windows.rs:887-966` | `publish_new_canonical_file_with_temporary_stem()`: `CreateFileW(CREATE_NEW | FILE_FLAG_WRITE_THROUGH)` (`line 871-878`), `WriteFile`, `FlushFileBuffers` before rename (`line 924-929`), `SetFileInformationByHandle` rename (`line 930`), `FlushFileBuffers` after rename on renamed file handle (`line 932-936`), reopen and re-read verify (`line 943-964`) | Atomic rename + post-rename verify | `FlushFileBuffers` before rename (file data confirmed); `FlushFileBuffers` after rename (renamed file handle only) | UNVERIFIED (F3b) | `validate_whole_ledger()` (`line 1268-1274`) validates claim digests, checks chain integrity | Post-rename content re-read and compare to original bytes; digest mismatch = corrupt; orphan chain = fail closed | Handle-based `ValidatedHostRoot` with NTFS validation, `ValidatedLeafName`, retained open handles | Ledger 01-30 (`replay_windows.rs:2068-2829`) |

### 2. Replaceable Current-State Record

May be replaced with a newer value (remove then recreate). Previous state is discarded. Only one instance exists at any time.

| Store | Module | Write Primitive | Atomic Visibility | File Durability | Dir-Entry Durability | Recovery Reader | Corruption Classification | Unsafe-Path Protection | Tests |
|---|---|---|---|---|---|---|---|---|---|
| Installation Publication Intent | `installation_publication_intent.rs` via `m3_store.rs` | `StoreRoot::create_json("current", intent)` — singleton `current.json`. `create()` requires empty store (0 entries); `remove_if_matches()` must be called to clear before a new intent can be created. Not a direct overwrite. | Atomic rename | `sync_all()` | UNVERIFIED (F3b) | `load()` expects exactly 0 or 1 entry named `current.json`; validates intent digest | `.tmp` = torn write; digest mismatch = corrupt; more than 1 entry = corrupt; conflicting intent with different digest = conflict | StoreRoot `verify_chain()` | `installation_publication_intent_tests.rs` |

### 3. Append-Only Causal Log

New entries appended; existing entries never modified.

| Store | Module | Write Primitive | Atomic Visibility | File Durability | Dir-Entry Durability | Recovery Reader | Corruption Classification | Unsafe-Path Protection | Tests |
|---|---|---|---|---|---|---|---|---|---|
| Trail (FileTrail) | `dispatch.rs:320-405` | JSONL line append: `writeln!`, `flush()`, `sync_data()` per line (`dispatch.rs:341-356`) | Per-line `sync_data()` — no rename | `sync_data()` after each line | UNVERIFIED (F3b) | `run_trail()` in `trail_command.rs:27` manually parses JSONL, filters by `execution_id`; no integrity footer or per-line digest | No per-line digest or checksum; no re-read verification | NONE — `FileTrail::open()` (`dispatch.rs:327-331`) accepts any path without chain verification | `file_trail_writes_durable_jsonl_intent` (`dispatch.rs:1156`), `file_trail_writes_durable_intent_and_outcome` (`dispatch.rs:1402`) |

### 4. Multi-Step Intent/Recovery Journal

Records intent through recovery steps. Intermediate states are valid and recoverable.

| Store | Module | Write Primitive | Atomic Visibility | File Durability | Dir-Entry Durability | Recovery Reader | Corruption Classification | Unsafe-Path Protection | Tests |
|---|---|---|---|---|---|---|---|---|---|
| Replay Generations (0-2) | `replay_windows.rs` | `publish_new_canonical_file_with_temporary_stem()` (same as identity claim above, `replay_windows.rs:900-966`) | Atomic rename + post-rename verify | Same as Replay Claim | UNVERIFIED (F3b) | `validate_whole_ledger()` (`line 1268-1274`) and `read_generation_directory()` (`line 1376-1400`) walk generation chain; predecessor mismatch fails closed; orphan chains fail whole ledger closed | Generation 3+ rejected on creation (`generation_filename`, `line 1213`); malformed claim digest fails closed; wrong predecessor breaks chain | `ValidatedHostRoot` + handle-based TOCTOU prevention | Ledger 01-30 in `replay_windows.rs:2068-2829` |
| Installation Recovery Staging | `installed.rs` | Multi-step journal: `install_disabled_with_authority()` (or the separated recovery functions `build_installation_recovery_staging`, `rename_installation_recovery_staging`, `publish_installation_recovery_record`) creates staging dir `.staging-{id}`, copies files, `fs::rename` to `plug-{id}`, then writes record via StoreRoot `create_json` | Each step individually atomic (staging rename, record rename); overall multi-step | Per-file `sync_all()` + record `sync_all()` | UNVERIFIED (F3b) | `observe_installation_recovery()` snapshots staging/destination/record state; `audit_installation_recovery_destinations()` cross-validates disk against records | Recovery staging detects missing/present destinations, mismatched file sets, reparse point intrusion, stale `.tmp` remnants | `verify_chain()` + `reject_reparse()` on install/record roots and each entry | `installation_recovery_destination_tests.rs`, `installation_recovery_audit_tests.rs`, `installation_recovery_execution_tests.rs`, `installation_recovery_observation_tests.rs` |
| Installation Execution Lock | `installation_execution.rs:30-151` | `CreateFileW` with `share_mode(0)` (exclusive) + `SetHandleInformation(HANDLE_FLAG_INHERIT, 0)` (`line 106-138`) | Exclusive handle — no concurrent access | N/A (lock file, no data written) | N/A | Lock automatically released on process exit (OS). Non-empty lock anchor (len > 0) rejected as stale (`line 95-99`); empty anchor from clean exit is acceptable. | Non-empty lock anchor rejected as stale; reparse points on lock path or parent rejected | `verify_chain()` on parent + `reject_reparse()` on lock path + post-acquisition reparse re-check (`line 141`) | `installation_execution_tests.rs`, `tests/j24k2_locked_single_step_executor.rs` |
| Local Anchor Admission Store | `local_anchor.rs:285-465` | `atomic_create()` (`line 515-527`): writes `{name}.tmp`, `sync_all()`, `fs::rename`; evaluation records via `atomic_create_bytes()` (`line 529-541`) with same pattern | Atomic rename | `sync_all()` on tmp before rename | UNVERIFIED (F3b) | `AdmissionStore::open()` (`line 286-341`) sorts by filename, validates `record_digest`, detects duplicates by SHA-256 name collision, detects evaluation completion records | `.tmp` = torn write; digest mismatch = corrupt; duplicate event record detected; schema version mismatch = corrupt | Partial: `safe_filename()` (SHA-256 hash) prevents traversal; source path canonicalized and checked but no `verify_chain()` or `reject_reparse()` on store root directory | `same_id_same_digest_is_duplicate_after_restart`, `corrupted_record_refuses_restart`, `notification_acknowledges_only_after_admission`, `identity_mismatch_is_refused_before_admission` |

---

## In-Memory Appendix

Process-local state with no filesystem persistence. Restart expiry is deliberate semantics.
These are not persistence stores and do not survive process restart.

| Store | Module | Durability | Rationale | Tests |
|---|---|---|---|---|
| Trusted Manifest Store | `trusted_store.rs` | In-memory `HashMap` | Runtime cache of verified manifests; identity + digest dual-index preflighted before insertion | Inline `mod tests` |
| Approval Store | `approval.rs` | In-memory `HashMap` | "Intentionally process-local" (comment at line 181); restart expiry is a deliberate semantic; state transitions guarded by exact state checks | Inline `mod tests` |
| Event Queue | `event_queue.rs` | In-memory queue | Per-session queue; not persisted | Inline `mod tests` |
| Socket Catalogue | `socket.rs` | In-memory | Discovered from MCP providers; rebuilt on restart | Inline `mod tests` |
| Scope Bindings | `operational_scope.rs` | In-memory | Constructed per-call from enablement records | Inline `mod tests` |
| Execution Environment | `execution_environment.rs` | In-memory | Constructed per-request | Inline `mod tests` |

---

## F3b Route Map

The following guarantees could not be independently verified by F3a and are routed to F3b (Windows primitive evidence):

| Store(s) | Unverified Claim | Risk |
|---|---|---|
| All `StoreRoot`-backed stores (7 stores) + Candidate Registry + Local Anchor Admission Store | Directory-entry durability after rename | NTFS metadata flush to directory requires explicit `FlushFileBuffers` on the parent directory handle or `FILE_FLAG_WRITE_THROUGH` on CreateFile. These stores use `sync_all()` on the file but do not flush the parent directory. Rename alone does not guarantee directory metadata durability. |
| Trail (FileTrail) | Line-level recovery integrity | No per-line digest, checksum, or integrity footer. A crash mid-line could leave a partial last line. No path safety verification (`FileTrail::open()` accepts any path). |
| Local Anchor Admission Store | Store root reparse safety | No `verify_chain()` or `reject_reparse()` on the store root directory. SHA-256 filenames prevent traversal in filenames but not on the root itself. |
| Replay Ledger (Claims and Generations) | Directory-entry durability after rename | The second `FlushFileBuffers` call occurs on the renamed file handle, not the parent directory. File data is flushed before rename; the renamed file handle is flushed after rename; final bytes are reopened and compared. Parent-directory durability remains unverified. |
| Replay Ledger | `FILE_FLAG_WRITE_THROUGH` adequacy on any NTFS volume | `FILE_FLAG_WRITE_THROUGH` behaviour varies by volume type and configuration. F3b should verify on the exact target volume class. |
| Installation Execution Lock | N/A (lock durability not checked) | Lock file durability is not relevant (no data written). Recovery on process crash relies on OS handle release. |

---

## Changes Made in F3a

- **Baseline updated** from `24428139` (F1) to `83eec98a` (accepted F2).
- **Installed Plug Registry staging naming corrected**: staging directories use `.staging-{id}` prefix, not `.{id}.tmp` suffix. Installation Recovery Staging and the Install Disabled path both use this convention.
- **Write-primitive descriptions clarified**: every store now records whether `sync_all()`/`FlushFileBuffers` occurs before or after rename, and on which handle.
- **Test citations corrected**: Launch Profile Evidence, Conformance Evidence, and Installation Approval stores have no inline `#[cfg(test)]` modules. Tests cited now reference the actual test files that exercise these stores. Installed Plug Registry test citation changed from a method name (`audit_installation_recovery_destinations`) to concrete test file names.
- **Installation Publication Intent clarified**: the write primitive is remove-then-recreate via `remove_if_matches()` + `create()`, not a direct overwrite.
- **All line-number references added** for concrete source/traceability.
- **In-Memory Appendix renamed** to clarify process-local state is not persistence.

## Key Differences From F1 Inventory

These differences from the original F1 inventory remain correct after F3a review:

- **Trail is NOT write-then-rename**: It appends JSONL lines and syncs in place. Previous version incorrectly grouped it under Immutable Atomic Record.
- **No store has confirmed directory-entry durability**: The Replay Ledger calls `FlushFileBuffers` after rename on the renamed file handle, and final bytes are reopened and compared, but parent-directory durability remains unverified (F3b). All `StoreRoot`-backed stores and the Trail lack this flush entirely. All directory-entry durability claims are routed to F3b.
- **`m3_store.rs` (StoreRoot) is the common persistence layer** for 7 stores, not an independent store. It provides a consistent write-then-rename-with-sync pattern.
- **No store uses the "Replaceable Current-State Record" pattern for write-then-rename overwrites**: Only Installation Publication Intent fits this class. Previous version incorrectly classified Candidate Registry as replaceable.
- **Installation Recovery Plan is NOT a store**: It is a read-only planner that coordinates through the intent store and installed registry. Previous version classified it as a store.
