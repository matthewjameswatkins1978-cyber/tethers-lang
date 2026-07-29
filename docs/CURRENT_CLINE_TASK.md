# Current Implementation Task

Control contract: `1`

Task: `J13B Packet 2 — strict public run command`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Codex`

Route: `Codex - J13B Packet 2 public execution boundary`

Worker note: `docs/worker-notes/2026-07-29-j13b-run-command.md`

Base branch: `main`

Base commit: `f04c17b325d54327a8da3f851d70ef38f4dd4334`

Branch: `codex/j13b-run-command`

## Objective

Add the one thin public `tethers-reference-host run` command. It accepts one
strictly parsed external Anchor and immutable Facts snapshot, admits that
external event durably, then calls the accepted `PreparedRuntime` and
`HostExecutionService` exactly once for one selected configured Tether.

## Relevant background and existing behaviour

J13A provides strict clap parsing, retained OCaml engine sessions, Windows child
supervision, and the public `check` coordinator. J13B Packet 1 provides the
typed `HostExecutionService`, strict planner-response classification, retained
provider sessions, and the accepted J05-J11 shared execution boundary.

The host owns policies, admission, Trails, replay, durable intent, approval, and
execution. Tethers Core receives explicit data and proposes Plans; it does not
authorise or execute Actions.

## Required behaviour

1. Add only `run --config <PATH> --engine <PATH> --input <PATH> --trail <ABSOLUTE_PATH> --host-data-root <ABSOLUTE_PATH>` to the public clap surface.
2. Parse a typed public run input with duplicate-key rejection at every depth, no unknown fields, exact `format_version: "1"`, and stable structured input errors.
3. Preserve non-empty, whitespace-free caller `evaluation_id` and `event.id` byte-for-byte; require non-empty Tether ID/version and event name; require object event data and Facts.
4. Forbid public source, capabilities, provider identity, policy, scope, pins, approval, generation, causal, replay, or execution identity fields.
5. Resolve config, engine, and input from the caller initial directory as canonical regular files; require absolute trail and host-data-root without creating or selecting an opam switch.
6. Select exactly one configured `PreparedTether` by exact ID and version before launching an engine or provider.
7. Before validation, provider launch, evaluation, or dispatch, admit one external event at generation 0 with correlation equal to event ID, no causation, and append and durably flush its Trail entry.
8. Construct one exact `PreparedEvaluationInput`, construct `HostExecutionService`, call it once with a one-item slice, and require one typed result without bypassing `execute_shared_boundary`.
9. Reuse the existing approval-request seam for Ask with a process-local store and Trail entry; expose evaluation ID, Action ID, and a redacted reason only.
10. Map every typed service result and service error to the frozen `OutcomeStatus` vocabulary, one `tethers.cli/1` JSON document, and `OutcomeStatus::exit_code`.
11. Add focused `j13b_run_` Rust tests for parse/path/admission/approval/service/result-mapping boundaries and preserve `check` and legacy routes.
12. Add a real-engine, real-stdio-provider `test-j13b-run.ps1` public acceptance script covering completed/replay/no-actions/deny/Ask/unavailable/invalid/CLI/interruption cases.
13. Append the frozen public-command decision and complete the worker note with actual evidence.

## Relevant components

- `tethers-0.1/host-rust/src/cli.rs` — public clap command surface and frozen envelopes
- `tethers-0.1/host-rust/src/run_input.rs` — new strict public input boundary
- `tethers-0.1/host-rust/src/run_command.rs` — new path, admission, service, and envelope coordinator
- `tethers-0.1/host-rust/src/main.rs` — thin command dispatch only
- `tethers-0.1/host-rust/src/host_execution.rs` — accepted typed service seam
- `tethers-0.1/host-rust/src/event_admission.rs`, `approval.rs`, `trail.rs` — shared host-owned boundaries
- `tethers-0.1/scripts/test-j13b-run.ps1` — new public acceptance script

## Frozen decisions and invariants

- The public input is exactly `{format_version,evaluation_id,tether,event,facts}` with nested Tether `{id,version}` and event `{id,name,data}`.
- Evaluation IDs are caller-supplied values, never generated, hashed, normalised, or replaced.
- One invocation evaluates one selected configured Tether, never the whole Set.
- Initial causal metadata is host-owned: external source, generation 0, event-ID correlation, and no causation.
- Input errors, invalid paths, and unknown Tethers fail before engine/provider launch.
- The existing replay backend remains the sole provisioning authority for host-data-root.
- Ask uses the existing exact approval seam, durable Trail entry, and process-local lifetime; no public approval ID or resume route exists.
- Planner errors map to `invalid_data` with `PLANNER_ERROR`; replay and Unattempted retain their frozen distinct status/error mapping.
- stdout is exactly one JSON document and every embedded exit code equals the process exit code.
- No Tethers language, protocol, manifest, runtime-config, replay identity/persistence, J06/J09 outcome, or J13A check-contract change is permitted.

## Acceptance criteria

1. The sole new public command has the exact five required options and rejects missing, duplicate, and unknown options as invalid CLI usage.
2. Strict run input rejects duplicate keys, unknown fields, wrong format, forbidden causal/authority fields, invalid scalars, and non-object event data/Facts with stable invalid-data errors.
3. Accepted evaluation ID and event ID are exact caller values, with no derivation helper.
4. All configured-file and absolute-path rules are enforced before runtime processes launch.
5. Exact Tether selection is proven before engine/provider launch; one input cannot evaluate every configured Tether.
6. The initial external admission is durable and ordered before intent/outcome; admission or Trail failure launches nothing.
7. Service invocation uses one selected `PreparedEvaluationInput`, one service call, and one typed result through the shared execution boundary.
8. Ask creates the real approval-request Trail entry, launches no provider, and returns no public approval ID.
9. Every `ExecutionServiceResult` and service error maps to the specified status, code, safe fields, and `OutcomeStatus::exit_code`.
10. Completed replay, failed replay, manual replay resolution, unavailable persistence, and Unattempted retain their frozen distinct public outcomes.
11. Focused `j13b_run_` tests cover every stated input, ordering, approval, service, and envelope mapping boundary.
12. `test-j13b-run.ps1` passes all named real-engine/public-provider acceptance cases, including interruption and exact stdout/exit agreement.
13. Existing J12, J13A, J13B Packet 1, fixture, engine, MCP transcript, and J13A public acceptance checks pass unchanged.
14. The Packet 2 decision and worker note honestly record the frozen contract and all actual evidence.

## Required verification

```powershell
# host-rust
cargo fmt --check
cargo check
cargo check --tests
cargo test j12_ -- --nocapture
cargo test j13a_ -- --nocapture
cargo test j13b_ -- --nocapture
cargo test j13b_run_ -- --nocapture
cargo test
cargo clippy --all-targets --all-features
cargo build
cargo build --release

# repository root
pwsh -NoProfile -ExecutionPolicy Bypass -File .github/scripts/check-tethers-task-packet.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File tethers-0.1/scripts/check-fixtures.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File tethers-0.1/scripts/test-engine.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File tethers-0.1/scripts/test-mcp-transcripts.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File tethers-0.1/scripts/test-j13a-check.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File tethers-0.1/scripts/test-j13b-run.ps1

# engine-ocaml, with the named external read-only switch
opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune build

git diff --check
git diff f04c17b325d54327a8da3f851d70ef38f4dd4334..HEAD
git status --short --branch
```

## Forbidden changes

Do not:
- add a public Trail command, approval decision/resume command, public follow-up orchestration, or J13C/J14 work;
- derive evaluation IDs, evaluate all configured Tethers, or expose caller-supplied policy/scope/pins/capabilities;
- change the runtime-config, Tethers protocol/language, manifest format, replay identity/persistence, J06/J09 semantics, existing J13A check contract, or dependencies;
- touch `D:\The Next Thing\Tethers Lang`, `docs/TETHERS_LUCY_NOTES.md`, or the external switch;
- merge, rebase, squash, amend accepted commits, delete branches, or push main.

## Stop conditions

Stop and report rather than guessing if:
- the exact public input needs a missing semantic or trust decision;
- the host cannot durably admit the initial event before runtime processes launch;
- the existing approval seam cannot record a request without exposing approval authority;
- strict path or replay-root handling would require a runtime-config or replay change;
- an existing J13A public contract changes;
- two materially similar implementation attempts fail.

## Expected pre-existing changes

None. Starting from clean `main` at `f04c17b325d54327a8da3f851d70ef38f4dd4334`.
