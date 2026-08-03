# Current Implementation Task

Control contract: `1`
Task: `J24A - Read-only Plug inspection CLI`
Owner: `OpenCode`
Status: `READY`
Task colour: `Amber`
Route: `OpenCode using HY3 for bounded CLI integration; Lucy performs final review`
Base branch: `main`
Base commit: `f6454e64bde98d1c9d137fc395f287ba25bbe65a`
Implementation branch: `opencode/j24a-plug-inspect-cli`
Worker note: `docs/worker-notes/2026-08-03-j24a-plug-inspect-cli.md`

## Objective

Expose the existing host-owned, non-executing `.tetherplug` package inspector
through one public CLI command:

```text
tethers-reference-host plug inspect --package <PATH>
```

This packet authorises inspection only. It does not authorise installation,
extraction, quarantine, trust, approval, enablement, provider launch, execution,
disablement, removal, release work, or architecture changes.

## Relevant background and existing behaviour

`package::inspect(path)` is already the sole package-inspection authority. It
reads a `.tetherplug` as hostile data, validates the fixed archive and package
profile, verifies payload and manifest evidence, and returns an
`InspectionReport`. Its contract explicitly forbids writing, extraction,
launch, binding, or runtime mutation.

The public CLI currently exposes `check`, `run`, and `trail` plus hidden
administrative and debug routes. `application::run` owns command routing,
envelope emission, and process exit. Command modules return a `CliEnvelope` and
matching exit code rather than exiting directly.

J23C3 is accepted and merged. `main` and
`opencode/j23c3-installed-pdf-execution` both point to the base commit above.

## Required behaviour

1. Add one public nested clap route exactly shaped as:

   ```text
   plug inspect --package <PATH>
   ```

   Accept `--package=<PATH>`. Reject a missing or duplicate `--package`, unknown
   options, extra positional arguments, and `plug` without a subcommand. Preserve
   all existing command behaviour.

2. Add one small command adapter, preferably
   `tethers-0.1/host-rust/src/plug_command.rs`, following the existing
   `check_command` and `run_command` pattern. It must expose a callable function
   equivalent to:

   ```rust
   run_inspect(package_path: &Path) -> PlugCommandResult
   ```

   The result contains one `CliEnvelope` and the matching process exit code. The
   command module must not call `std::process::exit`.

3. Call `package::inspect` exactly once. Do not create a second parser, ZIP
   reader, validator, report builder, or package format interpretation.

4. On success, emit the existing `tethers.cli/1` envelope with:

   - command: `plug inspect`
   - status: `ok`
   - exit_code: `0`
   - error absent
   - `data.inspection` containing the complete public `InspectionReport`
     evidence.

   The public inspection JSON must include:

   - `inspection_format_version`
   - `inspection_evidence_digest`
   - `package`
   - `raw_archive_digest`
   - `raw_archive_size`
   - `provider_id`
   - `provider_version`
   - `provider_launch_path`
   - `provider_launch_arguments`
   - `provider_working_directory`
   - `provider_operation_namespace`
   - `selected_platform`
   - `plug_json`
   - `payloads`
   - `capabilities`
   - `signature_files`
   - `signatures_present`

   The private filesystem `archive_path` must not appear in serialised output.
   Prefer deriving or implementing `Serialize` for `InspectionReport` and
   skipping only that private field. Preserve the existing `archive_path()`
   accessor.

5. Map failures without changing their meaning:

   - `PackageError.code == "archive_read"` -> status `unavailable`, exit code `4`;
   - every other `PackageError` -> status `invalid_data`, exit code `3`.

   Preserve the existing package error code in `error.code` and use the package
   error message in `error.message`. Do not emit Rust debug formatting, stack
   information, or a newly canonicalised absolute path.

6. Route the command through `application::run` and the existing
   `emit_envelope_and_exit` boundary. Do not alter legacy execution, event
   probes, replay provisioning, Trail, Check, or Run behaviour.

7. The command remains strictly read-only. It must not create directories,
   extract files, create scratch space, write Trail records, create candidates,
   access trust stores, create approvals, create installed or enablement records,
   launch providers, or mutate runtime configuration.

## Relevant components

- `tethers-0.1/host-rust/src/cli.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/src/package.rs`
- `tethers-0.1/host-rust/src/check_command.rs`
- `tethers-0.1/host-rust/src/run_command.rs`
- existing deterministic PDF `.tetherplug` package builder and fixtures

## Frozen decisions and invariants

- `package::inspect` remains the only inspection authority.
- This is a read-only CLI adapter, not a lifecycle implementation.
- The envelope schema remains `tethers.cli/1`.
- Existing package validation rules, archive limits, identities, versions,
  digests, manifests, and capability contracts do not change.
- Existing CLI routes and exit-code vocabulary remain unchanged.
- Tethers Core and OCaml semantics remain untouched.
- No dependency or lockfile change is authorised.
- Do not add lifecycle placeholders or speculative abstractions.

## Acceptance criteria

1. Exact clap syntax succeeds, including `--package=<PATH>`, and malformed command
   shapes are rejected.
2. A valid deterministic PDF `.tetherplug` returns exit code `0` and one valid
   JSON envelope.
3. Success evidence reports package ID `tethers.pdf-tools`, package version
   `1.0.0`, provider ID `tethers-pdf-provider`, and capability `pdf.inspect`
   version `1`.
4. `inspection_evidence_digest` is a 71-character lowercase SHA-256 value.
5. `archive_path` is absent from serialised output.
6. An invalid extension returns status `invalid_data`, exit code `3`, and error
   code `invalid_archive`.
7. A missing package returns status `unavailable`, exit code `4`, and error code
   `archive_read`.
8. Inspection leaves the source bytes unchanged and creates no additional
   filesystem entries.
9. The compiled binary's real process exit code matches the embedded envelope
   exit code.
10. Existing CLI parsing and regression tests remain green apart from the five
    documented `execution_environment` PowerShell failures when `pwsh.exe` is
    unavailable.
11. The worker note exists at the exact authorised path and records the actual
    branch, commit, files, tests, and any baseline failures.

## Required verification

```powershell
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 test cli --locked
cargo +1.89.0 test plug_command --locked
cargo +1.89.0 test --test j24a_plug_inspect_cli --locked
cargo +1.89.0 test --all-targets --all-features --locked
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
git diff --check
```

Use the existing deterministic PDF package builder rather than inventing another
package format or fixture model.

## Permitted changes

Expected implementation files are limited to:

- `tethers-0.1/host-rust/src/cli.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/src/package.rs`
- `tethers-0.1/host-rust/src/plug_command.rs`
- `tethers-0.1/host-rust/tests/j24a_plug_inspect_cli.rs`
- `docs/worker-notes/2026-08-03-j24a-plug-inspect-cli.md`

Not every listed file must change. Stop and report before changing any other
file.

## Forbidden changes

No OCaml or Tether syntax/semantic change; no manifest, package identity,
provider identity, capability version, package-validation, archive-limit,
trust, conformance, installation, approval, enablement, launch, dispatch,
policy, replay, Trail, runtime-configuration, dependency, `Cargo.toml`,
`Cargo.lock`, architecture, project-control, release, tag, or version change.

Do not add install, list, approve, enable, disable, remove, or lifecycle stubs.
Do not refactor unrelated CLI code. Do not amend, rebase after publishing,
force-push, merge, tag, or release.

## Stop conditions

Stop cleanly and report one smallest unresolved question if:

- the command cannot be implemented as a thin adapter around `package::inspect`;
- serialising the public report requires exposing `archive_path` or duplicating
  package semantics;
- the required binary test needs package or lifecycle mutation;
- an existing CLI contract must change;
- a dependency, lockfile, manifest, package identity, or architecture change
  appears necessary;
- branch-specific failures remain after two materially different attempts.

## Git and return contract

Create `opencode/j24a-plug-inspect-cli` from the exact base commit. Make one
normal commit with suggested message:

```text
J24A: add read-only Plug inspection CLI
```

Push normally. Do not amend or force-push. Do not merge.

Return the branch name, full commit SHA, exact files changed, concise
implementation summary, focused test results, complete-suite result, rustfmt and
`git diff --check` results, confirmation of the worker-note path, and explicit
confirmation that no package, lifecycle, or runtime mutation was introduced.

## Expected pre-existing changes

None.
