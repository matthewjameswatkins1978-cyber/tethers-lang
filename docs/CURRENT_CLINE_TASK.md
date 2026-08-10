# Current Implementation Task

Control contract: `1`
Task: `TETHERS-0.3-P1-R1D — Reference Providers Obey Generic Scope`
Owner: `Codex`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `Codex implements bounded repair`
Worker note: `docs/worker-notes/2026-08-10-0.3-p1-r1d-reference-provider-scope.md`
Base branch: `feature/0.3-p1-r1c-canonical-directory`
Base commit: `20cdb463e9c84f35b6d70997916305a1443cfd1d`
Implementation branch: `feature/0.3-p1-r1d-reference-provider-scope`
Implementation checkpoint: `WORKTREE`
OCaml switch path: `not applicable`
Rust toolchain: `1.97.1`
Rust change class: `AMBER_PROVIDER_CONFIGURATION_REPAIR`

## Objective

Make the File Tools and PDF reference providers consume their existing generic Operational Scope exactly. In normal installed operation, absent, malformed, incomplete, wrong-typed, or out-of-bounds scope configuration must refuse rather than select a fallback. Host conformance is the sole exception and activates only when `TETHERS_CONFORMANCE == "1"`.

## Required behaviour

1. File Tools requires `query_root`, `move_source_root`, `move_destination_root`, and `max_content_bytes`, and applies its exact validated limit.
2. PDF requires `query_root` and `max_bytes` in normal installed mode, and applies the exact validated limit.
3. Unset `TETHERS_CONFORMANCE`, `"0"`, and every value other than exact `"1"` are normal installed mode.
4. Exact `"1"` alone preserves the existing TEMP-based bounded conformance fallback.
5. Configuration parsing is locally testable; startup remains the only process-exit boundary.

## Frozen decisions and invariants

1. The providers may understand only their own declared scope; no File/PDF knowledge moves into generic host scope, enablement, candidate/install records, schema validation, or launch machinery.
2. No new dependency, OS sandbox, migration tool, conformance redesign, P2 work, or concurrency change.
3. Overall P1 remains `completion repair in progress`.

## Acceptance criteria

1. File Tools has no normal-mode current-directory or default-limit fallback.
2. PDF has no normal-mode `MAX_PDF_BYTES` fallback.
3. All requested success and fail-closed configuration branches have focused tests.
4. Existing relevant File Tools and PDF provider tests pass.
5. `cargo check --all-targets --all-features --locked`, `cargo fmt --all -- --check`, and `git diff --check` pass.
6. Finished branch is pushed; remote equals local; worktree is clean.

## Authorised paths

- `tethers-0.1/host-rust/src/bin/file_tools_provider.rs`
- `tethers-0.1/host-rust/src/bin/pdf_tools_provider.rs`
- `tethers-0.1/host-rust/tests/m4_file_tools.rs`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-10-0.3-p1-r1d-reference-provider-scope.md`

## Required verification

1. Focused File Tools provider configuration tests.
2. Focused PDF provider configuration tests.
3. Relevant existing File Tools and PDF provider tests.
4. `cargo check --all-targets --all-features --locked`
5. `cargo fmt --all -- --check`
6. `git diff --check`
7. `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`

## Forbidden changes

- No synthetic Plug.
- No `j23c2` conformance repair.
- No old placeholder repair.
- No plug pack/inspect/conform public surface.
- No generic Operational Scope architecture change.
- No P2, migration tooling, concurrency, or unrelated cleanup.
- No `just verify-agent`, engine fixture suite, MCP transcript suite, or full final P1 gate.

## Stop conditions

- Any need to redesign generic host scope, host conformance, provider architecture, or a concurrent execution boundary.
- A required test or check has two materially similar failed attempts.
- An edit causes unrelated formatting or line-ending churn.

## Expected pre-existing changes

None.
