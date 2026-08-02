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

Substitution is closed: a worker cannot derive `Select-String`, direct Cargo,
or any other replacement after the contract is issued. Version policies are
`exact`, `minimum`, or `any`; a toolchain pin is never represented as a hint.

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
