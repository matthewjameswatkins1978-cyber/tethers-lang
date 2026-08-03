# Worker Note

Task: `J24B - Read-only Plug list CLI`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `13f6a3caffa00904f6357c7975a8a0937a6c2d5c`

Implementation checkpoint: `pending correction commit`

## Requested outcome

Expose `plug list --host-data-root <ABSOLUTE_PATH>` as a strictly observational
CLI command. It reads the existing installed registry and enablement authority,
derives current state, and never mutates lifecycle state.

## Changes made

- Added existing-only `StoreRoot`, `InstalledPlugRegistry`, and
  `EnablementStore` opening seams.
- Added the public `plug list` clap route, application routing, stable envelope,
  fail-closed layout and cross-record validation, and deterministic ordering.
- Added `tests/j24b_plug_list_cli.rs` for empty, missing, partial, process exit,
  and filesystem non-mutation evidence.

## Decisions and assumptions

- Existing validators and chain validation remain the sole record authorities.
- Installed records without an enablement transition report `disabled`.
- Store I/O remains `unavailable`; malformed or conflicting evidence remains
  `invalid_data`.

## Evidence

- Branch created from current `origin/main`; base commit is an ancestor.
- `cargo +1.89.0 fmt --all -- --check` passed.
- `cargo +1.89.0 test cli --locked` passed: 33 tests.
- `cargo +1.89.0 test plug_command --locked` passed: 2 tests.
- `cargo +1.89.0 test --test j24a_plug_inspect_cli --locked` passed: 3 tests.
- `cargo +1.89.0 test --test j24b_plug_list_cli --locked` passed: 2 tests.
- Full suite passed 903 tests with five documented `pwsh.exe not found`
  execution-environment baseline failures.
- `git diff --check` passed before completion.

Correction evidence:

- Transition selection now explicitly retains the greatest `sequence`,
  independent of UUID filename order.
- `cargo +1.89.0 test plug_command --locked` passed with the sequence-order
  regression test.
- `cargo +1.89.0 test --test j24b_plug_list_cli --locked` passed.

## Discoveries

None.

## Remaining risks

None known within packet scope.

## Smallest next action

Lucy performs final review of the pushed branch.

## References

- Branch: `opencode/j24b-plug-list-cli`
- J24A base: `13f6a3caffa00904f6357c7975a8a0937a6c2d5c`
- Integration test: `tethers-0.1/host-rust/tests/j24b_plug_list_cli.rs`
