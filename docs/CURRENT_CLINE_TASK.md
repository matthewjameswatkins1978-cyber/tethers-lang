# Current Implementation Task

Control contract: `1`
Task: `J24E - Idempotent candidate preparation seam`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode using DeepSeek Pro V4 for bounded cross-module candidate orchestration; Lucy performs final review`
Base branch: `main`
Base commit: `f8c63b907efca1e0f9f1839d542f79221c7298f2`
Implementation branch: `opencode/j24e-candidate-preparation-seam`
Worker note: `docs/worker-notes/2026-08-03-j24e-candidate-preparation-seam.md`
Implementation blueprint: `docs/architecture/J24E_CANDIDATE_PREPARATION_BLUEPRINT.md`
Implementation checkpoint: `94134eb2b65243074ecf31e937fd52dc88123d3c`

## Objective

Add one internal host-owned candidate-preparation application seam that composes
existing package inspection, quarantine extraction and candidate registry
authorities:

```rust
pub fn prepare_installation_candidate(
    host_data_root: &Path,
    package_path: &Path,
) -> Result<CandidatePreparation, PackageError>
```

J24E adds no CLI. Its purpose is to make J24F a thin `plug stage` adapter that
cannot accidentally reopen archive, quarantine, replay or candidate-identity
design.

The prepared object remains an untrusted, unapproved, uninstalled, disabled and
non-operational installation candidate.

## Relevant background and existing behaviour

J24A through J24D are accepted. The public Plug surface can inspect packages,
list installed state, enable from a permission file and disable through immutable
transitions.

The low-level candidate path is already accepted:

- `package::inspect` performs strict, non-executing `.tetherplug` inspection;
- `candidate::extract_to_quarantine` repeats inspection, verifies bytes and
  publishes one immutable quarantine directory;
- `CandidateRegistry::open`, `load_all` and `create` own candidate storage,
  validation and immutable identity.

These functions are deliberately separate. J24E may compose them but must not
replace their parsers, path checks, digest verification, record validation or
publication rules.

Read the implementation blueprint in full before editing. It supplies the exact
service shape, ordering, replay comparison, rollback boundary and fixture recipe.

## Required behaviour

1. Start from current `origin/main` after this packet is merged:

   - run `git fetch origin`;
   - verify the worktree is clean;
   - verify base commit `f8c63b907efca1e0f9f1839d542f79221c7298f2`
     is an ancestor of `origin/main`;
   - verify the packet names J24E, OpenCode and `READY`;
   - verify the blueprint exists;
   - create `opencode/j24e-candidate-preparation-seam` from current
     `origin/main`;
   - if that branch already exists locally or remotely, stop and report rather
     than resetting or overwriting it.

2. Add `src/candidate_preparation.rs` with exactly these public value types:

   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum CandidatePreparationDisposition {
       Created,
       Existing,
   }

   #[derive(Debug, Clone, PartialEq, Eq)]
   pub struct CandidatePreparation {
       pub candidate: CandidateRecord,
       pub disposition: CandidatePreparationDisposition,
   }
   ```

   Add the exact public function named in the Objective. Export the module from
   `src/lib.rs`. Do not expose internal matching or cleanup helpers.

3. The service owns this first public host layout beneath one supplied existing
   host data root:

   ```text
   candidates/
   quarantine/
   ```

   Require `host_data_root` and `package_path` to be absolute. Require the host
   root to already exist as an ordinary directory with a safe non-reparse path
   chain. Never create the host root. Require the package path to identify an
   existing ordinary file.

4. Inspect before mutation:

   - call `package::inspect(package_path)` before creating either child root;
   - preserve every package inspector error code;
   - malformed, unreadable, unsupported or unsafe packages create no candidate
     or quarantine path;
   - never launch or execute any package payload.

5. Open and validate candidate state only through existing authorities:

   - derive `candidates` and `quarantine` below the host root;
   - record whether each child existed before the call;
   - call `CandidateRegistry::open` once;
   - call `CandidateRegistry::load_all` once;
   - do not add a second record parser, quarantine verifier, semantic digest
     calculator or registry implementation.

6. Exact replay is idempotent:

   - find candidates with the same raw archive digest;
   - zero matches continues preparation;
   - more than one match fails `candidate_conflict`;
   - exactly one match is reusable only when every report-pinned field listed in
     the blueprint agrees;
   - agreeing evidence returns that same candidate ID with disposition
     `Existing` and performs no write;
   - disagreeing evidence fails `record_invalid` and performs no write.

7. Refuse semantic conflict before extraction:

   - same package ID and package version with a different semantic package
     digest returns the existing `semantic_conflict` code;
   - the candidate and quarantine trees remain byte-for-byte unchanged;
   - different raw archives with the same semantic package digest are not
     automatically treated as exact replay because signature evidence may differ.

8. New preparation follows the accepted authorities in this order:

   - call `candidate::extract_to_quarantine` once;
   - call `CandidateRegistry::create` once;
   - return the created record with disposition `Created`;
   - do not hand-build a candidate record, quarantine directory or package
     evidence value.

9. Rollback is narrow and honest:

   - incomplete extractor staging remains owned by the existing extractor;
   - if record publication fails after final quarantine publication, remove only
     that newly returned quarantine directory;
   - prefer one narrowly scoped `pub(crate)` helper in `candidate.rs` if needed
     to reuse its existing confinement authority rather than duplicating path
     safety in the new module;
   - remove a child root created by this call only when it is empty;
   - never remove a pre-existing root or the host data root;
   - cleanup failure returns `candidate_rollback_failed` and must not claim a
     clean refusal;
   - error messages disclose no absolute path.

10. Keep the boundary non-operational:

   - no trust, developer approval, publisher trust or revocation access;
   - no launch-profile preparation or provider process;
   - no conformance run or evidence;
   - no installation approval or installed record;
   - no enablement transition, policy, dispatch, replay, event, Anchor or Trail;
   - do not create `install`, `installed-records`, `enablements`, trust,
     conformance or approval paths.

11. Keep J24A through J24D behaviour unchanged. No CLI variant, route, envelope
    or public command is authorised in J24E.

## Relevant components

- `docs/architecture/J24E_CANDIDATE_PREPARATION_BLUEPRINT.md`
- `tethers-0.1/host-rust/src/package.rs`
- `tethers-0.1/host-rust/src/candidate.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/tests/j23c2_pdf_conformance.rs`
- `tethers-0.1/host-rust/tests/j23c3_installed_pdf_execution.rs`
- `tethers-0.1/host-rust/src/pdf_tools.rs`

## Frozen decisions and invariants

- Candidate identity is not installed identity.
- Candidate preparation grants no trust, approval, installation, permission or
  operational availability.
- Package inspection and quarantine perform no execution.
- Existing package and candidate record schemas remain unchanged.
- Existing low-level authorities remain sole validators.
- Exact replay may return a validated existing candidate but may not mutate it.
- Same release with different semantic evidence fails before extraction.
- Different raw archives may carry different detached signature evidence even
  when semantic package identity agrees.
- The host data root is caller-owned and pre-existing; J24E may create only its
  two named children.
- Supervision, conformance and installed publication are later boundaries.
- Tethers Core, OCaml semantics and public CLI remain untouched.
- No dependency, package format, manifest, capability identity, limit, trust,
  conformance, approval, installation or enablement contract change is
  authorised.

## Acceptance criteria

1. The new module exposes only the frozen service and result types and is
   exported through `lib.rs`.
2. A real deterministic PDF `.tetherplug` prepared with deliberately
   non-executable provider bytes succeeds, proving no provider launch is needed.
3. First preparation returns `Created`, publishes exactly one candidate JSON
   record and one immutable quarantine subtree, and creates no other lifecycle
   path.
4. Exact archive replay returns `Existing` with the same candidate ID and leaves
   every relative path and SHA-256 file digest unchanged.
5. Same PDF package release built from different provider bytes fails
   `semantic_conflict` before extraction and leaves the tree unchanged.
6. Malformed package input fails before `candidates/` or `quarantine/` is
   created.
7. Relative or missing package path and relative, missing, non-directory or
   unsafe host root fail closed without creating the host root or child paths.
8. Corrupt existing candidate evidence fails before extraction and changes
   nothing.
9. Rollback helpers remove only newly created confined candidate material and
   never delete pre-existing roots or the host root.
10. No install, installed-record, enablement, trust, conformance, approval,
    provider, policy, replay, event, Anchor or Trail state is created or touched.
11. Existing J24A, J24B, J24C and J24D tests remain green.
12. Full suite remains green apart from the five documented `pwsh.exe not found`
    environment failures.
13. Rustfmt, packet checker and `git diff --check` pass.

## Required verification

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 test candidate_preparation --locked
cargo +1.89.0 test --test j24e_candidate_preparation --locked
cargo +1.89.0 test --test j24a_plug_inspect_cli --locked
cargo +1.89.0 test --test j24b_plug_list_cli --locked
cargo +1.89.0 test --test j24c_plug_disable_cli --locked
cargo +1.89.0 test --test j24d_plug_enable_scope_file --locked
cargo +1.89.0 test --all-targets --all-features --locked
git diff --check
```

Use `pdf_tools::build_reference_package` with two different non-executable
provider byte strings for the successful and semantic-conflict fixtures. Use
relative-path and SHA-256 snapshots of the complete host tree.

## Permitted changes

Expected files are limited to:

- `tethers-0.1/host-rust/src/candidate_preparation.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/src/candidate.rs` only for one narrow `pub(crate)`
  confinement/rollback helper if required; do not change schemas or accepted
  extraction/registry behaviour
- `tethers-0.1/host-rust/tests/j24e_candidate_preparation.rs`
- `docs/worker-notes/2026-08-03-j24e-candidate-preparation-seam.md`
- `docs/CURRENT_CLINE_TASK.md` only for the authorised status transitions and
  final full implementation checkpoint

Not every permitted file must change. Stop before changing any other file.

## Forbidden changes

No CLI, `application.rs` or `plug_command.rs` change. No OCaml/Tether semantic
change. No dependency or lockfile change. No package format, manifest,
capability, provider, platform or archive-limit change. No trust, signature,
revocation, developer approval, launch profile, conformance, installation
approval, installed registry, enablement, removal, policy, dispatch, replay,
Trail, Anchor, release, tag or version change.

Do not add `plug stage`, `plug install`, `plug approve`, `plug conformance`,
`plug remove` or placeholders in J24E. Do not amend, rebase, reset, cherry-pick,
force-push or merge into `main`.

## Stop conditions

Stop cleanly and report the smallest unresolved question if:

- the implementation branch already exists;
- current `origin/main` lacks the accepted J24D checkpoint or J24E packet;
- composing the existing authorities requires weakening or duplicating package,
  quarantine or candidate validation;
- exact replay cannot be decided from existing immutable evidence;
- cleanup would require deleting a pre-existing path or recursively removing the
  host data root;
- candidate preparation would need trust, launch, conformance, installation or
  enablement access;
- a forbidden file, schema, dependency or architecture change appears necessary;
- branch-specific failures remain after two materially different attempts.

## Git and return contract

Create `opencode/j24e-candidate-preparation-seam` from current `origin/main`.
Use ordinary commits and normal push only. Do not rewrite history or merge into
`main`.

After all required checks pass, change the packet status to `COMPLETE`, record
one full 40-character implementation checkpoint, create the authorised worker
note and push normally.

Return the branch, final full SHA, exact changed files, concise implementation
summary, focused and complete test evidence, rustfmt, packet-checker and
`git diff --check` results, worker-note path, and explicit proof that candidate
preparation launched nothing, installed nothing, enabled nothing and exact
replay changed no byte.

## Expected pre-existing changes

None.
