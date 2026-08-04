# Current Implementation Task

Control contract: `1`
Task: `J24F - Public Plug stage CLI`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode using Luna for a thin public CLI adapter over the accepted J24E service; Lucy performs final review`
Base branch: `main`
Base commit: `9ceb7b2711bc387365b9a5382b84af1bb285384b`
Implementation branch: `opencode/j24f-plug-stage-cli`
Worker note: `docs/worker-notes/2026-08-03-j24f-plug-stage-cli.md`
Implementation blueprint: `docs/architecture/J24F_PLUG_STAGE_CLI_BLUEPRINT.md`
Implementation checkpoint: `191273ff5297c1d93f64c6c491c87fc5961e6ce1`

## Objective

Expose the accepted J24E candidate-preparation service through one public
command:

```text
tethers-reference-host plug stage \
  --host-data-root <ABSOLUTE_PATH> \
  --package <ABSOLUTE_TETHERPLUG_PATH>
```

J24F is deliberately a thin adapter. It owns strict Clap syntax, two
absolute-path CLI checks, stable `tethers.cli/1` envelope mapping and public
candidate formatting. All package inspection, quarantine, candidate identity,
exact replay, semantic conflict and rollback behaviour remains solely inside
`candidate_preparation::prepare_installation_candidate`.

Read the implementation blueprint in full before editing. It freezes the exact
output allowlist, error mapping and compiled-binary evidence.

## Relevant background and existing behaviour

J24E is accepted at
`9ceb7b2711bc387365b9a5382b84af1bb285384b`.

It provides
`candidate_preparation::prepare_installation_candidate`, which already owns:

- hostile package inspection;
- ordinary-file and safe-path validation;
- immutable quarantine extraction;
- candidate registry validation and publication;
- exact archive replay;
- semantic-conflict refusal;
- bounded rollback and cleanup.

J24F must remain a public CLI adapter over that accepted service. It must not
reimplement or weaken any J24E package, candidate, replay, quarantine or rollback
behaviour.

The existing public Plug commands are `inspect`, `list`, `enable` and `disable`.
They use strict Clap parsing, one-line `tethers.cli/1` envelopes and matching
process/envelope exit codes.

## Startup procedure

The current worktree may still be on an older implementation branch. Do not read
that branch's packet as current authority.

1. Confirm the worktree is clean. Stop if it is not.
2. Run `git fetch origin`.
3. Verify checkpoint `9ceb7b2711bc387365b9a5382b84af1bb285384b` is an ancestor of `origin/main`.
4. Inspect the first lines of the packet directly from `origin/main`:

   ```powershell
   git show origin/main:docs/CURRENT_CLINE_TASK.md | Select-Object -First 16
   ```

   Require J24F, OpenCode and `READY`.
5. Verify the blueprint directly from `origin/main`:

   ```powershell
   git cat-file -e origin/main:docs/architecture/J24F_PLUG_STAGE_CLI_BLUEPRINT.md
   ```

6. Check that `opencode/j24f-plug-stage-cli` does not exist locally or remotely.
   If it exists, stop without resetting or overwriting it.
7. Create and switch to it from current `origin/main`:

   ```powershell
   git switch --create opencode/j24f-plug-stage-cli origin/main
   ```

8. Read the checked-out packet and blueprint completely before editing.

## Required behaviour

1. Add exactly this `PlugCommand` variant:

   ```rust
   Stage {
       #[arg(long = "host-data-root", value_name = "ABSOLUTE_PATH")]
       host_data_root: PathBuf,
       #[arg(long = "package", value_name = "ABSOLUTE_TETHERPLUG_PATH")]
       package: PathBuf,
   }
   ```

2. Add one application route that calls:

   ```rust
   plug_command::run_stage(&host_data_root, &package)
   ```

3. Add one public adapter:

   ```rust
   pub fn run_stage(host_data_root: &Path, package_path: &Path) -> PlugCommandResult
   ```

4. Before calling J24E, perform only these CLI checks:

   - relative `host_data_root` returns `invalid_cli_usage`, exit `2`, code
     `invalid_cli_usage`, field `/host-data-root`;
   - relative `package_path` returns `invalid_cli_usage`, exit `2`, code
     `invalid_cli_usage`, field `/package`.

5. After those checks, call
   `candidate_preparation::prepare_installation_candidate` exactly once.

6. On success, emit command `plug stage`, status `ok`, exit `0` and exactly the
   public data shape frozen in the blueprint.

7. Map dispositions exactly:

   - `Created` to `created`;
   - `Existing` to `existing`.

8. Sort capabilities by `(name, version, operation)` before serialising them.

9. Preserve the J24E service's exact error code and message. Map status only by
   code:

   - `archive_read` or `candidate_io` to `unavailable`, exit `4`;
   - `candidate_rollback_failed` or `clock` to `failed`, exit `6`;
   - every other service code to `invalid_data`, exit `3`.

10. Emit exactly one JSON line for every parsed command. Process exit and
    envelope exit must agree.

11. Do not call package, candidate, registry or quarantine authorities directly.
    The production route must reach candidate preparation only through J24E.

12. Preserve J24A through J24E behaviour and tests.

## Relevant components

- `tethers-0.1/host-rust/src/cli.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/plug_command.rs`
- `tethers-0.1/host-rust/src/candidate_preparation.rs`
- `tethers-0.1/host-rust/tests/j24f_plug_stage_cli.rs`
- `docs/architecture/J24F_PLUG_STAGE_CLI_BLUEPRINT.md`
- `docs/CURRENT_CLINE_TASK.md`

## Frozen decisions and invariants

- J24E remains the sole candidate-preparation application authority.
- J24F performs only absolute-path CLI checks, one J24E call, public formatting
  and stable envelope mapping.
- Candidate identity remains distinct from installed identity.
- Staging grants no trust, approval, installation, permission or operational
  availability.
- Exact archive replay returns the same candidate identity and performs no
  mutation.
- Public output exposes only the blueprint allowlist.
- Absolute paths, quarantine locations, launch details, payload evidence,
  inspection evidence and internal record digests remain private.
- Package, candidate and quarantine schemas remain unchanged.
- Process exit code and envelope exit code must always agree.
- Tethers Core and OCaml language semantics remain untouched.

## Public output allowlist

The success `data` object contains exactly one key, `candidate`.

The candidate object contains exactly:

```text
candidate_id
disposition
state
package_id
package_version
semantic_package_digest
raw_archive_digest
provider_id
provider_version
platform
capabilities
created_unix_ms
```

`platform` contains exactly `os` and `architecture`.

Each capability contains exactly:

```text
name
version
manifest_digest
operation
```

Do not expose any absolute or quarantine path, payload evidence, launch details,
internal record digest, inspection digest, trust, approval, conformance,
installed or enablement evidence.

## Acceptance criteria

1. Strict CLI syntax accepts both split and `--name=value` forms and rejects
   missing, duplicate, unknown and positional extras.
2. A real deterministic PDF package made with non-executable provider bytes
   stages through the compiled binary and returns `created`.
3. The success envelope has the exact public allowlist and no forbidden field.
4. The output pins the exact PDF package/provider/platform and
   `pdf.inspect@1` capability.
5. First success creates exactly one candidate record and one immutable
   quarantine subtree, and no other lifecycle path.
6. Exact replay returns `existing`, the same candidate ID and changes no byte.
7. Malformed package returns exit `3`, `invalid_data` and its exact inspector
   code without creating candidate state.
8. Missing package returns exit `4`, `unavailable`, code `archive_read` and
   creates no candidate state.
9. Semantic conflict returns exit `3`, code `semantic_conflict` and changes no
   byte.
10. Corrupt candidate evidence returns exit `3`, code `record_invalid` and
    changes no byte.
11. Relative CLI paths return exit `2` with the exact field pointer before
    service mutation.
12. On Windows, a junction-backed package path maps to exit `3`, code
    `unsafe_destination` and creates no candidate state.
13. Every integration scenario proves process/envelope exit parity.
14. J24A through J24E focused tests remain green.
15. Full suite remains green apart from the five documented `pwsh.exe not found`
    environment failures.
16. Rustfmt, packet checker and `git diff --check` pass.

## Required verification

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 test cli --locked
cargo +1.89.0 test plug_command --locked
cargo +1.89.0 test --test j24f_plug_stage_cli --locked
cargo +1.89.0 test --test j24a_plug_inspect_cli --locked
cargo +1.89.0 test --test j24b_plug_list_cli --locked
cargo +1.89.0 test --test j24c_plug_disable_cli --locked
cargo +1.89.0 test --test j24d_plug_enable_scope_file --locked
cargo +1.89.0 test candidate_preparation --locked
cargo +1.89.0 test --test j24e_candidate_preparation --locked
cargo +1.89.0 test --all-targets --all-features --locked
git diff --check
```

## Permitted changes

Expected files are limited to:

- `tethers-0.1/host-rust/src/cli.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/plug_command.rs`
- `tethers-0.1/host-rust/tests/j24f_plug_stage_cli.rs`
- `docs/worker-notes/2026-08-03-j24f-plug-stage-cli.md`
- `docs/CURRENT_CLINE_TASK.md` only for status transitions and the final full
  implementation checkpoint

Stop before changing any other file.

## Forbidden changes

Do not modify `candidate_preparation.rs`, `candidate.rs`, `package.rs`, package
schemas, candidate schemas, dependencies or lockfiles.

No trust, signature, revocation, developer approval, launch, conformance,
installation approval, installed publication, enablement, removal, update,
download, registry, policy, replay, event, Anchor, Trail, OCaml, Tether syntax,
release, tag or version work.

Do not add `plug install`, `plug approve`, `plug conformance`, `plug remove` or
placeholders.

Do not amend, reset, rebase, cherry-pick, force-push or merge into `main`.

## Stop conditions

Stop cleanly and report the smallest unresolved question if:

- the implementation branch already exists;
- current `origin/main` lacks J24E or the J24F packet/blueprint;
- the CLI cannot remain a thin one-call adapter over J24E;
- a new package/candidate parser, registry read or filesystem mutation appears
  necessary;
- a forbidden file or architecture change appears necessary;
- branch-specific failures remain after two materially different attempts.

## Expected pre-existing changes

None.

## Git and return contract

Use ordinary commits and normal push only.

After all required checks pass:

- create the authorised worker note;
- set the packet to `COMPLETE`;
- record the full 40-character implementation checkpoint;
- push normally.

Return the branch, remote final SHA, implementation checkpoint, exact changed
files, focused and full test evidence, packet/rustfmt/diff results, worker-note
path, first-stage and exact-replay evidence, and explicit confirmation that the
CLI itself launched nothing, installed nothing and enabled nothing.
