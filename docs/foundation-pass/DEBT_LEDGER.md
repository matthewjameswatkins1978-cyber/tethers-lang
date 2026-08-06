# F1 Foundation Debt Ledger

Date: 2026-08-06
Baseline: `24428139807cac0adeb0b62264547e61ca809d16` (`origin/main`)
All items cite live evidence. No repair attempted.

---

## Confirmed Defects

### D1: Flaky M3 Windows Handle Allow-List Test
- **Classification:** Confirmed defect (nondeterministic test behaviour)
- **Evidence:** `tests/m3_lifecycle.rs:884` — `m3_windows_handle_allow_list_excludes_unrelated_inheritable_handle` failed on first `cargo test` run at baseline, passed on second and subsequent runs. Assertion: `left: Failed, right: Passed`.
- **Disposition:** F2
- **Notes:** The confirmed defect is nondeterministic test behaviour. The production root cause (likely timing-dependent Windows handle enumeration) remains unconfirmed. The test asserts handle allow-list exclusion but the test fixture's child process may not have closed handles by the time enumeration runs.

---

## Contract Ambiguity

### A1: Directory Durability Not Explicitly Tested
- **Classification:** Contract ambiguity
- **Evidence:** `PERSISTENCE_INVENTORY.md` — no store has confirmed directory-entry durability. The Replay Ledger calls `FlushFileBuffers` after rename on the renamed file handle and verifies byte equality by reopening and comparing, but the parent directory entry is not explicitly flushed. All 7 `StoreRoot`-backed stores, the Trail, and the Admission Store lack any directory-level durability even at the file-handle level. F3b (Windows primitive evidence) is tasked with establishing the supported Windows guarantee for every store.
- **Disposition:** F3b
- **Notes:** NTFS metadata durability requires explicit directory handle flush after rename when `FILE_FLAG_WRITE_THROUGH` is not used on the parent. Replay's post-rename `FlushFileBuffers` flushes the renamed file handle, not the parent directory. Current tests may pass but the contract is ambiguous for all stores.

---

## Maintainability Debt

### M1: Dead Code Accumulation in `application.rs`
- **Classification:** Maintainability debt
- **Evidence:** 10 unused items with `#[warn(dead_code)]`: `PROVISION_USAGE`, `parse_provision_args`, `run_event_admission_probe`, `run_event_admission_trail_probe`, `HumanApprovalDecision`, `record_human_approval_decision`, `submit_local_root_anchor`, `short_event_digest`, `process_local_notification`, `resume_and_execute_exact_approval`, `authorise_and_execute`. All in `src/application.rs`.
- **Disposition:** F2 (remove dead code)
- **Notes:** These appear to be earlier architectural probes or abandoned features.

### M2: 137 Clippy Warnings Across Codebase
- **Classification:** Maintainability debt
- **Evidence:** `WARNING_INVENTORY.md` — 137 distinct Clippy warning occurrences in lib, tests, and bins.
- **Disposition:** F8 (warnings cleanup)
- **Notes:** Categories include: dead code, unused imports/variables, `too_many_arguments`, `items_after_test_module`, `needless_borrow`, `cmp_owned`, `useless_format`, `unnecessary_map_or`, `type_complexity`.

### M3: Monolithic `application.rs` (8,260 lines)
- **Classification:** Maintainability debt
- **Evidence:** `src/application.rs` — 8,260 lines, the largest file by far. Contains CLI dispatch, runtime orchestration, anchor writing, test support, and dead code.
- **Disposition:** F5 (structural extraction)
- **Notes:** The file mixes concerns: command routing, runtime parts construction, test infrastructure, and abandoned probe functions. Structural extraction would reduce ownership hazards.

### M4: No OCaml `.mli` Interfaces
- **Classification:** Maintainability debt
- **Evidence:** All 6 OCaml `.ml` files in `tethers-0.1/engine-ocaml/bin/` have zero `.mli` interface files. Every module exposes its full internal implementation. `tether_parser.ml` exposes helper functions (`trim`, `starts_with`, `drop_prefix`) that should be private.
- **Disposition:** F5 (structural extraction) or defer to OCaml-specific task
- **Notes:** No abstraction boundaries exist. Internal parser types, evaluator internals, and MCP server state are publicly accessible. No direct runtime harm demonstrated — the modules still compile and function correctly; the debt is structural maintainability.

### M5: `SupervisedChild.max_line_bytes` Declared but Never Read
- **Classification:** Maintainability debt
- **Evidence:** `src/child_process.rs:214` — field `max_line_bytes: usize` on `SupervisedChild` struct, flagged `#[warn(dead_code)]`.
- **Disposition:** F2 (remove dead code) or complete bounding implementation
- **Notes:** No direct runtime harm demonstrated. The field is declared but never consumed. Suggests incomplete stderr bounding implementation or abandoned feature. Related to the live-stderr candidate (see F2 candidate below).

### M6: `FailingResultAnchorWriter` Never Constructed
- **Classification:** Maintainability debt
- **Evidence:** `src/application.rs:1870` — struct declared but never instantiated. `#[warn(dead_code)]`.
- **Disposition:** F2 (remove dead code) or defer
- **Notes:** No direct runtime harm demonstrated. Test infrastructure stub that was never wired. May indicate incomplete error-path testing.

### M7: No OCaml-Native Tests
- **Classification:** Maintainability debt
- **Evidence:** `tethers-0.1/engine-ocaml/` — no `.ml` test files, no test directory. 15 PowerShell integration scripts test the engine via subprocess, but no OCaml-native unit tests exist for the parser, evaluator, or protocol modules.
- **Disposition:** F7 (test consolidation) or defer
- **Notes:** All engine testing goes through the Rust host/CLI boundary. No direct OCaml-level test coverage. This is a test-coverage gap, not a correctness defect.

### M8: Test Modules Inside Production Files
- **Classification:** Maintainability debt
- **Evidence:** 13 files with `_tests.rs` suffix in `src/` (e.g., `src/current_trust_tests.rs`, `src/installation_recovery_plan_tests.rs`). These are `#[cfg(test)]` modules in separate files, blurring the production/test boundary. Some test-only modules are declared `pub(crate)` in `lib.rs`, making test infrastructure visible to the crate.
- **Disposition:** F7 (test consolidation)
- **Notes:** No direct runtime harm. The test infrastructure is properly `#[cfg(test)]` gated at compilation. The debt is structural clarity and crate-visibility hygiene.

### M9: Test Infrastructure `pub(crate)` Visibility
- **Classification:** Maintainability debt
- **Evidence:** `src/lib.rs` declares `pub(crate)` modules for test-only files. `src/current_trust_tests.rs`, `src/installation_recovery_*.rs` test modules are accessible crate-wide instead of being confined to `#[cfg(test)]`.
- **Disposition:** F7 (test consolidation)
- **Notes:** Some test helpers are unconditionally `pub(crate)`, widening the visible API surface.

---

## Performance Hypotheses

### P1: `application.rs` Compile Time
- **Classification:** Performance hypothesis
- **Evidence:** 8,260-line monolithic file with many type parameters and generics. Not measured.
- **Disposition:** F6 (if measurements justify)
- **Notes:** Extraction (F5) may improve incremental compile times. This hypothesis needs measurement before action.

### P2: `result_large_err` at 160+ bytes
- **Classification:** Performance hypothesis
- **Evidence:** `src/run_command.rs:998` — Clippy `result_large_err` reports `Err` variant is at least 160 bytes.
- **Disposition:** F6 (if measurements justify)
- **Notes:** Boxing the error variant would reduce stack copy cost on the hot path. Measure first.

---

## Documentation Debt

### DOC1: No OCaml API Documentation
- **Classification:** Documentation debt
- **Evidence:** OCaml modules have no documentation comments, no `.mli` signatures, no README in `engine-ocaml/`.
- **Disposition:** F9
- **Notes:** Engine protocol (JSON line protocol) is partially documented in `SPEC.md` but internal module contracts are undocumented.

### DOC2: Test Coverage Report Missing
- **Classification:** Documentation debt
- **Evidence:** No coverage tooling configured. No `tarpaulin`, `grcov`, or `llvm-cov` setup.
- **Disposition:** Defer
- **Notes:** Coverage data would help prioritise F2-F8 work but is not a blocking requirement.

### DOC3: No Architecture Decision Record for Live-stderr
- **Classification:** Documentation debt
- **Evidence:** `docs/DECISIONS.md` — no decision record for the stderr capture approach in `child_process.rs`. The Foundation Pass plan references a "live-stderr-tail issue" candidate.
- **Disposition:** F2 (if confirmed) or F9
- **Notes:** The approach to child process stderr capture (bounded, line-buffered, distinction of timeout/EOF/I/O/exit/kill/join) needs explicit documentation.

---

## F2 Candidate (Not Classified)

### Candidate: Live-stderr Output Not Captured
- **Classification:** F2 candidate only — not a confirmed defect
- **Evidence:** The Foundation Pass programme plan references a live-stderr issue. F1 has not captured direct reproduction evidence. The `max_line_bytes` field (M5 above) and the stderr bounding in `child_process.rs` (line count, tail limit, timeout/EOF/I/O/exit/kill/join handling) are related infrastructure, but F1 cannot confirm a runtime defect without a test that fails because stderr is missing or incorrectly routed.
- **Disposition:** F2 investigation
- **Notes:** F2 should attempt reproduction before classification. If confirmed, classify as Confirmed defect with the reproduction test as evidence. If not reproducible, reclassify as Documentation debt (DOC3) and close.

---

## Summary

| Class | Count | Items | Assigned |
|---|---|---|---|
| Confirmed defect | 1 | D1 (flaky test) | F2 |
| Contract ambiguity | 1 | A1 (directory durability) | F3b |
| Maintainability debt | 9 | M1-M9 | F2 (M1, M5, M6), F5 (M3, M4), F7 (M7, M8, M9), F8 (M2) |
| Performance hypothesis | 2 | P1, P2 | F6 |
| Documentation debt | 3 | DOC1-DOC3 | F9, defer, F2/F9 |
| F2 candidate | 1 | Live-stderr | F2 (investigate) |
| **Total** | **17** | | |

## Changes From Previous Ledger

- **D2 (missing .mli) + M4 (same)**: Consolidated into single M4 as Maintainability debt. No direct runtime harm demonstrated.
- **D3 (dead max_line_bytes)**: Reclassified from Confirmed defect to Maintainability debt (M5). Dead code with no proven runtime harm.
- **D4 (dead FailingResultAnchorWriter)**: Reclassified from Confirmed defect to Maintainability debt (M6). Never-constructed struct with no proven runtime harm.
- **A2 (no OCaml tests)**: Reclassified from Contract ambiguity to Maintainability debt (M7). No contract is violated; the gap is test coverage.
- **A3 (test modules in src/)**: Reclassified from Contract ambiguity to Maintainability debt (M8). No contract is violated; the debt is structural clarity.
- **D1 (flaky M3 test)**: Retained as Confirmed defect but explicitly qualified: the confirmed defect is nondeterministic test behaviour; production root cause remains unconfirmed.
- **Live-stderr**: Explicitly marked as F2 candidate, not Confirmed defect. No direct reproduction evidence captured in F1.

No items repaired. All assigned to F-packages, deferred, or marked for F2 investigation.
