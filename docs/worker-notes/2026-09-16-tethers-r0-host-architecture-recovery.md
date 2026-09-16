# TETHERS R0 - Host Architecture Recovery Worker Note

Task: `TETHERS R0 / Host Architecture Recovery`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `4493e563a81b7c193c0c815db4ac74b81d1c6e59`

Implementation checkpoint: `48f3a54b9362534156a00dfd9d3d8795cd89aae6`

Publication checkpoint independently reviewed and verified on `origin`:
`6593b16717ae4d07bff4191e376a81112598352d`.

## Requested outcome

Recover and freeze the Tethers Host architecture without changing production
code. Preserve the accepted P0-P4b implementation and current repository
content, classify the reference Host versus external application Hosts, and
leave an evidence-backed R1 boundary.

## Changes made

- Added `docs/architecture/TETHERS_HOST_ARCHITECTURE.md` as the durable
  ownership model.
- Added `docs/decisions/ADR_HOST_EXECUTION_OWNERSHIP.md` recording the corrected
  execution-ownership decision.
- Added `docs/recovery/TETHERS_RESOLVE_RECLASSIFICATION.md` classifying current
  Tethers, Resolve and historical integration areas.
- Added `docs/recovery/TETHERS_R0_PRESERVATION.md` with the current worktree,
  branch, pull-request and unreachable-commit audit.
- Narrowly clarified the Core/reference-Host distinction in `README.md`.
- Replaced the active task packet with the R0 scope and acceptance contract.
- Protected all eight currently unreachable commits under
  `refs/archive/tethers-r0/unreachable/<commit-sha>`.

## Decisions and assumptions

The Tethers Rust runtime remains a legitimate, capable reference Host. It is
not mandatory execution architecture for an external application that already
owns its Host lifecycle, such as Resolve01. P0-P4b is retained as historical
implementation frozen for architecture recovery; its original integration
direction is superseded, not invalidated. Lantern work remains paused.

The current fetched audit is authoritative for R0 preservation: 13 registered
worktrees, one dirty worktree, and eight currently unreachable commits. The
dirty 0.7 worktree and its eight untracked release files were not inspected or
altered.

## Evidence

- R0 base and `origin/main` at start: `4493e563a81b7c193c0c815db4ac74b81d1c6e59`.
- First documentation checkpoint: `48f3a54b9362534156a00dfd9d3d8795cd89aae6`.
- Task checker self-tests: 9 passed, 0 failed.
- Task packet checker: pass for the R0 packet.
- Repository tool diagnostic: all required tools found.
- Rust formatting: pass with the repository manifest path.
- `git diff --check`: pass; only normal Windows line-ending notices were
  emitted by Git.
- Source verification with the exact existing OCaml switch
  `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`: pass. The engine
  was built from R0 source commit `4493e563...` and source tree
  `ca06f7363997cbb9feaea9441bcac9b55b5477b7`; engine SHA-256 was
  `3042361e68fe4a40df6825fc64a07125a6634e068a2eb6dcf4c877704abaaca5`.
  The aggregate report recorded 10 PASS, 0 FAIL, 0 skipped.
- A switch audit also found `C:\Users\Matmus` with Yojson 3.0.0. It was not
  used because the repository requires Yojson 2.2.2 for this engine.
- Archive ref audit: 8 refs resolve to the eight listed unreachable commits.
- No production source, OCaml, Core, Plan, syntax, Resolve or Lantern files
  were changed. No branch, tag or worktree was deleted.

## Discoveries

The older preservation packet's 40/10/21 counts are historical, not current:
the fetched repository exposes 13/1/8. The canonical named checkout is
occupied by the historical P3a branch, so R0 used a fresh worktree from
accepted `origin/main` and did not switch the occupied checkout.

The default verifier needs an explicit switch in a fresh worktree. With no
switch it fails before dependent suites with:

`No OCaml switch was supplied. Set TETHERS_OCAML_SWITCH or create the repository-local tethers-0.1/engine-ocaml switch.`

With the exact existing 2.2.2 switch passed explicitly, the engine and all
recorded suites pass. No toolchain was installed or mutated by R0.

## Remaining risks

R0 does not implement the next Resolve Host boundary. The P0-P4b code remains
available, but its reference-Host insertion point must not be reused as the
external Resolve execution architecture without an R1 design. The canonical
checkout remains occupied and must be refreshed safely before a future task.

## Smallest next action

Begin `TETHERS R1 - Resolve Host Boundary Reclassification & Minimal Repair`
from a fresh checkout of accepted `origin/main`, with the R0 ownership model as
the architecture authority.

## References

- `docs/architecture/TETHERS_HOST_ARCHITECTURE.md`
- `docs/decisions/ADR_HOST_EXECUTION_OWNERSHIP.md`
- `docs/recovery/TETHERS_RESOLVE_RECLASSIFICATION.md`
- `docs/recovery/TETHERS_R0_PRESERVATION.md`
- `refs/archive/tethers-r0/unreachable/`
