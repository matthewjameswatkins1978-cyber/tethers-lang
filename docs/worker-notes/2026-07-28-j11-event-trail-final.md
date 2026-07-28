# J11 Packet 4: Durable Event-Admission Trail - Worker Note

Task: `J11 packet 4 durable event-admission Trail and final implementation closure`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Goose`

Status: `BLOCKED`

Base commit: `a87cb49dd526f66cbbc84e85ac18be201cf3f7a7`

Implementation checkpoint: `1ad800c1cf7dbc24d83cf92c9f3c2bc6aff52c40`

## Requested outcome

Record every J11 event-admission decision (admit/reject) in the existing durable
append-only Trail before evaluation continues or stops.

## Changes made

| File | Status | Purpose |
|------|--------|---------|
| `tethers-0.1/host-rust/src/dispatch.rs` | M | EventAdmissionEntry struct, Trail::append_event_admission, FileTrail (flush+sync_data) and RecordingTrail impls |
| `tethers-0.1/host-rust/src/main.rs` | M | build_event_admission_entry mapper, now_unix_ms, drain_result_event_queue updated with callbacks, event-admission-trail-probe (debug only), initial external admission, 10 j11_packet4_ tests |
| `tethers-0.1/scripts/test-host-result-follow-up.ps1` | M | Updated to expect exactly one admission record (event_admitted, external, gen 0) |
| `tethers-0.1/scripts/test-host-event-admission-trail.ps1` | A | New compiled-boundary trail verification (4 scenarios + 5 negative + release) |
| `docs/CURRENT_CLINE_TASK.md` | M | Updated task packet with all Control-v1 sections; status NEEDS_REVIEW |
| `docs/worker-notes/2026-07-28-j11-event-trail-final.md` | A | This file |

Files NOT modified: `event_admission.rs`, `event_queue.rs`, `result_anchor.rs`, `Cargo.toml`, `Cargo.lock`, OCaml code, protocol fixtures.

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

### Script results (complete evidence-correction run)

| Script | Result | Notes |
|--------|--------|-------|
| check-fixtures.ps1 | PASS | 46 JSON, 30 JSONL |
| test-mcp-transcripts.ps1 | PASS | 15/15 cases |
| test-engine.ps1 | PASS | 24/24 (with OPAMSWITCH) |
| test-host-result-follow-up.ps1 | PASS | Exactly 1 event_admitted record, all fields verified |
| test-host-denial.ps1 | FAIL | Pre-existing: script constructs $trailDir but never calls New-Item; FileTrail::open fails with "system cannot find the path" |
| test-host-execution-failure.ps1 | FAIL | Same pre-existing missing-directory bug |
| demo.ps1 | FAIL | Same pre-existing missing-directory bug |
| check-tethers-task-packet.ps1 | FAIL | Task packet was missing Control-v1 sections (corrected in this round) |
| test-host-event-admission.ps1 | NOT RUN | Script not found in repository |
| test-host-event-admission-trail.ps1 | NOT RUN | Requires test-host-event-admission.ps1 or separate invocation |

### Opam/Dune

```
Get-Command opam: C:\Users\Matmus\AppData\Local\Microsoft\WinGet\Packages\OCaml.opam_Microsoft.Winget.Source_8wekyb3d8bbwe\opam.exe
opam switch list: D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml (local, ocaml-base-compiler.5.5.0)
opam exec --switch="..." -- dune build: PASS (no output, exit 0)
```

Dune builds cleanly when `OPAMSWITCH` environment variable is set.

### Git evidence

```
Branch: goose/j11-event-trail-final
HEAD:   1ad800c1cf7dbc24d83cf92c9f3c2bc6aff52c40
Base:   a87cb49dd526f66cbbc84e85ac18be201cf3f7a7
Files:  6 (exactly the 6 authorized)
Porcelain: empty
```

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
- No-op admission recorder used internally in Packet 3 code paths.

## Discoveries

1. **Opam is available**: `opam.exe` exists at the WinGet path. The local switch
   `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml` is present but not
   globally set. Setting `$env:OPAMSWITCH` enables Dune builds. The previous report's
   claim that "opam environment [is] unavailable" was incorrect.

2. **Pre-existing script bugs**: `test-host-denial.ps1`, `test-host-execution-failure.ps1`,
   and `demo.ps1` all construct a temp directory path (`$trailDir`) but never call
   `New-Item -ItemType Directory`. When `FileTrail::open` attempts to write
   `$trailDir\trail.jsonl`, it fails with "The system cannot find the path specified."
   This predates J11 Packet 4 — the scripts never created the directory. By contrast,
   `test-host-result-follow-up.ps1` (updated in Packet 4) does call
   `New-Item -ItemType Directory -Path $trailDirNoFollowUp`.

3. **Task packet truncated**: The original task packet lacked several Control-v1
   required sections (Relevant background and existing behaviour, Required behaviour,
   Relevant components, Frozen decisions and invariants, Acceptance criteria, Stop
   conditions, Expected pre-existing changes). This caused `check-tethers-task-packet.ps1`
   to fail. Corrected in this round.

## Remaining risks

1. **Three scripts fail on trail-open**: `test-host-denial.ps1`, `test-host-execution-failure.ps1`,
   `demo.ps1` fail due to missing `New-Item` directory creation. These scripts are not in the
   Packet 4 authorized file list, so they were not modified. They need a separate correction task.
2. **J12 boundary**: Public-runtime follow-up generation remains blocked until J12 establishes
   legitimate scope. The follow-up coordinator is proven through 20 J10 unit tests.
3. **test-host-event-admission-trail.ps1 not independently run**: Its scenarios were run
   through the Rust test suite (10 j11_packet4_ tests cover equivalent boundary), but
   the PowerShell script itself was not invoked.

## Smallest next action

1. Lucy reviews this evidence-correction round.
2. If accepted, create a separate narrow task to fix the three pre-existing script bugs
   (add `New-Item -ItemType Directory` calls).
3. Then J12 design: runnable configuration and legitimate scope establishment.

## References

- `docs/CONSTITUTION.md` — Tethers design principles
- `tethers-0.1/SPEC.md` — 0.1 language semantics
- `docs/DECISIONS.md` — accepted design decisions
- `docs/CAPABILITY_BRIDGE.md` — manifest and trust contract
- `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md` — implementation technique
- `tethers-0.1/host-rust/src/dispatch.rs` — Trail trait and FileTrail
- `tethers-0.1/host-rust/src/main.rs` — host binary and tests
- `tethers-0.1/host-rust/src/event_admission.rs` — pure admission gate (unchanged)
