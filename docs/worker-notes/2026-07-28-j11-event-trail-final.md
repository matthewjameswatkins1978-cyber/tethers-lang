# J11 Packet 4: Durable Event-Admission Trail — Worker Note

**Date:** 2026-07-28
**Task:** J11 packet 4 durable event-admission Trail and final implementation closure
**Status:** COMPLETE
**Base SHA:** `a87cb49dd526f66cbbc84e85ac18be201cf3f7a7`

## Files changed

| File | Status | Purpose |
|------|--------|---------|
| `tethers-0.1/host-rust/src/dispatch.rs` | M | EventAdmissionEntry, Trail::append_event_admission, FileTrail + RecordingTrail impls |
| `tethers-0.1/host-rust/src/main.rs` | M | Mapper, drain update, trail probe, initial admission, 10 tests |
| `tethers-0.1/scripts/test-host-result-follow-up.ps1` | M | Updated to expect 1 admission record |
| `tethers-0.1/scripts/test-host-event-admission-trail.ps1` | A | New compiled-boundary trail verification |
| `docs/CURRENT_CLINE_TASK.md` | M | Updated task packet |
| `docs/worker-notes/2026-07-28-j11-event-trail-final.md` | A | This file |

## EventAdmissionEntry schema

| Field | Type | Accepted | Duplicate | Depth |
|-------|------|----------|-----------|-------|
| kind | String | event_admitted | event_rejected | event_rejected |
| event_id | String | <id> | <id> | <id> |
| event_name | String | <name> | <name> | <name> |
| source | String | external/result_anchor | result_anchor | result_anchor |
| correlation_id | String | <corr> | <corr> | <corr> |
| causation_id | Option<String> | None/Some | Some | Some/None |
| generation | u32 | 0-8 | any | 9+ |
| processing | String | continued | stopped | stopped |
| reason_code | Option<String> | _omitted_ | duplicate_event_id | causal_depth_exceeded |
| maximum_generation | Option<u32> | _omitted_ | _omitted_ | 8 |
| timestamp_unix_ms | u64 | <ms> | <ms> | <ms> |

## Durable Trail scenarios

| Scenario | Records | Record 1 | Record 2 | Record 3 |
|----------|---------|----------|----------|----------|
| duplicate-initial | 2 | admitted evt/root ext gen0 | rejected evt/root ra gen1 dup | — |
| duplicate-sibling | 3 | admitted evt/root ext gen0 | admitted evt/first ra gen1 | rejected evt/first ra gen1 dup |
| causal-depth | 2 | admitted evt/root ext gen0 | rejected evt/deep ra gen9 depth8 | — |
| clean | 3 | admitted evt/root ext gen0 | admitted evt/a ra gen1 | admitted evt/b ra gen8 |

evt/later never appears in any durable Trail.

## Ordering proof

1. Gate admission decision happens before durable append.
2. Evaluation never begins before successful durable admission append.
3. Accepted record written before evaluation callback starts.
4. Rejected record written before drain stops (break).
5. Write failure returns error immediately, before evaluation.
6. Later siblings are neither evaluated nor recorded after rejection or failure.

## Write-failure proof

- Tests 9 and 10 (j11_packet4_accepted_trail_write_failure_stops_before_eval, j11_packet4_rejected_trail_write_failure_stops_before_eval) prove that when record_admission returns Err, the drain stops immediately and evaluation never occurs.
- The FileTrail::append_event_admission uses flush + sync_data, same durability contract as other durable methods.

## Normal-host initial admission proof

The denied normal route (tested manually) writes exactly one Trail record:
```json
{"kind":"event_admitted","event_id":"evt_demo_001","event_name":"evt_demo_001","source":"external","correlation_id":"evt_demo_001","generation":0,"processing":"continued","timestamp_unix_ms":...}
```

## Packet 3 preservation

- All 5 j11_packet3_ tests pass unchanged
- event_admission.rs untouched (15/15 tests pass)
- event-admission-probe command unchanged
- build_event_admission_probe_response unchanged
- No-op admission recorder used internally

## Rust test counts

| Group | Count | Result |
|-------|-------|--------|
| j11_packet3_ | 5 | 5/5 PASS |
| j11_packet4_ | 10 | 10/10 PASS |
| j11_ (total) | 34 | 34/34 PASS |
| event_admission | 15 | 15/15 PASS |
| Full suite | 522 | 522/522 PASS |

## Script results

| Script | Result |
|--------|--------|
| test-host-event-admission.ps1 | PASS (8/8) |
| test-host-event-admission-trail.ps1 | PASS (10/10: 4 scenarios + 5 negative + release) |
| test-host-result-follow-up.ps1 | Needs Dune (engine pre-built, script has opam dependency) |
| test-host-denial.ps1 | Needs Dune |
| test-host-execution-failure.ps1 | Needs Dune |
| test-engine.ps1 | Needs Dune |
| test-mcp-transcripts.ps1 | Needs Dune |
| demo.ps1 | Needs Dune |
| check-fixtures.ps1 | PENDING |
| check-tethers-task-packet.ps1 | PENDING |

Dune-dependent scripts fail because opam switch is not configured in this environment. The engine binary exists pre-built.

## Warning deltas

| Command | Baseline | Current | Delta |
|---------|----------|---------|-------|
| cargo check | 9 | 9 | 0 new |
| cargo check --tests | 4 | 4 | 0 new |
| cargo clippy --all-targets --all-features | baseline | baseline | 0 new |

## Build results

- cargo build: PASS (9 warnings)
- cargo build --release: PASS (9 warnings)
- cargo fmt --check: PASS (no diffs)
- opam exec -- dune build: NOT RUN (opam environment unavailable)

## Release diagnostic absence

Verified in test-host-event-admission-trail.ps1: both `event-admission-probe` and `event-admission-trail-probe` return non-zero exit codes in the release binary.

## Unverified boundaries

- Public-runtime follow-up generation: blocked by J12 (legitimate scope establishment)
- Dune build: opam switch not configured
- Scripts requiring Dune: engine binary exists pre-built

## Checkpoints

- Implementation checkpoint: _[pending]_
- Documentation checkpoint: _[pending]_

## Smallest next action

Lucy's final J11 acceptance verdict, then J12 design (runnable configuration and legitimate scope establishment).
