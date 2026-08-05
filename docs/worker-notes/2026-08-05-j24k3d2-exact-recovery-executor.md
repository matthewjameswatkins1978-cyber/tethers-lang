# Worker Note

Task: `J24K3d2 - Exact installation recovery executor`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Model: `Luna`
Status: `COMPLETE`
Base commit: `ea4076085ed246a95eb2c0edab462b8c69d461fc`
Implementation checkpoint: `371136913c99a67c08eb61484d6a69e3576ea5ad`
Verification checkpoint: `c1ccb8e22c51aa292ae885b4f2ae7e61cdd64090`

## Requested outcome

Implement one crate-private recovery executor that consumes only a sealed J24K3d1 plan, rechecks the authoritative current state immediately before mutation, performs the exact accepted recovery sequence, and proves recovery returns to idle.

The package must complete only recovery. It must not create new publication intents, build a new staging directory, rename staging into a final destination, acquire the installation lock, run J24J, execute an ordinary installation action, or wire the public executor.

## Changes made

- Added crate-private `installation_recovery_execution` with exact fresh-plan equality, disposition-specific mutation ordering, dependent replans, exact intent removal, and final idle proof.
- Added exact staging cleanup and exact precomputed installed-record publication to `InstalledPlugRegistry`.
- Added sealed-plan `PartialEq`/`Eq` support and private module registrations.
- Added 20 direct `j24k3d2` production-entry tests covering successful routes, conflicts, stale evidence, resumability, path safety, exact identity preservation, unrelated roots, and non-adoption.
- Added the checker-required equivalent packet headings without changing the frozen J24K3d2 scope.

## Decisions and assumptions

- J24K3d1 remains the sole planner and classifier composition boundary.
- The executor accepts only `ValidatedInstallationRecoveryPlan`; callers cannot supply an intent, disposition, booleans, paths, or repair policy.
- A fresh J24K3d1 plan must exactly match the supplied sealed plan immediately before the first mutation.
- Staging cleanup and exact installed-record publication are narrow host-owned registry operations.
- `InstallationPublicationIntentStore::remove_if_matches` remains the only intent-removal seam.
- Failed staging cleanup or failed record publication retains the authoritative intent.
- After staging cleanup, recovery must replan to `RemoveIntentOnly` before removing the intent.
- After exact record publication, recovery must replan to `VerifyCompletedPublicationThenRemoveIntent` before removing the intent.
- The final postcondition is a fresh idle recovery plan after the global installed-root audit.
- Lock integration remains a later package. This crate-private seam is not wired into any public entry point in J24K3d2.
- Workers record implementation and verification checkpoints only. Do not commit a final remote tip field.

## Evidence

- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` - PASS at completion candidate `c1ccb8e22c51aa292ae885b4f2ae7e61cdd64090`.
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` - PASS.
- `cargo nextest run --config-file .config/nextest.toml --manifest-path tethers-0.1/host-rust/Cargo.toml --all-features --locked -E 'test(j24k3d2)'` - PASS, 20 passed, 0 failed, 0 retries.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --lib j24k3d2 --locked` - PASS, 20 passed.
- J24K3d1 regression - PASS, 28 passed, 2 platform-appropriate ignored tests.
- J24K3c4 regression - PASS, 24 passed.
- J24K3c3 regression - PASS, 44 passed.
- J24K3c2 regression - PASS, 21 passed.
- J24K3c1 regression - PASS, 20 passed.
- J24K3b regression - PASS, 16 passed.
- J24K3a regression - PASS, 25 passed.
- J24K2 regression - PASS, 26 passed.
- J24J regression - PASS, 24 passed.
- M3 lifecycle regression - PASS, 13 passed. The first parallel invocation had one Windows teardown contention failure; the exact test rerun serially passed, and the required full serial verification also passed.
- `$env:PATH = "$HOME\.cargo\bin;$PSHOME;$env:PATH"; $env:RUST_TEST_THREADS = "1"; just verify` - PASS, 1164 library tests passed, 2 ignored, and all integration suites passed.
- `Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256` - PASS, `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`.
- `git diff --check` - PASS.
- Final working-tree status — clean after the implementation and evidence commits.

## Discoveries

- The synchronized packet initially omitted headings required by the repository checker; equivalent headings and acceptance mapping were added within the permitted packet file.
- Windows junction fixtures emitted `mklink` diagnostic lines while all relevant path-safety tests passed.

## Remaining risks

- The later lock-integration package must ensure planning and recovery execution occur inside one held installation lock lifetime.
- The later publication package must create the durable intent and staging/final destination transaction that this executor recovers.

## Smallest next action

Implement only the packet, verify the complete recovery matrix, push the branch, and return it for Lucy’s independent review.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_recovery_plan.rs`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installed.rs`
