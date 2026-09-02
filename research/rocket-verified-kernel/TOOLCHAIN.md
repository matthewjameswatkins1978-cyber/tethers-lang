# Rocq Toolchain

Use a dedicated opam switch. Never install Rocq into the Tethers production switch.

Target for Experiment 1:

- Rocq 9.2.0
- Rocq standard library
- OCaml extraction target

Codex startup should:

1. Inspect available opam and compilers.
2. Create an independent switch named something unambiguous such as `rocket-rocq`.
3. Add/update the official Rocq released package repository if required.
4. Install and pin Rocq 9.2.0 using the official `rocq-prover` / `rocq-core` packages.
5. Verify with `rocq -v`.
6. Record exact opam, OCaml and Rocq versions in docs/WORKER_NOTE.md.

Prefer the smallest toolchain that can complete the proof. Do not install Equations, MathComp, QuickChick or other plugins unless a concrete proof obligation requires one.

The extracted OCaml should also be compiled/tested under the existing authorised Tethers OCaml switch as a separate research harness check.

Do not modify the production switch to satisfy Rocq.
