# Tethers Lang

Tethers uses a layered set of authoritative and operational documents:

- `docs/CONSTITUTION.md` records the enduring Tethers language principles.
- `tethers-0.1/SPEC.md` defines the current precise 0.1 language and protocol
  semantics.
- [`docs/architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md`](docs/architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md)
  is the joint architectural contract and build foundation for Tethers and
  Lantern Keeper.
- `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md` defines how senior engineers and AI
  agents use OCaml, Rust, PowerShell, protocol formats, and future implementation
  languages.
- `docs/MCP_PLAN.md` records the approved post-0.1 direction for an OCaml Tethers
  MCP interface.
- `docs/OCAML_GUIDE_FOR_AGENTS.md` gives version-specific OCaml environment and
  project guidance.
- `docs/PROJECT_CONTROL.md` defines task ownership, evidence, worker notes, and
  review.
- `docs/AGENT_WORKFLOW.md` defines the current **Gorilla Coding 🦄** route.
- `docs/CLINE_HANDOFF.md` defines the short Cline-to-Lucy return journey.
- `docs/TASK_PACKET_TEMPLATE.md` and `docs/WORKER_NOTE_TEMPLATE.md` define the
  two durable sides of each implementation handoff.
- `docs/PROJECT_DASHBOARD.md` is Matthew's short current-state view.
- `docs/ROAD_TO_0_2.md` is the dependency-ordered release programme through
  Tethers 0.2.

## Release Candidate

- Tethers product version: 0.2.0
- Status: awaiting independent J17 sign-off
- Tag: not yet created
- Release notes: [`docs/releases/v0.2.0.md`](docs/releases/v0.2.0.md)

Current operating route:

```text
Lucy controls architecture, tasks, review, and continuation
-> Luna on OpenCode implements bounded Green and ordinary Amber work
-> DeepSeek Pro V4 handles thicker middle implementation requiring review
-> Codex Terra High handles Red work, machine failures, and release gates
-> Matthew routes concise worker reports back to Lucy
```

Copilot is not part of the active workflow.

The active prototype and runtime development tree is `tethers-0.1/`.

## MCP Direction

Tethers owns its MCP interface directly in OCaml. Lantern Keeper is one
connected host and capability provider, not the MCP hub. The current MCP surface
is planning and authoring support over stdio: evaluate a complete Tethers request
or validate Tether source without executing Actions.

## Joint Runtime Direction

The accepted joint architecture keeps Tethers as the general coordination and
behaviour layer. Tethers Core has no built-in knowledge of Lantern Keeper,
memory, AI, MCP business meanings, or provider-specific effects. AI judgement
is invoked only through explicit Capability Actions; its structured result
normally becomes a new Anchor for deterministic follow-up evaluation.
