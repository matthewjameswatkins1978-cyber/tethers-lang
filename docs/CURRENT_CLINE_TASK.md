# Current Implementation Task

Control contract: `1`
Task: `J24I - Exact-candidate installation trust`
Owner: `OpenCode`
Status: `READY`
Task colour: `Amber`
Route: `OpenCode using DeepSeek Pro V4 for a security-sensitive but bounded trust record and evidence extension; Lucy performs final review`
Base branch: `main`
Base commit: `712ae4d27a969375e7b2b8980b2e17c5d26e3377`
Implementation branch: `opencode/j24i-exact-candidate-installation-trust`
Worker note: `docs/worker-notes/2026-08-04-j24i-exact-candidate-installation-trust.md`
Implementation blueprint: `docs/architecture/J24I_EXACT_CANDIDATE_INSTALLATION_TRUST.md`

## Objective

Implement the missing exact-candidate trust authority required by the frozen
installation request.

J24I must:

- add one immutable trust record pinned to candidate ID and candidate-record
  digest;
- persist it through the existing audited `StoreRoot` authority;
- extend `PackageTrustEvidence` with one exact-candidate mode;
- make that mode usable for read-only planning and exact candidate matching;
- deliberately refuse current-authority revalidation until the future locked
  executor supplies the exact trust store.

J24I performs no installation planning, provider launch, conformance, approval,
payload copying, installed publication, enablement, lock, or CLI work.

Read `docs/architecture/J24I_EXACT_CANDIDATE_INSTALLATION_TRUST.md` completely
before editing. It freezes the record, store methods, trust-evidence variant,
errors, validation order, fail-closed execution boundary, and evidence matrix.

## Relevant background and existing behaviour

J24G is accepted and provides the typed hostile-input-safe request:

```json
{
  "schema": "tethers.plug-install/1",
  "candidate_id": "<canonical UUID>",
  "trust": { "scope": "exact_candidate" },
  "conformance": {
    "allow_non_isolated_supervised_execution": true
  },
  "installation": { "target_state": "disabled" }
}
```

J24H is accepted on `main` at
`b3d4b04605155575a974127b33b4147700d3b428`. It adds durable launch-profile
evidence and non-creating store-opening seams.

The existing trust model has two modes:

- signed publisher trust, which applies to a signing key and optional namespace;
- unsigned developer approval, which applies to one semantic package digest.

Neither mode is pinned to one candidate ID and candidate-record digest. The
installation request explicitly asks for `exact_candidate`; J24I must not
silently reinterpret that as publisher-wide or semantic-digest-wide trust.

The future read-only planner needs to decide whether exact-candidate trust must
be created or can be reused. J24I supplies only that authority. J24J will build
the planner.

## Startup procedure

The current worktree may still be on an older implementation branch. Do not use
that branch's packet as current authority.

1. Confirm the worktree is clean. Stop if it is not.
2. Run `git fetch origin`.
3. Verify blueprint checkpoint
   `712ae4d27a969375e7b2b8980b2e17c5d26e3377` is an ancestor of
   `origin/main`.
4. Verify accepted J24H is an ancestor of `origin/main`:

   ```powershell
   git merge-base --is-ancestor b3d4b04605155575a974127b33b4147700d3b428 origin/main
   ```

5. Inspect the first packet lines directly from `origin/main`:

   ```powershell
   git show origin/main:docs/CURRENT_CLINE_TASK.md | Select-Object -First 16
   ```

   Require J24I, OpenCode, `READY`, and branch
   `opencode/j24i-exact-candidate-installation-trust`.
6. Verify the blueprint directly from `origin/main`:

   ```powershell
   git cat-file -e origin/main:docs/architecture/J24I_EXACT_CANDIDATE_INSTALLATION_TRUST.md
   ```

7. Confirm the implementation branch does not exist locally or remotely:

   ```powershell
   git branch --list opencode/j24i-exact-candidate-installation-trust
   git branch --remotes --list origin/opencode/j24i-exact-candidate-installation-trust
   ```

   If either command reports the branch, stop without resetting or overwriting
   it.
8. Create and switch to the implementation branch from current remote main:

   ```powershell
   git switch --create opencode/j24i-exact-candidate-installation-trust origin/main
   ```

9. Read the checked-out packet and blueprint completely before editing.

## Required behaviour

1. Add `tethers-0.1/host-rust/src/installation_trust.rs` and export it from
   `lib.rs`.

2. Implement exactly the `ExactCandidateTrustRecord` fields frozen in the
   blueprint. Introduce no second UUID; candidate ID is the record identity.

3. Implement `ExactCandidateTrustStore` with exactly:

   ```rust
   pub fn open(path: &Path) -> Result<Self>;
   pub fn open_existing(path: &Path) -> Result<Self>;
   pub fn create(
       &self,
       candidate: &CandidateRecord,
       request: &InstallationRequest,
       approving_authority: &str,
   ) -> Result<ExactCandidateTrustRecord>;
   pub fn find(
       &self,
       candidate_id: &str,
   ) -> Result<Option<ExactCandidateTrustRecord>>;
   pub fn load_all(&self) -> Result<Vec<ExactCandidateTrustRecord>>;
   ```

4. `open` and `open_existing` must delegate to the corresponding `StoreRoot`
   methods. Do not duplicate store-root verification or atomic writing.

5. `create` must revalidate the complete typed installation request, candidate
   binding, exact trust scope, explicit supervised-execution approval, disabled
   target, and non-empty approving authority before any publication.

6. Copy only frozen candidate identity/evidence fields into the record, calculate
   its canonical digest, validate it, and publish through
   `StoreRoot::create_json(candidate_id, record)`.

7. Add `ExactCandidateTrustRecord::require_for_candidate` with the exact binding
   checks and stable mismatch error frozen in the blueprint.

8. `load_all` must reject temporary, non-JSON, malformed, and filename-mismatched
   evidence, retain a defensive duplicate-candidate check, and sort by candidate
   ID.

9. `find` must inspect the validated store view. Corrupt evidence must never be
   treated as absence.

10. Extend `TrustModeEvidence` with exactly:

    ```rust
    ExactCandidate {
        candidate_id: String,
        candidate_record_digest: String,
        installation_trust_record_digest: String,
        approving_authority: String,
    }
    ```

11. Add `PackageTrustEvidence::exact_candidate(record)` and compute normal
    package-trust evidence deterministically from the validated record.

12. Extend `PackageTrustEvidence::validate` and `require_for_candidate` so the
    new mode is strictly validated and accepts only the exact candidate.

13. Preserve the serialised shape and behaviour of existing signed-publisher and
    unsigned-developer modes.

14. `PackageTrustEvidence::revalidate_current` must fail closed for the new mode
    with:

    - code `trust_exact_candidate_authority_required`
    - message `exact-candidate trust requires current installation-trust authority`

15. Do not wire the new mode into provider launch, conformance execution,
    installation approval, installed publication, or operational launch.

16. Add the complete focused evidence matrix from the blueprint.

## Relevant components

- `tethers-0.1/host-rust/src/installation_trust.rs`
- `tethers-0.1/host-rust/src/trust.rs`
- `tethers-0.1/host-rust/src/installation_request.rs`
- `tethers-0.1/host-rust/src/candidate.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/tests/j24i_exact_candidate_installation_trust.rs`
- `docs/architecture/J24I_EXACT_CANDIDATE_INSTALLATION_TRUST.md`
- `docs/CURRENT_CLINE_TASK.md`

## Frozen decisions and invariants

- `exact_candidate` means one candidate ID plus one candidate-record digest.
- Publisher trust is not granted or widened by this task.
- Semantic-digest developer approval is not silently rebranded as exact-candidate
  trust.
- Candidate ID is the store identity; no extra UUID or timestamp identity is
  added.
- The trust record is immutable and atomically published by `StoreRoot`.
- The full typed request is rechecked before record creation even though it was
  parsed by J24G.
- The trust record contains no conformance, installed, enablement, policy,
  credential, Trail, or Anchor authority.
- Exact package-trust evidence is deterministic from the validated trust record.
- Existing signed and developer evidence remain byte/schema compatible.
- Exact trust evidence cannot pass current-authority revalidation in J24I.
- The future planner may inspect it; existing execution paths must refuse it.
- Candidate/quarantine, trust, conformance, installation approval, installed,
  and enablement authorities remain separate.
- Tethers Core and OCaml semantics remain untouched.

## Acceptance criteria

1. The new module and `lib.rs` export compile without dependency or lockfile
   changes.
2. A valid exact installation request and candidate create one valid exact trust
   record.
3. The file is named exactly from the candidate ID with no new lifecycle UUID.
4. `load_all` and `find` return the exact record and preserve recursive snapshots.
5. Missing `open_existing` roots remain missing.
6. A second exact create returns `record_conflict` and changes no byte.
7. Wrong schema, wrong candidate, non-exact request construction, false execution
   approval, wrong target, and empty authority fail before publication.
8. Temporary, non-JSON, malformed, and filename-mismatched evidence fail closed.
9. Copied evidence under another filename returns the frozen filename-mismatch
   refusal; no structurally impossible duplicate fixture is required.
10. A record refuses a different candidate even when semantic digest text is the
    same.
11. Exact `PackageTrustEvidence` is deterministic and validates.
12. Exact package-trust evidence accepts only its exact candidate.
13. Current-authority revalidation refuses exact trust with the frozen code and
    message.
14. Existing signed-publisher and unsigned-developer trust suites remain green.
15. J24E through J24H focused regressions remain green.
16. Full suite remains green apart from the five documented `pwsh.exe not found`
    environment failures.
17. Packet checker, Rustfmt, and `git diff --check` pass.
18. J24I launches no process and creates no conformance, approval, installed,
    enablement, policy, Trail, or Anchor state.

## Required verification

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 test installation_trust --locked
cargo +1.89.0 test trust --locked
cargo +1.89.0 test --test j24i_exact_candidate_installation_trust --locked
cargo +1.89.0 test installation_request --locked
cargo +1.89.0 test --test j24g_installation_request --locked
cargo +1.89.0 test launch_profile --locked
cargo +1.89.0 test --test j24h_installation_evidence_access --locked
cargo +1.89.0 test candidate_preparation --locked
cargo +1.89.0 test --test j24e_candidate_preparation --locked
cargo +1.89.0 test --test j24f_plug_stage_cli --locked
cargo +1.89.0 test --all-targets --all-features --locked
git diff --check
```

## Editing recovery discipline

If an exact replacement reports that `oldString` was not found:

1. do not retry the identical replacement;
2. reread the current file;
3. locate the smallest stable surrounding anchor;
4. make a fresh, smaller patch against the latest contents;
5. stop after two materially different failures rather than rewriting the whole
   file.

## Permitted changes

Expected files are limited to:

- `tethers-0.1/host-rust/src/installation_trust.rs`
- `tethers-0.1/host-rust/src/trust.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/tests/j24i_exact_candidate_installation_trust.rs`
- `docs/worker-notes/2026-08-04-j24i-exact-candidate-installation-trust.md`
- `docs/CURRENT_CLINE_TASK.md` only for status transitions and the final verified
  implementation checkpoint

Stop before changing any other file.

`installation_request.rs`, `candidate.rs`, and `m3_store.rs` are read-only
references for this task.

## Forbidden changes

Do not modify installation request, candidate, package, store-root,
launch-profile, conformance, installation-approval, installed, enablement,
operational-scope, CLI, application, or Plug-command code.

Do not add the planner, host-data-root orchestration, installation lock, provider
preparation or launch, conformance execution, approval creation, payload copying,
installed publication, enablement, `plug install`, another CLI command, download,
update, removal, registry, policy, replay, event, Anchor, Trail, OCaml, Tether
syntax, release, tag, or version work.

Do not change dependencies or lockfiles.

Do not amend, reset, rebase, cherry-pick, force-push, or merge into `main`.

## Stop conditions

Stop cleanly and report the smallest unresolved question if:

- the implementation branch already exists;
- current `origin/main` lacks accepted J24H or the J24I packet/blueprint;
- exact-candidate trust cannot be added without modifying an existing evidence
  record outside `TrustModeEvidence`;
- existing signed or developer trust serialisation would change;
- a planner, process launch, lifecycle mutation, dependency, lockfile, or
  forbidden file appears necessary;
- a stale exact-edit replacement fails twice after rereading and using materially
  different anchors;
- branch-specific failures remain after two materially different attempts.

## Expected pre-existing changes

None.

## Git and return contract

Use ordinary commits and normal push only.

After all required checks pass:

- create the authorised worker note;
- set the packet to `COMPLETE`;
- make the implementation commit normally;
- obtain the real 40-character SHA from Git;
- verify it exists with `git cat-file -e <SHA>^{commit}` before recording it;
- record that exact SHA in both packet and worker note;
- create completion documentation separately;
- push normally.

Return the branch, remote final SHA, verified implementation checkpoint, exact
changed files, focused and full test evidence, packet/rustfmt/diff results,
worker-note path, exact-candidate binding evidence, fail-closed current-authority
evidence, and explicit confirmation that J24I planned nothing, launched nothing,
and changed no installation lifecycle state beyond its new immutable trust
record store fixture tests.
