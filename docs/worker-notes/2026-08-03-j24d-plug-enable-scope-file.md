# Worker Note

Task: `J24D - Permission-file Plug enable CLI`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `fb354dea734e7a2d37254a9cfbca4fd0daad5939`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Add `plug enable --host-data-root <ABSOLUTE_PATH> --installed-id <UUID> --scope <ABSOLUTE_JSON_PATH>` as the first permission-file-based enablement command. Parse the permission request as hostile input with exact duplicate-key rejection, validate the complete installed and enablement state before mutation, construct scope only through `PdfOperationalScopeBinding::create`, and append enablement only through `EnablementStore::enable`.

## Changes made

- `tethers-0.1/host-rust/src/cli.rs` — added `Enable` variant to `PlugCommand` with `--host-data-root`, `--installed-id`, and `--scope` options; added 11 CLI syntax tests.
- `tethers-0.1/host-rust/src/application.rs` — routed `PlugCommand::Enable` to `plug_command::run_enable`.
- `tethers-0.1/host-rust/src/plug_command.rs` — added `PlugScopeRequest`, `CapabilityRequest`, `PermissionRequest` types with custom `Deserialize` implementations for exact duplicate-key rejection via `visit_map` key tracking; added `parse_scope_file` for hostile input parsing (16 KiB limit, BOM rejection, UTF-8 check, trailing content rejection, strict schema/capability/max_bytes/query_root validation); added `run_enable` with full validate-before-mutate logic (lifecycle layout, installed registry, enablement chain, cross-record consistency, already-enabled rejection, scope construction, enablement authority).
- `tethers-0.1/host-rust/tests/j24d_plug_enable_scope_file.rs` — 16 integration tests covering success (never-enabled, re-enable after disable), envelope shape, already-enabled failure, unknown ID, malformed/oversized/duplicate-key/wrong-schema/wrong-capability/missing-field scope files, missing scope file, missing root, partial layout, and CLI validation.
- `docs/CURRENT_CLINE_TASK.md` — status set to `IN_PROGRESS`.
- `docs/worker-notes/2026-08-03-j24d-plug-enable-scope-file.md` — this note.

## Decisions and assumptions

- Implemented duplicate JSON key rejection through custom `Deserialize` implementations on `PlugScopeRequest`, `CapabilityRequest`, and `PermissionRequest`. Each uses a `visit_map` with a `BTreeSet<String>` to track seen keys, rejecting duplicates at every nesting level. No new dependencies required.
- Used `serde_json::Deserializer::from_slice` with explicit `de.end()` call to reject trailing content after valid JSON (serde_json normally ignores trailing bytes).
- The `parse_scope_file` function returns `M3Error` with code `"store_io"` for file read failures so the upstream `run_enable` maps it to `OutcomeStatus::Unavailable` (exit 4) as required by the packet. All validation errors use `"scope_request_invalid"`.
- Chose to place the permission request types and `run_enable` in `plug_command.rs` rather than a separate `plug_scope.rs` module, since the types are tightly coupled to the enable command and a separate module did not add enforcement value.
- Reused `select_latest_transition`, `EnablementRecord::consistent_with`, `PdfOperationalScopeBinding::create`, and `EnablementStore::enable` unchanged.

## Evidence

- `cargo +1.89.0 fmt --all -- --check` — PASS (no diff)
- `cargo +1.89.0 test cli --locked` — 35 passed
- `cargo +1.89.0 test plug_command --locked` — 3 passed
- `cargo +1.89.0 test --test j24a_plug_inspect_cli --locked` — 3 passed
- `cargo +1.89.0 test --test j24b_plug_list_cli --locked` — 4 passed
- `cargo +1.89.0 test --test j24c_plug_disable_cli --locked` — 9 passed
- `cargo +1.89.0 test --test j24d_plug_enable_scope_file --locked` — 16 passed
- `cargo +1.89.0 test --all-targets --all-features --locked` — 906 passed, 5 documented `pwsh.exe not found` baseline failures
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — PASS
- `git diff --check` — PASS

## Discoveries

None.

## Remaining risks

None known within packet scope.

## Smallest next action

Lucy reviews the pushed branch and accepts or rejects J24D.

## References

- Branch: `opencode/j24d-plug-enable-scope-file`
- Base: `fb354dea734e7a2d37254a9cfbca4fd0daad5939`
- Final: `<pending commit>`
- Tests: `tethers-0.1/host-rust/tests/j24d_plug_enable_scope_file.rs`
- J24C: `tethers-0.1/host-rust/tests/j24c_plug_disable_cli.rs`
