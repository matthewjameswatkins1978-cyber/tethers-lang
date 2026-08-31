# Tethers Portable 0.2.0

Tethers is a small, portable authority layer for local AI workbenches. Given
an actor, action, resource, and context it returns `ALLOW`, `ASK`, or `DENY`.
It decides authority; it does not execute actions. There is no server, daemon,
database, scheduler, agent loop, MCP service, telemetry, or LLM evaluation.

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

The response is machine-readable and includes the schema version, decision,
matched rule, deterministic reason, and policy identity. `ASK` is reserved for
a deliberate policy requirement for human authority. Malformed requests,
unknown actions, invalid policies, missing binaries, timeouts, malformed
responses, and other operational uncertainty must be treated as `DENY` by
callers.

Portable 0.1 requests using `{ "action": { "name": "...", "version": 1 },
"context": {}, "policy": {} }` remain accepted. The frozen 0.1 tag and
artifact are not modified.

## Explain mode and policy tests

```powershell
.\tethers.exe explain --input .\examples\gary-worker-request.json --policy .\policies\default.json
.\tethers.exe test .\policies\default.json .\examples\workbench-policy-tests.json
.\tethers.exe test .\policies\default.json .\examples\workbench-policy-tests.json --json
```

Explain output adds `evaluated_conditions`. Sensitive values are never echoed.
The policy runner exits zero only when every case passes and reports expected,
actual, and matched rule on failures.

## Capability manifests and scopes

```powershell
.\tethers.exe validate-manifest .\examples\gary-worker-manifest.json
Get-Content .\examples\gary-worker-request.json -Raw |
  .\tethers.exe evaluate --policy .\policies\default.json --manifest .\examples\gary-worker-manifest.json
```

An undeclared or unknown capability is denied. A scope can narrow authority
with `allowed_actions` and exact workspace-relative `allowed_files`; path
traversal, absolute paths, and missing paths under a file scope fail closed.
Project policies can narrow authority further. Built-in hard denies cannot be
broadened by a project policy.

## Integration examples

GARY workers may inspect and edit explicitly scoped files, test, inspect Git,
and commit when the packet permits it. Push and merge return `ASK`; force push,
unrelated files, secrets, destructive operations, and production deployment
return `DENY`.

Resolve AI can map `workspace.read`, `apply_patch`, `test.run`, `git.status`,
`git.diff`, and `git.commit` to `ALLOW`; `git.push` and `git.merge` to `ASK`;
and `deploy.production` to `DENY`.

CALL-E can invoke the executable as a subprocess and map only the three known
decisions. The wrappers under `wrappers/` are intentionally thin subprocess
adapters and do not duplicate policy evaluation.

## Build, test, and package

```powershell
cargo test --locked
cargo build --release --locked
pwsh -NoProfile -File .\scripts\package-portable.ps1
```

Windows produces a self-contained `windows-x64` bundle with a deterministic
ZIP. Linux CI builds the `x86_64-unknown-linux-musl` bundle reproducibly. The package layout is
`bin/`, `policies/`, `schemas/`, `examples/`, wrapper sources, documentation,
`VERSION`, and `SHA256SUMS`.

## Versioning

This release is `0.2.0`, based exactly on Portable 0.1.0 commit
`b62f8f3e31b319c13ce281e913f45f40640aad58`. Do not overwrite or retag
`tethers-portable-v0.1.0`.
