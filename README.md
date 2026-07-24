# Tethers Lang

Tethers is governed by two complementary documents:

- `docs/CONSTITUTION.md` records the enduring design principles.
- `tethers-0.1/SPEC.md` defines the current precise 0.1 language and protocol
  semantics.
- [`docs/architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md`](docs/architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md)
  is the joint architectural contract and build foundation for Tethers and
  Lantern Keeper.
- `docs/MCP_PLAN.md` records the approved post-0.1 direction for an OCaml
  Tethers MCP interface.
- `docs/OCAML_GUIDE_FOR_AGENTS.md` gives version-specific OCaml guidance for AI
  coding agents.
- `docs/PROJECT_CONTROL.md` defines the bounded task, ownership, evidence,
  worker-note, and review loop used to build the project.
- `docs/TASK_PACKET_TEMPLATE.md` and `docs/WORKER_NOTE_TEMPLATE.md` define the
  two sides of each agent handoff.
- `docs/PROJECT_DASHBOARD.md` is Matthew's short current-state view.
- `docs/ROAD_TO_0_2.md` is the dependency-ordered release programme, job
  routing and handoff map through Tethers 0.2.

The active prototype tree is `tethers-0.1/`.

## MCP Direction

Tethers owns its MCP interface directly in OCaml. Lantern Keeper is one
connected host and capability provider, not the MCP hub. The first MCP surface
is planner-only over stdio: evaluate a complete Tethers request and return the
existing Plan and Trail envelope without executing Actions.

## Joint Runtime Direction

The accepted joint architecture keeps Tethers as the general coordination and
behaviour layer. Tethers Core has no built-in knowledge of Lantern Keeper,
memory, AI, MCP business meanings, or provider-specific effects. AI judgement
is invoked only through explicit capability Actions; its structured result
normally becomes a new Anchor for deterministic follow-up evaluation.
