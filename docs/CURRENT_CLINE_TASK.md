# Current Implementation Task

Control contract: `1`
Task: `F2 - Operational correctness defects`
Owner: `OpenCode`
Model: `OpenCode`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `OpenCode implements both subpackages; Lucy performs one bounded review`
Worker note: `docs/worker-notes/2026-08-07-f2-operational-correctness.md`
Base branch: `main`
Base commit: `f295daa288f4d3dc48181888d6655df798675033`
Implementation branch: `foundation/f2-operational-correctness`
Parent branch: `main`
Parent tip: `f295daa288f4d3dc48181888d6655df798675033`
Preparation checkpoint: (will be captured after first commit)
OCaml switch path: `N/A`
Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory
Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1`

## Objective

Repair two demonstrated operational-correctness defects from the F1 baseline:
truthful live stderr capture in `child_process.rs` (F2a) and nondeterministic
M3 handle allow-list test behaviour (F2b). Preserve all public CLI, JSON,
exit-code, and protocol contracts. The work is one review gate with two serial
subpackages.

## Relevant background and existing behaviour

F1 baseline at `f295daa288f4d3dc48181888d6655df798675033` recorded:

- **Live-stderr candidate (debt ledger, F2 candidate):** The stderr capture
  thread in `child_process.rs:410-431` accumulates bytes in a local `buf` and
  only copies to the shared `Arc<Mutex<Vec<u8>>>` after the reader loop exits
  (EOF or error). `stderr_tail()` returns an empty `String` while the child is
  alive and producing stderr. The `max_line_bytes` field on `SupervisedChild`
  (line 214) is declared but never read.

- **D1 — Flaky M3 handle allow-list test:** `m3_windows_handle_allow_list_excludes_unrelated_inheritable_handle`
  at `tests/m3_lifecycle.rs:840` failed on first cold `cargo test` run,
  passed on second and subsequent runs. Assertion: `left: Failed, right: Passed`.
  The test creates an inheritable event handle, passes its raw value to the
  supervised child through the candidate fixture arguments, and expects
  `run_host_conformance` to detect the handle is not accessible in the child
  (via `unrelated_inheritable_handle_accessible` in `m3_fixture_provider.rs:14-33`).

The governing F1 evidence includes: `DEBT_LEDGER.md`, `BASELINE_TRANSCRIPT.md`,
`TEST_INVENTORY.md`, `WARNING_INVENTORY.md`, and `docs/worker-notes/2026-08-06-f1-baseline.md`.

## Required behaviour

### F2a: Truthful live stderr and child cleanup

**Reproduce before repairing.** Add a direct regression test at the private
`child_process.rs` boundary using a real supervised child that performs this
deterministic sequence: write a fixed ASCII marker to stderr, flush stderr,
write a fixed ready line to stdout, flush stdout, and remain alive without
closing stderr.

After the host receives the stdout ready line, prove:

- the stderr marker becomes visible through `stderr_tail()` while the child
  is still alive;
- visibility occurs within a bounded observation deadline;
- the test does not depend on the child exiting;
- the test fails against the accepted main for the named reason.

The observation deadline may poll the real concurrent state. Do not disguise the
defect with an arbitrary sleep.

Add direct tests proving:

1. **Live visibility:** bytes already observed by the reader are visible before EOF.
2. **Bounded storage:** with a deliberately small configured tail, storage never
   exceeds the byte limit and contains the exact newest bytes.
3. **Timeout truth:** a stdout protocol timeout remains classified as timeout
   while stderr emitted before the timeout remains available.
4. **Exit truth:** actual child exit remains distinguishable from timeout and
   unexpected stdout disconnection.
5. **Windows cleanup:** shutdown terminates and reaps the child and joins both
   reader threads without leaving the fixture process alive.

**Repair constraints.** The smallest acceptable repair should:

- update the shared stderr tail incrementally after each successful read;
- retain only the configured newest bytes;
- bound memory continuously, not merely when the child exits;
- retain raw bytes internally and perform lossy UTF-8 conversion only when
  producing diagnostic text;
- preserve the exact existing public CLI, JSON, exit-code and protocol contracts;
- preserve Job Object and handle-containment behaviour;
- avoid a polling worker in production;
- avoid a new dependency;
- avoid widening production visibility for tests.

Do not remove `SupervisedChild.max_line_bytes` merely to silence a warning.
Alter or remove it only if direct F2 evidence proves that doing so is part of
the correctness repair.

Do not touch unrelated dead code.

Where cleanup operations can fail, diagnostics must not falsely label kill, wait
or reader-thread join failures as timeout, EOF or successful cleanup. Use the
smallest internal representation that preserves truthful classification without
changing public wire formats.

### F2b: M3 handle allow-list nondeterminism

The confirmed defect is nondeterministic test behaviour. The production root
cause is not yet established.

Characterise the existing exact test:

`m3_windows_handle_allow_list_excludes_unrelated_inheritable_handle`

A bounded pre-change characterisation may run the exact test serially up to 20
times. Record every result. Do not run indefinitely.

Determine which statement is true:

(A) the supervised child sometimes inherits the unrelated event handle;
(B) the test passes a numeric handle value that can alias a different permitted
    child handle;
(C) fixture startup or teardown creates another evidenced race;
(D) another directly demonstrated cause.

The final regression must prove the event-handle exclusion property itself, not
merely that a numeric handle value is invalid.

Permitted outcomes:

- If production leaks the event handle, make the smallest production fix and add
  a deterministic regression.
- If the test is producing a false positive through handle-value aliasing or
  another fixture flaw, correct the test so it inspects the intended event object
  directly.
- If neither cause can be proven, leave F2 `BLOCKED`. Do not add retries, sleeps,
  ignored status or "run until green" behaviour.

The final mandatory run gets one attempt. A failure blocks `COMPLETE`.

## Relevant components

- `tethers-0.1/host-rust/src/child_process.rs` (live-stderr repair, F2a tests)
- `tethers-0.1/host-rust/src/bin/m3_fixture_provider.rs` (handle allow-list fixture, read-only except for potential test-only fix)
- `tethers-0.1/host-rust/tests/m3_lifecycle.rs` (M3 handle allow-list test, characterisation and potential repair)
- `tethers-0.1/host-rust/src/conformance.rs` (handle-allow-list assertion at line 484-501)
- `docs/foundation-pass/DEBT_LEDGER.md` (evidence baseline)
- `docs/worker-notes/2026-08-07-f2-operational-correctness.md` (new)
- `docs/CURRENT_GOAL.md` (update)
- `docs/CURRENT_CLINE_TASK.md` (this packet)

## Frozen decisions and invariants

- The accepted main is `f295daa288f4d3dc48181888d6655df798675033`. If live
  `origin/main` differs, record the direct Git evidence and stop.
- Test access never justifies widening production visibility or adding a public
  test seam. Stderr and child-process tests belong at the private module boundary
  (`#[cfg(test)] mod tests` in `child_process.rs`).
- The packet is authoritative. Do not weaken it to fit findings; stop on a
  genuine contradiction.
- Preserve external JSON, CLI output, exit codes, Trail shape, replay digests,
  and recovery semantics.
- Every mandatory verification command must have one PASS, FAIL, or NOT RUN
  result. Any mandatory NOT RUN blocks COMPLETE.
- After the final code or test change, run the complete required matrix
  serially before claiming COMPLETE.
- Do not modify F1 literal fixtures. They are compatibility evidence.
- Do not squash away the failing-regression checkpoint for F2a.

## Acceptance criteria

1. **F2a regression test** runs against accepted main and fails because stderr
   is not visible before child exit. Evidence: test output captured with the
   failing-regression checkpoint SHA.
2. **F2a repair** makes the same regression test pass, and all five named
   sub-tests (live visibility, bounded storage, timeout truth, exit truth,
   Windows cleanup) pass independently. Evidence: full test output.
3. **F2a repair** preserves all public contracts: CLI, JSON, exit codes, Trail,
   replay digests. Evidence: fixture diff is empty (`git diff --exit-code origin/main...HEAD -- docs/foundation-pass/fixtures`).
4. **F2b characterisation** records exact serial test results for the handle
   allow-list test (up to 20 runs). Evidence: worker note.
5. **F2b root cause** is established from direct evidence among the four
   permitted categories. Evidence: worker note with supporting code references
   or test output.
6. **F2b repair** (production or test) makes the handle-allow-list property
   deterministically testable. The final mandatory run passes. Evidence: test
   output and worker note.
7. **Complete branch diff** changes only production/test code in
   `child_process.rs` (+ tests), optionally `conformance.rs`,
   `m3_fixture_provider.rs`, `m3_lifecycle.rs`, and the packet/worker-note/goal
   documentation. Evidence: complete branch diff.
8. **Final matrix** runs serially after the last edit and all mandatory commands
   are PASS. Evidence: worker note verification table.

## Required verification

Run the following serially after the final code change. Record each result as
PASS, FAIL, or NOT RUN; a mandatory NOT RUN blocks COMPLETE.

```powershell
git fetch origin --prune
git rev-parse origin/main
git rev-parse HEAD
git status --short --branch
rustup show
cargo --version
cargo fmt --all -- --check
cargo check --all-targets --all-features --locked
cargo test --all-targets --all-features --locked
cargo clippy --all-targets --all-features --locked -- -W clippy::all
just verify
just verify-agent
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
git diff --exit-code origin/main...HEAD -- docs/foundation-pass/fixtures
git diff --check origin/main...HEAD
git diff --name-only origin/main...HEAD
git status --short --branch
```

Also run the focused F2a tests and the exact M3 handle test explicitly before
the complete matrix. Record their exact commands in the worker note.

## Forbidden changes

Do not perform:

- broad dead-code removal;
- general Clippy cleanup;
- `application.rs` extraction;
- test relocation or consolidation;
- OCaml interface work;
- persistence or directory-durability work;
- Trail or replay redesign;
- outcome/protocol redesign;
- new capabilities or CLI commands;
- dependency upgrades;
- F3 or later Foundation work.

Do not modify F1 literal fixtures. They are compatibility evidence.

Do not remove `SupervisedChild.max_line_bytes` merely to silence a warning.
Alter or remove it only if direct F2 evidence proves that doing so is part of
the correctness repair.

Do not add retries, sleeps, ignored status or "run until green" behaviour for
the M3 handle test.

## Checkpoints

Create and push:

1. F2 packet checkpoint.
2. F2a regression checkpoint showing the reproduced failure.
3. F2a repair checkpoint.
4. F2b characterisation and repair checkpoint, only if directly proven.
5. Final worker-note documentation checkpoint.

Do not squash away the failing-regression checkpoint.

## Stop conditions

Stop and report direct evidence if `origin/main` differs from
`f295daa288f4d3dc48181888d6655df798675033`; the worktree/branch/base is
unexpected; the tree is dirty before F2 edits; a required claim needs a
wider-scope change than permitted; a fixture must be altered; the M3 handle
cause cannot be proven; a required command fails; or two materially similar
attempts fail. Return one smallest unresolved question. Do not use a packet
edit to bypass a stop condition.

## Expected pre-existing changes

None.
