# Current Implementation Task

Control contract: `1`
Task: `J24K3c1 correction - preserve unsafe root-chain refusal`
Owner: `OpenCode`
Status: `READY`
Task colour: `Red`
Route: `OpenCode using DeepSeek Pro V4 for one bounded security-sensitive Rust filesystem correction; Lucy performs independent review and routine safe merge`
Base branch: `opencode/j24k3c1-recovery-observer`
Base commit: `77438622e431e68bb3c57c1b89fd23abbdf68e34`
Implementation branch: `opencode/j24k3c1-recovery-observer`
Worker note: `docs/worker-notes/2026-08-05-j24k3c1-recovery-observer.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Accepted main: `753724c45500a03f876ca9008f7835d2147e2ea8`

## Objective

Correct one error-mapping defect in the otherwise complete J24K3c1 read-only recovery observer.

The reviewed implementation at:

```text
4bf11118369d5dc1d7ae50d4b1b86be969b96db9
```

preserves `unsafe_store_path` when an exact staging, destination, or record child is a symlink, junction, or reparse point. However, its install-root and record-root `verify_chain` calls currently map every error to `installation_recovery_io`.

An explicit unsafe-path refusal discovered while rechecking either accepted root chain must remain `unsafe_store_path`. Genuine metadata, access, or store-I/O failures must continue to map to the stable `installation_recovery_io` contract.

Do not otherwise change J24K3c1.

## Relevant background and existing behaviour

Accepted main remains:

```text
753724c45500a03f876ca9008f7835d2147e2ea8
```

The branch already contains the complete J24K3c1 observer, snapshot bridge, sixteen direct tests, passing full verification, and no mutation or public API.

The exact-child helpers already use the required pattern:

- preserve `unsafe_store_path` from `reject_reparse`;
- map other observation failures to `installation_recovery_io`;
- classify malformed or non-ordinary exact entries as `installation_recovery_conflict`.

Only the two root-chain checks are inconsistent with that rule.

## Required behaviour

1. Preserve explicit unsafe-path refusal from both root-chain checks.

Replace the broad error erasure around:

```rust
verify_chain(install_root)
verify_chain(record_root)
```

with a narrow mapping that returns the original error only when `error.code == "unsafe_store_path"`. Every other root-chain verification error must return the existing stable `installation_recovery_io` error and must not expose an OS error or path.

2. Preserve every accepted J24K3c1 semantic and boundary.

Intent validation remains first. The observer still inspects only the exact intent-derived staging, destination, and record paths. Snapshot shape, classifier bridge, strict JSON decoding, exact-child handling, absence semantics, and all error messages remain unchanged. Do not add mutation, broad accessors, caller-supplied paths, destination verification, evidence revalidation, global audit, cleanup, publication, lock integration, or executor wiring.

3. Add direct production-seam root-chain regression tests.

Using accepted repository platform-fixture conventions, prove separately that:

- after an already-opened install root is replaced at its original path by a symlink, Windows junction, or equivalent reparse fixture, `observe_installation_recovery` returns `unsafe_store_path`;
- after an already-opened record root is replaced at its original path by the same class of unsafe fixture, `observe_installation_recovery` returns `unsafe_store_path`.

The tests must exercise the production observer, not a source-string check or only a private mapping helper. Retain all sixteen existing `j24k3c1` tests.

4. Complete the original verification packet and record exact evidence.

Run the packet checker, formatting, focused J24K3c1 unit and Nextest suites, all listed regressions, full `just verify`, Cargo.lock hash, `git diff --check`, status, and recent log. Update the worker note with the exact implementation checkpoint, final remote tip, commands, counts, fixture behaviour, discoveries, and remaining risks.

## Relevant components

- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_recovery_observation_tests.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-05-j24k3c1-recovery-observer.md`
- `verify_chain`
- `reject_reparse`
- `InstalledPlugRegistry::observe_installation_recovery`

`m3_store.rs` and `installation_recovery.rs` are accepted references and are not permitted edit targets.

## Frozen decisions and invariants

- The observer remains crate-private and read-only.
- Intent validation remains the first operation.
- Explicit `unsafe_store_path` remains explicit at both roots and exact children.
- Other root-chain observation failures map to `installation_recovery_io` with the existing stable message.
- Exact-child malformed or non-ordinary state remains `installation_recovery_conflict`.
- Absence remains exact-path `NotFound` only.
- The snapshot and classifier bridge do not change.
- Existing public installation behaviour and `InstalledPlugRegistry::load_all()` do not change.
- No public API, dependency, Cargo configuration, Cargo.lock, CLI, prompt, output, enablement, operational-scope, packaging, release, or OCaml change is permitted.

## Acceptance criteria

1. Install-root `verify_chain` preserves explicit `unsafe_store_path`.
2. Record-root `verify_chain` preserves explicit `unsafe_store_path`.
3. Other root-chain failures still map to stable `installation_recovery_io` without path or OS detail.
4. Intent validation remains first.
5. Exact child observation and all four snapshot facts remain unchanged.
6. Snapshot shape and the J24K3b bridge remain unchanged.
7. The observer remains read-only, crate-private, and exact-transaction-scoped.
8. Direct tests exercise unsafe replacement of the already-opened install root and record root separately.
9. All existing sixteen J24K3c1 tests remain green.
10. Focused Nextest passes with zero retries.
11. J24K3b, J24K3a, J24K2, J24J, and representative M3 regressions remain green.
12. Full `just verify` and the task packet checker pass.
13. Cargo.lock remains byte-identical and only permitted files change.
14. The task packet and worker note contain exact final evidence and a clean remote tip.

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
  -E 'test(j24k3c1)'

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
git log --oneline --decorate -8
```

Cargo.lock must remain:

```text
D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

Do not substitute `just test-rust` for full `just verify`. A pre-existing intermittent Windows handle-contention failure must be identified precisely, rerun serially, and pass before handoff.

## Forbidden changes

- No edit to the frozen architecture.
- No edit to `m3_store.rs`, `installation_recovery.rs`, `installation_publication_intent.rs`, or `installation_execution.rs`.
- No change to snapshot fields, classifier observation, recovery dispositions, or classifier matrix.
- No filesystem mutation in production code.
- No broad root accessor, caller-supplied observation path, directory enumeration, destination-content verification, evidence access, installed-root audit, cleanup, publication, repair, lock, planner, or executor wiring.
- No public API, dependency, Cargo configuration, Cargo.lock, CLI, packaging, release, or OCaml change.
- No unrelated refactor or test-helper expansion beyond the two direct root-chain fixtures.
- No files outside the permitted set.

Permitted files:

- `tethers-0.1/host-rust/src/installed.rs`;
- `tethers-0.1/host-rust/src/installation_recovery_observation_tests.rs`;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-05-j24k3c1-recovery-observer.md`.

## Stop conditions

Stop as `BLOCKED` only if preserving root-chain `unsafe_store_path` requires changing an accepted storage primitive, public API, dependency, Cargo.lock, snapshot or classifier type, or filesystem mutation in production; or if full verification still fails after one evidence-led correction.

Do not stop for failed LSP, a stale local ref, one ineffective Nextest filter, or an initial platform-fixture command that can be narrowed using an accepted repository convention.

## Expected pre-existing changes

None. The branch is expected to be clean at handoff. The worker-note scaffold commit named by `Base commit` is the correction base; the task-packet commit after it changes only `docs/CURRENT_CLINE_TASK.md`.
