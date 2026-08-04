# Current Implementation Task

Control contract: `1`
Task: `J24I - Exact-candidate installation trust`
Owner: `OpenCode`
Status: `IN_PROGRESS`
Implementation checkpoint: `TBD`
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
before editing.

## Relevant background and existing behaviour

J24G provides the strict typed installation request for one candidate, exact
trust, explicit non-isolated supervised execution, and disabled installation.

J24H is accepted on `main` at
`b3d4b04605155575a974127b33b4147700d3b428`. It adds durable launch-profile
evidence and non-creating store-opening seams.

The existing trust model has two different scopes:

- signed publisher trust applies to a signing key and optional namespace;
- unsigned developer approval applies to one semantic package digest.

Neither is pinned to one candidate ID and candidate-record digest. J24I must not
silently reinterpret either as `exact_candidate`.

`InstallationTrustScope` and `InstallationTargetState` deliberately each expose
only one legal enum variant. Their exact values are type-level guarantees. The
request schema, candidate ID, approval boolean, and authority remain runtime
checks because public fields can be manually constructed or altered.

J24J will build the read-only planner on this exact trust authority.

## Startup procedure

1. Confirm the worktree is clean. Stop if it is not.
2. Run `git fetch origin`.
3. Verify blueprint checkpoint
   `712ae4d27a969375e7b2b8980b2e17c5d26e3377` is an ancestor of
   `origin/main`.
4. Verify accepted J24H is an ancestor of `origin/main`:

   ```powershell
   git merge-base --is-ancestor b3d4b04605155575a974127b33b4147700d3b428 origin/main
   ```

5. Inspect the packet directly from `origin/main`:

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

   Stop without overwriting it if either command reports the branch.
8. Create it from current remote main:

   ```powershell
   git switch --create opencode/j24i-exact-candidate-installation-trust origin/main
   ```

9. Read the checked-out packet and blueprint completely before editing.

## Required behaviour

1. Add `tethers-0.1/host-rust/src/installation_trust.rs` and export it from
   `lib.rs`.

2. Implement exactly the `ExactCandidateTrustRecord` fields frozen in the
   blueprint. Candidate ID is the record identity; add no second UUID.

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

4. Delegate creating, existing-only opening, path safety, and atomic JSON
   publication to the corresponding `StoreRoot` methods.

5. Before publication, validate the candidate, request schema, matching candidate
   ID, `true` supervised-execution approval, and non-empty approving authority.
   Confirm the single legal trust-scope and target-state variants without unsafe
   or impossible negative fixtures.

6. Copy only the frozen candidate fields into the record, calculate its canonical
   digest, validate it, and publish through
   `StoreRoot::create_json(candidate_id, record)`.

7. Add `ExactCandidateTrustRecord::require_for_candidate` with the frozen exact
   binding checks and mismatch error.

8. `load_all` must reject temporary, non-JSON, malformed, and filename-mismatched
   evidence, retain a defensive duplicate-candidate check, and sort by candidate
   ID.

9. `find` must use the validated store view. Corrupt evidence must never be
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

11. Add `PackageTrustEvidence::exact_candidate(record)` and calculate normal
    package-trust evidence deterministically from the validated record.

12. Extend `PackageTrustEvidence::validate` and `require_for_candidate` so the
    new mode accepts only its exact candidate.

13. Preserve the serialised fields and behaviour of existing signed-publisher
    and unsigned-developer evidence.

14. `PackageTrustEvidence::revalidate_current` must fail closed for the new mode
    with:

    - code `trust_exact_candidate_authority_required`
    - message `exact-candidate trust requires current installation-trust authority`

15. Do not wire the mode into provider launch, conformance, installation
    approval, installed publication, or operational launch.

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
- Publisher trust is not granted or widened.
- Semantic-digest developer approval is not rebranded as exact-candidate trust.
- Candidate ID is the store identity; no extra UUID appears.
- The record is immutable and atomically published through `StoreRoot`.
- Public request strings and the approval boolean are rechecked before creation.
- Single-variant trust-scope and target-state enums are compile-time guarantees;
  no unsafe fixture may fabricate alternatives.
- Exact package-trust evidence is deterministic from the validated record.
- Existing signed and developer evidence remain schema compatible.
- Exact trust evidence cannot pass current-authority revalidation in J24I.
- The future planner may inspect it; existing execution paths must refuse it.
- Candidate, trust, conformance, installation approval, installed, and enablement
  authorities remain separate.
- Tethers Core and OCaml semantics remain untouched.

## Acceptance criteria

1. The module and `lib.rs` export compile without dependency or lockfile changes.
2. A valid request and candidate create one valid exact trust record.
3. The filename is exactly the candidate ID with no new lifecycle UUID.
4. `load_all` and `find` round-trip it without changing unrelated files.
5. Missing `open_existing` roots remain missing.
6. A second exact create returns `record_conflict` and changes no byte.
7. Manually constructed wrong schema, mismatched candidate ID, false execution
   approval, and empty authority fail before publication.
8. Trust scope and target state remain guaranteed by their single-variant types;
   no unsafe or impossible negative enum fixture is required.
9. Temporary, non-JSON, malformed, and filename-mismatched evidence fail closed.
10. Copied evidence under another filename returns the frozen filename mismatch;
    no structurally impossible duplicate fixture is required.
11. A record refuses a different candidate even when semantic digest text is the
    same.
12. Exact `PackageTrustEvidence` is deterministic and validates.
13. Exact package-trust evidence accepts only its exact candidate.
14. Current-authority revalidation refuses exact trust with the frozen result.
15. Existing signed and developer trust suites remain green.
16. J24E through J24H focused regressions remain green.
17. Full suite remains green apart from the five documented `pwsh.exe not found`
    environment failures.
18. Packet checker, Rustfmt, and `git diff --check` pass.
19. J24I launches no process and creates no conformance, approval, installed,
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

`installation_request.rs`, `candidate.rs`, and `m3_store.rs` are read-only
references. Stop before changing any other file.

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
- exact trust cannot be added without changing an existing evidence record
  outside `TrustModeEvidence`;
- existing signed or developer trust serialisation would change;
- an impossible enum test appears necessary;
- a planner, process launch, lifecycle mutation, dependency, lockfile, or
  forbidden file appears necessary;
- an exact-edit replacement fails twice after rereading and using materially
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
