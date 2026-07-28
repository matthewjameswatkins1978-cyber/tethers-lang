# J11 Packet 3: Compiled Host Rejection Verification

Task: `J11 packet 3 compiled host rejection verification`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Status: `COMPLETE`

Owner: `Goose`

Base branch: `goose/tethers-final-evidence-skill`

Base commit: `85d8bfa3c9e4816f10c8df5afe00bf150574ec58`

Branch: `goose/j11-production-rejection-verification`

Implementation checkpoint: `307d4f926231651a1d8d5bc69c18fc2e56a55e90`

Documentation checkpoint: `35a67f445c1c64d7a5d8c657d4c74786d82fbf36`

Pre-correction finalisation SHA: `243b4249b684cd302490c2fb8875d921876e4a6f`

Final evidence correction: current branch HEAD reported in the final acceptance report

## Requested outcome

A debug-build-only compiled host diagnostic route that proves J11 rejection
behaviour through the real production admission components, with one shared
`build_event_admission_probe_response` function used by both the compiled
subcommand and all four behavioural Rust tests. Five focused Rust tests and a
PowerShell compiled-boundary script with positive and negative CLI checks.

## Changes made

- `M tethers-0.1/host-rust/src/main.rs` — added `#[cfg(debug_assertions)]` shared `build_event_admission_probe_response` function and thin `run_event_admission_probe` CLI wrapper with exact argument-count validation; updated dispatch in `main()`; five focused tests calling the shared function; removed duplicated `run_event_admission_probe_for_test`; repaired ESC byte defect in doc comment.
- `A tethers-0.1/scripts/test-host-event-admission.ps1` — compiled-boundary test script for all four scenarios plus three negative CLI checks (missing scenario, unknown scenario, extra argument).
- `M docs/CURRENT_CLINE_TASK.md` — updated to J11 Packet 3 with evidence-correction details.
- `A docs/worker-notes/2026-07-28-j11-production-rejection-verification.md` — this note.

## Decisions and assumptions

- The diagnostic boundary exercises production admission components directly with synthetic anchors. This proves the compiled route through the admission helper but does NOT prove normal engine-driven Result Anchor generation through the public runtime.
- A deterministic evaluation callback is used; it returns structured JSON and never enqueues new anchors.
- `#[cfg(debug_assertions)]` gates the entire diagnostic route. Release builds are free of it.
- No environment-variable activation, network, engine invocation, policy, replay, Trail, or filesystem state is used.
- One shared `build_event_admission_probe_response` function. All four behavioural tests call it directly. No separate test helper. No duplicated scenario construction, queue drain, or response assembly.

## Evidence

### Scenario table

| Scenario | First anchor | Second | Third | Expected |
|---|---|---|---|---|
| `duplicate-initial` | `evt/root` gen 1 | `evt/later` gen 1 | — | Duplicate rejection (`kind: duplicate_event_id`, `processing: stopped`); no `follow_up_evaluations`; remaining `["evt/later"]` |
| `duplicate-sibling` | `evt/first` gen 1 | `evt/first` gen 1 | `evt/later` gen 1 | First evaluated (status `evaluated`, event_id `evt/first`, gen 1); duplicate rejected; remaining `["evt/later"]` |
| `causal-depth` | `evt/deep` gen 9 | `evt/later` gen 1 | — | Depth rejection (`kind: causal_depth_exceeded`, `maximum_generation: 8`, `processing: stopped`); no `follow_up_evaluations`; remaining `["evt/later"]` |
| `clean` | `evt/a` gen 1 | `evt/b` gen 8 | — | Two FIFO evaluations (both `evaluated`); no `event_admission_rejection`; empty remaining |

### Negative CLI checks

| Case | Args | Expected |
|---|---|---|
| Missing scenario | `event-admission-probe` | Non-zero exit; usage text |
| Unknown scenario | `event-admission-probe nonexistent` | Non-zero exit; usage text |
| Extra argument | `event-admission-probe clean extra` | Non-zero exit; usage text |

### Directly verified

- Compiled debug host route through `event-admission-probe`
- Real production `EventAdmissionGate`, `ResultEventQueue`, `drain_result_event_queue`, `EventDrainOutcome::apply_to_response`
- Exact JSON output for all four scenarios
- Duplicate and generation-9 rejection with exact field shapes
- Completed follow-up preservation
- Later sibling stopping
- Generation-8 clean evaluation
- Rejection omission on clean run
- Process-boundary PowerShell positive and negative checks
- Unknown scenario, missing scenario, and extra argument all fail closed

### Not verified by this packet

- Normal engine-driven creation of Result Anchors through the public runtime
- Legitimate runtime permission-scope establishment
- Native replay-backed successful execution
- Trail admission and rejection records

### Shared-function test boundary

The first four behavioural `j11_packet3_` tests call `build_event_admission_probe_response(scenario).unwrap()` directly — the same function used by the compiled `event-admission-probe` subcommand. No separate test helper or duplicated implementation exists.

### Test and script results

- Focused `j11_packet3_` tests: 5/5
- Existing `j11_` tests: 24/24 (19 existing + 5 new)
- `event_admission` tests: 15/15
- Full Rust suite: 512/512
- PowerShell `test-host-event-admission.ps1`: 4 scenarios PASS, 3 negative CLI PASS
- Task packet checker: PASS
- Dune build: PASS
- Control character scan: no unexpected characters in all four authorised files

### Warning baselines

- `cargo check`: 9 baseline, 0 new
- `cargo check --tests`: 4 baseline, 0 new
- `cargo clippy --all-targets --all-features`: baseline only, 0 new

## Discoveries

- Pre-existing ESC byte (`0x1B`) in `drain_result_event_queue` doc comment immediately before `valuate`. Repaired so the comment reads: "Each queued anchor is admitted before the evaluate callback is invoked."
- `ResultAnchor::with_event_id` is `#[cfg(test)]` only. The diagnostic function constructs anchors and sets `event_id` directly on the public field.
- The initial Packet 3 implementation duplicated scenario construction, queue drain, and response assembly between a test-only helper and the production function. The evidence correction unified both behind one shared function.

## Remaining risks

The diagnostic boundary exercises the production admission helper directly with synthetic anchors. The normal engine route cannot yet produce Result Anchors. Packet 4 owns Trail admission and rejection records. J12 owns runnable configuration and legitimate scope establishment.

## Smallest next action

J11 Packet 4: Trail admission and rejection visibility, followed by final J11 acceptance.

## References

- Packet 1: `event_admission.rs` — EventAdmissionGate foundation
- Packet 2: `main.rs` — drain_result_event_queue, EventDrainOutcome, coordinator wiring
- SPEC.md: Tethers 0.1 language and protocol
- `tethers-0.1/host-rust/src/result_anchor.rs` — ResultAnchor type

## Checks not run

None. All required checks executed.
