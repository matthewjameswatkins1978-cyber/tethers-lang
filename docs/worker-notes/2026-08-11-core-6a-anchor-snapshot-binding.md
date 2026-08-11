# Worker Note

Task: `CORE-6A — Anchor Snapshot Binding`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `d1ef28d737ac1c8205473e324bc231a4ce2c99af`

Implementation checkpoint: `9333cd71ed080792a348ff2bef0d677540133943`

## Requested outcome

Add faithful Runtime Plan support for Core `Anchor_value of origin_id * string list` using runtime-supplied Anchor snapshots. The bridge must continue to consume Core meaning + runtime occurrence context + approved Capability projections + runtime Anchor snapshot data, and produce concrete Runtime Plan. Do not reinterpret Human syntax in the planner.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.mli` — added `anchor_snapshot` type, extended `planning_context` with `anchors` field, added five new error variants for anchor snapshot resolution
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml` — implemented anchor snapshot resolution logic: `find_snapshot`, `traverse_path`, `json_value_of_terminal`, `resolve_anchor_value`; modified `binding_error` to accept `Anchor_value`; modified `plan_action` to resolve `Anchor_value` bindings
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml` — added tests T1..T12 for anchor snapshot binding; removed old `test_unsupported_anchor_value` test; updated error string function
- `docs/CURRENT_CLINE_TASK.md` — updated to CORE-6A with implementation checkpoint

## Decisions and assumptions

- Anchor snapshot lookup is identity-based, not first-match, not order-dependent (deterministic)
- Path traversal follows ordered semantic data; fails explicitly on missing components, non-object traversal, or unsupported terminal values
- CORE-6A supports string, integer, boolean terminal values; anything else fails closed with explicit typed error
- Existing `Literal_value` planning behaviour remains unchanged; Actions may contain a mixture of `Literal_value` and `Anchor_value`
- `Fact_from_origin`, `Fact_through_role`, `Batch_item_context` retain existing fail-closed errors
- If planning context contains data for `O_other_anchor` but Core requests `O_anchor`, planning fails; no fallback to "the only available anchor"

## Evidence

- OCaml build: `dune build @all` — PASS (exit 0)
- All tests: `dune runtest --force` — PASS (lowerer 49/49, validator 51/51, plan bridge 67/67)
- Whitespace: `git diff --check` — PASS (no errors)
- Cargo fmt: `cargo fmt --check` — PASS (RUST_UNCHANGED)
- Diff inspection: only authorised files changed (docs/CURRENT_CLINE_TASK.md, tethers_core_plan.ml, tethers_core_plan.mli, tethers_core_plan_test.ml)
- Git status: clean worktree
- Task-packet checker: `control-v1/COMPLETE` (after worker note creation)

## Publication evidence

Branch: `feature/core-6-anchor-snapshot-binding`. Push to origin pending (awaiting closeout documentation commit).

## Discoveries

- The existing `Unsupported_anchor_value` error was removed from `planning_error` type since we now support `Anchor_value` bindings
- The test `test_unsupported_anchor_value` was removed and replaced with new tests T1..T12 that verify anchor snapshot resolution
- The `mk_context` helper function was updated to include the `anchors` field

## Remaining risks

None known within packet scope.

## Smallest next action

Push the finished branch to origin and confirm local HEAD == remote HEAD.

## References

- Task packet: `docs/CURRENT_CLINE_TASK.md`
- Implementation: `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml`, `tethers_core_plan.mli`, `tethers_core_plan_test.ml`
- Base commit: `d1ef28d737ac1c8205473e324bc231a4ce2c99af`
- Implementation commit: `9333cd71ed080792a348ff2bef0d677540133943`