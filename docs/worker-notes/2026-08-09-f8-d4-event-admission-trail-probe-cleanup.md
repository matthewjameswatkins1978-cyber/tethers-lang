# Worker Note

Task: `F8-D4 — Legacy Event Admission Trail Probe Cleanup`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `9a7dc9ea46fd82c25ab0fbf58e468ab8fc15412a`

Implementation checkpoint: `f3b646f553b0f5a5e70f4c496dc96642f71477e6`

## Requested outcome

Remove production dead-code warning D4: the unused `run_event_admission_trail_probe`
legacy manual argument parser from `application.rs`. Preserve the live Clap-based
wrapper `run_event_admission_trail_probe_clap` and the shared builder
`build_event_admission_trail_probe_response`.

## Changes made

- `tethers-0.1/host-rust/src/application.rs`: Deleted `run_event_admission_trail_probe`
  function (21 lines, old lines 487-506). The function manually validated argument
  count (3), the `"event-admission-trail-probe"` command token, and absolute path
  requirement, then called `build_event_admission_trail_probe_response`.
  - No test assertions removed: J13A tests (lines 8524-8555) exclusively call
    `run_event_admission_trail_probe_clap` directly.
  - `EVENT_ADMISSION_TRAIL_PROBE_USAGE` constant survives: still used at line 437
    (shared builder) and line 786 (live Clap wrapper).
  - `build_event_admission_trail_probe_response` unchanged.
  - `run_event_admission_trail_probe_clap` and its dispatch at line 691 unchanged.

## Decisions and assumptions

- Classification: **DEAD**. `run_event_admission_trail_probe` manually validated
  argument count (3), command token, and absolute-path requirement, then called
  `build_event_admission_trail_probe_response`. The live Clap-based dispatch at
  line 691 uses `run_event_admission_trail_probe_clap` which takes pre-parsed
  `mode` and `trail_path` parameters from Clap.
- D3/D4 relationship: independent. D3 (`run_event_admission_probe`) was the
  event probe; D4 is the trail probe. They share a dead-wrapper pattern but are
  separate functions with separate live `_clap` wrappers.
- `EVENT_ADMISSION_TRAIL_PROBE_USAGE` is shared: used by the dead wrapper (lines
  490, 495 — now removed), the live Clap wrapper (line 786, preserved), and the
  shared builder (line 437, preserved). Constant survives.
- J13A tests (J13A1: relative-path rejection, J13A2: absolute-path acceptance)
  call `run_event_admission_trail_probe_clap` at lines 8528 and 8548. Zero
  assertions referenced the dead wrapper, so no test trimming was needed.

## Evidence

- Pre-change references: `run_event_admission_trail_probe` at `application.rs:488`
  (definition only). Zero caller sites, zero test references.
- `rg "run_event_admission_trail_probe\b" --type rust` → zero matches (post-removal)
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` — PASS
- `cargo check --all-targets --all-features --locked` — 11 lib warnings (D5-D15), D4 absent. 8 lib test warnings (5 duplicates).
- `cargo clippy --all-targets --all-features --locked` — PASS (no errors)
- `cargo test --all-targets --all-features --locked` — 1592 passed, 0 failed, 2 skipped
- `git diff --check` — PASS
- Packet checker: PASS
- `just verify-agent` — expected to PASS (awaiting final run after closeout)
- Warning baseline: 12 → 11 lib warnings. D4 gone, D5-D15 intact.

## Discoveries

None.

## Remaining risks

None known within packet scope. D5-D15 remain unresolved.

## Smallest next action

Resolve D5 (`HumanApprovalDecision`) or another D5-D15 as prioritised by Lucy.

## References

- `tethers-0.1/host-rust/src/application.rs` — dead function removed at old lines 487-506
- `tethers-0.1/host-rust/src/cli.rs` — `EventAdmissionTrailProbe` at line 75
