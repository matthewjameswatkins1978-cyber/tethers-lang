# Worker Note

Task: `J20-H2 — Operational Execution-Environment Gateway`

Task packet: `docs/CURRENT_CLINE_TASK.md` (J20-H2 given directly by Matthew with bootstrap exception)

Owner: `deepseek-pro-v4-opencode`

Status: `COMPLETE`

Base commit: `9d71d75c4c807d904e4375c4120fe7dd64336e7d`

Implementation checkpoint: Worktree — intentionally uncommitted until review

## Requested outcome

Connect the already-accepted execution-environment handshake Rust library to a
runnable host CLI (`tethers-env`) and an OpenCode custom bash tool that enforces
the contract's `permit()` boundary. Turn the library from implemented into
operationally usable.

## Changes made

### New files

- `tethers-0.1/host-rust/src/bin/tethers_env.rs` — CLI binary with `observe`,
  `issue`, `inspect`, `run` subcommands
- `.opencode/tools/bash.ts` — Custom bash tool replacement that enforces
  `tethers-run <approved-command-id>` only

### Modified files

- `tethers-0.1/host-rust/Cargo.toml` — Added `[[bin]]` entry for `tethers-env`
- `tethers-0.1/host-rust/src/execution_environment.rs` — Added three methods:
  - `to_stored_json()` — serialises the contract into the stored format
  - `permit_by_id()` — permits a command by ID only (no invocation needed)
  - `finish_permit()` — shared script-integrity verification helper
- `opencode.json` — Registered custom bash plugin; denied webfetch, websearch,
  subagents, external-directory
- `.gitignore` — Added `.tethers/execution/`
- `docs/architecture/TETHERS_EXECUTION_ENVIRONMENT_HANDSHAKE_V1.md` — Added
  operational gateway section documenting the CLI, OpenCode tool, permissions,
  and probe-vs-contract distinction

## Decisions and assumptions

- Added a dedicated binary (`tethers-env`) rather than extending the existing
  `tethers-reference-host` binary. The handshake gateway is a separate concern
  from the Tethers reference host runtime.
- `permit_by_id()` was added to the library because the operational `run` flow
  should not require callers to reconstruct the full `CommandInvocation`; the
  contract already knows the approved command details.
- `to_stored_json()` was added because `ContractData` fields are private and
  external serialisation of the stored format was impossible without a library
  method. This is a pure additive change that does not alter contract semantics.
- The OpenCode custom bash tool uses the `tool.execute.before` plugin hook as
  documented in the `customize-opencode` skill. The exact TypeScript API surface
  was assumed from the available documentation since network access was
  forbidden.
- Capability probing in `observe` is limited to locally resolvable tools;
  OCaml and PowerShell capabilities report `unavailable` without fallback
  invention.

## Evidence

Commands run and results:

```
# Startup checks
git rev-parse --show-toplevel → D:/The Next Thing/Tethers Lang - J20 Operational Handshake
git branch --show-current → opencode/j20-operational-handshake-gateway
git rev-parse HEAD → 9d71d75c4c807d904e4375c4120fe7dd64336e7d
git rev-parse origin/main → 9d71d75c4c807d904e4375c4120fe7dd64336e7d (exact match)

# Environment probe
pwsh -NoProfile -File scripts/check-tethers-environment.ps1 -Profile rust-host → all passes

# Formatting
rustfmt +1.89.0 --check execution_environment.rs → ok
rustfmt +1.89.0 --check tethers_env.rs → ok

# Full test suite
cargo +1.89.0 test --all-targets --all-features --locked → 846 passed, 10 failed
  (10 failures pre-existing: 5 engine_stdio + 5 pwsh.exe resolution)

# New tests added: 18
  Library: permit_by_id_succeeds, _refuses_unknown, _refuses_blocked,
    to_stored_json_round_trips, _produces_valid_json, _is_stable,
    permit_by_id_verifies_script_integrity, _launches_supervised_child,
    permit_and_permit_by_id_produce_same_config
  Binary: observe_writes_valid_json_output, issue_produces_all_three_digests,
    inspect_reloads_valid_contract, inspect_refuses_tampered_contract,
    run_refuses_blocked_contract, run_refuses_unknown_command_id,
    run_refuses_altered_contract, run_launches_permitted_command,
    integration_observe_issue_inspect_run

# Other verification
cargo +1.89.0 check --all-targets --all-features --locked --offline → ok
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1 → PASS
git diff --check → ok (LF/CRLF warning only, informational)
```

Commands not run: OCaml `dune build` and `dune runtest` — no OCaml changes were
made and the authorised OCaml switch was not available in this worktree.

## Discoveries

- The handshake library was implemented and tested but had no serialisation
  path for writing stored contracts (`to_stored_json` was missing) and no way
  to permit a command without constructing a full `CommandInvocation`
  (`permit_by_id` was missing). Both are now added.
- The existing binary tests for Windows supervised launch use a hardcoded
  `C:/Windows/System32/cmd.exe` path that may fail on 64-bit Windows due to
  filesystem redirection. The new binary test uses `resolve_program("cmd.exe")`
  and skips gracefully if unavailable.
- Library `pwsh.exe` integration tests panic on `resolve_pwsh()` failure (5
  tests). This is pre-existing and occurs when `where.exe pwsh.exe` fails
  in the test runner's execution context.
- Engine stdio integration tests (5 tests) require an OCaml engine binary that
  is not built in this worktree. Pre-existing.

## Remaining risks

- The OpenCode custom bash tool has not been verified end-to-end with OpenCode
  itself. The plugin hook API was assumed from the `customize-opencode` skill
  documentation since network access was forbidden. The exact `tool.execute.before`
  API surface (input/output shapes) should be validated when OpenCode loads the plugin.
- `observe` does not probe OCaml toolchain capabilities (returns `unavailable`).
  This is intentional — OCaml probing requires the authorised switch path which
  is per-task, not discoverable from the workbench profile alone.
- The `PermissionScope` `"C:\"` prefix matching in `canonical_as_prefix` may not
  correctly cover some canonicalized paths. This is a pre-existing library concern.

## Smallest next action

Verify that OpenCode loads `.opencode/tools/bash.ts` and the custom bash tool
correctly intercepts and validates `tethers-run` commands. If the plugin API
surface differs, adjust the tool file.

## References

- `tethers-0.1/host-rust/src/bin/tethers_env.rs`
- `.opencode/tools/bash.ts`
- `opencode.json`
- `docs/architecture/TETHERS_EXECUTION_ENVIRONMENT_HANDSHAKE_V1.md`
- `tethers-0.1/host-rust/src/execution_environment.rs` (lines 347-349 for `to_stored_json`, 598-622 for `permit_by_id`)
- Branch: `opencode/j20-operational-handshake-gateway` at `9d71d75c4c807d904e4375c4120fe7dd64336e7d`
