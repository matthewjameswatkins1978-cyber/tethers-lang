# Tethers 0.6

Tethers 0.6 is the current practical product release line. It keeps the
frozen Tethers 0.1 language/Core semantics and the native host and Portable
Workbench compatibility identities at 0.2.2, while making the agent-facing
Plan, execution, replay, and Trail proof path the primary product journey.

## What is in the release

- Deterministic `tethers plan` with the `tethers.plan/1` machine-readable
  result and no authority, provider, or Trail side effects.
- Explicit run approval/execution through the trusted native host, with
  durable replay protection and bounded Trail receipt inspection.
- The existing discovery, Capability, Plug, workspace, Git, process, and
  verification surfaces from the practical platform line.
- Frozen Core and ProgramDigest identity, Rocket portfolio exactness, and
  bounded Together concurrency without changing the language version.
- Deterministic Windows x64 and Linux x64 musl packaging with SHA-256
  manifests and sidecar checksums.

## Cold-agent proof

The release proof starts with a fresh workspace and follows:

```text
describe/check -> plan -> explicit run -> Trail/receipt -> exact replay
```

The plan stage proves zero provider invocations and zero Trail execution
entries. The completed local scenario then proves one provider effect, a
Result Anchor, a bounded Trail receipt, and replay that returns the original
execution identity without repeating the provider effect.

The exact commands and observed values are recorded in
`docs/evidence/tethers-0.6-cold-agent-proof.md`.

## Compatibility identities

| Surface | Identity |
| --- | --- |
| Human Tether language/protocol semantics | `0.1` |
| Rust reference-host package | `0.2.2` |
| Portable Workbench | `0.2.2` |
| Public product release line | `0.6` |

## Install and verify

Download the platform asset from the `tethers-v0.6.0` GitHub release, verify
the adjacent `.sha256` file, and extract it. The same Windows bundle can be
rebuilt locally with:

```powershell
pwsh -NoProfile -File .\scripts\package-tethers-release.ps1 -Target windows-x64
```

Linux x64 musl packaging is built by the release workflow on GitHub Actions.
