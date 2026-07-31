# Worker Note — J16A clean native Windows checkout baseline

Task: `J16A - establish the clean native Windows checkout baseline`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `75186ce4413c0fbf860d258b86d7adecadcff780`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Create the isolated J16 clean native Windows checkout baseline and record its
exact starting repository and toolchain state without building or testing.

## Changes made

- Created the fresh checkout at `D:\The Next Thing\Tethers Lang - J16 Clean` from the configured origin remote.
- Created and published `codex/j16-clean-checkout-proof` at the exact starting SHA.
- Replaced the completed J15D packet with this checker-compatible J16A record and added this worker note.

## Decisions and assumptions

- Origin URL, with no credentials or tokens: `https://github.com/matthewjameswatkins1978-cyber/tethers-lang.git`.
- The fresh checkout began clean at `75186ce4413c0fbf860d258b86d7adecadcff780`; `origin/main` and the published proof branch resolved to the same SHA.
- The default clone refspec initially tracked only `main`; the already-published proof branch was fetched explicitly into `refs/remotes/origin/codex/j16-clean-checkout-proof` so its required remote SHA could be verified without changing checkout content.
- No opam switch was created, restored, selected, moved, or copied. The path-bound `tethers-0.1/engine-ocaml` directory was not moved.

## Evidence

- `cmd.exe /c ver` — `Microsoft Windows [Version 10.0.22631.6199]` (exit `0`).
- `pwsh.exe --version` — `PowerShell 7.6.4` (exit `0`).
- `git --version` — `git version 2.54.0.windows.1` (exit `0`).
- `rustup --version` — `rustup 1.29.0 (28d1352db 2026-03-05)` (exit `0`).
- `rustup toolchain list` — `stable-x86_64-pc-windows-msvc (default)` and `1.89.0-x86_64-pc-windows-msvc (active)` (exit `0`).
- `rustup run 1.89.0 rustc --version` — `rustc 1.89.0 (29483883e 2025-08-04)` (exit `0`).
- `rustup run 1.89.0 cargo --version` — `cargo 1.89.0 (c24e10642 2025-06-23)` (exit `0`).
- `opam --version` — `2.5.2` (exit `0`).
- `opam exec -- ocamlc -version` — unavailable: `[ERROR] No switch is currently set. Please use 'opam switch' to set or install a switch` (exit `50`).
- `opam exec -- dune --version` — unavailable: `[ERROR] No switch is currently set. Please use 'opam switch' to set or install a switch` (exit `50`).
- No install, update, Rust build, OCaml build, test, verification suite, restart proof, replay proof, or switch creation occurred.

## Discoveries

- The absence of a default opam switch is expected and not a J16A failure. J16B will create or restore the required local switch before the clean native build.

## Remaining risks

- J16A establishes only the checkout and inventory baseline. Clean build, restart, replay, and final verification are intentionally deferred; J17 has not begun.

## Smallest next action

J16B: create or restore the required local OCaml switch and perform the clean native build under its separately authorised packet.

## References

- `AGENTS.md`
- `docs/PROJECT_CONTROL.md`
- `docs/AGENT_WORKFLOW.md`
- `docs/ROAD_TO_0_2.md`
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`
- `docs/OCAML_GUIDE_FOR_AGENTS.md`
- `.github/scripts/check-tethers-task-packet.ps1`
