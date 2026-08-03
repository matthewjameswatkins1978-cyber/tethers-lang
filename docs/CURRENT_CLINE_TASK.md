# Current Implementation Task

Control contract: `1`
Task: `J24D - Permission-file Plug enable CLI`
Owner: `OpenCode`
Status: `READY`
Task colour: `Amber`
Route: `OpenCode using DeepSeek Pro V4 for permission parsing and bounded lifecycle mutation; Lucy performs final review`
Base branch: `main`
Base commit: `fb354dea734e7a2d37254a9cfbca4fd0daad5939`
Implementation branch: `opencode/j24d-plug-enable-scope-file`
Worker note: `docs/worker-notes/2026-08-03-j24d-plug-enable-scope-file.md`

## Objective

Add one explicit permission-file-based enable command:

```text
tethers-reference-host plug enable \
  --host-data-root <ABSOLUTE_PATH> \
  --installed-id <UUID> \
  --scope <ABSOLUTE_JSON_PATH>
```

The scope file is the stable human/automation-facing permission request. The
host validates it, binds it to the exact installed Plug, constructs host-owned
operational-scope evidence and appends one immutable enabled transition through
the existing `EnablementStore::enable` authority.

J24D supports only the accepted PDF Plug and `pdf.inspect@1`. It establishes the
permission-file foundation without inventing a general permissions language or
adding Plug-specific command flags.

## Relevant background and existing behaviour

J24A inspection, J24B listing and J24C explicit disablement are accepted on
`main`. Installed-registry and enablement-chain validation already provide the
sole lifecycle authorities. `EnablementRecord::consistent_with` and latest-by-
sequence selection already reconcile installed and current enablement truth.

`PdfOperationalScopeBinding::create` already owns canonical PDF scope creation.
It validates an existing absolute directory, a maximum byte limit in
`1..=67108864`, exact `pdf.inspect@1` identity, authority and integrity digest.
`EnablementStore::enable` already appends one immutable enabled transition and
refuses an already-enabled Plug.

The permission request file is not operational-scope evidence and must not ask
the user to manufacture internal fields. It contains exactly:

```json
{
  "schema": "tethers.plug-scope/1",
  "capability": {
    "name": "pdf.inspect",
    "version": 1
  },
  "permissions": {
    "query_root": "C:\\Documents",
    "max_bytes": 20971520
  }
}
```

The host supplies the installed ID, fixed authority
`tethers-reference-host-cli`, canonical path and integrity digest.

## Required behaviour

1. Start from current `origin/main` after this packet is merged. Verify the
   worktree is clean, base commit above is an ancestor, the packet names
   J24D/OpenCode/READY, and the implementation branch does not already exist.
   Create `opencode/j24d-plug-enable-scope-file` from current `origin/main`.

2. Add exactly:

   ```text
   plug enable --host-data-root <ABSOLUTE_PATH> --installed-id <UUID> --scope <ABSOLUTE_JSON_PATH>
   ```

   Accept equals syntax. Reject missing or duplicate options, unknown options,
   extra positionals, non-absolute host/scope paths and malformed UUIDs.
   Preserve inspect, list, disable and all prior routes.

3. Parse the permission request as hostile input:

   - maximum file size: 16 KiB;
   - UTF-8 JSON object only;
   - `serde(deny_unknown_fields)` or equally exact parsing;
   - exact schema `tethers.plug-scope/1`;
   - exact capability `pdf.inspect@1`;
   - exact permission fields `query_root` and `max_bytes`;
   - `query_root` must be an absolute JSON string path;
   - `max_bytes` must be an exact positive integer no greater than 67108864;
   - reject floats, negative values, overflow, duplicate JSON keys, trailing
     content, BOM, unknown fields and alternate spellings;
   - do not canonicalise or reveal the scope-file path in an error message.

   Prefer one narrowly named request type in `plug_command.rs` or a small
   `plug_scope.rs` module. Do not add a generic schema framework.

4. Validate all lifecycle and permission evidence before mutation:

   - require an existing ordinary host root and complete ordinary
     `install/`, `installed-records/`, `enablements/` layout;
   - load and validate installed records once;
   - locate exactly one installed record by installed ID;
   - require package ID `tethers.pdf-tools`, provider ID
     `tethers-pdf-provider` and capability `pdf.inspect@1`;
   - load and chain-validate enablement records once;
   - reject unknown-installed transitions and cross-record drift;
   - select current state by greatest sequence;
   - reject an already-enabled target;
   - allow a never-enabled or currently-disabled target;
   - parse and validate the permission request;
   - call `PdfOperationalScopeBinding::create` exactly once with the target
     installed ID, request `query_root`, request `max_bytes`, and authority
     `tethers-reference-host-cli`;
   - do not hand-build operational scope evidence or its digest.

5. Append enablement only through:

   ```rust
   EnablementStore::enable(
       installed_record,
       OperationalScope::Pdf(binding),
       "tethers-reference-host-cli"
   )
   ```

   Do not directly create or write an `EnablementRecord`.

6. On success emit one `tethers.cli/1` envelope:

   - command `plug enable`;
   - status `ok`;
   - exit `0`;
   - data fields only:
     - `installed_id`
     - `package_id`
     - `state`, exactly `enabled`
     - `sequence`
     - `record_digest`
     - `scope_digest`

   Do not expose the query root, max bytes, scope-file path, authority,
   predecessor digest, capabilities, trust, approval, conformance, timestamps or
   internal paths.

7. Failure mapping:

   - malformed CLI, non-absolute paths or malformed UUID:
     `invalid_cli_usage`, exit 2;
   - missing/unreadable host root, permission file or ordinary store I/O:
     `unavailable`, exit 4;
   - malformed/oversized/unsupported permission request:
     `invalid_data`, exit 3, stable code `scope_request_invalid`;
   - unknown installed ID: `invalid_data`, exit 3, `installed_not_found`;
   - unsupported installed Plug/capability: `invalid_data`, exit 3,
     `scope_unsupported`;
   - partial/unsafe/corrupt lifecycle layout, already enabled, chain conflict or
     cross-record mismatch: `invalid_data`, exit 3;
   - preserve stable underlying store codes where applicable;
   - never include raw JSON, absolute paths or debug formatting in errors.

8. Mutation boundary:

   - success creates exactly one new canonical JSON file under `enablements/`;
   - every pre-existing path and byte remains unchanged;
   - permission request file remains byte-identical;
   - no file changes under `install/` or `installed-records/`;
   - every failure creates no path and changes no byte;
   - no provider launch, package inspection/extraction, candidate, trust,
     conformance, approval, policy, replay, Trail or Anchor access.

## Relevant components

- `tethers-0.1/host-rust/src/cli.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/plug_command.rs`
- `tethers-0.1/host-rust/src/enablement.rs`
- `tethers-0.1/host-rust/src/operational_scope.rs`
- `tethers-0.1/host-rust/src/pdf_tools.rs`
- `tethers-0.1/host-rust/tests/j24c_plug_disable_cli.rs`
- existing deterministic PDF package and lifecycle builders

## Frozen decisions and invariants

- The permission file is the stable public input; friendly flags may generate it
  in a later task but never become a second permission authority.
- The request file describes permission intent only. Installed ID, authority,
  canonicalisation and integrity evidence remain host-owned.
- J24D supports only `tethers.pdf-tools` / `pdf.inspect@1`.
- Installed and enablement validation retain their existing sole authorities.
- `PdfOperationalScopeBinding::create` remains the sole PDF scope constructor.
- `EnablementStore::enable` remains the sole enablement writer.
- Never-enabled and currently-disabled Plugs may be enabled; currently-enabled
  Plugs fail closed rather than writing an idempotent duplicate.
- Success appends exactly one record. Failure is completely non-mutating.
- The CLI envelope remains `tethers.cli/1` with matching process/envelope exit.
- Tethers Core and OCaml syntax or semantics remain untouched.
- No dependency, package format, manifest, capability identity, archive limit,
  trust, conformance, approval, installation or security-contract change is
  authorised.

## Acceptance criteria

1. Exact command and equals syntax succeed; malformed variants fail with exit 2.
2. A real installed but never-enabled PDF Plug is enabled through the compiled
   binary from a valid permission file.
3. A previously enabled then disabled PDF Plug is re-enabled with sequence +1
   and correct predecessor linkage.
4. Success creates exactly one enablement JSON record, validates, uses authority
   `tethers-reference-host-cli`, and embeds a PDF scope matching the canonical
   requested root and exact max bytes.
5. The success envelope/process exits agree and expose only the authorised six
   data fields without revealing permission values or paths.
6. A subsequent compiled `plug list` reports the same installed ID as enabled.
7. Already enabled, unknown installed ID, unsupported Plug/capability,
   malformed/oversized request, duplicate JSON keys, missing permission file,
   absent query root, partial layout, valid cross-record drift and corrupt chain
   all fail closed without mutation.
8. Reversed UUID filename ordering cannot alter current-state selection.
9. Recursive relative-path and SHA-256 snapshots prove success changes only one
   new enablement file and every failure changes nothing. The request file is
   unchanged.
10. J24A/J24B/J24C and full-suite tests remain green apart from the five
    documented `pwsh.exe not found` environment failures.
11. Packet checker, rustfmt and `git diff --check` pass.

## Required verification

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 test cli --locked
cargo +1.89.0 test plug_command --locked
cargo +1.89.0 test --test j24a_plug_inspect_cli --locked
cargo +1.89.0 test --test j24b_plug_list_cli --locked
cargo +1.89.0 test --test j24c_plug_disable_cli --locked
cargo +1.89.0 test --test j24d_plug_enable_scope_file --locked
cargo +1.89.0 test --all-targets --all-features --locked
git diff --check
```

## Permitted changes

- `tethers-0.1/host-rust/src/cli.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/plug_command.rs`
- optional `tethers-0.1/host-rust/src/plug_scope.rs` plus `src/lib.rs` only if a
  dedicated exact request parser is materially cleaner
- `tethers-0.1/host-rust/tests/j24d_plug_enable_scope_file.rs`
- `docs/worker-notes/2026-08-03-j24d-plug-enable-scope-file.md`
- `docs/CURRENT_CLINE_TASK.md` only for IN_PROGRESS/COMPLETE and checkpoint

Stop before changing any other file.

## Forbidden changes

No OCaml/Tether semantics; dependency or lockfile; generic permissions language;
package, manifest, capability or provider identity; archive; trust;
conformance; approval; installation; disablement authority; provider launch or
session; policy; dispatch; replay; Trail; Anchor; architecture; release; tag or
version change.

Do not add install, conformance, approve, remove or friendly permission flags.
Do not add enablement support for File Tools or arbitrary Plugs. Do not delete
branches. Do not amend, rebase, reset, cherry-pick, force-push or merge into
`main`.

## Stop conditions

Stop and report if the branch already exists; current main lacks this packet;
exact duplicate-key rejection cannot be implemented narrowly without a new
dependency; the existing PDF scope constructor or enable authority cannot be
used unchanged; more than one new record must be written; support for another
Plug would be required; or a forbidden file/contract change appears necessary.

## Git and return contract

Create the implementation branch from current `origin/main`. Use normal commits
and normal push only. After all checks pass, set status `COMPLETE`, record the
full implementation checkpoint and update the worker note.

Return branch, final SHA, exact files, implementation summary, focused/full test
results, packet/rustfmt/diff results, worker note, and proof that success wrote
exactly one enablement record while every failure wrote nothing.

## Expected pre-existing changes

None.
