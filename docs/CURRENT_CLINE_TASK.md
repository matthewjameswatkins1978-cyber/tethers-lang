# Current Implementation Task

Control contract: `1`

Status: `NO ACTIVE IMPLEMENTATION PACKET`

Updated: 2026-08-14

## Current position

The previous packet, `TETHERS CORE-9C - Canonical Core Production Cutover`, is complete and is preserved in the worker-note / commit history. This file is the living handoff location and must not continue presenting that completed packet as current work.

Tethers is now in the 0.4 concurrency programme. C1 (`together` deterministic fan-out / join semantics plus reference-host join behaviour) is complete. C2-A1, the Core-native Together semantic bridge, is COMPLETE, independently ACCEPTED, and MERGED at `ec56220220fd6d668d74007d6a2f44e76320349f`. It allows Core to emit flat Runtime Plan actions plus additive non-empty groups while preserving frozen Canonical V2 / Rocket meaning and the Rust serial C1 execution mechanism.

## Next planned task

**C2-A2 — replay ownership + Trail semantic/physical ordering foundation**

C2-A2 design review / packet compilation is the next route. C2-A2 implementation has **NOT STARTED**.

No implementation owner is assigned and no code-authorisation packet exists yet. Naming C2-A2 here is not authority to implement it.

Before an implementation packet is issued, Lucy must review the present replay and Trail boundaries and compile a bounded design that preserves C1 observable semantics. Physical concurrency, provider overlap, approval consumption, result anchors, and follow-up queues remain unstarted and out of scope.

## Required C2 design questions

The future packet must explicitly settle, at minimum:

1. Whether replay-admission ownership can be separated from logical-key exclusion without changing replay identity, persistence, or terminal-state semantics.
2. Whether Trail needs distinct semantic-position and physical-append information before future physical overlap.
3. Which Trail compatibility and recovery constraints require an additive change or a separate migration decision.
4. What remains serial and is explicitly deferred to later physical-concurrency work.

## Frozen boundaries

C2 must not casually redefine:

- source-language `together` semantics,
- Runtime Plan meaning,
- Canonical V2 identity / canonical ordering,
- permission or approval policy,
- provider business semantics,
- replay identity,
- Trail truthfulness,
- existing deterministic join success / failure rules.

Any required semantic migration must be explicit and reviewed before implementation.

## Route

Lucy architecture / packet compilation → implementation owner selected from task risk → independent review → acceptance / continuation.

## Historical packet

The completed CORE-9C packet is recorded at:

- Worker note: `docs/worker-notes/2026-08-12-core-9c-production-cutover.md`
- Implementation checkpoint: `227f54f70a18b80abc498f3ac8ba26edffc82465`

Do not use the old CORE-9C task text as current implementation authority.
