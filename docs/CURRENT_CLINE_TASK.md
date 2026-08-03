# Current Implementation Task

Control contract: `1`
Task: `J24C - Explicit Plug disable CLI`
Owner: `OpenCode`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `OpenCode using DeepSeek Pro for bounded lifecycle mutation; Lucy performs final review`
Base branch: `main`
Base commit: `726c6aa780c6809fce32de39427200217cbad12f`
Implementation branch: `opencode/j24c-plug-disable-cli`
Worker note: `docs/worker-notes/2026-08-03-j24c-plug-disable-cli.md`

## Objective

Add the first public lifecycle mutation command:

```text
tethers-reference-host plug disable \
  --host-data-root <ABSOLUTE_PATH> \
  --installed-id <UUID>
```

The command appends one host-owned disablement transition for one exact,
currently enabled installed Plug. It must reuse the accepted installed and
enablement authorities, fail closed on inconsistent evidence, and change
nothing except the one new immutable enablement record.

This packet does not authorise installation, conformance, approval, enablement,
removal, provider launch, policy, replay, Trail, Anchor, package or language
changes.

## Relevant background and existing behaviour

J24A and J24B are accepted on `main`. `plug list` already validates the lifecycle
layout, installed records, enablement chains, latest-by-sequence state and exact
installed-versus-enablement pins. `EnablementStore::disable` already appends an
immutable disabled transition and refuses a Plug that is not currently enabled.

J24C must not create a second interpretation of lifecycle truth. Extract or
reuse one narrow shared reconciliation seam where necessary so list and disable
agree on the exact current installed/enabled identity.

The public lifecycle paths remain:

```text
install/
installed-records/
enablements/
```

## Required behaviour

1. Start from current `origin/main` after this packet is merged. Verify the
   worktree is clean, the base commit above is an ancestor, the packet names
   J24C/OpenCode/READY, and the implementation branch does not already exist.
   Create `opencode/j24c-plug-disable-cli` from current `origin/main`.

2. Add exactly:

   ```text
   plug disable --host-data-root <ABSOLUTE_PATH> --installed-id <UUID>
   ```

   Accept equals syntax. Reject missing or duplicate options, unknown options,
   extra positionals, non-absolute roots and malformed UUIDs. Preserve inspect,
   list and all prior routes.

3. Open only the existing host root and all three existing lifecycle stores.
   Missing root is unavailable. Missing/partial/unsafe lifecycle layout is
   invalid data. Do not create or repair any directory.

4. Validate before mutation:

   - load and validate installed records once;
   - locate exactly one record by installed ID;
   - load and chain-validate enablement records once;
   - select the highest sequence transition for that installed ID;
   - require the current transition to be `Enabled`;
   - require exact agreement with the installed record for package ID, semantic
     digest, provider ID/version, conformance digest, installation approval ID
     and complete capability bindings;
   - fail closed on unknown installed ID, absent enablement, already disabled,
     unknown-installed transitions, corrupt/forked chains or cross-record drift.

   Prefer one reusable pure reconciliation helper shared with `plug list` rather
   than copying J24B's comparison logic.

5. Append disablement through the existing `EnablementStore::disable` authority.
   Do not hand-build or directly write an `EnablementRecord`. Use the stable CLI
   authority string `tethers-reference-host-cli`; do not accept a public
   authority/person field or pretend the caller is Matthew.

6. On success emit one `tethers.cli/1` envelope:

   - command `plug disable`
   - status `ok`
   - exit `0`
   - data fields only:
     - `installed_id`
     - `package_id`
     - `state` exactly `disabled`
     - `sequence`
     - `record_digest`

   Do not expose scope, paths, authority, approval, trust, conformance,
   predecessor digest, timestamps, capabilities or internal record contents.

7. Failure mapping:

   - malformed CLI/non-absolute root/malformed UUID: `invalid_cli_usage`, exit 2;
   - missing/unreadable host root or ordinary store I/O: `unavailable`, exit 4;
   - unknown installed ID, partial/unsafe/corrupt layout, absent enablement,
     already disabled, chain conflict or cross-record mismatch: `invalid_data`,
     exit 3;
   - preserve stable underlying store codes where applicable;
   - use `installed_not_found` for an otherwise valid UUID absent from the
     installed registry.

8. Mutation boundary:

   - exactly one new canonical JSON file may appear under `enablements/`;
   - every pre-existing path and byte must remain unchanged;
   - no file may change under `install/` or `installed-records/`;
   - failed commands create no path and change no byte;
- no provider is launched or stopped because this host currently has no
  persistent provider-session registry; durable availability removal is the
  complete bounded J24C effect;
- no package, candidate, trust, conformance, approval, policy, replay, Trail
  or Anchor access.

## Relevant components

- `tethers-0.1/host-rust/src/cli.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/plug_command.rs`
- `tethers-0.1/host-rust/src/enablement.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
- `tethers-0.1/host-rust/tests/j24c_plug_disable_cli.rs`
- existing deterministic PDF package and lifecycle builders

## Frozen decisions and invariants

- J24A inspection and J24B listing remain unchanged.
- Installed registry validation remains owned by `InstalledPlugRegistry`.
- Enablement record and chain validation remain owned by `EnablementStore`.
- `EnablementStore::disable` is the only authorised disablement authority and
  appends an immutable disabled transition.
- Existing mutable store constructors retain their behaviour for authorised
  lifecycle writes.
- J24C introduces no generic repair, migration, optional validation or lenient
  loading mode.
- Disablement requires the target to be exactly installed, cross-record
  consistent and currently enabled.
- The CLI authority is `tethers-reference-host-cli` and is not caller-supplied.
- Failed commands create and change nothing; success creates exactly one new
  enablement JSON record.
- The CLI envelope remains `tethers.cli/1` with matching embedded/process exit.
- Tethers Core and OCaml syntax or semantics remain untouched.
- No dependency, package format, manifest, capability identity, archive limit,
  trust, conformance, approval, installation, enablement or security contract
  changes are authorised.

## Acceptance criteria

1. A real installed and enabled PDF Plug is disabled through the compiled binary.
2. The success envelope and real process exit agree and expose only authorised
   fields.
3. A subsequent compiled `plug list` reports the same installed ID as disabled.
4. The appended transition is sequence +1, predecessor-linked to the prior
   enabled record, validates, and has authority `tethers-reference-host-cli`.
5. Exactly one new enablement JSON record appears; all existing relative paths
   and SHA-256 digests are unchanged.
6. A second disable attempt fails with exit 3 and creates nothing.
7. Installed-but-never-enabled, unknown installed ID, valid cross-record drift,
   corrupt/forked chain, missing root and partial layout all fail closed without
   mutation.
8. Reversed UUID filename order cannot alter latest-by-sequence selection.
9. Existing J24A/J24B tests and the full suite remain green apart from the five
   documented `pwsh.exe not found` baseline failures.
10. Packet checker, rustfmt and `git diff --check` pass.

## Required verification

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 test cli --locked
cargo +1.89.0 test plug_command --locked
cargo +1.89.0 test --test j24a_plug_inspect_cli --locked
cargo +1.89.0 test --test j24b_plug_list_cli --locked
cargo +1.89.0 test --test j24c_plug_disable_cli --locked
cargo +1.89.0 test --all-targets --all-features --locked
git diff --check
```

## Permitted changes

- `tethers-0.1/host-rust/src/cli.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/plug_command.rs`
- `tethers-0.1/host-rust/src/enablement.rs`
- one narrowly named lifecycle projection module if extracting shared J24B logic
  is materially cleaner, plus `src/lib.rs` only to export it
- `tethers-0.1/host-rust/tests/j24c_plug_disable_cli.rs`
- `docs/worker-notes/2026-08-03-j24c-plug-disable-cli.md`
- `docs/CURRENT_CLINE_TASK.md` only for IN_PROGRESS/COMPLETE and checkpoint

Stop before changing any other file.

## Forbidden changes

No OCaml/Tether semantics, dependency/lockfile, package/manifest/capability
identity, archive, trust, conformance, approval, installation, enablement,
removal, provider launch/session, policy, dispatch, replay, Trail, Anchor,
architecture, release, tag or version change.

Do not add install, conformance, approve, enable or remove commands or stubs. Do
not delete branches. Do not amend, rebase, reset, cherry-pick, force-push or
merge into `main`.

## Stop conditions

Stop and report if the branch already exists; current main lacks the packet; the
existing disable authority cannot be used without weakening validation; exact
cross-record reconciliation cannot be shared or reused narrowly; more than one
new record must be written; provider/session behaviour would need invention; or
a forbidden file/contract change appears necessary.

## Git and return contract

Create the implementation branch from current `origin/main`. Use ordinary
commits and normal push only. After all checks pass, set status `COMPLETE`, record
the full implementation checkpoint and update the worker note.

Return branch, final SHA, exact files, implementation summary, focused/full test
results, packet/rustfmt/diff results, worker note, and proof that success wrote
exactly one disablement record while every failure wrote nothing.

## Expected pre-existing changes

None.
