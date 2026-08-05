# Worker Note

Task: `J24K3c3 - Exact recovery evidence-chain revalidator`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `374cb57ba50e685e3fe8716ecd6f2166a6f6e9b5`
Implementation checkpoint: `727a20944270a5c71484f8c1728c339d0d7f1dbf`
Final branch tip: `861794c581462366c9760e12d19559e2416da911`
Final branch: `opencode/j24k3c3-evidence-revalidator`

## Requested outcome

Add one crate-private, read-only recovery evidence-chain revalidator. Given the current typed installation request, one validated publication intent, and the existing candidate, exact-trust, launch-profile, conformance, and approval stores, prove that the complete precomputed installed record is still justified by current host-owned evidence.

The package must revalidate the exact request and candidate, current exact-candidate authority, reconstructed package-trust evidence, pinned launch profile, passed conformance against the current suite, complete installation approval chain, and the precomputed installed record.

## Changes made

- Added `tethers-0.1/host-rust/src/installation_recovery_evidence.rs` with the crate-private, read-only `InstallationRecoveryEvidenceContext` and `revalidate_installation_recovery_evidence` entry point.
- Added `tethers-0.1/host-rust/src/installation_recovery_evidence_tests.rs` with 44 focused tests covering the full success and failure matrix.
- Extended `tethers-0.1/host-rust/src/installed.rs` with `InstallationApprovalRecord::require_for_recovery` and `InstalledPlugRecord::require_for_recovery` for narrow recovery-chain validation, and made `reviewed_capabilities` crate-private.
- Registered the new production and test modules in `tethers-0.1/host-rust/src/lib.rs`.

Only the permitted files were changed; no dependency, Cargo.lock, public API, or unrelated module edits.

## Decisions and assumptions

- Kimi K2.7Code is the selected implementation model for this second measured repository-reading package.
- This package is read-only and performs no destination verification, global installed-root audit, recovery classification, cleanup, publication, intent removal, lock integration, or executor wiring.
- Existing stores and validation seams remain authoritative; no parallel trust model is permitted.

## Evidence

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` passed.
- `cargo nextest run --config-file .config/nextest.toml --manifest-path tethers-0.1/host-rust/Cargo.toml --all-features --locked -E 'test(j24k3c3)'` passed: 44/44.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --lib j24k3c3 --locked` passed: 44/44.
- Regression suites passed: J24K3c2 (21/21), J24K3c1 (20/20), J24K3b (16/16), J24K3a (25/25), J24K2 (26/26), J24I (30/30), J24H (19/19), J24J (24/24), M3 lifecycle (13/13).
- Full `$env:PATH = "$PSHOME;$env:PATH"; just verify` failed once on the pre-existing intermittent Windows handle-contention failure in `m3_lifecycle::m3_malformed_and_interrupted_conformance_fail_without_retry_or_install` (`Os { code: 5, kind: PermissionDenied }`). Rerunning the failing test serially with `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --test m3_lifecycle --locked m3_malformed_and_interrupted_conformance_fail_without_retry_or_install` passed.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` passed after final commit: `PASS task packet consistency (control-v1/COMPLETE): base 374cb57, HEAD 727a209`.
- `Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256` returned `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`.
- `git diff --check` reported only the expected LF-to-CRLF conversion warnings for the edited Rust files; no trailing-whitespace or whitespace errors.
- No `Cargo.lock` change; only permitted files were modified.

## Discoveries

- The `m3_lifecycle` integration test `m3_malformed_and_interrupted_conformance_fail_without_retry_or_install` exhibits a pre-existing intermittent Windows handle-contention failure under parallel execution (`PermissionDenied`, code 5). It is unrelated to the J24K3c3 changes and passes when rerun serially.
- The revalidator can be implemented entirely by composing the existing accepted seams without changing any evidence schema or store contract.

## Remaining risks

- The `m3_lifecycle` intermittent handle-contention failure may still occasionally fail parallel runs; it is unrelated to this change and serial rerun is the accepted mitigation.
- The recovery revalidator deliberately relies on the exact candidate ID, request field, and every pinned digest. Any future loosening of the request fields or approval-conformance cross-pin checks would need a new design gate.
- Lower-layer error messages are mapped to the three stable recovery-facing codes, but individual helpers must continue to use only those mapped error families for any new paths.

## Smallest next action

Hand off to Lucy for independent review and routine safe merge. No further implementation work is required.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_request.rs`
- `tethers-0.1/host-rust/src/candidate.rs`
- `tethers-0.1/host-rust/src/current_trust.rs`
- `tethers-0.1/host-rust/src/installation_trust.rs`
- `tethers-0.1/host-rust/src/launch_profile.rs`
- `tethers-0.1/host-rust/src/conformance.rs`
- `tethers-0.1/host-rust/src/installed.rs`
