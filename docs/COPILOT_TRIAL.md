# GitHub Copilot Trial

Status: archived on 2026-07-25

Copilot is not part of the current Tethers workflow. This file is retained only
as historical evidence of an earlier routing experiment.

The active workflow is defined by `docs/AGENT_WORKFLOW.md`:

- Lucy controls architecture, task compilation, GitHub-visible review,
  acceptance, and continuation.
- Cline is the default implementation owner for ordinary Green and Amber work.
- Codex handles Red work, difficult local failure, Git/environment/recovery, and
  machine-required diagnosis.
- Matthew may paste Cline's concise report to Lucy.

Do not add new trial rows, use this file as a milestone counter, or route work
through Copilot unless Matthew and Lucy explicitly reopen the experiment.

## Preserved Evidence

The trial asked whether Copilot could reduce total implementation and
architecture-review effort without weakening scope, correctness, or trust
boundaries.

| Date | Task | Colour | Agent/model | Worktree | First pass | Correction loops | Verification complete | Architecture review | Usage | Outcome | Notes |
| --- | --- | --- | --- | --- | --- | ---: | --- | --- | --- | --- | --- |
| 2026-07-23 | Host-owned stdio provider binding fixture | Green | Cline + Codex | main | no | 1 | yes | required capability-trust review; Codex sign-off complete | unavailable | corrected | Initial implementation was corrected after review; final evidence included 21 focused provider tests, 283 Rust tests, 15 MCP transcript cases, fixture checks, host denial, host execution failure, demo, OCaml build, formatting, and whitespace checks. |

This evidence remains useful as a warning against confusing an agent report or a
green suite with independent trust-boundary sign-off.
