# Current Implementation Task

Control contract: `1`
Task: `J24K3c4 correction - preserve unsafe installed-state paths`
Owner: `OpenCode`
Status: `READY`
Task colour: `Red`
Route: `OpenCode using DeepSeek Pro for one bounded error-mapping and regression-test correction; Lucy performs independent review and routine safe merge`
Base branch: `opencode/j24k3c4-installed-root-audit`
Base commit: `66047d7f475d9221b293cb7d178c5ed61cd0bd75`
Implementation branch: `opencode/j24k3c4-installed-root-audit`
Worker note: `docs/worker-notes/2026-08-05-j24k3c4-correction.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Accepted main: `e95e061e815d69b91b0637d08f84caaa602f1772`
Reviewed OpenCode tip: `37fe0440493986847e72be53852048f9703ace24`
Reviewed implementation checkpoint: `31c741b663e08ffd631004de7ca0d3556f5cedfe`

## Objective

Correct one narrow independent-review finding in the otherwise sound J24K3c4 global installed-root consistency auditor.

The current audit uses:

```rust
self.load_all().map_err(|_| recovery_conflict())?
```

That collapses every accepted installed-state validation failure into `installation_recovery_conflict`, including an explicit `unsafe_store_path` produced when a tracked installed destination or installed-record entry is a symlink, junction, or Windows reparse point.

The frozen J24K3c4 contract requires explicit unsafe-path refusal to remain `unsafe_store_path`.

Do not redesign the audit or add later recovery behaviour.

## Relevant background and existing behaviour

J24K3c4 is implemented and reviewed at:

```text
37fe0440493986847e72be53852048f9703ace24
```

The implementation correctly:

- validates an optional intent first;
- requires both already-opened roots to remain safe and present;
- loads the full installed-state set through `InstalledPlugRegistry::load_all()`;
- enforces canonical record identities and exact `plug-<installed_id>` destinations;
- cross-checks intent and record ownership;
- enumerates only direct final-namespace children;
- preserves explicit unsafe errors found during that later direct enumeration;
- performs no mutation.

The defect is limited to the earlier `load_all()` error translation. The existing Windows-junction and Unix-symlink tests create an untracked direct `plug-*` child, so they bypass `load_all()` and do not cover a reparse path tracked by a validated installed record.

## Required behaviour

### 1. Add one narrow installed-state error mapper

Replace the blanket `map_err` with a small private mapper used only by the recovery audit.

Required mapping:

- `unsafe_store_path` -> preserve the complete accepted error unchanged;
- genuine accepted store-access error such as `store_io` -> `installation_recovery_io` with the fixed existing recovery-owned message;
- malformed, contradictory, missing, drifted, or otherwise invalid installed state -> `installation_recovery_conflict` with the fixed existing recovery-owned message.

Do not copy lower-layer messages, paths, record strings, JSON, or operating-system diagnostics into a new error.

Do not change public `InstalledPlugRegistry::load_all()` behaviour.

### 2. Preserve all accepted J24K3c4 semantics

Do not change:

- optional-intent validation ordering;
- root guards;
- record identity checks;
- intent cross-checking;
- direct-only final namespace enumeration;
- malformed/untracked destination handling;
- accounted non-directory handling;
- non-final entry exclusions;
- read-only behaviour;
- stable messages or codes outside this correction.

### 3. Add production-entry-point regressions

Add platform-appropriate tests that create a complete valid installed record and then make that record's exact tracked final destination unsafe:

- on Windows, replace the tracked destination with a directory junction and require `unsafe_store_path`;
- on Unix, replace the tracked destination with a symbolic link and require `unsafe_store_path`.

The fixture must include the record, a valid destination tree before replacement, and correct record bytes/digests. The failure must come through `audit_installation_recovery_destinations`, not a helper.

Retain the existing untracked direct-child reparse tests. They prove the later direct-enumeration route; the new tests prove the earlier `load_all()` route.

A narrow installed-record-entry reparse test may also be added when it can be made reliable without privileges, public seams, or platform-specific flakiness, but it is not required if the tracked-destination regression proves the mapping defect directly.

## Frozen decisions and invariants

1. J24K3c4 remains crate-private and read-only.
2. A supplied intent remains the first production operation.
3. Explicit reparse state remains `unsafe_store_path` at every audit route.
4. Genuine audit/store access failure maps to `installation_recovery_io` without detail.
5. Contradictory installed state maps to `installation_recovery_conflict` without detail.
6. Public `load_all()` semantics remain untouched.
7. No dependency, Cargo configuration, Cargo.lock, schema, public API, CLI, packaging, release, enablement, operational-scope, or OCaml change.
8. Recovery classification, cleanup, record publication, intent removal, locking, planner, and executor wiring remain later work.

## Acceptance criteria

1. The blanket `load_all()` error collapse is removed.
2. `unsafe_store_path` from tracked installed-state validation survives unchanged.
3. `store_io` from installed-state loading maps to `installation_recovery_io`.
4. Other `load_all()` failures remain `installation_recovery_conflict`.
5. No lower-layer message or path escapes.
6. Existing J24K3c4 success and failure semantics remain unchanged.
7. New tracked-destination reparse tests exercise the production audit seam.
8. Existing untracked reparse tests remain green.
9. Focused Nextest passes with zero retries.
10. J24K3c3, J24K3c2, J24K3c1, J24K3b, J24K3a, J24K2, J24J, and M3 lifecycle regressions remain green.
11. Full `just verify` and the task packet checker pass.
12. Cargo.lock remains byte-identical and only permitted files change.
13. The worker note records the exact correction checkpoint, counts, verification, discoveries, risks, and final remote tip.

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
  -E 'test(j24k3c4)'

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
just verify

Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
git diff --check
git status --short
git log --oneline --decorate -12
```

Cargo.lock must remain:

```text
D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

If the documented `m3_lifecycle` Windows handle-contention failure occurs, identify the exact known failure, rerun that exact test serially, and require it to pass. Do not relabel another failure as pre-existing.

## Forbidden changes

- No edit to the frozen architecture.
- No edit to `installation_publication_intent.rs`, `installation_recovery.rs`, `installation_recovery_evidence.rs`, `installation_execution.rs`, or `m3_store.rs`.
- No public `load_all()` behaviour change.
- No intent-destination content verification beyond accepted installed-record validation.
- No recursive scan of unrelated entries.
- No staging classification or cleanup.
- No adoption, deletion, repair, rename, record creation, intent removal, lock, planner, or executor wiring.
- No public API, schema, dependency, Cargo configuration, Cargo.lock, CLI, packaging, release, enablement, operational-scope, or OCaml change.
- No unrelated refactor or broad test framework.

Permitted files:

- `tethers-0.1/host-rust/src/installed.rs` only for the recovery-audit mapper and call site;
- `tethers-0.1/host-rust/src/installation_recovery_audit_tests.rs`;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-05-j24k3c4-correction.md`.

## Stop conditions

Stop as `BLOCKED` only if preserving unsafe-path state requires changing a public API, accepted store primitive, evidence schema, dependency, Cargo.lock, or production mutation; or if full verification still fails after one evidence-led correction.

Do not stop for failed LSP, a stale local ref, adding platform-gated tracked-destination fixtures, or the documented intermittent Windows handle-contention fixture.

## Expected pre-existing changes

The branch contains the complete reviewed J24K3c4 implementation and evidence at `37fe0440493986847e72be53852048f9703ace24`, followed by the correction worker-note scaffold at the `Base commit`. No correction production code has yet been applied.
