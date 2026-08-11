# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-4 — Canonicalisation + ProgramDigest`

Owner: `OpenCode`

Status: `IN_PROGRESS`

Task colour: `Amber`

Route: `OpenCode implementation + evidence → Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-11-core-4-canonicalisation.md`

Base branch: `feature/core-3-static-validator`

Base commit: `7e94924d813bb7bd29ff234559cdb590bdddd016`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust change class: `RUST_UNCHANGED`

## Objective

Implement the first authoritative canonical semantic identity layer for Tethers Core: semantic projection, structural fingerprinting, canonical ordering, internal ID assignment, reference rewriting, canonical byte encoding, SHA-256, and ProgramDigest.

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical.ml` — new
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical.mli` — new
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_test.ml` — new
- `tethers-0.1/engine-ocaml/bin/dune` — modified
- `tethers-0.1/engine-ocaml/tethers_engine.opam` — modified (digestif dep)
- `tethers-0.1/engine-ocaml/tethers_engine.opam.locked` — modified

## Required verification

1. OCaml build: `dune build`
2. Canonicalisation tests: `dune runtest`
3. Whitespace check: `git diff --check`
4. Cargo fmt: `cargo fmt --check` (RUST_UNCHANGED)
5. Complete diff inspection: only authorised files
6. Git status: clean worktree

## Forbidden changes

No evaluator/protocol/outcome/CORE-2/CORE-3 changes. No Rust changes. No runtime wiring. No Core type changes. No Human Tethers changes.

## Stop conditions

Commit CORE-4 implementation checkpoint. STOP. Do NOT begin CORE-5.
