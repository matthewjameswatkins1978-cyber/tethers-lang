# TETHERS 0.7.1 - FAST-TRACK WINDOWS RELEASE

Task: `TETHERS 0.7.1 / Fast-track Windows release`

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Red`

Owner: `Codex`

Route: `Prepare, verify, publish and record the bounded Windows x64 0.7.1 maintenance release from fresh origin/main. Preserve all language, protocol, package and platform boundaries.`

Base commit: `63262b8b27b2d48c6b5ef023fa54c826c565dfe2`

Evidence checkpoint: `24a46128e349483209f2ac296d54689fae28d81f`

Worker note: `docs/worker-notes/2026-09-16-tethers-0-7-1-release.md`

Suggested branch:

`release/tethers-v0.7.1`

## Objective

Ship the current corrected Tethers 0.7 runtime as a truthful Windows x86-64
maintenance release. Product identity becomes 0.7.1. Human Tether language,
Core/wire protocol, Plan, Trail, Capability manifest, Plug package/socket and
MCP protocol identities remain unchanged.

## Relevant background and existing behaviour

Fetched `origin/main` is the accepted release starting point. The current
product identity is 0.7.0. The generic release packer still defaults to 0.6.0
and copies the stale 0.6 release note. The 0.7-specific packer is hard-coded
to 0.7.0 and uses non-deterministic archive creation. Existing 0.7 release
history is evidence only and must not be rewritten.

## Required behaviour

1. Change authoritative product/release identity to 0.7.1 without changing
   independent language, protocol, Plan, Trail, manifest, Plug, Socket, MCP or
   Portable Workbench identities.
2. Repair release packaging so the current release note is included, stale
   0.6 references are not silently bundled, version mismatches fail clearly,
   and artifacts are deterministic and bound to the exact source SHA.
3. Create the 0.7.1 release note with the Windows-only support promise and
   explicit non-claims for Linux, MCP upgrade, Resolve and Lantern.
4. Run the authoritative verification and direct compatibility, package,
   checksum and clean-package smoke evidence.
5. Inspect merged PR #40 only for contradiction with the accepted R0/R1 Host
   model; make no architecture project or Lantern/OpenShell redesign.
6. Publish the reviewed branch through a normal PR merge, then create the
   immutable `tethers-v0.7.1` tag and GitHub release with the required assets.

## Relevant components

- `VERSION`
- `tethers-0.1/host-rust/Cargo.toml`
- `tethers-0.1/host-rust/Cargo.lock`
- `scripts/package-tethers-release.ps1`
- `scripts/package-0.7.ps1`
- `scripts/verify-tethers.ps1`
- `scripts/check-compatibility-corpus.ps1`
- `docs/VERSIONING.md`
- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`
- `docs/PROJECT_OVERVIEW.md`
- `docs/SECURITY.md`
- `docs/ROAD_TO_1_0.md`
- `README.md`
- `QUICKSTART.md`
- `docs/AGENT_QUICKSTART.md`
- `docs/TETHERS_0_7_1_RELEASE.md`

## Frozen decisions and invariants

- This is a Windows x86-64 release only. Linux is not an official 0.7.1
  runtime target.
- Tethers Core remains deterministic and application-agnostic.
- A Plan remains a request, not permission; provider execution and Trail truth
  remain unchanged.
- Existing MCP protocol support remains the declared Tethers revision.
- Resolve and Lantern are not required product components.
- Existing historical release notes and evidence remain historical records.
- No force-push, history rewrite, direct main update, or unrelated cleanup.
- The tag must never be moved after creation.

## Acceptance criteria

1. Product identity reports 0.7.1 in every authoritative current product
   surface; independent semantic/protocol identities remain unchanged.
2. No stale 0.6 release document is included by the current Windows packer,
   and packer version/source checks fail closed.
3. `just verify` passes with release eligibility, required suites, clean
   source and current engine provenance.
4. Cargo formatting/checking, packet checker, compatibility corpus, MCP
   transcripts and `git diff --check` pass.
5. `Tethers-0.7.1-windows-x64.zip`, its manifest, `SHA256SUMS` and archive
   hash are produced from one exact release source SHA.
6. A clean extracted package passes version, init, doctor, describe, capability
   discovery, side-effect-free plan, representative workspace read, bounded
   argv execution where promised, and engine launch checks without developer
   tree binaries.
7. A representative supported 0.7 input preserves Plan meaning, Action order,
   authority semantics, package major and Trail truth.
8. The PR diff is reviewed and merged normally; the tag points to the merged
   release source and the GitHub release assets download with matching hashes.
9. The worker note records exact SHAs, artifacts, evidence, exclusions and
   remaining 0.8 work, and the final worktree is clean.

## Required verification

Run from the release worktree:

- `pwsh -NoProfile -File scripts/check-dev-tools.ps1`
- `pwsh -NoProfile -File .github/scripts/check-tethers-toolchains.ps1`
- `just verify`
- Cargo fmt/check/build and relevant locked release tests
- task packet checker
- compatibility corpus
- MCP transcript suite
- `git diff --check`
- deterministic package build and manifest/checksum verification
- clean extracted-package smoke using only packaged binaries
- complete branch diff review against `origin/main`
- PR checks/diff review, merged-main SHA, tag target and downloadable asset
  hash verification

## Forbidden changes

- No 0.8 programme work, Linux parity, MCP protocol upgrade, new syntax,
  generic Host SDK, new package format, Resolve rewrite or Lantern Plug.
- No changes to OCaml/Core semantics, Plan meaning, Trail/result taxonomy,
  replay, provider ordering or authority semantics.
- No dependency refresh unless a demonstrated release blocker requires it.
- No rewriting historical release docs merely to change their recorded version.
- No force-push, history rewrite, direct main mutation or stale-branch
  continuation.

## Stop conditions

Stop before publication if product/version axes cannot be classified without
changing semantics; current verification does not pass; the engine cannot be
proven current; the package smoke depends on developer-tree binaries; source,
manifest or checksums disagree; PR #40 contradicts R0/R1 in shipped behaviour;
GitHub reveals an unexpected diff/check failure; or tag/release publication
would be based on a different source SHA than the verified artifacts.

## Expected pre-existing changes

None

## Implementation scope

- `VERSION`
- `tethers-0.1/host-rust/Cargo.toml`
- `tethers-0.1/host-rust/Cargo.lock`
- `scripts/package-tethers-release.ps1`
- `scripts/package-0.7.ps1`
- `docs/VERSIONING.md`
- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`
- `docs/PROJECT_OVERVIEW.md`
- `docs/SECURITY.md`
- `docs/ROAD_TO_1_0.md`
- `README.md`
- `QUICKSTART.md`
- `docs/AGENT_QUICKSTART.md`
- `docs/TETHERS_0_7_1_RELEASE.md`
