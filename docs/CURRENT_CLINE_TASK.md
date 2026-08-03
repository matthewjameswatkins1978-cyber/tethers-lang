# Current Implementation Task

Control contract: `1`
Task: `J24B - Read-only Plug list CLI`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode using DeepSeek Pro for cross-store read-only CLI integration; Lucy performs final review`
Base branch: `main`
Base commit: `13f6a3caffa00904f6357c7975a8a0937a6c2d5c`
Implementation branch: `opencode/j24b-plug-list-cli`
Worker note: `docs/worker-notes/2026-08-03-j24b-plug-list-cli.md`
Implementation checkpoint: `bfd0c220fd2e05978a3c380d0c714421a423d08a`

## Objective

Add one public, strictly read-only Plug lifecycle command:

```text
tethers-reference-host plug list --host-data-root <ABSOLUTE_PATH>
```

The command reports the host's validated installed Plug identities and their
current enabled or disabled state. It combines existing installed-registry and
enablement authority without creating, repairing, installing, approving,
enabling, disabling, launching or removing anything.

This is the second bounded P19 CLI slice after J24A inspection. It does not
authorise any lifecycle mutation.

## Relevant background and existing behaviour

J24A is accepted and merged at the base commit above. The public CLI now exposes
`plug inspect` through `plug_command.rs` and the existing `tethers.cli/1`
envelope boundary.

`InstalledPlugRegistry::load_all` already validates installed records, exact
payload sets, payload hashes, read-only attributes, release uniqueness and
installation-root containment. `EnablementStore::load_all` already validates
immutable transition records and complete predecessor chains. Installed records
remain `present_disabled`; only the latest valid enablement transition determines
whether one exact installed identity is operationally enabled.

The existing mutable constructors use `StoreRoot::open`, which creates missing
directories. A list command must not use that behaviour. J24B may add an
explicit existing-only constructor that verifies and opens an already-present
store without calling `create_dir_all` or performing any write.

The first public lifecycle layout under `--host-data-root` is:

```text
install/
installed-records/
enablements/
```

These names match the accepted J23C3 lifecycle fixture. J24B freezes only these
three first-slice paths for public lifecycle commands. It does not reorganise
other host data or replay storage.

## Required behaviour

1. Start from current `origin/main` after this task packet is merged:

   - run `git fetch origin`;
   - verify the worktree is clean;
   - verify base commit `13f6a3caffa00904f6357c7975a8a0937a6c2d5c`
     is an ancestor of `origin/main`;
   - verify `docs/CURRENT_CLINE_TASK.md` names J24B, OpenCode and `READY`;
   - create `opencode/j24b-plug-list-cli` from current `origin/main`;
   - do not require branch HEAD to equal the base commit, because the authorised
     task-packet commit is an expected planning descendant;
   - if the implementation branch already exists locally or remotely, stop and
     report it rather than resetting or overwriting it.

2. Add one nested public command exactly shaped as:

   ```text
   plug list --host-data-root <ABSOLUTE_PATH>
   ```

   Accept `--host-data-root=<PATH>`. Reject a missing or duplicate option,
   unknown options, extra positional arguments, and non-absolute paths. Preserve
   `plug inspect` and every existing public and hidden CLI route.

3. Open lifecycle state without mutation:

   - keep existing mutable `StoreRoot::open` behaviour unchanged;
   - add a narrowly named existing-only StoreRoot constructor that requires an
     existing absolute directory, verifies the complete non-reparse path chain,
     canonicalises it, and never creates a directory or file;
   - expose matching existing-only constructors for `InstalledPlugRegistry` and
     `EnablementStore`, or an equally narrow adapter that reuses their existing
     validation and loading authority;
   - do not add a second JSON parser, record validator, chain validator, payload
     verifier or registry implementation.

4. Resolve the three lifecycle children below the supplied host root:

   - `install`
   - `installed-records`
   - `enablements`

   Behaviour is exact:

   - if none of the three paths exists, return a successful empty list and
     create nothing;
   - if all three exist as ordinary directories, validate and load them;
   - if only some exist, or any is not an ordinary directory, fail closed as
     invalid data;
   - symbolic links and Windows reparse points are refused;
   - a missing or non-directory host root is unavailable and is never created.

5. Derive current lifecycle truth from validated records:

   - load installed records once;
   - load and chain-validate enablement records once;
   - choose the highest sequence record for each installed identity;
   - no enablement record means `disabled`;
   - latest `EnablementState::Enabled` means `enabled`;
   - latest `EnablementState::Disabled` means `disabled`;
   - an enablement record for an unknown installed identity fails closed;
   - the latest transition's package ID, semantic package digest, provider ID,
     provider version, conformance digest, installation approval ID and exact
     capability set must agree with the installed record, otherwise fail closed;
   - do not infer, repair or discard conflicting evidence.

6. Emit one stable `tethers.cli/1` envelope:

   - command: `plug list`
   - status: `ok`
   - exit code: `0`
   - error absent
   - data contains `count` and `plugs`

   Each item contains only:

   - `installed_id`
   - `package_id`
   - `package_version`
   - `semantic_package_digest`
   - `provider_id`
   - `provider_version`
   - `state`, exactly `enabled` or `disabled`
   - `capabilities`, each containing `name`, `version`, `manifest_digest` and
     `provider_operation_name`
   - `created_unix_ms`

   Sort Plug items by package ID, package version, then installed ID. Sort each
   capability list by name then version. Do not expose installation paths,
   operational scope paths, trust records, authorities, approval evidence,
   conformance records, transition history or internal store paths.

7. Map failures consistently:

   - non-absolute `--host-data-root`: `invalid_cli_usage`, exit `2`, field
     `/host-data-root`;
   - missing or unreadable host root and ordinary store I/O failures:
     `unavailable`, exit `4`;
   - partial layout, unsafe path, corrupt record, chain conflict, unknown
     installed identity or cross-record mismatch: `invalid_data`, exit `3`;
   - preserve the underlying stable store error code where one exists;
   - use stable J24B codes `plug_data_root_unavailable` and
     `plug_store_incomplete` for the two command-owned cases above;
   - never emit debug formatting, a stack, raw record contents or a newly
     disclosed absolute path.

8. Keep the command strictly observational:

   - no `create_dir_all`, file creation, temporary file, repair, migration or
     normalisation;
   - no package inspection or extraction;
   - no candidate, trust, conformance, approval, installation, enablement,
     disablement or removal write;
   - no provider launch, discovery, policy, dispatch, replay, Trail or Anchor;
   - no update of access-independent application state;
   - the filesystem entry set and file bytes beneath the supplied host root must
     be identical before and after the command.

## Relevant components

- `tethers-0.1/host-rust/src/cli.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/plug_command.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/enablement.rs`
- `tethers-0.1/host-rust/tests/j23c3_installed_pdf_execution.rs`
- existing deterministic PDF package and lifecycle builders

## Frozen decisions and invariants

- J24A inspection remains unchanged.
- Installed registry validation remains owned by `InstalledPlugRegistry`.
- Enablement record and chain validation remain owned by `EnablementStore`.
- Existing mutable store constructors retain their behaviour for authorised
  lifecycle writes.
- J24B introduces no generic repair, migration, optional validation or lenient
  loading mode.
- Installed material alone is disabled. Only the latest exact valid enablement
  transition can report enabled.
- Listing does not establish readiness, policy permission or provider health.
- Empty state is successful only when all three lifecycle child paths are absent.
- Partial state is never presented as an empty or partially trusted result.
- The CLI envelope remains `tethers.cli/1` with matching embedded/process exit.
- Tethers Core and OCaml syntax or semantics remain untouched.
- No dependency, package format, manifest, capability identity, archive limit,
  trust, conformance, approval, installation, enablement or security contract
  changes are authorised.

## Acceptance criteria

1. The branch starts from current `origin/main`, retains the accepted J24A
   history, and contains no unrelated or rewritten commits.
2. Exact `plug list --host-data-root <ABSOLUTE_PATH>` syntax and equals syntax
   succeed; malformed forms and non-absolute roots are rejected correctly.
3. An existing empty host root with none of the three lifecycle children returns
   status `ok`, exit `0`, `count: 0`, `plugs: []`, and creates no child paths.
4. A partial lifecycle layout returns status `invalid_data`, exit `3`, and error
   code `plug_store_incomplete` without changing the layout.
5. A missing host root returns status `unavailable`, exit `4`, and error code
   `plug_data_root_unavailable` without creating it.
6. Existing-only store opening performs no directory or file creation and
   refuses symlink/reparse paths.
7. A real installed PDF Plug with no enablement is listed as `disabled` with the
   exact package, provider and capability identity.
8. The same Plug with a current enabled transition is listed as `enabled`; after
   a valid disable transition it is listed as `disabled`.
9. Unknown-installed enablement evidence and any required cross-record mismatch
   fail closed as invalid data rather than being omitted or repaired.
10. Output contains only the authorised fields, has stable Plug and capability
    ordering, and contains no absolute path, operational scope, authority, trust,
    approval, conformance or transition-history field.
11. The compiled binary emits one valid JSON envelope and the real process exit
    code matches the embedded exit code for success and failure cases.
12. Snapshot evidence proves filesystem entries and file bytes below the host
    root are unchanged by every list invocation.
13. Existing J24A CLI tests and all prior Rust tests remain green apart from the
    five documented `execution_environment` failures when `pwsh.exe` is
    unavailable.
14. The task packet checker passes and the worker note records the exact branch,
    final full SHA, files, focused/full test evidence and baseline failures.

## Required verification

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 test cli --locked
cargo +1.89.0 test plug_command --locked
cargo +1.89.0 test --test j24a_plug_inspect_cli --locked
cargo +1.89.0 test --test j24b_plug_list_cli --locked
cargo +1.89.0 test --all-targets --all-features --locked
git diff --check
```

Use the existing deterministic PDF package builder and lifecycle stores for
integration evidence. Test setup may create disposable lifecycle state; the
`plug list` command itself may not mutate it.

## Permitted changes

Expected files are limited to:

- `tethers-0.1/host-rust/src/cli.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/plug_command.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/enablement.rs`
- `tethers-0.1/host-rust/tests/j24b_plug_list_cli.rs`
- `docs/worker-notes/2026-08-03-j24b-plug-list-cli.md`
- `docs/CURRENT_CLINE_TASK.md` only for the authorised final transition from
  `READY` or `IN_PROGRESS` to `COMPLETE` and the final full implementation
  checkpoint

Not every listed file must change. Stop and report before changing any other
file.

## Forbidden changes

No OCaml or Tether syntax/semantic change; no package parser or format change;
no manifest, capability, provider or package identity/version change; no archive
limit change; no trust, conformance, approval, installation, enablement,
disablement, scope, policy, dispatch, replay, Trail, Anchor or provider-launch
behaviour change; no dependency, `Cargo.toml`, `Cargo.lock`, architecture,
project-control redesign, release, tag or version change.

Do not add `plug install`, `plug conformance`, `plug approve`, `plug enable`,
`plug disable`, `plug remove` or placeholders for them. Do not refactor unrelated
CLI or store code. Do not amend, rebase, reset, cherry-pick, force-push, merge
into `main`, tag or release.

## Stop conditions

Stop cleanly and report one smallest unresolved question if:

- the implementation branch already exists;
- current `origin/main` does not contain the named base commit or J24B packet;
- read-only listing requires use of a constructor that may create or rewrite
  state and an existing-only seam cannot be added narrowly;
- installed and enablement truth cannot be reconciled without changing their
  frozen record contracts;
- a required output field would expose an absolute path, operational scope,
  credential, trust authority or unreviewed record content;
- a dependency, lockfile, package, manifest, capability or architecture change
  appears necessary;
- branch-specific failures remain after two materially different attempts.

## Git and return contract

After this packet is merged to `main`, create
`opencode/j24b-plug-list-cli` from current `origin/main`. The packet's Base
commit is the accepted J24A product checkpoint; the later task-packet commit is
an expected planning descendant and must remain in branch history.

Make ordinary commits and push normally. Do not amend, rebase, reset,
cherry-pick or force-push. Do not merge into `main`.

After all required checks pass, change the packet status to `COMPLETE`, add one
full 40-character `Implementation checkpoint`, create the authorised worker note
and push the final branch.

Return the branch name, final full commit SHA, exact files changed, concise
implementation summary, focused test results, complete-suite result, rustfmt,
packet-checker and `git diff --check` results, worker-note path, and explicit
confirmation that listing created, changed or repaired no lifecycle state.

## Expected pre-existing changes

None.
