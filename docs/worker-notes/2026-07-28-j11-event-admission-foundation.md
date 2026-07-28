# J11 Worker Note

Task: `J11 packet 1 event admission foundation`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Goose`

Status: `COMPLETE`

Base commit: `6950c4328f4df83701bbc4fc4287c96eee1e2386`

Implementation checkpoint: `91496be1c843fa0f384696388d6e6ede7a121900`

## Requested outcome

Implement a host-local event admission gate (`EventAdmissionGate`) in a new
`event_admission.rs` module. The gate accepts each exact event ID once per host
invocation, rejects duplicate event IDs, accepts causal generations 0 through 8,
rejects generation 9 or greater, and mutates no state on rejection. Unit tests
prove every required behaviour. The gate is not wired into runtime execution.

## Changes made

- Created `tethers-0.1/host-rust/src/event_admission.rs` (317 lines) containing:
  - `MAX_CAUSAL_GENERATION` constant (value `8`).
  - `EventAdmissionRejection` enum with variants `DuplicateEventId { event_id }` and
    `CausalDepthExceeded { event_id, generation, maximum_generation }`.
  - `EventAdmissionGate` struct wrapping `HashSet<String>` with `new()`, `admit()`,
    and `admitted_count()` methods.
  - 15 focused unit tests proving all required behaviours.
- Added `pub mod event_admission;` declaration to `tethers-0.1/host-rust/src/main.rs`
  (exactly one line inserted between `pub mod dispatch;` and `mod event_queue;`).
- Updated `docs/CURRENT_CLINE_TASK.md` with the J11 packet 1 control-v1 packet.
- Created this worker note.

No other files were changed. `main.rs` was changed in exactly one place.

## Decisions and assumptions

- Module is declared `pub mod event_admission` to match the project convention
  (`pub mod approval`, `pub mod dispatch`, etc.), allowing crate-internal use in
  Packet 2 without rework.
- `admitted_count()` returns `usize` for consistency with Rust collection
  conventions and `ResultEventQueue::len()`.
- Depth validation precedes duplicate lookup as mandated. Proved by test
  `depth_rejection_precedes_duplicate_rejection`.
- `EventAdmissionGate` implements `Default` via derive, matching the packet
  design; explicit `new()` is also provided for clarity.
- No `Default` was added to `EventAdmissionRejection` — it has no meaningful
  default.
- `HashSet` import already exists in `main.rs`; the new module brings its own
  `use std::collections::HashSet;`.

## Evidence

All checks pass on the implementation commit (WORKTREE):

| Check | Result |
|---|---|
| `cargo fmt --check` | PASS — no formatting errors |
| `cargo check` | PASS — 9 warnings (baseline unchanged) |
| `cargo test event_admission -- --nocapture` | 15/15 passed |
| `cargo test` (full suite) | 488/488 passed (473 + 15) |
| `check-tethers-task-packet.ps1` | PASS — control-v1/COMPLETE |
| `check-fixtures.ps1` | PASS — 46 JSON + 30 JSONL valid |
| `test-engine.ps1` | PASS — 24/24 engine cases |
| `test-mcp-transcripts.ps1` | PASS — 15/15 MCP cases |
| `test-host-denial.ps1` | PASS |
| `test-host-execution-failure.ps1` | PASS |
| `test-host-result-follow-up.ps1` | PASS (documented J10 public boundary) |
| `demo.ps1` | PASS |
| `opam exec -- dune build` (OCaml) | PASS — no output |
| `git diff --check` | PASS — no whitespace errors |
| `git status --short` | only 4 authorised paths |

Focused test names and count:

1. `fresh_gate_has_zero_admitted_events`
2. `unique_generation_zero_event_accepted`
3. `distinct_event_ids_accepted`
4. `second_admission_of_same_id_rejected_as_duplicate`
5. `duplicate_matching_is_case_sensitive`
6. `generation_eight_accepted`
7. `generation_nine_rejected`
8. `generation_above_nine_rejected`
9. `depth_rejection_does_not_reserve_event_id`
10. `duplicate_rejection_does_not_change_admitted_count`
11. `depth_rejection_precedes_duplicate_rejection`
12. `accepted_id_remains_recorded_with_no_removal_surface`
13. `consecutive_distinct_events_at_max_depth_accepted`
14. `multiple_rejections_do_not_alter_admitted_set`
15. `max_depth_rejects_generation_nine_fresh`

## Discoveries

- The `edit` tool was unable to match text across line boundaries in `main.rs`.
  Used PowerShell `StreamWriter` with line-by-line insertion as a reliable
  workaround on Windows.
- The existing `cargo check` baseline is 9 warnings (same as J10). No new
  warnings were introduced.
- The project convention uses `pub mod` for modules that are part of the
  crate's public API (`approval`, `dispatch`, `policy`, etc.) and private
  `mod` for internal implementation (`event_queue`, `manifest`, `outcome`,
  etc.). `event_admission` follows the public pattern since Packet 2 will
  consume it from `main.rs`.

## Remaining risks

- Packet 2 must decide whether a rejected event stops the queue or skips to
  the next item. This is a runtime-coordination question deferred by design.
- Empty event-ID handling is intentionally left to existing upstream behaviour
  in `process_one_event`.
- The gate is process-local and in-memory. No persistence or cross-invocation
  deduplication exists. This is correct for a per-invocation safety component
  and must not be confused with J09 durable replay protection.

## Smallest next action

Packet 2 of 4: wire `EventAdmissionGate` into the runtime coordinator. Decide
whether a rejected event stops the queue or skips to the next item, and
implement that behaviour in `main()` or `process_one_event`.

## References

- Base commit: `6950c4328f4df83701bbc4fc4287c96eee1e2386`
- Branch: `goose/j11-event-admission-foundation`
- Implementation: `tethers-0.1/host-rust/src/event_admission.rs`
- Module declaration: `tethers-0.1/host-rust/src/main.rs` line 3
- J10 queue: `tethers-0.1/host-rust/src/event_queue.rs`
- `InputEventContext`: `tethers-0.1/host-rust/src/main.rs` line ~812
- Task packet: `docs/CURRENT_CLINE_TASK.md`
