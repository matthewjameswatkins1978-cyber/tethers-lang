# Worker Note

Task: `J24C - Explicit Plug disable CLI`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `726c6aa780c6809fce32de39427200217cbad12f`

Implementation checkpoint: `aac395a522e9d90573870a7f53e00b4fb075a4d7`

## Requested outcome

Add `plug disable --host-data-root <ABSOLUTE_PATH> --installed-id <UUID>` as
the first public lifecycle mutation command. It must reuse the accepted installed
and enablement authorities, fail closed on inconsistent evidence, and change
nothing except one new immutable enablement record.

## Changes made

- `tethers-0.1/host-rust/src/cli.rs` — added `Disable` variant to `PlugCommand`
  with `--host-data-root` and `--installed-id` options; added 11 CLI syntax tests.
- `tethers-0.1/host-rust/src/application.rs` — routed `PlugCommand::Disable` to
  `plug_command::run_disable`.
- `tethers-0.1/host-rust/src/enablement.rs` — added `EnablementRecord::consistent_with`
  method for cross-record consistency validation shared by list and disable.
- `tethers-0.1/host-rust/src/plug_command.rs` — extracted `select_latest_transition`
  helper; refactored `run_list` to use `consistent_with`; added `run_disable` with
  full validate-before-mutate logic.
- `tethers-0.1/host-rust/tests/j24c_plug_disable_cli.rs` — 9 integration tests
  covering success, second-attempt failure, never-enabled, unknown ID,
  cross-record drift, forked chain, missing/partial roots, CLI validation, and
  envelope shape.
- `docs/CURRENT_CLINE_TASK.md` — status transitions and required section titles.
- `docs/worker-notes/2026-08-03-j24c-plug-disable-cli.md` — this note.

## Decisions and assumptions

- Chose to add `EnablementRecord::consistent_with` and `select_latest_transition`
  as shared helpers rather than a separate lifecycle module, since the
  reconciliation surface is narrow (one method, one function) and the module
  boundary did not add enforcement.
- Reused `EnablementStore::open_existing` for disable — the mutation itself calls
  `EnablementStore::disable` which creates one new JSON file through the existing
  `StoreRoot::create_json` authority.
- The CLI authority is hardcoded as `tethers-reference-host-cli`; no caller field.

## Evidence

- `cargo +1.89.0 fmt --all -- --check` — PASS (no diff)
- `cargo +1.89.0 test cli --locked` — 34 passed
- `cargo +1.89.0 test plug_command --locked` — 3 passed
- `cargo +1.89.0 test --test j24a_plug_inspect_cli --locked` — 3 passed
- `cargo +1.89.0 test --test j24b_plug_list_cli --locked` — 4 passed
- `cargo +1.89.0 test --test j24c_plug_disable_cli --locked` — 9 passed
- `cargo +1.89.0 test --all-targets --all-features --locked` — 905 passed,
  5 documented `pwsh.exe not found` baseline failures
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — PASS
- `git diff --check` — PASS

## Discoveries

- The Lucy-authored J24C packet on origin/main used section title "Relevant
  accepted behaviour" which the packet checker did not recognise; changed to
  "Relevant background and existing behaviour" mechanically.
- The J24C packet also omitted "Relevant components" and "Frozen decisions and
  invariants" sections required by the checker; added with appropriate content.

## Remaining risks

None known within packet scope.

## Smallest next action

Lucy reviews the pushed branch and accepts or rejects J24C.

## References

- Branch: `opencode/j24c-plug-disable-cli`
- Base: `726c6aa780c6809fce32de39427200217cbad12f`
- Final: `aac395a522e9d90573870a7f53e00b4fb075a4d7`
- Tests: `tethers-0.1/host-rust/tests/j24c_plug_disable_cli.rs`
- J24B: `tethers-0.1/host-rust/tests/j24b_plug_list_cli.rs`
