# Tethers Project Dashboard

Updated: 2026-08-14

## Current Milestone

Tethers 0.4 — Concurrency.

C1, the deterministic `together` fan-out / join semantic foundation, is complete and accepted. C2-A1, the Core-native Together semantic bridge, is complete, independently accepted, and merged at `ec56220220fd6d668d74007d6a2f44e76320349f`. Core can now carry `Together_origin` semantics into flat Runtime Plan actions plus additive non-empty groups. Canonical V2 and Rocket meaning remain frozen and preserved; Rust execution remains the serial C1 reference mechanism.

## Current State

There is **no active implementation packet**.

The next planned work is design review / packet compilation for:

- **C2-A2 — replay ownership + Trail semantic/physical ordering foundation**
- State: design review required; **no active C2-A2 implementation packet**
- Purpose: establish bounded future-concurrency foundations while keeping execution serial
- Risk: Red; physical concurrency, provider overlap, replay/Trail concurrency work, approval work, and result-anchor work have **not** started

## Last Completed Development State

- 0.3 Plug authoring P1–P6: FINAL ACCEPTED
- Evil Bunny adversarial proof: FINAL ACCEPTED
- 0.4 C1 Together semantics and reference-host join behaviour: complete / accepted
- 0.4 C2-A1 Core-native Together semantic bridge: COMPLETE / ACCEPTED / MERGED at `ec56220220fd6d668d74007d6a2f44e76320349f`
- Core phases 1–9: accepted; production evaluation route cut over to Core
- Performance R1 / Phase A / C-B1: accepted
- Canonical V2: frozen, implemented and heavily differential-tested
- Rocket V2 reductions: integrated into the production canonicaliser
- Final Rocket cutover reconciliation: CUTOVER CLEARED

## Matthew Decision Required

None.

## Next Route

Lucy reviews the present runtime architecture and the proposed C2-A2 boundary. No agent is authorised to implement C2-A2 merely because it is named in these status documents; physical concurrency remains out of scope.

The intended 0.4 sequence remains:

1. C1 — semantic fan-out / join foundation ✓
2. C2-A1 — Core-native Together semantic bridge ✓
3. C2-A2 — replay and Trail foundation design review
4. C2-A3 — later physical execution design / implementation
5. C3–C5 — resource bounds, adversarial proof, and fresh-agent proof

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
