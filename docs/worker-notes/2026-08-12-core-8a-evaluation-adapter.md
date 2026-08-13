# Worker Note: CORE-8A

Task: `TETHERS CORE-8A - Human Request to Canonical Evaluation Adapter`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `97eb5e637b9c4cfcba729d8ced71360784922e7b`

Implementation checkpoint: `6bdd91babe4eaed5a84c3ecc650de1292edfe20c`

## Requested outcome

Create one pure OCaml adapter that turns a Human-world evaluation input into
the already-accepted canonical Core evaluation path: parse -> lower ->
canonicalize -> evaluate_canonicalized. The adapter receives explicit semantic
lowering identities from its caller and does not switch the production
evaluator.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core_evaluation_adapter.mli` -- new
  interface with typed environment, evaluation_input, and adapter_error types
- `tethers-0.1/engine-ocaml/bin/tethers_core_evaluation_adapter.ml` -- new
  implementation with one-call evaluate pipeline
- `tethers-0.1/engine-ocaml/bin/tethers_core_evaluation_adapter_test.ml` -- new
  test file with T1-T15 + E2E (16 tests total, 40 assertions)
- `tethers-0.1/engine-ocaml/bin/dune` -- added adapter test stanza
- `docs/CURRENT_CLINE_TASK.md` -- updated to CORE-8A

## Decisions and assumptions

- Adapter types mirror `Tethers_core_lowerer` naming where the domain matches
  (capability_binding, input_fact_binding, lowering_environment) but add the
  runtime capability projection in a single binding to guarantee identity
  sharing.
- Parser errors (`Tethers_error`) are caught at the adapter boundary and
  wrapped in a typed `Parse_error` constructor.
- `program_id` is NOT included in canonical bytes (confirmed by inspecting
  `encode_program` in `tethers_core_canonical.ml:1354`), so different
  `program_id` values with the same Tether source produce the same ProgramDigest.
- Plan bridge uses `projection.runtime.name` for the `capability` field in
  plan actions, not the Core `capability_id`. This is existing behaviour
  confirmed by the plan bridge tests.
- Tether parser requires all four sections (tether, anchor, when, do) even when
  conditions list is empty. Empty `when` section used in unguarded test sources.

## Evidence

### Build and test (run against committed checkpoint)

```
dune build @all                       -- PASS (exit 0)
dune runtest --force                  -- PASS
  lowerer tests:       49/49
  validator tests:     51/51
  plan bridge tests:  179/179  (T15: existing Core tests remain green)
  adapter tests:       40/40
git diff --check                      -- PASS (LF/CRLF warnings only)
```

### Rust

```
cargo fmt --all -- --check            -- N/A (RUST_UNCHANGED, no Cargo.toml)
```

### Git

```
git status: clean worktree
```

### Test coverage

| Test | Description | Result |
|------|-------------|--------|
| T1 | Minimal unguarded Human Tether -> Matched | PASS |
| T2 | Full guarded Anchor-value Human flow | PASS |
| T3 | Wrong event -> Not_matched | PASS |
| T4 | Guard false -> Not_matched | PASS |
| T5 | Missing required Fact -> Planning_error | PASS |
| T6 | Unknown runtime Fact name -> Unknown_runtime_fact_name | PASS |
| T7 | Duplicate runtime Fact name -> Duplicate_runtime_fact_name | PASS |
| T8 | Ambiguous env Fact source name -> Ambiguous_runtime_fact_name | PASS |
| T9 | Capability source-name resolution | PASS |
| T10 | Source name differs from Core CapabilityId | PASS |
| T11 | Wrong projection identity cannot substitute | PASS |
| T12 | ProgramDigest invariant across occurrence data | PASS |
| T13 | ProgramId changes do not alter occurrence identity | PASS |
| T14 | evaluation_id changes occurrence only | PASS |
| T15 | Existing low-level Core tests remain green (179/179) | PASS |
| E2E | One-call adapter proof (no manual pipeline calls) | PASS |

## Publication evidence

Branch `feature/core-8a-evaluation-adapter` pushed to origin. Full remote HEAD
SHA to be confirmed after push.

## Discoveries

- The Tether parser REQUIRES the `when` keyword in the syntax even when there
  are no conditions. The pattern match in `tether_parser.ml:234` explicitly
  looks for `"when"` in the token list. An empty `when` section produces zero
  conditions.
- The plan bridge's `capability` field in plan actions uses the runtime
  capability name (`projection.runtime.name`), NOT the Core capability_id.
  This is correct existing behaviour: the runtime name is the Human authoring
  name, and the Core identity is validated internally during planning.
- OCaml record field disambiguation requires explicit type annotations when
  multiple record types share field names (e.g. `source_name`) and are used
  in the same module. The adapter functions `lowerer_capabilities`,
  `lowerer_facts`, and `plan_projections` use explicit type annotations on
  parameters and return types.

## Remaining risks

None known within packet scope.

## Smallest next action

Wait for Lucy's independent GitHub review. The production evaluator wiring
(CORE-8B) is explicitly out of scope for this packet.

## References

- Task packet: `docs/CURRENT_CLINE_TASK.md`
- Base branch: `feature/core-7b-anchor-reception`
- Base commit: `97eb5e637b9c4cfcba729d8ced71360784922e7b`
- Implementation checkpoint: `6bdd91babe4eaed5a84c3ecc650de1292edfe20c`
- OCaml switch: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`
