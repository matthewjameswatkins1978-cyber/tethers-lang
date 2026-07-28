# J11 Packet 3: Compiled Host Rejection Verification

Task: `J11 packet 3 compiled host rejection verification`

Status: `IN_PROGRESS`

Owner: `Goose`

Base branch: `goose/tethers-final-evidence-skill`

Base SHA: `85d8bfa3c9e4816f10c8df5afe00bf150574ec58`

Branch: `goose/j11-production-rejection-verification`

Implementation SHA: `307d4f926231651a1d8d5bc69c18fc2e56a55e90`

Documentation SHA: `PENDING_DOCUMENTATION`

## Requested outcome

A debug-build-only compiled host diagnostic route that proves J11 rejection
behaviour through the real production admission components, with five focused
Rust tests and a PowerShell compiled-boundary script.

## Changes made

- `M tethers-0.1/host-rust/src/main.rs` – added `#[cfg(debug_assertions)]` diagnostic subcommand `event-admission-probe <SCENARIO>`, usage constant, dispatch in `main()`, production `run_event_admission_probe` function, five focused tests, and repaired ESC byte defect in doc comment.
- `A tethers-0.1/scripts/test-host-event-admission.ps1` – compiled-boundary test script for all four scenarios.
- `M docs/CURRENT_CLINE_TASK.md` – updated to J11 Packet 3.
- `A docs/worker-notes/2026-07-28-j11-production-rejection-verification.md` – this note.

## Decisions and assumptions

- The diagnostic boundary exercises production admission components directly with synthetic anchors. This proves the compiled route through the admission helper but does NOT prove normal engine-driven Result Anchor generation through the public runtime.
- A deterministic evaluation callback is used; it returns structured JSON and never enqueues new anchors.
- `#[cfg(debug_assertions)]` gates the entire diagnostic route. Release builds are free of it.
- No environment-variable activation, network, engine invocation, policy, replay, Trail, or filesystem state is used.

## Evidence

### Directly verified

- Compiled debug host route.
- Real production gate, queue, drain, and response mutation.
- Exact JSON output for all four scenarios.
- Duplicate and generation-9 rejection.
- Completed earlier follow-up preservation.
- Later sibling stopping.
- Generation-8 clean evaluation.
- Rejection omission on clean run.

### Not verified by this packet

- Normal engine-driven creation of Result Anchors through the public runtime.
- Legitimate runtime permission-scope establishment.
- Native replay-backed successful execution.
- Trail admission and rejection records.

### Test and script results

- Focused `j11_packet3_` tests: 5/5
- Existing `j11_` tests: 24/24 (19 existing + 5 new)
- `event_admission` tests: 15/15
- Full Rust suite: 512/512
- PowerShell `test-host-event-admission.ps1`: 4/4 scenarios PASS

### Warning baselines

- `cargo check`: 9 baseline, 0 new
- `cargo check --tests`: 4 baseline, 0 new
- `cargo clippy --all-targets --all-features`: baseline only, 0 new

## Discoveries

- Pre-existing ESC byte (`0x1B`) in `drain_result_event_queue` doc comment immediately before `valuate`. Repaired so the comment reads: "Each queued anchor is admitted before the evaluate callback is invoked."
- `ResultAnchor::with_event_id` is `#[cfg(test)]` only. The production diagnostic function constructs anchors and sets `event_id` directly on the public field.

## Remaining risks

The diagnostic boundary exercises the production admission helper directly with synthetic anchors. The normal engine route cannot yet produce Result Anchors. Packet 4 owns Trail admission and rejection records. J12 owns runnable configuration and legitimate scope establishment.

## Smallest next action

J11 Packet 4: Trail admission and rejection visibility, followed by final J11 acceptance.

## References

- Packet 1: `event_admission.rs` – EventAdmissionGate foundation
- Packet 2: `main.rs` – drain_result_event_queue, EventDrainOutcome, coordinator wiring
- SPEC.md: Tethers 0.1 language and protocol
- `tethers-0.1/host-rust/src/result_anchor.rs` – ResultAnchor type
