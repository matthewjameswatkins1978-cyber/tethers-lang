# Tethers Portable 0.1

This is the small machine-facing decision façade for the current Tethers host
policy model. It is additive: the OCaml Core parser/evaluator, the existing
reference-host commands, and the Tethers language are unchanged.

The façade proposes a policy decision only. It never executes an Action, reads
live state, starts a server, or writes a Trail.

## Contract

Run one evaluation per process:

```powershell
Get-Content .\examples\allow.json -Raw | .\tethers.exe evaluate --policy .\policies\default.json
```

Input may be read from stdin or from `--input PATH`. `--policy PATH` is
optional and overrides an embedded `policy` object. Without either policy
source, the result is `DENY` with `error: "missing policy"`.

```json
{
  "action": { "name": "git.push", "version": 1 },
  "context": { "branch": "main" },
  "policy": {
    "default": "deny",
    "rules": [
      {
        "name": "git.push",
        "version": 1,
        "decision": "ask",
        "reason": "remote mutation requires operator approval"
      }
    ]
  }
}
```

The response is exactly one JSON document on stdout:

```json
{
  "decision": "ASK",
  "rule": "git.push@1",
  "reason": "remote mutation requires operator approval"
}
```

Valid policy outcomes are `ALLOW`, `ASK`, and `DENY`. A malformed request,
missing or invalid policy, invalid action, unknown field/condition, duplicate
rule, or evaluator failure returns `DENY` with an `error` field. A valid
explicit `DENY` is a normal decision and has no `error` field. Context is
validated as an object and remains host-owned opaque data at this seam.

The policy shape intentionally mirrors current Tethers host-local policy:
exact action name/version rules override a default posture. There is no fuzzy
matching, implicit default allow, condition language, execution, networking,
MCP, database, GUI, plugin discovery, or daemon.

## Build and test

```powershell
cargo test --locked --manifest-path .\Cargo.toml
cargo build --release --locked --manifest-path .\Cargo.toml
```

The Windows release executable is built with the MSVC C runtime statically
linked, so the bundle does not require the Visual C++ Redistributable. The
Linux release target is `x86_64-unknown-linux-musl`, which produces a
self-contained x64 Linux executable when built on Linux with the standard
musl tools.

## Packaging

From `tethers-0.1`:

```powershell
pwsh -NoProfile -File .\scripts\package-portable.ps1
```

On Windows this creates the native `windows-x64` bundle and statically links
the MSVC runtime. On a Linux host, use
`-Target linux-x64-musl` to create the self-contained Linux bundle; that
requires the usual Rust musl target and `musl-tools` package. The repository
CI workflow builds and packages both targets reproducibly. Each zip contains
the executable, `policies/`, `examples/`, this README, `VERSION`, and
`SHA256SUMS`.
