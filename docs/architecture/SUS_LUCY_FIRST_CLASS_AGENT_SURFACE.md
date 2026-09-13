# SUS / Tethers Lucy-First Agent Surface

Status: implementation design for spike
Date: 2026-09-13
Branch: `feature/sus-lucy-first-class`

## Decision

Do not build a new shell first.

Build SUS initially as a deterministic computer-operation surface on top of the existing Tethers 0.7 Agent Core, with two equal clients:

1. a machine-native structured client for AI/automation (Lucy-first);
2. a Nushell-facing human client built over the same semantic capabilities.

Neither client defines the semantics. Tethers capability contracts, scope, policy, providers, and Trail remain authoritative.

The defining requirement is:

> A competent AI should be able to perform ordinary computer operations without generating shell source for the common path.

SUS is therefore not "an AI shell". It is a deterministic operation interface that is unusually efficient and safe for both humans and machine callers.

## Why this direction

Tethers 0.7 already contains most of the difficult execution-boundary machinery SUS would otherwise need to reinvent:

- discoverable typed capabilities;
- bounded workspace operations;
- structured local Git operations;
- argv-only process execution that does not insert a shell;
- configured verification checks;
- capability versions and manifests;
- scope and policy;
- provider binding;
- ALLOW / ASK / DENY authority decisions;
- Result Anchors and Trail evidence;
- fail-closed behaviour;
- expected-preimage protection for workspace replacement;
- optional explicit Threadmoth integration.

Nushell already supplies most of the human shell machinery we do not want to rebuild:

- parser and REPL;
- structured pipelines;
- records, tables, lists, bytes and text;
- streaming and backpressure;
- block-scoped environment and working-directory behaviour;
- terminal interaction, completion and history;
- rich rendering and output conversion.

The new work should therefore concentrate on the genuinely missing part: one compact, machine-oriented, host-owned operation surface that makes Tethers capabilities pleasant to call directly and exposes the same operations to Nu without semantic duplication.

## Architectural rule

There is exactly one semantic execution path:

```text
Lucy / machine client             Matthew / Nushell
        |                                |
        | structured calls               | typed Nu commands
        |                                |
        +---------------+----------------+
                        |
                        v
                  SUS host surface
                        |
                        v
                 Tethers host/core
           capability + scope + policy
                        |
                 ALLOW / ASK / DENY
                        |
                        v
                  bound provider
                        |
             OS / Git / Threadmoth / tool
                        |
                        v
              structured result + Trail
```

No frontend may call a provider directly when claiming SUS guarantees.

No AI-only authority path exists.

No Nu-only semantics exist.

A capability means the same thing regardless of the client that requested it.

## First-class Lucy requirement

Lucy is a first-class client when all of the following are true:

1. Normal operations are discoverable as typed schemas, not prose documentation.
2. Lucy can call capabilities directly without constructing shell source.
3. Arguments remain structured values until the platform boundary.
4. Results are compact structured values by default.
5. Large output is bounded or streamed and does not have to be injected into context in full.
6. Failures identify the actual failed boundary and provide enough structured evidence to choose a safe continuation.
7. Consequential operations pass through Tethers authority and scope exactly as human requests do.
8. The interface never requires Lucy to infer success from terminal formatting.
9. Raw evidence remains available on demand.
10. Capability discovery is cheap enough that a new agent can learn the usable surface without reading a large manual.

The design target is not merely "LLMs can use it". The target is materially fewer retries, less repeated inspection, less context consumption, fewer syntax mistakes, and fewer unsafe command-construction opportunities than Bash, PowerShell, or raw native CLI use.

## What SUS means in this architecture

SUS is the shared operation model and user-facing identity for this layer. It is not initially a new parser, process runtime, build system, persistent filesystem database, configuration manager, or sandbox.

The first implementation should be thin enough that removing the SUS adapter does not damage Tethers Core.

SUS may later earn an independent language/runtime only if concrete limits in Nushell or the Tethers host prevent important guarantees.

## Existing Tethers responsibilities remain unchanged

Tethers Core remains deterministic and does not become an AI loop.

Plans request actions; they do not grant permission.

The host owns:

- scope;
- policy;
- authority;
- provider lifecycle;
- execution;
- replay/idempotency decisions where supported;
- Trail evidence.

Providers perform effects.

The machine-facing SUS surface is a client-facing adapter over this architecture, not a bypass around it.

## Machine surface: v0.1

The v0.1 machine surface should expose a deliberately small tool family.

### Discovery

#### `sus.describe`

Returns compact host/version/surface information.

Example result:

```json
{
  "surface": "sus.agent/1",
  "tethers": "0.7.x",
  "mode": "local",
  "features": ["workspace", "git", "process", "verification"]
}
```

#### `sus.capabilities`

Returns enabled, integrity-checked capability identities and a compact summary.

Do not dump complete manifests by default.

#### `sus.capability`

Returns the trusted schema and execution properties for one exact capability/version.

Ambiguous versions must fail closed rather than selecting one silently.

### Workspace

Initial wrappers should map onto the existing reviewed workspace provider rather than inventing a second filesystem engine:

- `sus.workspace.list`
- `sus.workspace.stat`
- `sus.workspace.read`
- `sus.workspace.read_range`
- `sus.workspace.search`
- `sus.workspace.compare`
- `sus.workspace.replace_exact`
- `sus.workspace.patch`
- `sus.workspace.hash`
- `sus.workspace.verify_hash`

The wrapper names may change, but the capability identity and semantics remain Tethers-owned.

Mutation operations must retain the existing expected-state / exact-match requirements. SUS must not weaken a refusal for convenience.

### Git

Expose the existing structured local Git subset:

- status;
- diff;
- log;
- show;
- branch list/current;
- add;
- branch create;
- checkout;
- commit.

Do not add remote mutation in this spike.

Do not accept Git pathspec glob syntax where the current Tethers provider rejects it.

### Process execution

Expose the existing bounded `process_execute` capability through a compact machine request:

```json
{
  "executable": "cargo",
  "args": ["test", "--package", "core engine"],
  "cwd": ".",
  "env": {"RUST_LOG": "debug"}
}
```

The interface must preserve the following rules:

- no shell insertion;
- executable and arguments remain distinct fields;
- cwd must remain within configured scope;
- executable must satisfy the existing allow-list policy;
- environment keys must satisfy configured allow-list policy;
- stdout and stderr remain distinct;
- timeout/output limits remain host-owned;
- Windows non-standard command-line consumers are not described as having universal argv guarantees;
- `.cmd` / `.bat` or explicit command-interpreter execution must be an explicit boundary outside ordinary structural-exec guarantees.

### Verification

Expose named `verification_run` operations.

For verification commands the caller supplies the check name, not arbitrary executable/argv/cwd/env. This is a particularly useful low-context path for AI callers.

## Result contract

Every SUS machine call should return a stable envelope conceptually shaped like:

```json
{
  "request_id": "...",
  "capability": "...",
  "authority": "ALLOW",
  "outcome": "SUCCESS",
  "summary": {},
  "result": {},
  "evidence": {
    "execution_id": "...",
    "trail_available": true
  }
}
```

Authority and execution outcome remain separate.

Allowed execution outcomes should preserve Tethers distinctions such as SUCCESS, FAILURE, UNCERTAIN, and CANCELLED where applicable.

A denied request is not reported as provider failure.

A malformed request is not reported as policy denial.

### Progressive output

Default machine responses should be concise.

Large stdout/stderr, diffs, search results, or file content must not be returned in full when bounded summaries or continuations can answer the request.

The initial spike may reuse the existing Tethers output limits. A later phase may add ephemeral result handles for expansion, but v0.1 must not introduce a persistent cache/database merely for agent convenience.

Raw evidence must remain explicitly retrievable where the existing Tethers contract permits it.

## Human surface: Nushell

The first Nu integration is a thin module/plugin over the same host surface.

It should favour normal Nu values rather than encoding Tethers responses as display strings.

Illustrative usage:

```nu
sus files ./src
| where size > 1mb
```

and:

```nu
sus exec cargo ["test" "--package" "core engine"]
```

and:

```nu
sus git status
```

The exact syntax is intentionally not frozen in this design. The spike should first prove that Nu can carry the semantic values and refusal/result model without fighting its engine.

Markdown remains a first-class presentation target, not the pipeline data model. Typed values flow internally; a human or agent can request Markdown when useful.

## Tethers MCP relationship

Tethers already has an MCP-facing Core path. Do not casually add a second interpretation of Tethers semantics.

The Lucy-first execution surface must preserve the distinction between:

- the deterministic Core/evaluator;
- the permissioned host that can actually execute providers.

The spike must choose one externally supported agent transport and document the boundary clearly.

Preferred direction:

- keep the OCaml Core/evaluator as the one semantic planner;
- make the permissioned Tethers host own the externally callable execution surface;
- if MCP is used for the external agent surface, it must be a thin host adapter and must not implement an alternative planner;
- all execution requests must enter the same host authority/capability/provider path used by normal Tethers execution.

Do not expose raw provider tools directly to the AI and then claim Tethers safety.

## Threadmoth relationship

Threadmoth remains an explicit provider/integration for mutation classes it handles well.

Do not hide Threadmoth behind generic operations in a way that changes their semantics silently.

Where Threadmoth is selected, its refusal model remains authoritative:

```text
OBSERVE -> IDENTIFY -> GUARD -> MUTATE -> VERIFY -> CERTIFY
```

If identity is ambiguous or stale, return the refusal rather than silently weakening the guard.

## Filesystem concurrency truth

SUS must not claim universal filesystem CAS.

A path-based precondition check followed by a path-based mutation retains a TOCTOU window on ordinary operating systems.

The truthful guarantee is:

> SUS-native guarded operations detect stale observations and use the strongest practical platform primitives to narrow race windows. They do not claim universal race-free mutation against concurrent uncooperative processes unless a specific provider/platform contract can prove a stronger guarantee.

Single-file and directory operations should document their exact guarantee level.

Do not market best-effort multi-file precondition validation as a transaction or atomic snapshot.

## Effects and sandboxing truth

Tethers effects and capability contracts remain useful for planning, inspection, scope, policy, and provider selection.

They are not a kernel sandbox for arbitrary external programs.

An external process runs with the authority actually granted by the operating-system execution environment unless an explicit sandbox provider is used.

No v0.1 claim may imply otherwise.

## Session references

A later Lucy-efficiency layer may expose ephemeral references such as:

```text
#F4
#R8
```

These are convenience handles, not durable truth.

If implemented in the spike, use bounded in-memory state only. A generation counter must prevent slot reuse from making an old reference resolve to unrelated data. Consequential use must revalidate underlying resource state where required by the capability.

Do not add SQLite, a daemon, or persistent filesystem shadow state for this feature.

## Streaming and cancellation

Large process output must be streamable or strictly bounded.

The host must define interruption semantics for native processes:

- receive caller cancellation / Ctrl+C;
- propagate interruption to the supervised child/process group where supported;
- wait according to bounded policy;
- escalate only according to an explicit host policy;
- reap children;
- return structured CANCELLED/FAILURE evidence.

No operation should leave an accidental orphan process merely because the frontend disconnected.

## First implementation slice

Do not attempt a complete shell.

Build one vertical slice proving all of the following:

1. An external machine client can discover enabled Tethers capabilities.
2. The client can inspect a trusted exact capability schema.
3. The client can perform a bounded workspace read without shell source.
4. The client can run one allow-listed native executable using structured argv without shell insertion.
5. The client can run one named verification check.
6. A consequential workspace mutation passes through Tethers policy/scope/provider execution and records Trail evidence.
7. A stale expected-state mutation refuses with a structured reason.
8. Large output is bounded or streamed.
9. Cancellation tears down the supervised process cleanly.
10. The same semantic operation can be invoked through a minimal Nu wrapper without changing its meaning.

## Failure Museum gates

The spike must include adversarial tests for at least:

- path containing spaces;
- path containing `$`, brackets, quotes, and Unicode;
- argument containing spaces;
- argument containing quotes;
- Windows trailing backslashes before quotes;
- empty argument;
- unbound/missing required input;
- attempted shell metacharacter injection as data;
- environment key outside allowed scope;
- cwd outside configured root;
- executable outside allow-list;
- stale preimage/hash during mutation;
- provider schema mismatch;
- adapter/tool version mismatch where relevant;
- stdout limit exceeded;
- stderr limit exceeded;
- process timeout;
- cancellation;
- provider crash;
- ASK without approval;
- DENY;
- UNCERTAIN execution outcome;
- raw external process behaving unexpectedly without being misreported as a Tethers guarantee.

Tests should assert structured outcomes, not only human-readable messages.

## Lucy-efficiency benchmark

The spike should compare common development operations through:

- PowerShell;
- raw Nushell;
- SUS/Tethers machine surface.

Measure:

- number of tool calls;
- repeated inspections;
- invalid invocations;
- retries;
- input bytes/tokens needed to specify the operation;
- output bytes/tokens returned by default;
- whether raw output had to be parsed;
- success/refusal classification accuracy;
- unsafe/unintended effects.

The goal is not an arbitrary token-score competition. It is evidence that the machine surface removes shell-mechanical reasoning while preserving or improving correctness.

## Non-goals for the spike

Do not build:

- a new SUS parser;
- a new terminal emulator;
- a general configuration manager;
- Salsa/Bazel-style caching of arbitrary process results;
- a persistent filesystem index;
- a package manager;
- a distributed cache;
- a universal OS sandbox;
- automatic natural-language interpretation;
- a second Tethers policy language;
- a second capability registry;
- a second provider lifecycle;
- an AI-specific permission bypass.

## Success criteria

The spike succeeds if:

- Lucy/machine callers can perform the selected operations without shell source;
- Nu can expose the same operations without semantic duplication;
- existing Tethers trust and execution boundaries remain intact;
- no provider bypass is required;
- Failure Museum cases are handled predictably across the supported test platforms;
- the new layer remains materially smaller than implementing a new shell/runtime;
- common agent tasks require less command-construction and output-parsing work than the current shell route.

## Stop / redesign criteria

Stop and reconsider a standalone SUS runtime if any of the following prove true:

- the machine surface cannot expose existing Tethers capabilities without bypassing the host authority path;
- Nu cannot represent guarded/custom resource results without repeated lossy conversion to strings;
- process streaming/cancellation cannot be made predictable through the chosen integration;
- Tethers' existing capability contracts are too coupled to current providers to support the operation model cleanly;
- the frontend requires a second policy or semantic system to be usable;
- cross-platform process argument handling forces unsafe hidden shell behaviour;
- ordinary machine calls still require the AI to generate substantial Nu/PowerShell/Bash source.

If those problems appear, document the exact failure before designing an independent shell.

## Principle

The project should optimise for this end state:

> Humans get a good shell. Machines get a good machine interface. Both use the same meaning.

Lucy should not have to pretend to be a human at a terminal in order to operate a computer safely.
