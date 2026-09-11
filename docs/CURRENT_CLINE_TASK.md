# Tethers 0.7.0 release integration and GitHub publication

Task: `Tethers 0.7.0 release integration and GitHub publication`

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Amber`

Owner: `Codex`

Route: `Integrate the approved 0.7 execution boundary onto current origin/main, resolve release blockers, verify the full runtime, then publish main and the GitHub release through normal Git operations.`

Base commit: `e353cbbbe7599c467e17b6e558f979b6c83a4b39`

Implementation checkpoint: `WORKTREE`

Worker note: `docs/worker-notes/2026-09-11-tethers-0.7-release.md`

Suggested branch:

`release/tethers-v0.7.0`

Source branch:

`origin/main`

Updated: 2026-09-11

## Objective

Integrate Tethers 0.7.0 with the current GitHub main line, preserve the
existing 0.6 host and Plug surfaces, make the 0.7 Agent Core discoverable and
usable, and publish a verified Windows full-runtime release on GitHub.

## Relevant background and existing behaviour

The current main line is the tagged 0.6.0 reference and contains the mature
host, Plug lifecycle, discovery, replay, and cross-language engine surfaces.
The approved 0.7 work adds a host-owned Agent Core for bounded workspace, Git,
exec, Threadmoth, health, and receipt operations plus Windows packaging.

## Required behaviour

1. Integrate the approved 0.7 Agent Core without removing current main-line
   host, Plug, discovery, replay, or cross-language behaviour.
2. Resolve the release-control and packaging issues that prevent a clean,
   reproducible 0.7.0 reference.
3. Publish current main, tag `tethers-v0.7.0`, and attach a verified Windows
   full-runtime package, checksum file, and release manifest on GitHub.

## Frozen decisions and invariants

- Tethers 0.1 language and existing authority, replay, Trail, and Plug
  semantics remain unchanged.
- The Agent Core is host-owned; plans do not grant authority and exec never
  inserts a shell.
- Workspace replacement remains expected-preimage protected; Git remote
  mutation is not exposed.
- The Windows package contains both `tethers.exe` and `tethers-engine.exe`.
- No force push, reset, published-history rewrite, or invented verification.

## Relevant components

- `tethers-0.1/host-rust/`
- `tethers-0.1/engine-ocaml/`
- `scripts/package-0.7.ps1`
- `docs/releases/v0.7.0.md`
- `README.md`, `docs/SECURITY.md`, and `justfile`
- `.github/workflows/`
- `.github/scripts/check-tethers-task-packet.ps1`

## Acceptance criteria

1. The integrated Rust host builds, formats, and its complete locked test
   suite passes; the current OCaml engine builds and the cross-language tests
   pass with that engine present.
2. `just verify` passes with a committed implementation checkpoint and worker
   note that agree with the branch and final evidence.
3. The 0.7.0 Windows full-runtime package contains both executables and an
   exact SHA-256 manifest bound to the published commit.
4. GitHub main contains the integrated release, the consistent
   `tethers-v0.7.0` tag exists, and the GitHub release assets download and
   hash exactly as published.

## Required verification

- `git diff --check`, Rust format/check/test, and the relevant full suite;
- explicit OCaml-switch `dune build @all` and cross-language tests;
- CLI smoke tests for version, help, describe, doctor, init, capabilities,
  bounded workspace/Git/exec surfaces, and package identity;
- task-packet consistency, branch ancestry, tag identity, release assets, and
  final clean-worktree proof.

## Stop conditions

Stop and report if the current main line cannot be integrated without
discarding user work, a required test exposes an unresolved defect after
bounded repair, GitHub rejects normal publication, or an artifact cannot be
proven to match the published commit.

## Forbidden changes

- no Tethers 0.1 syntax or semantic redesign;
- no removal or weakening of current host, Plug, replay, Trail, or engine
  tests;
- no force push, reset, rebase of published history, or blanket cleanup;
- no release claim for an unbuilt or unverified platform.

## Expected pre-existing changes

None.
