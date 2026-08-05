# Current Implementation Task

Control contract: `1`
Task: `J24K3d1 correction - planner path-safety regressions and complete verification`
Owner: `OpenCode`
Status: `BLOCKED`
Task colour: `Red`
Route: `OpenCode using DeepSeek Pro for one bounded regression-test, verification, and evidence correction; Lucy performs independent review and routine safe merge`
Base branch: `opencode/j24k3d1-validated-recovery-plan`
Base commit: `96902b715cbb8d62aad12d468a474ae03abfaaed`
Implementation branch: `opencode/j24k3d1-validated-recovery-plan`
Worker note: `docs/worker-notes/2026-08-05-j24k3d1-correction.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Accepted main: `20cd25f328568aa2726505580689d67b6219449c`
Reviewed OpenCode tip: `26ecf14b9f3d4449f6b872d5cbf04e273c426881`
Actual original implementation checkpoint: `351a2782b59d1b08c5529bd18caf8a7fa29cde6b`
Actual prior completion candidate: `b76691c5b97bd1b3a82de824535365fec4676c20`
Implementation checkpoint: `9a48be4e08d06e636cb53e21c9686ef65fbca8c8`
Correction implementation checkpoint: `9a48be4e08d06e636cb53e21c9686ef65fbca8c8`
Verification checkpoint: `WORKTREE`

## Objective

Correct only the remaining independent-review evidence gaps in J24K3d1.

The production planner is accepted as structurally correct:

```text
load authoritative intent
  -> audit global installed-root namespace
  -> idle only after successful no-intent audit
  -> observe exact transaction
  -> classify through the accepted classifier
  -> for publication-bearing dispositions, revalidate evidence then destination
  -> return a sealed read-only plan
```

Do not redesign this production sequence.

The correction must:

1. add missing direct planner-entry path-safety and root-state regressions;
2. correct the two nonexistent checkpoint SHAs in the task and worker documentation;
3. run focused Nextest with zero retries;
4. finish one full serial `just verify` with zero failures;
5. record actual checkpoints without a self-referential final-tip field.

## Relevant background and existing behaviour

Accepted main is exactly:

```text
20cd25f328568aa2726505580689d67b6219449c
```

The reviewed branch tip before this correction is:

```text
26ecf14b9f3d4449f6b872d5cbf04e273c426881
```

GitHub proves the real production commit is:

```text
351a2782b59d1b08c5529bd18caf8a7fa29cde6b
```

The completed packet instead records nonexistent SHA:

```text
351a27867078f4f37bca80bd2f481e790cdfb5cf
```

GitHub proves the prior completion-candidate commit is:

```text
b76691c5b97bd1b3a82de824535365fec4676c20
```

The completed packet instead records nonexistent SHA:

```text
b76691c529fde3ce0f09bce8c4d7ea6a4ef33407
```

The existing 25 direct tests cover the four dispositions, idle state, global untracked detection, evidence staleness, file-set/digest/size/permission drift and one full read-only snapshot. They do not contain a planner-entry symlink, junction, reparse, or explicit unsafe-root test.

The prior handoff also reports:

- focused Nextest not run;
- one `m3_lifecycle` failure;
- full verification totals with that failure excluded.

That does not satisfy the frozen acceptance criteria.

## Required behaviour

1. Preserve the accepted production planner ordering and read-only behavior.
2. Add direct destination-reparse planner tests.
3. Add direct root-state planner tests.
4. Correct documentary checkpoint identities.
5. Complete focused Nextest with zero retries.
6. Complete one genuinely green full serial verification.

### 1. Preserve the accepted production planner

Do not change `installation_recovery_plan.rs` unless a newly added production-entry regression demonstrates a real defect.

Preserve exactly:

- authoritative intent loading first;
- global installed-root audit for both `Some` and `None`;
- accepted observation and classifier only;
- no package evidence for cleanup-only dispositions;
- evidence revalidation before destination verification for both publication-bearing dispositions;
- sealed private plan fields;
- no mutation.

### 2. Add direct destination-reparse planner tests

In `installation_recovery_plan_tests.rs`, add platform-appropriate production-entry tests:

- Windows: create a complete valid destination-only recovery fixture, replace the exact intent destination with a directory junction, call `plan_installation_recovery`, and require `unsafe_store_path`.
- Unix: create the equivalent symbolic-link fixture and require `unsafe_store_path`.

The target may contain the valid copied destination bytes. The rejection must be caused by the destination path being a reparse link, not by missing or malformed evidence.

Retain all existing drift tests.

### 3. Add direct root-state planner tests

Exercise `plan_installation_recovery` itself and prove at minimum:

- an already-opened install root removed before planning returns `installation_recovery_io`;
- an already-opened record root replaced by a Windows junction or Unix symbolic link returns `unsafe_store_path`.

Use the accepted safe platform fixture pattern. Do not add unsafe representation tricks, public seams, or production test hooks.

## Relevant components

- `tethers-0.1/host-rust/src/installation_recovery_plan.rs`: accepted read-only planner entry point.
- `tethers-0.1/host-rust/src/installation_recovery_plan_tests.rs`: direct planner-entry regression fixtures.
- `tethers-0.1/host-rust/src/installed.rs`: authoritative installed-root audit and destination verification.
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`: authoritative intent store.

## Frozen decisions and invariants

- Preserve the accepted planner ordering and sealed plan invariant.
- Keep path safety fail-closed: missing roots return `installation_recovery_io`; reparse roots return `unsafe_store_path`.
- Tests must use real platform path representations and must not add production hooks or unsafe test seams.
- The correction remains test and evidence documentation only; no mutation or executor behavior is added.

## Acceptance criteria

1. Direct planner-entry destination junction or symlink regression returns `unsafe_store_path`.
2. Direct planner-entry missing opened install-root regression returns `installation_recovery_io`.
3. Direct planner-entry record-root junction or symlink regression returns `unsafe_store_path`.
4. Existing J24K3d1 tests and all named regression suites pass without excluded failures.
5. Focused Nextest passes with zero retries and records platform skips honestly.
6. Full serial `just verify` passes with zero failures and Cargo.lock retains the required hash.
7. Corrected checkpoint identities and final packet evidence are committed and pushed normally.

## Expected pre-existing changes

- The branch is synchronized to the authoritative READY correction packet commit `ab5dd86186a9df0c10434cd5552915fd5e055f1a` before implementation.
- The accepted J24K3d1 planner and its original 25 tests are already present and are not redesigned.

### 4. Correct documentary checkpoint identities

Update both:

- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-05-j24k3d1-validated-recovery-plan.md`.

Replace the nonexistent original implementation SHA with:

```text
351a2782b59d1b08c5529bd18caf8a7fa29cde6b
```

Do not preserve the prior failed verification attempt as the final verification checkpoint. It may be identified as the prior completion candidate:

```text
b76691c5b97bd1b3a82de824535365fec4676c20
```

The correction worker note must record:

- original implementation checkpoint;
- correction implementation checkpoint;
- final successful verification checkpoint.

Do not add a final remote tip field to committed documents.

### 5. Complete focused Nextest

Run the required focused command exactly:

```powershell
cargo nextest run `
  --config-file .config/nextest.toml `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-features --locked `
  -E 'test(j24k3d1)'
```

It must pass with zero retries.

Do not install software automatically. If `cargo nextest` is unavailable, stop as `BLOCKED` and report the exact command-not-found evidence. Do not mark the packet complete.

### 6. Complete one genuinely green full serial verification

Set:

```powershell
$env:PATH = "$PSHOME;$env:PATH"
$env:RUST_TEST_THREADS = "1"
```

Then run:

```powershell
just verify
```

It must finish with zero failures.

If the documented `m3_lifecycle` Windows handle-contention failure occurs:

1. record the exact failing test name and error;
2. rerun that exact test serially and require it to pass;
3. rerun full serial `just verify` once;
4. require the full rerun to pass.

Do not exclude the failed test from totals. Do not call a failed verify run complete.

## Direct test acceptance

Directly prove through `plan_installation_recovery`:

- all 25 existing J24K3d1 tests remain green;
- destination junction/symlink returns `unsafe_store_path`;
- missing already-opened install root returns `installation_recovery_io`;
- unsafe record-root junction/symlink returns `unsafe_store_path`;
- all new fixtures leave unrelated stores untouched.

Expected direct count should increase by the platform-appropriate tests. Record exact platform skips rather than pretending both platform branches ran.

## Verification procedure and checkpoints

Avoid the SHA chase.

1. Change task and correction worker-note status `READY` -> `IN_PROGRESS`.
2. Replace correction worker-note Base commit `WORKTREE` with:

```text
96902b715cbb8d62aad12d468a474ae03abfaaed
```

3. Apply only test and documentary corrections.
4. Commit the test changes and checkpoint corrections. Record that commit as the correction implementation checkpoint.
5. Update task and correction worker note to `COMPLETE`, with Verification checkpoint still `WORKTREE`.
6. Commit that completion candidate.
7. At that exact completion-candidate SHA, run every required verification command below.
8. Only after everything passes, record that tested completion-candidate SHA as Verification checkpoint in the task and correction worker note.
9. Commit and push that final evidence update.
10. Run the packet checker at the final documentation tip.
11. Return the final remote tip externally. Do not commit it into the branch.

## Required verification

Run from repository root:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1

cargo fmt `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all -- --check

cargo nextest run `
  --config-file .config/nextest.toml `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-features --locked `
  -E 'test(j24k3d1)'

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3d1 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3c4 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3c3 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3c2 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3c1 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3b `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3a `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k2 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test j24j_installation_reconciliation `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test m3_lifecycle `
  --locked

$env:PATH = "$PSHOME;$env:PATH"
$env:RUST_TEST_THREADS = "1"
just verify

Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
git diff --check
git status --short
git log --oneline --decorate -16
```

Cargo.lock must remain:

```text
D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

## Permitted files

- `tethers-0.1/host-rust/src/installation_recovery_plan_tests.rs`;
- `tethers-0.1/host-rust/src/installation_recovery_plan.rs` only if a new direct regression exposes a real defect;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-05-j24k3d1-validated-recovery-plan.md` only for checkpoint and truthful evidence correction;
- `docs/worker-notes/2026-08-05-j24k3d1-correction.md`.

## Forbidden changes

- No mutation implementation.
- No intent removal, staging deletion or record publication.
- No lock, planner, or executor wiring.
- No public API or schema changes.
- No dependency, Cargo configuration, Cargo.lock, CLI, packaging, release, enablement, operational-scope or OCaml change.
- No automatic tool installation.
- No unrelated refactor.

## Stop conditions

Stop as `BLOCKED` if:

- Nextest is unavailable;
- a new direct test requires changing an accepted lower-level contract rather than preserving it;
- full serial `just verify` still fails after the one permitted evidence-led rerun;
- Cargo.lock changes.

Do not stop for adding platform-gated fixtures, correcting nonexistent SHAs, or the first occurrence of the exact documented `m3_lifecycle` handle-contention failure when its serial rerun passes.
