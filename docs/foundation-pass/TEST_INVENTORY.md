# F1 Test Inventory

This is a **representative** inventory mapped to J-prefix contracts (J13-J24), not an exhaustive test-by-test catalogue. Tests are grouped by contract and module, with representative test names to locate the coverage area. Not every one of the 1,254 unit tests is individually listed.

All Rust tests at `tethers-0.1/host-rust/`. Baseline: `24428139807cac0adeb0b62264547e61ca809d16`.

## Unit Tests (inline `#[cfg(test)]` modules)

1,254 tests passed (warm), 2 ignored.

### CLI & Output (J13)

| Contract | File | Test count | Evidence |
|---|---|---|---|
| J13A: CLI parsing, path resolution, engine session, output envelope | `src/tests/j13a_cli.rs` | 29 | `j13a_envelope_has_correct_schema`, `j13a_exit_code_matches_envelope`, `j13a_error_envelope_has_code_and_message`, etc. |
| J13B: Run command approval, admission, service boundary | `src/run_command.rs`, `src/run_input.rs` | 13 | `j13b_run_initial_admission_is_external_durable_and_host_owned`, `j13b_run_rejections_do_not_reach_the_service_boundary`, etc. |
| J13C: Trail reading, filtering, CRLF, malformed input | `src/trail_command.rs` | 21 | `j13c_crlf_preserves_internal_data`, `j13c_success_envelope_has_correct_shape`, etc. |

### PDF Capability (J23)

| Contract | File | Test count | Evidence |
|---|---|---|---|
| J23A: PDF provider (MCP stdio) | `tests/j23a_pdf_provider.rs` | 7 | `provider_initializes_and_lists_only_pdf_inspect`, `binary_non_utf8_body_is_accepted_over_stdio` |
| J23B: PDF package building | `tests/j23b_pdf_package.rs` | 1 | `package_build_is_deterministic_and_matches_contract` |
| J23C1: Operational scope binding | `tests/j23c1_operational_scope.rs` | 23 | Scope creation, tampering, enable/disable |
| J23C2: PDF conformance | `tests/j23c2_pdf_conformance.rs` | 8 | Conformance protocol, placeholder validation |
| J23C3: Installed PDF execution | `tests/j23c3_installed_pdf_execution.rs` | 1 | End-to-end installed plug lifecycle |

### Plug Management (J24)

| Contract | File | Test count | Evidence |
|---|---|---|---|
| J24A: Plug inspect CLI | `tests/j24a_plug_inspect_cli.rs` | 3 | Public inspect envelope, malformed command shapes |
| J24B: Plug list CLI | `tests/j24b_plug_list_cli.rs` | 4 | Stable ordering, read-only, fail-closed |
| J24C: Plug disable CLI | `tests/j24c_plug_disable_cli.rs` | 9 | Cross-record drift, corrupt chains, re-entrant disable |
| J24D: Plug enable scope file | `tests/j24d_plug_enable_scope_file.rs` | 16 | Scope file validation, re-enable with predecessor linkage |
| J24E: Candidate preparation | `tests/j24e_candidate_preparation.rs` | 17 | Junction rejection, quarantine immutability, exact replay |
| J24F: Plug stage CLI | `tests/j24f_plug_stage_cli.rs` | 6 | Junction rejection, service failure preservation |
| J24G: Installation request | `tests/j24g_installation_request.rs` | 16 | Byte boundary, pointer-annotated rejections, no FS mutation |
| J24H: Installation evidence access | `tests/j24h_installation_evidence_access.rs` | 19 | Junction rejection, filename digests, round-trip |
| J24I: Exact candidate trust | `tests/j24i_exact_candidate_installation_trust.rs` | 30 | Deterministic evidence, digest recompute, record conflicts |
| J24J: Installation reconciliation | `tests/j24j_installation_reconciliation.rs` | 24 | Staleness, conformance selection, planning immutability |
| J24K2: Locked single-step executor | `tests/j24k2_locked_single_step_executor.rs` | 9 | Lock semantics, tampering detection, postplan failure resume |
| J24L2: Plug install CLI | `tests/j24l2_plug_install_cli.rs` | 10 | E2E install and reinstall, Clap parsing |

### M3, M4, M5 Lifecycle

| Contract | File | Test count | Evidence |
|---|---|---|---|
| M3: Trust lifecycle (Ed25519, candidate signing, conformance, approval) | `tests/m3_lifecycle.rs` | 13 | `m3_golden_schemas_are_committed_and_strictly_typed`, `m3_windows_handle_allow_list_excludes_unrelated_inheritable_handle` **(flaky)**, handle containment |
| M4: File Tools provider | `tests/m4_file_tools.rs` | 4 | Native provider launch, query, non-overwriting move |
| M5: Local anchor | `tests/m5_local_anchor.rs` | 1 | Durable restart, duplicate conflict, scope |

### Core Services

| Contract | Module | Representative Tests |
|---|---|---|
| Trust & Signatures | `src/trust.rs` | RFC 8032 vectors, signature refusal matrix, trust transitions |
| Trusted Store | `src/trusted_store.rs` | Identity/digest conflicts, reinsertion, case sensitivity |
| Policy | `src/policy.rs` | Declaration, override, deny/default posture, scope assessment |
| Provider | `src/provider.rs` | Manifest admission, PIN verification, identity conflicts |
| Resolver | `src/resolver.rs` | Availability, projection determinism, version scoping |
| Runtime Config (J12) | `src/runtime_config.rs` | Duplicate rejection, materialization, relative path resolution |
| Validation | `src/validation.rs` | JSON Schema subset, integer boundaries, SHA-256 pattern |
| Manifest | `src/manifest.rs` | Canonicalization, unknown fields, schema coverage |
| Package | `src/package.rs` | ZIP profile enforcement, payload index, archive boundaries |
| Replay & Windows Persistence | `src/replay_windows.rs` | Ledger 01-30, NTFS volume, lock semantics, recovery |
| Socket | `src/socket.rs` | Discovery pagination, schema drift, catalogue notifications |
| Stdio Provider | `src/stdio_provider.rs` | MCP discovery, protocol mismatch, process exit |
| Result Anchor | `src/result_anchor.rs` | Provider error anchors, serialization shapes |
| PDF Tools | `src/pdf_tools.rs` | Committed manifests, frozen digests, page marker scanning |
| Local Anchor | `src/local_anchor.rs` | Admission binding, coordinator |
| Installation Recovery | `src/installation_recovery_*.rs` | Plan, evidence, execution, audit, destination tests |
| Installation Publication | `src/installation_publication_*.rs` | Intent, mutation, preparation tests |

## Known Gaps

1. No OCaml-native unit tests exist. Engine tested only via Rust integration tests and PowerShell scripts.
2. Test modules within production files (e.g., `src/current_trust_tests.rs`) blur the production/test boundary.
3. The flaky M3 handle-allow-list test has non-deterministic pass/fail — likely a timing-dependent Windows handle enumeration issue.
