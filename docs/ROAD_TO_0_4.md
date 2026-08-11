# Road to Tethers 0.4 — Concurrency

> Concurrency belongs in Tethers semantics. Parallelism mostly belongs in the
> runtime.

A Tether declares which Actions are independent. The engine does not declare
threads, workers, cores, async runtimes, operating-system processes, provider
concurrency, or physical start timing.

## Status

C1 — Together semantic foundation — implemented: the `together` fan-out / join
block is deterministic language semantics in the OCaml engine (flat
source-order `plan.actions` plus the additive `plan.groups` array,
`group_planned` planner Trail evidence, malformed-group refusal, byte-compatible
output without `together`), and the reference host now respects the semantic
group boundary in execution: every group member is attempted once, the join
succeeds only when every member succeeded, and a non-success join blocks later
Actions. The serial reference schedule is a valid C1 schedule.

C2–C5 below are provisional future increments and have **NOT started**.

## Sequence

### C1 — Together semantic foundation ✓

Deterministic fan-out / join: concurrency semantics, no physical parallelism.

### C2 — Physical parallel execution

Actual concurrent member execution in the runtime, with the same observable
semantics C1 established serially.

### C3 — Concurrency limits / resource bounds

Bounded concurrency and resource accounting for group execution.

### C4 — Adversarial concurrency crucible

Hostile-provider and hostile-runtime concurrency evidence, in the spirit of
P6 The Evil Bunny Test.

### C5 — Fresh-agent concurrency proof

A fresh agent authors a real multi-capability `together` Tether end to end,
proving the concurrency authoring surface is usable and deterministic.

## Design principle

Concurrency belongs in Tethers semantics. Parallelism mostly belongs in the
runtime.

## Future

```text
0.3 Plug extensibility
→ 0.4 concurrency
→ 0.5 HQ foundations
```
