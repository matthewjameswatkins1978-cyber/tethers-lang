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

## Relevant background and existing behaviour

J11 Packet 3 introduced `EventAdmissionGate` (in-memory deduplication + causal-depth guard,
max generation 8) and `ResultEventQueue` for follow-up anchors. Admission decisions were
made in memory but not durably recorded. The `Trail` trait (`dispatch.rs`) supported
`append_action` entries but had no event-admission variant. The existing `FileTrail` uses
`flush` + `sync_data` for durability; `RecordingTrail` captures entries for tests.

J10 provided the serial follow-up coordinator with 20 unit tests. J09 established the
`FileReplayAuthority` ledger model. Packet 4 is the final J11 implementation closure.

## Required behaviour

1. Define `EventAdmissionEntry` in `dispatch.rs` with the frozen schema: kind, event_id,
   event_name, source, correlation_id, causation_id?, generation, processing, reason_code?,
   maximum_generation?, timestamp_unix_ms.
2. Extend the sealed `Trail` trait with `append_event_admission` and implement for
   `FileTrail` (flush + sync_data) and `RecordingTrail` (capture).
3. In `main.rs`, create `build_event_admission_entry` (pure admission-decision mapper)
   and `now_unix_ms` (wall-clock helper).
4. Update `drain_result_event_queue` to accept `now_unix_ms` and `record_admission`
   callbacks; record admission durably before evaluating each queued anchor.
5. In the normal host route, record the initial external admission durably before
   `process_one_event`.
6. Add a debug-only `event-admission-trail-probe` command that writes actual durable
   Trail files for four scenarios (duplicate-initial, duplicate-sibling, causal-depth,
   clean) and prints the same JSON response format as Packet 3.
7. Preserve Packet 3 command and tests unchanged.
8. Update `test-host-result-follow-up.ps1` to expect exactly one admission record.
9. Create `test-host-event-admission-trail.ps1` that builds debug + release, runs all
   scenarios, checks trail records, negative CLI checks, and release diagnostic absence.
10. Add exactly ten `j11_packet4_` Rust tests covering accepted/rejected fields,
    RecordingTrail ordering, and write-failure behaviour.

## Relevant components

- `tethers-0.1/host-rust/src/dispatch.rs` - Trail trait, EventAdmissionEntry, FileTrail, RecordingTrail
- `tethers-0.1/host-rust/src/main.rs` - mapper, drain, probes, initial admission, tests
- `tethers-0.1/scripts/test-host-result-follow-up.ps1` - updated follow-up smoke
- `tethers-0.1/scripts/test-host-event-admission-trail.ps1` - new compiled-boundary verification
- `event_admission.rs` - pure gate, unchanged
- `event_queue.rs` - queue, unchanged

## Frozen decisions and invariants

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
- Warning baseline: cargo check 9, cargo check --tests 4, clippy baseline only

## Acceptance criteria

1. All 522 Rust tests pass (including 5 j11_packet3_, 10 j11_packet4_, 15 event_admission).
2. `cargo fmt --check` reports no diffs.
3. `cargo clippy --all-targets --all-features` produces zero new warnings.
4. `cargo build` and `cargo build --release` succeed.
5. `check-fixtures.ps1` passes.
6. `test-engine.ps1` passes (24/24).
7. `test-mcp-transcripts.ps1` passes (15/15).
8. `test-host-result-follow-up.ps1` passes with exactly one admission record.
9. `test-host-event-admission-trail.ps1` passes (debug + release scenarios).
10. `test-host-denial.ps1` passes.
11. `test-host-execution-failure.ps1` passes.
12. `demo.ps1` passes.
13. `check-tethers-task-packet.ps1` passes.
14. `opam exec -- dune build` succeeds.
15. Branch pushed to origin with matching local/remote SHA.

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

## Forbidden changes

- `event_admission.rs`, `event_queue.rs`, `result_anchor.rs` - not modified
- `Cargo.toml`, `Cargo.lock` - not modified
- OCaml, protocol fixtures, existing Trail entry semantics - not modified
- `docs/ROAD_TO_0_2.md` - not modified
- No new dependencies

## Stop conditions

- Task packet checker fails due to missing sections.
- Any mandatory script produces unexpected results beyond documented pre-existing issues.
- Git status is not clean after expected changes.
- Branch cannot be pushed or remote SHA does not match local.
- Base commit does not resolve or is not an ancestor of HEAD.

## Correction round (2026-07-28)

The previous NEEDS REVIEW diagnosis (`e115ca5`) incorrectly classified three script
failures as pre-existing. The actual regression was:

- Before Packet 4, `process_one_event` created the Trail parent directory.
- Packet 4 now opens the Trail for the initial admission before `process_one_event`.
- The new initial-admission path did not create the parent directory first.

Two scripts (`test-host-execution-failure.ps1`, `demo.ps1`) also retained obsolete
zero-durable-record expectations that were incompatible with the Packet 4 contract
of exactly one initial external admission record.

This correction expands the authorised Packet 4 file set from six to nine:

1. `tethers-0.1/host-rust/src/main.rs` — add `fs::create_dir_all` before initial admission
2. `tethers-0.1/scripts/test-host-denial.ps1` — add one-record admission assertion
3. `tethers-0.1/scripts/test-host-execution-failure.ps1` — replace zero-record with one-record assertion
4. `tethers-0.1/scripts/demo.ps1` — replace zero-record with one-record assertion
5. `docs/CURRENT_CLINE_TASK.md` — this file
6. `docs/worker-notes/2026-07-28-j11-event-trail-final.md` — updated worker note

The implementation is accepted only after the complete mandatory suite passes and
status returns to COMPLETE.

## Hash ledger

| Role | SHA | Verified |
|------|-----|----------|
| Base (Packet 3) | `a87cb49dd526f66cbbc84e85ac18be201cf3f7a7` | yes |
| Implementation checkpoint | `c8003019214f1708260500b20e1cc143e37dd0d0` | yes |
| Documentation checkpoint | `1ad800c1cf7dbc24d83cf92c9f3c2bc6aff52c40` | yes |
| Evidence-correction (BLOCKED) | `e115ca573a6dee62d29fc67130724191d5f78fe4` | yes |
| Implementation correction | `648980c668085bcf6f2dd449e692a73ca7d250e9` | yes |

## J12/public-runtime boundary

The normal engine-driven route still cannot currently generate successful follow-up
Result Anchors because legitimate scope establishment belongs to J12.

This is the final J11 implementation candidate. All acceptance criteria pass.
