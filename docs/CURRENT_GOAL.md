# Current Goal

## Goal

Prepare the first MCP implementation milestone by completing the read-only
dependency survey, extracting the canonical evaluator boundary, and adding the
test-first MCP transcript fixtures before server implementation.

## Immediate Definition Of Done

- `docs/MCP_PLAN.md` is preserved substantively as the approved architecture
  plan.
- `AGENTS.md`, the root `README.md`, and `tethers-0.1/README.md` point agents
  to the MCP plan where appropriate.
- `docs/DECISIONS.md` records that MCP connects directly to Tethers, the MCP
  implementation belongs in OCaml, Lantern Keeper is a host and capability
  provider, and the first MCP surface is planner-only over stdio.
- This document and `docs/TASK_QUEUE.md` name M0 as the documentation
  checkpoint and M1 as a read-only OCaml MCP dependency survey.
- No production source, language semantics, fixtures, dependencies, generated
  files, or opam switch paths are changed.

## Verified State On 2026-07-20

- Native opam is visible: `opam 2.5.2`.
- `opam init -y` was run. The first invocation exceeded the command timeout, but
  opam finished initialising enough to report the native Windows opam root via
  `opam var root` and usable switch operations.
- A project-local switch exists at `tethers-0.1/engine-ocaml` using
  `ocaml-base-compiler.5.5.0`.
- Installed local switch versions:
  - OCaml `5.5.0`
  - opam `2.5.2`
  - Dune `3.24.0`
  - yojson `2.2.2`
- Dependencies installed by the local opam package are Dune and yojson.
- The first switch creation attempt without `--deps-only` installed the compiler
  and dependencies but failed when opam tried to install the local package.
  Cause: Dune package metadata had no installable stanza.
- The switch was then recreated successfully with `--deps-only`, which installed
  only the declared dependency set.
- Compile-only defects fixed:
  - attached the engine executable to the Dune package with `public_name`;
  - removed an unused `Yojson.Safe` open;
  - removed an unused value renderer;
  - marked the parsed Tether title as deliberately read.
- Verification results:
  - `scripts/check-fixtures.ps1`: passed, `JSON fixtures are valid`.
  - `cargo test`: passed, `2 passed; 0 failed`.
  - `opam exec -- dune build`: passed.
  - `scripts/test-engine.ps1`: passed, engine response semantically matches
    `protocol/expected-response.json`.
  - `scripts/demo.ps1`: passed, full round trip completed.

## Round-Trip Evidence

The demo produced a matched Plan requiring `lantern.write`, the Rust host
authorised all required Effects, mock Action `lantern.task.record` completed,
and the final `execution_status` was `completed`.

The successful Trail contains all four stages:

- reception: `event_received`
- evaluation: `anchor_checked`, `condition_checked`, `action_planned`
- authorisation: `plan_authorised`
- execution: `action_started`, `action_completed`

## Near-Term Working Posture

Tethers 0.1 now has a verified native Windows baseline. Future work should keep
the core application-agnostic and make only small, explicit changes against the
documented 0.1 semantics. `tethers-0.1/` is the active development tree for the
0.1 cycle; do not move or rename it while the path-bound local opam switch is in
use.

PowerShell 7 (`pwsh.exe`) is the required shell for Tethers automation and Cline
tasks. Windows PowerShell 5.1 (`powershell.exe`) is not a project requirement.

## MCP Working Posture

M0 is complete. M1 is complete and recorded in
`docs/MCP_DEPENDENCY_SURVEY.md`.

Dependency decision: do not add `ocaml-mcp` or `snf_mcp` as a first-server
dependency. Use both as references only. The OCaml `jsonrpc` package remains a
possible later helper after M2/M3, but no dependency should be added before an
explicit implementation task.

M2 is complete: `tethers-0.1/engine-ocaml/bin/tethers_evaluator.ml` now exposes
`evaluate_request`, and the existing engine executable calls
`Tethers_evaluator.process_line`.

Verification after M2:

- `opam exec -- dune build`: passed.
- `scripts/check-fixtures.ps1`: passed, `JSON fixtures are valid (44 files)`.
- `scripts/test-engine.ps1`: passed all fixture cases and deterministic repeat.
- `scripts/test-host-denial.ps1`: passed.
- `scripts/test-host-execution-failure.ps1`: passed.
- `scripts/demo.ps1`: passed, full round trip completed.
- `cargo test`: passed, `2 passed; 0 failed`.

M3 is complete: `tethers-0.1/protocol/mcp-transcripts/` now contains eleven
newline-delimited JSON-RPC transcript fixture cases, each with `stdin.jsonl`
and `stdout.jsonl`.

The transcript set covers:

- initialization success;
- incompatible MCP protocol version;
- `tools/list`;
- successful `tethers.evaluate` returning `matched`;
- successful `tethers.evaluate` returning `not_matched`;
- minimal Tethers error result;
- correlated Tethers error result;
- malformed tool arguments;
- unknown tool;
- tool call before initialization;
- clean EOF/shutdown.

`tethers-0.1/scripts/test-mcp-transcripts.ps1` validates the fixture set
without running an MCP server. It compares JSON messages semantically by
ignoring object-key order, preserves transcript message order, preserves array
order, checks request/response ID correlation, verifies `notifications/initialized`
appears before normal operations, confirms `tools/list` exposes only
`tethers.evaluate`, and checks that Tethers planner errors remain
`isError: false` tool results with matching `structuredContent` and text
content.

Verification after M3:

- `scripts/check-fixtures.ps1`: passed, `JSON fixtures are valid (44 JSON files, 22 JSONL files)`.
- `scripts/test-mcp-transcripts.ps1`: passed all eleven transcript cases.
- Transcript validator deterministic repeat: passed.

Next milestone: M4 minimal OCaml stdio server.

M4 is complete on 2026-07-21.

The real OCaml stdio MCP server passes all eleven transcript cases:
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml` — JSON-RPC parsing, lifecycle state machine, method dispatch, `tethers.evaluate` delegation to `Tethers_evaluator.evaluate_request`.
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_main.ml` — stdio loop executable; reads one JSON-RPC message per line, writes only protocol JSON to stdout, sends diagnostics to stderr, exits cleanly on EOF.
- `tethers-0.1/engine-ocaml/bin/dune` — builds both `tethers_engine` (existing) and `tethers_mcp_server` (new) executables from shared source modules.
- `tethers-0.1/scripts/test-mcp-transcripts.ps1` — now launches the real `tethers_mcp_main.exe` server via .NET `Process`, pipes each `stdin.jsonl` fixture, and compares actual stdout semantically against the expected `stdout.jsonl`.

Supported MCP methods: `initialize`, `notifications/initialized`, `ping`, `tools/list`, `tools/call`.
Lifecycle: `Uninitialized → Initializing → Initialized`; calls before init rejected with -32002.
`tools/list` advertises exactly `tethers.evaluate` with the declared input schema.
`tools/call` for `tethers.evaluate` returns `structuredContent` + text mirror and `isError: false`.
Unknown tools → -32602; malformed arguments → -32602; unknown methods → -32601.
Incompatible protocol version → -32602 with `requested`/`supported` data.
Notifications (including unknown) are silently ignored; no response for EOF.
Uses only Yojson; no new dependencies added.

Verification after M4:
- `scripts/check-fixtures.ps1`: passed.
- `scripts/test-mcp-transcripts.ps1`: passed all eleven transcript cases.
- Deterministic repeat of `test-mcp-transcripts.ps1`: passed.
- `opam exec -- dune build`: passed.
- `scripts/test-engine.ps1`: passed all fixture cases and deterministic repeat.
- `scripts/test-host-denial.ps1`: passed.
- `scripts/test-host-execution-failure.ps1`: passed.
- `scripts/demo.ps1`: passed.
- `cargo test`: passed, `2 passed; 0 failed`.
- `git diff --check`: passed, whitespace clean.
- No dependency added, removed, or changed.

Next milestone: M5 — Real client verification per `docs/MCP_PLAN.md`.

M5 Cline real-client verification is complete on 2026-07-21.

Cline MCP configuration:
- Settings file: `%APPDATA%\Code\User\globalStorage\saoudrizwan.claude-dev\settings\cline_mcp_settings.json`
- Launcher: `pwsh.exe -NoProfile -File "D:\The Next Thing\Tethers Lang\tethers-0.1\scripts\launch-mcp-server.ps1"`
- The launcher invokes `opam exec -- tethers_mcp_main.exe` within the project-local opam switch at `tethers-0.1/engine-ocaml`.

Real-client evidence:
- Cline discovered `tethers.evaluate` via `tools/list` with the declared input schema (`request` object, `additionalProperties: false`).
- A matched call using the happy-path Tether returned `status: "matched"`, a Plan with one Action (`lantern.task.record`), and a full 5-entry evaluation Trail.
- A not-matched call using the same Tether with `coding.task_started` returned `status: "not_matched"`, null Plan, and a 2-entry evaluation Trail.
- `structuredContent` was returned to the client with `isError: false`.
- No Action was executed. The Rust host was never invoked.
- Server stdout contained only protocol JSON; server stderr was empty.
- MCP transcript tests continue to pass after adding the launcher script.

M5 Codex real-client verification is complete on 2026-07-21.

Codex MCP configuration:
- Project-scoped settings file: `.codex/config.toml`
- Launcher: `pwsh.exe -NoProfile -File .\tethers-0.1\scripts\launch-mcp-server.ps1`
- The launcher invokes `opam exec -- tethers_mcp_main.exe` within the project-local opam switch at `tethers-0.1/engine-ocaml`.

Real-client evidence:
- `codex mcp list` and `codex mcp get tethers` showed the enabled `tethers` stdio server from the project-scoped config.
- A Codex `exec` session discovered and called `tethers/tethers.evaluate` through Codex's MCP client.
- The matched call returned `status: "matched"`, Plan `eval_demo_001/plan`, required Effects `["lantern.write"]`, one proposed Action (`lantern.task.record`), and a 5-entry evaluation Trail.
- The not-matched call returned `status: "not_matched"`, null Plan, and a 2-entry evaluation Trail.
- Both MCP tool calls completed with `isError: false`.
- No Action was executed. The response contained only a proposed Plan and deterministic evaluation Trail; no authorisation or execution Trail entries, no Action result entries, and no Rust host invocation occurred.

M5 is complete.

M6 is complete on 2026-07-21.

`tethers.validate` has been added as a second MCP tool, using the shared
`Tether_parser.parse_tether` boundary. It validates Tethers 0.1 source syntax
and structure without requiring event data, Facts, or Capability schemas.

The validate tool accepts a single string argument `source` and returns:
- `valid: true`, `title`, `anchor`, `condition_count`, and `action_count` for
  well-formed Tether source;
- `valid: false` with a structured `error` object (`code` and `message`) for
  invalid source.

The tool reuses the existing `parse_tether` function directly; it does not
duplicate parser logic, simulate evaluation, or touch the filesystem.
Parse errors remain `isError: false` MCP tool results — they are deterministic
planner diagnostics, not transport failures.

`tools/list` now advertises both `tethers.evaluate` and `tethers.validate`
with their respective input schemas.

Three new MCP transcript fixture cases cover the validate tool:
- `validate-valid` — canonical happy-path Tether returns `valid: true` with
  correct metadata;
- `validate-invalid` — syntactically broken source returns `valid: false` with
  a precise parse_error;
- `validate-missing-source` — call without the `source` argument returns a
  JSON-RPC -32602 error.

All twelve original MCP transcript cases continue to pass. The full
verification suite (engine fixtures, host integration, Rust tests, demo,
deterministic repeat) passes unchanged.

No new dependency added. No Action executed. No Rust host invoked.
No Tethers language syntax or semantics changed.

Verification after M6:
- `scripts/check-fixtures.ps1`: passed (44 JSON files, 30 JSONL files).
- `scripts/test-mcp-transcripts.ps1`: passed all fifteen transcript cases.
- Deterministic repeat of `test-mcp-transcripts.ps1`: passed.
- `opam exec -- dune build`: passed.
- `scripts/test-engine.ps1`: passed all fixture cases and deterministic repeat.
- `scripts/test-host-denial.ps1`: passed.
- `scripts/test-host-execution-failure.ps1`: passed.
- `scripts/demo.ps1`: passed.
- `cargo test`: passed, `2 passed; 0 failed`.
- `git diff --check`: passed, whitespace clean.
- `tethers.evaluate` confirmed working through live Cline MCP client.
- `tethers.validate` confirmed functional through fresh server process
  (transcript tests). Live client requires server restart to pick up the new
  binary.

M7 is complete on 2026-07-21.

`docs/CAPABILITY_BRIDGE.md` defines the universal plug contract for connecting
discovered MCP tools to trusted Tethers capabilities. It is a design document;
no foreign MCP tool invocation, no automatic execution, no provider integration,
no networking or credential management, no Tethers syntax or semantic changes,
no new dependency, and no production implementation are part of M7.

The design establishes:

- A five-layer trust model: discovered MCP tool (untrusted) -> trusted capability
  manifest -> Tethers planner (deterministic) -> permissioned host -> execution
  Trail.
- The trusted manifest format: a canonical JSON structure with execution-
  authoritative fields (capability name/version, input/output schemas, effects,
  permission scope, reversibility, determinism, idempotency mechanism,
  confirmation policy, timeout/retry policy, provider identity, binding) and
  a SHA-256 contract digest computed over a canonical JSON representation
  excluding only the digest field itself and non-authoritative display metadata.
- Manifest digest coverage includes all execution-authoritative fields, not
  only schemas and binding identity.
- Distinction between a manifest's `confirmation_policy` (declaring what is
  acceptable) and actual standing approval (separate host-controlled state
  bound to manifest digest, approved scope, approving identity, and
  creation/revocation information).
- Provider identity uses host-assigned identity (`identity_source:
  "host_configuration"`) because MCP `serverInfo` is self-reported and mutable.
- Idempotency mechanisms are concrete: `argument_key` (host supplies
  `evaluation_id/action_id` as a key argument), `server_dedup` (server
  deduplicates internally), or `none`. The word `"conditional"` alone is
  insufficient.
- Schema-drift lifecycle: `notifications/tools/list_changed` triggers
  rediscovery; mismatched contracts become unavailable pending re-review; no
  automatic reapproval; Plan digest pinning prevents time-of-check/time-of-use
  substitution.
- Typed input/output validation rules, effect and permission scope envelopes,
  confirmation policy and standing approval separation, determinism/
  idempotency/reversibility as three distinct properties, timeout/retry/
  outcome-unknown semantics, and planning-to-execution handoff sequence.
- Execution Trail additions: capability_name, capability_version,
  manifest_digest, provider_identity, execution_id, attempt_id,
  permission_decision, confirmation_decision, dispatch_state,
  result_validation, status (completed/failed/denied/outcome_unknown),
  timestamp, and redaction rules.
- Two worked examples (read-only `obsidian.note.read`, scoped write
  `notes.note.create`) and ten explicit rejected cases.
- Eight future implementation pieces identified but not built.
- Five unresolved questions honestly stated.

The MCP plan milestone sequence (M0-M7) defined in `docs/MCP_PLAN.md` is now
complete. Genuinely deferred work from the MCP plan includes:
- Streamable HTTP, remote deployment, OAuth, network listeners.
- Action execution through permissioned hosts.
- Automatic MCP server discovery.
- Automatic conversion of arbitrary MCP tools into trusted capabilities.
- Lantern Keeper-specific language features.
- HQ UI.
- Prompts, sampling, elicitation, or long-running MCP tasks.
- Replacing the existing Tethers engine protocol.

No new milestone is invented beyond the canonical plan.

Verification after M7:
- Only documentation changed.
- Working tree was clean before M7.
- `git diff --check`: whitespace clean.

Next phase: deferred work per `docs/MCP_PLAN.md` non-goals and deferred
sections. No M8 is defined in the canonical plan.

## Fixture Contract Follow-Up

The evaluation fixture contract now covers the canonical happy path, Anchor
mismatch, false Condition, and the current sparse missing-Fact error response.
The missing-Fact case intentionally documents current behaviour only; a
correlated evaluation-error envelope remains a queued design task before the
error contract expands.

The fixture contract also now covers the inclusive boundary for
`greater_than_or_equal`: `task.changed_files = 3` with
`greater_than_or_equal 3` evaluates to `matched` and plans
`lantern.task.record`.

The OCaml Tether parser has been mechanically extracted from `main.ml` into
`engine-ocaml/bin/tether_parser.ml` without changing the verified fixture or demo
behaviour.

The JSON/Capability protocol helpers have been mechanically extracted from
`main.ml` into `engine-ocaml/bin/tethers_protocol.ml` without changing behaviour.
Module dependency chain: `main.ml` → `Tethers_protocol` → `Tether_parser`.
All seven fixture cases, the demo round-trip, and fixture validation continue to
pass.

The missing-Fact fixture now uses the correlated evaluation-error envelope for
`missing_fact` raised during Condition evaluation. The canonical missing-Fact
request still includes `project.type` and omits `task.changed_files`, so the
error Trail preserves the matched first Condition before appending a single
`condition_failed` entry for `Missing Fact: task.changed_files`. Other
contextual evaluation errors remain a separate migration task.

`docs/CONSTITUTION.md` now records the enduring Tethers design principles.
Project guidance references it as the constitutional authority, while
`tethers-0.1/SPEC.md` remains the authority for current precise 0.1 language and
protocol semantics.

The Condition type-error fixture now uses the correlated evaluation-error
envelope for `type_error` raised during Condition evaluation. The fixture keeps
the canonical Tether source and changes `project.type` to integer `7`, so the
engine preserves reception and matched Anchor Trail entries before appending one
`condition_failed` entry at sequence 3.

`docs/OCAML_GUIDE_FOR_AGENTS.md` now records the verified OCaml 5.5.0 local
toolchain, current engine module structure, project OCaml subset, Yojson usage,
and official source links for AI coding agents. `.clinerules/30-ocaml.md`
points Cline to the guide for OCaml implementation tasks without duplicating it.

`docs/TETHERS_LUCY_NOTES.md` now preserves Lucy's compact project-orientation
notes. AGENTS.md references it as optional orientation, not as an authoritative
source for semantics.

The unknown-Capability fixture now uses the correlated error envelope for
`unknown_capability` raised during Action planning. The fixture copies the
canonical happy-path Tether and changes the Action capability to
`lantern.task.save` (not supplied in capabilities), so the engine preserves
the full evaluation Trail (reception, Anchor match, both matched Conditions)
before appending one `action_planning_failed` entry at sequence 5.

The missing-Action-argument fixture now uses the correlated error envelope for
`missing_argument` raised during Action planning. The fixture copies the
canonical happy-path Tether and removes the required `task` argument from the
Action, so the engine preserves the full evaluation Trail before appending one
`action_planning_failed` entry at sequence 5.

The unknown-Action-argument fixture now uses the correlated error envelope for
`unknown_argument` raised during Action planning. The fixture copies the
canonical happy-path Tether and adds an undeclared `extra` argument to the
Action, so the engine preserves the full evaluation Trail before appending one
`action_planning_failed` entry at sequence 5.

The Action-type-error fixture now uses the correlated error envelope for
`type_error` raised during Action planning. The fixture copies the canonical
happy-path Tether and changes the `task` argument from a string to integer
`42` (capability declares `task` as `string`), so the engine preserves the
full evaluation Trail before appending one `action_planning_failed` entry at
sequence 5.

The missing-Action-reference fixture now uses the correlated error envelope
for `missing_reference` raised during Action planning. The fixture copies the
canonical happy-path Tether and removes `task` from the event data while the
Action references `anchor.task`, so the engine preserves the full evaluation
Trail before appending one `action_planning_failed` entry at sequence 5.

The Tether parse-error fixture now documents the existing minimal
pre-evaluation error contract for `parse_error`. The fixture changes the Tether
opening keyword from `tether` to `bad`; the engine returns only
`protocol_version`, `status`, and `error` — no evaluation identifiers, plan,
or Trail. `docs/DECISIONS.md` records the design decision that parse errors
remain minimal because evaluation has not begun and the two-category error
model (minimal pre-evaluation, fully correlated evaluation/planning) is
preferred over partial correlation.

The duplicate-Action-argument fixture enforces that each argument name may
appear at most once per Action. The Tether source duplicates `task` with
a discernibly different value; the parser rejects it as a `parse_error` before
evaluation begins. Different Actions may independently use the same argument
name.

The reused-argument-across-actions fixture proves that the same argument
name (`task`) may be used once in each of two separate
`lantern.task.record` Actions within a single Tether. Both Actions resolve
`anchor.task` independently, both appear in the Plan in source order with
consecutive `action_id` values (`action_1`, `action_2`), and the Trail
contains two `action_planned` entries at sequential positions 5 and 6.
`required_effects` remains deduplicated (`["lantern.write"]`). Duplicate
argument names inside a single Action remain rejected; reuse across separate
Actions is valid.

The duplicate-capability fixture enforces that every Capability name must
be unique within a request. The fixture duplicates the `lantern.task.record`
Capability name with a different version; the engine rejects it as a minimal
pre-evaluation `invalid_capability` error with no evaluation identifiers, plan,
or Trail. Capability names are compared without regard to version because
Actions address Capabilities by name. The uniqueness check runs after
Capability declarations are parsed but before evaluation begins, preserving
original order for valid requests without changing Action lookup behaviour.

The incompatible-protocol and incompatible-language fixtures prove version
rejection per SPEC §10. Both derive from the happy-path request and change
only the relevant version field to `"9.0"`. The engine returns a minimal
pre-evaluation error envelope (`incompatible_protocol` or
`incompatible_language`) with no identifiers, plan, or Trail.
`protocol_version` in the response is always `"0.1"` (the engine's native
protocol version), not the request value. SPEC §11.1 now explicitly
enumerates the four categories of pre-evaluation errors.

The host-denial integration test (`scripts/test-host-denial.ps1`) proves the
end-to-end denied-plan boundary. A happy-path request with the Capability
effect changed to `network.write` produces a valid `matched` Plan from the
engine. The Rust host inspects `required_effects`, finds an unpermitted
effect, denies the Plan, appends a `plan_denied` Trail entry, sets
`execution_status: "denied"`, and executes zero Actions. The Plan remains
present and atomic despite denial.

The host-execution-failure integration test (`scripts/test-host-execution-failure.ps1`)
proves the authorised-but-failed boundary. A happy-path request uses an
unsupported Capability (`lantern.task.fail`) with a permitted effect
(`lantern.write`). The engine produces a valid `matched` Plan. The host
authorises it (`plan_authorised`), starts the Action (`action_started`),
receives the executor error `"no host executor is installed for
lantern.task.fail"`, records `action_failed`, sets `execution_status:
"failed"`, and stops without attempting further Actions. No `action_completed`
entry appears.

Condition expected values are now restricted to literals (strings, integers,
booleans). The `parse_condition_value` function in `tether_parser.ml` rejects
`anchor.*` references during Condition parsing with a clear `parse_error`
message (`"Condition expected value must be a literal, got: ..."`) before
evaluation begins. The `condition-reference-rejected` fixture proves this by
replacing the first Condition's literal `"software"` with
`anchor.project_type`. Action `anchor.*` references remain valid and continue
to resolve in the host round-trip demo.

The contains-condition fixture proves positive substring matching
(`project.type contains "soft"` matches `"software"`). The boolean-condition
fixture proves boolean literal matching (`project.approved is true`)
with a real JSON boolean Fact value. Both produce full matched Plans and
Trails. All four condition operators (`is`, `contains`, `greater_than`,
`greater_than_or_equal`) now have focused fixture coverage.

The invalid-indentation fixture closes a final review gap: the parser now
rejects noncanonical Tether indentation instead of accepting unindented Anchor,
Condition, or Action lines as undocumented alternative syntax. Version 0.1
requires four-space indentation for Anchor, Conditions, and Action names, and
eight-space indentation for Action arguments.

## Final Tethers 0.1 Sign-Off On 2026-07-20

The Tethers 0.1 engineering baseline is verified. The language and protocol
now prove the complete 0.1 round trip: a host supplies an event, immutable
Facts, Capability schemas, and Tether source; the OCaml engine parses,
validates, evaluates, and plans deterministically; the Rust reference host
authorises required Effects, executes mock Capabilities sequentially, preserves
idempotency behaviour, and appends authorisation and execution Trail entries.

The complete verification suite passed for the signed-off state: OCaml build,
fixture validation, golden engine fixtures, denied-plan host integration,
failed-Action host integration, full demo, Rust unit tests, and Git whitespace
checks.

No 0.1 blockers remain. Post-0.1 work should be treated as deliberate
hardening or expansion, not as part of the signed-off semantic baseline.
