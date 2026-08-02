# Tethers Execution Environment Handshake v1

Status: accepted host/task-execution boundary

## Purpose

This contract gives one development task a truthful, immutable answer to a
small question: can its named worker execute its required evidence commands in
this exact worktree? It is a host concern. It does not change Tether syntax,
OCaml Core semantics, Plug policy, or provider execution.

The handshake is deliberately one-shot:

```text
agent request -> host observation and probes -> one frozen contract -> work or stop
```

The worker does not negotiate a different toolchain after a probe fails. The
host never installs software, changes global configuration, switches shell, or
invent a fallback. A changed repository binding, observation, toolchain, or
command requires a new handshake.

## Shared workbench and optional overlays

`docs/execution-environment/tethers-development-workbench-v1.json` is the one
shared Windows development workbench for Lucy/Codex, Luna via OpenCode,
DeepSeek Pro V4 via OpenCode, and HY3 via OpenCode. It defines common
capabilities, not four duplicated machine passports.

`docs/execution-environment/worker-overlays-v1.json` contains only genuine
role constraints. It never selects a worker. Every request includes:

```json
"worker_assignment": { "selected_by": "Matthew", "worker_id": "luna-opencode" }
```

The issuer rejects any other selector. Matthew may select or replace any worker
for cost, timing, availability, quality, or strength. Agents may recommend a
worker but neither appoint themselves nor inherit ownership.

## Task request and host observation

The request binds a task ID, session ID, scope, repository root, branch and
full HEAD; capability class and explicit version policy; requested scopes; and
the expected argument arrays and working directories for evidence commands.

The host reports live facts and cheap execution probes. Tool discovery is
advisory; a required capability is verified only by its host-approved command.
`scripts/check-tethers-environment.ps1` reuses the existing developer-tools
diagnostic, runs selected `rust-host`, `ocaml-core`, or `cross-language` probes,
and emits machine-readable evidence. A docs-only task does not need a
toolchain handshake.

The OCaml profile requires the packet-authorised absolute `OcamlSwitchPath`.
It never discovers a neighbouring worktree switch. Rust probes use the frozen
`+1.89.0`, `--locked`, and offline metadata path. PowerShell probes use
`pwsh.exe -NoProfile`; a PowerShell command may use only `-File` plus a reviewed
script digest, never `-Command` or `-EncodedCommand`.

## Capability semantics

| Class | Absent or failed probe |
| --- | --- |
| `required` | Contract is `blocked`; no process may launch. |
| `preferred` | Contract is `degraded`; only a host-named exact substitute may run. |
| `replaceable` | Contract records the host-named substitute or unavailability; it does not block. |
| `optional` | Recorded as unavailable without degradation. |

Capability substitution is explicitly deferred from executable v1. The shared
workbench profile documents host-named substitutes as advisory, but the issuer
does not resolve them at runtime. A preferred or replaceable capability whose
host probe fails gates the contract as specified above; no replacement is
invented by the agent. Substitution resolution will be addressed in a future
version.

## Frozen contract and digests

The issuer uses JCS canonical JSON plus SHA-256 for three identities:

- `request_digest` binds what the worker declared;
- `observation_digest` binds live host facts and probes;
- `contract_digest` binds the host-issued agreement.

Contracts belong under ignored `.tethers/handshake/<task-id>/`; the worker note
records the digest and execution evidence rather than committing volatile local
facts. The public schema is
`docs/schemas/tethers-execution-environment-contract-v1.schema.json`; a Rust
host request example is in `docs/examples/`.

## Command and process enforcement

A capability is not permission to run an executable. The host approves one
full tuple:

```text
program absolute path + argument array + working directory + reviewed script digest
```

The Rust issuer in `execution_environment.rs` rejects a command ID absent from
the frozen contract and any program, argument, or cwd mismatch. A successful
permit builds `child_process::ChildConfig` with an empty inherited environment
and `assign_before_execution=true`; launch then passes through the existing
`SupervisedChild` owner. On Windows that creates the process suspended, joins
it to the Job Object before execution, and kills/reaps the owned process tree.
Executable-name allowlisting alone is expressly insufficient.

Filesystem and network scopes are host-authorised fields bound into the
contract. They are not a claim that a Windows Job Object is a filesystem or
network sandbox: unsupported OS-level mediation remains denied rather than
silently asserted. Job Object supervision is process containment and cleanup,
not hostile-code isolation.

## Lifecycle and refusal

The host emits exactly one `agreed`, `degraded`, or `blocked` contract. Required
absence stops before edits; the blocked evidence names the failed command,
exit code, and unprovable acceptance criterion. A task that needs a new
capability, changed HEAD, changed permission, different command, installation,
or global configuration must stop and begin a new request. There is no
renegotiation loop, automatic retry, or automatic installation.

The contract is evidence about command readiness, not a safety or correctness
certificate. Existing task packets, review, tests, Trails, and host trust
boundaries remain authoritative.

## Operational gateway (J20-H2)

The handshake library defines the contract model. The operational gateway makes
it runnable.

### tethers-env CLI

`tethers-env.exe` is a dedicated Rust binary under `src/bin/tethers_env.rs`
that turns the handshake library into an operational host command.

```text
tethers-env observe  --request <request.json>  --output <observation.json>
tethers-env issue    --request <request.json>  --observation <observation.json> --output <contract.json>
tethers-env inspect  --contract <contract.json>
tethers-env run      --contract <contract.json> --command-id <approved-command-id>
```

`observe` probes the live Windows environment, resolves executable paths, runs
version checks, and writes a `HostEnvironmentObservation`. `issue` reads a
request and observation, calls `ExecutionEnvironmentContract::issue()`, and
writes the stored contract with all three JCS SHA-256 digests. `inspect`
reloads through `from_stored()` and verifies the stored digest. `run` calls
`permit_by_id()` followed by `CommandPermit::launch()` through SupervisedChild;
it accepts only a command ID and never a replacement executable, argument, or
shell string.

All file writes are atomic (write to `.tmp`, then rename).

### Distinction: probe vs contract

Running `check-tethers-environment.ps1` alone is not an issued contract. The
PowerShell diagnostic proves tool availability; it does not bind a task, a
worker, a repository HEAD, a command array, or a scope. A contract requires
`observe` then `issue` with an explicit task request.

### OpenCode custom bash tool

`.opencode/tools/bash.ts` replaces OpenCode's built-in bash tool. It accepts
only the exact form `tethers-run <approved-command-id>` and delegates to
`tethers-env run`. All arbitrary shell commands, pipelines, redirections, and
executable paths are refused.

### Permission configuration

`opencode.json` registers the custom bash tool as a plugin and denies
`webfetch`, `websearch`, `subagents` (`task: { "*": "deny" }`), and
external-directory access. The custom bash tool itself blocks unsanctioned
commands at the tool level.

### Active task files

Runtime contracts and evidence belong under `.tethers/execution/` (gitignored).
Schemas and examples are committed; machine-specific absolute paths and active
contracts are not.

### Bootstrap exception

J20-H2 was authorised by Matthew as a bounded bootstrap task to connect the
already-accepted handshake library to the host and OpenCode. This exception
expires with J20-H2 and must never be reused for ordinary tasks.
