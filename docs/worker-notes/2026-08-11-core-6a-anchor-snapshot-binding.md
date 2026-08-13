# Worker Note

Task: `CORE-6A1 — Human → Core → Plan Proof`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `6e1aba9f2ade3c24c43badc77d20b2094e791f3a`

Implementation checkpoint: `7586b29d20133879af47ca8fd0d22878c85710de`

## Requested outcome

Add the end-to-end test requested by CORE-6A: prove the actual Human → parser → Core lowerer → planner chain works correctly for Anchor snapshot binding. The previous report said this was not possible without parser/lowerer integration changes. Independent review found that it is possible entirely inside the test layer. `Tether_parser.parse_tether` and `Tethers_core_lowerer.lower` are already public APIs. A Dune test-module dependency change is authorised.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml` — added `test_e2e_human_to_plan` that parses a Human Tether with `anchor.document.title`, lowers it with explicit capability mapping, verifies the lowered Core contains `Anchor_value (O_anchor, ["document"; "title"])`, then plans with a runtime snapshot and asserts the resolved argument is `"title": "Tethers"`
- `tethers-0.1/engine-ocaml/bin/dune` — added `tethers_core_lowerer` to the plan-test module list to enable direct lowerer calls in the test
- `docs/CURRENT_CLINE_TASK.md` — updated to CORE-6A1 with implementation checkpoint

## Decisions and assumptions

- The test uses `Tether_parser.parse_tether` and `Tethers_core_lowerer.lower` directly, proving the existing public APIs work end-to-end without any production code changes
- A Dune test-module dependency change is authorised per the task packet
- No parser, lowerer, or planner semantics were modified; the test only exercises existing code paths
- The test constructs a `lowering_environment` with a single capability binding mapping `notify` to `cap.notify` with a known contract digest

## Evidence

- OCaml build: `dune build` — PASS (exit 0)
- All tests: `dune runtest` — PASS (74/74 plan bridge tests)
- Whitespace: `git diff --check` — PASS (no errors, only line-ending conversion warnings)
- Cargo fmt: `cargo fmt --check` — PASS (RUST_UNCHANGED)
- Diff inspection: only authorised files changed (tethers_core_plan_test.ml, dune, CURRENT_CLINE_TASK.md)
- Git status: clean worktree
- Task-packet checker: `control-v1/COMPLETE` (after worker note creation)

## Publication evidence

Branch: `feature/core-6-anchor-snapshot-binding`. Push to origin pending (awaiting closeout documentation commit).

## Discoveries

- `Tether_parser.parse_tether` and `Tethers_core_lowerer.lower` are already public APIs that can be called directly from tests
- The lowerer produces `Anchor_value (O_anchor, ["document"; "title"])` from `anchor.document.title` syntax, confirming the Human → Core lowering path
- The planner correctly resolves the anchor value through the full pipeline, producing `"title": "Tethers"` as the concrete Runtime Plan argument
- Adding `tethers_core_lowerer` to the plan-test module list was the only build configuration change needed

## Remaining risks

None known within packet scope.

## Smallest next action

Push the finished branch to origin and confirm local HEAD == remote HEAD.

## References

- Task packet: `docs/CURRENT_CLINE_TASK.md`
- Implementation: `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml`, `tethers-0.1/engine-ocaml/bin/dune`
- Base commit: `6e1aba9f2ade3c24c43badc77d20b2094e791f3a`
- Implementation commit: `7586b29d20133879af47ca8fd0d22878c85710de`
