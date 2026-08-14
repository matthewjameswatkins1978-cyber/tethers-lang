# Tethers Project Dashboard

Updated: 2026-08-14

## Current Milestone

Tethers 0.4 — Concurrency.

C1, the deterministic `together` fan-out / join semantic foundation, is complete and accepted. The repository subsequently completed Core phases 1–9, the production Core cutover, performance work, Canonical V2, and Rocket V2 integration. The final Rocket reconciliation is recorded at `cce91229935d77a7f2ea79d2cae5b9b7cd535a59` and marks the cutover cleared.

## Current State

There is **no active implementation packet**.

The next planned increment is:

- **C2 — Physical Parallel Execution**
- State: design / task compilation required; implementation NOT STARTED
- Purpose: execute members of a `together` group concurrently while preserving the observable semantics established by C1
- Risk: Red until the present post-Core / Canonical-V2 runtime boundary has been reviewed

## Last Completed Development State

- 0.3 Plug authoring P1–P6: FINAL ACCEPTED
- Evil Bunny adversarial proof: FINAL ACCEPTED
- 0.4 C1 Together semantics and reference-host join behaviour: complete / accepted
- Core phases 1–9: accepted; production evaluation route cut over to Core
- Performance R1 / Phase A / C-B1: accepted
- Canonical V2: frozen, implemented and heavily differential-tested
- Rocket V2 reductions: integrated into the production canonicaliser
- Final Rocket cutover reconciliation: CUTOVER CLEARED

## Matthew Decision Required

None.

## Next Route

Lucy reviews the present runtime architecture and compiles the bounded C2 design / implementation packet. No C2 code should begin merely from the old C1 assumptions.

The intended 0.4 sequence remains:

1. C1 — semantic fan-out / join foundation ✓
2. C2 — physical parallel execution
3. C3 — concurrency limits / resource bounds
4. C4 — adversarial concurrency crucible
5. C5 — fresh-agent concurrency proof

## Operating Mode

**Gorilla Coding 🦄**

- Lucy: architecture, task compilation, GitHub review, acceptance, continuation.
- OpenCode: ordinary Green and Amber implementation, checks, report, worker note.
- Codex: Red work, difficult local diagnosis, Git/environment/recovery, and machine-required verification.
- Matthew: product authority and the short report-routing bridge.

## Cost And Drift Rules

- One implementation owner per bounded task.
- Broad discovery before packet compilation.
- Cheap checks early. Expensive proof once.
- Historical acceptance reports, worker notes, frozen specs and performance evidence are records of their time and are not rewritten merely to sound current.
- Living status documents must agree with current `main` and accepted evidence.
- C2 must preserve C1 observable semantics and the post-Core / Canonical-V2 identity, replay, Trail, permission and failure boundaries unless an explicit design decision says otherwise.

## Where Details Live

- Present goal and boundaries: `docs/CURRENT_GOAL.md`
- Current / next task packet: `docs/CURRENT_CLINE_TASK.md`
- Concurrency roadmap: `docs/ROAD_TO_0_4.md`
- Completed Plug programme: `docs/ROAD_TO_0_3.md`
- Final Rocket reconciliation: `docs/perf/FINAL_ROCKET_CUTOVER_BASE_RECONCILIATION.md`
- Foundation architecture: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- Evidence and reviews: `docs/worker-notes/`, `docs/review/`, `docs/perf/`
