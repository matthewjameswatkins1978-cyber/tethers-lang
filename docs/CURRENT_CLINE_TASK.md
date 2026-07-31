# Current Implementation Task

Control contract: `1`

Task: `J16B - reconstruct the clean toolchains and build both runtimes`

Owner: `Codex`

Status: `COMPLETE`

Task colour: `Red`

Route: `Codex native Windows clean-build proof`

Base commit: `75186ce4413c0fbf860d258b86d7adecadcff780`

Branch: `codex/j16-clean-checkout-proof`

Worker note: `docs/worker-notes/2026-07-31-j16b-clean-build.md`

## Objective

Reconstruct the committed native Windows toolchains in the clean checkout and
prove both OCaml and Rust runtimes build without changing source or locks.

## Relevant background and existing behaviour

J16A established the clean checkout at `75186ce4413c0fbf860d258b86d7adecadcff780`.
The J16B starting SHA was `354c46b35ecbbcff7fb18e38eecfaf4af2733e36`, one
commit ahead of `origin/main`. The OCaml switch must remain a path-bound local
environment beneath this checkout.

## Required behaviour

1. Create the exact path-bound OCaml 5.5.0 switch at the fresh OCaml root.
2. Install only the committed locked OCaml dependencies and pass the repository toolchain gate.
3. Build the OCaml runtime through the local switch.
4. Build the Rust runtime in locked debug and release modes with process-local `RUSTUP_AUTO_INSTALL=0` restoration.
5. Prove generated `_opam` and `target` directories are confined to the fresh checkout and the four committed input hashes are unchanged.
6. Update only this packet and the named worker note after all environment and build checks pass.

## Relevant components

- `tethers-0.1/engine-ocaml/tethers_engine.opam`, `tethers_engine.opam.locked`, and `dune-project`.
- `rust-toolchain.toml`, `tethers-0.1/host-rust/Cargo.toml`, and committed `Cargo.lock`.
- `.github/scripts/check-tethers-toolchains.ps1` and `.github/scripts/check-tethers-task-packet.ps1`.

## Frozen decisions and invariants

- Fresh checkout: `D:\The Next Thing\Tethers Lang - J16 Clean`.
- OCaml switch root: `D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\engine-ocaml`.
- Rust root: `D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\host-rust`.
- Only `docs/CURRENT_CLINE_TASK.md` and `docs/worker-notes/2026-07-31-j16b-clean-build.md` may change.
- The exact locked dependency command installed OCaml `5.5.0`, Dune `3.24.0`, and Yojson `2.2.2`; the toolchain gate passed all checks.
- OCaml build passed in `00:00:03.7795542` (exit `0`); Rust debug passed in `00:00:19.5149661` (exit `0`); Rust release passed in `00:00:16.2295630` (exit `0`).
- SHA-256 values were byte-identical before and after: Cargo.lock `D323870EA02F09391A5D0D9AA0E9A701CF686A5AC005B840EE7218E70EDB5602`; opam `54D0FB7C8A88DC90DD61D1033672F6B74DCF1E7BB06E3781704F2A9CD6ABB87A`; opam.locked `CC2C2F1818E8A4E9AF1FFEDE4F384514384C319AEAADA76A2D2A715D9D19C495`; rust-toolchain `7C3E6D894826E0E8846092BB8E037303CD71B4CA210BF70F64D9BC4B7C819969`.
- No tests, J15 matrix, restart proof, replay proof, lock regeneration, or source, manifest, lock, test, or script modification occurred. J16C and J17 have not begun.

## Acceptance criteria

1. The J16B starting branch, SHA, and clean preconditions are recorded.
2. The switch and locked-install commands, exact local paths, and locked OCaml/Dune/Yojson versions are recorded.
3. The repository toolchain gate passes.
4. OCaml, Rust debug, and Rust release builds each pass with recorded duration and exit `0`.
5. `RUSTUP_AUTO_INSTALL` restoration, confined generated directories, and all four byte-identical input hashes are recorded.
6. No forbidden build, test, verification, restart, replay, lock, manifest, source, or script action occurred.
7. Only the two authorised documentation paths change and final packet/whitespace checks pass before one commit and push.

## Required verification

- `pwsh -NoProfile -ExecutionPolicy Bypass -File .\.github\scripts\check-tethers-toolchains.ps1 -OcamlSwitchPath "D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\engine-ocaml"`
- `opam exec --switch="D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\engine-ocaml" -- dune build`
- `rustup run 1.89.0 cargo build --locked`
- `rustup run 1.89.0 cargo build --release --locked`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File .\.github\scripts\check-tethers-task-packet.ps1`
- `git diff --check`, `git diff --name-status`, and `git status --porcelain=v1 --untracked-files=all`

## Forbidden changes

- Any repository path other than the two authorised documentation paths.
- Rust or OCaml tests, Cargo format checks, Clippy, J15 consolidated verification, restart proof, replay proof, J16C, or J17.
- `opam update`, `opam upgrade`, lock regeneration, package pinning, copying or using a neighbouring switch, moving the engine directory, or changing the global default switch.
- Pushing `main`, force-pushing, or creating a second completion commit.

## Stop conditions

- Any preflight, clean-build precondition, locked-install, toolchain-gate, build, hash, artifact-location, or final-check requirement fails.
- Any source, lock, manifest, test, script, or unauthorised repository path changes.
- Two materially similar failed attempts occur; return the exact evidence and smallest unresolved issue.

## Expected pre-existing changes

None. J16B began from the clean J16A proof commit.

## Commit and publication boundary

Create exactly one commit:

`build: prove clean j16 runtime builds`

Push only `codex/j16-clean-checkout-proof`. Do not push `main`, create a second
completion commit, run a test suite after committing, or begin J16C.

## Return contract

Return `COMPLETE` with the commit SHA, exact switch path, OCaml/Dune/Yojson
versions, toolchain-gate result, three build durations and exits, four unchanged
hashes, changed paths, ahead/behind state, and final worktree status. Otherwise
return `BLOCKED` with the exact failed command, exit code, and smallest
environmental or lockfile issue. Stop after reporting.
