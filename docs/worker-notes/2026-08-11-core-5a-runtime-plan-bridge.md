# Worker Note

Task: `TETHERS CORE-5A — Minimal Core → Runtime Plan Bridge`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `b29b0d348d057cec19faf544f64b64989111fa09`

Implementation checkpoint: `10596e0a2a7222c211d6d1b048ae923dba60c2ec`

## Requested outcome

Create the first executable sidecar bridge from validated Tethers Core to the
existing Runtime Plan representation (`Tethers_outcome.plan`), deriving a
sequential executable plan from semantic control flow. The bridge validates
Core first, fails closed on unsupported constructs with precise typed errors,
and reuses the existing Runtime Plan model without wiring production dispatch.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml` — new bridge implementation
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.mli` — new bridge interface
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml` — new tests T1..T14
- `tethers-0.1/engine-ocaml/bin/dune` — added `tethers_core_plan_test` test stanza
- `docs/CURRENT_CLINE_TASK.md` — task packet updated to CORE-5A

## Decisions and assumptions

1. **Runtime Plan reuse.** The bridge returns `Tethers_outcome.plan` directly
   (`id`, `required_effects`, `actions`, `groups`). No second competing Runtime
   Plan model was created. `required_effects` and `groups` are empty because
   Core's `capability_contract` carries no effects vocabulary here and Together
   execution is unsupported.

2. **Planned action shape.** Each planned Action JSON carries `action_id`
   (sequential `action_<n>`), `capability` (the exact `CapabilityId`), and
   `capability_contract_digest` (the exact `CapabilityContractDigest`). It does
   not carry `idempotency_key`/`capability_version`/`effects` because Core does
   not provide an evaluation ID, capability version, or effect list in this
   bridge context; fabricating them would violate the no-placeholder invariant.

3. **`plan.id` = `program_id`.** The plan's logical identity is the program's
   logical identity. No evaluation ID is fabricated.

4. **Control flow is the only execution order.** The walk starts at
   `entry_origin` and follows `success_continuation` edges to `Program_complete`
   or a missing continuation. `origin_sites` storage order is ignored for
   ordering. A `Together_origin` or `Batch_site` cannot reach the walk because
   the pre-scan rejects them first; the walk match arms for them are defensive
   totality, not reachable paths.

5. **Fail-closed pre-scan precedence.** Deterministic order: Together, then
   Batch (site or `Batch_item_context` input), then Branch, then item templates,
   then `Role_proxy` facts, then input bindings, then execution constraints.
   This matches the required T5..T7 and T9..T13 failure branches.

6. **Representation incompatibilities reported as errors.** `Anchor_value` and
   `Fact_from_origin` bindings return explicit typed errors
   (`Unsupported_anchor_value`, `Unsupported_fact_binding`) because the existing
   Runtime Plan carries concrete resolved argument values and has no
   event-data/fact-carrying vocabulary. `Deadline` returns
   `Unsupported_execution_constraint` because the plan has no deadline field.
   Item templates return `Unsupported_item_template`.

7. **Defensive errors.** `Flow_cycle` and `Unresolved_origin` are unreachable
   for validated Core (the validator rejects success cycles, unknown entry
   origins, and missing continuation targets) but keep the walk total.

## Evidence

All commands ran against implementation checkpoint
`10596e0a2a7222c211d6d1b048ae923dba60c2ec`.

| Command | Result |
| --- | --- |
| `dune build @all` | PASS (exit 0) |
| `dune runtest` | PASS — lowerer 49/49, validator 51/51, plan bridge 31/31 |
| `cargo fmt --check` | PASS (RUST_UNCHANGED) |
| `git diff --check` | PASS |
| `git diff --name-status b29b0d3...HEAD` | only the 5 authorised paths |
| `git status --short --branch` | clean |
| `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` | `control-v1/IN_PROGRESS` before closeout; `control-v1/COMPLETE` after closeout |

**New tests:** 14 test functions, 31 assertions:

- T1 `test_minimal_action` — Anchor → Action A → Program_complete plans one
  Action with correct CapabilityId, digest, inputs, and `plan.id` = program_id
- T2 `test_sequential_two_actions` — A then B in control-flow order
- T3 `test_storage_order_independence` — reversed `origin_sites` produces equal plans
- T4 `test_capability_digest_preservation` — CapabilityId + digest preserved exactly
- T5 `test_unsupported_together` — Together fails closed
- T6 `test_unsupported_batch` — Batch fails closed
- T7 `test_unsupported_role_binding` — Fact_through_role fails closed
- T8 `test_invalid_core` — Invalid Core returns `Invalid_core`, never a plan
- T9 `test_anchor_value_binding` — Anchor_value fails closed
- T10 `test_fact_from_origin_binding` — Fact_from_origin fails closed
- T11 `test_branch_semantics` — Branch fails closed
- T12 `test_execution_constraint` — Deadline fails closed
- T13 `test_item_template` — item templates fail closed
- T14 `test_missing_entry` — no `entry_origin` returns `Missing_entry_origin`

**Commands not run:**

- Fixture suite (`check-fixtures.ps1`, `test-engine.ps1`), MCP transcript suite,
  Rust host tests: NOT RUN — CORE-5A is a dormant sidecar bridge; no evaluator,
  protocol, MCP, or Rust code changed and no production wiring was added.

## Publication evidence

Branch `feature/core-5-runtime-plan-bridge` pushed normally to `origin`.
Remote HEAD resolved and confirmed equal to local HEAD; `git status --short
--branch` clean. See completion report for the full remote HEAD SHA.

## Discoveries

- The existing `Tethers_outcome.plan` `actions` field is `planned_action list`
  (`Yojson.Safe.t`), so the plan layer cannot faithfully carry unresolved
  event-data or fact references. This confirms the packet's frozen decision
  that `Anchor_value` and `Fact_from_origin` must fail closed rather than be
  stringified into arguments.

## Remaining risks

- None known within packet scope. The bridge is dormant and not wired to any
  executable or runtime; its only consumer is the new test executable.

## Smallest next action

Lucy reviews the pushed branch against the acceptance criteria; CORE-5B (any
runtime wiring) is not started by this worker.

## References

- Branch: `feature/core-5-runtime-plan-bridge`
- Base: `b29b0d348d057cec19faf544f64b64989111fa09` (CORE-4 accepted HEAD)
- Implementation checkpoint: `10596e0a2a7222c211d6d1b048ae923dba60c2ec`
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml`
- `tethers-0.1/engine-ocaml/bin/dune`
- `docs/CURRENT_CLINE_TASK.md`
