# Rocket Verified Kernel Lab

Status: research side project

Purpose: test whether the mathematically delicate parts of Rocket V3 can be specified and proved in Rocq, then extracted to OCaml and compared against the existing Tethers implementation and frozen Enc_V2 oracle.

This directory is deliberately quarantined from the production Tethers build and from docs/CURRENT_CLINE_TASK.md.

## First experiment

Re-prove the already accepted R3-3B2 simple-success-path canonicaliser before attempting any new tree theorem.

Three-way authority:

1. existing hand-written OCaml B2 implementation;
2. Rocq definition + machine-checked proof + extracted OCaml;
3. frozen exhaustive/differential Enc_V2 authority.

All three must agree.

## Toolchain

Use a dedicated opam switch for this lab. Do not modify the existing Tethers OCaml switch.

Target Rocq: 9.2.0 initially, pinned in the research switch.
Extraction target: OCaml.

No extracted code enters production Tethers during Experiment 1.
