# Worker Note

Task: `J24B - Read-only Plug list CLI`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `IN_PROGRESS`

Base commit: `13f6a3caffa00904f6357c7975a8a0937a6c2d5c`

Implementation checkpoint: `pending final integration commit`

## Requested outcome

Expose `plug list --host-data-root <ABSOLUTE_PATH>` as a strictly observational
CLI command. It reads the existing installed registry and enablement authority,
derives current state, and never mutates lifecycle state.

## Changes made

- Added existing-only `StoreRoot`, `InstalledPlugRegistry`, and
  `EnablementStore` opening seams.
- Added the public `plug list` clap route, application routing, stable envelope,
  fail-closed layout and cross-record validation, and deterministic ordering.
- Added `tests/j24b_plug_list_cli.rs` for empty, missing, partial, real PDF
  install/enable/disable lifecycle state, sequence ordering, unknown identity,
  cross-record mismatch, output filtering, stable ordering, process exit parity,
  and recursive filesystem non-mutation evidence.

## Decisions and assumptions

- Existing validators and chain validation remain the sole record authorities.
- Installed records without an enablement transition report `disabled`.
- Store I/O remains `unavailable`; malformed or conflicting evidence remains
  `invalid_data`.

## Evidence

- Branch created from current `origin/main`; base commit is an ancestor.
- `cargo +1.89.0 fmt --all -- --check` passed.
- `cargo +1.89.0 test cli --locked` passed: 33 tests.
- `cargo +1.89.0 test plug_command --locked` passed: 3 tests.
- `cargo +1.89.0 test --test j24a_plug_inspect_cli --locked` passed: 3 tests.
- `cargo +1.89.0 test --test j24b_plug_list_cli --locked` passed: 4 tests.
- Full suite passed 904 tests with five documented `pwsh.exe not found`
  execution-environment baseline failures.
- `git diff --check` passed before completion.

Correction evidence:

- Transition selection now explicitly retains the greatest `sequence`,
  independent of UUID filename order.
- `cargo +1.89.0 test plug_command --locked` passed with the sequence-order
  regression test.
- `cargo +1.89.0 test --test j24b_plug_list_cli --locked` passed.

Integration evidence:

- The compiled binary lists the real `tethers.pdf-tools` Plug as disabled,
  enabled after a valid transition, and disabled after a valid disable
  transition.
- Enablement UUID filenames are deliberately ordered opposite to transition
  sequence; the compiled command still selects the sequence-two disablement.
- Re-signed structurally valid unknown-identity and provider-version mismatch
  records fail closed as `invalid_data` with exit code 3.
- Every invocation snapshots all lifecycle entries and file SHA-256 digests;
  before and after snapshots are equal, and process exit equals envelope exit.
- Output-key and forbidden-field assertions cover paths, scope, authority,
  trust, approval, conformance, transition history, and internal paths.
- A compiled-binary ordering fixture proves Plug and capability ordering.

## Discoveries

None.

## Remaining risks

The full suite retains five documented environment failures because `pwsh.exe`
is unavailable. No J24B-specific failure remains.

## Smallest next action

After the final integration commit, return the packet to `COMPLETE`, record its
full SHA, and push the branch for Lucy's final review.

## References

- Branch: `opencode/j24b-plug-list-cli`
- J24A base: `13f6a3caffa00904f6357c7975a8a0937a6c2d5c`
- Integration test: `tethers-0.1/host-rust/tests/j24b_plug_list_cli.rs`
