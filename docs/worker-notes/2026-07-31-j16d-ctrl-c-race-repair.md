# Worker Note

Task: `J16D-F1 - make Ctrl+C classification deterministic`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `75186ce4413c0fbf860d258b86d7adecadcff780`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Repair the J16D-R2-discovered Windows Ctrl+C classification race without
weakening J13B's public interruption acceptance. Preserve ordinary provider
exit behaviour and leave J16D-R3 as the complete release gate.

## Changes made

- `tethers-0.1/host-rust/src/child_process.rs` — on stdout-reader disconnect,
  constructs the ordinary disconnect error then observes the host interruption
  flag for at most 50 ms in 1 ms pauses. A visible flag returns
  `ChildError::Interrupted`; otherwise the original error, including
  `ChildError::ProcessExited`, is returned.
- `tethers-0.1/host-rust/src/child_process.rs` — four injected clock/pause tests
  prove immediate interrupt, slightly late interrupt, ordinary exit, and bounded
  termination without scheduler luck.
- `docs/CURRENT_CLINE_TASK.md` — records J16D-F1 completion.

## Decisions and assumptions

The production seam, rather than the provider mapping, owns the precedence
decision because it alone sees both reader disconnection and the host interrupt
state. The 50 ms window is deliberately small relative to the J13B five-second
public limit and is bounded to avoid materially delaying ordinary provider
failure. No public mapping changed: the existing `stdio_provider` mapping still
converts a non-interrupted `ProcessExited` to the ordinary unavailable path.

## Evidence

- J16D-R2 retained command `pwsh -NoProfile -ExecutionPolicy Bypass -File
  .\tethers-0.1\scripts\verify-0.2.ps1` — exit `1`, `00:01:29.7983394`;
  J13B test 10 expected `interrupted` and observed `unavailable`.
- The historical J13B note records the same earlier non-repeatable unavailable
  observation before a clean repeat passed.
- The initial F1 `cargo fmt --check` stopped only for formatting. This explicit
  continuation ran `rustup run 1.89.0 cargo fmt --all` once, exit `0`, and it
  changed no path other than `child_process.rs`.
- `rustup run 1.89.0 cargo fmt --check` — exit `0`, `00:00:00.5536832`.
- `rustup run 1.89.0 cargo check --locked` — exit `0`, `00:00:01.0589177`.
- `rustup run 1.89.0 cargo test --locked child_process -- --nocapture` — exit
  `0`, `00:00:08.8299791`: 13 child-process tests plus one name-matched replay
  test passed; 0 failed, 0 ignored.
- `rustup run 1.89.0 cargo test --locked j13b -- --nocapture` — exit `0`,
  `00:00:01.4170768`: 2 CLI and 49 host tests passed; 0 failed, 0 ignored.
- `rustup run 1.89.0 cargo test --locked` — exit `0`; 768 tests passed across
  the 724-unit and 44 CLI test targets, 0 failed, 0 ignored.
- Five independent `pwsh -NoProfile -ExecutionPolicy Bypass -File
  .\tethers-0.1\scripts\test-j13b-run.ps1` processes all exited `0` and printed
  `J13B public run acceptance: 10 passed, 0 failed`: iteration 1
  `00:00:07.2837846`; iteration 2 `00:00:07.2603643`; iteration 3
  `00:00:07.2702947`; iteration 4 `00:00:07.3232824`; iteration 5
  `00:00:07.2214413`.
- Test 10 in the existing public script directly asserts public status
  `interrupted`, exit `10`, error code `INTERRUPTED`, interrupt duration at most
  five seconds, and zero `tools/call`; no expectation was weakened. Iteration 1
  passed before an external recorder incorrectly searched for a non-emitted
  zero-call output string; source inspection confirmed the assertion is internal
  to the passing test, and iterations 2–5 ran separately without that recorder
  condition. No test process was rerun.
- `RUSTUP_AUTO_INSTALL` and `OPAMSWITCH` were set only in their respective
  verification process scopes and restored in `finally`.
- Both retained evidence directories were hash-checked unchanged. No executable
  path beneath J16 Clean and no `Tethers J13B run *` temporary directory remained.

## Discoveries

The stdout reader had a correct pre-receive interruption check but no bounded
precedence observation after the disconnected-receiver/process-exit seam. That
allowed Windows scheduling of the shared console event to surface the provider
exit first.

## Remaining risks

The five-run public sample demonstrates stability but is not a replacement for
the complete J16D-R3 release verification gate, which remains required.

## Smallest next action

Lucy independently reviews this Red repair, then authorises J16D-R3 for the
complete clean verification gate.

## References

- Branch: `codex/j16-clean-checkout-proof`; starting SHA:
  `e683077eeb83e688ddd90f71030d721165111a63`.
- Retained partial evidence:
  `C:\Users\Matmus\AppData\Local\Temp\J16D-8d5b20f3-7c55-47d8-85c9-4057b64f3e11`.
- Retained J16D-R2 evidence:
  `C:\Users\Matmus\AppData\Local\Temp\J16D-R2-674d594e-43ab-4859-9933-f252c5a4f40e`.
- Failed consolidated log:
  `C:\Users\Matmus\AppData\Local\Temp\J16D-R2-674d594e-43ab-4859-9933-f252c5a4f40e\22.log`.
