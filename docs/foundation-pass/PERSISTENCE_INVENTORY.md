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
| Candidate Registry | `candidate.rs:439-506` | `create()`: `write_new()` writes `.{id}.tmp` with `sync_all()` on tmp, then `fs::rename` to `{id}.json` | write-then-rename; atomic visibility guarantee UNVERIFIED (F3b) | `sync_all()` on tmp file before rename | UNVERIFIED (F3b) | `load_all()` (`candidate.rs:507-554`) rejects `.tmp`, validates `record_digest`, checks `candidate_id` matches file stem | `.tmp` = torn write; digest mismatch = corrupt; path/filename divergence = tamper; identity mismatch = tamper | `verify_existing_chain()` (`candidate.rs:71-100`) checks all ancestors for reparse points | `torn_temporary_record_fails_closed`, `filename_disagreement_and_duplicate_identity_evidence_fail_closed`, `preexisting_staging_target_and_escape_destination_fail_closed_without_write` |
| Publisher Trust Store | `trust.rs:262-414` via `m3_store.rs` | `StoreRoot::create_json()` (`trust.rs:414`) writes `.{id}.tmp`, `sync_all()` on tmp, `fs::rename` | write-then-rename; atomic visibility guarantee UNVERIFIED (F3b) | `sync_all()` | UNVERIFIED (F3b) | `current()` (`trust.rs:284`) rejects `.tmp`, validates record digest, predecessor chain continuity | `.tmp` = torn write; digest mismatch = corrupt; predecessor gap = chain break | `verify_chain()` on StoreRoot checks every ancestor for reparse | `trust_transitions_restart_and_revocation_fail_closed` |
| Developer Approval Store | `trust.rs:500-772` via `m3_store.rs` | `StoreRoot::create_json()` | write-then-rename; atomic visibility guarantee UNVERIFIED (F3b) | `sync_all()` | UNVERIFIED (F3b) | `find()` (`trust.rs:776`) rejects `.tmp`, validates record digest | Same as Publisher Trust | Same as Publisher Trust | `developer_approval_is_exact_digest_only` |
| Launch Profile Evidence | `launch_profile.rs` via `m3_store.rs` | `StoreRoot::create_json()` | write-then-rename; atomic visibility guarantee UNVERIFIED (F3b) | `sync_all()` | UNVERIFIED (F3b) | `load_all()` rejects `.tmp`, validates `profile_evidence_digest`, checks filename is digest | `.tmp` = torn write; digest mismatch = corrupt; filename divergence = tamper | StoreRoot `verify_chain()` | No inline tests; exercised through `tests/j24e_candidate_preparation.rs`, `tests/j24h_installation_evidence_access.rs`, `tests/j24j_installation_reconciliation.rs` |
| Conformance Evidence | `conformance.rs` via `m3_store.rs` | `StoreRoot::create_json()` | write-then-rename; atomic visibility guarantee UNVERIFIED (F3b) | `sync_all()` | UNVERIFIED (F3b) | `load_all()` rejects `.tmp`, validates evidence digest, checks `evidence_id` matches file stem | `.tmp` = torn write; digest mismatch = corrupt; identity mismatch = tamper | StoreRoot `verify_chain()` | No inline tests; exercised through `tests/m3_lifecycle.rs`, `tests/j23c2_pdf_conformance.rs`, `tests/j24k2_locked_single_step_executor.rs` |
| Installation Approval | `installed.rs:188-320` via `m3_store.rs` | `StoreRoot::create_json()` via the `InstallationApprovalStore::approve()` method | write-then-rename; atomic visibility guarantee UNVERIFIED (F3b) | `sync_all()` | UNVERIFIED (F3b) | `load_all()` (`installed.rs:290-319`) rejects `.tmp`, validates record digest, checks `approval_id` matches file stem | `.tmp` = torn write; digest mismatch = corrupt; identity mismatch = tamper; duplicate approval = conflict | StoreRoot `verify_chain()` | No inline tests; exercised through `installation_execution_tests.rs`, `tests/j24k2_locked_single_step_executor.rs`, `tests/m3_lifecycle.rs` |
| Installed Plug Registry | `installed.rs:662-1288` | Two-part: (1) `install_disabled()` creates staging dir `.staging-{id}`, copies files with per-file `sync_all()`, `fs::rename` staging to final `plug-{id}`; (2) `create_json` on record StoreRoot writes JSON record | rename of staging directory + StoreRoot write-then-rename; atomic visibility guarantee UNVERIFIED (F3b) | Per-file `sync_all()`; record `sync_all()` on tmp before rename | UNVERIFIED (F3b) | `load_all()` (`installed.rs:868-943`) rejects `.tmp` records, validates `record_digest`, checks `installed_id` matches file stem, cross-checks installed file set against records | `.tmp` = torn write; digest mismatch = corrupt; missing/excess/drifted payload = tamper; escaped path = tamper; duplicate identity or release = conflict | `verify_chain()` via StoreRoot; `reject_reparse()` on each entry; `recovery_expected_path()` destination validation | `installation_recovery_destination_tests.rs`, `installation_recovery_audit_tests.rs`, `installation_publication_mutation_tests.rs`, `tests/j24l2_plug_install_cli.rs` |
| Enablement Records | `enablement.rs:148-365` via `m3_store.rs` | `StoreRoot::create_json()` | write-then-rename; atomic visibility guarantee UNVERIFIED (F3b) | `sync_all()` | UNVERIFIED (F3b) | `load_all()` (`enablement.rs:291`) rejects `.tmp`, validates record digest, checks predecessor chain per installed identity | `.tmp` = torn write; digest mismatch = corrupt; predecessor gap = chain break | StoreRoot `verify_chain()` | `enablement_is_explicit_and_disable_removes_availability` |
| Replay Claim (identity) | `replay_windows.rs:887-966` | `publish_new_canonical_file_with_temporary_stem()`: `CreateFileW(CREATE_NEW \| FILE_FLAG_WRITE_THROUGH)` (`line 871-878`), `WriteFile`, `FlushFileBuffers` before rename (`line 924-929`), `SetFileInformationByHandle` rename (`line 930`), `FlushFileBuffers` after rename on renamed file handle (`line 932-936`), reopen and re-read verify (`line 943-964`) | handle-based rename + post-rename byte verification; atomic visibility guarantee UNVERIFIED (F3b) | `FlushFileBuffers` before rename (file data confirmed); `FlushFileBuffers` after rename (renamed file handle only) | UNVERIFIED (F3b) | `validate_whole_ledger()` (`line 1268-1274`) validates claim digests, checks chain integrity | Post-rename content re-read and compare to original bytes; digest mismatch = corrupt; orphan chain = fail closed | Handle-based `ValidatedHostRoot` with volume/ACL validation, `ValidatedLeafName`, retained open handles | Ledger 01-30 (`replay_windows.rs:2068-2829`) |

### 2. Replaceable Current-State Record

May be replaced with a newer value (remove then recreate). Previous state is discarded. Only one instance exists at any time.

| Store | Module | Write Primitive | Atomic Visibility | File Durability | Dir-Entry Durability | Recovery Reader | Corruption Classification | Unsafe-Path Protection | Tests |
|---|---|---|---|---|---|---|---|---|---|
| Installation Publication Intent | `installation_publication_intent.rs` via `m3_store.rs` | `StoreRoot::create_json("current", intent)` — singleton `current.json`. `create()` requires empty store (0 entries); `remove_if_matches()` must be called to clear before a new intent can be created. Not a direct overwrite. | write-then-rename; atomic visibility guarantee UNVERIFIED (F3b) | `sync_all()` | UNVERIFIED (F3b) | `load()` expects exactly 0 or 1 entry named `current.json`; validates intent digest | `.tmp` = torn write; digest mismatch = corrupt; more than 1 entry = corrupt; conflicting intent with different digest = conflict | StoreRoot `verify_chain()` | `installation_publication_intent_tests.rs` (19 tests), `f3c_installation_intent_publication_evidence.rs` (43 F3c characterization tests) |

### 3. Append-Only Causal Log

New entries appended; existing entries never modified.

| Store | Module | Write Primitive | Atomic Visibility | File Durability | Dir-Entry Durability | Recovery Reader | Corruption Classification | Unsafe-Path Protection | Tests |
|---|---|---|---|---|---|---|---|---|---|
| Trail (FileTrail) | `dispatch.rs:320-405` | JSONL line append: `writeln!`, `flush()`, `sync_data()` per line (`dispatch.rs:341-356`) | Per-line `sync_data()` — no rename | `sync_data()` after each line | UNVERIFIED (F3b) | `run_trail()` in `trail_command.rs:27` manually parses JSONL, filters by `execution_id`; no integrity footer or per-line digest | No per-line digest or checksum; no re-read verification | NONE — `FileTrail::open()` (`dispatch.rs:327-331`) accepts any path without chain verification | `file_trail_writes_durable_jsonl_intent` (`dispatch.rs:1156`), `file_trail_writes_durable_intent_and_outcome` (`dispatch.rs:1402`) |

### 4. Multi-Step Intent/Recovery Journal

Records intent through recovery steps. Intermediate states are valid and recoverable.

| Store | Module | Write Primitive | Atomic Visibility | File Durability | Dir-Entry Durability | Recovery Reader | Corruption Classification | Unsafe-Path Protection | Tests |
|---|---|---|---|---|---|---|---|---|---|
| Replay Generations (0-2) | `replay_windows.rs` | `publish_new_canonical_file_with_temporary_stem()` (same as identity claim above, `replay_windows.rs:900-966`) | handle-based rename + post-rename byte verification; atomic visibility guarantee UNVERIFIED (F3b) | Same as Replay Claim | UNVERIFIED (F3b) | `validate_whole_ledger()` (`line 1268-1274`) and `read_generation_directory()` (`line 1376-1400`) walk generation chain; predecessor mismatch fails closed; orphan chains fail whole ledger closed | Generation 3+ rejected on creation (`generation_filename`, `line 1213`); malformed claim digest fails closed; wrong predecessor breaks chain | `ValidatedHostRoot` + handle-based TOCTOU prevention | Ledger 01-30 in `replay_windows.rs:2068-2829` |
| Installation Recovery Staging | `installed.rs` | Multi-step journal: `install_disabled_with_authority()` (or the separated recovery functions `build_installation_recovery_staging`, `rename_installation_recovery_staging`, `publish_installation_recovery_record`) creates staging dir `.staging-{id}`, copies files, `fs::rename` to `plug-{id}`, then writes record via StoreRoot `create_json` | rename of staging directory + StoreRoot write-then-rename; each step rename-based; atomic visibility guarantee UNVERIFIED (F3b) | Per-file `sync_all()` + record `sync_all()` | UNVERIFIED (F3b) | `observe_installation_recovery()` snapshots staging/destination/record state; `audit_installation_recovery_destinations()` cross-validates disk against records | Recovery staging detects missing/present destinations, mismatched file sets, reparse point intrusion, stale `.tmp` remnants | `verify_chain()` + `reject_reparse()` on install/record roots and each entry | `installation_recovery_destination_tests.rs`, `installation_recovery_audit_tests.rs`, `installation_recovery_execution_tests.rs`, `installation_recovery_observation_tests.rs` |
| Local Anchor Admission Store | `local_anchor.rs:285-465` | `atomic_create()` (`line 515-527`): writes `{name}.tmp`, `sync_all()`, `fs::rename`; evaluation records via `atomic_create_bytes()` (`line 529-541`) with same pattern | write-then-rename; atomic visibility guarantee UNVERIFIED (F3b) | `sync_all()` on tmp before rename | UNVERIFIED (F3b) | `AdmissionStore::open()` (`line 286-341`) sorts by filename, validates `record_digest`, detects duplicates by SHA-256 name collision, detects evaluation completion records | `.tmp` = torn write; digest mismatch = corrupt; duplicate event record detected; schema version mismatch = corrupt | Partial: `safe_filename()` (SHA-256 hash) prevents traversal; source path canonicalized and checked but no `verify_chain()` or `reject_reparse()` on store root directory | `same_id_same_digest_is_duplicate_after_restart`, `corrupted_record_refuses_restart`, `notification_acknowledges_only_after_admission`, `identity_mismatch_is_refused_before_admission` |

---

## Filesystem Coordination Artifacts — Not Persistence Stores

Artifacts that use the filesystem for coordination (locking, handle semantics)
without encoding durable intent, recovery state, or causal history. These are
not classified under the four persistence-store vocabulary.

| Artifact | Module | Mechanism | Protection | Durable State | OS Semantics | Tests |
|---|---|---|---|---|---|---|
| Installation Execution Lock | `installation_execution.rs:30-151` | Empty file anchor; `CreateFileW` with exclusive `share_mode(0)` + `SetHandleInformation(HANDLE_FLAG_INHERIT, 0)` (`line 106-138`). Holder never writes bytes. | `verify_chain()` on parent, `reject_reparse()` on lock path, post-acquisition reparse re-check (`line 141`). Non-empty lock anchor (len > 0) rejected as stale (`line 95-99`). | No durable data persisted. | Lock released on process exit (OS handle close). Empty anchor from clean exit is acceptable; non-empty anchor rejected. | `installation_execution_tests.rs`, `tests/j24k2_locked_single_step_executor.rs` |

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

The following evidence gaps are routed to F3b (Windows primitive evidence).
Each entry records the observed accepted-main primitive and the question F3b must answer.

| Store(s) | Observed Primitive | Outstanding F3b Question |
|---|---|---|
| All `StoreRoot`-backed stores (7 stores), Candidate Registry, Local Anchor Admission Store | Write to `.tmp` file, `sync_all()` / `FlushFileBuffers` on the temporary file, then `fs::rename` to final name. No flush on the parent directory. | Does the `fs::rename` after `sync_all()` guarantee that the directory entry for the renamed file persists after interruption or power loss? Is an explicit parent-directory flush required? |
| Trail (FileTrail) | JSONL line append with `writeln!`, `flush()`, `sync_data()` per line. No per-line digest, checksum, or integrity footer. | After interruption or power loss, can a partially written final line be recovered? Is line-level integrity adequate without per-line digests? |
| Local Anchor Admission Store | `create_dir_all()` on store root without preceding `verify_chain()` or `reject_reparse()`. SHA-256 filenames prevent traversal in individual filenames. | Can a reparse point on the store root directory subvert admission records? |
| Replay Ledger (Claims and Generations) | `FlushFileBuffers` before rename; `SetFileInformationByHandle` rename; `FlushFileBuffers` on the renamed file handle; reopen and re-read compare. No explicit flush on the parent directory. | Does the post-rename `FlushFileBuffers` on the renamed file handle ensure the parent directory entry is durable? Does the reopen/re-read comparison detect all cases of an interrupted rename? |
| Replay Ledger | `CreateFileW(CREATE_NEW \| FILE_FLAG_WRITE_THROUGH)` for temporary files. | Does `FILE_FLAG_WRITE_THROUGH` provide the expected file-data durability on the exact NTFS volume class and configuration used in production? |

---

## F3b Findings

F3b established direct Windows evidence for the five primitive clusters identified
by the F3a route map. Findings are recorded at `bedf96a` (initial) with corrections
at the branch tip.

Every PROVEN label corresponds to a hard assertion that would FAIL if the property
were false on the tested target. No PROVEN label is inferred from API names alone.

### F3b-1: `sync_all()` + `fs::rename` (StoreRoot / Candidate / Local Anchor)

**Test file:** `tests/f3b_windows_persistence_evidence.rs`

| Property | Status | Test Evidence |
|---|---|---|
| `sync_all()` returns success | PROVEN (F3b) | `sync_all_rename_flush_accepted` |
| Exact bytes survive close/reopen | PROVEN (F3b) | `sync_all_rename_bytes_survive_close_and_reopen` |
| Final path absent before rename | PROVEN (F3b) | `sync_all_rename_final_absent_before_rename` |
| Final path complete bytes after rename | PROVEN (F3b) | `sync_all_rename_final_absent_before_rename` |
| Temporary path disappears after rename | PROVEN (F3b) | `sync_all_rename_temporary_disappears_after_rename` |
| Multiple records independent | PROVEN (F3b) | `sync_all_rename_multiple_records_independent` |
| Stale .tmp visible after rename failure | PROVEN (F3b) | `sync_all_stale_tmp_visible_after_failure` |
| Atomic visibility during rename (concurrent) | UNVERIFIED (F3b) | No concurrent observer in any test |
| File data survives sudden power loss | UNVERIFIED (F3b) | Not tested; would require destructive simulation |
| Directory entry durable after power loss | UNVERIFIED (F3b) | `fs::rename` does not flush parent directory |

### F3b-2: Parent-directory durability feasibility

**Test file:** `tests/f3b_windows_persistence_evidence.rs`

| Property | Status | Test Evidence |
|---|---|---|
| CreateFileW opens directory with FILE_GENERIC_WRITE | PROVEN (F3b) | `parent_directory_flush_feasibility` (hard assertion) |
| FlushFileBuffers on directory handle accepted | PROVEN (F3b) | `parent_directory_flush_feasibility` (hard assertion) |
| Current implementation performs parent-directory flush | DISPROVEN (F3b) | Source audit: no store calls FlushFileBuffers on a directory |
| Directory entry survives power loss after flush | UNVERIFIED (F3b) | Depends on volume write-cache behaviour |

The test directly asserts that both directory-open and FlushFileBuffers succeed.
If either fails, the test FAILS and the F3b-2 route is UNVERIFIED for this target.
Windows accepted the flush on this opened directory handle on the tested primary target.
This does NOT mean the directory entry would survive sudden power loss.

### F3b-3: Replay Windows publish primitive

**Test file:** `src/replay_windows.rs` inline tests

| Property | Status | Test Evidence |
|---|---|---|
| CreateFileW(CREATE_NEW \| FILE_FLAG_WRITE_THROUGH) accepted | PROVEN (F3b) | `f3b3_create_write_through_open_and_write` |
| WriteFile writes complete bytes | PROVEN (F3b) | `f3b3_create_write_through_open_and_write` |
| FlushFileBuffers before rename accepted | PROVEN (F3b) | `f3b3_flush_before_rename_file_data_durability` |
| SetFileInformationByHandle rename accepted | PROVEN (F3b) | `f3b3_flush_before_rename_file_data_durability` |
| FlushFileBuffers on renamed handle accepted | PROVEN (F3b) | `f3b3_flush_before_rename_file_data_durability` |
| Exact bytes survive close/reopen | PROVEN (F3b) | `f3b3_flush_before_rename_file_data_durability` |
| CREATE_NEW rejects existing file | PROVEN (F3b) | `f3b3_create_new_prevents_overwrite` |
| ReplaceIfExists:false blocks replacement | PROVEN (F3b) | `f3b3_rename_without_replacement_defence` |
| Atomic visibility during rename (concurrent) | UNVERIFIED (F3b) | No concurrent observer in any test |
| File data survives sudden power loss | UNVERIFIED (F3b) | Not tested |
| Parent directory entry durable after power loss | UNVERIFIED (F3b) | Post-rename FlushFileBuffers on file handle, not parent dir |

### F3b-4: Trail JSONL interruption behaviour

**Test file:** `src/dispatch.rs` inline tests

| Property | Status | Test Evidence |
|---|---|---|
| Complete line survives close/reopen | PROVEN (F3b) | `trail_complete_line_survives_close_and_reopen` |
| Multiple complete lines ordered and parseable | PROVEN (F3b) | `trail_multiple_complete_lines_ordered_and_parseable` |
| Truncated final line present and non-parseable (raw serde_json) | PROVEN (F3b) | `trail_truncated_final_line_present_and_non_parseable` |
| Incomplete-line raw bytes present in file | PROVEN (F3b) | `trail_incomplete_line_bytes_present_in_file` |
| Production Trail reader classification of truncated entry | UNVERIFIED (F3b) | Tested with raw `serde_json::from_str` only; production reader at `trail_command.rs:run_trail()` not exercised here |

### F3b-5: Local Anchor root reparse-point safety

**Test file:** `tests/f3b_windows_persistence_evidence.rs`

| Property | Status | Test Evidence |
|---|---|---|
| Reparse point on root redirects admission writes | UNVERIFIED (F3b) | `mklink /J` requires admin/dev-mode; test tooling limitation prevents characterization on this host |
| SHA-256 safe filenames prevent traversal in individual names | PROVEN (F3b) | `safe_filename()` uses SHA-256 hash; no path separators possible |
| Root has `verify_chain()` / `reject_reparse()` protection | DISPROVEN (F3b) | Source audit: `AdmissionStore::open()` calls `fs::create_dir_all()` without chain verification (`local_anchor.rs:288`) |

If the junction test succeeds on an elevated-privilege re-run, the exposure would be
DISPROVEN (defect confirmed). The test currently passes (returns UNVERIFIED due to
tooling limitation) rather than silently claiming safety.

### Corrected evidence matrix

For every primitive on the primary Windows target:

| Property | StoreRoot-style | Replay | Trail |
|---|---|---|---|
| Flush accepted/succeeded | PROVEN (F3b) | PROVEN (F3b) | PROVEN (F3b) |
| Exact bytes survive close/reopen | PROVEN (F3b) | PROVEN (F3b) | PROVEN (F3b) |
| Final-name absent before rename | PROVEN (F3b) | PROVEN (F3b) | N/A |
| Final-name complete after rename | PROVEN (F3b) | PROVEN (F3b) | N/A |
| Atomic visibility during rename | **UNVERIFIED** | **UNVERIFIED** | N/A |
| File data survives power loss | **UNVERIFIED** | **UNVERIFIED** | **UNVERIFIED** |
| Directory entry durable after power loss | **UNVERIFIED** | **UNVERIFIED** | **UNVERIFIED** |
| Truncated line raw present & non-parseable | N/A | N/A | PROVEN (F3b) |
| Production reader classification of truncation | N/A | N/A | UNVERIFIED (F3b) |
| CREATE_NEW exclusion | N/A | PROVEN (F3b) | N/A |
| ReplaceIfExists:false blocks replacement | N/A | PROVEN (F3b) | N/A |
| Parent-directory flush feasible | PROVEN (F3b) | N/A | N/A |
| Production performs parent-directory flush | DISPROVEN (F3b) | DISPROVEN (F3b) | DISPROVEN (F3b) |
| Reparse-point defence on root | N/A | N/A | UNVERIFIED (F3b) |

---

---
## F3c Evidence — Installation intent and publication contract

Date: 2026-08-07
Baseline: `71f79f7c80b2a09921ee59ac4b1acfa3926bf834` (accepted F3b)
Evidence sources:
- `tethers-0.1/host-rust/src/f3c_installation_intent_publication_evidence.rs` (44 F3c characterization tests)
- `tethers-0.1/host-rust/src/installation_publication_mutation_tests.rs` (existing j24k3e2_* mutation tests)
- `tethers-0.1/host-rust/src/installation_recovery_execution_tests.rs` (existing j24k3d2_* executor tests)
- `tethers-0.1/host-rust/src/installation_execution_tests.rs` (existing j24k3f_* tests)

Every PROVEN property maps to at least one hard assertion in exactly one named test. Evidence not directly asserted by a F3c characterization test cites the specific existing test that provides the proof.

### F3c-1 — Publication intent identity: PROVEN

| Property | Verification |
|---|---|
| Intent has one canonical identity via digest | PROVEN — `from_precomputed_record` produces deterministic digest; same record → same intent |
| Stored bytes bind exact installation operation | PROVEN — tampering any content field invalidates digest and `load()` returns `installation_intent_invalid` |
| Conflicting intent cannot silently replace existing | PROVEN — `create()` with different intent returns `installation_intent_conflict`; original bytes unchanged |
| Exact duplicate/retry is deterministic | PROVEN — second `create()` with same intent returns `installation_intent_conflict` |
| Singleton `current.json` contract enforced | PROVEN — zero entries → `None`, wrong filename → invalid, extra entries → invalid |
| Malformed/duplicate state fails closed | PROVEN — invalid JSON, unknown fields, `.tmp` remnants all return `installation_intent_invalid` |

### F3c-2 — Exact-match removal: PROVEN

| Property | Verification |
|---|---|
| Only exact match removed | PROVEN — `remove_if_matches` returns `Ok(true)` only when `current == expected` |
| Wrong digest cannot remove | PROVEN — different record → different intent → `installation_intent_conflict` |
| Wrong installation identity cannot remove | PROVEN — different `installed_id` → mismatch → conflict |
| Stale intent cannot remove newer/different | PROVEN — any non-matching intent → conflict |
| Invalid expected does not mutate store | PROVEN — `expected.validate()` fails before any I/O; bytes preserved |
| Malformed state cannot become absence | PROVEN — invalid expected → `installation_intent_invalid`, not `Ok(false)` |
| Missing distinguished from mismatched | PROVEN — absent → `Ok(false)`, present but different → `Err(conflict)` |

### F3c-3 — Publication ordering: PROVEN

The publication sequence in `execute_prepared_disabled_installation_publication` follows a strict order:

| Step | State | Evidence |
|---|---|---|
| 0. Pre-condition | No intent, idle recovery | PROVEN — `f3c3_intent_lifecycle_is_deterministic` |
| 1. Intent created | `current.json` present, canonical bytes, no staging/destination/records | PROVEN — `f3c3_intent_creation_is_the_first_publication_step` |
| 2-5. Staging → destination → record → intent removal | Full production sequence | PROVEN — `j24k3e2_valid_prepared_publication_completes_exactly_once` calls `execute_prepared_disabled_installation_publication`; hard-asserts final state (destination exists, record exists, intent removed, staging gone) |
| Post-intent failure boundary | Intent persists but staging/destination/records NOT created | PROVEN — `j24k3f_test_only_post_intent_failure_is_recoverable_and_publishes_once` uses `post_intent_failure_test_hook`; hard-asserts intent loaded, no staging, no destination, no records |

Power-loss durability of intermediate states: UNVERIFIED (F3b).

### F3c-4 — Recovery state matrix: PROVEN

| Staging | Destination | Record | Disposition | Evidence |
|---|---|---|---|---|
| false | false | None | RemoveIntentOnly | PROVEN |
| true | false | None | RemoveStagingThenIntent | PROVEN |
| false | true | None | RevalidateDestinationThenPublishRecord | PROVEN |
| false | true | Matching | VerifyCompletedPublicationThenRemoveIntent | PROVEN |
| false | false | Some | **conflict** | PROVEN |
| true | false | Some | **conflict** | PROVEN |
| true | true | None | **conflict** | PROVEN |
| true | true | Matching | **conflict** | PROVEN |

All 4 invalid states return `installation_recovery_conflict`. Invalid intent returns `installation_intent_invalid`. Classification is deterministic and idempotent. Matching is exact equality (`==`).

### F3c-5 — Recovery must not destroy evidence: PROVEN

Classification-level (F3c tests): the classifier returns the correct disposition or error. These assertions prove the decision, not the filesystem effect.

Executor-level (existing j24k3d2_* tests): the recovery executor (`execute_validated_installation_recovery`) is directly exercised and filesystem state snapshotted before/after.

| Property | Classification | Executor |
|---|---|---|
| Mismatched destination never deleted | PROVEN — `f3c5_classifier_mismatched_destination_returns_revalidate_not_delete` | PROVEN — `j24k3d2_recovery_never_adopts_or_deletes_final_destination` (tree_snapshot before/after byte-identical) |
| Mismatched record never overwritten | PROVEN — `f3c5_classifier_mismatched_record_returns_conflict_not_overwrite` | PROVEN — `j24k3d2_completed_publication_removes_only_intent` (destination + record tree_snapshot byte-identical) |
| Unrelated staging never removed | PROVEN — `f3c5_classifier_staging_plus_destination_returns_conflict` | PROVEN — `j24k3d2_staging_recovery_removes_exact_staging_then_intent` (sibling `.staging-*` survives) |
| Wrong intent never cleared | PROVEN — `f3c5_wrong_intent_is_never_cleared` (byte snapshot before/after) | Same test |
| Corruption evidence preserved | PROVEN — `f3c5_corruption_tamper_evidence_preserved` (tampered file remains on disk) | Same test |
| Ambiguous states fail closed | Classification PROVEN — `f3c5_all_four_classified_invalid_states_return_error` | Specific executor states PROVEN (j24k3d2_* tests); broad proof across all 4 invalid states UNVERIFIED |
| Unrelated stores unchanged | — | PROVEN — `j24k3d2_unrelated_stores_remain_unchanged` (6 unrelated stores byte-identical) |
| Idle performs no mutation | — | PROVEN — `j24k3d2_idle_plan_performs_no_mutation` (tree_snapshot before/after byte-identical) |

### F3c-6 — Canonical bytes / digest truth: PROVEN

| Property | Verification |
|---|---|
| Digest equals sha256 of exact covered representation | PROVEN — `f3c6_digest_computed_over_canonical_representation`: independently clears `intent_digest`, canonical serializes, sha256, asserts `== intent.intent_digest` |
| Read-back identity checked | PROVEN — `f3c6_read_back_identity_is_checked`: `load()` calls `intent.validate()` which recomputes digest from `covered_bytes()` |
| Filename identity disagreement fails closed | PROVEN — `f3c6_filename_record_identity_disagreement_fails_closed`: `load()` requires exactly `current.json` |
| Recovery decisions use validated persisted state | PROVEN — `f3c6_recovery_decisions_use_validated_persisted_state`: `classify_installation_recovery` validates intent before classification |
| Written bytes are canonical | PROVEN — `f3c6_written_bytes_are_canonical_intent`: `create()` output bytes == `canonical(intent)` |
| All content fields digest-covered | PROVEN — `f3c6_every_content_field_is_digest_covered`: tampering any of 4 content fields invalidates digest |

### F3c unresolved

- Power-loss durability: UNVERIFIED for all 7 publication steps (F3b)
- Concurrent rename atomicity: UNVERIFIED (F3b)
- Parent-directory flush: production does not perform it (F3b)

No defects found in the installation intent/publication/recovery contract. All F3c properties are PROVEN with hard assertions.

## Changes Made in F3a

### Initial F3a pass

- **Baseline updated** from `24428139` (F1) to `83eec98a` (accepted F2).
- **Installed Plug Registry staging naming corrected**: staging directories use `.staging-{id}` prefix, not `.{id}.tmp` suffix. Installation Recovery Staging and the Install Disabled path both use this convention.
- **Write-primitive descriptions clarified**: every store now records whether `sync_all()`/`FlushFileBuffers` occurs before or after rename, and on which handle.
- **Test citations corrected**: Launch Profile Evidence, Conformance Evidence, and Installation Approval stores have no inline `#[cfg(test)]` modules. Tests cited now reference the actual test files that exercise these stores.
- **Installation Publication Intent clarified**: the write primitive is remove-then-recreate via `remove_if_matches()` + `create()`, not a direct overwrite.
- **All line-number references added** for concrete source/traceability.
- **In-Memory Appendix renamed** to clarify process-local state is not persistence.

### Correction pass (r2)

- **Atomic Visibility column rewritten**: every cell now records the observed primitive (e.g., "write-then-rename") with the qualification "atomic visibility guarantee UNVERIFIED (F3b)". The presence of `rename` in accepted-main source is not evidence of the exact Windows atomic-visibility or crash guarantee.
- **F3b Route Map rewritten as questions/evidence gaps**: prescriptive statements about what NTFS "requires" or what "does not guarantee" replaced by (a) the observed accepted-main primitive and (b) the outstanding F3b question. F3a does not answer F3b.
- **Installation Execution Lock reclassified**: removed from the four-class persistence-store inventory. Added as a filesystem coordination artifact in a separate appendix. The lock creates an empty file anchor solely to hold an exclusive OS handle; it encodes no durable intent, recovery state, or causal history.
- **Totals recalculated**: 14 classified persistence stores (9 immutable atomic records + 1 replaceable current-state record + 1 append-only causal log + 3 multi-step journals), plus 1 coordination artifact and 6 in-memory stores.

## Key Differences From F1 Inventory

These differences from the original F1 inventory remain after F3a review:

- **Trail is NOT write-then-rename**: It appends JSONL lines and syncs in place. Previous version incorrectly grouped it under Immutable Atomic Record.
- **No store has a proven directory-entry durability guarantee**: Across all stores, the accepted-main code performs file-level sync (`sync_all()`, `FlushFileBuffers`) either before or after a rename, but contains no direct test establishing persistence of the directory entry after interruption or power loss. The Replay Ledger additionally reopens and re-reads the final bytes for comparison but does not flush the parent directory. All atomic visibility and directory-entry durability questions are routed to F3b.
- **`m3_store.rs` (StoreRoot) is the common persistence layer** for 7 stores, not an independent store. It provides a consistent write-then-rename-with-sync pattern.
- **No store uses the "Replaceable Current-State Record" pattern for direct overwrite**: Only Installation Publication Intent fits this class, using an explicit remove-then-recreate pattern. Previous version incorrectly classified Candidate Registry as replaceable.
- **Installation Recovery Plan is NOT a store**: It is a read-only planner that coordinates through the intent store and installed registry. Previous version classified it as a store.
- **Installation Execution Lock is a filesystem coordination artifact**: It creates an empty file anchor solely to hold an exclusive OS handle. It encodes no durable intent, recovery state, or causal history and is not classified under the four persistence-store vocabulary.
