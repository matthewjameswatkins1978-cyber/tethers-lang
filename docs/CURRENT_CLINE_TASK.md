# Current Implementation Task

Control contract: `1`

Status: `NO ACTIVE IMPLEMENTATION PACKET`

Updated: 2026-08-14

## Current position

The previous packet, `TETHERS CORE-9C - Canonical Core Production Cutover`, is complete and is preserved in the worker-note / commit history. This file is the living handoff location and must not continue presenting that completed packet as current work.

Tethers is now in the 0.4 concurrency programme. C1 (`together` deterministic fan-out / join semantics plus reference-host join behaviour) is complete. Since C1, Core phases 1–9, performance work, Canonical V2 and Rocket V2 have also been completed and integrated into the current `main` history.

## Next planned task

**C2 — Physical Parallel Execution**

C2 has **NOT STARTED**.

No implementation owner is assigned and no code-authorisation packet exists yet.

Before a C2 packet is issued, Lucy must review the present post-Core / Canonical-V2 runtime execution boundary and compile a bounded design that preserves C1 observable semantics.

## Required C2 design questions

The future packet must explicitly settle, at minimum:

1. Where physical concurrency lives in the current Rust runtime.
2. How all members of one `together` group are started / attempted without changing the C1 join contract.
3. How results are collected and ordered deterministically.
4. How failure, cancellation and later-Action blocking remain truthful.
5. How Trail evidence records physical concurrency without inventing an order that did not occur.
6. How approval, provider dispatch, replay identity and recovery interact with concurrent members.
7. What is deliberately postponed to C3 resource limits.
8. What serial compatibility behaviour must remain byte- or semantics-compatible.

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
