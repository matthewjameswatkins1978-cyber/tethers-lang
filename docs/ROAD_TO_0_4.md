# Road to Tethers 0.4 — Concurrency

> Concurrency belongs in Tethers semantics. Parallelism mostly belongs in the runtime.

A Tether declares which Actions are independent. The engine does not declare threads, workers, cores, async runtimes, operating-system processes, provider concurrency, or physical start timing.

## Status

C1 — Together semantic foundation — is complete and accepted. C2-A1 — Core-native Together semantic bridge — is complete, independently accepted, and merged at `ec56220220fd6d668d74007d6a2f44e76320349f`. Core now carries `Together_origin` semantics into flat source-order Runtime Plan `actions` plus additive non-empty `groups`; Canonical V2 / Rocket meaning remains frozen and the reference host remains the serial C1 schedule.

C2-A2a (replay admission ownership) and C2-A2b (Trail semantic/physical ordering) are complete and merged at `a07e258eeab4fc099e3d020f40689b2ab9561ee8`.

C2-A3 physical concurrency design is complete (`docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`). **Physical concurrency implementation has NOT started.**

## Sequence

### C1 — Together semantic foundation ✓

Deterministic fan-out / join: concurrency semantics, no required physical parallelism.

End state:

- `together` expresses independent Actions in one semantic group.
- All group members are attempted before the join is resolved.
- Join success requires all members to succeed.
- A failed / non-success join blocks later Actions.
- The reference host's serial execution is a valid C1 schedule.
- Tethers without `together` retains the established non-group behaviour.

### C2-A1 — Core-native Together semantic bridge ✓

Core-native propagation of `Together_origin` into Runtime Plan groups is complete, accepted, and merged at `ec56220220fd6d668d74007d6a2f44e76320349f`. This is a semantic bridge only; it did not add physical provider overlap.

### C2-A2 — replay ownership + Trail ordering foundation ✓

Complete. A2a (replay admission ownership) and A2b (Trail semantic/physical ordering) are merged at `a07e258eeab4fc099e3d020f40689b2ab9561ee8`.

### C2-A3 — physical parallel execution

**DESIGN COMPLETE — IMPLEMENTATION NOT STARTED.**

Design artifact at `docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`.

Execute members of a `together` group concurrently in the runtime while preserving C1 observable semantics. The smallest safe boundary is C2-A3a: provider invocation overlap under coordinator ownership.

Before implementation, a separate approved task packet is required.

### C3 — Concurrency limits / resource bounds

**NOT STARTED.**

Bounded concurrency and resource accounting for group execution. C2 should not smuggle general resource scheduling into its scope unless required for correctness.

### C4 — Adversarial concurrency crucible

**NOT STARTED.**

Hostile-provider and hostile-runtime concurrency evidence, in the spirit of P6 The Evil Bunny Test.

### C5 — Fresh-agent concurrency proof

**NOT STARTED.**

A fresh agent authors a real multi-capability `together` Tether end to end, proving the concurrency authoring surface is usable and deterministic.

## Design principle

Concurrency belongs in Tethers semantics. Parallelism mostly belongs in the runtime.

The semantic contract should describe what may happen together and what must be true at the join. Runtime implementation details must not leak upward into source semantics unless they affect observable correctness.

## Current control documents

- Current state: `docs/PROJECT_DASHBOARD.md`
- Current goal / boundaries: `docs/CURRENT_GOAL.md`
- Current or next task packet: `docs/CURRENT_CLINE_TASK.md`
- Final Rocket reconciliation: `docs/perf/FINAL_ROCKET_CUTOVER_BASE_RECONCILIATION.md`

## Future

```text
0.3 Plug extensibility ✓
→ 0.4 concurrency
→ 0.5 HQ foundations
```
