# Independent Review Note

Task: `J24K3c4 - Global installed-root consistency auditor and correction`
Reviewer: `Lucy`
Status: `ACCEPTED`
Accepted main before merge: `e95e061e815d69b91b0637d08f84caaa602f1772`
Reviewed branch tip before this note: `cf2251ed5d10b30a008b5b2bc4a3414d22f7f59f`
Original implementation checkpoint: `31c741b663e08ffd631004de7ca0d3556f5cedfe`
Correction implementation checkpoint: `ff75243693c3b9fd0709cd9043f1642ab43e614b`

## Review outcome

Accepted for routine fast-forward merge.

The crate-private recovery audit:

- validates an optional publication intent before filesystem or store access;
- revalidates both already-opened registry roots;
- loads the complete installed-record set through the accepted `load_all()` boundary;
- requires canonical lowercase UUID installed identities and exact `plug-<installed_id>` destinations;
- rejects duplicate or contradictory destination claims;
- allows one validated current intent to account only for its exact destination;
- enumerates only direct `plug-*` namespace children;
- rejects malformed, untracked, non-directory, non-UTF-8, symlink, junction, and reparse state with the frozen recovery-facing classifications;
- performs no mutation.

The review correction replaced the blanket installed-state error collapse with a private mapper that preserves `unsafe_store_path`, maps `store_io` to `installation_recovery_io`, and maps other invalid installed state to `installation_recovery_conflict`. Platform-gated regressions prove the tracked-destination `load_all()` route, while the original direct-enumeration reparse tests remain intact.

No production correction remains.

## Verification evidence

OpenCode reported:

- focused J24K3c4 Nextest: 24 passed, 0 failed, 0 retries;
- full `just verify` with serial Rust tests: 1,355 passed, 0 failed, 0 skipped;
- library tests: 1,116 passed;
- integration tests: 239 passed;
- task packet checker: PASS, control-v1/COMPLETE;
- Cargo.lock SHA-256 unchanged: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`;
- `git diff --check`: clean;
- working tree: clean.

The reviewer inspected the actual production code, direct tests, changed-file boundary, and commit ancestry. The Rust suite was not independently rerun by the reviewer.

## SHA-note clarification

The branch accumulated several documentation-only commits attempting to record their own future final SHA. That creates an unavoidable self-reference chase. GitHub shows:

- verified documentation chain checkpoint: `3362cd7f78472fee2fe6ec7f7123bb8f02f1eb63`;
- two later commits through reviewed tip `cf2251ed5d10b30a008b5b2bc4a3414d22f7f59f` changed only one SHA line in `docs/CURRENT_CLINE_TASK.md` and one SHA line in the correction worker note;
- no Rust, tests, Cargo files, or Cargo.lock changed after the verified checkpoint.

The final pasted SHA `cf2251e1d8c9c6ddc6e16411cce480dd69eddcdf` was a transcription error. The actual reviewed remote tip was `cf2251ed5d10b30a008b5b2bc4a3414d22f7f59f`.

This independent note is the external fixed point for the audit trail and deliberately does not attempt to name its own resulting commit.

## Remaining boundary

Recovery classification, staging cleanup, exact record publication, intent removal, lock integration, planner reconciliation, and executor wiring remain later J24K3 work.