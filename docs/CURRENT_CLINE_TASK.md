# Current Implementation Task

Control contract: `1`
Task: `TETHERS-0.3-P1-R1A — Pin Operational Scope Schema Evidence`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode implements bounded correction`
Worker note: `docs/worker-notes/2026-08-09-0.3-p1-r1a-schema-evidence.md`
Base branch: `origin/main`
Base commit: `c0fd57780156bee023d8dcff884737ea470d096c`
Implementation branch: `feature/0.3-p1-r1a-schema-evidence`
Implementation checkpoint: `a2cb38a67be89abf104b67ad09ebf85d1e0d0f94`
OCaml switch path: `resolve from existing machine state only`
Rust toolchain: `1.97.1`
Rust change class: `AMBER_ARCHITECTURE_CORRECTION`

## Relevant background and existing behaviour

The existing `OperationalScope` enum (operational_scope.rs) ties the generic
enablement and launch lifecycle to two concrete plug subjects: FileTools and
Pdf. Enablement checks package_id, provider_id, and capability_name against
hardcoded PDF constants. Launch replaces `__TETHERS_PDF_*` and `__TETHERS_FILE_*`
placeholders and injects plug-specific environment variables. This prevents
any new Plug from being enabled or launched without core code changes.

## Objective

Remove Plug-subject-specific operational-scope knowledge from the generic
Tethers lifecycle. Replace the File/PDF `OperationalScope` enum with a generic
`OperationalScopeEvidence` model that carries canonicalised scope, integrity
proof, and schema binding without any plug-subject-specific branching.

## Required behaviour

1. Replace the `OperationalScope` enum with generic `OperationalScopeEvidence`.
2. Remove all plug-subject-specific scope branching from generic lifecycle.
3. Generic `plug enable` accepts Plug-declared scope shapes.
4. Scope schemas are package-pinned through the `plug.json` provider section.
5. Scope evidence is canonical (JCS) and tamper-evident.
6. Installed launch uses one generic scope-delivery mechanism.
7. Remove File/PDF launch placeholders and environment variables from generic core.
8. Reference providers consume the generic operational-scope contract.

## Frozen decisions and invariants

1. Generic `OperationalScopeEvidence` replaces `OperationalScope` enum
2. Generic launch delivery via `TETHERS_OPERATIONAL_SCOPE_JSON`/`DIGEST`
3. Plug-declared `operational_scope_schema` in `plug.json` (provider section)
4. No Plug-family enum, no subject-specific branching in generic lifecycle
5. Reference implementations consume generic scope contract
6. No secrets in operational scope
7. Conservative existing JSON Schema validator reused
8. `x-tethers-path: "canonical-directory"` extension for scope paths
9. Pre-0.3 Plug state migration explicitly authorised

## Acceptance criteria

1. File/PDF `OperationalScope` enum is gone — DONE
2. Generic lifecycle has no Plug-subject-specific scope branching — DONE
3. `plug enable` accepts Plug-declared scope shapes — DONE
4. Scope schemas are package-pinned — DONE
5. Scope evidence is canonical and tamper-evident — DONE
6. Installed launch uses one generic scope-delivery mechanism — DONE
7. File/PDF launch placeholders/env variables gone from generic core — DONE
8. Language semantics remain 0.1 — PRESERVED
9. No dependency change — PRESERVED
10. Focused verification passes — PENDING
11. One `just verify-agent` passes — PENDING
12. `git diff --check` clean — PENDING
13. Branch is pushed, remote equals local, worktree clean — PENDING
14. No-knowledge search clean — DONE

## Stop conditions

Already resolved: complete lifecycle generic without breaking existing behaviour.

## Expected pre-existing changes

None.

## Relevant components

### AUTHORISED PATHS
- `tethers-0.1/host-rust/src/operational_scope.rs`
- `tethers-0.1/host-rust/src/enablement.rs`
- `tethers-0.1/host-rust/src/launch_profile.rs`
- `tethers-0.1/host-rust/src/plug_command.rs`
- `tethers-0.1/host-rust/src/package.rs`
- `tethers-0.1/host-rust/src/pdf_tools.rs`
- `tethers-0.1/host-rust/src/file_tools.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/bin/pdf_tools_provider.rs`
- `tethers-0.1/host-rust/src/bin/file_tools_provider.rs`
- `tethers-0.1/host-rust/tests/j23c1_operational_scope.rs`
- `tethers-0.1/host-rust/tests/j23c3_installed_pdf_execution.rs`
- `tethers-0.1/host-rust/tests/m4_file_tools.rs`
- `tethers-0.1/host-rust/tests/j24b_plug_list_cli.rs`
- `tethers-0.1/host-rust/tests/j24c_plug_disable_cli.rs`
- `tethers-0.1/host-rust/tests/j24d_plug_enable_scope_file.rs`
- `tethers-0.1/host-rust/src/f3d_bounded_persistence_stores_evidence.rs`
- `docs/ROAD_TO_0_3.md`
- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-0.3-p1-generic-operational-scope.md`

## Required verification

1. No-knowledge search: no plug-subject-specific constants in generic lifecycle.
2. Lib tests pass (1340 tests).
3. `cargo fmt --all -- --check` clean.
4. `cargo check --all-targets --all-features --locked` clean.
5. `cargo clippy --all-targets --all-features --locked` passes.
6. `git diff --check` clean.
7. `just verify-agent` passes.

## Forbidden changes

- No Tethers language change
- No concurrency
- No plug pack/inspect/conform implementation
- No registry, marketplace, HTTP/WebSocket/gRPC, SDK, secret store, OAuth, OS sandbox
- No dependency update
- No physical extraction into `reference-plugs/`
- No unrelated cleanup

## Stop conditions

Already resolved: complete lifecycle generic without breaking existing behaviour.
