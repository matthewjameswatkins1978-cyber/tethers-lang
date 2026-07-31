# Current Implementation Task

Control contract: `1`

Task: `J16C - prove restart and durable replay from the clean checkout`

Owner: `Codex`

Status: `COMPLETE`

Task colour: `Red`

Route: `Codex native Windows replay proof`

Base commit: `75186ce4413c0fbf860d258b86d7adecadcff780`

Branch: `codex/j16-clean-checkout-proof`

Worker note: `docs/worker-notes/2026-07-31-j16c-restart-replay.md`

## Objective

Prove restart and durable replay from `D:\The Next Thing\Tethers Lang - J16 Clean`.

## Relevant background and existing behaviour

J16C began at `3aa1108d159a4d358c408752f0c31389ed9d383e`; J16B's path-bound switch remains at `tethers-0.1\engine-ocaml`.

## Required behaviour

1. Run the existing focused replay inventory and suite once.
2. Map every restart/replay claim to named existing tests.
3. Run J14C once through the public route.
4. Prove no J16 Clean executable remains running.
5. Update only this packet and worker note.

## Relevant components

- `tethers-0.1/host-rust` replay tests; `tethers-0.1/scripts/verify-0.2.ps1`.

## Frozen decisions and invariants

- Replay suite: `118 passed, 0 failed, 0 ignored`, `00:00:01.1373973`, exit `0`.
- J14C: `9 rows, 9 passed, 0 failed`, `196 assertions`, `RESULT: PASS`, `00:00:08.1891185`, exit `0`.
- First move count `1`; replay move count `0`; execution ID was retained.
- PID `18360` is `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml\_build\default\bin\tethers_mcp_main.exe`, parent PID `25156` `opam.exe`, and unrelated; zero executable paths are beneath J16 Clean.
- No test rerun, source, lock, fixture, test, or script change occurred. J16D and J17 have not begun.

## Acceptance criteria

1. Toolchain gate PASS and clean preflight are recorded.
2. Claims map to `j09_replay_runtime_native_fresh_success_restart_makes_zero_second_call`; `ledger_06_restart_recovers_same_execution_identity`/`ledger_30_restart_never_generates_new_uuid_for_existing_tuple`; `ledger_24`/`ledger_25`; recovered claim/g0/g1/uncertain tests; recovered-generation and terminal-mutation tests; `ledger_01`; `ledger_02`; `ledger_03`; and `ledger_09_binding_mismatch_fails_closed`.
3. Replay and J14C results above pass exactly once.
4. Only the two authorised documentation paths change.
5. Packet and whitespace checks pass before the single commit.

## Required verification

- Captured `cargo test replay -- --list`, `cargo test replay -- --nocapture`, J14C, and toolchain-gate results.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File .\.github\scripts\check-tethers-task-packet.ps1`; `git diff --check`; status checks.

## Forbidden changes

- Tests, source, locks, manifests, scripts, fixtures, J16D, J17, or `main`.

## Stop conditions

- Any changed unauthorised path, failed required evidence, or executable beneath J16 Clean.

## Expected pre-existing changes

None.

## Commit and publication boundary

Create exactly one commit: `test: prove j16 restart and durable replay`; push only `codex/j16-clean-checkout-proof`.

## Return contract

Return the replay/J14C proof, process provenance, commit, Git state, and stop.
