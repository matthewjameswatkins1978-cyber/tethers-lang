# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-3 — Static Core Validator`

Owner: `OpenCode`

Status: `COMPLETE`

Task colour: `Amber`

Route: `OpenCode implementation + evidence → Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-11-core-3-static-validator.md`

Base branch: `feature/core-2-human-to-core-lowering`

Base commit: `68c3510188d0a6db464fbb2e1814f0ce87b4bc3b`

Implementation checkpoint: `b9763ad440d4500577535430a0e7f6b3b3d00910`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory

Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1` (run; all tools present)

Rust change class: `RUST_UNCHANGED`

## Objective

Implement a standalone static validator for `Tethers_core.program`. The validator validates that a Core Program is internally well-formed according to current Core semantics. It MUST NOT repair, infer, reorder, canonicalise, or execute.

## Relevant background and existing behaviour

CORE-2 lowered Human Tether AST to `Tethers_core.program` values. The lowerer enforces its own rules during translation but produces a Core Program whose internal consistency across the full Core vocabulary (identities, references, control flow, capability contracts, fact provenance, branches, roles, together, item templates, batch, deadlines) is not yet validated. CORE-3 adds that static validation as a separate pass.

The existing evaluator path remains unchanged. Production still evaluates Human Tether source through the parser and evaluator directly, without Core. CORE-3 is a new side path: parse → lower → validate → stop.

## Required behaviour

1. Validate identity uniqueness of all static semantic identities (OriginId, FactId, RoleId, CapabilityId, BranchId, GroupId, BatchId, ItemTemplateId) at Program scope including nested Item Template identities
2. Validate origin reference integrity: every referenced OriginId resolves to an existing Origin Site (entry_origin, success_continuations, Branch subjects and targets, Together members, Anchor_value origins, Fact_from_origin origins)
3. Validate entry integrity: actionable programs require a valid entry_origin; zero-Action programs may have None
4. Validate success continuation integrity: no duplicate from_origin, cycle-free success-flow graph
5. Reject success-flow cycles (self-cycle and multi-node)
6. Validate capability contract integrity: every Action_origin matches a pinned program contract with same CapabilityId and contract digest
7. Validate input fact integrity: unique FactIds, Evaluation_input provenance, guard facts declared
8. Validate fact provenance integrity: Origin_provenance references existing Origin; Role_proxy references existing Role
9. Validate fact dependency DAG: static provenance/dependency relationships must be acyclic
10. Validate anchor binding integrity: referenced Origin exists and is an Anchor_origin, path non-empty with no empty components
11. Validate fact-from-origin integrity: Fact provenance compatible with Origin
12. Validate fact-through-role integrity: Role exists and Fact Contract exposes the Fact
13. Validate branch integrity: unique BranchId, outcomes not duplicated per branch, continue targets exist, subject exists
14. Validate role integrity: unique RoleId, scope item template exists, Fact Contract references valid Facts
15. Validate item template integrity: unique ItemTemplateId, nested identity uniqueness, objective Required_role references in-template Role
16. Validate together integrity: at least 2 members, no duplicate members, no self-membership, all members exist
17. Validate batch structural integrity: unique BatchId, referenced ItemTemplateId exists
18. Reject empty Deadline strings
19. Produce deterministic error ordering across repeated identical input

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core.ml` / `.mli` — Core semantic type vocabulary
- `tethers-0.1/engine-ocaml/bin/tethers_core_validator.ml` / `.mli` — new validator module
- `tethers-0.1/engine-ocaml/bin/tethers_core_validator_test.ml` — focused tests
- `tethers-0.1/engine-ocaml/bin/dune` — module graph (add validator test)

## Frozen decisions and invariants

- CORE-3 validates Core. It never repairs Core. No guessing, no deduplication, no reordering.
- Validation errors are collected across categories where practical.
- Error ordering is deterministic using fixed traversal order.
- Multi-error API preferred: `(unit, validation_error list) result`.
- No Core type changes.
- No Rust changes. No runtime wiring. No evaluator/protocol/outcome changes.
- Batch semantics opaque placeholders are validated only structurally, not interpretatively.

## Acceptance criteria

1. Valid lowered CORE-2 Program validates OK
2. Duplicate Origin rejected
3. Missing entry target rejected
4. Duplicate success continuation rejected
5. Success-flow self-cycle rejected
6. Multi-node success cycle rejected
7. Missing Capability contract rejected
8. Contract digest mismatch rejected
9. Duplicate input Fact ID rejected
10. Guard unknown Fact rejected
11. Bad Anchor Origin reference rejected
12. Anchor path empty rejected
13. Fact provenance missing Origin rejected
14. Fact_from_origin provenance mismatch rejected
15. Role missing rejected
16. Role Fact Contract mismatch rejected
17. Branch duplicate Outcome rejected
18. Branch missing target rejected
19. Together one member rejected
20. Together duplicate member rejected
21. Together unknown member rejected
22. Item objective missing Role rejected
23. Batch missing Item Template rejected
24. Determinism: repeated validation returns identical errors in identical order
25. Integration test: parse → lower → validate → OK

## Required verification

1. Packet checker at closeout: `control-v1/COMPLETE`
2. OCaml build: `dune build`
3. Validator tests: `dune runtest` — 42/42 assertions
4. Existing lowerer tests: passing (same `dune runtest`)
5. Fixture suite: `check-fixtures.ps1` — 64 JSON + 32 JSONL
6. Engine suite: `test-engine.ps1` — NOT RUN (pre-existing environment: worktree lacks local `_opam`; script does not accept `--switch` parameter)
7. MCP transcript suite: `test-mcp-transcripts.ps1` — 16 cases
8. Whitespace check: `git diff --check`
9. Rust formatter: `cargo fmt --check` (exit 0)
10. Complete diff inspection: only authorised files
11. Git status: clean worktree

## Forbidden changes

No evaluator/protocol/outcome changes. No Rust changes. No runtime wiring. No Core type changes.

## Stop conditions

Commit CORE-3. STOP. Do NOT begin CORE-4 canonicalisation, ProgramDigest, JSON wire protocol, or Rust ingestion.

## Expected pre-existing changes

None.
