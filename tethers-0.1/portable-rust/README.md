# Tethers Workbench 0.2.2

Tethers Workbench is the small, portable, decision-only sibling of the wider Tethers platform.

Use it when you already have an agent, script, CI worker, or local workbench that owns execution, but you do **not** want that caller improvising its own authority rules at the moment something consequential is about to happen.

Given an actor, action, resource, and context, the workbench returns exactly one authority decision:

```text
ALLOW
ASK
DENY
```

It decides authority. It does not execute actions.

There is no server, daemon, database, scheduler, agent loop, MCP service, telemetry, or LLM evaluation hidden inside it.

## Why use the portable workbench?

A caller can remain flexible about planning while the authority boundary stays deterministic and boring:

```text
agent decides: "I want to push this branch"
                    |
                    v
             Tethers Workbench
                    |
            policy + scope + contract
                    |
          +---------+---------+
          |         |         |
        ALLOW      ASK       DENY
```

That gives an integration:

- one machine-readable decision vocabulary;
- fail-closed handling for malformed or unknown input;
- explicit human approval through `ASK`;
- deterministic reasons and policy provenance;
- scoped Capability manifests;
- trace and audit surfaces that do not need another service;
- thin wrappers that do not reimplement policy logic;
- stable decision exit codes for shell and CI integration.

The important boundary is simple:

> **The caller may decide what it wants. The workbench decides whether that request is authorised.**

If you need Tethers itself to plan deterministic behaviour, execute approved Capabilities through Plugs, manage replay, emit Result Anchors, and record a causal Trail, use the full Tethers host/runtime instead. Do not mistake this smaller façade for the entire project.

## Canonical JSON protocol

```json
{
  "schema_version":"1",
  "actor":"gary.worker",
  "action":"git.push",
  "resource":"github:owner/project",
  "context":{"branch":"main","tests_passed":true,"human_present":false}
}
```

Evaluate with a policy file:

```powershell
Get-Content .\examples\gary-worker-request.json -Raw |
  .\tethers.exe evaluate --policy .\policies\default.json
```

The response is machine-readable and includes the schema version, decision, matched rule, deterministic reason, and policy identity. `ASK` is reserved for a deliberate policy requirement for human authority.

Malformed requests, unknown actions, invalid policies, missing binaries, timeouts, malformed responses, and other operational uncertainty must be treated as `DENY` by callers.

Portable 0.1 requests using `{ "action": { "name": "...", "version": 1 }, "context": {}, "policy": {} }` remain accepted. The frozen 0.1 tag and artifact are not modified.

## Explain, trace, audit, and policy tests

```powershell
.\tethers.exe explain --input .\examples\gary-worker-request.json --policy .\policies\default.json
.\tethers.exe test .\policies\default.json .\examples\workbench-policy-tests.json
.\tethers.exe test .\policies\default.json .\examples\workbench-policy-tests.json --json
```

Explain output adds `evaluated_conditions`. Sensitive values are never echoed.

The policy runner exits zero only when every case passes and reports expected, actual, and matched rule on failures.

`evaluate --trace` adds redacted rule/condition steps. `evaluate --audit PATH` appends decision metadata as JSONL and fails closed if the audit file cannot be written. Responses include `decision_id`, `policy_sha256`, and the Tethers version for provenance.

`lint POLICY` validates policy shape and reports broad wildcard/default warnings.

`init --profile coding-agent-default|read-only-agent|ci-worker|gary-worker` creates a small runnable configuration. `doctor --json` checks the bundled profile without contacting a service.

## Agent and shell usage

The complete CLI contract, including the frozen exit-code table, is in `docs/CLI.md`.

The short path is:

```text
tethers check request.json
cat request.json | tethers check -
tethers check --action git.push
tethers check request.json --json
tethers check request.json --quiet
tethers check request.json --explain
tethers validate policy.json
tethers doctor
tethers version --json
```

`check` is the script-friendly façade over the canonical evaluator.

Decision exit codes are distinct from invocation/configuration errors. A process error never means `ALLOW`.

A useful coding-agent posture is intentionally unsurprising:

```text
workspace.read     -> ALLOW
test.run           -> ALLOW
git.status         -> ALLOW
git.commit         -> ALLOW when policy permits
git.push           -> ASK
git.merge          -> ASK
git.force_push     -> DENY
deploy.production  -> DENY
```

The exact policy remains explicit data rather than an agent convention.

## Capability manifests and scopes

```powershell
.\tethers.exe validate-manifest .\examples\gary-worker-manifest.json
Get-Content .\examples\gary-worker-request.json -Raw |
  .\tethers.exe evaluate --policy .\policies\default.json --manifest .\examples\gary-worker-manifest.json
```

An undeclared or unknown Capability is denied.

A scope can narrow authority with `allowed_actions` and exact workspace-relative `allowed_files`; path traversal, absolute paths, and missing paths under a file scope fail closed.

Project policies can narrow authority further. Built-in hard denies cannot be broadened by a project policy.

## Integration examples

GARY workers may inspect and edit explicitly scoped files, test, inspect Git, and commit when the packet permits it. Push and merge can return `ASK`; force push, unrelated files, secrets, destructive operations, and production deployment can return `DENY`.

Resolve AI can map `workspace.read`, `apply_patch`, `test.run`, `git.status`, `git.diff`, and `git.commit` to `ALLOW`; `git.push` and `git.merge` to `ASK`; and `deploy.production` to `DENY`.

CALL-E can invoke the executable as a subprocess and map only the three known decisions.

The wrappers under `wrappers/` are intentionally thin subprocess adapters and do not duplicate policy evaluation.

## Profiles, registry, and translators

The explicit profiles live under `policies/`; the action registry is `registry/capabilities.json`.

The official thin translators in `plugs/` cover Git, filesystem containment, process/shell, HTTP, secrets, containers, generic tools, MCP fingerprints, databases, messaging/email, and deployment request shapes.

They construct requests only and never execute them.

This distinction is important: a translator making a request is not a second authority engine and is not evidence that the underlying operation happened.

## Build, test, parity, and package

```powershell
cargo test --locked
cargo build --release --locked
pwsh -NoProfile -File .\scripts\package-portable.ps1
python scripts/run_parity.py --windows .\target\release\tethers.exe --linux .\target\release\tethers.exe
python scripts/benchmark.py .\target\release\tethers.exe
```

Windows produces a self-contained `windows-x64` bundle with a deterministic ZIP. Linux CI builds the `x86_64-unknown-linux-musl` bundle reproducibly.

The package layout is `bin/`, `policies/`, `schemas/`, `examples/`, wrapper sources, documentation, `VERSION`, and `SHA256SUMS`.

## Relationship to Tethers 0.5

The workbench remains separately versioned at **0.2.2** for compatibility even though the wider project has a published **Tethers 0.5** practical release line.

The 0.5 host bundle includes this smaller workbench alongside the full native host and agent-facing manuals.

Latest published product release:

[Tethers 0.5 (`tethers-v0.5.8`)](https://github.com/matthewjameswatkins1978-cyber/tethers-lang/releases/tag/tethers-v0.5.8)

The default `main` branch currently points to an earlier checkpoint than the published 0.5 tag. Use the tagged source when reproducing the exact 0.5 host bundle; use this directory's 0.2.2 contract when embedding the portable authority workbench itself.

## Versioning

This portable release is `0.2.2`, based on the published Portable 0.2.1 lineage. Do not overwrite or retag `tethers-portable-v0.1.0`.

The portable version is intentionally not the Human Tether language version and not the wider product release number.

## Read next

- [`AI-INTEGRATION.md`](AI-INTEGRATION.md) - caller-owned execution contract.
- [`docs/CLI.md`](docs/CLI.md) - exact command and exit-code contract.
- [`../../README.md`](../../README.md) - the whole Tethers product story.
- [`../../docs/SECURITY.md`](../../docs/SECURITY.md) - wider host trust boundary and limitations.
