# Worker Note

Task: `J24G - Strict Plug installation request contract`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `f5e621bee4338a496888daaf78e2f029e4ab0914`

Implementation checkpoint: `fa3ffcf4f7c8e96c0a7f5e2b3f8d7a9c6b1e4d2f`

## Requested outcome

Implement the strict, hostile-input-safe JSON installation request boundary for
one exact candidate, exact-candidate trust, explicitly approved supervised
conformance, and disabled installation only. The boundary must be bounded,
read-only, duplicate-aware, and limited to the frozen stable error contract.

## Changes made

- Added `tethers-0.1/host-rust/src/installation_request.rs` with the frozen
  typed request, enums, stable error shape, bounded loader, shared duplicate-key
  parser use, exact shape validation, canonical UUID validation, RFC 6901
  pointers, and private error construction.
- Exported `installation_request` from
  `tethers-0.1/host-rust/src/lib.rs`.
- Added `tethers-0.1/host-rust/tests/j24g_installation_request.rs` covering the
  valid request, byte limits, hostile JSON, every required field and nested
  shape, semantic values, UUID forms, path failures, stable errors, and
  filesystem no-mutation evidence.
- Updated `docs/CURRENT_CLINE_TASK.md` to `COMPLETE` with this checkpoint.

## Decisions and assumptions

- File loading uses `symlink_metadata`, rejects non-regular final entries, and
  reads through `File::take(INSTALLATION_REQUEST_MAX_BYTES + 1)` without
  canonicalising the path or using `fs::read`.
- The shared `crate::manifest::parse_value_no_dupes` parser owns malformed,
  duplicate-key, and trailing-content rejection before shape validation.
- The Windows symlink test skips only when the host returns
  `ERROR_PRIVILEGE_NOT_HELD` (1314); other fixture errors fail the test.

## Evidence

- `pwsh -NoProfile -File scripts/check-dev-tools.ps1` — all repository-required
  tools detected.
- `git merge-base --is-ancestor f5e621bee4338a496888daaf78e2f029e4ab0914 origin/main`
  — PASS.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` while
  `IN_PROGRESS` — PASS (`control-v1/IN_PROGRESS`).
- `cargo +1.89.0 fmt --all -- --check` — PASS.
- `cargo +1.89.0 test installation_request --locked` — 2 passed.
- `cargo +1.89.0 test --test j24g_installation_request --locked` — 16 passed.
- `cargo +1.89.0 test candidate_preparation --locked` — 10 passed.
- `cargo +1.89.0 test --test j24e_candidate_preparation --locked` — 17 passed.
- `cargo +1.89.0 test --test j24f_plug_stage_cli --locked` — 6 passed.
- `cargo +1.89.0 test --all-targets --all-features --locked` — 921 passed and
  5 failed only because `pwsh.exe` was not found, matching the packet’s
  documented environment failures.
- `git diff --check` — PASS before the completion documentation changes.
- Integration tests prove the 16 KiB boundary and bounded loader, stable
  `installation_request_invalid` and `installation_request_io` codes, exact
  frozen messages and field pointers, and no filesystem path creation,
  deletion, or modification by parsing/loading.
- J24G implementation commit: `fa3ffcf4f7c8e96c0a7f5e2b3f8d7a9c6b1e4d2f`.

## Discoveries

- This Windows environment does not grant symbolic-link creation privilege;
  the final-symlink test therefore takes its explicit 1314 skip path. Existing
  Windows junction-based tests remain runnable.

## Remaining risks

The five documented full-suite `pwsh.exe not found` failures remain an
environment limitation. No known J24G implementation risk remains within the
packet scope.

## Smallest next action

Lucy performs the bounded final review of the pushed J24G branch.

## References

- `docs/architecture/J24G_INSTALLATION_REQUEST_CONTRACT.md`
- `tethers-0.1/host-rust/src/installation_request.rs`
- `tethers-0.1/host-rust/tests/j24g_installation_request.rs`
- Branch: `opencode/j24g-installation-request-contract`
- Implementation commit: `fa3ffcf4f7c8e96c0a7f5e2b3f8d7a9c6b1e4d2f`
