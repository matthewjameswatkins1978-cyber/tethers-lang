# Worker Note: CORE-8B1 — Total Request Parsing + Fact Fidelity Proof

Task: `TETHERS CORE-8B1 — Total Request Parsing + Fact Fidelity Proof`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `dcdc6dbe5e568b863f86dde29d05b6bf80a9b3a5`

Implementation checkpoint: `c333812c1092f196654cf6c0556e156ae2adb3cc`

## Requested outcome

Close four narrow review findings in CORE-8B: total request parsing,
fact fidelity, T3 reception-before-guard proof, and strengthened
T7/T13 assertions. Add R1-R8 regression tests.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core_request_adapter.ml` -- modified
  - Added `core_env_string` Result-returning helper for core_environment fields
  - Rewrote `resolve_one_capability` to use `core_env_string` instead of `raise Exit`
  - Rewrote `parse_one_fact` to use `core_env_string` instead of `raise Exit`
  - Added `core_environment` object validation before parsing
  - Required `facts` field to be an object (missing/null → Invalid_request)
  - Removed `filter_map`; pass occurrence facts pairs through unchanged
  - Removed unused `json_assoc` helper
- `tethers-0.1/engine-ocaml/bin/tethers_core_request_adapter_test.ml` -- modified
  - T3: strengthened with guarded tether, wrong event, no facts
  - T7: destructured nested error to assert exact HOST_KEY_771 key
  - T13: added exact idempotency key assertions (eid/action_1)
  - R1-R8: eight new regression tests

## Decisions and assumptions

- `core_env_string` returns `Invalid_core_environment` (not `Invalid_request`)
  for core_environment field type errors — distinguishes layer ownership.
- R3 ("core_environment": "oops") returns `Invalid_core_environment` (present
  but wrong type), not `Missing_core_environment`.
- `facts` field missing from JSON returns `Invalid_request` per packet.
- `facts: null` returns `Invalid_request` per packet.
- Non-scalar fact values (e.g. arrays) pass through to CORE-8A, which
  produces `Fact_snapshot_type_mismatch` (not `Missing_fact_snapshot`).

## Evidence

- OCaml build: `dune build @all` -- PASS (exit 0)
- All tests: `dune runtest --force` -- PASS
  - lowerer: 49/49
  - validator: 51/51
  - plan bridge: 179/179
  - adapter: 43/43
  - request adapter: 67/67
- Whitespace: `git diff --check` -- PASS (only LF/CRLF warnings on Windows)
- Diff inspection: only authorised files changed (2 files)
- Git status: clean worktree
- Implementation checkpoint: `c333812c1092f196654cf6c0556e156ae2adb3cc`

## Publication evidence

- Branch pushed: `feature/core-8b-request-boundary`
- Remote HEAD SHA: `c333812c1092f196654cf6c0556e156ae2adb3cc`
- Local HEAD == remote HEAD: confirmed
- Git status: clean

## Discoveries

- `Yojson.Safe.Util.member` throws `Type_error` when called on non-object
  values. Must validate core_environment is `Assoc` before passing to
  `parse_core_env`.
- The `facts` field was silently treating missing/null as empty via
  `json_assoc` fallback. The packet requires strict object validation.

## Remaining risks

None known within packet scope.

## Smallest next action

Lucy independent GitHub review of the pushed branch and worker note.

## References

- Branch: `feature/core-8b-request-boundary`
- Base: `dcdc6dbe5e568b863f86dde29d05b6bf80a9b3a5`
- Implementation checkpoint: `c333812c1092f196654cf6c0556e156ae2adb3cc`
- Packet: `docs/CURRENT_CLINE_TASK.md`
- Previous: CORE-8B worker note `docs/worker-notes/2026-08-12-core-8b-request-boundary.md`
