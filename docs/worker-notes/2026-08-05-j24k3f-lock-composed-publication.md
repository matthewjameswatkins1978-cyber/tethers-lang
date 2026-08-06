# Worker Note

Task: `J24K3f - Lock-composed disabled installation publication`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Model: `DeepSeek Pro`
Status: `BLOCKED`
Base commit: `13cae687dc59c0dae74363b24d0ab57547702c53`
Implementation checkpoint: `WORKTREE`
Verification checkpoint: `WORKTREE`

## Requested outcome

Compose accepted J24K3e1 preparation and J24K3e2 exact mutation into the existing locked single-step executor for `PublishDisabledInstallation`, then require the fresh J24J after-plan to be `Complete`.

## Changes made

Original implementation (checkpoint `eaa0e125744616af08a1a1c6dd57e16cccc3b41f`):

- `tethers-0.1/host-rust/src/installation_execution.rs`: added `executor_state_root: &'a Path` to `InstallationExecutionContext` (frozen blueprint field). Replaced `handle_deferred_publication` with `handle_publication` that opens `InstallationPublicationIntentStore` from `context.executor_state_root`, constructs `InstallationRecoveryPlanningContext`, calls J24K3e1 preparation, J24K3e2 mutation, replans with J24J, validates the `PublishDisabledInstallation -> Complete` transition, and returns `Advanced { executed: PublishDisabledInstallation }`.
- `tethers-0.1/host-rust/src/installation_execution_tests.rs`: added `PublicationReadyFixture` helper and 8 direct tests. Updated existing test fixture with `executor_state_root`.
- `tethers-0.1/host-rust/tests/j24k2_locked_single_step_executor.rs`: added `executor_state_root` to `make_context` function and all 7 call sites. Updated `j24k2_full_passed_conformance_and_approval_chain` to expect successful publication.

Second review correction (rejected tip `efcd5b545d01ce0233fb992c6eadd1e25a40caea`; the first review correction's two tests were invalid and are removed):

- `tethers-0.1/host-rust/src/installation_execution_tests.rs`: removed the invalid `load_evidence` and `write_intent` helpers and the two rejected tests (`j24k3f_pre_intent_failure_returns_error_no_creation_lock_released`, `j24k3f_post_intent_failure_state_recoverable_lock_released`). Removed the now-unused imports `InstallationPublicationIntent` and `execute_validated_installation_recovery`. Added one genuine pre-intent test:
  - `j24k3f_global_untracked_destination_blocks_publication_pre_intent_lock_released`: begins with an EMPTY intent store (no pre-created intent), places a foreign untracked `plug-<uuid>` destination directory with no owning record (the existing accepted filesystem obstruction already used by `j24k3e1_global_untracked_final_destination_blocks_preparation`), invokes the public locked `execute_next_installation_action`. Because installed records remain empty, `plan_installation` still returns `PublishDisabledInstallation` so the executor reaches the accepted J24K3e1 preparation boundary through the publication route; preparation's idle-recovery global destination audit fails with `installation_destination_untracked` BEFORE any durable intent is created. The test proves: intent store stays empty, install tree unchanged (no destination created), no installed record exists, the records root unchanged, and the installation lock was released (a second public invocation returns `installation_destination_untracked`, NOT `installation_busy`, which also proves no second mutation).

Post-intent failure-boundary test NOT added: see `Discoveries` and `Remaining risks` for the blocker.

No lock acquisition change, no new public API, no loop/retry, no J24L, no Cargo.lock change, no production fault injection.

## Decisions and assumptions

- `executor_state_root` is supplied explicitly by the caller; no fallback derivation from other roots.
- `InstallationPublicationIntentStore` is opened from exactly `context.executor_state_root`.
- J24K3e1 receives the exact locked `before` plan produced by the executor's initial `plan_installation` call.
- J24K3e2 consumes the sealed prepared value inside the same lock, with no identity regeneration.
- Replan and transition validation use the existing `replan` and `validate_transition` helpers.
- The `j24k2_full_passed_conformance_and_approval_chain` test was a legitimate expectation change: it previously tested that publication returns `installation_publication_deferred`; now publication succeeds and the test exercises the full 5-call chain through Complete.
- The pre-intent boundary is exercised through a foreign untracked destination directory, not through a pre-created intent. `plan_installation` (J24J) does not audit the install root for untracked destinations, so the before-plan still reaches `PublishDisabledInstallation`; preparation's `require_idle_recovery` -> `plan_installation_recovery` performs that global destination audit and raises `installation_destination_untracked` before any durable write. This is identical to the accepted `j24k3e1_global_untracked_final_destination_blocks_preparation` seam, driven through the public locked executor.

## Evidence

Direct tests at this correction (worktree):

- `cargo test --lib -p tethers-reference-host j24k3f --no-fail-fast --locked` — PASS, 9 passed (8 original + 1 new genuine pre-intent), 0 failed.
- `cargo test --lib -p tethers-reference-host j24k3f_global_untracked_destination_blocks_publication_pre_intent_lock_released --no-fail-fast --locked` — PASS, 1 passed.

Named regressions (each `cargo test -p tethers-reference-host <filter> --no-fail-fast --locked` — all PASS):

- `j24k3e2` — 26 passed, 0 failed
- `j24k3e1` — 30 passed, 0 failed
- `j24k3d2` — 20 passed, 0 failed
- `j24k2` — 26 lib + 9 integration = 35 passed, 0 failed
- `j24j` — 0 matched by name filter (descriptive names); `cargo test --test j24j_installation_reconciliation` — 24 passed, 0 failed
- installed-state regression: `cargo test --lib -p tethers-reference-host installed` — 20 passed, 0 failed
- j24k3 + recovery regression: `cargo test --lib -p tethers-reference-host j24k3` — 263 passed, 0 failed, 2 ignored

Full serial verification with `RUST_TEST_THREADS=1`:

- `just verify` — PASS. lib 1229 passed, 0 failed, 2 ignored; all integration suites 0 failed (`exit=0`). (Net lib count is 1229 + 2 ignored = 1231; the prior rejected tip reported "1230 lib passed, 2 ignored" with 10 j24k3f tests; this correction has 9 j24k3f tests, hence one fewer running test.)

Final gates:

- `cargo fmt --all -- --check` — PASS.
- `cargo clippy --all-targets --all-features --locked` — PASS (only preexisting warnings; no new warning introduced by this change; `empty_plan`/`plan_with` dead-helper warnings are preexisting and untouched).
- `git diff --check` — PASS (only the preexisting LF->CRLF warning, intentionally not changed per the Git line-ending guide).
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — PASS (control-v1/BLOCKED).
- Cargo.lock unchanged (empty `git diff origin/main...HEAD -- Cargo.lock`).

## Discoveries

- The durable publication intent write occurs in `execute_prepared_disabled_installation_publication` at `installation_publication_mutation.rs:136` AFTER the pre-intent checks (stale-evidence revalidation line 106; recovery-idle check line 108; global destination audit line 115; duplicate-release check line 119; intent+record validation line 132). The post-intent failure points are staging build (line 144), staging path-safety reparse check, rename, record publication, and final recovery classification.
- The publication `transaction_id` equals the installed record's `installed_id`, which `prepare_disabled_installation_record` generates as `Uuid::new_v4()` (`installed.rs:852`) — a fresh random UUID per preparation call. The staging path is `.staging-{transaction_id}` and the destination path is `plug-{transaction_id}`; both depend on this freshly-generated, unobservable-beforehand identity.
- The two existing accepted post-intent obstructions both require the exact staging path and therefore the freshly-generated `transaction_id`:
  - `j24k3e2_existing_staging_is_not_overwritten_or_adopted` (`installation_recovery_io`, intent retained) pre-creates `.staging-{txid}` using a prepared value computed before obstruction.
  - `j24k3e2_reparse_staging_path_fails_closed_with_unsafe_store_path` (`unsafe_store_path`, intent retained) places a junction at `.staging-{txid}`.
  The public `execute_next_installation_action` performs preparation and mutation inside ONE locked call (`handle_publication`), and exposes no prepared value, `transaction_id`, or staging path to the caller beforehand. Thus neither obstruction can be pre-positioned through the public entry point.
- A foreign untracked `plug-<uuid>` destination directory with no owning record makes `plan_installation` (J24J) still return `PublishDisabledInstallation` (it reads installed records, not the install-root directory audit) while preparation's `require_idle_recovery` global audit raises `installation_destination_untracked` pre-intent. This is the genuine pre-intent boundary used by the new test, and is the same obstruction already accepted in `j24k3e1_global_untracked_final_destination_blocks_preparation`.

## Remaining risks

- BLOCKER on the post-intent failure-boundary test (packet acceptance criteria #11 and the reviewer's requirement B). Through the public locked executor no EXISTING test-only mechanism produces a durable-intent-then-fail boundary. Producing one would require at least one of the forbidden options:
  - production fault injection;
  - a new public API or a new `InstallationExecutionContext` field exposing the prepared value, `transaction_id`, or staging path;
  - publication or recovery redesign (e.g., splitting preparation from mutation across the public seam);
  - a platform-fragile permission trick (e.g., making `install_root` read-only so staging-dir creation fails regardless of `transaction_id`, which depends on NTFS read-only-directory semantics and is not an existing accepted obstruction);
  - Cargo.lock or dependency changes.
- Per the reviewer's instruction B, the post-intent test was NOT fabricated by renaming the seeded-recovery coverage. The two rejected tests were removed; only the genuine pre-intent test was added.
- Preexisting compiler/clippy warnings remain; none introduced by this change.
- J24L multi-step loop is the next composition layer and is out of scope here.

## Smallest next action

Resolve the B blocker for Lucy: is a Windows-NTFS read-only-directory obstruction of `install_root` (causing staging creation to fail with `installation_recovery_io` after the durable intent write, then restored for recovery) an acceptable existing-style test mechanism, or is it a forbidden platform-fragile permission trick? If forbidden, this packet requires a design decision on how to expose a post-intent failure seam for the public locked executor before the post-intent test can be written — e.g., a frozen test-only seam decision or an authorised minimal change to the publication boundary. The post-intent test cannot be added without that decision. Do not merge.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_execution.rs`
- `tethers-0.1/host-rust/src/installation_execution_tests.rs`
- `tethers-0.1/host-rust/tests/j24k2_locked_single_step_executor.rs`
- `tethers-0.1/host-rust/src/installation_publication_preparation.rs`
- `tethers-0.1/host-rust/src/installation_publication_mutation.rs`
- `tethers-0.1/host-rust/src/installation_publication_mutation_tests.rs`
- `tethers-0.1/host-rust/src/installation_publication_preparation_tests.rs`