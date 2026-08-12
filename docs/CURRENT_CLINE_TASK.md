# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-8B — Explicit Core Evaluation Request Boundary`

Owner: `OpenCode`

Implementation checkpoint: `829f8f1846cc376e92c7d3750ec2d3870faf4a71`

Status: `COMPLETE`

Task colour: `Amber`

Route: `OpenCode implementation + evidence, Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-12-core-8b-request-boundary.md`

Base branch: `feature/core-8a-evaluation-adapter`

Base commit: `3924ea15ae67b23c2b34caf591a225067733d82e`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust change class: `RUST_UNCHANGED`

## Objective

Create one dormant OCaml request-boundary module that consumes an extended
Tethers 0.1 request JSON and calls the accepted:

Tethers_core_evaluation_adapter.evaluate

This establishes the exact wire contract required for later production wiring.

Do NOT switch the existing production `tethers.evaluate` MCP tool to this path yet.

Do NOT change Rust yet.

Do NOT derive Core semantic identity from runtime capability names,
manifest digests, Human Fact names, list order, or convenience.

## Relevant background and existing behaviour

CORE-8A now provides:

Human source → parse → lower → canonicalize → canonical reception →
guards → Runtime Plan

through one typed adapter call.

But production currently supplies only the historical request envelope.

CORE-8B defines that missing request boundary.

## Required behaviour

1. New module: `tethers_core_request_adapter.ml` / `.mli`
2. Extended request shape with core_environment
3. Runtime capability join: resolve runtime_name against top-level caps
4. Identity separation: source_name, capability_id, contract_digest, runtime_name
5. Manifest digest ≠ Core contract digest (transport only)
6. Input fact declarations with explicit HostSnapshotKey and FactId
7. Typed request_error preserving layer ownership
8. request_context preserving correlation info
9. One-call evaluate_request API
10. Legacy evaluator remains unchanged

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core_request_adapter.ml` -- new
- `tethers-0.1/engine-ocaml/bin/tethers_core_request_adapter.mli` -- new
- `tethers-0.1/engine-ocaml/bin/tethers_core_request_adapter_test.ml` -- new
- `tethers-0.1/engine-ocaml/bin/dune` -- modified (test stanza)

## Frozen decisions and invariants

- core_environment is a dormant extension used only by this new module
- Runtime capability join: 0 → Missing, 1 → use, 2+ → Ambiguous
- Manifest digest ≠ Core contract digest (transport only)
- HostSnapshotKey is NOT derived from source_name
- FactId is NOT derived from source_name
- ProgramId is NOT derived from tether_id
- ProgramDigest emerges only from canonicalisation
- protocol_version = "0.1", language_version = "0.1" required
- Legacy evaluator unchanged

## Acceptance criteria

1. T1: Complete extended request → Matched
2. T2: Full invoice flow with anchor.value resolution
3. T3: Wrong event → Not_matched
4. T4: Human name != Core ID != runtime name
5. T5: Missing runtime capability → Missing_runtime_capability_binding
6. T6: Contract digest and manifest digest are distinct
7. T7: Explicit HostSnapshotKey → Missing_fact_snapshot
8. T8: Explicit FactId
9. T9: Invalid scalar type → Invalid_scalar_type
10. T10: Missing core_environment → Missing_core_environment
11. T11: Correlation preservation
12. T12: ProgramDigest occurrence invariance
13. T13: evaluation_id occurrence identity
14. T14: Runtime capability field fidelity
15. T15: Duplicate top-level runtime capability names
16. T16: All existing Core tests remain green
17. E2E: One-call extended request proof

## Required verification

1. OCaml build: `dune build @all` -- PASS (exit 0)
2. All tests: `dune runtest --force` -- PASS
3. Whitespace: `git diff --check` -- PASS
4. Diff inspection: only authorised files changed
5. Git status: clean worktree
6. Task-packet checker at closeout: `control-v1/COMPLETE`
7. Push branch to origin and confirm local HEAD == remote HEAD

## Forbidden changes

No production evaluator changes, no main.ml changes, no MCP protocol changes,
no Rust changes, no Human syntax changes, no parser semantic changes, no
lowerer semantic changes, no Core vocabulary changes, no validator semantic
changes, no canonicalisation semantic changes, no Runtime Plan representation
changes, no new external dependencies, no MCP tool schema changes, no
runtime_config changes.

## Stop conditions

Commit CORE-8B implementation checkpoint. STOP. Do NOT begin CORE-8B
production wiring.

## Expected pre-existing changes

CORE-8A evaluation adapter (accepted).
