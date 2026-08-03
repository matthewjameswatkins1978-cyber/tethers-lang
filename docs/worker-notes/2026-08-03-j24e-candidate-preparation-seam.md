# Worker Note

Task: `J24E - Idempotent candidate preparation seam`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `f8c63b907efca1e0f9f1839d542f79221c7298f2`

Implementation checkpoint: `fbb5535a6626c985606c76a94218b1b6df518504`

## Requested outcome

Add one internal host-owned candidate-preparation service that composes
`package::inspect`, `candidate::extract_to_quarantine`, and `CandidateRegistry`
(open, load_all, create) into a single idempotent seam. No CLI, no trust,
no installation, no enablement, no provider launch.

## Post-acceptance corrections

1. **Enforce ordinary, safe package file** — `require_ordinary_absolute_file`
   now uses `fs::symlink_metadata` for file-type checks and calls
   `verify_existing_chain` on the package path to reject symlink, junction and
   reparse-backed paths. Stable error codes: `invalid_archive` for relative,
   `archive_read` for missing/unreadable, `unsafe_destination` for
   symlink/reparse paths.

2. **Make cleanup failure honest** — `is_empty_dir` changed from
   `fn(Path) -> bool` to `fn(Path) -> Result<bool, PackageError>`. A failure to
   inspect a newly created root returns `candidate_rollback_failed` rather than
   silently treating an unreadable directory as non-empty.

3. **Harden rollback confinement** — `rollback_new_quarantine` now runs
   `verify_existing_chain` on both the quarantine root and directory before
   canonicalisation, checks `symlink_metadata(...).file_type().is_dir()`,
   canonicalises both, requires strict descendant confinement, then removes.
   Any unsafe path, reparse drift, metadata failure or deletion failure returns
   `candidate_rollback_failed`.

4. **Rollback-helper unit tests** — 10 unit tests inside
   `candidate_preparation.rs` covering: rollback removes confined directory
   while preserving root/sibling/unrelated file; rollback refuses quarantine
   root itself and directories outside root; cleanup removes newly created empty
   roots while preserving pre-existing and non-empty; is_empty_dir fails on
   unreadable paths; ordinary file validation rejects relative and missing
   paths.

5. **Junction integration tests** — Two Windows-only integration tests prove
   `unsafe_destination` rejection for both junction-backed package paths and
   junction host data roots, and prove `candidates/` and `quarantine/` are not
   created.

## Changes made

- `tethers-0.1/host-rust/src/candidate.rs` — exposed `verify_existing_chain` as
  `pub(crate)` to reuse its path-safety authority in the new module.
- `tethers-0.1/host-rust/src/candidate_preparation.rs` — new module with:
  - `CandidatePreparationDisposition` enum (Created, Existing)
  - `CandidatePreparation` struct (candidate, disposition)
  - `prepare_installation_candidate` public function
  - `require_ordinary_absolute_file` with symlink_metadata + verify_existing_chain
  - Fallible `is_empty_dir` returning `Result<bool, PackageError>`
  - Hardened `rollback_new_quarantine` with verify_existing_chain + is_dir checks
  - `cleanup_new_empty_roots` using fallible is_empty_dir
  - Internal helpers: `exact_replay`, `refuse_semantic_conflict`,
    `require_absolute_existing_safe_directory`
  - 10 unit tests for rollback and cleanup helpers
- `tethers-0.1/host-rust/src/lib.rs` — exported `pub mod candidate_preparation`
- `tethers-0.1/host-rust/tests/j24e_candidate_preparation.rs` — 17 integration tests
  (original 15 + 2 Windows junction tests)
- `docs/worker-notes/2026-08-03-j24e-candidate-preparation-seam.md` — this note

## Evidence

- `cargo +1.89.0 fmt --all -- --check` — PASS
- `cargo +1.89.0 test candidate_preparation --locked` — 10 unit tests passed
- `cargo +1.89.0 test --test j24e_candidate_preparation --locked` — 17 passed
- `cargo +1.89.0 test --test j24a_plug_inspect_cli --locked` — 3 passed
- `cargo +1.89.0 test --test j24b_plug_list_cli --locked` — 4 passed
- `cargo +1.89.0 test --test j24c_plug_disable_cli --locked` — 9 passed (serially; pre-existing parallel temp-dir race)
- `cargo +1.89.0 test --test j24d_plug_enable_scope_file --locked` — 16 passed
- `cargo +1.89.0 test --all-targets --all-features --locked` — 916 passed, 5 documented `pwsh.exe not found` baseline failures
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — PASS
- `git diff --check` — PASS

## Discoveries

- Pre-existing J24C test race condition (PermissionDenied on temp-dir cleanup
  during parallel runs) noted but not caused by J24E.
- Junction-backed paths are correctly detected and rejected by the existing
  `verify_existing_chain` / `reject_reparse_or_link` authorities.

## Remaining risks

None known within packet scope.

## Smallest next action

J24F can now call `candidate_preparation::prepare_installation_candidate` as a
thin `plug stage` adapter without reopening archive or quarantine design.

## References

- Branch: `opencode/j24e-candidate-preparation-seam`
- Base: `f8c63b907efca1e0f9f1839d542f79221c7298f2`
- Blueprint: `docs/architecture/J24E_CANDIDATE_PREPARATION_BLUEPRINT.md`
- Implementation: `tethers-0.1/host-rust/src/candidate_preparation.rs`
- Tests: `tethers-0.1/host-rust/tests/j24e_candidate_preparation.rs`
