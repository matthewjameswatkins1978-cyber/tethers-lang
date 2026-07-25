# Tethers Project Dashboard

Updated: 2026-07-25

## Current Milestone

Complete the first vertical Tethers 0.2 runtime slice around the verified
manifest store, capability projection, effective host policy, conservative
dispatch, and honest Trail/Result Anchor behaviour.

## Verified Checkpoint

Latest accepted implementation:
`d5ed278d4a2cae5e9ab8a3e1d8700fdcba7ae851`
(`feat: resolve effective policy fail closed`).

Accepted baseline includes:

- verified manifest admission and trusted-store resolution;
- deterministic bridge capability projection;
- manifest, capability-version, and provider pins carried through planning;
- intent-first dispatch through `DispatchReadyAction`;
- configured local stdio MCP provider admission;
- output validation and known-outcome Result Anchors;
- effective policy outcomes: `allow`, `ask`, `deny`, and `unavailable`;
- fail-closed stale-digest and unestablished-scope handling.

Recorded verification at the accepted runtime checkpoint includes Rust
`297 passed; 0 failed`, OCaml build, fixture and engine checks, MCP transcripts,
host denial/failure checks, demo, packet checker, and whitespace checks.

## Active Task

- Task: none authorised for implementation
- State: stopped after accepted J04a
- Owner: none
- Risk: next gate is Red design

## Last Accepted Result

J04a corrected both demonstrated fail-open paths:

1. A non-empty planned manifest digest must exactly match the current verified
   manifest digest or policy returns `unavailable` before dispatch.
2. Structured scope without a host/binding-owned assessor is
   `scope_not_established` and denies before durable intent or executor work.

The rejected J04 attempt and its evidence remain in worker notes and Git
history. They are not active project state.

## Matthew Decision Required

None.

## Next Route

Lucy compiles the separate Red J05 design packet for approval and resume
semantics. No implementation begins until that design is explicit.

After design approval, Codex is the expected Red implementation or
computer-enabled review route. Cline may receive only later bounded work that
Lucy classifies as Green or Amber.

The design must freeze one-shot approval binding, creation, expiry,
invalidation, consumption, resumed-Ask precedence, denial/cancellation Trail
semantics, replay protection, stale-plan handling, and double-consumption
failure behaviour.

## Operating Mode

**Gorilla Coding 🦄**

- Lucy: architecture, task compilation, GitHub review, acceptance, continuation.
- Cline: ordinary Green and Amber implementation, checks, report, worker note.
- Codex: Red work, difficult local diagnosis, Git/environment/recovery, and
  machine-required verification.
- Matthew: product authority and the short report-routing bridge.
- Copilot: not in the active workflow.

Matthew may paste Cline's concise report to Lucy. The report is a handoff; Git,
code, tests, Trails, packets, and worker notes remain evidence.

## Cost And Drift

- Use ordinary chat Lucy for all repository-visible thought and review.
- Use Cline as the normal coding engine.
- Spend Codex only when Red risk, machine access, recovery, or demonstrated Cline
  difficulty justifies it.
- Use one implementation owner per task.
- Load task-bounded context, not the project archive.
- Do not reopen accepted manifest-pin or J03/J03a/J03b policy decisions without a
  demonstrated defect.

## Where Details Live

- Present goal and boundaries: `docs/CURRENT_GOAL.md`
- Active task contract: `docs/CURRENT_CLINE_TASK.md`
- Operating workflow: `docs/AGENT_WORKFLOW.md`
- Cline handoff: `docs/CLINE_HANDOFF.md`
- Runtime programme: `docs/ROAD_TO_0_2.md`
- Detailed queue and completed milestones: `docs/TASK_QUEUE.md`
- Accepted design decisions: `docs/DECISIONS.md`
- Evidence and reviews: `docs/worker-notes/`
