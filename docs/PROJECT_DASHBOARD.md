# Tethers Project Dashboard

Updated: 2026-08-15

## Current Milestone

**Tethers 0.4 — Concurrency: COMPLETE / ACCEPTED on the current integration chain.**

Accepted integration tip before this documentation refresh:

`14b2c65d1a830b4fc0a7a893ee3e72b684b09740`

`origin/main` remains:

`f189361e80bdb43c13989200e48513cdb68bd004`

So the code is accepted but not yet fully integrated into `main`.

## Current State

- C1 — Together semantic fan-out / join foundation: ACCEPTED.
- C2 — Core bridge, replay/Trail foundation, physical provider overlap: ACCEPTED.
- C3 — bounded Together concurrency: ACCEPTED.
- C4 — adversarial concurrency crucible / **Bunny Baptism**: ACCEPTED.
- C5 — fresh-agent proof: RETIRED as a redundant concurrency gate.
- `check` provider server-name bug discovered during C5 salvage: FIXED / ACCEPTED.
- Full Rust suite at the accepted bugfix checkpoint: 1550 passed, 0 failed, 2 ignored.

No further concurrency feature work is currently justified.

## Last Useful Discovery

The abandoned C5 exploration was salvaged rather than chased through another loop. It revealed:

- a real `check` server-name bug, now fixed;
- an undocumented `core_environment` requirement on the current run path;
- scope-binding / permission configuration friction.

The remaining findings belong to later authoring/HQ usability work, not to 0.4 concurrency correctness.

## Matthew Decision Required

**Integration to `main`.**

Lucy will not merge the accepted chain into `main` without Matthew's explicit authorisation.

## Next Route

1. Merge/integrate the accepted chain when Matthew says yes.
2. Freeze 0.4 concurrency.
3. Pivot to the active hackathons.
4. Return later to 0.5 HQ / authoring-surface improvements.

## Operating Mode

**Gorilla Bunny Coding Shop 🦍🐇**

- **Matthew:** product direction, taste, priorities, final human judgement, and useful copy/paste relay that keeps him visibly in the loop.
- **Lucy:** architecture department and operational controller: decomposes work, freezes important decisions, routes agents, reviews evidence, accepts/rejects work, and continuously improves the shop.
- **Gem:** peer senior technical sparring partner for genuinely difficult architecture or areas where another strong technical model can challenge Lucy's assumptions.
- **Agents:** replaceable specialist labour, scouts, implementers, reviewers, proof engineers and adversarial attackers chosen according to the job and economics.

The shop optimises for accepted correct work with minimum unnecessary compute, retries, elapsed time and Matthew effort. Process must earn its keep.

## Cost And Drift Rules

- Use the cheapest capable route, not automatically the cheapest model.
- Count retries and human intervention as real cost.
- One bounded implementation owner at a time unless a task explicitly benefits from another structure.
- Use independent review when it removes meaningful uncertainty, not as ceremony.
- Broad architecture discussion belongs with Lucy and, when valuable, Gem; implementation packets should stay bounded.
- Historical acceptance reports, worker notes, frozen specs and performance evidence remain records of their time and are not rewritten merely to sound current.
- Living status documents must agree with accepted evidence.
- Stop when evidence is enough.

## Where Details Live

- Present goal and boundaries: `docs/CURRENT_GOAL.md`
- Current / last task packet: `docs/CURRENT_CLINE_TASK.md`
- Concurrency roadmap: `docs/ROAD_TO_0_4.md`
- Operating procedure: `docs/PROJECT_CONTROL.md`
- Gorilla Bunny doctrine: `docs/GORILLA_BUNNY_MANIFESTO.md`
- Completed Plug programme: `docs/ROAD_TO_0_3.md`
- Foundation architecture: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- Concurrency designs: `docs/concurrency/`
- Evidence and reviews: `docs/worker-notes/`, `docs/review/`, `docs/perf/`
