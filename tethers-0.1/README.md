# Tethers 0.1

Tethers is a small deterministic behaviour language and capability protocol.
Applications expose typed events, facts, and actions; Tethers connects them
through readable rules; hosts authorise and execute the resulting plans; the
Trail records why every decision and effect occurred.

> Apps provide the sockets. Tethers provides the cables.

This repository is the semantic baseline and first reference round trip:

```text
Rust reference host
    event + facts + capability schemas + Tether source
        | NDJSON over stdin/stdout
        v
OCaml Tethers engine
    parse -> validate -> evaluate -> plan
        |
        v
Rust reference host
    authorise effects -> execute mock capability -> append Trail
```

## What 0.1 proves

- A small textual Tether can be parsed without application-specific grammar.
- Evaluation uses only the supplied immutable snapshot.
- The same complete input produces the same plan and evaluation Trail.
- Capability calls are typed and validated before planning.
- Tethers proposes effects but cannot authorise or execute them.
- The host can execute a planned action exactly once using its idempotency key.
- Evaluation and execution records form one causal Trail.

## What 0.1 deliberately excludes

- loops, parallel actions, and branching inside `do`
- conditions based on action results
- live fact queries
- retries and compensation execution
- adapters, package management, scheduling, HQ, and AI integration

Action results should normally become new events. Another Tether can then make
a deterministic decision from the visible result.

## Repository map

- `../docs/CONSTITUTION.md` — enduring design principles
- `../docs/OCAML_GUIDE_FOR_AGENTS.md` — OCaml guidance for AI coding agents
- `SPEC.md` — current 0.1 language and protocol semantics
- `protocol/` — request, response, and capability examples
- `engine-ocaml/` — line-oriented parser, validator, and evaluator
- `host-rust/` — reference host and mock capability executor
- `examples/` — the first Tether
- `scripts/demo.ps1` — builds both programs and runs the round trip on Windows

## Intended demo

Prerequisites: Rust/Cargo and the project-local opam switch in
`engine-ocaml/`.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File .\scripts\demo.ps1
```

The native Windows verification scripts are the current project automation
entry points.
