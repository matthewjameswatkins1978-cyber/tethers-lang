Task: `CORE-7A`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `29e5cc72b1e71fdae2cccef4478f075878d0e288`

Implementation checkpoint: `c9cfc20fefae80a02ee9658317af65aaf699bfe2`

## Requested outcome

Add deterministic runtime evaluation of canonical Core entry guards before
Runtime Plan creation: canonical Core + runtime Fact snapshots -> evaluate
entry guards -> MATCHED produces Runtime Plan, NOT MATCHED produces no plan,
ERROR produces typed failure. Do not wire production execution.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml` -- added fact_snapshot
  type, guard evaluation logic (resolve, type-check, compare), evaluate_canonicalized
  returning Matched/Not_matched, Unresolved_entry_guards error, plan_internal
  helper, guard bypass checks in plan() and plan_canonicalized()
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.mli` -- exposed fact_snapshot,
  facts field in planning_context, canonical_evaluation type, evaluate_canonicalized
  signature, 5 new error constructors
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml` -- added G1-T1 through
  G1-T17, E2E human-to-guard-to-plan test, canonical identity adversarial test,
  updated mk_context and string_of_planning_error

## Decisions and assumptions

- Facts are keyed by HostSnapshotKey, not canonical FactId, because FactId is
  rewritten by canonicalisation while HostSnapshotKey is the external semantic key
- evaluate_single_guard uses a local guard_single_result type (Guard_ok |
  Guard_false) to cleanly separate guard-false from planning-error without
  polluting the planning_error type
- plan_internal is an internal helper that skips the entry_guards check, used
  by evaluate_canonicalized after guards have been evaluated
- validate_guard_expected returns bool (not result) because it is a pure
  precondition check on operator/expected/type combination validity
- Contains comparison uses a manual substring search (OCaml stdlib has no
  String.contains_string); empty expected string is treated as always-contains
- Tests extract canonical fact_ids from canonicalized programs rather than
  assuming pre-canonical names, because canonicalisation rewrites fact_ids

## Evidence

- `dune build @all` -- PASS (exit 0)
- `dune runtest --force` -- PASS (136/136 plan bridge tests)
- `git diff --check` -- PASS (CRLF warnings only)
- `cargo fmt --all -- --check` -- PASS (RUST_UNCHANGED)
- `git status --short` -- clean (3 files changed)

### Test coverage

| Test | Requirement | Result |
|------|-------------|--------|
| G1-T1 | Equals string matches | PASS |
| G1-T2 | Equals string false -> Not_matched | PASS |
| G1-T3 | Integer Greater_than | PASS |
| G1-T4 | Integer Greater_than_or_equal | PASS |
| G1-T5 | String Contains | PASS |
| G1-T6 | Boolean Equals | PASS |
| G1-T7 | Multiple guards AND together (3 guards) | PASS |
| G1-T8 | Missing runtime Fact -> Missing_fact_snapshot | PASS |
| G1-T9 | Wrong HostSnapshotKey -> still Missing_fact_snapshot | PASS |
| G1-T10 | Duplicate HostSnapshotKey -> Ambiguous_fact_snapshot | PASS |
| G1-T11 | Reversed duplicate order -> same error | PASS |
| G1-T12 | Runtime type mismatch -> Fact_snapshot_type_mismatch | PASS |
| G1-T13 | Invalid comparison typing -> Invalid_guard_comparison | PASS |
| G1-T14 | Low-level plan guard bypass blocked | PASS |
| G1-T15 | plan_canonicalized guard bypass blocked | PASS |
| G1-T16 | Unguarded existing behaviour preserved | PASS |
| G1-T17 | ProgramDigest invariant across runtime facts | PASS |
| E2E | Human -> parser -> lowerer -> canonicalize -> evaluate_canonicalized | PASS |
| Adv | Canonical identity independence across temporary FactIds | PASS |

## Publication evidence

- Branch pushed: `feature/core-7a-entry-guards`
- Implementation commit SHA: `c9cfc20fefae80a02ee9658317af65aaf699bfe2`
- Final remote HEAD SHA: `c1d7a1470bec174b7b1459260c320d8ab3919563`
- Local HEAD: `c1d7a1470bec174b7b1459260c320d8ab3919563`
- Local HEAD == remote HEAD: confirmed
- `git status --short`: clean

## Discoveries

- `Tethers_core_canonical.canonicalize` rewrites fact_ids (e.g. "F_name" -> "F1").
  Tests that compare planning_error payloads containing fact_ids must use the
  canonical fact_id from the canonicalized program, not the pre-canonical string.
- `host_snapshot_key` is NOT rewritten by canonicalisation (it is preserved in
  Evaluation_input provenance), so tests comparing HostSnapshotKey-valued errors
  can use the pre-canonical key directly.
- OCaml stdlib has no String.contains_string; a manual substring search was
  implemented for the Contains guard operator.

## Remaining risks

None known within packet scope.

## Smallest next action

Await independent review of pushed evidence. If accepted, Lucy may compile the
next task packet (e.g. CORE-7B runtime wiring or a different milestone).

## References

- Task packet: `docs/CURRENT_CLINE_TASK.md`
- Base commit: `29e5cc72b1e71fdae2cccef4478f075878d0e288`
- Implementation commit: `c9cfc20fefae80a02ee9658317af65aaf699bfe2`
- Branch: `feature/core-7a-entry-guards`
- Files changed: `tethers_core_plan.ml`, `tethers_core_plan.mli`,
  `tethers_core_plan_test.ml`
