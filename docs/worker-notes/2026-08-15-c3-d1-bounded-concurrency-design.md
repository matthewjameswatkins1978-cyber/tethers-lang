# Worker Note: C3-D1 — Bounded Concurrency Design

Task: `C3-D1 — Bounded Concurrency Design`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `C3-D1 Architecture Agent`

Status: `COMPLETE`

Base commit: `f189361e80bdb43c13989200e48513cdb68bd004`

Implementation checkpoint: `<to be filled after commit>`

## Requested outcome

Freeze the exact C3 bounded-concurrency architecture as a design document
before any implementation begins. The design must bound physical Together
provider invocations within one group execution without changing C2-A3a frozen
semantics.

## Changes made

- `docs/CURRENT_CLINE_TASK.md` — replaced C2-A3a packet with C3-D1 design-only packet.
- `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md` — new C3 bounded concurrency design document.
- `docs/worker-notes/2026-08-15-c3-d1-bounded-concurrency-design.md` — this worker note.

## Decisions and assumptions

- C3-A bounds exactly one resource: active Together provider invocations within one group execution.
- `max_active_together_invocations = N` is group-local, not host-wide.
- Stage A remains serial and A3-compatible. G0 without G1 is valid crash/recovery evidence.
- `PREPARED_WAITING` is a runtime scheduling condition, not a source-language state or Trail terminal.
- Capacity is derived from GroupMemberState, not a second mutable counter.
- Admission order: earliest semantic-order PREPARED_WAITING member when capacity is available.
- Launch boundary: capacity eligibility → deadline start → G1 → worker launch.
- Slot release: after successful coordinator Stage C terminalisation, not merely on WorkerResult arrival.
- Stage C durability failure and replay G2 failure both fail closed: no new provider workers may launch.
- Final non-success selection remains by semantic Runtime Plan member order.
- N=1, N=2, N>=group-size must all be semantically equivalent.

## Evidence

Design-only task. No Rust build/test verification required.

## Publication evidence

Branch: `feature/c3-bounded-window-design`

## Discoveries

None.

## Remaining risks

Design is a candidate. Implementation may reveal that capacity tracking needs a
small helper struct invariant-checked against GroupMemberState. This is expected
and does not contradict the design.

## Smallest next action

C3-A1 — Minimal bounded launch window implementation (after Lucy design acceptance).

## References

- `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`
- `docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`
- `docs/ROAD_TO_0_4.md`
- `tethers-0.1/host-rust/src/host_execution.rs`
