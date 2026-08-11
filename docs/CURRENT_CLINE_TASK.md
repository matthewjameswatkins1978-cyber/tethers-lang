# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-3A — Validator Dependency & Scope Correction`

Owner: `OpenCode`

Status: `COMPLETE`

Task colour: `Amber`

Route: `OpenCode implementation + evidence → Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-11-core-3a-validator-correction.md`

Base branch: `feature/core-3-static-validator`

Base commit: `45ec528ac214fbdb5528c794541159bd006b8311`

Implementation checkpoint: `c4b42b2e164fd54986c592287904a0d31b4ef1f2`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory

Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1` (run; all tools present)

Rust change class: `RUST_UNCHANGED`

## Objective

Correct four bounded defects found during independent review of CORE-3: (1) Fact dependency DAG construction is semantically wrong; (2) Origin identity uniqueness is not enforced across the complete static Core namespace; (3) Fact input bindings can reference nonexistent Facts without precise rejection; (4) Role resolution is not sufficiently scope-aware.

## Relevant background and existing behaviour

CORE-3 implemented a static Core validator. Independent review identified four defects: Fact dependencies were derived from Origin_provenance alone (sibling Facts incorrectly treated as mutual dependencies); Origin ID uniqueness checked only within program.origin_sites, not across item templates; Fact_from_origin and Fact_through_role bindings did not emit Missing_fact when the referenced Fact ID didn't exist; and Role resolution used a flat global lookup ignoring scope boundaries.

## Required behaviour

1. Correct Fact dependency DAG: derive edges from Action_origin input/output bindings only, not from Origin_provenance alone
2. Global Origin identity uniqueness: check across program.origin_sites and all item_template.origin_sites
3. Fact binding existence: report Missing_fact for Fact_from_origin and Fact_through_role when the referenced Fact does not exist
4. Role scope isolation: program Origins resolve only program-scope Roles; item-template Origins resolve only Roles from their own Item Template

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core_validator.ml` / `.mli` — modified
- `tethers-0.1/engine-ocaml/bin/tethers_core_validator_test.ml` — 9 new tests added

## Frozen decisions and invariants

- CORE-3 validates Core. It never repairs Core.
- Origin_provenance does not create Fact dependency edges.
- Literal_value and Anchor_value introduce no Fact dependency edge.
- Batch aggregate placeholders are not interpreted for dependencies.
- Program Origins resolve program-scope Roles only; item-template Origins resolve same-template Roles only.
- Do not reopen correct CORE-3 behaviour.

## Acceptance criteria

1. Ordinary Origin Fact does not self-cycle (no spurious Fact_dependency_cycle)
2. Real dependency from Action input/output validated OK
3. Real dependency cycle rejected with Fact_dependency_cycle
4. Global program/template Origin collision rejected
5. Missing Fact_from_origin Fact rejected with Missing_fact
6. Missing Fact_through_role Fact rejected with Missing_fact
7. Program cannot use item-template Role (scope isolation)
8. Cross-template Role reference rejected (template isolation)
9. Correct same-template Role reference validates OK

## Required verification

1. Packet checker at closeout: `control-v1/COMPLETE`
2. OCaml build: `dune build`
3. Validator tests: `dune runtest` — 51/51 assertions
4. Existing lowerer tests: passing (same `dune runtest`)
5. Fixture suite: `check-fixtures.ps1` — 64 JSON + 32 JSONL
6. MCP transcript suite: `test-mcp-transcripts.ps1` — 16 cases
7. Whitespace check: `git diff --check`
8. Rust formatter: `cargo fmt --check` (exit 0)
9. Complete diff inspection: only authorised files
10. Git status: clean worktree

## Forbidden changes

No evaluator/protocol/outcome changes. No Rust changes. No runtime wiring. No Core type changes. Do not reopen correct CORE-3 behaviour.

## Stop conditions

Commit CORE-3A. STOP. Do NOT begin CORE-4.

## Expected pre-existing changes

None.
