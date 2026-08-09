# Warning and Tooling Reconciliation — F8a Evidence

**Status:** EVIDENCE-ONLY
**Audit date:** 2026-08-09
**Audit checkpoint:** `74904309d9af04024cd1a0b60c4cf654b8617481`
**Rust toolchain:** 1.97.1 (pinned via `rust-toolchain.toml`)
**OCaml switch:** N/A (no switch set)

---

## 1. Environment

| Item | Value |
| --- | --- |
| OS | Windows (native) |
| Shell | PowerShell 7.6.4 |
| Git | 2.54.0.windows.1 |
| Rust | 1.97.1-x86_64-pc-windows-msvc |
| OCaml (opam) | No switch set |
| just | 1.57.0 |
| rg | 15.2.0 |
| fd | 10.4.2 |
| jq | 1.8.2 |
| yq | v4.53.3 |
| gh | 2.97.0 |

---

## 2. Command-Result Table

| # | Command | Result | Notes |
| --- | --- | --- | --- |
| 1 | `git status --short` | PASS (clean) | No dirty files |
| 2 | `cargo check --all-targets --all-features --locked` | PASS (33 warnings) | 15 lib + 18 test |
| 3 | `cargo test --all-targets --all-features --locked` | PASS (1331 pass, 0 fail, 2 ignored) | All tests green |
| 4 | `cargo clippy --all-targets --all-features --locked -- -W clippy::all` | PASS (81 warnings, 36 duplicates) | 45 unique lint sites |
| 5 | `cargo fmt --all -- --check` | **FAIL** | `replay_windows.rs:3277` |
| 6 | `just verify` | **FAIL** (at fmt) | Stopped at step 2/4 |
| 7 | `just verify-agent` | **FAIL** (at fmt) | Stopped at verify (first dep) |
| 8 | `opam exec -- dune build` | **FAIL** | No OCaml switch set |
| 9 | `test-engine.ps1` | **FAIL** | Depends on dune build |
| 10 | `test-mcp-transcripts.ps1` | PASS | 15 cases all passed |
| 11 | `check-fixtures.ps1` | PASS | 46 JSON, 30 JSONL valid |
| 12 | `git diff --check` | PASS (clean) | No whitespace issues |
| 13 | `check-tethers-task-packet.ps1` | PASS | control-v1/IN_PROGRESS |

---

## 3. Warning Counts by Command

| Command | Total | Lib | Test | Bin |
| --- | --- | --- | --- | --- |
| `cargo check` | 33 | 15 | 18 | 0 |
| `cargo clippy` (distinct) | 45 | ~19 | ~21 | ~5 |
| `cargo clippy` (raw emitted) | 81 | — | — | — |

---

## 4. Distinct Warning / Root-Cause Inventory

### 4.1 Production Lib Dead Code (`cargo check`)

All in `src/application.rs` unless noted.

| # | Item | Kind | File:Line |
| --- | --- | --- | --- |
| D1 | `PROVISION_USAGE` const | unused constant | `application.rs:24` |
| D2 | `parse_provision_args` | unused function | `application.rs:89` |
| D3 | `run_event_admission_probe` | unused function | `application.rs:377` |
| D4 | `run_event_admission_trail_probe` | unused function | `application.rs:511` |
| D5 | `HumanApprovalDecision` enum | unused type | `application.rs:1320` |
| D6 | `record_human_approval_decision` | unused function | `application.rs:1387` |
| D7 | `submit_local_root_anchor` | unused function | `application.rs:1777` |
| D8 | `short_event_digest` | unused function | `application.rs:1819` |
| D9 | `process_local_notification` | unused function | `application.rs:1832` |
| D10 | `resume_and_execute_exact_approval` | unused function | `application.rs:1906` |
| D11 | `authorise_and_execute` | unused function | `application.rs:2049` |
| D12 | `max_line_bytes` field | unread field | `child_process.rs:239` |
| D13 | `open_existing`, `root_path` | unused methods | `installation_publication_intent.rs:124,179` |
| D14 | `revalidate_current_trust`, `launch_for_candidate` | unused methods | `launch_profile.rs:541,639` |
| D15 | `ProviderError`, `ResultValidationFailed` | unused enum variants | `result_anchor.rs:42,44` |

### 4.2 Test Dead Code (`cargo check`)

| # | Item | Kind | File:Line |
| --- | --- | --- | --- |
| T1 | `std::io::Write` | unused import | `tests/j13a_cli.rs:4` |
| T2 | `code` (3 sites) | unused variable | `tests/j13a_cli.rs:84,98,188` |
| T3 | `envelope` | unused variable | `tests/j13a_cli.rs:284` |
| T4 | `serde_json::Value` | unused import | `tests/j23b_pdf_package.rs:13` |
| T5 | `std::io::Write`, `PathBuf`, `MAX_PDF_BYTES` | unused imports | `tests/j23c3_installed_pdf_execution.rs:5,6,18` |
| T6 | `before` | unused variable | `tests/j24d_plug_enable_scope_file.rs:315` |
| T7 | `canonical` | unused function | `tests/j24d_plug_enable_scope_file.rs:32` |
| T8 | `InstallationPlanAction`, `DisabledBindingRecord` | unused imports | `src/installation_publication_mutation_tests.rs:8,28` |
| T9 | `error` | unused variable | `src/installation_publication_mutation_tests.rs:968` |
| T10 | `PayloadEvidence` | unused import | `src/installation_publication_preparation_tests.rs:28` |
| T11 | `empty_plan` | unused function | `src/installation_execution_tests.rs:38` |
| T12 | `plan_with` | unused function | `src/installation_execution_tests.rs:57` |
| T13 | Fixture struct fields | unread fields | `src/installation_publication_mutation_tests.rs:134-137` |
| T14 | FullFixture struct fields | unread fields | `src/installation_recovery_plan_tests.rs:297-299` |
| T15 | `FailingResultAnchorWriter` | unused struct | `src/application.rs:1871` (lib test) |

### 4.3 Clippy Lints Grouped by Root Cause

#### A. Preference Warnings (Not Defects)

| ID | Lint | Occurrences | Root Cause |
| --- | --- | --- | --- |
| P1 | `too_many_arguments` | 16 sites | Functions with 8-10 params in test helpers, build-constructors, and lib fn sigs |
| P2 | `type_complexity` | 3 sites | Complex return types in `child_process.rs` and test-only `application.rs:5758` |
| P3 | `result_large_err` | 5 sites | `RunResult` and `CheckResult` over 160 bytes; boxing suggested |
| P4 | `new_without_default` | 1 site | `RecordingTrail::new()` without `Default` impl |
| P5 | `items_after_test_module` | 2 sites | Test module defined before production code in `candidate_preparation.rs`, `installation_publication_mutation.rs` |
| P6 | `single_component_path_imports` | 1 site | Redundant `use serde_json_canonicalizer` in `j24j_installation_reconciliation.rs:3` |

#### B. Actionable Cleanup (Mechanically Fixable)

| ID | Lint | Occurrences | Root Cause |
| --- | --- | --- | --- |
| C1 | Unused imports / variables (check) | 18 sites | Legacy test code with accumulated unused bindings |
| C2 | `needless_borrow` | 5 sites | `&var.iter()` where `var.iter()` works |
| C3 | `needless_borrows_for_generic_args` | 5 sites | `&expr` where generic inference accepts owned |
| C4 | `useless_format` | 2 sites | `format!("{x}")` instead of `x.to_string()` |
| C5 | `cmp_owned` | 3 sites | `PathBuf::from("...")` in comparison |
| C6 | `unnecessary_map_or` | 7 sites | `map_or(false, ...)` → `is_some_and(...)` or `is_ok_and(...)` |
| C7 | `unnecessary_get_then_check` | 4 sites | `.get(k).is_some()` → `.contains_key(k)` |
| C8 | `cmp_null` | 2 sites | `handle != ptr::null_mut()` → `!handle.is_null()` |
| C9 | `for_kv_map` | 4 sites | `for (k, _) in map` → `for k in map.keys()` |
| C10 | `useless_conversion` | 2 sites | `PathBuf::from(x)` where x is already `&Path` in `file_tools_provider.rs` |
| C11 | `collapsible_if` | 1 site | Nested if blocks in `installed.rs:1235` |
| C12 | `clone_on_copy` | 1 site | `.clone()` on Copy type in `installation_execution.rs:706` |
| C13 | `bind_instead_of_map` | 1 site | `.and_then(|x| Ok(y))` → `.map(|x| y)` in `local_anchor.rs:454` |
| C14 | `len_zero` | 1 site | `archive.len() == 0` → `archive.is_empty()` in `package.rs:356` |
| C15 | `ptr_arg` | 1 site | `&PathBuf` → `&Path` in `stdio_provider.rs:30` |
| C16 | `needless_question_mark` | 1 site | `Ok(x?)` → `x` in `host_execution.rs:368` |
| C17 | `redundant_guards` | 1 site | `Ok(data) if data.is_empty()` → `Ok([])` in `child_process.rs:657` |
| C18 | `field_reassign_with_default` | 1 site | Post-default field assignment in `application.rs:5768` |
| C19 | `excessive_precision` | 1 site | Float literal with excess digits in `manifest.rs:2065` |
| C20 | `manual_range_contains` | 1 site | `n >= 1 && n <= MAX` → `(1..=MAX).contains(&n)` in `pdf_tools_provider.rs:101` |
| C21 | Production dead code (check D1-D15) | 15 sites | Unused items in application/child_process/launch_profile/result_anchor |
| C22 | Test dead code (check T1-T15) | 18 sites | Unused items in tests |
| C23 | `doc_overindented_list_items` | 1 site | Documentation formatting in `validation.rs:16` |
| C24 | `suspicious_open_options` | 2 sites | `.create(true)` without `.truncate(true)` in `installation_execution.rs:109`, `j24k2`:324 |

#### C. JUSTIFIED Warnings (Not Authorised for Change)

| ID | Lint | Occurrences | Justification |
| --- | --- | --- | --- |
| J1 | `permissions_set_readonly_false` | 13 sites | Native Windows test helpers that need writable permissions; Windows-only code; the Clippy lint is Unix-focused |

---

## 5. Current `cargo fmt` Failure Characterization

- **File:** `tethers-0.1/host-rust/src/replay_windows.rs`
- **Region:** Line 3277 (within test-only code, likely a `#[cfg(test)]` block)
- **Issue:** A single `assert!` call has its argument exceeding the line length, and rustfmt wants to break the chained method calls onto separate lines.
- **Formatting-only change:** YES. rustfmt would make whitespace-only changes (line breaks and indentation).
- **Semantic impact:** NONE. The reformatted expression is semantically identical.
- **Safety to treat as simple formatting:** SAFE. No source/semantic interaction. The chained `.with_file_name(...).exists()` is already on one line; rustfmt wants to split it.
- **Actual diff applied:**

```diff
-        assert!(!g0_path.with_file_name("g0000000000000000.json.tmp").exists());
+        assert!(!g0_path
+            .with_file_name("g0000000000000000.json.tmp")
+            .exists());
```

---

## 6. `just verify` and `just verify-agent` Behaviour

### `just verify`

Defined in `justfile:34-38`:

```just
verify:
    pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
    cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check
    cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked
    cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked
```

Steps 1-4 run sequentially; failure at any step stops the recipe.

**Current behaviour:** Step 1 (packet checker) PASSES. Step 2 (`cargo fmt --check`) FAILS at `replay_windows.rs:3277`. Steps 3-4 are never reached. Exit code: 1.

### `just verify-agent`

Defined in `justfile:55`:

```just
verify-agent: verify agent-tools deps-policy deps-advisories test-agent
```

`verify` is the first dependency. Since `verify` fails at `cargo fmt --check`, `verify-agent` stops at the same point and never runs `agent-tools`, `deps-policy`, `deps-advisories`, or `test-agent`. Exit code: 1.

---

## 7. Configuration Inventory (Read-Only)

### Cargo.toml (`tethers-0.1/host-rust/Cargo.toml`)
- **No `[lints]` section.** No workspace-level lint configuration.
- **No `[profile]` overrides** for warnings.
- Single crate (not a workspace).
- Edition: 2021, rust-version: 1.97.

### rustfmt Configuration
- **No `rustfmt.toml` or `.rustfmt.toml` exists** anywhere in the repository.
- rustfmt uses its default configuration.

### Clippy Configuration
- **No `clippy.toml` or `.clippy.toml` exists** anywhere in the repository.
- Clippy uses its default configuration with `-W clippy::all`.

### CI/Workflow Warning Enforcement
- **No CI workflow files exist** under `.github/workflows/`.
- No automated warning denial or enforcement.

### Rust Toolchain
- `rust-toolchain.toml` pins to channel `1.97.1`, profile `minimal`.
- Components: `rustfmt`, `clippy`, `rust-analyzer`.

### Justfile Verification
- `verify`: packet-checker → fmt-check → cargo-check → cargo-test
- `verify-agent`: verify + agent-tools + deps-policy + deps-advisories + test-agent
- Both short-circuit at `cargo fmt --check` failure.

### OCaml
- No OCaml switch configured. `opam exec -- dune build` fails with error 50.
- `test-engine.ps1` depends on `dune build` and fails.

---

## 8. Classification Table

| Category | Count | Description |
| --- | --- | --- |
| ACTIONABLE CLEANUP | ~70 | Unused imports/variables, mechanical Clippy fixes (needless_borrow, cmp_owned, unnecessary_map_or, etc.), dead code removal, doc formatting |
| JUSTIFIED WARNING | 13 | `permissions_set_readonly_false` — Windows test helpers, Unix-focused lint |
| STALE / NO LONGER PRESENT | 0 | All warnings verified live at HEAD `7490430` |
| TOOLING/CONFIGURATION ISSUE | 2 | No OCaml switch (engine tests unavailable); fmt failure blocks verify/verify-agent |
| UNVERIFIED | 0 | All identified warnings traced to concrete file:line |

---

## 9. Protected Contracts

These warnings affect code whose cleanup could require non-trivial judgement:

| Warning | Protected Contract |
| --- | --- |
| D1-D11 (application.rs dead code) | May be reserved for future CLI routes or unshipped features; removal requires feature-authority review |
| D13 (installation_publication_intent.rs) | `open_existing` / `root_path` may be part of the store contract |
| D14 (launch_profile.rs) | `revalidate_current_trust` / `launch_for_candidate` are part of the launch lifecycle; may be used externally |
| D15 (result_anchor.rs) | `ProviderError` / `ResultValidationFailed` are enum variants in a public API type |
| P1 (too_many_arguments, 16 sites) | Prefence lint; most functions represent honest domain signatures; restructuring would be non-trivial |
| P3 (result_large_err, 5 sites) | Would require boxing `RunResult`/`CheckResult`, changing error type layout |
| P5 (items_after_test_module, 2 sites) | Reordering code structure in production files |
| J1 (permissions_set_readonly_false, 13 sites) | Windows-only test code; lint is Unix-specific |
| D12 (max_line_bytes) | Field in `SupervisedChild` struct; removal changes public type |

---

## 10. Proposed Bounded F8 Cleanup Packages

### F8-PACKAGE-1: Mechanical Test Cleanup (LOW RISK, ~30 fixes)
**Files:** `tests/j13a_cli.rs`, `tests/j23b_pdf_package.rs`, `tests/j23c3_installed_pdf_execution.rs`, `tests/j24d_plug_enable_scope_file.rs`, `tests/j24c_plug_disable_cli.rs`, `tests/j24e_candidate_preparation.rs`, `tests/j24j_installation_reconciliation.rs`, `tests/j24k2_locked_single_step_executor.rs`, `tests/f3b_windows_persistence_evidence.rs`, `tests/m3_lifecycle.rs`, `src/bin/file_tools_provider.rs`, `src/bin/pdf_tools_provider.rs`

Fixes: unused imports, unused variables (prefix `_`), `needless_borrow`, `useless_format`, `cmp_owned`, `for_kv_map`, `len_zero`, `useless_conversion`, `cmp_null`, `manual_range_contains`, `single_component_path_imports`, `collapsible_if`, `clone_on_copy`, `bind_instead_of_map`, `needless_borrows_for_generic_args`, `redundant_guards`, `excessive_precision`, `doc_overindented_list_items`, `suspicious_open_options`, `ptr_arg`, `needless_question_mark`, `field_reassign_with_default`.

All changes mechanically fixable with `cargo clippy --fix` or trivial manual edits.

### F8-PACKAGE-2: Production Dead Code Audit (MEDIUM RISK)
**Files:** `src/application.rs`, `src/child_process.rs`, `src/installation_publication_intent.rs`, `src/launch_profile.rs`, `src/result_anchor.rs`

Requires: per-item decision whether unused code is reserved for future features or removable. Each removal should be a separate commit. `pub(crate)` items may have external callers. Variant removal from public enums requires broader review.

### F8-PACKAGE-3: Lib Test Dead Code Cleanup (LOW RISK)
**Files:** `src/installation_publication_mutation_tests.rs`, `src/installation_publication_preparation_tests.rs`, `src/installation_execution_tests.rs`, `src/installation_recovery_plan_tests.rs`

Unused imports, variables, and struct fields in `src/*_tests.rs`. `#[cfg(test)]` code; no public API impact.

### F8-PACKAGE-4: Preference Lint Justification (NO-RISK)
All `too_many_arguments`, `type_complexity`, `result_large_err`, `new_without_default`, `items_after_test_module` sites: either add `#[allow(clippy::...)]` with justification comment, or explicitly document as accepted preferences. No code restructuring required.

### F8-PACKAGE-5: `unnecessary_map_or` / `unnecessary_get_then_check` Cleanup (LOW RISK)
**Files:** `src/validation.rs`, `src/f3d_bounded_persistence_stores_evidence.rs`, `src/installed.rs`, `src/current_trust_tests.rs`, `src/trusted_store.rs`

Mechanical replacements: `map_or(false, ...)` → `is_some_and(...)`, `.get(k).is_some()` → `.contains_key(k)`. All in `src/` but mechanically correct.

### F8-FMT: Formatting Fix (TRIVIAL, SHOULD BE OWN PACKAGE)
**File:** `src/replay_windows.rs:3277`

Single formatting-only change in test code. Apply `cargo fmt`. This unblocks `just verify` and `just verify-agent`.

**Recommendation:** F8-FMT should be its own tiny F8 package, applied FIRST because it unblocks the verification pipeline for all subsequent work.

---

## 11. Recommended F8 Execution Order

1. **F8-FMT** — Apply `cargo fmt`. Committing this immediately unblocks `just verify` and `just verify-agent`.
2. **F8-PACKAGE-1** — Mechanical test cleanup. No risk, trivially reversible.
3. **F8-PACKAGE-3** — Lib test dead code cleanup. Same risk profile.
4. **F8-PACKAGE-5** — Modern API usage (`is_some_and`, `contains_key`).
5. **F8-PACKAGE-4** — Justify or allow preference lints.
6. **F8-PACKAGE-2** — Production dead code audit (requires per-item judgement).

After all packages, run a final `cargo check --all-targets --all-features` and `cargo clippy` to confirm zero remaining warnings before F8b (gate activation).

---

## 12. Explicit Non-Authorisations

- No CI enforcement configuration
- No `[lints]` workspace denial
- No `#[deny(warnings)]` or `#![deny(clippy::all)]`
- No `permissions_set_readonly_false` changes (justified for Windows tests)
- No `too_many_arguments` / `type_complexity` / `result_large_err` restructuring (prefence lints)
- No OCaml switch installation
- No public API signature changes
- No `items_after_test_module` reordering (structural change in prod files — deferred)
- No F8b gate activation

---

## 13. Causal Limits

- **OCaml engine:** Unavailable (no switch). Engine tests cannot run. This does not block F8 Rust cleanup. The OCaml tooling state is recorded for awareness.
- **verify-agent sub-tools:** Not reachable until `just verify` passes. Their state is unknown (not exercised). They are expected to work once fmt is fixed and all dep/agent tool binaries are available.
- **Historical F1/F5 numbers:** Confirmed stale. Current numbers differ (check: 33 vs 16; clippy: 45 distinct vs 81 raw).

---

## 14. Smallest Next Move

Apply `cargo fmt` to `replay_windows.rs:3277` as **F8-FMT** (a single, tiny, formatting-only commit). This unblocks `just verify` and `just verify-agent`, establishing the verification baseline for all subsequent F8 cleanup packages.

---

## 15. Git References

| Item | SHA |
| --- | --- |
| Base commit | `5ecf54e17752096e7c553e059d014ef263cbb136` |
| Audit checkpoint | `74904309d9af04024cd1a0b60c4cf654b8617481` |
| Branch | `foundation/f8a-warning-tooling-reconciliation` |
| Git status at checkpoint | Clean (documentation only) |
