# Current Implementation Task

Control contract: `1`
Task packet: `F8-D3 — Legacy run_event_admission_probe Cleanup`
Owner: `OpenCode`
Status: `IN_PROGRESS`
Task colour: `Green`
Route: `OpenCode removes D3 (run_event_admission_probe) dead legacy parser`
Worker note: `docs/worker-notes/2026-08-09-f8-d3-event-admission-probe-cleanup.md`
Base branch: `foundation/f8-d2-parse-provision-args-cleanup`
Base commit: `f674ac669f38c3557fff43af873ba9bbd7b5bbd0`
Implementation branch: `foundation/f8-d3-event-admission-probe-cleanup`
Implementation checkpoint: `TBD`
Rust change class: `RUST`

## Objective

Remove production dead-code warning D3: the unused `run_event_admission_probe`
legacy manual argument parser from `application.rs`.

## Relevant background and existing behaviour

`run_event_admission_probe` (line 364, `#[cfg(debug_assertions)]`) is a legacy
manual argument parser for the `event-admission-probe` debug subcommand. It
manually validates argument count and the `"event-admission-probe"` token.
It calls the shared function `build_event_admission_probe_response`.

The live Clap-based dispatch at line 685 uses `run_event_admission_probe_clap`
(line 783), which takes a pre-parsed `mode` parameter from Clap and calls the
same shared `build_event_admission_probe_response`. The Clap-based wrapper
leverages Clap's built-in argument validation.

The test `j11_packet3_invalid_scenario_and_argument_counts_fail_closed` at
line 8243 has four assertions:
1. `build_event_admission_probe_response("nonexistent")` — tests live shared logic (KEEP)
2-4. `run_event_admission_probe(...)` with wrong shapes — tests dead parser (REMOVE)

Classification: **DEAD** — genuinely unused in production. The Clap-based
`EventAdmissionProbe` command handles all argument parsing. The shared logic
`build_event_admission_probe_response` remains tested.

## Required behaviour

1. Delete `run_event_admission_probe` function (lines 363-372).
2. Remove the 3 dead-parser assertions from the test, keeping only the
   `build_event_admission_probe_response("nonexistent")` assertion.
3. Optionally add Clap parse tests for `EventAdmissionProbe` argument shapes.
4. Run `cargo fmt` on changed files.
5. Confirm D3 warning is gone from `cargo check --all-targets --all-features --locked`.
6. Confirm D4-D15 warnings remain otherwise unchanged.
7. Run full `just verify-agent` once.

## Frozen decisions and invariants

- Do not resolve or suppress D4-D15.
- Do not add `#[allow(dead_code)]` suppression.
- Do not rename, refactor, or opportunistically clean code outside D3.
- Preserve all runtime, event admission, Trail, replay/recovery, CLI behaviour.
- `build_event_admission_probe_response` must remain tested and unchanged.
- `run_event_admission_probe_clap` live dispatch must remain unchanged.

## Acceptance criteria

1. D3 warning absent from `cargo check` lib target.
2. `run_event_admission_probe` function removed from `application.rs`.
3. Dead-parser test assertions removed; shared-logic assertion retained.
4. No replacement suppression added.
5. D4-D15 warnings remain otherwise unchanged.
6. No production semantics changed.
7. `cargo fmt` only touches `application.rs`.
8. `just verify-agent` passes once.
9. Branch pushed and local == remote.

## Required verification

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`
- `cargo check --all-targets --all-features --locked`
- `cargo clippy --all-targets --all-features --locked`
- `cargo test --all-targets --all-features --locked`
- `git diff --check`
- Packet checker
- `just verify-agent` (full regression)

## Relevant components

### AUTHORISED PATHS
- `tethers-0.1/host-rust/src/application.rs` — remove dead function, trim test

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-d3-event-admission-probe-cleanup.md`

## Forbidden changes

- No D4-D15 resolution or suppression
- No OCaml source changes
- No other Rust source changes outside authorised paths
- No `#[allow(...)]` suppression additions

## Stop conditions

STOP if removing `run_event_admission_probe` would break any live code path.
STOP if rustfmt touches any file other than `application.rs`.
STOP if verification fails.

## Expected pre-existing changes

None.
