# OCaml engine discovery repair

## Outcome

The canonical verifier now discovers a compatible installed opam switch when
neither `--ocaml-switch` nor `TETHERS_OCAML_SWITCH` is supplied. It prefers a
checkout-local switch, then inspects the registered opam switches. Selection
requires OCaml 5.5.0, Dune 3.24.0, Yojson 2.2.2, and Digestif 1.3.1. An explicit
selection remains authoritative and fails closed. The verifier builds the
engine from the active checkout and preserves commit/tree/binary provenance.
It does not install or modify toolchains.

The root cause was that `verify-tethers.py` substituted the active checkout's
engine directory on Windows, even though fresh worktrees do not contain its
ignored `_opam` directory. Linux similarly relied on one hardcoded switch
name. A second reporting defect was that the hand-written PATH scan did not
apply Windows `PATHEXT`, so the final toolchain report marked installed `.exe`
tools missing. Tool lookup now uses Python's cross-platform `shutil.which`.

## Verification

- `scripts/test-verify-tethers.py`: passed, including registry discovery,
  incompatible candidates, explicit selection, fail-closed override, and
  Windows-compatible temporary paths.
- `python -m py_compile` for the changed Python files: passed.
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`:
  passed.
- `scripts/verify-tethers.py` on the R2 source base `5a3ba8ce5f3adec56a18b7e161567c664fc559d2`:
  all ten suites passed, including engine build/provenance, OCaml tests, full
  Rust and cross-language tests, R2 gate tests, and protocol checks.
- After correcting Windows executable lookup, a focused live toolchain-status
  check passed for Git, Cargo, Rust, rustfmt, opam, jq, Dune, OCaml 5.5.0, and
  Dune 3.24.0. The full suite was not repeated after this report-only lookup
  correction.
- `git diff --check`: passed.

The full verifier selected the registered switch at
`D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\engine-ocaml` and
produced engine provenance for the current source commit and tree. The switch
provides tools and packages only; the engine binary was built from this task's
worktree.
