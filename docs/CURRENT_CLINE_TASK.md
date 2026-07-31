# Current Implementation Task

Control contract: `1`

Task: `J16D - complete the repaired clean native Windows verification gate`

Owner: `Codex`

Status: `COMPLETE`

Task colour: `Red`

Route: `Codex native Windows complete clean verification`

Base commit: `75186ce4413c0fbf860d258b86d7adecadcff780`

Branch: `codex/j16-clean-checkout-proof`

Worker note: `docs/worker-notes/2026-08-01-j16d-complete-clean-verification.md`

## Objective

Complete the repaired clean native Windows verification gate from the J16
checkout and retain reproducible external evidence for Lucy's independent
acceptance review.

## Relevant background and existing behaviour

J16A established the clean checkout, J16B reconstructed its native toolchains,
J16C proved restart/replay, and J16D-F1 repaired the observed Ctrl+C race. The
previous J16D harness had incomplete evidence; J16D-R2 captured the genuine
race, and J16D-R3 reran the complete gate with one durable log per command.

## Required behaviour

1. Run each complete-gate command once with durable external evidence.
2. Prove both toolchains, all repository scripts, the runner contract, and the
   consolidated matrix pass from the clean checkout.
3. Reconcile the full Rust suite by its actual Cargo targets.
4. Preserve prior evidence, source boundaries, process integrity, and temporary
   directory integrity.
5. Record completion without publishing to `main` or beginning J17.

## Relevant components

- J16 checkout: `D:\The Next Thing\Tethers Lang - J16 Clean`.
- External evidence: `C:\Users\Matmus\AppData\Local\Temp\J16D-R3-6d997f4f-cb20-4d68-bc04-7de37261399e`.
- Rust host, path-bound OCaml switch, fixture/engine/MCP/host/demo scripts, and
  the J15 consolidated verifier.

## Frozen decisions and invariants

- All 20 planned verification children completed once with exit `0`.
- Rust totals are `797 passed, 0 failed, 0 ignored`: 44 `src/lib.rs` tests, 724
  `src/main.rs` tests, 29 `tests/j13a_cli.rs` integration tests, and 0 doctests.
  The earlier 768 figure omitted the successful 29-test integration target.
- Runner contract: six rows passed with 49 assertions. Consolidated matrix: six
  suites, six passed, zero failed, 79 accepted release cases/rows, `RESULT: PASS`.
- J13B test 10 passed as `interrupted`, not `unavailable`.
- J16D proves verification only; publication to `main` and J17 remain deferred.

## Acceptance criteria

1. Toolchain, Rust, OCaml, fixture, engine, MCP, host, demo, runner-contract,
   and consolidated-matrix steps have one durable exit-0 record each.
2. Cargo target headings and totals reconcile exactly to 797 with no failures or
   ignored tests.
3. The runner contract and six-suite, 79-case matrix contain their required PASS
   markers and assertion counts.
4. Retained evidence, source/lock boundaries, process state, and current test
   temporary roots are clean.
5. Only this packet and its worker note change; main is not pushed and J17 does
   not begin.

## Required verification

- J16D-R3 `steps.jsonl`, `summary.txt`, separate logs `00` through `19`,
  `runner.ps1`, `plan.json`, and inventory in the retained external directory.
- Final packet checker, whitespace, changed-path, and Git-status checks captured
  as separate external final steps before the completion commit.

## Forbidden changes

- No implementation, test, script, fixture, manifest, lock, `main`, or J17
  change; no verification child may be rerun.

## Stop conditions

- A non-zero verification child, incomplete evidence, changed unauthorised path,
  altered retained evidence, live J16 executable, or unresolved temporary-root
  residue stops the task.

## Expected pre-existing changes

None.

## Commit and publication boundary

Create exactly one commit: `test: complete j16 clean verification`; push only
`codex/j16-clean-checkout-proof`.

## Return contract

Return the evidence location and hashes, exact Rust target totals, all twenty
exit-0 results, matrix/runner results, changed paths, branch topology, and final
cleanliness.
