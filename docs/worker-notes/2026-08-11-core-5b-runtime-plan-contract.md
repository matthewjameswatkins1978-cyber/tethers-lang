# Worker Note

Task: `TETHERS CORE-5B — Runtime Plan Contract and Terminal-Path Correction`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `a28bdf483db7959f5471b4b950802e863093d9f8`

Implementation checkpoint: `b0e194b2da9331ef2455674a30bd427a5d1873d8`

## Requested outcome

Correct the two independently reviewed CORE-5A Runtime Plan boundary defects
(success paths must terminate explicitly; runtime occurrence identity must not
be ProgramId), then bring the bridge into the existing Runtime Plan Action
contract using an explicit runtime planning context and approved, digest-pinned
capability projections. Do not redesign Core.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml` — modified: added
  `Incomplete_success_path`, `Missing_capability_projection`,
  `Capability_projection_identity_mismatch`, `Capability_projection_digest_mismatch`,
  `Capability_projection_incomplete` errors; added `runtime_capability_projection`
  and `planning_context`; `plan` now takes the context; walk requires explicit
  `Program_complete`; `plan.id = evaluation_id ^ "/plan"`; planned Actions now
  carry the existing Runtime Plan Action contract (`action_id`,
  `idempotency_key`, `capability`, `capability_version`, `arguments`, `effects`,
  plus bridge fields from the projection); `required_effects` aggregated with
  deterministic first-occurrence uniqueness
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.mli` — modified: new types and
  errors, new `plan` signature, updated docs
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml` — modified: tests
  T1..T13 (43 assertions)
- `tethers-0.1/engine-ocaml/bin/dune` — modified: plan test module list gains
  `tethers_protocol tether_parser tethers_error`
- `docs/CURRENT_CLINE_TASK.md` — task packet updated to CORE-5B

## Decisions and assumptions

1. **`runtime_capability_projection` reuses `Tethers_protocol.capability`.**
   The projection record is `{ capability_id; contract_digest; runtime }` where
   `runtime : Tethers_protocol.capability` carries the existing runtime
   capability schema (name, version, inputs, effects, and optional
   manifest/bridge fields). This reuses the accepted existing type instead of
   duplicating its fields.

2. **Projection resolution is fail-closed with precise typed errors.** For an
   Action's `capability_id` + `contract_digest`: no projection for that identity
   → `Missing_capability_projection`; identity absent but digest present under
   another identity → `Capability_projection_identity_mismatch`; identity
   present but digest mismatch → `Capability_projection_digest_mismatch`;
   identity+digest match but runtime metadata incomplete (empty name/version or
   partially-present bridge fields) → `Capability_projection_incomplete`. The
   bridge never substitutes another capability version or contract.

3. **Explicit completion.** The walk returns `Incomplete_success_path(origin)`
   when an origin has no `success_continuation`; only an explicit
   `Program_complete` target terminates the plan. This replaces CORE-5A's
   "missing continuation = completion" behaviour.

4. **Occurrence identity.** `plan.id = evaluation_id ^ "/plan"` and
   `idempotency_key = evaluation_id ^ "/action_N"`. `program_id` is never used
   as an occurrence identity; it remains Core logical identity only.

5. **Existing action contract.** Each planned Action carries exactly the
   existing Runtime Plan Action vocabulary from `tethers_evaluator.ml`:
   `action_id`, `idempotency_key`, `capability`, `capability_version`,
   `arguments`, `effects`, plus `manifest_digest`/`bridge_capability_version`/
   `bridge_provider_identity` when present on the projection. The CORE-5A
   nonstandard `capability_contract_digest` field was removed; the Core digest
   is now consumed by projection pinning verification, not emitted as an
   action field.

6. **Effects aggregation.** `required_effects` aggregates each planned
   Action's projection effects in control-flow order and deduplicates with
   first-occurrence uniqueness, matching the existing evaluator's `unique`
   behaviour. Effects come from the approved projections only; nothing is
   inferred from capability names.

7. **Pre-scan unchanged.** The CORE-5A unsupported-construct pre-scan
   (Together, Batch, Branch, item templates, Role_proxy, input bindings,
   execution constraints) is preserved verbatim; no support was broadened.

## Evidence

All commands ran against implementation checkpoint
`b0e194b2da9331ef2455674a30bd427a5d1873d8`.

| Command | Result |
| --- | --- |
| `dune build @all` | PASS (exit 0) |
| `dune runtest --force` | PASS — lowerer 49/49, validator 51/51, plan bridge 43/43 |
| `git diff --check` | PASS |
| `cargo fmt --check` | PASS (RUST_UNCHANGED) |
| `git show --name-status b0e194b` | only the 4 authorised code paths |
| `git status --short --branch` | clean except uncommitted packet (closeout) |
| `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` | `control-v1/IN_PROGRESS` before closeout; `control-v1/COMPLETE` after closeout |

**New/updated tests:** 21 test functions, 43 assertions:

- T1 `test_explicit_completion` — A → Program_complete plans one Action with
  correct occurrence id and capability identity
- T2 `test_missing_terminal_continuation` — A → B with B having no
  continuation returns `Incomplete_success_path(B)`
- T3 `test_runtime_occurrence_identity` — eval_123 + MY_PROGRAM yields
  `plan.id = eval_123/plan`, not `MY_PROGRAM`
- T4 `test_program_id_not_occurrence` — different ProgramIds, same context,
  equal occurrence-derived `plan.id`
- T5 `test_existing_action_shape` — one literal Action carries `action_id`,
  `idempotency_key`, `capability`, `capability_version`, `arguments`,
  `effects`, `manifest_digest`, `bridge_capability_version`,
  `bridge_provider_identity` exactly
- T6 `test_capability_projection_missing` — missing projection fails explicitly
- T7 `test_capability_digest_mismatch` — identity match, digest mismatch fails
- T8 `test_effects_aggregation` — overlapping effects yield unique
  first-occurrence `required_effects`
- T9 `test_idempotency_keys` — eval_X/action_1 and eval_X/action_2
- T10 `test_storage_order_independence` — reversed `origin_sites` produce equal
  Runtime Plans
- T11 (9 tests) — Together, Batch, Branch, Fact_through_role, Anchor_value,
  Fact_from_origin, Deadline, ItemTemplate, invalid Core all fail closed as in
  CORE-5A
- T12 `test_capability_identity_mismatch` — digest approved only under a
  different CapabilityId fails with identity-mismatch
- T13 `test_capability_projection_incomplete` — approved projection with empty
  runtime name fails closed

**Commands not run:**

- Fixture suite (`check-fixtures.ps1`, `test-engine.ps1`), MCP transcript suite,
  Rust host tests: NOT RUN — CORE-5B is a dormant sidecar bridge; no evaluator,
  protocol, MCP, or Rust code changed and no production wiring was added.

## Publication evidence

Branch `feature/core-5-runtime-plan-bridge` pushed normally to `origin`.
Remote HEAD resolved and confirmed equal to local HEAD; `git status --short
--branch` clean. See completion report for the full remote HEAD SHA.

## Discoveries

- `tethers_protocol.capability` already carries the full runtime plan action
  vocabulary (name, version, effects, and optional manifest/bridge fields), so
  the projection could reuse it directly rather than introducing a parallel
  type. The `inputs` field is present in the reused type but is not consumed by
  this bridge (argument values come from Core literal inputs).

## Remaining risks

- None known within packet scope. The bridge is dormant and not wired to any
  executable or runtime; its only consumer is the new test executable. The
  packet's frozen decision that `Anchor_value` and `Fact_from_origin` fail
  closed (representation incompatibility) remains unchanged.

## Smallest next action

Lucy reviews the pushed branch against the acceptance criteria; CORE-5C (any
runtime wiring) is not started by this worker.

## References

- Branch: `feature/core-5-runtime-plan-bridge`
- Base: `a28bdf483db7959f5471b4b950802e863093d9f8` (CORE-5A closeout HEAD)
- Implementation checkpoint: `b0e194b2da9331ef2455674a30bd427a5d1873d8`
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml`
- `tethers-0.1/engine-ocaml/bin/dune`
- `docs/CURRENT_CLINE_TASK.md`
