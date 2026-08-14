# Road to Tethers 0.4 — Concurrency

> Concurrency belongs in Tethers semantics. Parallelism mostly belongs in the runtime.

A Tether declares which Actions are independent. The engine does not declare threads, workers, cores, async runtimes, operating-system processes, provider concurrency, or physical start timing.

## Status

C1 — Together semantic foundation — is complete and accepted. The OCaml engine provides the deterministic `together` fan-out / join semantic model (flat source-order `plan.actions` plus additive `plan.groups`, planner Trail evidence, malformed-group refusal, and compatible output when `together` is absent), and the reference host respects the group boundary: every group member is attempted once, the join succeeds only when every member succeeded, and a non-success join blocks later Actions.

Since C1, the repository has also completed the Core phases 1–9 production route, performance work, Canonical V2 and Rocket V2 integration. The final Rocket reconciliation is present in current `main` at `cce91229935d77a7f2ea79d2cae5b9b7cd535a59` and records the cutover as cleared.

**C2–C5 have NOT started.** There is currently no active C2 implementation packet.

C2 remains the next planned increment, but its design must be compiled against the present post-Core / Canonical-V2 runtime rather than copied mechanically from older C1 assumptions.

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

### C2 — Physical parallel execution

**NEXT PLANNED — NOT STARTED.**

Execute members of a `together` group concurrently in the runtime while preserving C1 observable semantics.

Before implementation, the C2 design must explicitly settle:

- the physical-concurrency boundary in the current Rust runtime,
- deterministic result collection / join observation,
- truthful Trail evidence for events that may physically overlap,
- failure and cancellation behaviour,
- approval / permission interaction,
- provider-dispatch and replay interaction,
- recovery behaviour,
- serial compatibility,
- the exact boundary between C2 and C3 resource management.

C2 must not casually redefine Canonical V2 identity, Runtime Plan meaning, permission semantics, replay identity, or C1 join semantics.

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
