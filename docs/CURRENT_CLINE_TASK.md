# Current Implementation Task

Control contract: `1`
Task: `J24H - Installation evidence access foundation`
Owner: `OpenCode`
Status: `COMPLETE`
Implementation checkpoint: `ea1252895c3b34172eb34800ced5dd4bd9b1e749`
Task colour: `Amber`
Route: `OpenCode using DeepSeek Pro V4 for narrow security-sensitive store seams; Lucy performs final review`
Base branch: `main`
Base commit: `1cfba49c0031f0e2f2f9fc136d466c8fce7994f9`
Implementation branch: `opencode/j24h-installation-evidence-access`
Worker note: `docs/worker-notes/2026-08-04-j24h-installation-evidence-access.md`
Implementation blueprint: `docs/architecture/J24H_INSTALLATION_EVIDENCE_ACCESS_FOUNDATION.md`

## Objective

Add the smallest persistence and read-only access foundation required before the
installation reconciliation planner can be implemented safely.

J24H must:

- persist complete immutable `LaunchProfileEvidence` through the existing
  audited `StoreRoot` authority;
- add non-creating `open_existing` constructors to the candidate, trust,
  conformance, and installation-approval stores needed by the future planner;
- prove every read-only opening leaves missing roots missing and valid stores
  byte-identical.

J24H performs no installation planning, trust mutation, provider launch,
conformance execution, approval creation, payload copying, installed
publication, enablement, lock, or CLI work.

Read `docs/architecture/J24H_INSTALLATION_EVIDENCE_ACCESS_FOUNDATION.md`
completely before editing. It freezes the exact methods, launch-profile record
identity, failure messages, and evidence matrix.

## Relevant background and existing behaviour

J24G is accepted. Its real implementation checkpoint is
`fa3ffcf42b613cc55219ab33210dcd07668d990a` and its accepted branch tip was
`ec467c308948178be1739ba48dc90ff8ce5ffc02` before the historical worker-note
correction.

The future installation pipeline is intentionally resumable. Trust,
conformance, and approval evidence may legitimately survive a later publication
failure. A later invocation must be able to inspect and reuse that evidence
without creating store roots merely by planning.

The repository already has immutable stores for publisher trust, exact-digest
developer approval, conformance evidence, installation approval, and installed
records. `InstalledPlugRegistry::open_existing` and
`StoreRoot::open_existing` already establish the desired non-creating pattern.

One required evidence object is not yet persisted: full
`LaunchProfileEvidence`. `ConformanceEvidence` pins only its digest. Without the
complete object, a later process cannot call `ConformanceEvidence::require_current`
and safely reuse a passed result after interruption.

J24H closes only that evidence-access gap. J24I will build the read-only planner
on top of it.

## Startup procedure

The worktree may still be on an older implementation branch. Do not inspect that
branch's packet as current authority.

1. Confirm the worktree is clean. Stop if it is not.
2. Run `git fetch origin`.
3. Verify checkpoint `1cfba49c0031f0e2f2f9fc136d466c8fce7994f9` is an ancestor of
   `origin/main`.
4. Inspect the first lines of the packet directly from `origin/main`:

   ```powershell
   git show origin/main:docs/CURRENT_CLINE_TASK.md | Select-Object -First 16
   ```

   Require J24H, OpenCode, `READY`, and branch
   `opencode/j24h-installation-evidence-access`.
5. Verify the blueprint directly from `origin/main`:

   ```powershell
   git cat-file -e origin/main:docs/architecture/J24H_INSTALLATION_EVIDENCE_ACCESS_FOUNDATION.md
   ```

6. Check that `opencode/j24h-installation-evidence-access` does not exist locally
   or remotely. If it exists, stop without resetting or overwriting it.
7. Create and switch to it from current `origin/main`:

   ```powershell
   git switch --create opencode/j24h-installation-evidence-access origin/main
   ```

8. Read the checked-out packet and blueprint completely before editing.

## Required behaviour

1. Add `CandidateRegistry::open_existing(root, quarantine_root)` with the exact
   read-only validation order frozen in the blueprint.

2. `CandidateRegistry::open_existing` must never call `create_safe_dir_all`,
   `create_dir_all`, `create_dir`, write, rename, delete, or alter permissions.

3. Add `open_existing(path)` to `PublisherTrustStore` and
   `DeveloperApprovalStore`, delegating only to `StoreRoot::open_existing`.

4. Add `open_existing(path)` to `ConformanceEvidenceStore`, delegating only to
   `StoreRoot::open_existing`.

5. Add `open_existing(path)` to `InstallationApprovalStore`, delegating only to
   `StoreRoot::open_existing`.

6. Add `LaunchProfileEvidenceStore` to `launch_profile.rs` with exactly:

   ```rust
   pub fn open(path: &Path) -> Result<Self>;
   pub fn open_existing(path: &Path) -> Result<Self>;
   pub fn create(&self, evidence: &LaunchProfileEvidence) -> Result<()>;
   pub fn load_all(&self) -> Result<Vec<LaunchProfileEvidence>>;
   ```

7. Name launch-profile evidence files from the 64-character lowercase hex suffix
   of `profile_evidence_digest`; introduce no UUID or timestamp identity.

8. `LaunchProfileEvidenceStore::create` must validate the evidence and publish
   only through `StoreRoot::create_json`. Do not add another temporary-file or
   atomic-write implementation.

9. `load_all` must reject torn temporary files, unexpected entries, filename
   mismatch, malformed evidence, and duplicate digest evidence exactly as
   frozen in the blueprint.

10. `load_all` must return records sorted by `profile_evidence_digest`.

11. Every new `open_existing` path must fail closed on missing, non-directory,
    symbolic-link, or Windows reparse/junction roots without creating anything.

12. Add focused recursive snapshot evidence proving every new read-only opening
    and load operation changes no byte or path.

13. Preserve all existing store creation, trust, launch, conformance, approval,
    installed, enablement, J24E, J24F, and J24G behaviour.

## Relevant components

- `tethers-0.1/host-rust/src/candidate.rs`
- `tethers-0.1/host-rust/src/trust.rs`
- `tethers-0.1/host-rust/src/conformance.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/launch_profile.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
- `tethers-0.1/host-rust/tests/j24h_installation_evidence_access.rs`
- `docs/architecture/J24H_INSTALLATION_EVIDENCE_ACCESS_FOUNDATION.md`
- `docs/CURRENT_CLINE_TASK.md`

## Frozen decisions and invariants

- J24H is an evidence-access foundation, not the installation planner.
- The actual read-only planner moves to J24I; the executor and CLI move one
  letter later.
- Full launch-profile evidence must be durable because conformance pins its
  digest and later revalidation requires the complete object.
- `LaunchProfileEvidence` schema and `PreparedSupervisedLaunch` behaviour remain
  unchanged.
- `StoreRoot::create_json` remains the only launch-profile publication
  authority.
- Existing `open` methods retain their current creating behaviour.
- New `open_existing` methods never create a missing root.
- Candidate and quarantine roots remain separate and are both required.
- Store corruption is never treated as an empty store.
- Temporary and unexpected entries fail closed.
- Launch-profile filenames are content identities, not new lifecycle IDs.
- Digest-derived filenames make two validly named duplicate-digest records
  structurally impossible in one flat directory; duplicate creation is
  prevented by `StoreRoot::create_json` returning `record_conflict`.
- No evidence record is rewritten or replaced.
- J24H launches no process and changes no lifecycle state.
- Tethers Core, OCaml semantics, package schemas, candidate schemas, and all
  existing evidence schemas remain unchanged.

## Acceptance criteria

1. `CandidateRegistry::open_existing` accepts two existing safe roots and
   `load_all` preserves their recursive byte snapshot.
2. Missing candidate and quarantine roots fail without creating either path.
3. Non-directory candidate or quarantine roots fail with
   `registry_invalid` and the frozen message.
4. Unsafe symbolic-link and Windows junction/reparse roots fail closed without
   creating a target child.
5. Publisher trust and developer approval `open_existing` methods accept
   existing roots and create no missing root.
6. Conformance and installation-approval `open_existing` methods accept existing
   roots and create no missing root.
7. One valid `LaunchProfileEvidence` round-trips exactly through the new store.
8. The launch-profile filename equals the digest suffix and contains no UUID or
   timestamp identity.
9. Opening and loading the launch-profile store changes no byte or path.
10. A second create of identical evidence returns `record_conflict` and changes
    no byte.
11. Torn `.tmp`, non-JSON, filename mismatch, and malformed evidence conditions
     fail closed with the frozen code/message. A copied duplicate under a
     different filename is rejected as a filename mismatch; two validly named
     duplicate-digest records are structurally impossible in one flat directory
     with digest-derived filenames.
12. Duplicate creation is prevented by `StoreRoot::create_json` returning
    `record_conflict`, proved by the duplicate-create test.
13. `load_all` returns multiple valid records sorted by digest.
14. Missing launch-profile and every other new existing-store root return the
    expected existing store error and remain absent.
15. Existing candidate, trust, conformance, launch-profile, and approval unit
    tests remain green.
16. J24E, J24F, and J24G focused tests remain green.
17. Full suite remains green apart from the five documented `pwsh.exe not found`
    environment failures.
18. Packet checker, Rustfmt, and `git diff --check` pass.

## Required verification

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 test candidate --locked
cargo +1.89.0 test trust --locked
cargo +1.89.0 test launch_profile --locked
cargo +1.89.0 test conformance --locked
cargo +1.89.0 test installed --locked
cargo +1.89.0 test --test j24h_installation_evidence_access --locked
cargo +1.89.0 test candidate_preparation --locked
cargo +1.89.0 test --test j24e_candidate_preparation --locked
cargo +1.89.0 test --test j24f_plug_stage_cli --locked
cargo +1.89.0 test installation_request --locked
cargo +1.89.0 test --test j24g_installation_request --locked
cargo +1.89.0 test --all-targets --all-features --locked
git diff --check
```

## Permitted changes

Expected files are limited to:

- `tethers-0.1/host-rust/src/candidate.rs`
- `tethers-0.1/host-rust/src/trust.rs`
- `tethers-0.1/host-rust/src/conformance.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/launch_profile.rs`
- `tethers-0.1/host-rust/tests/j24h_installation_evidence_access.rs`
- `docs/worker-notes/2026-08-04-j24h-installation-evidence-access.md`
- `docs/CURRENT_CLINE_TASK.md` only for status transitions and the final real
  implementation checkpoint

Do not change `m3_store.rs`; it is a reference and reused authority only.
Stop before changing any other file.

## Forbidden changes

Do not modify `StoreRoot`, `LaunchProfileEvidence`, `PreparedSupervisedLaunch`,
installation request types, package or candidate record schemas, trust evidence
schemas, conformance schemas, installation approval schemas, installed record
schemas, dependencies, or lockfiles.

Do not add the installation planner, executor, request digest, host lock,
provider launch, conformance execution, approval mutation, payload copying,
installed publication, enablement, `plug install`, another CLI command, download,
update, removal, registry, policy, replay, event, Anchor, Trail, OCaml, Tether
syntax, release, tag, or version work.

Do not duplicate `StoreRoot` atomic writing, path verification, JSON parsing, or
canonicalisation.

Do not amend, reset, rebase, cherry-pick, force-push, or merge into `main`.

## Stop conditions

Stop cleanly and report the smallest unresolved question if:

- the implementation branch already exists;
- current `origin/main` lacks accepted J24G or the J24H packet/blueprint;
- an existing evidence schema must change;
- `StoreRoot::open_existing` or `create_json` cannot be reused as frozen;
- a planner, process launch, lifecycle mutation, dependency, lockfile, or
  forbidden file appears necessary;
- the read-only guarantee cannot be proved with recursive snapshots;
- branch-specific failures remain after two materially different attempts.

## Expected pre-existing changes

None.

## Git and return contract

Use ordinary commits and normal push only.

After all required checks pass:

- create the authorised worker note;
- set the packet to `COMPLETE`;
- record the real full 40-character implementation commit returned by Git;
- verify that commit exists with `git cat-file -e <SHA>^{commit}` before writing
  it into the packet or worker note;
- push normally.

Return the branch, remote final SHA, real implementation checkpoint, exact
changed files, focused and full test evidence, packet/rustfmt/diff results,
worker-note path, launch-profile round-trip evidence, recursive no-mutation
proof, and explicit confirmation that J24H planned nothing, launched nothing,
and changed no installation lifecycle state.
