# Tethers Lang

**Tethers is the execution boundary between an AI's intentions and your computer.**

A Tether describes what should happen in a form designed to be readable, predictable and inspectable:

```text
AI intention → explicit action → authority → policy → bounded execution → receipt
```

The language stays deliberately small. The runtime handles capabilities, permissions, providers, durable execution, recovery and Trails. Planning is separate from permission and execution, effects are explicit, and uncertain outcomes stay uncertain.

**Make things happen. Keep the receipts.**

## Tethers 0.7.0

- Tethers product version: 0.7.0
- Status: release candidate implementation
- Language semantics: 0.1
- Release notes: [`docs/releases/v0.7.0.md`](docs/releases/v0.7.0.md)

Tethers 0.7 keeps the language deliberately small and adds a discoverable
host-owned Agent Core for ordinary local repository work. Core plans; hosts
authorise and execute; providers perform effects; Trails record what happened.

Production Core is tested with OCaml suites and reference oracles. The separate
Rocq research tree explores machine-checked specifications; it is not a claim
that production Tethers is formally verified or extracted from Rocq.

## Cold start

From a repository, a new AI can discover the boundary with:

```powershell
tethers --help
tethers init
tethers doctor
tethers capability list
tethers capability inspect workspace.read
```

The Agent Core exposes bounded `workspace`, structured local `git`, and direct
`exec` operations. Workspace replacement requires an expected preimage hash;
Git remote mutation is not exposed; exec never inserts a shell. `threadmoth`
is an optional explicit integration and is never hidden inside `workspace.replace`.

The normal Windows full-runtime package contains `tethers.exe` and its sibling
`tethers-engine.exe`; Rust, Cargo, OCaml, opam and Dune are development tools,
not end-user prerequisites. A portable host can inspect and plan without
claiming the full local execution surface.

## Repository Map

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
- `docs/CLINE_HANDOFF.md` is the current worker-neutral Gorilla handoff guide
  (historical filename).
- `docs/TASK_PACKET_TEMPLATE.md` and `docs/WORKER_NOTE_TEMPLATE.md` define the
  two durable sides of each implementation handoff.
- `docs/PROJECT_DASHBOARD.md` is Matthew's short current-state view.
- `docs/releases/v0.7.0.md` records the current 0.7.0 release-candidate gates;
  older `ROAD_TO_0_2.md` material is retained as historical project context.

Current operating route:

```text
Lucy controls architecture, tasks, review, and continuation
    -> OpenCode implements bounded Green and ordinary Amber work
    -> Codex handles Red work, machine failures, and release gates
    -> Matthew routes concise worker reports back to Lucy
```

Copilot, Cline, and Goose are not part of the current active workflow.
Transient model names are not encoded in durable repository guidance.

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
