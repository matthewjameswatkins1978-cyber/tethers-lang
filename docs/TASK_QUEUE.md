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

### C1 — Manifest Parsing and Digesting (complete)

Final checkpoint: `34330b3` — feat: validate Columbo manifest semantics

1. [x] C1a1 data types and structured error model.
2. [x] C1a2 strict parsing, unknown-field handling, and recursive duplicate-key
   rejection.
3. [x] C1b1 investigate and verify the RFC 8785/JCS implementation/dependency
   against official vectors. Selected `serde_json_canonicalizer` 0.3.x.
4. [x] C1b2 canonicalisation, fixed SHA-256 digesting, and official/golden
   vectors.
5. [x] C1c semantic and cross-field validation.

### C2 — Trusted Manifest Store (complete)

Final checkpoint: `25ab2bb` — feat: add trusted manifest store

1. [x] C2a verify declared manifest digest.
2. [x] C2b store verified manifests with identity and digest indexes,
   insertion conflicts, idempotency, and retrieval semantics.
3. [x] C2c merged into C2b because conflict and duplicate detection are part of
   the insertion contract.

The 10-minute implementation-step limit is a clean-stop limit, not a promise
that each task must finish in ten minutes. Incomplete tasks must stop cleanly
and report remaining work.

## Joint Runtime Slice Queue

The accepted build foundation is
[`architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md`](architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md).
It maps the remaining Tethers work into one immediate vertical runtime slice
rather than separate architectural empires for provider admission, capability
resolution, permission, dispatch, result Anchors, and Trail writing.

Next:

1. [ ] Define the smallest configured local provider binding and one real stdio
   MCP provider fixture.
2. [ ] Admit one verified manifest through the Trusted Manifest Store and derive
   the live capability projection for one Tether Set with exact capability
   versions.
3. [ ] Carry the opaque manifest digest through deterministic planning for
   bridge-backed capabilities without making Tethers Core inspect complete
   manifests.
4. [ ] Implement conservative effective policy outcomes:
   `allow`, `ask`, `deny`, and `unavailable`.
5. [ ] Dispatch serially with no automatic retries, intent-first Trail entries,
   honest `succeeded`/`failed`/`uncertain` classification, output validation,
   and standard result Anchors.
    - [x] Dispatch intent preparation proof boundary hardened: `Allow` now
          carries a policy-created exact-capability token, resolved capability
          fields are private/read-only outside the resolver, and write failures
          return no dispatch-ready token without claiming atomic JSONL append.
          The production intent recorder trait is sealed to the file-backed
          append/flush/sync implementation; the non-durable recorder is test-only.
    - [x] Dispatch proof boundary globally enforced: every production
          provider/executor invocation requires `&DispatchReadyAction`.
          `authorise_and_execute()` now enforces exactly one Action, verifies
          capability name and provider identity match the resolved binding,
          calls `prepare_and_record()` before execution, and performs zero
          executor calls on any preparation failure.  The old `HostPolicy`
          effect-check bypass has been removed.  19 focused Rust tests prove
          internal dispatch-boundary invariants and branches (212 total, all
          pass).
    - [x] Active process-level host scripts complement the Rust tests:
          `test-host-denial.ps1` exercises the real OCaml engine -> Rust host
          route through Deny policy and canonical `intent_failed` behaviour,
          while `test-host-execution-failure.ps1` uses Allow policy, durable
          `prepare_and_record()`, executor mode `fail`, and `FailingExecutor`
          to prove one durable intent, one `action_started`, one
          `action_failed`, and zero `action_completed` entries.  Both scripts
          use unique GUID-based temporary Trails and clean them afterward.
     - [x] Executor output validation now runs after executor `Ok(result)` and
           before durable success outcome recording or `action_completed`.
           Validation failures record one failed durable outcome with no result,
           append `action_failed`, preserve failed status, and do not retry.
           Executor errors bypass output validation and keep their original
           failure message. Independent review also closed fail-open handling of
           array items, enum/const constraints, schema-valued additional
           properties, and unsupported assertion keywords.
     - [x] Known-outcome Result Anchors (`capability.succeeded` after valid
           successful output, `capability.failed` after executor error,
           `capability.failed` after output-validation failure) emitted through
           a focused `result_anchor` module.  No Result Anchor is created when
           the Action was never attempted (Ask, Deny, Unavailable, identity
           mismatch, intent-write failure).  `capability.uncertain`, event
           queuing, deduplication, causal-depth enforcement, and follow-up
           evaluation remain deferred.

Later:

- Lantern Keeper capability-provider integration after Lantern Keeper exposes a
  small stable capability surface.
- Safe retry only after idempotency is proved end to end.
- Additional providers, automatic discovery, HQ, remote transports, and
  package/marketplace work.

## 0.1 Finishing Queue

1. [x] Add version-rejection fixtures (`incompatible_protocol`, `incompatible_language`).
2. [x] Add denied host integration test (prove `execution_status: denied`
       and canonical `intent_failed` behaviour end-to-end).
3. [x] Add execution-failure host test (prove process-level
       `action_failed` path through `FailingExecutor` after durable intent).
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
