# Worker Note

Task: `F8-D3 — Legacy run_event_admission_probe Cleanup`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `f674ac669f38c3557fff43af873ba9bbd7b5bbd0`

Implementation checkpoint: `29f0ac6f45829d5148997c1147d6ed8fe54722c7`

## Requested outcome

Remove production dead-code warning D3: the unused `run_event_admission_probe`
legacy manual argument parser from `application.rs`. Preserve the live
shared core `build_event_admission_probe_response` and its test coverage.

## Changes made

- `tethers-0.1/host-rust/src/application.rs`: Deleted `run_event_admission_probe`
  function (10 lines, old lines 364-372). Trimmed the test
  `j11_packet3_invalid_scenario_and_argument_counts_fail_closed` — removed the
  3 dead-parser assertions (missing scenario, extra argument, wrong command token)
  while retaining the single live shared-logic assertion
  `build_event_admission_probe_response("nonexistent").is_err()`.

## Decisions and assumptions

- Classification: **DEAD**. `run_event_admission_probe` manually validated
  argument count and command token, then called `build_event_admission_probe_response`.
  The live Clap-based dispatch at line 685 uses `run_event_admission_probe_clap`
  which takes a pre-parsed `mode` parameter directly from Clap.
- `EVENT_ADMISSION_PROBE_USAGE` constant survives — it's still used by
  `build_event_admission_probe_response` at line 321 for unknown-scenario errors.
- `build_event_admission_probe_response` remains fully tested with 5 test
  assertions across multiple test functions. The existing Clap integration test
  `j13a_provision_replay_hidden_accessible` (in tests) covers the live dispatch path.
- D4-D15 remain entirely unchanged (12 lib warnings).

## Evidence

- Pre-change references: `run_event_admission_probe` at `application.rs:364`
  (definition), `application.rs:685` (dispatch uses `_clap` variant),
  `application.rs:783` (Clap wrapper), and 3 test call sites at old lines 8251,
  8262, 8269.
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` — PASS
- `cargo check --all-targets --all-features --locked` — 12 lib warnings (D4-D15), D3 absent. 9 lib test warnings (7 duplicates).
- `cargo clippy --all-targets --all-features --locked` — PASS (no errors)
- `cargo test --all-targets --all-features --locked` — 1592 passed, 0 failed, 2 skipped
- `git diff --check` — PASS
- Packet checker: PASS
- `just verify-agent` — PASS (1592/1592, nextest 38.8s)
- `rg run_event_admission_probe[^_]` in Rust source — Zero matches

## Publication evidence

Branch `foundation/f8-d3-event-admission-probe-cleanup` pushed to `origin`.

## Discoveries

None.

## Remaining risks

None known within packet scope. D4-D15 remain unresolved.

## Smallest next action

Resolve D4 (`run_event_admission_trail_probe`) as prioritised by Lucy.

## References

- `tethers-0.1/host-rust/src/application.rs` — function and test assertions removed
- `tethers-0.1/host-rust/src/cli.rs` — `EventAdmissionProbe` at line 68
