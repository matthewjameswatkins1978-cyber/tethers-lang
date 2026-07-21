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
6. [x] M5 real client verification:
   - [x] Cline configured and verified (initialize, tools/list, matched
     and not-matched tethers.evaluate calls, no Action executed, no Rust
     host invoked).
   - [x] Codex configured and verified through the project-scoped
     `.codex/config.toml` and launcher.
   - [x] M5 fully complete after both Cline and Codex real-client verification.
7. [x] M6 MCP authoring support: `tethers.validate` tool using shared
   `parse_tether` boundary, three new transcript fixtures
   (validate-valid, validate-invalid, validate-missing-source), `tools/list`
   updated to advertise both tools, all fifteen MCP transcript cases pass, full
   regression suite passes.
8. [x] M7 capability bridge design, not automatic execution.
   - [x] Corrected the deterministic `manifest_digest` flow and schema-drift
         fail-closed rules in `docs/CAPABILITY_BRIDGE.md`.

## Columbo Manifest Validation Queue

1. [x] C1a1 data types and structured error model.
2. [x] C1a2 strict parsing, unknown-field handling, and recursive duplicate-key
   rejection.
3. [x] C1b1 investigate and verify the RFC 8785/JCS implementation/dependency
   against official vectors; stop for a separate design decision if no suitable
   Rust implementation is verified. Selected `serde_json_canonicalizer` 0.3.x
   after reviewing version 0.3.2 against RFC examples and the cyberphone
   reference corpus.
4. [ ] C1b2 canonicalisation, fixed SHA-256 digesting, and official/golden
   vectors.
5. [ ] C1c semantic and cross-field validation.

The 10-minute implementation-step limit is a clean-stop limit, not a promise
that each task must finish in ten minutes. Incomplete tasks must stop cleanly
and report remaining work.

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
