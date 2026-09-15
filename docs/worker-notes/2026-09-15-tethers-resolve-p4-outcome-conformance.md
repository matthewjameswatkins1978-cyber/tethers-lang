# Tethers x Resolve01 P4 worker note

Task: `Tethers x Resolve01 / P4 - Outcome Delivery and Conformance`

Task packet: `User-authorized P4 prompt with Resolve R0 semantic authority`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `07870c356e034103573c5499347c61fce0700218`

Implementation checkpoint: `b003b88`

## Requested outcome

Deliver the smallest Tethers-owned boundary for sending a durable provider
outcome to Resolve, preserve provider truth across delivery failure, retain
delivery recovery state, and prove the pinned Resolve R0 semantic handshake
without adding transport or rebuilding accepted guard/execution mechanisms.

## Changes made

Added the typed `resolve_outcome` boundary with bounded opaque action
references, the closed `SUCCEEDED`/`FAILED`/`UNCERTAIN` outcome set, exact
repeat delivery, contradiction rejection, and an append-only file-backed
`Pending`/`Delivered`/`Indeterminate` recovery journal. Added bounded Trail
coordination evidence and a read-only projection from the existing shared
execution taxonomy. Added the cross-project conformance fixture and P4
architecture note pinned to Resolve R0.

## Decisions and assumptions

The existing replay terminal publication remains the durable Tethers provider
outcome authority. Delivery is invoked only with that already-known outcome;
the delivery coordinator has no provider or executor reference. A separate
narrow journal is used because lifecycle/replay authority and Trail evidence
are distinct state families; this is not a queue or generic message bus.

## Evidence

The focused outcome-delivery unit suite passed, the cross-project conformance
fixture passed, and the pre-change authoritative `just verify` passed on the
pinned base. Formatting and all-target Rust checking passed after the P4
changes.

## Discoveries

P2 already leaves provider invocation behind a durable replay terminal and
records bounded guard evidence in Trail. The appropriate P4 seam therefore
projects only the closed outcome and action reference after those existing
boundaries; no Core, syntax, Plan, policy, provider classification, or Result
Anchor change was required.

## Remaining risks

HTTP, MCP, A2A, live Resolve transport, and Resolve-side persistence remain
outside this section. The fixture proves the semantic boundary and restartable
delivery journal, not a live cross-process transport.

## Smallest next action

Stop after P4 and obtain independent review of the branch and CI result before
authorising any subsequent Tethers/Resolve section.

## References

- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P4_OUTCOME_DELIVERY.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P2_GUARD_ADMISSION.md`
- Resolve R0 commit `8d42e5b061f86b2b2a2c1949c629654968a550ff`
