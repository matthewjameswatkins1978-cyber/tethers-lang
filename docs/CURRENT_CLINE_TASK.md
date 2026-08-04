# Current Implementation Task

Control contract: `1`
Task: `J24G - Strict Plug installation request contract`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode using Luna for a bounded JSON contract and parser; Lucy performs final review`
Base branch: `main`
Base commit: `f5e621bee4338a496888daaf78e2f029e4ab0914`
Implementation branch: `opencode/j24g-installation-request-contract`
Implementation checkpoint: `fa3ffcf4f7c8e96c0a7f5e2b3f8d7a9c6b1e4d2f`
Worker note: `docs/worker-notes/2026-08-04-j24g-installation-request-contract.md`
Implementation blueprint: `docs/architecture/J24G_INSTALLATION_REQUEST_CONTRACT.md`

## Objective

Implement the exact, hostile-input-safe request contract for the future public
Plug installation command.

J24G turns one small JSON file into a typed `InstallationRequest` expressing
only:

- one exact candidate identity;
- exact-candidate trust;
- explicit permission for non-isolated supervised conformance execution;
- installation to the disabled state only.

J24G performs no candidate lookup, planning, trust mutation, provider launch,
conformance, approval, payload copying, installed publication, enablement, or
CLI work.

Read `docs/architecture/J24G_INSTALLATION_REQUEST_CONTRACT.md` completely before
editing. It freezes the public JSON, Rust seam, error codes, messages, field
pointers, validation order, and evidence matrix.

## Relevant background and existing behaviour

J24E and J24F are accepted on `main`. Together they provide a safe public
package-intake boundary:

- `plug stage` inspects a hostile `.tetherplug`;
- the package is extracted into immutable quarantine;
- one candidate record is published;
- exact replay returns the same candidate identity without mutation;
- staging grants no trust, approval, installation, permission, or operational
  availability.

The future installation pipeline is frozen as separate internal gates behind
one simple public operation:

```text
request
→ read-only reconciliation plan
→ host installation lock and replan
→ exact-candidate trust
→ supervised conformance
→ installation approval
→ atomic installed publication
→ present disabled
```

J24G owns only the request boundary. J24H will later consume its typed output in
a read-only reconciliation planner.

The repository already provides
`crate::manifest::parse_value_no_dupes`, which parses one complete JSON value,
rejects duplicate keys recursively, and rejects trailing non-whitespace
content. Reuse it rather than implementing another parser.

`run_input.rs` provides a useful style reference for exact object validation,
stable errors, and RFC 6901 field pointers, but J24G must use the contract and
messages frozen in its own blueprint.

## Startup procedure

The current worktree may still be on an older implementation branch. Do not read
that branch's packet as current authority.

1. Confirm the worktree is clean. Stop if it is not.
2. Run `git fetch origin`.
3. Verify checkpoint `f5e621bee4338a496888daaf78e2f029e4ab0914` is an ancestor of `origin/main`.
4. Inspect the first lines of the packet directly from `origin/main`:

   ```powershell
   git show origin/main:docs/CURRENT_CLINE_TASK.md | Select-Object -First 16
   ```

   Require J24G, OpenCode, `READY`, and branch
   `opencode/j24g-installation-request-contract`.
5. Verify the blueprint directly from `origin/main`:

   ```powershell
   git cat-file -e origin/main:docs/architecture/J24G_INSTALLATION_REQUEST_CONTRACT.md
   ```

6. Check that `opencode/j24g-installation-request-contract` does not exist
   locally or remotely. If it exists, stop without resetting or overwriting it.
7. Create and switch to it from current `origin/main`:

   ```powershell
   git switch --create opencode/j24g-installation-request-contract origin/main
   ```

8. Read the checked-out packet and blueprint completely before editing.

## Required behaviour

1. Add `tethers-0.1/host-rust/src/installation_request.rs` and export it from
   `lib.rs`.

2. Implement exactly the public constants, types, enums, error shape, and two
   public functions frozen in the blueprint:

   ```rust
   load_installation_request(path: &Path)
   parse_installation_request_bytes(bytes: &[u8])
   ```

3. Keep the public request shape exactly:

   ```json
   {
     "schema": "tethers.plug-install/1",
     "candidate_id": "<canonical-lowercase-hyphenated-uuid>",
     "trust": { "scope": "exact_candidate" },
     "conformance": {
       "allow_non_isolated_supervised_execution": true
     },
     "installation": { "target_state": "disabled" }
   }
   ```

   Every field is required and no unknown field is permitted at any depth.

4. Implement bounded file loading: require an absolute ordinary file, reject a
   final symlink or directory, and read at most 16 KiB plus one byte through a
   bounded reader. Do not use `fs::read`.

5. Validate bytes in the frozen order: size, BOM, UTF-8, shared duplicate-key
   parser, then exact shape and semantic values.

6. Reuse `crate::manifest::parse_value_no_dupes`. Do not add a custom JSON
   parser, custom Serde visitor, or dependency.

7. Require `candidate_id` to be a canonical lowercase hyphenated UUID by
   parsing it and comparing it with `parsed.hyphenated().to_string()`.

8. Require exact-candidate trust, the JSON boolean `true` for
   `allow_non_isolated_supervised_execution`, and disabled target state. No
   alternative value is accepted.

9. Use only the two frozen public error codes:

   - `installation_request_io` for metadata, open, or read failures;
   - `installation_request_invalid` for every path or content validation
     failure.

10. Preserve every frozen error message and RFC 6901 field pointer. Never expose
    an operating-system path, raw request content, or platform I/O message.

11. Return only the typed request. Do not retain the original JSON value,
    compute a request digest, access any lifecycle store, or create any evidence.

12. Add comprehensive unit and integration evidence covering the complete
    blueprint matrix while proving parsing and loading create, delete, or modify
    no filesystem path.

## Relevant components

- `tethers-0.1/host-rust/src/installation_request.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/src/manifest.rs`
- `tethers-0.1/host-rust/src/run_input.rs`
- `tethers-0.1/host-rust/tests/j24g_installation_request.rs`
- `docs/architecture/J24G_INSTALLATION_REQUEST_CONTRACT.md`
- `docs/CURRENT_CLINE_TASK.md`

## Frozen decisions and invariants

- The installation request is a human decision, not host-generated evidence.
- The request applies only to one exact immutable candidate.
- Publisher-wide trust is not part of the first installation path.
- The wording `allow_non_isolated_supervised_execution` remains explicit and
  long because supervision is not a sandbox.
- `false`, a missing field, or a string such as `"true"` is not approval.
- Installation can target only `disabled`.
- The user never supplies timestamps, authorities, digests, evidence IDs,
  installation paths, quarantine paths, installed IDs, or enablement state.
- The request file is read once; its validated typed contents, not its path,
  become input to later gates.
- All reads are bounded before JSON parsing.
- Duplicate keys and trailing JSON are rejected through the existing shared
  parser.
- J24G performs no mutation and executes no provider code.
- Candidate identity remains distinct from installed identity.
- Tethers Core, OCaml semantics, package schemas, candidate schemas, and
  lifecycle evidence formats remain unchanged.

## Acceptance criteria

1. The new module and `lib.rs` export compile without dependency or lockfile
   changes.
2. The exact valid request parses into the exact typed values frozen in the
   blueprint.
3. An absolute ordinary request file loads successfully through a bounded read.
4. A valid request padded with JSON whitespace to exactly 16 KiB succeeds, and
   16 KiB plus one byte fails with the frozen limit error.
5. BOM, invalid UTF-8, malformed JSON, a second trailing JSON value, root
   duplicates, and duplicates in every nested object are rejected.
6. Every missing field is rejected with code `installation_request_invalid`,
   the frozen message, and its exact JSON pointer.
7. Unknown fields at the root and in `trust`, `conformance`, and `installation`
   are rejected with their exact escaped pointers.
8. Wrong root, nested-object, string, and boolean types are rejected with the
   frozen code, message, and pointer.
9. Unsupported schema values are rejected at `/schema`.
10. Invalid, uppercase, simple, braced, and otherwise non-canonical UUID text is
    rejected at `/candidate_id`.
11. Any trust scope other than `exact_candidate` is rejected at `/trust/scope`.
12. Missing, false, or non-boolean supervised-execution approval is rejected at
    `/conformance/allow_non_isolated_supervised_execution`.
13. Any target state other than `disabled` is rejected at
    `/installation/target_state`.
14. Relative, missing, directory, and final-symlink request paths are rejected
    with the frozen code and message; platform paths and raw I/O errors are not
    exposed.
15. Filesystem snapshots prove valid and invalid parsing/loading alter no byte
    and create or remove no path.
16. J24E and J24F focused tests remain green.
17. The full suite remains green apart from the five documented
    `pwsh.exe not found` environment failures.
18. Rustfmt, packet checker, and `git diff --check` pass.

## Required verification

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 test installation_request --locked
cargo +1.89.0 test --test j24g_installation_request --locked
cargo +1.89.0 test candidate_preparation --locked
cargo +1.89.0 test --test j24e_candidate_preparation --locked
cargo +1.89.0 test --test j24f_plug_stage_cli --locked
cargo +1.89.0 test --all-targets --all-features --locked
git diff --check
```

## Permitted changes

Expected files are limited to:

- `tethers-0.1/host-rust/src/installation_request.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/tests/j24g_installation_request.rs`
- `docs/worker-notes/2026-08-04-j24g-installation-request-contract.md`
- `docs/CURRENT_CLINE_TASK.md` only for status transitions and the final full
  implementation checkpoint

Stop before changing any other file.

## Forbidden changes

Do not modify `manifest.rs`, `run_input.rs`, candidate preparation, candidate,
package, trust, trusted store, launch profile, conformance, approval, installed,
enablement, CLI, application routing, Plug command formatting, dependencies, or
lockfiles.

Do not add `plug install`, a placeholder command, a reconciliation planner, a
request digest, a lock, atomic evidence writing, trust mutation, provider
launch, conformance execution, installation approval, payload copying,
installed publication, enablement, removal, update, download, registry, policy,
replay, event, Anchor, Trail, OCaml, Tether syntax, release, tag, or version
work.

Do not broaden trust beyond `exact_candidate` or allow installation to any state
other than `disabled`.

Do not amend, reset, rebase, cherry-pick, force-push, or merge into `main`.

## Stop conditions

Stop cleanly and report the smallest unresolved question if:

- the implementation branch already exists;
- current `origin/main` lacks accepted J24F or the J24G packet/blueprint;
- the shared duplicate-key parser cannot be reused without changing
  `manifest.rs`;
- a dependency, lockfile, CLI, lifecycle store, provider launch, or forbidden
  file appears necessary;
- the exact contract cannot be implemented with bounded read-only input;
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
files, unit and integration test counts, full-suite result, packet/rustfmt/diff
results, worker-note path, stable-error evidence, bounded-read evidence, and
explicit confirmation that J24G launched nothing and changed no lifecycle
state.
