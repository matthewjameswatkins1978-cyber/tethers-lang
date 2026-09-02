# Tethers 0.5

Tethers 0.5 is the practical release line: the frozen Tethers Core and host
remain the semantic and authority centre, while the repository now exposes a
usable agent-facing discovery/toolbelt path and an exact Rocket V3 portfolio.

## What is in the release

- Frozen Enc_V2 and ProgramDigest V2 identity, with no semantic format change.
- Rocket V3 R3-2 typed refinement, accepted exact success-path solving, and a
  portfolio seam with explicit runtime-only routing and a permanent exhaustive
  reference mode.
- Native host discovery for descriptions, trusted capability listings and
  contract inspection, plus installed Plug inspection.
- Reviewed workspace/text/hash/patch, structured Git, argv-only process, and
  named verification reference Plugs.
- Deterministic Windows x64 and Linux x64 musl packaging through the pinned Rust
  toolchain and the repository release workflow.

## Tethers and Tether Sets

A Tether is one deterministic behavioural rule. A Tether Set is the installable
collection of related Tethers selected by a host configuration. The Set declares
the exact capability names and versions it requires; a Plug may provide those
capabilities, but neither the Set nor an agent grants itself trust or permission.
The host still validates the Set, binds trusted installed Plugs, applies scope
and policy, executes admitted Plans, and records Trail evidence.

## Exactness boundary

Rocket chooses a backend for runtime reasons only. A backend is allowed to emit
an answer only when its exactness is established for the supported shape; other
inputs go to the existing frozen V2 exact search, and a bounded-search refusal
falls through to the explicit exhaustive reference engine. The existing V2 IR
search supplies its certified pruning/memoisation machinery without changing
the frozen encoder. The R3-3B3A/B3B/B3C research notes remain research
evidence, not silently promoted production theorems.

## Install

Download the asset for the platform from the GitHub release, verify its
`.sha256` file, and extract it. The bundle contains the native host in `bin/`,
the smaller ALLOW / ASK / DENY workbench in `portable/`, and the front-door
manuals in `docs/`.

On Windows, the same bundle can be rebuilt locally with:

```powershell
pwsh -NoProfile -File .\scripts\package-tethers-release.ps1 -Target windows-x64
```

Linux x64 musl packages are built and tested by GitHub Actions because the
Windows development machine is not treated as a Linux toolchain.

## Verification record

The bounded local evidence before tagging is recorded in the
[cold-agent transcript](evidence/tethers-0.5-cold-agent-transcript.md) and
[Rocket benchmark record](evidence/tethers-0.5-rocket-benchmark.md).
The final release record is completed from the tagged commit with exact asset
SHA-256 values, the workflow run URL, Rocket differential totals, native host
test totals, and final local/remote Git identities. No CI-only or physical
installation result is implied before those values are recorded.
