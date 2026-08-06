# F1 Warning Inventory

Total: **137 Clippy warning occurrences** (across lib, tests, and bins).
Compiler (`cargo check`): additional dead_code/unused warnings.

Source: `cargo clippy --all-targets --all-features --locked -- -W clippy::all` on Rust 1.97.1.

## By Category

### Production Code (lib)

| Warning | Count | Files |
|---|---|---|
| `dead_code` (unused functions/constants) | 10 | `application.rs` (7: `PROVISION_USAGE`, `parse_provision_args`, `run_event_admission_probe`, `run_event_admission_trail_probe`, `HumanApprovalDecision`, `record_human_approval_decision`, `submit_local_root_anchor`, `short_event_digest`, `process_local_notification`, `resume_and_execute_exact_approval`, `authorise_and_execute`) |
| `dead_code` (never-constructed struct) | 1 | `application.rs` (`FailingResultAnchorWriter`) |
| `dead_code` (unread struct field) | 1 | `child_process.rs` (`SupervisedChild.max_line_bytes`) |
| `dead_code` (unused associated items) | 2 | `installation_publication_intent.rs` (`open_existing`, `root_path`) |
| `dead_code` (unused methods) | 2 | `launch_profile.rs` (`revalidate_current_trust`, `launch_for_candidate`) |
| `dead_code` (never-constructed enum variants) | 2 | `result_anchor.rs` (`ProviderError`, `ResultValidationFailed`) |
| `items_after_test_module` | 2 | `candidate_preparation.rs`, `installation_publication_mutation.rs` |
| `type_complexity` | 2 | `application.rs:5741`, `child_process.rs:857` |
| `field_reassign_with_default` | 1 | `application.rs:5751` |
| `cmp_owned` | 3 | `cli.rs:589`, `cli.rs:819`, `cli.rs:820` |
| `new_without_default` | 1 | `dispatch.rs` (`RecordingTrail`) |
| `needless_borrows_for_generic_args` | 1 | `execution_environment.rs:1369` |
| `useless_format` | 1 | `manifest.rs:1605` |
| `unnecessary_map_or` | 2 | `current_trust_tests.rs:669`, `current_trust_tests.rs:676` |
| `excessive_precision` | 1 | `manifest.rs:2065` |
| `unnecessary_get_then_check` | 4 | `trusted_store.rs:543,546,596,598` |
| `result_large_err` | 1 | `run_command.rs:998` |
| `unnecessary_map_or` (production) | 1 | `installed.rs:1274` |

### Test Code (inline `#[cfg(test)]` modules)

| Warning | Count | Files |
|---|---|---|
| `dead_code` (unused test functions) | 2 | `installation_execution_tests.rs` (`empty_plan`, `plan_with`) |
| `too_many_arguments` | 2 | `installation_execution_tests.rs:57`, `installation_recovery_plan_tests.rs:491` |
| `dead_code` (unread struct fields in test fixtures) | 7 | `installation_publication_mutation_tests.rs:134-137`, `installation_recovery_plan_tests.rs:297-299` |
| `unused_variables` | 1 | `installation_publication_mutation_tests.rs:968` |
| `unused_imports` | 3 | `installation_publication_mutation_tests.rs`, `installation_publication_preparation_tests.rs` |
| `permissions_set_readonly_false` | 11 | Multiple test files (Windows-only platform, Unix warning) |

### Integration Tests

| Warning | Count | Files |
|---|---|---|
| `unused_imports` | 5 | `j13a_cli.rs`, `j23b_pdf_package.rs`, `j23c3_installed_pdf_execution.rs` (3) |
| `unused_variables` | 5 | `j13a_cli.rs` (4), `j24d_plug_enable_scope_file.rs` |
| `dead_code` | 1 | `j24d_plug_enable_scope_file.rs` (`canonical`) |
| `useless_format` | 1 | `j13a_cli.rs:190` |
| `needless_borrow` | 4 | `j24c_plug_disable_cli.rs`, `j24d_plug_enable_scope_file.rs` (3) |
| `needless_borrows_for_generic_args` | 1 | `j24j_installation_reconciliation.rs:1413` |
| `single_component_path_imports` | 1 | `j24j_installation_reconciliation.rs` |
| `too_many_arguments` | 2 | `j24j_installation_reconciliation.rs:284`, `j24k2_locked_single_step_executor.rs:202` |
| `suspicious_open_options` | 1 | `j24k2_locked_single_step_executor.rs:324` |
| `permissions_set_readonly_false` | 3 | `m3_lifecycle.rs`, `j24k2_locked_single_step_executor.rs`, `j24e_candidate_preparation.rs` |
| `for_kv_map` | 1 | `j24e_candidate_preparation.rs:388` |
| `manual_range_contains` | 1 | `pdf_tools_provider.rs:101` |

### Provider Binaries

| Warning | Count | Files |
|---|---|---|
| `useless_conversion` | 2 | `file_tools_provider.rs:40` |
| `unused_variable` | 1 | `m3_fixture_provider.rs` |

## Compiler Warnings (`cargo check`)

| Warning | Count | Notes |
|---|---|---|
| `dead_code` (lib) | 15 | Same unused functions/structs as Clippy |
| `unused_imports` (tests) | 9 | Various test files |
| `unused_variables` (tests) | 3 | |

## Notable Patterns

1. **`permissions_set_readonly_false`**: 14 occurrences across test code, all irrelevant on Windows but flagged for Unix portability. This is a cross-platform lint that doesn't apply to a Windows-only codebase.
2. **Dead code in `application.rs`**: 10 unused items from earlier architecture/probe functions, suggesting accumulated technical debt from evolution.
3. **`FailingResultAnchorWriter`**: Declared but never constructed dead code.
4. **`items_after_test_module`**: 2 files place public items after `#[cfg(test)]` modules, violating Rust convention.
5. **Missing `.mli` interfaces in OCaml**: All OCaml modules expose their full internals — no abstraction boundaries.
6. **No OCaml warnings available**: Engine was not compiled during this baseline (F1 is documentation-only; OCaml switch is N/A).
