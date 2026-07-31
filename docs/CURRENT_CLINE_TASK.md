# Current Implementation Task

Control contract: `1`

Task: `J16A - establish the clean native Windows checkout baseline`

Owner: `Codex`

Status: `COMPLETE`

Task colour: `Red`

Route: `Codex native Windows release-engineering baseline`

Base commit: `75186ce4413c0fbf860d258b86d7adecadcff780`

Branch: `codex/j16-clean-checkout-proof`

Worker note: `docs/worker-notes/2026-07-31-j16a-clean-checkout-baseline.md`

## Objective

Establish and document a clean native Windows checkout at the accepted J16 base,
ready for the separately authorised J16B clean build.

## Relevant background and existing behaviour

J16 requires a clean native Windows proof. The original integration worktree is
not part of this task. The OCaml directory switch is path-bound and must not be
moved, copied, selected, or recreated for this baseline.

## Required behaviour

1. Clone the configured `origin` remote into `D:\The Next Thing\Tethers Lang - J16 Clean` at the exact base commit.
2. Confirm the fresh checkout and published proof branch both resolve to `75186ce4413c0fbf860d258b86d7adecadcff780` and begin clean.
3. Record the exact native Windows, PowerShell, Git, Rustup, Rust, Cargo, and opam inventory.
4. Record the availability result of default `opam exec` OCaml and Dune without creating or restoring a switch.
5. Record that no install, update, Rust or OCaml build, verification suite, restart proof, or replay proof occurred.
6. Update only this packet and the named worker note.

## Relevant components

- `AGENTS.md`, `docs/PROJECT_CONTROL.md`, `docs/AGENT_WORKFLOW.md`, and `docs/ROAD_TO_0_2.md`.
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md` and `docs/OCAML_GUIDE_FOR_AGENTS.md`.
- `.github/scripts/check-tethers-task-packet.ps1`.

## Frozen decisions and invariants

- Fresh checkout path: `D:\The Next Thing\Tethers Lang - J16 Clean`.
- Configured origin remote: `https://github.com/matthewjameswatkins1978-cyber/tethers-lang.git`.
- Exact clean starting SHA: `75186ce4413c0fbf860d258b86d7adecadcff780`.
- The only authorised paths are `docs/CURRENT_CLINE_TASK.md` and `docs/worker-notes/2026-07-31-j16a-clean-checkout-baseline.md`.
- Windows: `Microsoft Windows [Version 10.0.22631.6199]`; PowerShell: `7.6.4`; Git: `2.54.0.windows.1`.
- Rustup: `1.29.0 (28d1352db 2026-03-05)`; toolchains: `stable-x86_64-pc-windows-msvc (default)` and `1.89.0-x86_64-pc-windows-msvc (active)`; Rust: `1.89.0 (29483883e 2025-08-04)`; Cargo: `1.89.0 (c24e10642 2025-06-23)`.
- opam: `2.5.2`; default `opam exec -- ocamlc -version` and `opam exec -- dune --version` both report no current switch and exit `50`.
- No build, verification suite, restart proof, replay proof, install, update, or opam switch creation occurred. J16B performs the clean build; J17 has not begun.

## Acceptance criteria

1. The clone path and configured origin remote are recorded exactly.
2. The starting local SHA, `origin/main`, and published proof-branch SHA are recorded as `75186ce4413c0fbf860d258b86d7adecadcff780`.
3. The starting checkout status is recorded as clean.
4. The complete requested Windows and toolchain inventory is recorded exactly.
5. OCaml and Dune default-opam availability is recorded honestly, including exit `50` and the absent current switch.
6. The record states that no installation, update, build, test, verification suite, switch creation, restart proof, or replay proof was performed.
7. Only the two authorised documentation paths change.
8. The task-packet checker and `git diff --check` pass before the one authorised commit and push.

## Required verification

- `pwsh -NoProfile -ExecutionPolicy Bypass -File .\.github\scripts\check-tethers-task-packet.ps1`
- `git diff --check`
- `git diff --name-status`
- `git status --porcelain=v1 --untracked-files=all`

No Rust or OCaml build, test, verification-suite, restart, or replay command is authorised for J16A.

## Forbidden changes

- Any path other than `docs/CURRENT_CLINE_TASK.md` and `docs/worker-notes/2026-07-31-j16a-clean-checkout-baseline.md`.
- Rust or OCaml builds, tests, verification suites, restart proof, or replay proof.
- Installing or updating software, running `rustup update`, or creating, moving, copying, restoring, or selecting an opam switch.
- Moving `tethers-0.1/engine-ocaml`, modifying the original integration worktree, pushing `main`, or beginning J16B or J17.

## Stop conditions

- The required local branch, local `HEAD`, `origin/main`, or published proof-branch SHA differs from the exact base.
- The checkout is not clean before authorised documentation changes.
- An unauthorised path changes, a required final check fails, or a required Git operation fails.
- Two materially similar failed attempts occur; return exact evidence and the smallest unresolved question.

## Expected pre-existing changes

None. The fresh checkout began clean at the exact base commit.

## Commit and publication boundary

Create exactly one commit:

`docs: establish j16 clean checkout baseline`

Push only `codex/j16-clean-checkout-proof`. Do not push `main`, create a second
completion commit, or begin J16B.

## Return contract

Return `COMPLETE` with the checkout path, commit SHA, branch, exact toolchain
versions, OCaml/Dune availability, packet-checker result, changed paths,
ahead/behind state, and worktree cleanliness. Otherwise return `BLOCKED` with
the exact failed requirement. Stop after reporting.
