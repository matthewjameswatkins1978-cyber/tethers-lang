# Road to Tethers 0.4 — Concurrency

> Concurrency belongs in Tethers semantics. Parallelism mostly belongs in the runtime.

A Tether declares which Actions are independent. The engine does not declare threads, workers, cores, async runtimes, operating-system processes, provider concurrency, or physical start timing.

## Status

C1 — Together semantic foundation — is complete and accepted. C2-A1 — Core-native Together semantic bridge — is complete, independently accepted, and merged at `ec56220220fd6d668d74007d6a2f44e76320349f`. Core now carries `Together_origin` semantics into flat source-order Runtime Plan `actions` plus additive non-empty `groups`; Canonical V2 / Rocket meaning remains frozen and the reference host remains the serial C1 schedule.

Since C1, the repository has also completed the Core phases 1–9 production route, performance work, Canonical V2 and Rocket V2 integration. The final Rocket reconciliation is present in current `main` at `cce91229935d77a7f2ea79d2cae5b9b7cd535a59` and records the cutover as cleared.

**Physical concurrency has NOT started.** There is currently no active C2-A2 implementation packet. Replay/Trail/provider/approval/result-anchor concurrency work has not started.

C2-A2 design review / packet compilation is next. It must be compiled against the present post-C2-A1 runtime rather than copied mechanically from older C1 assumptions; no agent is authorised to implement it merely because this roadmap names it.

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

### C2-A2 — replay ownership + Trail ordering foundation

**NEXT DESIGN REVIEW — IMPLEMENTATION NOT STARTED.**

Evaluate the smallest safe replay-admission ownership and Trail semantic/physical ordering foundations while retaining the serial C1 executor. The work must preserve per-logical-key replay exclusion, intent-before-effect, prompt durable outcomes, group joins, and external Trail/recovery compatibility. It must not introduce provider overlap, async execution, worker pools, coordinator lanes, retries, approval redesign, result-anchor redesign, or any Canonical V2 / Rocket semantic change.

### Later C2 — physical parallel execution

**NOT STARTED.**

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
