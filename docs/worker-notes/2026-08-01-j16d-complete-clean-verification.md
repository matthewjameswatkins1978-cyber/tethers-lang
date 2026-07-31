# Worker Note

Task: `J16D - complete the repaired clean native Windows verification gate`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `75186ce4413c0fbf860d258b86d7adecadcff780`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Run the complete repaired J16 clean-checkout verification gate once, retain one
durable external record per child process, and record accurate acceptance
evidence without beginning J17 or publishing to main.

## Changes made

- `docs/CURRENT_CLINE_TASK.md` — replaces J16D-F1 with the completed J16D gate.
- `docs/worker-notes/2026-08-01-j16d-complete-clean-verification.md` — records
  the complete J16D-R3 evidence reconciliation.

## Decisions and assumptions

The original `768` Rust figure was not a product failure: the retained Cargo
output names a successful 29-test `tests\j13a_cli.rs` integration target that
the earlier arithmetic omitted. The authoritative total is therefore
`44 + 724 + 29 + 0 = 797`, with zero failed and zero ignored. The initial R3
runner had an optional-plan-field strict-mode error before starting any child;
its evidence manifest contained zero records, it was corrected externally, and
each planned verification child then ran exactly once.

## Evidence

Checkout: `D:\The Next Thing\Tethers Lang - J16 Clean`.

Toolchains: Rust target `D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\host-rust\target`; OCaml switch `D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\engine-ocaml\_opam`.

J16D-R3 evidence directory: `C:\Users\Matmus\AppData\Local\Temp\J16D-R3-6d997f4f-cb20-4d68-bc04-7de37261399e`.

- `runner.ps1` SHA-256 `D57AE505FFC740A804BCFB3E6B96433BD2FB5B2B09A2B83D8952F8BC604CBE5C`.
- `plan.json` SHA-256 `21AC25C0035CC96D561D24A4620ED5889CE12B7F843672C479AC53B9EA02FF0B`.
- `steps.jsonl` SHA-256 `807AAA6699F97848CF65704847F63E4A44E9B8290B2CE9C26ED052FBE8A66A3B`.
- `summary.txt` SHA-256 `EE5673B8A1F029AD911D863EAF7F38A46D9B8790CFA308E5F817DBDF1BD32189`.
- Step `00` inventory — exit `0`, `00:00:00.0573788`, `00.log`, SHA-256 `62AFD05BB5791A6344192F5962008EA1A22A2246B421AD37E9EE651F2FA63A6F`.
- Step `01` toolchain gate — exit `0`, `00:00:04.9772700`, `01.log`, SHA-256 `2545B1A09868F8E0A8EEE97B279F4717C8593BAF57BA1FC7452B401980ECB50C`; `All toolchain checks passed.`
- Step `02` focused toolchain test — exit `0`, `00:00:10.4202133`, `02.log`, SHA-256 `7963500671F6300E9565D71649C758E8F574CDB3D8741BF405594D0F93DBCCC8`; 23 passed, 0 failed.
- Steps `03`–`07` Rust fmt/check/test/debug-build/release-build — all exit `0`: `00:00:00.4924504`, `00:00:00.2529437`, `00:00:03.4570203`, `00:00:00.1973501`, `00:00:07.9927919`; logs `03.log`–`07.log`, SHA-256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855`, `A3330B634F9876B3F7E99CFE9B2635D6B73D629C8C515D0AD605912F6F7C6577`, `B07A58C583B40B540242144327FDF02740BA2690572EF5769AB9F9A8BC74C3A2`, `0EB9AF1228C879977E6E29D5874788B5DC00809F019EFB128470724592F24540`, `A9F4C9A1EB9CBE318A04D29E7E550C29B574A820E20FD5E31F3BA6504E5A9939`.
- Step `05` exact target headings: `unittests src\lib.rs` 44 passed; `unittests src\main.rs` 724 passed; `tests\j13a_cli.rs` 29 passed; `Doc-tests tethers_reference_host` 0 passed. Aggregate: 797 passed, 0 failed, 0 ignored.
- Step `08` OCaml `dune build` — exit `0`, `00:00:00.8227336`, `08.log`, SHA-256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855`.
- Step `09` fixtures — exit `0`, `00:00:00.4741967`, `09.log`, SHA-256 `FA88DE85E340FBD92AD66CC5A4057AED3FC0A6730B4A92F792C85F8D93A7674E`; 46 JSON and 30 JSONL files valid.
- Step `10` engine fixtures — exit `0`, `00:00:02.7859541`, `10.log`, SHA-256 `EE93DA4F2C13FCA7C0505EF289FCD50BE33D14461D312AD7FF40AB07C9A7BB30`; all listed fixture cases match.
- Step `11` MCP transcripts — exit `0`, `00:00:01.0163474`, `11.log`, SHA-256 `065B386864740F6F891B1EEC565AC8823C904BF05B012CC445046FBE8818A58C`; 15 cases complete.
- Host inventory, in lexical order: `test-host-denial.ps1`, `test-host-event-admission-trail.ps1`, `test-host-event-admission.ps1`, `test-host-execution-failure.ps1`, `test-host-result-follow-up.ps1`.
- Steps `12`–`16` host scripts — all exit `0`: `00:00:01.5369750`, `00:00:01.5210477`, `00:00:00.6426979`, `00:00:01.4351728`, `00:00:01.2628041`; logs `12.log`–`16.log`, SHA-256 `9B3574FE72CEC304016AA5833A0C076DD89B365CD7C1E625CF521E2F2F95A4DD`, `D0F4C4D3D0A29B4D884AA5B016EBE0B074B663341D1D76B187F6A72BA3F902AA`, `207B14D11A2AD028C552737BC6C5BE90728B454971E6FA7BECCDD802DA77418E`, `714CD000D9DB040D5335A814B53EBE888596B7C98201F260976B93F3BB345FFD`, `730EF87A1C3020E60A15FAC69CAD64BA053A5F1AC220234BC41AD5056E8067E3`.
- Step `17` demo — exit `0`, `00:00:01.4604862`, `17.log`, SHA-256 `700A0E9193385BB0793209BE750F7A8D836963D3596D27FF18CCB54D59F82207`; existing unassessed-scope denial demo passed.
- Step `18` runner contract — exit `0`, `00:00:04.7756861`, `18.log`, SHA-256 `277031B3624ADC13DA7595C4D4FAFED3CD631E2610402FF85BF1C11DF7785601`; rows R01–R06, 6/6 passed, 0 failed, 49 assertions.
- Step `19` consolidated matrix — exit `0`, `00:01:20.9873802`, `19.log`, SHA-256 `36DEEE0F9D6B4A9781A156B956B5DB316240A3943807AB30EE6059C64EF4D1F7`; J13A 25/0, J13B 10/0 with test 10 interrupted, J13C 19/0, J14A 5/0 and 95 assertions, J14B 11/0 and 243 assertions, J14C 9/0 and 196 assertions; 6/6 suites, 79 accepted cases/rows, `RESULT: PASS`.
- `RUSTUP_AUTO_INSTALL` restored before step 08 and `OPAMSWITCH` restored in runner `finally`; no verification command was rerun.
- Prior preserved evidence: `C:\Users\Matmus\AppData\Local\Temp\J16D-8d5b20f3-7c55-47d8-85c9-4057b64f3e11` and `C:\Users\Matmus\AppData\Local\Temp\J16D-R2-674d594e-43ab-4859-9933-f252c5a4f40e`; both inventories were hash-identical to their recorded retained values.
- The previous J16D harness stopped before durable combined-chain evidence. J16D-R2 captured the genuine J13B Ctrl+C race; J16D-F1 repaired it in commit `b4f2cda96bb8e65f05866e3f37948af15caf8273`.

## Discoveries

Restricted single-branch fetch configuration left this checkout's local
`origin/codex/j16-clean-checkout-proof` tracking ref stale while GitHub and
local HEAD were already correct. One explicit one-off refspec fetch refreshed
that tracking ref; no repository file or permanent Git configuration changed.

## Remaining risks

J16 is complete pending Lucy's independent acceptance. Publication to main and
J17 remain outside this task.

## Smallest next action

Lucy reviews the retained J16D-R3 evidence and issues either acceptance or the
smallest bounded correction; do not begin J17 without that decision.

## References

- Starting SHA: `b4f2cda96bb8e65f05866e3f37948af15caf8273` on `codex/j16-clean-checkout-proof`.
- Base: `75186ce4413c0fbf860d258b86d7adecadcff780`.
- Current J14C cleanup in `19.log` removed its own temporary root; no executable
  path beneath J16 Clean remained after the gate.
