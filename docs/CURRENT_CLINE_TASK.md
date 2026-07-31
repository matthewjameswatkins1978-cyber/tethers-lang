# Current Implementation Task

Control contract: `1`

Task: `J16D-F1 - make Ctrl+C classification deterministic`

Owner: `Codex`

Status: `COMPLETE`

Task colour: `Red`

Route: `Codex native Windows interruption-race repair`

Base commit: `75186ce4413c0fbf860d258b86d7adecadcff780`

Branch: `codex/j16-clean-checkout-proof`

Worker note: `docs/worker-notes/2026-07-31-j16d-ctrl-c-race-repair.md`

## Objective

Make the Windows Ctrl+C classification at the provider stdout-disconnect and
process-exit seam deterministic without changing public interruption, provider,
replay, permission, or Trail semantics.

## Relevant background and existing behaviour

J16D-R2 step 22 found that the blocked-provider J13B case could report
`unavailable` after `CTRL_C_EVENT`: the provider could disconnect stdout before
the host reader saw the global interruption flag. The historical J13B worker
note had recorded the same non-repeatable observation. Ordinary provider exits
without a host interruption remain `unavailable`.

## Required behaviour

1. Give explicit host interruption precedence over a concurrently caused stdout
   disconnect or process exit.
2. Preserve ordinary provider-exit classification when no interruption appears.
3. Prove immediate, slightly late, absent, and bounded interruption observation
   deterministically in the child-process module.
4. Prove public J13B interruption stability in five planned independent runs.
5. Preserve the frozen public interruption expectation and defer J16D-R3.

## Relevant components

- `tethers-0.1/host-rust/src/child_process.rs` owns the supervised stdout reader.
- `tethers-0.1/scripts/test-j13b-run.ps1` is the existing public acceptance proof.

## Frozen decisions and invariants

- On reader disconnect, the host observes `INTERRUPTED` for at most `50 ms`, in
  `1 ms` pauses; a visible interruption returns `ChildError::Interrupted`.
- The helper is clock/pause-injected for deterministic unit tests. It returns
  the original `ChildError::ProcessExited` when no interrupt becomes visible.
- The public interruption envelope remains `interrupted`, exit `10`, machine
  code `INTERRUPTED`; the acceptance continues to require zero `tools/call` and
  interruption within five seconds.
- No public envelope, replay, permission, Trail, provider protocol, fixture, or
  test expectation changed.

## Acceptance criteria

1. The disconnect seam returns `ChildError::Interrupted` when the host flag is
   observed in the bounded window.
2. A non-interrupted process exit remains `ChildError::ProcessExited` and maps
   through the existing provider path to `unavailable`.
3. Four deterministic child-process tests cover immediate, late, absent, and
   bounded observation; all Rust checks pass.
4. Five separate J13B runs each pass `10 passed, 0 failed`; their tenth case
   asserts `interrupted`/`10`/`INTERRUPTED`, at most five seconds, and zero
   `tools/call`.
5. Only the authorised source and documentation paths change; the retained
   J16D and J16D-R2 evidence remains unchanged; J16D-R3 and J17 have not begun.

## Required verification

- `rustup run 1.89.0 cargo fmt --all`, then `cargo fmt --check`, `cargo check
  --locked`, focused `child_process`, focused `j13b`, and full `cargo test
  --locked`, all under Rust `1.89.0` with `RUSTUP_AUTO_INSTALL=0` scoped and
  restored.
- Five separately started `test-j13b-run.ps1` processes under the explicit J16
  OCaml switch, all passing.
- Packet checker, `git diff --check`, changed-path inspection, status, retained
  evidence hashes, process inspection, and temporary-root inspection.

## Forbidden changes

- No public-envelope, replay, permission, Trail, provider-protocol, fixture,
  J13B expectation, `Cargo.lock`, J16D-R3, J17, or `main` change.

## Stop conditions

- Stop on a substantive verification failure, an unauthorised changed path,
  altered retained evidence, a J16 executable remaining, or an unexpected
  temporary test root.

## Expected pre-existing changes

None.

## Commit and publication boundary

Create exactly one commit: `fix: make interrupted provider exit deterministic`;
push only `codex/j16-clean-checkout-proof`.

## Return contract

Return the bounded precedence rule, focused and full Rust evidence, all five
public J13B results, changed paths, branch topology, and final cleanliness.
