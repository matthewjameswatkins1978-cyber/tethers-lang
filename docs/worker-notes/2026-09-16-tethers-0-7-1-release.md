# Tethers 0.7.1 Windows release

Task: `TETHERS 0.7.1 / Fast-track Windows release`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `Codex`
Status: `COMPLETE`
Base commit: `63262b8b27b2d48c6b5ef023fa54c826c565dfe2`
Implementation checkpoint: `24a46128e349483209f2ac296d54689fae28d81f`

## Requested outcome

Prepare and publish the bounded Tethers 0.7.1 Windows x86-64 maintenance release from a clean source branch, with deterministic package provenance, a clean-archive smoke test, compatibility evidence, and normal PR/tag/release publication.

## Changes made

- Updated the native product identity from 0.7.0 to 0.7.1 while preserving the separate Portable Workbench 0.2.2 identity and the existing language, wire, plan, trail, manifest, Plug, Socket, and MCP compatibility axes.
- Repaired the generic release packer so its default product version and release-note selection are derived from the repository VERSION and current release note instead of stale 0.6.0 defaults.
- Replaced the 0.7 packer with a deterministic Windows x86-64 packer that requires a clean source tree, an explicit/current OCaml switch, current engine provenance, sorted package contents, a reproducible ZIP, a versioned manifest, and SHA-256 sums.
- Added the 0.7.1 release note and updated living current-release documentation without rewriting historical 0.7.0 evidence.

## Decisions and assumptions

- The official 0.7.1 target is Windows x86-64 only. Linux remains targeted development rather than an official 0.7.1 release claim.
- `engine-ocaml/serverInfo.version` remains 0.2.0 because repository history treats it as the engine adapter identity; the native host/package identity is 0.7.1.
- The fixture provider used by the representative plan smoke does not implement the optional provider initialization command. This is not a release failure because the plan path is side-effect-free and does not require provider execution.

## Evidence

- Repository-owned tool diagnostics passed: `scripts/check-dev-tools.ps1`.
- Task packet checker passed at the implementation checkpoint: `check-tethers-task-packet.ps1`.
- Rust formatting passed: `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`.
- Full `just verify` passed on the release branch with the explicit repository OCaml switch after the final evidence update. It reported current engine provenance, OCaml tests, Rust checks/tests, cross-language tests, protocol fixtures, MCP transcripts, compatibility corpus, warning ratchet, and a passing machine report.
- Candidate package build passed with `scripts/package-0.7.ps1 -OcamlSwitch 'D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml'` from source commit `26c2548`.
- Candidate archive SHA-256: `dc659de4c29b1e4f4b983c5460b42d0b6614df0fb78ab01d4aa58854ad8f1de3`.
- Candidate manifest SHA-256: `9ff2bbbda8b06a786c641cb809806ce4caae8f6bad68ea67fb858cb8b0d660e8`.
- Candidate executable identities: `tethers.exe` `e9a7eadd30ba85f454e11c2f1832f442d6a23ac3294c72cd63a03bbb6906b194`; `tethers-engine.exe` `870f7ed1aae662a514ce4cc5581f14ab72318f05042422edece2f571ee871172`.
- Extracted-archive smoke passed for version, init, doctor, describe, capability discovery, workspace stat/read, and bounded argv execution. The bounded execution used only the extracted packaged executable and matched the packaged executable hash.
- Extracted-archive plan smoke passed with `status=completed`, `schema=tethers.plan/1`, `authority.granted=false`, `execution.performed=false`, and `provider_invocations=0`. No provider marker was written.
- `git diff --check` passed.

## Discoveries

- The generic packer had stale 0.6.0 defaults and copied a stale 0.6 release note; both behaviours are now rejected or corrected from current repository identity.
- The archive smoke plan is valid even though the fixture provider is not an initialization-capable provider; `check` therefore reports `PROVIDER_INITIALIZE_FAILED`, while the side-effect-free `plan` route remains the correct evidence for this smoke.
- Existing duplicate-target Rust output is an accepted warning and did not increase in this bounded release change.

## Remaining risks

- The final authoritative package hashes must be regenerated after the final evidence note and task packet status are committed, so the manifest binds to the final accepted source SHA.
- The public publication steps are governed by the release packet: normal branch push, PR review, merge, exact-source tag, GitHub release assets, remote hash verification, and safe cleanup.

## Smallest next action

Push the verified release branch and inspect the PR checks before the authorised merge and release publication.

## References

- `docs/TETHERS_0_7_1_RELEASE.md`
- `scripts/package-0.7.ps1`
- `scripts/package-tethers-release.ps1`
- `verification/tethers-verification.json`
- `verification/current-engine-provenance.json`
