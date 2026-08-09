# Current Implementation Task

Control contract: `1`
Task packet: `F8-D4 — Legacy Event Admission Trail Probe Cleanup`
Owner: `OpenCode`
Status: `IN_PROGRESS`
Task colour: `Green`
Route: `OpenCode removes D4 (run_event_admission_trail_probe) dead legacy parser`
Worker note: `docs/worker-notes/2026-08-09-f8-d4-event-admission-trail-probe-cleanup.md`
Base branch: `foundation/f8-d3-event-admission-probe-cleanup`
Base commit: `9a7dc9ea46fd82c25ab0fbf58e468ab8fc15412a`
Implementation branch: `foundation/f8-d3-event-admission-probe-cleanup`
Implementation checkpoint: `f3b646f`
Rust change class: `RUST`

## Objective

Resolve production dead-code warning D4: the unused `run_event_admission_trail_probe`
legacy manual argument parser from `application.rs`.

## Relevant background and existing behaviour

`run_event_admission_trail_probe` (line 488, `#[cfg(debug_assertions)]`) is a legacy
manual argument parser for the `event-admission-trail-probe` debug subcommand. It
manually validates argument count (3), the `"event-admission-trail-probe"` token,
and absolute path requirement. It calls the shared function
`build_event_admission_trail_probe_response` and writes a JSONL probe trail to disk.

The live Clap-based dispatch at line 691 uses `run_event_admission_trail_probe_clap`
(line 781), which takes pre-parsed `mode` and `trail_path` parameters from Clap and
calls the same shared `build_event_admission_trail_probe_response`. The Clap-based
wrapper leverages Clap's built-in argument validation.

Tests J13A (lines 8524-8555) call `run_event_admission_trail_probe_clap` directly,
covering relative-path rejection and absolute-path acceptance. No tests exist for
the dead legacy manual parser.

Relationship to D3: independent. D3 was `run_event_admission_probe` (event probe,
not trail). D4 is `run_event_admission_trail_probe` (trail probe). They share
a pattern but are separate dead wrappers. D4 was correctly resolved independently.

Classification: **DEAD** — zero callers, zero test references. The Clap-based
`EventAdmissionTrailProbe` command handles all argument parsing. The shared logic
`build_event_admission_trail_probe_response` and live tests remain intact.

## Required behaviour

1. Delete `run_event_admission_trail_probe` function (old lines 487-506).
2. Preserve `run_event_admission_trail_probe_clap` and its live dispatch.
3. Preserve `build_event_admission_trail_probe_response` shared logic.
4. Preserve `EVENT_ADMISSION_TRAIL_PROBE_USAGE` constant (still used by live paths).
5. Preserve J13A tests (already use `_clap` directly).
6. No test assertions to remove (none targeted the dead wrapper).
7. Run `cargo fmt` on changed files.
8. Confirm D4 warning is gone from `cargo check --all-targets --all-features --locked`.
9. Confirm D5-D15 warnings remain otherwise unchanged (11 lib warnings).
10. Run full `just verify-agent` once.

## Frozen decisions and invariants

- Do not resolve or suppress D5-D15.
- Do not add `#[allow(dead_code)]` suppression.
- Do not rename, refactor, or opportunistically clean code outside D4.
- Preserve all runtime, event admission, Trail, replay/recovery, CLI behaviour.
- `build_event_admission_trail_probe_response` must remain unchanged.
- `run_event_admission_trail_probe_clap` live dispatch must remain unchanged.
- `EVENT_ADMISSION_TRAIL_PROBE_USAGE` constant survives (used by live paths).
- Durable FileTrail behaviour preserved.

## Acceptance criteria

1. D4 warning absent from `cargo check` lib target.
2. `run_event_admission_trail_probe` function removed from `application.rs`.
3. `run_event_admission_trail_probe_clap` still present and dispatch-connected.
4. `build_event_admission_trail_probe_response` still present.
5. `EVENT_ADMISSION_TRAIL_PROBE_USAGE` still present.
6. J13A tests pass unchanged.
7. No replacement suppression added.
8. D5-D15 warnings remain otherwise unchanged.
9. No production semantics changed.
10. `cargo fmt` only touches `application.rs`.
11. `just verify-agent` passes once.
12. Branch pushed and local == remote.

## Required verification

- `rg "run_event_admission_trail_probe\b" --type rust` → zero matches
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`
- `cargo check --all-targets --all-features --locked`
- `cargo clippy --all-targets --all-features --locked`
- `cargo test --all-targets --all-features --locked`
- `git diff --check`
- Packet checker
- `just verify-agent` (full regression)

## Relevant components

### AUTHORISED PATHS
- `tethers-0.1/host-rust/src/application.rs` — remove dead function

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-d4-event-admission-trail-probe-cleanup.md`

## Forbidden changes

- No D5-D15 resolution or suppression
- No OCaml source changes
- No other Rust source changes outside authorised paths
- No `#[allow(...)]` suppression additions
- No removing `build_event_admission_trail_probe_response`
- No removing `run_event_admission_trail_probe_clap`
- No removing `EVENT_ADMISSION_TRAIL_PROBE_USAGE`
- No FileTrail behaviour changes

## Stop conditions

STOP if removing `run_event_admission_trail_probe` would break any live code path.
STOP if rustfmt touches any file other than `application.rs`.
STOP if verification fails.

## Expected pre-existing changes

None.
