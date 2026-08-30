# Tethers Lang

**Tethers is a small deterministic automation language for connecting events to actions across tools, services and AI.**

A Tether describes what should happen in a form designed to be readable, predictable and inspectable:

```text
event → conditions → actions → result
```

The language stays deliberately small. The runtime handles capabilities, permissions, providers, durable execution, recovery and Trails. Planning is separate from permission and execution, effects are explicit, and uncertain outcomes stay uncertain.

**Make things happen. Keep the receipts.**

## Released Version

### Tethers 0.2.2

- Tethers product version: 0.2.2
- Status: released
- Tag: `v0.2.2`
- Language semantics: 0.1
- Release notes: [`docs/releases/v0.2.2.md`](docs/releases/v0.2.2.md)

Tethers 0.2.2 is the Foundation-hardened, independently verified 0.2 runtime. It adds no new major product capability; Foundation makes the existing system more trustworthy, maintainable and reproducible.

The previous published release was Tethers 0.2.0. Plug functionality is not part of the 0.2.2 release.

### Tethers Portable 0.1.0

The Windows x64 Portable release is the small host-policy decision façade. It
proposes `ALLOW`, `ASK`, or `DENY`; it is not a packaged rewrite of the OCaml
Core evaluator and never executes Actions. Release evidence and the immutable
artifact checksum are recorded in
[`tethers-0.1/portable-rust/RELEASE.md`](tethers-0.1/portable-rust/RELEASE.md).

The canonical application icon is
[`assets/tethers-icon.png`](assets/tethers-icon.png).

## Current Development

Development is ahead of the latest public release.

- Tethers 0.3 public Plug authoring P1–P6: complete / accepted.
- Tethers 0.4 concurrency C1–C4: complete / accepted on the current integration chain.
- C5 fresh-agent proof: retired as a redundant concurrency gate.
- The `check` provider server-name bug found during C5 salvage: fixed / accepted.
- Core phases 1–9 and production Core cutover: complete / accepted.
- Canonical V2 and Rocket V2: implemented, proved and integrated into current `main`.
- Immediate action: integrate the accepted 0.4 chain into `main` only when
  Matthew explicitly authorises it.
- Next active focus: hackathon work; later 0.5 HQ/authoring-surface work is not
  an authorised implementation increment.

For live project state, use [`docs/PROJECT_DASHBOARD.md`](docs/PROJECT_DASHBOARD.md). For the concurrency programme, use [`docs/ROAD_TO_0_4.md`](docs/ROAD_TO_0_4.md).

## Repository Map

Tethers uses a layered set of authoritative, operational and historical documents:

- `docs/CONSTITUTION.md` records the enduring Tethers language principles.
- `tethers-0.1/SPEC.md` defines the current precise 0.1 language and protocol semantics.
- [`docs/architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md`](docs/architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md) is the joint architectural contract and build foundation for Tethers and Lantern Keeper.
- `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md` defines how senior engineers and AI agents use OCaml, Rust, PowerShell, protocol formats, and future implementation languages.
- `docs/MCP_PLAN.md` records the approved post-0.1 direction for an OCaml Tethers MCP interface.
- `docs/OCAML_GUIDE_FOR_AGENTS.md` gives version-specific OCaml environment and project guidance.
- `docs/PROJECT_CONTROL.md` defines task ownership, evidence, worker notes, and review.
- `docs/AGENT_WORKFLOW.md` defines the current **Gorilla Bunny Coding Shop 🦍🐇** route.
- `docs/CLINE_HANDOFF.md` is the current worker-neutral handoff guide (historical filename).
- `docs/TASK_PACKET_TEMPLATE.md` and `docs/WORKER_NOTE_TEMPLATE.md` define the two durable sides of each implementation handoff.
- `docs/PROJECT_DASHBOARD.md` is Matthew's short current-state view.
- `docs/CURRENT_GOAL.md` records the current development goal and boundaries.
- `docs/CURRENT_CLINE_TASK.md` is the living implementation-packet handoff location; when no packet is active it must say so explicitly.
- `docs/ROAD_TO_0_3.md` is the completed 0.3 Plug-authoring programme.
- `docs/ROAD_TO_0_4.md` is the completed concurrency programme and its accepted state.
- `docs/perf/`, `docs/review/`, and `docs/worker-notes/` contain evidence and historical records. These are not rewritten merely to make old reports sound current.

Current operating route:

```text
Lucy controls architecture, tasks, review, and continuation
    -> Gem joins only when peer technical debate adds value
    -> a suitable named agent implements bounded work
    -> Matthew may route concise worker reports back to Lucy
```

Agents and tools are replaceable and selected for fit, risk, economics and any
local-machine requirement. Transient model names are not encoded in durable
repository guidance.

The active prototype and runtime development tree is `tethers-0.1/`.

## MCP Direction

Tethers owns its MCP interface directly in OCaml. Lantern Keeper is one connected host and capability provider, not the MCP hub. The current MCP surface is planning and authoring support over stdio: evaluate a complete Tethers request or validate Tether source without executing Actions.

## Joint Runtime Direction

The accepted joint architecture keeps Tethers as the general coordination and behaviour layer. Tethers Core has no built-in knowledge of Lantern Keeper, memory, AI, MCP business meanings, or provider-specific effects. AI judgement is invoked only through explicit Capability Actions; its structured result normally becomes a new Anchor for deterministic follow-up evaluation.
