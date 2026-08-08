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

## F3d Evidence — Remaining bounded persistence stores

Date: 2026-08-07
Baseline: `40ec42eb2aac108901d428af3cbfe264d3edd6dc` (accepted F3c)

F3d corrects the evidence presentation, not production behaviour. A property is
`PROVEN` below only when the cited test contains the stated hard assertion. A
shared `StoreRoot` implementation is not itself evidence for a consuming store.
Every dimension not named in the `PROVEN` column is `UNVERIFIED`; that includes
power-loss and directory-entry durability from F3b. No F3d test upgrades Local
Anchor root reparse safety, which remains `UNVERIFIED (F3b)`.

| Store | PROVEN — exact hard assertion | UNVERIFIED in F3d |
|---|---|---|
| Candidate Registry | `torn_temporary_record_fails_closed`: `load_all()` returns `record_invalid` for `.candidate.tmp`; `filename_disagreement_and_duplicate_identity_evidence_fail_closed`: a wrong filename returns `record_invalid` and duplicate logical evidence returns `Err`; `preexisting_staging_target_and_escape_destination_fail_closed_without_write`: existing staging returns `already_exists`, an escaping destination returns `unsafe_destination`, and no outside marker exists. | Close/reopen, record-digest corruption, and root reparse protection are not claimed without another direct assertion. |
| Publisher Trust Store (`PublisherTrustStore` in `trust.rs`) | `trust_transitions_restart_and_revocation_fail_closed`: after reopen, a revoked key makes `require_trusted` return `trust_not_current`; duplicate append returns `trust_conflict`; `.torn.tmp` makes `current()` return `trust_store_invalid`; the stored publisher identity is asserted distinct from package presentation text. | Filename/content disagreement and unsafe-path protection have no F3d-cited direct negative test. `ExactCandidateTrustStore` is a separate installation-evidence store, not Publisher Trust. |
| Developer Approval Store | `f3d_dev_approval_duplicate_digest_is_conflict`: second approval returns `developer_approval_conflict`; `f3d_dev_approval_torn_tmp_detected_in_find`: `.tmp` makes `find` return `developer_approval_invalid`; `f3d_dev_approval_filename_mismatch_detected`: renamed record makes `find` return `developer_approval_invalid`; `f3d_dev_approval_reopen_preserves_record`: reopened `find` returns the exact digest and `visibly_unsigned`. | Unsafe-path protection and power-loss/directory-entry durability. |
| Launch Profile Evidence | `launch_profile_round_trip_is_exact`: one loaded record equals the created evidence; `launch_profile_duplicate_create_returns_record_conflict_and_no_mutation`: duplicate returns `record_conflict` and the snapshot is unchanged; `launch_profile_torn_tmp_rejected`: `.tmp` returns `launch_profile_store_invalid`; `launch_profile_filename_mismatch_rejected`: renamed evidence returns `launch_profile_store_invalid`; `launch_profile_malformed_evidence_rejected`: malformed bytes make `load_all` fail. | Unsafe-path protection and power-loss/directory-entry durability. |
| Conformance Evidence | `corrupt_conformance_evidence_fails_closed`: both `.tmp` and a non-JSON entry make planning return an `invalid` error and preserve the filesystem snapshot. | Create conflict, canonical filename, close/reopen, and unsafe-path protection have no F3d-cited direct assertion. |
| Installation Approval | `current_installation_approval_returns_publish_disabled_installation`: after `open_existing`, the persisted approval produces `PublishDisabledInstallation` with the exact approval ID and digest pins; `corrupt_approval_evidence_fails_closed`: `.tmp` makes planning return `invalid` and preserves the snapshot. | Create conflict, filename/content agreement, and unsafe-path protection have no F3d-cited direct assertion. |
| Installed Plug Registry | `corrupt_installed_evidence_fails_closed`: `.tmp` in the record root makes planning return `invalid` and preserves the snapshot; `j24k3c4_windows_junction_tracked_destination_load_all_refused`: a real Windows junction produces `unsafe_store_path`; `j24k3c4_record_destination_not_exact_plug_id_fails_closed` covers mismatched record/destination identity. | Close/reopen and exact duplicate-create behaviour have no F3d-cited direct assertion. |
| Enablement Records | `f3d_enablement_record_filename_mismatch_detected`: a syntactically valid record renamed to the wrong stem makes `load_all()` return `Err`; `enablement_is_explicit_and_disable_removes_availability`: availability is false before enable, true after enable, and false after disable. | Chain predecessor/sequence, close/reopen, corruption, and unsafe-path protection have no F3d-cited direct assertion. |
| Local Anchor Admission Store | `durable_local_anchor_restart_duplicate_conflict_and_scope`: after restart, the same event returns `DuplicateCompleted` with `sha256:terminal`, the duplicate keeps the anchor event ID, and different event content returns `Err`; `f3d_local_anchor_conflicting_evaluation_result_is_corrupt`: a second distinct result returns `EventError::Corrupt`. | Root reparse safety remains `UNVERIFIED (F3b)`; power-loss and directory-entry durability remain `UNVERIFIED (F3b)`. |

No directly demonstrated production defect arose from this evidence pass, so
`DEBT_LEDGER.md` is intentionally unchanged.

---

## F3e1 Evidence — Trail evidence harvest

Date: 2026-08-07
Baseline: `c9332bab072ce273db3aecc367faf64be71a8586` (accepted F3d)
Branch: `foundation/f3e1-trail-evidence`
Implementation checkpoint: `fb07c607a5c938d326489a03a7e1b474d6e88461`
Evidence sources:
- `tethers-0.1/host-rust/src/dispatch.rs` inline tests (F3b Trail characterization + F3e1 path safety)
- `tethers-0.1/host-rust/src/trail_command.rs` inline tests (production reader)

F3e1 was an evidence harvest only. Three characterization tests were added to close the remaining F3b UNVERIFIED gaps. No production code changed and no defect was found.

### Trail evidence summary

| Property | Status | Test | Exact Hard Assertion |
|---|---|---|---|
| Append order preserved | PROVEN | `trail_multiple_complete_lines_ordered_and_parseable` (dispatch.rs) | `assert_eq!(parsed[i]["arguments"]["idx"], i as u64)` |
| One JSONL record per completed write | PROVEN | `trail_complete_line_survives_close_and_reopen` (dispatch.rs) | `assert_eq!(lines.len(), 1)` |
| Flush/sync accepted | PROVEN (F3b) | F3b-1: `sync_all_rename_bytes_survive_close_and_reopen` (f3b_windows_persistence_evidence.rs) — sync survival proven at primitive level | Exact bytes survive close/reopen after sync |
| Close/reopen readback | PROVEN | 4 tests: `trail_complete_line_survives_close_and_reopen`, `trail_multiple_complete_lines_ordered_and_parseable`, `file_trail_writes_durable_jsonl_intent`, `file_trail_writes_durable_intent_and_outcome` (dispatch.rs) | Hard line-count and content assertions |
| Truncated final line: raw bytes present and non-parseable | PROVEN (F3b) | `trail_truncated_final_line_present_and_non_parseable` (dispatch.rs) | `assert!(parse_result.is_err())` |
| Production reader classification of truncated final line | PROVEN | `f3e1_truncated_final_line_maps_to_audit_failed` (trail_command.rs) | `assert_eq!(envelope["status"], "audit_failed"); assert_eq!(envelope["error"]["code"], "TRAIL_INVALID")` |
| Malformed complete line fails entire file | PROVEN | 6 `j13c_*_maps_to_audit_failed` tests (trail_command.rs): bad JSON, blank line, non-object, duplicate keys, non-string execution_id, oversize line | `assert_eq!(envelope["status"], "audit_failed")` |
| Fail-closed: later malformed line prevents all output | PROVEN | `j13c_malformed_later_prevents_all_output` (trail_command.rs) | `assert!(result.is_err())` |
| Execution_id filtering | PROVEN | 5 tests: `j13c_matching_entries_returned_in_original_order`, `j13c_unrelated_execution_ids_omitted`, `j13c_valid_audit_entries_without_execution_id_skipped`, `j13c_zero_matches_maps_to_not_found`, `j13c_non_string_execution_id_maps_to_audit_failed` (trail_command.rs) | Exact entry counts, content, and not-found assertions |
| FileTrail::open accepts relative paths | PROVEN | `f3e1_file_trail_open_accepts_relative_path` (dispatch.rs) | `assert!(abs_path.exists())` |
| Path validation inside FileTrail::open | DISPROVEN | `f3e1_file_trail_open_accepts_relative_path` (dispatch.rs) — relative path accepted without validation | Relative path succeeds; file created |
| Power-loss durability | UNVERIFIED (F3b) | Never upgrade | — |
| Directory-entry durability | UNVERIFIED (F3b) | Never upgrade | — |
| Parent-directory flush in production | DISPROVEN (F3b) | Source audit: no store performs parent-directory flush | — |

### F3e1 findings

- Production reader correctly fail-closes on truncated final line (TRAIL_INVALID, entire file rejected).
- FileTrail::open() provides no root/reparse/chain/absolute-path validation; callers (application.rs, run_trail()) enforce path safety independently.
- No production defect found.
- Replay was untouched.
- 58 Trail tests pass (55 existing + 3 new F3e1 characterization tests).

---

## F3e2a Evidence — Replay Claim evidence harvest

Date: 2026-08-08
Baseline: `dfae673407ecef38a9dcf8376b06ddbad4a97abc` (accepted F3e1)
Branch: `foundation/f3e2a-replay-claim-evidence`
Evidence sources:
- `tethers-0.1/host-rust/src/replay.rs` inline tests (unit-level Claim identity)
- `tethers-0.1/host-rust/src/replay_windows.rs` inline tests (ledger 01–30, F3b-3 primitives, F3e2a characterization)

F3e2a was an evidence harvest limited to the Replay Claim slice (LogicalExecutionKey, ExecutionId, ExecutionBinding, Claim, publication, scan, recovery). Replay Generations (0/1/2) are explicitly deferred to the next slice.

One characterization test added to close the filename/content identity disagreement gap. No production code changed and no defect was found.

### Replay Claim evidence summary

| # | Property | Status | Test | Exact Hard Assertion |
|---|---|---|---|---|
| 1 | Canonical logical-key identity | PROVEN | `sibling_actions_are_distinct` (replay.rs:589), `different_evaluations_are_distinct` (replay.rs:600) | `assert_ne!(key1.as_digest(), key2.as_digest())` — different action/eval IDs → different SHA-256 digests |
| 2 | Fresh immutable Claim creation | PROVEN | `ledger_05_fresh_claim_creates_one_host_execution_identity` (replay_windows.rs:2145): durable creation. `claim_round_trip_is_exact_canonical_and_redacted` (replay.rs:613): canonical form, redaction | `assert!(admission.is_fresh()); assert_eq!(dir.len(), 1)` — store-level durable creation. `assert_eq!(recovered, claim)` — canonical round-trip preserves identity; no `raw_argument` in canonical bytes |
| 3 | Execution identity creation | PROVEN | `ledger_05_fresh_claim_creates_one_host_execution_identity` (replay_windows.rs:2145) | `assert!(admission.is_fresh())`; `ExecutionId::parse` returns `Ok` — fresh claim creates parseable UUIDv4 execution_id |
| 4 | Close/reopen recovery of same Claim identity | PROVEN | `ledger_06_restart_recovers_same_execution_identity` (replay_windows.rs:2162) | `assert!(!recovered.is_fresh())`; `assert_eq!(recovered.execution_id(), first)` — ledger drop/reopen preserves identity |
| 5 | Existing Claim behaviour (collision) | PROVEN | `ledger_08_exact_claim_collision_recovers_only_valid_winner` (replay_windows.rs:2206) | `assert!(!recovered.is_fresh())`; `assert_eq!(recovered.execution_id(), winner)` — same tuple → same execution_id |
| 6 | Conflicting binding behaviour | PROVEN | `ledger_09_binding_mismatch_fails_closed` (replay_windows.rs:2226) | `assert!(matches!(result, Err(ReplayError::BindingMismatch)))` — different argument_digest → fail-closed |
| 7 | Malformed/noncanonical Claim handling | PROVEN | `non_canonical_or_unknown_claim_is_rejected` (replay.rs:635) | Spaced JSON and unknown-field JSON: `assert!(Claim::from_canonical_bytes(...).is_err())` |
| 8 | Claim digest corruption handling | PROVEN | `ledger_10_malformed_or_digest_invalid_claim_fails_closed` (replay_windows.rs:2246) | Forged `claim_digest` → `assert!(matches!(ReplayLedger::open(&root), Err(ReplayError::PersistenceUnavailable)))` |
| 9 | Filename/content identity agreement | PROVEN | `f3e2a_claim_filename_content_disagreement_fails_closed` (replay_windows.rs) NEW | Claim renamed to different-hex filename → `assert!(matches!(ReplayLedger::open(&root), Err(ReplayError::PersistenceUnavailable)))` — `from_canonical_bytes` checks `record.logical_key_digest == expected_logical_key.as_digest()` |
| 10 | Collision/non-replacement at Claim boundary | PROVEN | `native_publication_survives_reopen_and_never_replaces` (replay_windows.rs:1868) | Second publish → `Err(PersistenceUnavailable)`; original bytes preserved; `.tmp` debris retained |
| 11 | Unexpected temporary/debris handling | PROVEN | `ledger_29_unexpected_ledger_entry_fails_closed` (replay_windows.rs:2722) | `.tmp` file in claims dir → `assert!(matches!(ReplayLedger::open(&root), Err(ReplayError::PersistenceUnavailable)))` |
| 12 | Unsafe-path protection at Claim boundary | PROVEN | `relative_root_is_rejected_before_win32` (replay_windows.rs:1777), `unc_roots_are_rejected_before_win32` (replay_windows.rs:1780), `traversal_ads_and_separator_final_filenames_are_rejected` (replay_windows.rs:1771), `validated_child_retains_complete_independent_handle_chain` (replay_windows.rs:1833) | Relative/UNC/reparse/device names rejected; `ValidatedLeafName` constrains Claim filenames to hex-only; handle chain prevents TOCTOU ancestor substitution |
| 13 | Exact bytes/readback | PROVEN | `claim_round_trip_is_exact_canonical_and_redacted` (replay.rs:613), `ledger_30_restart_never_generates_new_uuid_for_existing_tuple` (replay_windows.rs:2742) | `assert_eq!(recovered, claim)` unit-level; `assert_eq!(claim_bytes, claim_before)` after 2 restarts |

### Remaining UNVERIFIED

- Power-loss durability: UNVERIFIED (F3b) — never upgrade
- Directory-entry durability: UNVERIFIED (F3b) — never upgrade
- Atomic visibility during rename: UNVERIFIED (F3b) — never upgrade
- Parent-directory flush in production: DISPROVEN (F3b)

### F3e2a findings

- One characterization test added: `f3e2a_claim_filename_content_disagreement_fails_closed` — publishes a valid claim, renames the file to a different logical-key hex digest, verifies ledger open returns `PersistenceUnavailable`. Filename/content identity agreement moved from UNVERIFIED to PROVEN.
- All 12 other dimensions are PROVEN by existing tests with hard assertions.
- No production code changed.
- No defect found in the Replay Claim slice.
- Replay Generations are explicitly deferred to F3e2b/F3e3.
- 122 Replay tests pass (121 existing + 1 new F3e2a characterization test).

---

## F3e2b Evidence — Replay Generations & Recovery evidence harvest

Date: 2026-08-08
Baseline: `477e2b901c0dfec55f4df6f9dca79a66e9294e0a` (accepted F3e2a-R1)
Branch: `foundation/f3e2b-replay-generations-evidence`
Evidence sources:
- `tethers-0.1/host-rust/src/replay.rs` inline tests (Generation model, validate_chain)
- `tethers-0.1/host-rust/src/replay_windows.rs` inline tests (ledger 12–30, all_bounded, recovered_*, populated_reopen)

F3e2b was an evidence harvest limited to the Replay Generation slice (Generation 0/1/2, publication, readback, chain validation, restart reconstruction, recovered-admission mutation blocking). Replay Claim identity was accepted in F3e2a and was not re-audited.

Zero characterization tests added. All 14 dimensions are PROVEN by existing tests with hard assertions. No production code changed and no defect was found.

### Generation evidence summary

| # | Property | Status | Test | Exact Hard Assertion |
|---|---|---|---|---|
| 1 | Canonical Generation representation | PROVEN | `generation_three_is_not_representable_or_parseable` (replay.rs:657) negative; reconstruction tests (ledger 21–26) positive | G3 bytes → `assert!(Generation::from_canonical_bytes(&bytes).is_err())`; reconstruction: publish G0→G1→G2, reopen, state matches expected — publish→reopen→parse round-trip |
| 2 | Generation 0 publication | PROVEN | `ledger_12_valid_generation_zero_publication` (replay_windows.rs:2287) | `assert_eq!(admission.state(), ReplayState::IntentRecorded)`; `assert_eq!(directory_names(&path), vec!["g0000000000000000.json"])` |
| 3 | G0 → G1 transition | PROVEN | `ledger_13_valid_generation_zero_to_one_transition` (replay_windows.rs:2302) | `assert_eq!(admission.state(), ReplayState::InvocationArmed)` |
| 4 | G2 terminal states (all 3) | PROVEN | `ledger_14_each_valid_generation_two_terminal_state` (replay_windows.rs:2316) | Loop over `[Succeeded, Failed, Uncertain]`: `assert_eq!(admission.state(), state)` for each |
| 5 | Missing-generation rejection (G1 without G0, G2 without G1) | PROVEN | `ledger_15_generation_one_without_zero_is_rejected` (replay_windows.rs:2340), `ledger_16_generation_two_without_one_is_rejected` (replay_windows.rs:2363) | G1-only → `assert!(ReplayLedger::open(&root).is_err())`; G0+G2 skip G1 → `assert!(ReplayLedger::open(&root).is_err())` |
| 6 | Illegal state-at-generation | PROVEN | `ledger_17_illegal_state_transition_is_rejected` (replay_windows.rs:2398); `chain_cannot_skip_armed` (replay.rs:644) | G0 with "succeeded" → open err; G0→G2 direct → `assert!(validate_chain(...).is_err())` |
| 7 | Predecessor-digest linkage | PROVEN | `ledger_18_predecessor_mismatch_is_rejected` (replay_windows.rs:2428) | G1 with tampered predecessor_digest → `assert!(ReplayLedger::open(&root).is_err())` |
| 8 | Generation immutability / non-replacement | PROVEN | `ledger_19_generation_collision_never_replaces_bytes` (replay_windows.rs:2463) | Pre-existing conflicting file → `publish_armed().is_err()`; `assert_eq!(std::fs::read(collision).unwrap(), b"different-immutable-bytes")` |
| 9 | Generation upper bound (model, filename, persistence) | PROVEN | `generation_three_is_not_representable_or_parseable` (replay.rs:657), `generation_filename(3)` (replay_windows.rs:1213), `ledger_20_generation_three_is_rejected` (replay_windows.rs:2493) | `from_canonical_bytes` rejects generation=3; filename rejects number>2; second terminal publish after G2 → err |
| 10 | Restart reconstruction (6 state variants) | PROVEN | ledger_21–26 (replay_windows.rs:2516–2614) | Claim-only → `ClaimedNoState`; G0 → `IntentRecorded`; G1 → `InvocationArmed`; G2 → `Succeeded`/`Failed`/`Uncertain` each asserted |
| 11 | Recovered admissions cannot advance history | PROVEN | `recovered_claim_g0_and_g1_admissions_cannot_advance_or_mutate` (replay_windows.rs:2616); `recovered_terminal_admission_cannot_publish_or_mutate` (replay_windows.rs:2653) | All mutation attempts → `Err(PersistenceUnavailable)`; `assert_eq!(tree_snapshot(&root), before)` — filesystem unchanged |
| 12 | Malformed/corrupt Generation-chain fail-closed | PROVEN | `ledger_28_malformed_chain_fails_closed`, `ledger_27_orphan_chain_fails_whole_ledger_closed`, ledger_17, ledger_18 (replay_windows.rs) | Malformed JSON → open err; orphan chain (no claim) → open err; tampered state/predecessor → open err |
| 13 | Filename/content agreement for Generations | PROVEN | `f3e2b_generation_filename_content_disagreement_fails_closed` (replay_windows.rs) NEW | Publish G0+G1, swap filenames (G0↔G1), reopen → `assert!(ReplayLedger::open(&root).is_err())` |
| 14 | Exact bytes / close-reopen | PROVEN | `f3e2b_generation_exact_bytes_survive_close_and_reopen` (replay_windows.rs) NEW; `ledger_30_restart_never_generates_new_uuid_for_existing_tuple` (replay_windows.rs:2741); `ledger_populated_valid_subtrees_reopen_without_reprovisioning` (replay_windows.rs:2769) | G0/G1/G2 bytes read before and after close/reopen, `assert_eq!(g0_after, g0_before)` etc. for all 3 |

### Remaining UNVERIFIED

- Power-loss durability: UNVERIFIED (F3b) — never upgrade
- Directory-entry durability: UNVERIFIED (F3b) — never upgrade
- Atomic visibility during rename: UNVERIFIED (F3b) — never upgrade
- Parent-directory flush in production: DISPROVEN (F3b)

### F3e2b findings

- All 14 Generation/reconstruction dimensions are PROVEN by existing tests with hard assertions.
- Two characterization tests added (F3e2b-R1):
  - `f3e2b_generation_exact_bytes_survive_close_and_reopen` — publishes G0/G1/G2, reads file bytes, drops/reopens ledger, asserts all three generation files have identical bytes.
  - `f3e2b_generation_filename_content_disagreement_fails_closed` — publishes G0+G1, swaps filenames (G0↔G1), asserts ledger open returns `PersistenceUnavailable`.
- No production code changed.
- No defect found in the Replay Generation slice.
- Replay Claim evidence from F3e2a preserved (not re-audited).
- 124 Replay tests pass (122 existing + 2 new F3e2b-R1 characterization tests).

---

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

---

## F3-GATE — Combined Persistence Contract Reconciliation

Date: 2026-08-08
Baseline: `ab58c83ba44680f3003db333f1e1ffd091aa5b3f` (accepted F3e2b-R1)
Branch: `foundation/f3-persistence-gate`

This is the final combined review required to close Foundation Pass F3. It answers one question: "Taken together, does the accepted F3 evidence describe one internally consistent persistence contract?"

### Verdict: READY FOR INDEPENDENT GATE REVIEW

The accepted F3 evidence corpus (F3a through F3e2b) describes one internally consistent persistence contract. Twelve audit areas were examined; zero material contradictions were found.

### Store vocabulary — consistent

14 classified persistence stores:
- 9 immutable atomic records (Candidate Registry, Publisher Trust Store, Developer Approval Store, Launch Profile Evidence, Conformance Evidence, Installation Approval, Installed Plug Registry, Enablement Records, Replay Claim)
- 1 replaceable current-state record (Installation Publication Intent)
- 1 append-only causal log (Trail)
- 3 multi-step intent/recovery journals (Replay Generations, Installation Recovery Staging, Local Anchor Admission Store)

Plus 1 filesystem coordination artifact (Installation Execution Lock) and 6 in-memory entries. No later F3 section reclassifies any store. Count is verified consistent across all F3 sections.

### Key contract properties — consistent

| Property | Final F3 status | Scope |
|---|---|---|
| Atomic visibility during rename | UNVERIFIED (F3b) | All stores using rename |
| File data survives sudden power loss | UNVERIFIED (F3b) | All stores |
| Directory entry survives power loss | UNVERIFIED (F3b) | All stores |
| Parent-directory flush in production | DISPROVEN (F3b) | All stores |
| Exact bytes survive ordinary close/reopen | PROVEN where directly tested (F3b/F3e2a/F3e2b); UNVERIFIED otherwise | Varies by store |
| Root reparse-point defence | PROVEN (StoreRoot/Replay); DISPROVEN (Local Anchor); UNVERIFIED (Candidate Registry) | Varies by store |
| Malformed input fail-closed | PROVEN (all tested stores) | Bounded to tested stores |
| Digest corruption rejected | PROVEN (stores with digests); N/A (Trail) | Bounded to tested stores |
| Orphan/partial state rejected | PROVEN (Trail, Replay, Candidate, Installation, Local Anchor) | Bounded to tested stores |
| Recovery correct state reconstruction | PROVEN (Replay G0/G1/G2, Installation Recovery, Local Anchor) | Bounded to tested stores |

### Contradiction ledger

| # | Area | Verdict |
|---|---|---|
| 1 | Store vocabulary — count and classification | No contradiction |
| 2 | Atomic visibility — never claimed PROVEN | No contradiction |
| 3 | File data power-loss durability — never claimed PROVEN | No contradiction |
| 4 | Directory-entry durability — never claimed PROVEN | No contradiction |
| 5 | Parent-directory flush DISPROVEN — consistently recorded | No contradiction |
| 6 | Ordinary close/reopen — not conflated with power-loss | No contradiction |
| 7 | Trail semantics — remains append-only causal log | No contradiction |
| 8 | Replay semantics — Claim/Generations retain distinct meaning | No contradiction |
| 9 | Installation semantics — remains installation-specific | No contradiction |
| 10 | Corruption/fail-closed — bounded to tested stores | No contradiction |
| 11 | Path safety — attributed to correct protection layer | No contradiction |
| 12 | Combined contract alignment — F3 subpackages have independent consistent evidence | No contradiction |

### Changes in this gate

- F3 combined-gate section added to PERSISTENCE_INVENTORY.md (this section).
- CURRENT_CLINE_TASK.md updated with F3-GATE task and findings.
- F3-GATE worker note created.
- No production code changed. No tests added. No earlier worker notes edited.
