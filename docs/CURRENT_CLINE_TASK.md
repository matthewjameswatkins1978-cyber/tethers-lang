# Current Implementation Task

Control contract: `1`
Task: `TETHERS-0.3-P1-R1G-FIX — Correct J23C3 Scope Digest Assertion`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode applies one-correction fix`
Worker note: `docs/worker-notes/2026-08-10-0.3-p1-r1g-fix-j23c3-scope-digest.md`
Base branch: `feature/0.3-p1-r1f-j23c2-generic-scope-conformance`
Base commit: `a0390fbfb5df439b3c000c39537d16e6ce198e7f`
Implementation branch: `feature/0.3-p1-r1g-fix-j23c3-scope-digest`
Implementation checkpoint: `83b419c5275dd6edc773eacea16dabbe4c286f7c`
OCaml switch path: `not applicable`
Rust toolchain: `1.97.1`
Rust change class: `TEST_CORRECTION_ONLY`

## Objective

Correct a stale J23C3 test assertion that contradicts the accepted Generic Operational Scope Evidence semantics. The R1G final verification gate exposed that `j23c3_installed_pdf_execution.rs:226` used `assert_eq!` where `assert_ne!` is required, because the integrity digest includes scope content by design.

## Relevant background and existing behaviour

- `OperationalScopeEvidence::create` (operational_scope.rs:66-81) includes `canonical_scope_json` in the integrity digest
- The test creates `mismatched_scope` with `max_bytes = operational_max + 1` vs `scope` with `max_bytes = operational_max`
- Different scope content → different canonical JSON → different integrity digest → `assert_ne!` required
- The pin check in `launch_profile.rs:241` catches digest mismatch as "enablement pins are stale" before the content comparison at line 249

## Required behaviour

1. Change `assert_eq!` to `assert_ne!` at line 226 (integrity digest must differ when scope content differs)
2. Update expected error message at line 242 from "enablement scope does not match supplied scope" to "enablement pins are stale" (the pin check at `launch_profile.rs:241` fires before content comparison)
3. No production code changes
4. No test changes beyond these two corrections

## Frozen decisions and invariants

1. No production code changes
2. No digest semantic changes
3. No enablement changes
4. No launch behaviour changes
5. Rejection proof must remain intact
6. P1 correction only — no P2

## Relevant components

### Authorised paths

- `tethers-0.1/host-rust/tests/j23c3_installed_pdf_execution.rs` (correction)
- `docs/CURRENT_CLINE_TASK.md` (update)
- `docs/worker-notes/2026-08-10-0.3-p1-r1g-fix-j23c3-scope-digest.md` (new)

## Acceptance criteria

1. `assert_eq!` → `assert_ne!` at line 226
2. Expected error message updated to "enablement pins are stale" at line 242
3. `cargo test --locked --test j23c3_installed_pdf_execution` PASS (1 test)
4. `cargo fmt --all -- --check` PASS
5. `git diff --check` PASS
6. No production files changed
7. Branch pushed, local == remote, clean status

## Required verification

1. `cargo test --locked --test j23c3_installed_pdf_execution` — PASS (1 test)
2. `cargo fmt --all -- --check` — PASS
3. `git diff --check` — PASS
4. `git diff -- tethers-0.1/host-rust/tests/j23c3_installed_pdf_execution.rs` — exactly 2 lines changed
5. `git diff --stat` — no production files
6. Task packet checker — `control-v1/COMPLETE`
7. `git push` + remote SHA + local == remote + clean status

## Forbidden changes

- No production code changes
- No digest semantic changes
- No enablement changes
- No launch behaviour changes
- No P2 implementation
- No unrelated cleanup

## Stop conditions

- Production file changes
- Test still fails after correction
- After two materially similar failed attempts

## Expected pre-existing changes

None. HEAD must descend from `a0390fbfb5df439b3c000c39537d16e6ce198e7f`.
