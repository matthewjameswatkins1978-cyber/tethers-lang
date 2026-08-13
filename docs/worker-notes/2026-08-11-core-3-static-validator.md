# Worker Note — TETHERS CORE-3

Task: `TETHERS CORE-3 — Static Core Validator`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `68c3510188d0a6db464fbb2e1814f0ce87b4bc3b`

Implementation checkpoint: `b9763ad440d4500577535430a0e7f6b3b3d00910`

## Requested outcome

Implement a standalone static validator for `Tethers_core.program` that deterministically validates Core Programs for internal well-formedness without repairing, inferring, reordering, canonicalising, or executing. The validator must be a separate side-path module, not wired into the existing evaluator pipeline.

## Changes made

Created:
- `tethers-0.1/engine-ocaml/bin/tethers_core_validator.ml` — full validator implementation with 39 error variants and deterministic multi-error collection
- `tethers-0.1/engine-ocaml/bin/tethers_core_validator.mli` — public interface exposing `validation_error` and `validate`
- `tethers-0.1/engine-ocaml/bin/tethers_core_validator_test.ml` — 42 focused assertions including integration test

Modified:
- `tethers-0.1/engine-ocaml/bin/dune` — added test stanza for validator test with modules `tethers_core_validator_test`, `tethers_core_validator`, `tethers_core`, `tethers_core_lowerer`, `tether_parser`, `tethers_error`. Existing lowerer test stanza re-indented from 1-space to 2-space (cosmetic, matches rest of file).

## Decisions and assumptions

1. Used `ref list` for error accumulation with deterministic append-in-order pattern, then `List.rev` at return. This keeps per-category validators simple while maintaining the specified error ordering.

2. Exhaustive 39 error variants covering identity, reference, entry, success continuation, cycle, capability, fact provenance, dependency DAG, anchor binding, fact-from-origin, fact-through-role, branch, role, item template, together, batch, and deadline categories.

3. Multi-error collection: the validator attempts to collect all independent errors across categories. It does not short-circuit on first error.

4. Cycle detection uses DFS with recusion-stack-based detection, producing deterministic ordered cycle paths.

5. Origin ID extraction: `origin_id_of_site` returns `option` since `Batch_site` carries a `batch_id` (distinct nominal type from `origin_id`). Batch IDs are validated separately.

6. Fact dependency DAG: modeled using provenance relationships — Origin_provenance depends on Facts declared by that Origin, Role_proxy depends on Facts in the Role's contract. Evaluation_input Facts have no dependencies.

7. Structurally: the validator is a function, not a module with mutable state. All lookups use `List` operations (consistent, deterministic for bounded programs). No hash tables.

## Evidence

### Build
- `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune build` — PASS (clean)

### Validator tests
- `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune runtest` — PASS: 42/42 assertions (validator tests) + all existing lowerer tests

### Fixture suite
- `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\check-fixtures.ps1` — PASS: 64 JSON files, 32 JSONL files

### MCP transcript suite
- `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-mcp-transcripts.ps1` — PASS: 16 cases

### Engine suite
- `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-engine.ps1` — NOT RUN. Pre-existing environment: this worktree has no local `_opam` directory; the script uses `opam exec` without `--switch` and does not accept a switch parameter. The existing evaluator protocol path is unchanged by CORE-3.

### Whitespace check
- `git diff --check` — PASS (working tree)

### Rust formatter
- `cargo fmt --check --manifest-path tethers-0.1\host-rust\Cargo.toml` — PASS (exit 0)

### Diff inspection
- Complete diff shows only authorised files: `dune` (modified), `tethers_core_validator.ml/.mli` (created), `tethers_core_validator_test.ml` (created)
- No Core type changes, no evaluator/protocol/outcome changes, no Rust changes

### Git status
- Clean worktree after commit. Branch: `feature/core-3-static-validator`

## Discoveries

1. The `test-engine.ps1` script does not accept an explicit `--switch` parameter and must run from a worktree with a local `_opam` directory. This pre-existing limitation is unrelated to CORE-3.

2. The dune file had mixed indentation (1-space vs 2-space). The edit normalised the lowerer test stanza to 2-space to match the rest of the file. This is cosmetic only.

## Remaining risks

None known within packet scope. The validator is a standalone module, not wired into the evaluator pipeline, so it cannot affect existing behavior even if it contains latent bugs.

## Smallest next action

Lucy reviews the pushed GitHub evidence and records one verdict (ACCEPTED or REJECTED) based on the implementation checkpoint `b9763ad440d4500577535430a0e7f6b3b3d00910`.

## References

- Branch: `feature/core-3-static-validator`
- Implementation checkpoint: `b9763ad440d4500577535430a0e7f6b3b3d00910`
- Base: `feature/core-2-human-to-core-lowering` at `68c3510188d0a6db464fbb2e1814f0ce87b4bc3b`
- Created files: `tethers_core_validator.ml`, `tethers_core_validator.mli`, `tethers_core_validator_test.ml`
- Modified file: `dune`
