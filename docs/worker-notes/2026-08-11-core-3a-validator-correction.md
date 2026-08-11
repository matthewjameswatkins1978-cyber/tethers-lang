# Worker Note — TETHERS CORE-3A

Task: `TETHERS CORE-3A — Validator Dependency & Scope Correction`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `45ec528ac214fbdb5528c794541159bd006b8311`

Implementation checkpoint: `c4b42b2e164fd54986c592287904a0d31b4ef1f2`

## Requested outcome

Correct four bounded defects in the CORE-3 static Core validator: (1) Fact dependency DAG derived edges from Origin_provenance alone instead of Action_origin input/output bindings; (2) Origin ID uniqueness checked only within program.origin_sites, missing cross-collection collisions with item template origin_sites; (3) Fact_from_origin and Fact_through_role bindings silently ignored nonexistent Facts; (4) Role resolution used flat global lookup ignoring scope boundaries between program and item template scopes.

## Changes made

Modified:
- `tethers-0.1/engine-ocaml/bin/tethers_core_validator.ml` — rewrote Fact dependency DAG (section 8) to derive edges only from Action_origin input/output bindings; changed duplicate origin check to global (across program + all item templates); added Missing_fact checks before provenance/contract matching in Fact_from_origin and Fact_through_role; implemented scope-aware Role resolution with separate program-scope and per-template role lookups; narrowed has_actions check to program_sites only.
- `tethers-0.1/engine-ocaml/bin/tethers_core_validator_test.ml` — added 9 new tests (3A-1 through 3A-9); updated test_role_fact_contract_mismatch to declare F_y as an existing Fact so contract mismatch (not Missing_fact) fires.

## Decisions and assumptions

1. Fact dependency DAG: dependencies are only derived from Action_origin bindings. For each Action_origin OA with output Facts {F_out} and input Fact bindings (Fact_from_origin, Fact_through_role) {F_in}, an edge F_out → F_in is created. Literal_value and Anchor_value create no edges. Origin_provenance alone creates no edges.

2. Global uniqueness: duplicate OriginId across program.origin_sites and any item_template.origin_sites produces Duplicate_origin_id. Item_template_duplicate_origin_id is preserved as additional within-template diagnostic.

3. Fact existence: Fact_from_origin and Fact_through_role now independently check `is_known_fact fid` before running provenance/contract checks. Missing Fact is reported as Missing_fact. If the Fact exists, the original provenance/contract checks run.

4. Role scope: program Origins resolve only Roles with Program_scope or within program.roles. Item-template Origins resolve only Roles from their own Item Template. Out-of-scope Role references produce Missing_role.

5. has_actions: now checks only program_sites, not all_sites (item template actions are templates, not directly executable at program level).

## Evidence

### Build
- `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune build` — PASS (clean, no warnings)

### Tests
- `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune runtest` — PASS: 51/51 assertions (42 original + 9 new CORE-3A)

### Fixture suite
- `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\check-fixtures.ps1` — PASS: 64 JSON + 32 JSONL

### MCP transcript suite
- `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-mcp-transcripts.ps1` — PASS: 16 cases

### Engine suite
- `test-engine.ps1` — NOT RUN (pre-existing environment: worktree lacks local `_opam`; script does not accept `--switch` parameter)

### Whitespace check
- `git diff --check` — PASS

### Rust formatter
- `cargo fmt --check --manifest-path tethers-0.1\host-rust\Cargo.toml` — PASS (exit 0)

### Diff inspection
- Only authorised files: `tethers_core_validator.ml` (+139/-48), `tethers_core_validator_test.ml` (+289/-5)
- No mli changes, no Core type changes, no Rust changes, no evaluator/protocol/outcome changes

### Git status
- Clean worktree after commit. Branch: `feature/core-3-static-validator`

## Discoveries

None beyond the four defects already documented in the task packet.

## Remaining risks

None known within packet scope.

## Smallest next action

Lucy reviews the pushed GitHub evidence and records one verdict (ACCEPTED or REJECTED) based on the implementation checkpoint `c4b42b2e164fd54986c592287904a0d31b4ef1f2`.

## References

- Branch: `feature/core-3-static-validator`
- Implementation checkpoint: `c4b42b2e164fd54986c592287904a0d31b4ef1f2`
- Base: `feature/core-3-static-validator` at `45ec528ac214fbdb5528c794541159bd006b8311`
- Modified files: `tethers_core_validator.ml`, `tethers_core_validator_test.ml`
