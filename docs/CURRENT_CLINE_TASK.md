# Current Implementation Task

Control contract: `1`

Task: `J11 packet 4 durable event-admission Trail and final implementation closure`

Status: `COMPLETE`

Task colour: `Green`

Owner: `Goose`

Route: `Goose - J11 packet 4 in local worktree`

Worker note: `docs/worker-notes/2026-07-28-j11-event-trail-final.md`

Base branch: `goose/j11-production-rejection-verification`

Base commit: `a87cb49dd526f66cbbc84e85ac18be201cf3f7a7`

Branch: `goose/j11-event-trail-final`

## Objective

Record every J11 event-admission decision in the existing durable append-only Trail
before evaluation continues or stops.

## Authorised files

- `tethers-0.1/host-rust/src/dispatch.rs` — EventAdmissionEntry, Trail extension, FileTrail + RecordingTrail impls
- `tethers-0.1/host-rust/src/main.rs` — admission-entry mapper, drain update, trail probe, initial admission, tests
- `tethers-0.1/scripts/test-host-result-follow-up.ps1` — updated to expect one admission record
- `tethers-0.1/scripts/test-host-event-admission-trail.ps1` — new compiled-boundary trail verification
- `docs/CURRENT_CLINE_TASK.md` — this file
- `docs/worker-notes/2026-07-28-j11-event-trail-final.md` — implementation evidence

## Forbidden changes

- `event_admission.rs`, `event_queue.rs`, `result_anchor.rs` — not modified
- `Cargo.toml`, `Cargo.lock` — not modified
- OCaml, protocol fixtures, existing Trail entry semantics — not modified
- `docs/ROAD_TO_0_2.md` — not modified
- No new dependencies

## Key invariants

- EventAdmissionEntry schema frozen: kind, event_id, event_name, source, correlation_id,
  causation_id?, generation, processing, reason_code?, maximum_generation?, timestamp_unix_ms
- Accepted: kind=event_admitted, processing=continued, reason_code/maximum_generation omitted
- Duplicate rejection: kind=event_rejected, reason_code=duplicate_event_id, processing=stopped
- Depth rejection: kind=event_rejected, reason_code=causal_depth_exceeded, maximum_generation=8
- Gate admission before durable append; evaluation never before successful durable append
- Four durable scenarios: duplicate-initial (2 records), duplicate-sibling (3), causal-depth (2), clean (3)
- Ten focused tests: j11_packet4_ prefix, 10/10
- Packet 3: all 5 tests preserved, event_admission.rs untouched
- Release builds: neither diagnostic route present

## Warning baseline

- `cargo check`: 9 baseline, zero new
- `cargo check --tests`: 4 baseline, zero new
- clippy: baseline only, zero new

## J12/public-runtime boundary

The normal engine-driven route still cannot currently generate successful follow-up
Result Anchors because legitimate scope establishment belongs to J12.

## Required verification

```powershell
cargo fmt --check
cargo check && cargo check --tests
cargo test j11_packet3_ && cargo test j11_packet4_ && cargo test j11_ && cargo test event_admission && cargo test
cargo clippy --all-targets --all-features
cargo build && cargo build --release
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
pwsh -NoProfile -File tethers-0.1/scripts/check-fixtures.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-engine.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-mcp-transcripts.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-denial.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-execution-failure.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-result-follow-up.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-event-admission.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-event-admission-trail.ps1
pwsh -NoProfile -File tethers-0.1/scripts/demo.ps1
opam exec -- dune build
```

This is the final J11 implementation candidate, pending Lucy's acceptance.
