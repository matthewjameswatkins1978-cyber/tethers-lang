# Worker Note

Task: `J24E - Idempotent candidate preparation seam`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `f8c63b907efca1e0f9f1839d542f79221c7298f2`

Implementation checkpoint: `94134eb2b65243074ecf31e937fd52dc88123d3c`

## Requested outcome

Add one internal host-owned candidate-preparation service that composes
`package::inspect`, `candidate::extract_to_quarantine`, and `CandidateRegistry`
(open, load_all, create) into a single idempotent seam. No CLI, no trust,
no installation, no enablement, no provider launch.

## Changes made

- `tethers-0.1/host-rust/src/candidate.rs` — exposed `verify_existing_chain` as
  `pub(crate)` to reuse its path-safety authority in the new module.
- `tethers-0.1/host-rust/src/candidate_preparation.rs` — new module with:
  - `CandidatePreparationDisposition` enum (Created, Existing)
  - `CandidatePreparation` struct (candidate, disposition)
  - `prepare_installation_candidate` public function
  - internal helpers: `exact_replay`, `refuse_semantic_conflict`,
    `cleanup_new_empty_roots`, `rollback_new_quarantine`,
    `require_absolute_existing_file`,
    `require_absolute_existing_safe_directory`
- `tethers-0.1/host-rust/src/lib.rs` — exported `pub mod candidate_preparation`
- `tethers-0.1/host-rust/tests/j24e_candidate_preparation.rs` — 15 integration tests
- `docs/worker-notes/2026-08-03-j24e-candidate-preparation-seam.md` — this note

## Decisions and assumptions

- Used `verify_existing_chain` from `candidate.rs` (made `pub(crate)`) rather
  than duplicating reparse-point validation. This is the sole candidate.rs
  change.
- Rollback uses inline canonicalisation + `starts_with` check rather than
  exposing `confined` — the confinement logic is simple enough to duplicate.
- Error construction uses `PackageError { code, message }` struct literal
  since the `PackageError::new` constructor is private.
- Exact replay comparison checks every report-pinned field listed in the
  blueprint against the existing candidate record fields.
- Zero raw-archive matches continues; one match verifies all pinned evidence;
  more than one returns `candidate_conflict`.
- Semantic conflict (same package ID/version, different semantic digest) is
  checked before extraction, preserving the existing `semantic_conflict` code.
- Cleanup only removes newly created empty roots and newly created confined
  quarantine directories. Never touches pre-existing paths.
- No CLI, no install/installed-records/enablements/trust/conformance/approvals.

## Evidence

- `cargo +1.89.0 fmt --all -- --check` — PASS
- `cargo +1.89.0 test --test j24a_plug_inspect_cli --locked` — 3 passed
- `cargo +1.89.0 test --test j24b_plug_list_cli --locked` — 4 passed
- `cargo +1.89.0 test --test j24c_plug_disable_cli --locked` — 9 passed (serially; one test has parallel temp-dir race on this machine, unrelated to J24E)
- `cargo +1.89.0 test --test j24d_plug_enable_scope_file --locked` — 16 passed
- `cargo +1.89.0 test --test j24e_candidate_preparation --locked` — 15 passed
- `cargo +1.89.0 test --all-targets --all-features --locked` — 906 passed, 5 documented `pwsh.exe not found` baseline failures
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — PASS
- `git diff --check` — PASS

## Discoveries

- J24C test `unknown_installed_id_fails_without_mutation` can fail with
  `PermissionDenied` when run in parallel with other temp-dir tests (race
  on `remove_dir_all`). Passes reliably with `--test-threads=1`. Pre-existing,
  not introduced by J24E.
- Two packages built with the same provider bytes produce byte-identical ZIP
  output (JCS-stable plug.json + deterministic ZIP timestamps), which is
  exactly what enables exact replay. Different provider bytes produce different
  archives with genuinely different raw_archive_digest values.

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
