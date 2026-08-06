# F1 Persistence Inventory

Baseline: `24428139807cac0adeb0b62264547e61ca809d16` (`origin/main`)
Source: `tethers-0.1/host-rust/src/`

## Durable Classes

### 1. Immutable Atomic Record

Once written via write-then-rename, never modified. Corrupt or missing record is invalid state.

| Store | Module | Write Primitive | Atomic Visibility | File Durability | Dir-Entry Durability | Recovery Reader | Corruption Classification | Unsafe-Path Protection | Tests |
|---|---|---|---|---|---|---|---|---|---|
| Candidate Registry | `candidate.rs` | `create()` writes `.{id}.tmp`, `fs::rename`; `sync_all()` on new file | Atomic rename (NTFS) | `sync_all()` on file write | UNVERIFIED (F3b) | `load_all()` with `.tmp` rejection, digest revalidation, quarantine immutability check | `.tmp` = torn write; digest mismatch = corrupt; path/filename divergence = tamper; payload mutation = tamper | `verify_existing_chain()` checks all ancestors for reparse points | `torn_temporary_record_fails_closed`, `filename_disagreement_and_duplicate_identity_evidence_fail_closed`, `preexisting_staging_target_and_escape_destination_fail_closed_without_write` |
| Publisher Trust Store | `trust.rs` via `m3_store.rs` | `StoreRoot::create_json()` writes `.{id}.tmp`, `sync_all()`, `fs::rename` | Atomic rename | `sync_all()` | UNVERIFIED (F3b) | `current()` rejects `.tmp`, validates record digest, predecessor chain continuity | `.tmp` = torn write; digest mismatch = corrupt; predecessor gap = chain break | `verify_chain()` on StoreRoot checks every ancestor for reparse | `trust_transitions_restart_and_revocation_fail_closed` |
| Developer Approval Store | `trust.rs` via `m3_store.rs` | `StoreRoot::create_json()` | Atomic rename | `sync_all()` | UNVERIFIED (F3b) | `find()` rejects `.tmp`, validates record digest | Same as Publisher Trust | Same as Publisher Trust | `developer_approval_is_exact_digest_only` |
| Launch Profile Evidence | `launch_profile.rs` via `m3_store.rs` | `StoreRoot::create_json()` | Atomic rename | `sync_all()` | UNVERIFIED (F3b) | `load_all()` rejects `.tmp`, validates `profile_evidence_digest`, checks filename is digest | `.tmp` = torn write; digest mismatch = corrupt; filename divergence = tamper | StoreRoot `verify_chain()` | Inline `mod tests` |
| Conformance Evidence | `conformance.rs` via `m3_store.rs` | `StoreRoot::create_json()` | Atomic rename | `sync_all()` | UNVERIFIED (F3b) | `load_all()` rejects `.tmp`, validates evidence digest, checks `evidence_id` matches file stem | `.tmp` = torn write; digest mismatch = corrupt; identity mismatch = tamper | StoreRoot `verify_chain()` | Inline `mod tests` |
| Installation Approval | `installed.rs` via `m3_store.rs` | `StoreRoot::create_json()` | Atomic rename | `sync_all()` | UNVERIFIED (F3b) | `load_all()` rejects `.tmp`, validates record digest, checks `approval_id` matches file | `.tmp` = torn write; digest mismatch = corrupt; identity mismatch = tamper | StoreRoot `verify_chain()` | Inline `mod tests` |
| Installed Plug Registry | `installed.rs` | Staging dir `.{id}.tmp` renamed to final; per-file `sync_all()` + StoreRoot record | Atomic rename of staging dir | Per-file `sync_all()` | UNVERIFIED (F3b) | `load_all()` rejects `.tmp` records, validates payload digests, re-reads file contents | `.tmp` = torn write; digest mismatch = corrupt; missing payload = tamper; escaped path = tamper | `verify_chain()` + `reject_reparse()` + `recovery_expected_path()` | `audit_installation_recovery_destinations` |
| Enablement Records | `enablement.rs` via `m3_store.rs` | `StoreRoot::create_json()` | Atomic rename | `sync_all()` | UNVERIFIED (F3b) | `load_all()` rejects `.tmp`, validates record digest, checks predecessor chain per installed identity | `.tmp` = torn write; digest mismatch = corrupt; predecessor gap = chain break | StoreRoot `verify_chain()` | `enablement_is_explicit_and_disable_removes_availability` |
| Replay Claim (identity) | `replay_windows.rs` | `publish_new_canonical_file_with_temporary_stem()`: `CreateFileW(CREATE_NEW | FILE_FLAG_WRITE_THROUGH)`, `WriteFile`, `FlushFileBuffers` before rename, `SetFileInformationByHandle` rename, `FlushFileBuffers` after rename, reopen and re-read verify | Atomic rename + post-rename verify | `FlushFileBuffers` before rename (file data confirmed); `FlushFileBuffers` after rename (renamed file handle only) | UNVERIFIED (F3b) | `validate_whole_ledger()` validates claim digests, checks chain integrity | Post-rename content re-read and compare to original bytes; digest mismatch = corrupt; orphan chain = fail closed | Handle-based `ValidatedHostRoot` with NTFS validation, `ValidatedLeafName`, retained open handles | `wire_a_deterministic_pair_of_claims_and_walk_them` (ledger tests 01-30) |

### 2. Replaceable Current-State Record

May be overwritten with a newer value. Previous state is discarded.

| Store | Module | Write Primitive | Atomic Visibility | File Durability | Dir-Entry Durability | Recovery Reader | Corruption Classification | Unsafe-Path Protection | Tests |
|---|---|---|---|---|---|---|---|---|---|
| Installation Publication Intent | `installation_publication_intent.rs` via `m3_store.rs` | `StoreRoot::create_json("current", intent)` — singleton `current.json` overwritten on each intent | Atomic rename | `sync_all()` | UNVERIFIED (F3b) | `load()` expects exactly 0 or 1 entry; validates intent digest | `.tmp` = torn write; digest mismatch = corrupt; more than 1 entry = corrupt | StoreRoot `verify_chain()` | `installation_publication_intent_tests.rs` |

### 3. Append-Only Causal Log

New entries appended; existing entries never modified.

| Store | Module | Write Primitive | Atomic Visibility | File Durability | Dir-Entry Durability | Recovery Reader | Corruption Classification | Unsafe-Path Protection | Tests |
|---|---|---|---|---|---|---|---|---|---|
| Trail (FileTrail) | `dispatch.rs` | JSONL line append: `writeln!`, `flush()`, `sync_data()` per line | Per-line `sync_data()` — no rename | `sync_data()` after each line | UNVERIFIED (F3b) | Manual JSONL parse via `trail_command.rs`; no integrity footer or per-line digest | No per-line digest or checksum; no re-read verification | NONE — `FileTrail::open()` accepts any path without chain verification | `file_trail_writes_durable_jsonl_intent` |

### 4. Multi-Step Intent/Recovery Journal

Records intent through recovery steps. Intermediate states are valid and recoverable.

| Store | Module | Write Primitive | Atomic Visibility | File Durability | Dir-Entry Durability | Recovery Reader | Corruption Classification | Unsafe-Path Protection | Tests |
|---|---|---|---|---|---|---|---|---|---|
| Replay Generations (0-2) | `replay_windows.rs` | `publish_new_canonical_file_with_temporary_stem()` (same as identity claim above) | Atomic rename + post-rename verify | `FlushFileBuffers` before rename (file data confirmed); `FlushFileBuffers` after rename (renamed file handle only) | UNVERIFIED (F3b) | `validate_whole_ledger()` walks generation chain; predecessor mismatch fails closed; orphan chains fail whole ledger closed | Generation 3+ rejected on creation; malformed claim digest fails closed; wrong predecessor breaks chain | `ValidatedHostRoot` + handle-based TOCTOU prevention | Ledger tests 01-30 in `replay_windows.rs` |
| Installation Recovery Staging | `installed.rs` | Staging dir with `install_disabled_with_authority()`: creates `.staging-{id}`, copies files, `fs::rename` to destination, then writes record | Atomic rename of staging dir | Per-file `sync_all()` + record `sync_all()` | UNVERIFIED (F3b) | `observe_installation_recovery()` snapshots staging/destination/record state; `audit_installation_recovery_destinations()` cross-validates disk against records | Recovery staging detects missing/present destinations, mismatched file sets, reparse point intrusion | `verify_chain()` + `reject_reparse()` | `audit_installation_recovery_destinations` |
| Installation Execution Lock | `installation_execution.rs` | `CreateFileW` with `share_mode(0)` (exclusive) + `SetHandleInformation(HANDLE_FLAG_INHERIT, 0)` | Exclusive handle — no concurrent access | N/A (lock file, no data written) | N/A | Lock automatically released on process exit (OS). Non-empty lock anchor = stale crash evidence | Non-empty lock anchor rejected as stale | `verify_chain()` + `reject_reparse()` + post-acquisition reparse re-check | `installation_execution_tests.rs` |
| Local Anchor Admission Store | `local_anchor.rs` | `atomic_create()`: writes `{name}.tmp`, `sync_all()`, `fs::rename` | Atomic rename | `sync_all()` | UNVERIFIED (F3b) | `AdmissionStore::open()` sorts by filename, validates `record_digest`, detects duplicates by SHA-256 name collision | `.tmp` = torn write; digest mismatch = corrupt; duplicate detection via SHA-256 name; schema version mismatch = corrupt | Partial: `safe_filename()` (SHA-256 hash) prevents traversal; source path canonicalized and checked but no `verify_chain()` on store root | `same_id_same_digest_is_duplicate_after_restart`, `corrupted_record_refuses_restart` |

---

## Non-Durable Appendix

In-memory stores with no filesystem persistence. Restart expiry is deliberate semantics.

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

The following guarantees could not be independently verified by F1 and are routed to F3b (Windows primitive evidence):

| Store(s) | Unverified Claim | Risk |
|---|---|---|
| All `StoreRoot`-backed stores (7 stores) | Directory-entry durability after rename | NTFS metadata flush to directory requires explicit `FlushFileBuffers` on the parent directory handle or `FILE_FLAG_WRITE_THROUGH` on CreateFile. StoreRoot does neither on the parent. Rename alone does not guarantee directory metadata durability. |
| Trail (FileTrail) | Line-level recovery integrity | No per-line digest, checksum, or integrity footer. A crash mid-line could leave a partial last line. No path safety verification. |
| Local Anchor Admission Store | Store root reparse safety | No `verify_chain()` or `reject_reparse()` on the store root directory. SHA-256 filenames prevent traversal in filenames but not on the root itself. |
| Replay Ledger | Directory-entry durability after rename | The second `FlushFileBuffers` call occurs on the renamed file handle, not the parent directory. File data is flushed before rename; the renamed file handle is flushed after rename; final bytes are reopened and compared. Parent-directory durability remains unverified. |
| Replay Ledger | `FILE_FLAG_WRITE_THROUGH` adequacy on any NTFS volume | `FILE_FLAG_WRITE_THROUGH` behaviour varies by volume type and configuration. F3b should verify on the exact target volume class. |

---

## Key Differences From Previous Inventory

- **Trail is NOT write-then-rename**: It appends JSONL lines and syncs in place. Previous version incorrectly grouped it under Immutable Atomic Record.
- **No store has confirmed directory-entry durability**: The Replay Ledger calls `FlushFileBuffers` after rename on the renamed file handle, and final bytes are reopened and compared, but parent-directory durability remains unverified (F3b). All `StoreRoot`-backed stores and the Trail lack this flush entirely. All directory-entry durability claims are routed to F3b.
- **`m3_store.rs` (StoreRoot) is the common persistence layer** for 7 stores, not an independent store. It provides a consistent write-then-rename-with-sync pattern.
- **No store uses the "Replaceable Current-State Record" pattern for write-then-rename overwrites**: Only Installation Publication Intent fits this class. Previous version incorrectly classified Candidate Registry as replaceable.
- **Installation Recovery Plan is NOT a store**: It is a read-only planner that coordinates through the intent store and installed registry. Previous version classified it as a store.
