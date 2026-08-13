# Worker Note: CORE-8B2 — Truly Total JSON Shape Validation

Task: `TETHERS CORE-8B2 — Truly Total JSON Shape Validation`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `10605078138ab7eab3e3cda8a5d4d14eec53d243`

Implementation checkpoint: `203393ae0715d53122ce98da47e3d4d31079919f`

## Requested outcome

Finish the total-request invariant from CORE-8B1. All structural JSON
parsing must be object-safe: prove `Assoc` before extracting fields.
No Yojson.Safe.Util.Type_error or other structural exception may escape
evaluate_request.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core_request_adapter.ml` -- modified
  - Replaced `json_string`, `json_list`, `json_member`, `core_env_string`
    with object-safe helpers: `expect_object`, `expect_object_core`,
    `field_string`, `field_string_core`, `field_list`, `field_list_core`,
    `field_maybe_null`
  - `parse_request`: validates root request is object, tether is object,
    event is object, capability items are objects before parse_capability
  - `parse_core_env`: uses object-safe helpers; validates capability
    binding items and fact declaration items are objects
  - `resolve_one_capability`: proves binding is object before field extraction
  - `parse_one_fact`: proves declaration is object before field extraction
  - `schema_description` is now required (not optional "")
  - Removed unused `json_string`, `json_list`, `json_member`, `core_env_string`
- `tethers-0.1/engine-ocaml/bin/tethers_core_request_adapter_test.ml` -- modified
  - Added `assert_no_exception` and `assert_planning_error` test helpers
  - Q1-Q12: 12 new regression tests for null/string/non-object root,
    non-object tether/event, non-object cap items, missing/typed
    schema_description, wrong-type program_id/core_version
  - Updated all `mk_core_fact` calls to supply schema_description

## Decisions and assumptions

- Object-safe helpers use `List.assoc_opt` on association lists directly,
  never `Yojson.Safe.Util.member` on unvalidated values.
- `field_maybe_null` returns `Ok None` for missing or explicit `Null`,
  used for the optional `input_facts` field in core_environment.
- Capability items in the top-level list are validated as objects before
  reaching `Tethers_protocol.parse_capability`, which would throw
  `Tethers_error` on non-objects.
- Q8 and Q9 test missing and non-string `schema_description` — the field
  is now required per the packet.

## Evidence

- OCaml build: `dune build @all` -- PASS (exit 0)
- All tests: `dune runtest --force` -- PASS
  - lowerer: 49/49
  - validator: 51/51
  - plan bridge: 179/179
  - adapter: 43/43
  - request adapter: 89/89
- Whitespace: `git diff --check` -- PASS (only LF/CRLF warnings on Windows)
- Diff inspection: only authorised files changed (2 files)
- Git status: clean worktree
- Implementation checkpoint: `203393ae0715d53122ce98da47e3d4d31079919f`

## Publication evidence

- Branch pushed: `feature/core-8b-request-boundary`
- Remote HEAD SHA: pending push
- Local HEAD == remote HEAD: to confirm after push
- Git status: clean

## Discoveries

- `Tethers_protocol.parse_capability` raises `Tethers_error` on non-object
  input. The new code validates each capability item is `Assoc` before
  calling `parse_capability`, converting the would-be exception into a
  typed `Invalid_request` error.
- The `Yojson.Safe.Util.member` function throws `Type_error` when called
  on non-object values. The old `json_string`/`json_list` helpers relied
  on this exception for type checking. The new helpers use `List.assoc_opt`
  on the association list after proving `Assoc`.

## Remaining risks

None known within packet scope.

## Smallest next action

Lucy independent GitHub review of the pushed branch and worker note.

## References

- Branch: `feature/core-8b-request-boundary`
- Base: `10605078138ab7eab3e3cda8a5d4d14eec53d243`
- Implementation checkpoint: `203393ae0715d53122ce98da47e3d4d31079919f`
- Packet: `docs/CURRENT_CLINE_TASK.md`
- Previous: CORE-8B1 worker note `docs/worker-notes/2026-08-12-core-8b1-total-parsing-fact-fidelity.md`
