# Tethers Project Dashboard

Updated: 2026-08-11

## Current Milestone

Tethers 0.3 public Plug authoring is complete: P1, P2, P3, P4, and P5 are FINAL
ACCEPTED; P6 — The Evil Bunny Test — is FINAL ACCEPTED at
`5ed7634d8abc4056e0faa1ff09924377dec6e645`. The active increment is 0.4 C1 —
Together: Deterministic Fan-Out / Join Foundation.

## Verified Checkpoint

P6 final implementation checkpoint:
`5ed7634d8abc4056e0faa1ff09924377dec6e645`.

## Active Task

- Task: TETHERS-0.4-C1C — Together Execution / Join Correction
- State: implementation complete; awaiting Lucy independent GitHub review
- Owner: OpenCode (implementation); Lucy (independent GitHub review and acceptance)
- Risk: Amber; first host execution change for `together` — plan-level
  fan-out/join execution through the shared boundary, with the Three Bunny
  Breakfast production-path crucible

## Last Accepted Result

P6 — The Evil Bunny Test — FINAL ACCEPTED at
`5ed7634d8abc4056e0faa1ff09924377dec6e645`.

A safe, deterministic adversarial protocol fixture (`reference-plugs/evil-bunny-proof/`)
proved that hostile providers cannot turn bad evidence into conformance
success; three genuine generic conformance gaps were corrected in
`tethers-0.1/host-rust/src/conformance.rs`.

## Matthew Decision Required

None.

## Next Route

Lucy review of C1C: the pushed branch `feature/0.4-c1-together-fan-out-join`
(implementation checkpoint `6519d92a06b54c64a38f931c65da446dcebd323a`), the
plan-level executor and schedule validation in `plan_execution.rs`, the
shared-boundary Action parameter refactor, the `GroupJoinEntry` Trail
extension, the Three Bunny Breakfast crucible and malformed-metadata tests,
and `docs/ROAD_TO_0_4.md`. C2–C5 are provisional and have NOT started.

## Operating Mode

**Gorilla Coding 🦄**

- Lucy: architecture, task compilation, GitHub review, acceptance, continuation.
- OpenCode: ordinary Green and Amber implementation, checks, report, worker note.
- Codex: Red work, difficult local diagnosis, Git/environment/recovery, and
  machine-required verification.
- Matthew: product authority and the short report-routing bridge.

## Cost And Drift

- One implementation owner per bounded task.
- Broad discovery before packet compilation.
- Cheap checks early. Expensive proof once.
- P6 proved the conformance suite itself still accepted an advertised-only
  `inputSchema` / mismatched `outputSchema`, an uncorrelated JSON-RPC response
  id, and a shutdown-refusing provider; those three generic gaps were corrected
  in `conformance.rs` with before/after evidence and regression tests.
- C1 introduced the first real concurrency primitive (`together`) as
  deterministic language semantics only: no scheduler, no threads, no
  physical-parallel execution; the flat `plan.actions` list remains a valid
  serial schedule and `plan.groups` is additive.
- C1C completed the execution half: the reference host now attempts every
  group member before the join, a non-success join blocks later Actions, and
  the serial schedule matches what a future genuinely concurrent runtime
  would observe (failure stops at the join, not inside the fan-out).

## Where Details Live

- Present goal and boundaries: `docs/CURRENT_GOAL.md`
- Current completed task contract: `docs/CURRENT_CLINE_TASK.md`
- Roadmap: `docs/ROAD_TO_0_3.md`
- Foundation Pass architecture: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- Persistence inventory: `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- Evidence and reviews: `docs/worker-notes/`
