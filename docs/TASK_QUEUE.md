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
      argument uniqueness, argument reuse, and capability-name uniqueness.
- [x] `docs/CONSTITUTION.md`, `docs/DECISIONS.md`, `docs/CURRENT_GOAL.md`,
      `docs/OCAML_GUIDE_FOR_AGENTS.md`, and `.clinerules/` established.

## 0.1 Finishing Queue

1. [x] Add version-rejection fixtures (`incompatible_protocol`, `incompatible_language`).
2. [x] Add denied-plan host integration test (prove `execution_status: denied` end-to-end).
3. [x] Add execution-failure host test (prove `action_failed` path).
4. [ ] Clarify or restrict Condition expected values to literals (no fixture proves `anchor.*` references in Condition expected-value position).
5. [ ] Final 0.1 milestone review and sign-off.

## Deferred

- Installing WSL, Docker, Bash, jq, or unrelated OCaml editor tooling.
- Changing parser, evaluator, host, fixtures, scripts, or examples beyond definite build defects.
- Adding adapters, package management, scheduling, HQ, or AI integration.
- Production CLI polishing, contribution setup notes, release/changelog.
- Architecture notes for adapters, HQ, and Trail inspection.