# Task Queue

## Completed Milestones

- [x] Initial workspace inspection, extraction, and integration.
- [x] Native Windows opam, OCaml 5.5.0, Dune, and yojson toolchain.
- [x] Fixture validation, Rust tests, OCaml build, golden engine test, full demo.
- [x] Verified native Windows baseline committed locally.
- [x] PowerShell 7 automation scripts for all verification workflows.
- [x] OCaml parser, protocol helpers, and correlated error envelopes.
- [x] Fixture contract covering happy path, anchor mismatch, false condition,
      condition boundary, condition errors, action planning errors, parse errors,
      invalid indentation, argument uniqueness, argument reuse, and
      capability-name uniqueness.
- [x] `docs/CONSTITUTION.md`, `docs/DECISIONS.md`, `docs/CURRENT_GOAL.md`,
      `docs/OCAML_GUIDE_FOR_AGENTS.md`, and `.clinerules/` established.
- [x] Tethers 0.1 semantic baseline signed off.

## MCP Direction Queue

1. [x] M0 documentation checkpoint: preserve `docs/MCP_PLAN.md`, reference it
   from project guidance, and record the OCaml-owned MCP decision.
2. [x] M1 read-only OCaml MCP dependency survey: inspect `ocaml-mcp`,
   `snf_mcp`, and the OCaml `jsonrpc` package for identity, licence, MCP
   revision, stdio framing, tools support, errors, tests, OCaml/Dune/Yojson
   compatibility, Windows-native behaviour, and reproducible use.
3. [x] M2 extract one canonical OCaml evaluator boundary used by both the
   existing engine executable and any future MCP tool.
4. [x] M3 MCP transcript fixtures covering initialization, tools/list,
   tethers.evaluate, planner-error pass-through, malformed MCP calls, unknown
   tools, call-before-initialization, and clean EOF/shutdown.
5. [x] M4 minimal OCaml stdio server.
6. [ ] M5 real client verification (configure one local MCP client, verify
   initialize → tools/list → tools/call against the built OCaml server,
   confirm a real Tether returns the expected Plan and Trail, confirm no
   Action is executed, then configure Codex and Cline).

## 0.1 Finishing Queue

1. [x] Add version-rejection fixtures (`incompatible_protocol`, `incompatible_language`).
2. [x] Add denied-plan host integration test (prove `execution_status: denied` end-to-end).
3. [x] Add execution-failure host test (prove `action_failed` path).
4. [x] Clarify or restrict Condition expected values to literals (no fixture proves `anchor.*` references in Condition expected-value position).
5. [x] Add focused `contains` and boolean Condition fixtures (all four operators now covered).
6. [x] Final 0.1 milestone review and sign-off.

## Deferred

- Installing WSL, Docker, Bash, jq, or unrelated OCaml editor tooling.
- Installing, pinning, vendoring, or selecting an MCP dependency before the M1
  survey is reviewed. M1 is now recorded in `docs/MCP_DEPENDENCY_SURVEY.md`;
  future dependency changes still require an explicit implementation task.
- Implementing MCP server code before the M3 transcript fixtures. M3 is now
  complete; server implementation belongs to M4.
- Changing parser, evaluator, host, fixtures, scripts, or examples beyond definite build defects.
- Adding adapters, package management, scheduling, HQ, or AI integration.
- Production CLI polishing, contribution setup notes, release/changelog.
- Architecture notes for adapters, HQ, and Trail inspection.
