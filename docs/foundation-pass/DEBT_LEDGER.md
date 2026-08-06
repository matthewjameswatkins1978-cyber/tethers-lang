# F1 Foundation Debt Ledger

Date: 2026-08-06
Baseline: `24428139807cac0adeb0b62264547e61ca809d16` (`origin/main`)
All items cite live evidence. No repair attempted.

---

## Confirmed Defects

### D1: Flaky M3 Windows Handle Allow-List Test
- **Classification:** Confirmed defect
- **Evidence:** `tests/m3_lifecycle.rs:884` — `m3_windows_handle_allow_list_excludes_unrelated_inheritable_handle` failed on first `cargo test` run, passed on second. Assertion: `left: Failed, right: Passed`.
- **Disposition:** F2
- **Notes:** Likely a timing-dependent Windows handle enumeration issue. Test asserts handle allow-list exclusion but the test fixture's child process may not have closed handles by the time enumeration runs.

### D2: Missing `.mli` Interfaces in OCaml Engine
- **Classification:** Confirmed defect (maintainability/architecture)
- **Evidence:** All 6 OCaml `.ml` files have zero `.mli` interface files. Every module exposes its full internal implementation.
- **Disposition:** F5 (structural extraction) or defer to OCaml-specific task
- **Notes:** No abstraction boundaries exist. Internal parser types, evaluator internals, and MCP server state are all publicly accessible.

### D3: `SupervisedChild.max_line_bytes` Declared but Never Read
- **Classification:** Confirmed defect
- **Evidence:** `src/child_process.rs:214` — field `max_line_bytes: usize` on `SupervisedChild` struct, `#[warn(dead_code)]`.
- **Disposition:** F2
- **Notes:** Suggests incomplete stderr bounding implementation or abandoned feature. Related to the live-stderr candidate (F1 does not classify that without reproduction evidence).

### D4: `FailingResultAnchorWriter` Never Constructed
- **Classification:** Confirmed defect
- **Evidence:** `src/application.rs:1870` — struct declared but never constructed. `#[warn(dead_code)]`.
- **Disposition:** F2 (remove dead code) or defer
- **Notes:** Test infrastructure stub that was never wired. May indicate incomplete error-path testing.

---

## Contract Ambiguity

### A1: Directory Durability Not Explicitly Tested
- **Classification:** Contract ambiguity
- **Evidence:** `PERSISTENCE_INVENTORY.md` — all file stores use write-then-rename. Tests close files but none test `FlushFileBuffers` on parent directories. F3b (Windows primitive evidence) is tasked with this.
- **Disposition:** F3b
- **Notes:** NTFS metadata durability requires explicit directory handle flush. Current tests may pass but the contract is ambiguous.

### A2: OCaml Engine Has No Direct Tests
- **Classification:** Contract ambiguity
- **Evidence:** `tethers-0.1/engine-ocaml/` — no `.ml` test files, no test directory. 15 PowerShell integration scripts test the engine via subprocess, but no OCaml-native unit tests exist.
- **Disposition:** F7 (test consolidation) or defer
- **Notes:** Parser, evaluator, and protocol modules have no direct OCaml test coverage. All testing goes through the Rust host/CLI boundary.

### A3: Test Modules Inside Production Files
- **Classification:** Contract ambiguity
- **Evidence:** 13 files with `_tests.rs` suffix in `src/` (e.g., `src/current_trust_tests.rs`, `src/installation_recovery_plan_tests.rs`). These are `#[cfg(test)]` modules in separate files, blurring the production/test boundary.
- **Disposition:** F7
- **Notes:** Some test-only modules are declared `pub(crate)` in `lib.rs`, making test infrastructure visible to the crate. This is a test-access boundary concern.

---

## Maintainability Debt

### M1: Dead Code Accumulation in `application.rs`
- **Classification:** Maintainability debt
- **Evidence:** 10 unused items: `PROVISION_USAGE`, `parse_provision_args`, `run_event_admission_probe`, `run_event_admission_trail_probe`, `HumanApprovalDecision`, `record_human_approval_decision`, `submit_local_root_anchor`, `short_event_digest`, `process_local_notification`, `resume_and_execute_exact_approval`, `authorise_and_execute`.
- **Disposition:** F2 (remove dead code)
- **Notes:** These appear to be earlier architectural probes or abandoned features. Carrying dead code increases the maintenance surface.

### M2: 137 Clippy Warnings Across Codebase
- **Classification:** Maintainability debt
- **Evidence:** `WARNING_INVENTORY.md` — 137 distinct clippy warning occurrences in lib, tests, and bins.
- **Disposition:** F8 (warnings cleanup)
- **Notes:** Categories include: dead code, unused imports/variables, `too_many_arguments`, `items_after_test_module`, `needless_borrow`, `cmp_owned`, `useless_format`, `unnecessary_map_or`, `type_complexity`.

### M3: Monolithic `application.rs` (8,260 lines)
- **Classification:** Maintainability debt
- **Evidence:** `src/application.rs` — 8,260 lines, the largest file by far. Contains CLI dispatch, runtime orchestration, anchor writing, test support, and dead code.
- **Disposition:** F5 (structural extraction)
- **Notes:** The file mixes concerns: command routing, runtime parts construction, test infrastructure, and abandoned probe functions. Structural extraction would reduce ownership hazards.

### M4: No OCaml `.mli` Interfaces
- **Classification:** Maintainability debt (also Confirmed defect D2)
- **Evidence:** See D2.
- **Disposition:** F5
- **Notes:** All OCaml modules expose internals. `tether_parser.ml` exposes helper functions like `trim`, `starts_with`, `drop_prefix` that should be private.

### M5: Test Infrastructure `pub(crate)` Visibility
- **Classification:** Maintainability debt
- **Evidence:** `src/lib.rs` declares `pub(crate)` modules for test-only files. `src/current_trust_tests.rs`, `src/installation_recovery_*.rs` test modules are accessible crate-wide.
- **Disposition:** F7
- **Notes:** Test infrastructure should use `#[cfg(test)]` gating; some test helpers are currently `pub(crate)` unconditionally.

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
- **Disposition:** F6
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
- **Notes:** Coverage data would help prioritize F2-F8 work but is not a blocking requirement.

### DOC3: No Architecture Decision Record for Live-stderr
- **Classification:** Documentation debt
- **Evidence:** `docs/DECISIONS.md` — no decision record for the stderr capture approach in `child_process.rs`. The Foundation Pass plan references a "live-stderr-tail issue" candidate.
- **Disposition:** F2 (if confirmed) or F9
- **Notes:** The approach to child process stderr capture (bounded, line-buffered, distinction of timeout/EOF/I/O/exit/kill/join) needs explicit documentation.

---

## Summary

| Class | Count | Assigned |
|---|---|---|
| Confirmed defect | 4 | F2 (D1, D3), F5 (D2), F2/defer (D4) |
| Contract ambiguity | 3 | F3b (A1), F7 (A2, A3) |
| Maintainability debt | 5 | F2 (M1), F8 (M2), F5 (M3, M4), F7 (M5) |
| Performance hypothesis | 2 | F6 (P1, P2) |
| Documentation debt | 3 | F9 (DOC1), defer (DOC2), F2/F9 (DOC3) |

No items repaired. All assigned to F-packages or explicitly deferred.
