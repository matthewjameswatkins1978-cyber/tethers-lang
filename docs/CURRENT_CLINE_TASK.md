# Current Implementation Task

Control contract: `1`
Task: `J24L1 - Bounded installation driver`
Owner: `DeepSeek Pro`
Model: `DeepSeek Pro`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode using DeepSeek Pro for one bounded Rust control-flow package; Lucy performs independent review and any later merge`
Base branch: `main`
Base commit: `190e834b8afeca060adb3b07c7a18554497aaf31`
Implementation branch: `opencode/j24l1-bounded-installation-driver`
Worker note: `docs/worker-notes/2026-08-06-j24l1-bounded-installation-driver.md`
Implementation blueprint: `docs/architecture/J24L_THIN_PUBLIC_PLUG_INSTALL_CLI.md`
Rust toolchain: `1.97.1`

## Objective

Implement only the crate-private bounded control-flow driver that repeatedly
invokes the accepted J24K single-step executor until one of these exact stopping
conditions occurs:

1. installation is complete;
2. conformance was recorded without advancing;
3. J24K returns an error;
4. four executor calls have occurred without reaching a legitimate stopping
   condition.

This package contains no CLI parsing, output formatting, store construction,
path layout, request-file loading, package staging, action-specific mutation,
retry, or recovery implementation.

## Relevant background and existing behaviour

J24K is complete. Its accepted public primitive is:

```rust
pub fn execute_next_installation_action(
    request: &InstallationRequest,
    context: &InstallationExecutionContext<'_>,
    options: &InstallationExecutionOptions<'_>,
) -> Result<InstallationStepResult>;
```

Each call acquires its own installation lock, creates its own current
authoritative plan after locking, performs zero or one durable ordinary
mutation, creates a fresh after-plan, and releases the lock on return.

The accepted progression is:

```text
CreateExactCandidateTrust
  -> RunSupervisedConformance
  -> CreateInstallationApproval
  -> PublishDisabledInstallation
  -> Complete
```

J24L must call that primitive at most four times. The driver must never acquire
or retain a lock itself. Each call to J24K must acquire a fresh lock and
produce a fresh authoritative plan.

## Required behaviour

1. Create `installation_driver.rs` with a crate-private driver entry point
   `drive_installation` and a private closure-based helper `drive_with` for
   testability.
2. Register the module and its `#[cfg(test)]` test module in `lib.rs`.
3. Return `AlreadyComplete` immediately after one call, preserving the exact
   step.
4. Return `Complete` when an `Advanced` result has `after.action == Complete`,
   without making a fifth confirmation call.
5. Drive a fresh four-action sequence (`CreateExactCandidateTrust` through
   `PublishDisabledInstallation -> Complete`) in exactly four calls.
6. Return `ConformanceRecordedWithoutAdvance` immediately when J24K returns
   that outcome, preserving the exact evidence ID and disposition, without
   retry.
7. Propagate J24K `M3Error` code and message exactly, without another call.
8. After exactly four returned non-completing `Advanced` results, return
   `installation_iteration_limit` with the exact message `installation did not
   complete within four executor calls`, without a fifth call.
9. Preserve returned steps exactly in order without rewriting or normalising.

## Relevant components

- `tethers-0.1/host-rust/src/installation_driver.rs` (new)
- `tethers-0.1/host-rust/src/installation_driver_tests.rs` (new)
- `tethers-0.1/host-rust/src/lib.rs`
- `docs/architecture/J24L_THIN_PUBLIC_PLUG_INSTALL_CLI.md` (new)
- `docs/CURRENT_CLINE_TASK.md` (replacement)
- `docs/worker-notes/2026-08-06-j24l1-bounded-installation-driver.md` (new)

## Frozen decisions and invariants

- Maximum four J24K calls.
- Fresh lock and fresh plan on every J24K call.
- Driver never acquires or retains a lock.
- No fifth confirmation call.
- No conformance retry.
- Exact J24K error propagation without wrapping.
- `installation_iteration_limit` code and message are exact and frozen.
- Driver does not validate plans; J24K owns all validation.
- Public API outside the crate is forbidden in J24L1.
- No serialization derives in J24L1.
- The `MAX_INSTALLATION_EXECUTOR_CALLS` constant is private.
- `drive_with` closure-based helper is the sole test seam.
- J24L2 responsibilities (CLI, stores, paths) are deferred and must not be
  invented here.

## Acceptance criteria

1. `drive_installation` exists as a `pub(crate)` function in
   `installation_driver.rs`. Evidence: compilation and direct unit test.
2. `lib.rs` registers `installation_driver` as a private module and
   `installation_driver_tests` under `#[cfg(test)]`. Evidence: compilation.
3. `j24l1_already_complete_stops_after_one_call` proves one call only, one
   step retained, stop is `Complete`, no second call. Evidence: test pass.
4. `j24l1_advanced_to_complete_stops_without_confirmation_call` proves an
   `Advanced` result with `after.action == Complete` stops after one call
   without a confirmation call. Evidence: test pass.
5. `j24l1_fresh_sequence_completes_in_exactly_four_calls` proves exactly four
   calls, stop is `Complete`, four steps retained in order, no fifth call.
   Evidence: test pass.
6. `j24l1_conformance_without_advance_stops_immediately` proves
   `ConformanceRecordedWithoutAdvance` stops after one call with exact
   evidence ID and disposition, no retry. Evidence: test pass.
7. `j24l1_executor_error_propagates_without_another_call` proves exact
   `M3Error` code and message returned, call count is one.
   Evidence: test pass.
8. `j24l1_four_noncomplete_advances_hit_exact_iteration_limit` proves exactly
   four calls, no fifth, exact code and message. Evidence: test pass.
9. `j24l1_preserves_returned_steps_without_rewriting` proves steps are
   returned exactly as supplied, in order. Evidence: test pass.

## Required verification

Direct tests:

```powershell
cargo test --lib -p tethers-reference-host j24l1_ --no-fail-fast --locked
```

J24K regressions:

```powershell
cargo test --lib -p tethers-reference-host j24k3f --no-fail-fast --locked
cargo test --lib -p tethers-reference-host j24k2 --no-fail-fast --locked
```

Planner regression:

```powershell
cargo test --test j24j_installation_reconciliation --locked
```

Formatting:

```powershell
cargo fmt --all -- --check
```

Clippy:

```powershell
cargo clippy --all-targets --all-features --locked
```

Full serial verification:

```powershell
$env:RUST_TEST_THREADS = "1"
just verify
Remove-Item Env:RUST_TEST_THREADS
```

Packet checker:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
```

Final hygiene:

```powershell
git diff --check
git status --short
git diff --stat main...HEAD
git diff main...HEAD
```

## Forbidden changes

Do not:

- add `plug install` to `cli.rs`;
- alter `application.rs`;
- alter `plug_command.rs`;
- load an installation request file;
- choose host-data-root subdirectory names;
- create or open candidates, trust, conformance, approval, installed or intent
  stores;
- construct an `InstallationExecutionContext`;
- change `InstallationExecutionContext`;
- change `InstallationExecutionOptions`;
- change `InstallationStepResult`;
- change `InstallationStepOutcome`;
- change `InstallationPlan`;
- change J24J or J24K;
- acquire a lock in the driver;
- hold one lock across multiple executor calls;
- call the planner outside J24K;
- execute any action-specific mutation;
- retry conformance;
- make a fifth confirmation call;
- add a general loop, configurable limit or caller-supplied iteration count;
- add public API outside the crate;
- add serialization or a new schema;
- add dependencies;
- change `Cargo.toml`;
- change `Cargo.lock`;
- change the Rust toolchain;
- alter OCaml, Tethers language semantics, package formats or protocols;
- merge the branch.

## Stop conditions

Stop and report exact evidence before further editing if:

- main does not equal the frozen base;
- the worktree has unexplained changes;
- the accepted J24K API differs from the packet;
- implementation appears to require changing J24K;
- implementation appears to require CLI or store-layout decisions;
- a fifth call seems necessary to prove completion;
- a test requires a global or timing-dependent hook;
- `Cargo.lock` changes;
- packet checker fails because the normative scope is contradictory;
- two materially similar implementation attempts fail;
- verification exposes an unrelated failure that prevents trustworthy
  completion.

## Expected pre-existing changes

None.

## Checkpoint procedure

1. Require the READY packet checker passes.
2. Change packet and worker-note status to `IN_PROGRESS`.
3. Implement production code and direct tests.
4. Commit implementation and capture one full implementation SHA.
5. Record that SHA in both documents.
6. Run all required verification at that exact checkpoint.
7. Complete the worker note honestly, including `## Changes made`.
8. Change both statuses to `COMPLETE`.
9. Commit verification documentation only.
10. Capture and record the verification checkpoint through a final
    documentation-only commit if required.
11. Require packet checker, fmt, diff check and clean status.
12. Push the branch and report exact SHAs and evidence.
13. Do not merge.
