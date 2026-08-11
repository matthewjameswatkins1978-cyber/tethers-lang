# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-2A — Ambiguous Environment Fail-Closed Correction`

Owner: `OpenCode`

Status: `COMPLETE`

Task colour: `Green`

Route: `OpenCode implementation + evidence → Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-11-core-2a-ambiguous-environment-fail-closed.md`

Base branch: `feature/core-2-human-to-core-lowering`

Base commit: `ca7d013effef4bf1e697141651301561f573435c`

Implementation checkpoint: `47cb5469d758cd0d2c4239a95f3c7ebe02de26bb`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory

Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1` (run; all tools present)

Rust change class: `RUST_UNCHANGED`

## Objective

Correct two ambiguity cases in CORE-2 lowering:

1. duplicate Human Fact bindings must be reported as ambiguity, not
   `Unknown_fact`;
2. conflicting Capability contracts sharing one `CapabilityId` must never be
   silently deduplicated.

No architecture changes. No runtime wiring.

## Relevant background and existing behaviour

CORE-2 established the `Tethers_core_lowerer` module translating the
sequential Tethers 0.1 subset into dormant Core programs. Two fail-closed gaps
remain in the lowering environment handling: `resolve_fact` mapped 2+ matches
to `Unknown_fact` (absence and ambiguity were conflated), and the
`capability_contracts` dedup silently retained the first digest seen for a
given `capability_id`, allowing a Program whose Action Origins and pinned
contract table disagree about semantic identity.

## Required behaviour

1. Add an explicit `Duplicate_fact` lowering error distinguishing duplicate
   source-name Fact bindings from an unknown Fact.
2. Change Fact resolution so 2+ matching input Fact bindings produce
   `Duplicate_fact` instead of `Unknown_fact`.
3. Detect when two used capability bindings share one `CapabilityId` but have
   different contract digests and produce an explicit conflict error.
4. Permit two source names resolving to the same `CapabilityId` with the same
   digest to collapse into one `capability_contract` entry.
5. Validate only the used semantic subset so unused conflicting environment
   entries do not poison lowering.
6. Leave Action Origin contract references unchanged.

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer.ml` / `.mli` — lowerer
  module; `resolve_fact` and `capability_contracts` construction.
- `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer_test.ml` — focused tests.
- `tethers-0.1/engine-ocaml/bin/dune` — module graph (unchanged).

## Frozen decisions and invariants

- Absence and ambiguity are distinct errors: `Unknown_fact` vs `Duplicate_fact`.
- A used `CapabilityId` must have exactly one pinned digest across all used
  source names.
- Equivalent repeated `(CapabilityId, CapabilityContractDigest)` pairs may
  collapse into one Program contract entry.
- Conflicting digests for one used `CapabilityId` fail before `Ok program`.
- Only capabilities actually referenced by the Tether are validated; unused
  environment entries do not poison lowering (documented and deterministic).
- Deterministic Origin IDs, literal lowering, Anchor path lowering, named
  inputs, entry guards, success continuations, and Together refusal are
  unchanged.
- No Core type changes. No Rust changes. No runtime wiring.

## Acceptance criteria

1. Two environment Fact bindings with `source_name = "file_type"` and a Human
   Condition `file_type is "pdf"` produce `Duplicate_fact "file_type"`.
2. Two source names (`saveA`, `saveB`) mapping to `C_save`/`D1`, both used,
   lower successfully with exactly one `C_save`/`D1` Program contract entry.
3. Two source names mapping to `C_save`/`D1` and `C_save`/`D2`, both used,
   produce an explicit conflicting-contract error.
4. Unused conflicting environment bindings do not poison lowering when only
   one source name is referenced.
5. All existing CORE-2 tests and legacy suites continue to pass.
6. Existing tests confirm each Action Origin retains the contract reference
   resolved from its own source-name binding after CORE-2A changes.

## Required verification

1. Packet checker at closeout: `control-v1/COMPLETE`
2. OCaml build: `dune build`
3. Lowerer tests: `dune runtest` — 49/49 assertions
4. Fixture suite: `check-fixtures.ps1` — 64 JSON + 32 JSONL
5. Engine suite: `test-engine.ps1` — 32 cases
6. MCP transcript suite: `test-mcp-transcripts.ps1` — 16 cases
7. Whitespace check: `git diff --check`
8. Rust formatter: `cargo fmt --check` (exit 0)
9. Complete diff inspection: only authorised files
10. Git status: clean worktree

## Forbidden changes

Do NOT modify: `tethers_evaluator.ml/.mli`, `tethers_protocol.ml/.mli`,
`tethers_outcome.ml/.mli`. Do not modify Rust. Do not change existing
runtime output. Do not modify `tethers_core.ml/.mli`. Do not change
deterministic Origin IDs, literal lowering, Anchor path lowering, named
inputs, entry guards, success continuations, or Together refusal.

## Stop conditions

Committed CORE-2A. STOP. Do NOT begin CORE-3, wire into the evaluator, or
serialize Core.

## Expected pre-existing changes

None.
