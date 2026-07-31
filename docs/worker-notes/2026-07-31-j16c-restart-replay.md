# Worker Note — J16C restart and durable replay

Task: `J16C - prove restart and durable replay from the clean checkout`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `75186ce4413c0fbf860d258b86d7adecadcff780`

Implementation checkpoint: `3aa1108d159a4d358c408752f0c31389ed9d383e`

## Requested outcome

Prove restart/replay and J14C from the clean checkout without implementation changes.

## Changes made

- Added only this note and the J16C packet.

## Decisions and assumptions

- Start: `codex/j16-clean-checkout-proof` at `3aa1108d159a4d358c408752f0c31389ed9d383e`, `2/0` against origin/main.
- J16B switch-creation wrapper timeout remains honestly recorded; the verified switch was reused.

## Evidence

- Toolchain gate PASS. Inventory: `rustup run 1.89.0 cargo test replay -- --list`.
- Claims: restart zero calls `j09_replay_runtime_native_fresh_success_restart_makes_zero_second_call`; ID recovery `ledger_06` and `ledger_30`; completed `ledger_24/25`; incomplete `recovered_claim/g0/g1/uncertain`; immutable generations `recovered_claim_g0_and_g1_admissions_cannot_advance_or_mutate` and `recovered_terminal_admission_cannot_publish_or_mutate`; exclusion `ledger_01`; termination `ledger_02`; independent keys `ledger_03`; binding mismatch `ledger_09`.
- `cargo test replay -- --nocapture`: 118 passed, 0 failed, 0 ignored, exit 0, `00:00:01.1373973`; fresh harness roots were created by its native mechanism and no test retried.
- J14C once: 9/9 rows, 196 assertions, RESULT PASS, exit 0, `00:00:08.1891185`; first move `1`, replay move `0`, same execution ID `exec_f2b14a5b-f502-47d7-ab27-4d419ecca951`.
- PID 18360 is unrelated original-checkout engine; parent 25156 `opam.exe`; zero executable paths beneath J16 Clean; no process terminated and no verification command reran after diagnosis.
- Final packet checker PASS; status before commit contains only the two authorised paths.

## Discoveries

- J16C scope excludes all global processes outside the clean checkout.

## Remaining risks

- J16D complete clean verification and J17 remain deferred.

## Smallest next action

J16D only under a new packet.

## References

- `docs/J09_DURABLE_REPLAY_DESIGN.md`
- `docs/worker-notes/2026-07-31-j16b-clean-build.md`
- `tethers-0.1/scripts/test-j14c-real-file-move.ps1`
