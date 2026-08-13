# Worker Note: CORE-8B — Explicit Core Evaluation Request Boundary

Task: `TETHERS CORE-8B — Explicit Core Evaluation Request Boundary`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `3924ea15ae67b23c2b34caf591a225067733d82e`

Implementation checkpoint: `829f8f1846cc376e92c7d3750ec2d3870faf4a71`

## Requested outcome

Create one dormant OCaml request-boundary module that consumes an extended
Tethers 0.1 request JSON (with `core_environment`) and calls the accepted
`Tethers_core_evaluation_adapter.evaluate`. This establishes the exact wire
contract required for later production wiring.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core_request_adapter.mli` -- new
  Public interface: `request_context`, `evaluated_request`, `request_error`
  types and `evaluate_request` API.
- `tethers-0.1/engine-ocaml/bin/tethers_core_request_adapter.ml` -- new
  Implementation: JSON request parsing, runtime capability join
  (0→Missing, 1→use, 2+→Ambiguous), Core environment assembly, scalar
  type validation, and delegation to CORE-8A adapter.
- `tethers-0.1/engine-ocaml/bin/tethers_core_request_adapter_test.ml` -- new
  17 tests (T1-T16 + E2E) covering complete request, invoice flow,
  wrong event, identity separation, missing/ambiguous runtime caps,
  digest distinction, HostSnapshotKey, FactId, scalar types, missing
  core_environment, correlation preservation, ProgramDigest invariance,
  evaluation_id identity, runtime cap fidelity, duplicate caps, existing
  tests placeholder, and one-call E2E proof.
- `tethers-0.1/engine-ocaml/bin/dune` -- modified (added request adapter
  test stanza).
- `docs/CURRENT_CLINE_TASK.md` -- updated to CORE-8B packet.

## Decisions and assumptions

- The `parsed_request` record is internal to the module, not exposed in
  the `.mli`. It carries protocol version, language version, evaluation_id,
  event_id, tether_id/version, source, event_name, event_data, facts_json,
  and top_level_caps.
- Runtime capability resolution uses `List.filter` + pattern matching on
  list length (0/1/2+) per the frozen CORE-8B specification.
- T6 test includes all three bridge fields (`manifest_digest`,
  `bridge_capability_version`, `bridge_provider_identity`) because
  `Tethers_protocol.parse_capability` requires them together.
- T12 test uses separate `evaluation_id` parameters for each request
  to correctly verify plan.id divergence.
- `json_assoc "facts"` gracefully handles missing facts field by
  falling back to `Null` (empty facts object), which is valid for
  unguarded tethers.

## Evidence

- OCaml build: `dune build @all` -- PASS (exit 0)
- All tests: `dune runtest --force` -- PASS
  - lowerer: 49/49
  - validator: 51/51
  - plan bridge: 179/179
  - adapter: 43/43
  - request adapter: 49/49
- Whitespace: `git diff --check` -- PASS (only LF/CRLF warnings on Windows)
- Diff inspection: only authorised files changed
- Git status: clean worktree
- Implementation checkpoint: `829f8f1846cc376e92c7d3750ec2d3870faf4a71`

## Publication evidence

- Branch pushed: `feature/core-8b-request-boundary`
- Remote HEAD SHA: `829f8f1846cc376e92c7d3750ec2d3870faf4a71`
- Local HEAD == remote HEAD: confirmed
- Git status: clean

## Discoveries

- `Tethers_protocol.parse_capability` requires manifest_digest,
  bridge_capability_version, and bridge_provider_identity to appear
  together. This is an existing protocol constraint that T6 must respect.
- The `parse_request` function catches `Tethers_error` exceptions from
  `parse_capability` and converts them to typed `Invalid_request` errors.

## Remaining risks

None known within packet scope.

## Smallest next action

Lucy independent GitHub review of the pushed branch and worker note.

## References

- Branch: `feature/core-8b-request-boundary`
- Base: `3924ea15ae67b23c2b34caf591a225067733d82e`
- Implementation checkpoint: `829f8f1846cc376e92c7d3750ec2d3870faf4a71`
- Packet: `docs/CURRENT_CLINE_TASK.md`
- Adapter: `tethers-0.1/engine-ocaml/bin/tethers_core_evaluation_adapter.ml`
- Protocol: `tethers-0.1/engine-ocaml/bin/tethers_protocol.ml`
- Plan: `tethers-0.1/engine-ocaml/bin/tethers_core_plan.mli`
