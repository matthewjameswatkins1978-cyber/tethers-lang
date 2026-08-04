Task: `J24H - Installation evidence access foundation`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `1cfba49c0031f0e2f2f9fc136d466c8fce7994f9`

Implementation checkpoint: `ea1252895c3b34172eb34800ced5dd4bd9b1e749`

## Requested outcome

Add the smallest persistence and read-only access foundation required before the
installation reconciliation planner. J24H persists `LaunchProfileEvidence` through
the existing `StoreRoot` authority and adds non-creating `open_existing`
constructors to candidate, trust, conformance, and installation-approval stores.

## Changes made

- `candidate.rs`: Added `CandidateRegistry::open_existing` following the exact
  blueprint validation order (lexical equality, chain verification, directory
  check, canonicalisation, same-location rejection). Never calls
  `create_safe_dir_all`, `create_dir_all`, or any mutating function.

- `trust.rs`: Added `open_existing(path)` to `PublisherTrustStore` and
  `DeveloperApprovalStore`, each delegating solely to
  `StoreRoot::open_existing`.

- `conformance.rs`: Added `open_existing(path)` to `ConformanceEvidenceStore`,
  delegating solely to `StoreRoot::open_existing`.

- `installed.rs`: Added `open_existing(path)` to `InstallationApprovalStore`,
  delegating solely to `StoreRoot::open_existing`.

- `launch_profile.rs`: Added `LaunchProfileEvidenceStore` with `open`,
  `open_existing`, `create` (via `StoreRoot::create_json`), and `load_all`.
  Filename identity is the 64-character lowercase hex digest suffix; no UUID or
  timestamp identity introduced. `load_all` validates every record, rejects torn
  temporary files, unexpected entries, filename mismatches, and duplicate digest
  evidence, then returns records sorted by `profile_evidence_digest`.

- `tests/j24h_installation_evidence_access.rs`: 19 focused integration tests
  covering candidate registry open-existing, M3 store open-existing,
  launch-profile round-trip, filename identity, no-mutation snapshots, duplicate
  creation refusal, torn/non-JSON/mismatch/malformed evidence rejections, missing
  root preservation, non-directory roots, junction-backed root refusal on Windows,
  and sorted multiple records.

- `docs/CURRENT_CLINE_TASK.md`: Status transitions and structural constraint
  documented (digest-derived filenames make two validly named duplicate-digest
  records structurally impossible in one flat store).

## Decisions and assumptions

- `CandidateRegistry::open_existing` uses `symlink_metadata` to inspect root
  entries before canonicalisation, matching the existing `open` safety posture
  without calling `create_safe_dir_all`.

- The duplicate-digest defensive check in `LaunchProfileEvidenceStore::load_all`
  remains in the code but is structurally unreachable through the normal store
  path because content-identity-derived filenames prevent two differently-named
  files from sharing the same `profile_evidence_digest`. A copied evidence record
  under a different filename is caught by the filename mismatch check.

- The Windows junction test uses `cmd /C mklink /J` consistent with existing
  J24E and J24F tests.

## Evidence

- `pwsh -NoProfile -File scripts/check-dev-tools.ps1` — all tools detected.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` —
  PASS (`control-v1/IN_PROGRESS`).
- `cargo +1.89.0 fmt --all -- --check` — PASS.
- `cargo +1.89.0 test candidate --locked` — 31 passed.
- `cargo +1.89.0 test trust --locked` — 31 passed.
- `cargo +1.89.0 test launch_profile --locked` — 11 passed.
- `cargo +1.89.0 test conformance --locked` — 6 passed.
- `cargo +1.89.0 test installed --locked` — 31 passed.
- `cargo +1.89.0 test --test j24h_installation_evidence_access --locked` —
  19 passed.
- `cargo +1.89.0 test candidate_preparation --locked` — 10 passed.
- `cargo +1.89.0 test --test j24e_candidate_preparation --locked` — 17 passed.
- `cargo +1.89.0 test --test j24f_plug_stage_cli --locked` — 6 passed.
- `cargo +1.89.0 test installation_request --locked` — 2 passed.
- `cargo +1.89.0 test --test j24g_installation_request --locked` — 16 passed.
- `cargo +1.89.0 test --all-targets --all-features --locked` — 921 passed,
  5 failed only because `pwsh.exe` was not found (documented environment
  failures).
- `git diff --check` — PASS.

## Discoveries

None beyond the structural impossibility of violating the duplicate-digest check
through normal store operations with digest-derived filenames, documented in the
packet.

## Remaining risks

The five documented `pwsh.exe not found` full-suite failures remain an
environment limitation. No known J24H implementation risk remains within the
packet scope.

## Smallest next action

Lucy performs the bounded final review of the pushed J24H branch.

## References

- `docs/architecture/J24H_INSTALLATION_EVIDENCE_ACCESS_FOUNDATION.md`
- `tethers-0.1/host-rust/src/candidate.rs`
- `tethers-0.1/host-rust/src/trust.rs`
- `tethers-0.1/host-rust/src/conformance.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/launch_profile.rs`
- `tethers-0.1/host-rust/tests/j24h_installation_evidence_access.rs`
- Branch: `opencode/j24h-installation-evidence-access`
- Implementation commit: `ea1252895c3b34172eb34800ced5dd4bd9b1e749`
