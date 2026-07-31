# Worker Note — J16B clean toolchain reconstruction and runtime builds

Task: `J16B - reconstruct the clean toolchains and build both runtimes`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `75186ce4413c0fbf860d258b86d7adecadcff780`

Implementation checkpoint: `354c46b35ecbbcff7fb18e38eecfaf4af2733e36`

## Requested outcome

Recreate the committed native Windows OCaml and Rust environments in the fresh
checkout, build both runtimes, and demonstrate unchanged committed inputs.

## Changes made

- Created the one local path-bound OCaml switch at `D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\engine-ocaml` and installed only locked dependencies.
- Built the OCaml runtime through that switch and the Rust runtime from `D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\host-rust` in locked debug and release modes.
- Replaced the completed J16A packet with the J16B result and added this worker note; no source, manifest, lock, test, or script file changed.

## Decisions and assumptions

- Starting branch and SHA: `codex/j16-clean-checkout-proof` at `354c46b35ecbbcff7fb18e38eecfaf4af2733e36`; `origin/main` remained `75186ce4413c0fbf860d258b86d7adecadcff780`, so the branch was `1` ahead and `0` behind.
- Clean preconditions passed: no `_opam` below the OCaml root, no Rust `target`, clean Git status, and committed Cargo.lock and tethers_engine.opam.locked present.
- Exact switch command: `opam switch create "D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\engine-ocaml" ocaml-base-compiler.5.5.0 --yes`. The terminal wrapper timed out at exit `124` without output, but direct post-command inspection proved the expected path-bound switch and OCaml/ocamlopt `5.5.0`; no repair, copy, reset, or neighbouring switch was used.
- Exact locked-install command: `opam install --switch="D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\engine-ocaml" . --deps-only --locked --yes` (exit `0`).
- The locked installation reported only existing package-definition metadata warnings for homepage, bug-reports, and SPDX formatting; no lock regeneration or package pinning occurred.

## Evidence

- Before/after Cargo.lock SHA-256: `D323870EA02F09391A5D0D9AA0E9A701CF686A5AC005B840EE7218E70EDB5602` / `D323870EA02F09391A5D0D9AA0E9A701CF686A5AC005B840EE7218E70EDB5602`.
- Before/after tethers_engine.opam SHA-256: `54D0FB7C8A88DC90DD61D1033672F6B74DCF1E7BB06E3781704F2A9CD6ABB87A` / `54D0FB7C8A88DC90DD61D1033672F6B74DCF1E7BB06E3781704F2A9CD6ABB87A`.
- Before/after tethers_engine.opam.locked SHA-256: `CC2C2F1818E8A4E9AF1FFEDE4F384514384C319AEAADA76A2D2A715D9D19C495` / `CC2C2F1818E8A4E9AF1FFEDE4F384514384C319AEAADA76A2D2A715D9D19C495`.
- Before/after rust-toolchain.toml SHA-256: `7C3E6D894826E0E8846092BB8E037303CD71B4CA210BF70F64D9BC4B7C819969` / `7C3E6D894826E0E8846092BB8E037303CD71B4CA210BF70F64D9BC4B7C819969`.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File .\.github\scripts\check-tethers-toolchains.ps1 -OcamlSwitchPath "D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\engine-ocaml"` — PASS: all Rust, local-switch, version, lock, and project checks.
- `opam exec --switch="D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\engine-ocaml" -- ocamlc -version` — `5.5.0`; `ocamlopt -version` — `5.5.0`; `dune --version` — `3.24.0`; installed Yojson — `2.2.2`.
- `opam exec --switch="D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\engine-ocaml" -- dune build` — PASS, `00:00:03.7795542`, exit `0`.
- `rustup run 1.89.0 cargo build --locked` — PASS, `00:00:19.5149661`, exit `0`.
- `rustup run 1.89.0 cargo build --release --locked` — PASS, `00:00:16.2295630`, exit `0`.
- `RUSTUP_AUTO_INSTALL` was process-locally set to `0` for both Rust builds and its prior state was restored in `finally`: `True`.
- `_opam` exists only at the named OCaml switch root; `target` exists only at the named fresh Rust root.
- Final packet checker — PASS; final status before commit contains only the two authorised documentation paths.

## Discoveries

- The existing Rust build emits warnings without errors: 3 library warnings; debug binary 10 warnings; release binary 8 warnings. These were observed only and were outside J16B scope.

## Remaining risks

- No tests, J15 consolidated matrix, restart proof, replay proof, or complete clean verification ran. Those remain deferred; J16C and J17 have not begun.

## Smallest next action

J16C: run the separately authorised restart, replay, and remaining clean-verification proof from this reconstructed native Windows checkout.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-07-31-j16a-clean-checkout-baseline.md`
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`
- `docs/OCAML_GUIDE_FOR_AGENTS.md`
- `tethers-0.1/engine-ocaml/tethers_engine.opam.locked`
- `.github/scripts/check-tethers-toolchains.ps1`
- `.github/scripts/check-tethers-task-packet.ps1`
