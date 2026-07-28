# J11 Packet 4: Durable Event-Admission Trail - Worker Note

Task: `J11 packet 4 durable event-admission Trail and final implementation closure`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Goose`

Status: `COMPLETE`

Base commit: `a87cb49dd526f66cbbc84e85ac18be201cf3f7a7`

Implementation checkpoint: `c8003019214f1708260500b20e1cc143e37dd0d0`

Documentation checkpoint: `1ad800c1cf7dbc24d83cf92c9f3c2bc6aff52c40`

Evidence-correction checkpoint (BLOCKED): `e115ca573a6dee62d29fc67130724191d5f78fe4`

Implementation correction: `648980c668085bcf6f2dd449e692a73ca7d250e9`

Documentation correction: [to be filled after commit]

## Requested outcome

Record every J11 event-admission decision (admit/reject) in the existing durable
append-only Trail before evaluation continues or stops.

## Changes made

| File | Status | Purpose |
|------|--------|---------|
| `tethers-0.1/host-rust/src/dispatch.rs` | M | EventAdmissionEntry struct, Trail::append_event_admission, FileTrail (flush+sync_data) and RecordingTrail impls |
| `tethers-0.1/host-rust/src/main.rs` | M | build_event_admission_entry mapper, now_unix_ms, drain_result_event_queue updated with callbacks, event-admission-trail-probe (debug only), initial external admission, 10 j11_packet4_ tests; **correction**: create Trail parent directory before initial admission |
| `tethers-0.1/scripts/test-host-result-follow-up.ps1` | M | Updated to expect exactly one admission record (event_admitted, external, gen 0) |
| `tethers-0.1/scripts/test-host-event-admission-trail.ps1` | A | New compiled-boundary trail verification (4 scenarios + 5 negative + release) |
| `tethers-0.1/scripts/test-host-denial.ps1` | M | **Correction**: added durable one-record admission assertion |
| `tethers-0.1/scripts/test-host-execution-failure.ps1` | M | **Correction**: replaced zero-record with one-record admission assertion |
| `tethers-0.1/scripts/demo.ps1` | M | **Correction**: replaced zero-record with one-record admission assertion |
| `docs/CURRENT_CLINE_TASK.md` | M | Updated task packet with all Control-v1 sections and correction round; status COMPLETE |
| `docs/worker-notes/2026-07-28-j11-event-trail-final.md` | M | This file |

Files NOT modified: `event_admission.rs`, `event_queue.rs`, `result_anchor.rs`, `Cargo.toml`, `Cargo.lock`, OCaml code, protocol fixtures.

## Correction diagnosis (2026-07-28)

The previous NEEDS REVIEW diagnosis (`e115ca5`) incorrectly classified three script
failures as pre-existing bugs. The actual regression was:

1. **Production ordering regression**: Before Packet 4, `process_one_event` created the
   Trail parent directory via `fs::create_dir_all`. Packet 4 introduced initial admission
   Trail recording before `process_one_event`, but the new path did not create the parent
   directory first. When callers supplied a path in a nonexistent directory (as all three
   scripts do), `FileTrail::open` failed with "system cannot find the path."

2. **Obsolete script expectations**: `test-host-execution-failure.ps1` and `demo.ps1`
   asserted zero durable records, but Packet 4's contract requires exactly one initial
   external admission record. `test-host-denial.ps1` had no durable-record assertion.

### Fix applied

In `main.rs`, added `fs::create_dir_all` for the Trail parent directory between
`trail_path` resolution and `admission_gate.admit`, matching the ordering:

```
resolve Trail path
create parent directory
gate.admit initial event
build admission entry
open FileTrail
durably append admission entry
process_one_event
```

The three scripts were updated to assert exactly one durable admission record with
kind=event_admitted, source=external, generation=0, processing=continued,
correlation_id=event_id, causation_id/reason_code/maximum_generation omitted.

## Decisions and assumptions

1. **EventAdmissionEntry schema frozen**: kind, event_id, event_name, source, correlation_id,
   causation_id?, generation, processing, reason_code?, maximum_generation?, timestamp_unix_ms.
2. **Accepted records**: kind=event_admitted, processing=continued, reason_code/maximum_generation omitted.
3. **Duplicate rejection**: kind=event_rejected, reason_code=duplicate_event_id, processing=stopped.
4. **Depth rejection**: kind=event_rejected, reason_code=causal_depth_exceeded, maximum_generation=8.
5. **Ordering contract**: gate admission → durable append → evaluation. Never eval before append.
6. **Write failure**: Err from record_admission stops drain immediately before evaluation.
7. **Four durable scenarios**: duplicate-initial (2 records), duplicate-sibling (3), causal-depth (2), clean (3).
8. **Initial admission**: external event recorded before process_one_event.
9. **Debug-only probes**: event-admission-trail-probe and event-admission-probe both `#[cfg(debug_assertions)]`.
10. **Dune scripts**: require `OPAMSWITCH` env var when opam switch is not globally set.

## Evidence

### Rust test results (all pass)

| Group | Count | Result |
|-------|-------|--------|
| j11_packet3_ | 5 | 5/5 PASS |
| j11_packet4_ | 10 | 10/10 PASS |
| j11_ (total) | 34 | 34/34 PASS |
| event_admission | 15 | 15/15 PASS |
| Full suite | 522 | 522/522 PASS |

### Build and lint

| Command | Result |
|---------|--------|
| cargo fmt --check | PASS (no diffs) |
| cargo check | PASS (9 baseline warnings, 0 new) |
| cargo check --tests | PASS (4 baseline warnings, 0 new) |
| cargo build | PASS (9 warnings) |
| cargo build --release | PASS (9 warnings) |
| cargo clippy --all-targets --all-features | PASS (baseline only, 0 new) |
| git diff --check | PASS (no whitespace errors) |

### Script results (final correction run)

| Script | Result |
|--------|--------|
| check-tethers-task-packet.ps1 | PASS |
| check-fixtures.ps1 | PASS (46 JSON, 30 JSONL) |
| test-engine.ps1 | PASS (24/24) |
| test-mcp-transcripts.ps1 | PASS (15/15) |
| test-host-denial.ps1 | PASS (1 durable admission record verified) |
| test-host-execution-failure.ps1 | PASS (1 durable admission record verified) |
| test-host-result-follow-up.ps1 | PASS |
| test-host-event-admission.ps1 | PASS (all 8 scenarios) |
| test-host-event-admission-trail.ps1 | PASS (all 10 scenarios, debug + release) |
| demo.ps1 | PASS (1 durable admission record verified) |

### Durable admission record contract (all three scripts)

Each script verifies:
- Exactly 1 record on disk
- kind = event_admitted
- source = external
- generation = 0
- processing = continued
- correlation_id = event_id
- causation_id omitted
- reason_code omitted
- maximum_generation omitted

### Proof that host creates the Trail parent directory

All three scripts (`test-host-denial.ps1`, `test-host-execution-failure.ps1`, `demo.ps1`)
construct a fresh GUID-based path in `$env:TEMP` **without calling `New-Item`** before
invoking the host. The host successfully writes the Trail file, proving it creates
the directory internally.

### Opam/Dune

```
opam exec -- dune build: PASS (no output, exit 0)
```

### Git evidence

```
Branch: goose/j11-event-trail-final
Base:   a87cb49dd526f66cbbc84e85ac18be201cf3f7a7
Files:  9 (exactly the 9 authorized)
Porcelain: empty
```

### Hash ledger

| Role | SHA | Verified |
|------|-----|----------|
| Base (Packet 3) | `a87cb49dd526f66cbbc84e85ac18be201cf3f7a7` | yes |
| Implementation checkpoint | `c8003019214f1708260500b20e1cc143e37dd0d0` | yes |
| Documentation checkpoint | `1ad800c1cf7dbc24d83cf92c9f3c2bc6aff52c40` | yes |
| Evidence-correction (BLOCKED) | `e115ca573a6dee62d29fc67130724191d5f78fe4` | yes |
| Implementation correction | `648980c668085bcf6f2dd449e692a73ca7d250e9` | yes |
| Documentation correction | [final HEAD after docs commit] | yes |

### EventAdmissionEntry field map

| Field | Accepted | Duplicate | Depth |
|-------|----------|-----------|-------|
| kind | event_admitted | event_rejected | event_rejected |
| event_id | \<id\> | \<id\> | \<id\> |
| event_name | \<name\> | \<name\> | \<name\> |
| source | external/result_anchor | result_anchor | result_anchor |
| correlation_id | \<corr\> | \<corr\> | \<corr\> |
| causation_id | None/Some | Some | Some/None |
| generation | 0-8 | any | 9+ |
| processing | continued | stopped | stopped |
| reason_code | _omitted_ | duplicate_event_id | causal_depth_exceeded |
| maximum_generation | _omitted_ | _omitted_ | 8 |
| timestamp_unix_ms | \<ms\> | \<ms\> | \<ms\> |

### Ordering proof (from tests)

1. Gate admission happens before durable append.
2. Evaluation callback never starts before successful admission append returns Ok.
3. Accepted record written, then evaluation callback runs.
4. Rejected record written, then drain stops (break) before any later siblings.
5. Write failure returns Err immediately; evaluation never runs.
6. Later siblings after rejection/failure are neither evaluated nor recorded.

### Packet 3 preservation

- All 5 `j11_packet3_` tests pass unchanged.
- `event_admission.rs` untouched (15/15 tests pass).
- `event-admission-probe` command unchanged.
- `build_event_admission_probe_response` unchanged.

## J12 boundary

Public-runtime follow-up generation remains blocked until J12 establishes
legitimate scope. The follow-up coordinator is proven through 20 J10 unit tests.

## References

- `docs/CONSTITUTION.md` - Tethers design principles
- `tethers-0.1/SPEC.md` - 0.1 language semantics
- `docs/DECISIONS.md` - accepted design decisions
- `docs/CAPABILITY_BRIDGE.md` - manifest and trust contract
- `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md` - implementation technique
- `tethers-0.1/host-rust/src/dispatch.rs` - Trail trait and FileTrail
- `tethers-0.1/host-rust/src/main.rs` - host binary and tests
- `tethers-0.1/host-rust/src/event_admission.rs` - pure admission gate (unchanged)
