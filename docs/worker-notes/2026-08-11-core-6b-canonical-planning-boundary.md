# Worker Note

- **Task Packet:** `docs/CURRENT_CLINE_TASK.md` (CORE-6B)
- **Owner:** OpenCode (MiMo-V2.5)
- **Status:** `COMPLETE`
- **Base Commit:** `534abc763938f573fa799619ffa22193206e3b15`
- **Implementation Commit:** `dac6cce92287b2ad853b3f435063c96359c8d1e5`

## Requested outcome

Make the canonical Core → Runtime Plan boundary explicit and tested: add a
`plan_canonicalized` entry point requiring already-canonicalised Core, prove
identity invariance across ProgramId, temporary OriginId, and storage order
variations, and prove the Human → parser → lowerer → canonicaliser → planner
chain works end-to-end with canonical Anchor snapshots.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml` — added `canonical_plan`
  record type and `plan_canonicalized` function (12 lines)
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.mli` — exposed
  `canonical_plan` type and `plan_canonicalized` signature with documentation
  (24 lines)
- `tethers-0.1/engine-ocaml/bin/dune` — added `tethers_core_canonical` module
  and `digestif` library to the plan-test stanza
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml` — added 8 CORE-6B
  test functions (439 lines), new test helpers `assert_ok_canonical` and
  `assert_ok_canonical_plan`

## Decisions and assumptions

- `plan_canonicalized` delegates to the existing `plan` function by extracting
  `canonical_program` from the `canonicalized` value. No planning logic is
  duplicated.
- `canonical_plan` wraps `Tethers_outcome.plan` with `ProgramDigest`; it is not
  a second Runtime Plan model.
- The existing `plan` function is preserved as the lower-level API. It remains
  usable for focused tests and non-canonical contexts.
- Anchor snapshot tests locate the canonical Anchor OriginId dynamically from
  the canonical program's `origin_sites` rather than assuming a fixed canonical
  ID, because the canonicaliser assigns IDs based on colour sorting.
- For B7 (storage order independence), programs with different pre-canonical
  OriginIds (`O_x`/`O_y` vs `O_a`/`O_b`) canonicalise to structurally equal
  programs, confirming the canonicaliser eliminates pre-canonical identity
  variation.
- For B8 (stale snapshot), the snapshot is keyed by the pre-canonical OriginId
  `O_anchor`. After canonicalisation, the anchor has a canonical OriginId (e.g.
  `O1`). The planner correctly returns `Missing_anchor_snapshot` for the
  canonical OriginId, proving the pre-canonical key does not silently work.

## Evidence

- `dune build @all` — PASS (exit 0)
- `dune runtest --force` — PASS
  - `PASS all lowerer tests (49/49)`
  - `PASS all validator tests (51/51)`
  - `PASS all plan bridge tests (101/101)` (was 93 before CORE-6B)
- `git diff --check` — PASS (CRLF checkout warnings only)
- `cargo fmt --all -- --check` — PASS (RUST_UNCHANGED)
- `git status --short` — clean

### Test coverage

| Test | Requirement | Result |
|------|-------------|--------|
| CB-T1 | Canonicalized entry point produces Runtime Plan | PASS |
| CB-T2 | Returned ProgramDigest matches canonicalized digest | PASS |
| CB-T3 | Human → parser → lowerer → canonicalize → planner E2E proof | PASS |
| CB-T4 | ProgramId variation leaves digest and occurrence unchanged | PASS |
| CB-T5 | Pre-canonical ID variation canonicalises to equal plans | PASS |
| CB-T6 | Anchor snapshot keyed by canonical OriginId resolves | PASS |
| CB-T7 | Stale pre-canonical Anchor OriginId fails | PASS |
| CB-T8 | Existing CORE-6A planner tests remain green (low-level + canonical) | PASS |

## Publication evidence

- Branch pushed: `feature/core-6b-canonical-planning`
- Remote HEAD SHA: `dac6cce92287b2ad853b3f435063c96359c8d1e5`
- Local HEAD: `dac6cce92287b2ad853b3f435063c96359c8d1e5`
- Local HEAD == remote HEAD: confirmed
- `git status --short`: clean

## Discoveries

- `origin_id = private Origin_id of string` — the `private` keyword restricts
  construction but structural equality works correctly via `origin_id_of_string`.
- Canonical Anchor OriginIds are assigned based on colour-sorted position (e.g.
  `O1`, `O2`), not by the pre-canonical name. Tests must locate the canonical
  OriginId dynamically from the canonical program's `origin_sites`.
- The `assert_plan_error` helper uses structural equality on `planning_error`,
  which means the full error payload (including OriginId) must match — string
  representations are insufficient for comparison.

## Remaining risks

None known within packet scope.

## Smallest next action

Await independent review of pushed evidence. If accepted, Lucy may compile the
next task packet (e.g. CORE-6C runtime wiring or a different milestone).

## References

- Task packet: `docs/CURRENT_CLINE_TASK.md`
- Base commit: `534abc763938f573fa799619ffa22193206e3b15`
- Implementation commit: `dac6cce92287b2ad853b3f435063c96359c8d1e5`
- Branch: `feature/core-6b-canonical-planning`
- Files changed: `tethers_core_plan.ml`, `tethers_core_plan.mli`,
  `tethers_core_plan_test.ml`, `dune`
