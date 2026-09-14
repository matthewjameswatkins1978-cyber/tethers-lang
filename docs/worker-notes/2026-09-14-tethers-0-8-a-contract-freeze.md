# Worker Note

- **Task Packet:** `TETHERS 0.8-A / 1.0 CONTRACT FREEZE` (user-authorised R1)
- **Owner:** Codex
- **Status:** `COMPLETE`
- **Base Commit:** `cd6cc23f2f0d7248bb31a5b650862f26699b2a65`
- **Final Commit:** see the final handoff; this note was written before the
  publication commits
- **Branch / Worktree:** `codex/tethers-0.8-contract-freeze` / `D:\The Next Thing\Tethers Lang - J16 Clean`

## Files Modified

- `README.md`
- `docs/COMPATIBILITY.md`
- `docs/CURRENT_GOAL.md`
- `docs/DECISIONS.md`
- `docs/DEPRECATION_POLICY.md`
- `docs/MCP_PLAN.md`
- `docs/PROJECT_CONTROL.md`
- `docs/PROJECT_DASHBOARD.md`
- `docs/PROJECT_OVERVIEW.md`
- `docs/ROAD_TO_1_0.md`
- `docs/SUPPORT_MATRIX.md`
- `docs/VERSIONING.md`

## Behavioural Result

R1 establishes a living Tethers 1.0 contract set. It defines 1.0 as stability
of the consequential boundary, inventories public contracts and version axes,
states compatibility and deprecation rules, distinguishes current Windows
truth from the Linux target, records non-goals and release classifications, and
links current SemVer, MCP, Apache-2.0, GitHub provenance, and SBOM evidence.
No Core, host, syntax, MCP implementation, licence file, or release workflow
was changed.

## Invariants Preserved

- Tethers Core remains the deterministic planner; hosts remain the authority
  and execution boundary.
- Plans remain requests, not permission, and MCP remains an adapter rather than
  a second evaluator.
- Existing version axes and accepted preflight history remain distinct and
  unsquashed.
- Historical packet/checkpoint records remain historical rather than being
  rewritten to satisfy the current contract.

## Negative Tests Added or Updated

- None. This packet is documentation and contract definition only.

## Commands Executed

- `git fetch origin --prune` — `PASS`; `origin/main` resolved to the required
  base `cd6cc23f2f0d7248bb31a5b650862f26699b2a65`.
- `pwsh -NoProfile -File scripts/check-dev-tools.ps1` — `PASS`; rg, fd, jq, yq,
  gh, just, git, and pwsh available.
- Local Markdown-link audit over changed documents — `PASS`.
- `git diff --check` — `PASS`.
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` —
  `PASS`.
- `just check` — `PASS`; Rust check completed with the existing duplicate
  `main.rs` multi-target warning.
- `just test-rust` — `FAIL`; 1,563 passed, 13 failed, 2 ignored. The first
  repeated production error was `unknown_capability` / `Unknown Capability:
  notify` in the OCaml-backed Core tests; the expected current Core environment
  was not available in this worktree.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` —
  `FAIL` before product checks; first error was `Implementation changed after
  recorded evidence checkpoint; establish a new implementation checkpoint and
  verify again`, listing historical non-closeout paths. No historical evidence
  was rewritten.

## Unrun Checks and Reason

- `cargo fmt --all -- --check` from repository root — `NOT RUN`: the repository
  has no root Cargo manifest; the repository-owned manifest-equivalent check
  above passed.
- OCaml build, fixture suite, engine smoke, MCP transcript suite, and native
  release route — `NOT RUN`: this documentation-only packet did not provide an
  authorised absolute `OcamlSwitchPath`, and this worktree has no `_opam` switch
  or current `tethers_mcp_main.exe`. The ignored executable present is a stale
  July artifact and must not be treated as current build evidence.
- Rocket action-count tests — `NOT RUN`: they were completed by the preceding
  preflight packet and R1 does not change implementation or test code.

## Discoveries

- The current external MCP specification resolves to `2026-07-28`, while the
  existing Tethers adapter and fixtures target `2025-11-25`; R4 owns that
  compatibility pass.
- The packet checker still enforces a historical implementation checkpoint and
  fails before product verification when later accepted evidence paths differ.
- Apache-2.0 remains a recommendation only; no `LICENSE` was added because
  owner approval was not present in the packet.

## Remaining Risks

- R2 must repair or supersede the historical aggregate packet-checker route and
  establish a fresh authoritative verification checkpoint.
- The current worktree needs an explicitly authorised OCaml switch path and a
  fresh engine build before current cross-language evidence can be claimed.

## Recommended Next Action

Proceed to `Tethers 0.8-B — Verification & Release Control` from freshly
accepted `origin/main`, with an explicit OCaml switch path and a decision on the
historical packet-checkpoint contract.
