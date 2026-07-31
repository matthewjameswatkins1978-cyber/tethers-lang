# Worker Note: J14C Real Local File Move Capability Proof

Task: `J14C - real local file move capability proof`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `e5c3328bf8dc54c738190134d4255bdaa9e7181f`

Implementation checkpoint: `b175c9dbcf599f42bd35398017ea9ea8682c5c22`

Implementation commit: `b175c9dbcf599f42bd35398017ea9ea8682c5c22`

Correction commits: three harness corrections; final SHA to be reported externally

## Requested outcome

Prove that Tethers performs one intelligible, externally visible job through the
accepted public Windows route. A readable Tether inspects a `folder.received_file`
event, matches PDF/invoice conditions, and plans `file.move`. A dedicated local
stdio MCP provider moves one real file from a bounded inbox to a bounded invoices
folder. The proof uses public `check`, `run`, and `trail` commands across nine
rows (F01-F09).

## Changes made

| Path | Change |
| --- | --- |
| `docs/CURRENT_CLINE_TASK.md` | Packet heading normalisation and status update |
| `docs/DECISIONS.md` | Added J14C decision at top |
| `docs/ROAD_TO_0_2.md` | Inserted J14C between J14 and J15, updated Stage E, release-acceptance claim, and date |
| `docs/worker-notes/2026-07-31-j14c-real-file-move.md` | This worker note |
| `tethers-0.1/providers/tethers-local-file-provider.ps1` | Dedicated real local stdio MCP provider (one tool: `file_move`) |
| `tethers-0.1/protocol/capability-manifests/file-move-local.json` | Reviewed `file.move` v1 capability manifest |
| `tethers-0.1/scenarios/j14c-real-file-move/README.md` | Human-readable scenario documentation |
| `tethers-0.1/scenarios/j14c-real-file-move/runtime.template.json` | Runtime template with five placeholders |
| `tethers-0.1/scenarios/j14c-real-file-move/input.invoice.json` | Matching invoice event input |
| `tethers-0.1/scenarios/j14c-real-file-move/input.photo.json` | Non-matching photo event input |
| `tethers-0.1/scenarios/j14c-real-file-move/tethers/sort-invoice.tether` | The scenario Tether |
| `tethers-0.1/scripts/test-j14c-real-file-move.ps1` | Public proof harness (F01-F09) |

### Correction

Three bounded harness corrections after review:

1. Removed the tenth "non-mutation" row; moved source/Cargo.lock non-mutation
   checks to ordinary assertions outside the row counter. Made F03, F04, and F05
   one continuous shared-workspace proof. Completed every missing packet
   assertion: provider output (moved, source_path, destination_path) in F03,
   Trail intent arguments in F04, structural Trail comparison in F05, no
   execution ID and no Result Anchor in F02 and F06, ACTION_FAILED machine code
   and Trail evidence in F07-F09, byte hashes for source/destination/photo in
   every relevant row. Wrapped all test execution in one try/finally with honest
   cleanup assertion.

2. Added explicit row-ID sequence tracking (`$script:rowIds`) and assertion of
   exact F01-F09 order. Fixed F05 structural Trail comparison to save filtered
   entries in F03 and assert equality after replay.

3. Replaced F07 traversal path from `workspace/invoices/../invoices/` (which
   normalises to the invoices prefix) to `workspace/invoices/../outside/` (which
   genuinely escapes the destination prefix). Created `workspace/outside`
   directory before the run and proved it remains empty.

## Decisions and assumptions

- Host scope binds `/source_path` only because the accepted runtime supports one
  JSON pointer per capability. The provider independently enforces both source
  and destination prefixes, canonical-root containment, and reparse-point
  inspection in every path component. Recorded in `docs/DECISIONS.md`.
- F06, F07, and F09 require custom input JSON (not the committed invoice input)
  because the event data must carry out-of-scope, traversal, or junction path
  arguments. The harness generates these with `Write-InputJson`.
- The replay root must be provisioned (`provision-replay`) before any `run` that
  produces an Action. The harness provisions the root for F03-F05 and F07-F09.
- F02 and F06 do not require replay provisioning because they produce
  `no_actions` and `denied` respectively, which do not record durable intent.

## Evidence

### Manifest

- Exact digest: `sha256:6a99459d4f01bca270ae7453757bcab9ce6b8fd4634f0be185a07ae13a34ac4e`
- Provider identity: `tethers-local-file-provider`
- Capability: `file.move` version `1`
- Binding: MCP server `tethers-local-file-provider`, tool `file_move`
- Host scope: `path_prefix` on `/source_path`, allowed prefix `workspace/inbox/`

### Row results

All nine rows PASS (9/9, 196 assertions). Row-ID sequence F01-F09 verified explicitly:

| Row | Result | Key evidence |
| --- | --- | --- |
| F01 | PASS | Check envelope `ok`, provider available, marker 1/1/0, no filesystem effect |
| F02 | PASS | Run `no_actions`, no execution ID, no Result Anchor, photo byte hash unchanged, no destination effect, marker 1/1/0 |
| F03 | PASS | Run `completed`, UUIDv4 execution ID, `capability.succeeded` anchor, provider output `moved:true` with exact source_path/destination_path, marker 1/1/1 |
| F04 | PASS | Trail `ok`, intent (file.move v1, provider identity, digest, source_path, destination_path) precedes terminal succeeded, exactly one terminal outcome |
| F05 | PASS | Replay `replay_blocked_completed_success`, same execution ID, marker 2/2/1 total, source absent, dest present and byte-identical, photo byte hash unchanged, no second external effect |
| F06 | PASS | Run `denied`, no execution ID, no Result Anchor, marker 1/1/0, source hash unchanged, destination absent |
| F07 | PASS | Run `failed` exit 6, `ACTION_FAILED`, UUIDv4 execution ID, marker 1/1/1, Trail terminal failed, source hash unchanged |
| F08 | PASS | Run `failed` exit 6, `ACTION_FAILED`, UUIDv4 execution ID, marker 1/1/1, Trail terminal failed, source and dest byte-identical to original |
| F09 | PASS | Run `failed` exit 6, `ACTION_FAILED`, UUIDv4 execution ID, marker 1/1/1, Trail terminal failed, source unchanged, no junction-escaped file |

### Non-mutation assertions (outside row counter)

- Committed tether SHA-256 unchanged
- Committed input SHA-256 unchanged
- Committed template SHA-256 unchanged
- Cargo.lock SHA-256: `d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602` — MATCH

### F03-F05 continuous proof

F03, F04, and F05 share one workspace, one replay root, one Trail file, and one
marker. F03 performs the successful move and captures the execution ID, source
hash, destination hash, and photo hash. F04 inspects the exact F03 Trail using
that execution ID, proving intent precedes terminal success with exact file.move
version 1, provider identity `tethers-local-file-provider`, manifest digest, and
visible source_path and destination_path arguments. F05 reruns the exact F03
input against the same runtime, replay root, and Trail, requiring the identical
execution ID, total tools/call count of 1 across both runs, structurally
identical filtered Trail entries, and zero second external effect.

### Successful move evidence

- Invoice SHA-256: `281071e7259b57c687d2d8fc5923fcac8b7258920f1969f98b5fb289813b0a10`
- Before: `workspace/inbox/invoice-july.pdf`
- After: `workspace/invoices/invoice-july.pdf` (byte-identical to source)
- Source absent after move, destination present
- Unrelated photo `workspace/inbox/holiday-photo.jpg` byte hash unchanged

### Provider safety boundaries

- Traversal destination (`workspace/invoices/../outside/invoice-july.pdf`):
  genuine prefix escape; refused by provider `Test-ResourcePath`, provider
  returns JSON-RPC error, ACTION_FAILED machine code, Trail terminal
  failed, no filesystem effect, source hash unchanged, `workspace/outside/`
  remains empty.
- Existing destination (pre-created with different bytes): refused, source and
  destination remain byte-identical to original content, terminal Trail failed.
- Junction escape (directory junction `escape-target` -> `outside`):
  `Get-ReparseEscape` detects reparse point, provider returns error,
  ACTION_FAILED, no file appears in outside directory.

### Execution identity

- F03 execution ID: `exec_8c698c71-9c07-4b73-a6b8-f87b363dd863` (verified UUIDv4)
- F04 Trail filtered to exact F03 execution ID
- F05 replay returns the identical execution ID
- Result Anchor serialised JSON does not contain `"execution_id"` (verified by `-notmatch`)
- F07/F08/F09 each expose trusted execution identity (validated UUIDv4)

### Trail evidence

- F04: Trail envelope `ok`, command `trail`, exit 0
- Intent entry: capability `file.move` v1, provider `tethers-local-file-provider`,
  manifest digest match, source_path and destination_path visible
- Terminal outcome: status `succeeded`, exactly one terminal outcome
- F07-F09: Trail terminal outcome status `failed`, exactly one terminal outcome,
  exactly one tools/call per row

### Provider containment

The provider implements:
- Resource path validation: empty, NUL, backslash, colon, rooted, absolute,
  drive-relative, UNC, device, empty segment, `.`, `..`, wildcard
- Canonical root containment with segment-boundary enforcement
- Source below configured source prefix, destination below destination prefix
- Reparse-point/junction/symbolic-link inspection in every path component
- `System.IO.File.Move(source, destination, false)` — no overwrite, no directory
  creation, no wildcard expansion
- JSON-RPC errors for all validation and filesystem failures

### Verification results

| Check | Result |
| --- | --- |
| J14C harness (corrected) | 9/9 PASS, 196 assertions |
| test-engine.ps1 | 27/27 PASS (fixture cases) |
| demo.ps1 | PASS |
| J14A | 5/5 PASS, 95 assertions |
| J14B | 11/11 PASS, 243 assertions |
| J13A | 25/25 PASS |
| J13B | 10/10 PASS (one retry, known flaky Ctrl+C test) |
| J13C | 19/19 PASS |
| check-fixtures | 46 JSON + 30 JSONL valid |
| test-mcp-transcripts | 15/15 PASS |
| Rust tests | 724 + 29 = 753 PASS |
| cargo build --locked | PASS (pre-existing warnings) |
| cargo build --locked --release | PASS (pre-existing warnings) |
| cargo fmt --check | PASS |
| Cargo.lock hash | MATCH |
| Packet checker | PASS |
| Toolchain preflight | 24/24 PASS |
| git diff --check | PASS |
| Authorised path check | 12 paths in branch range, clean final worktree |

### Engine and demo

`test-engine.ps1` and `demo.ps1` require the opam switch at
`D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`. Both were run
successfully using `opam env --switch` to set the external switch in the shell
before invoking the scripts. No OCaml source changes exist in this branch.
The engine binary is proven working by nine harness rows.

## Discoveries

- The `run` command returns `unavailable` (status and embedded exit code 4) with
  `REPLAY_PERSISTENCE_UNAVAILABLE` when the host data root has not been
  provisioned via `provision-replay`. This was the root cause of the initial F03
  failure; all run cases that produce Actions must call `Provision-ReplayRoot`.
- F06 (`denied`) and F02 (`no_actions`) envelope data objects may not include an
  `execution_id` property. Assertions must use
  `PSObject.Properties["execution_id"]` rather than direct property access.
- The `New-Workspace` function creates `workspace` as a flat directory; each
  test case must individually create `workspace/inbox`, `workspace/invoices`, or
  `workspace/outsider` subdirectories before writing test files.
- The opam switch at the original worktree is an external switch. Both
  `test-engine.ps1` and `demo.ps1` require `opam env --switch <path>` to set
  the switch in the shell before invocation.

## Remaining risks

None known within packet scope. The provider is confined to one canonical root
and does not support symlinks, network paths, or mount points. These are
deliberate scope boundaries, not defects. The junction escape test (F09)
requires `cmd.exe /c mklink /J` with administrator privilege or developer mode
enabled.

## Smallest next action

J15 consolidation: create a discoverable Windows verification entry point that
reports every 0.2 release case separately and honestly, incorporating the J14C
rows into the consolidated matrix.

## References

- Task packet: `docs/CURRENT_CLINE_TASK.md`
- Proof harness: `tethers-0.1/scripts/test-j14c-real-file-move.ps1`
- Provider: `tethers-0.1/providers/tethers-local-file-provider.ps1`
- Manifest: `tethers-0.1/protocol/capability-manifests/file-move-local.json`
- Scenario: `tethers-0.1/scenarios/j14c-real-file-move/`
- Programme: `docs/ROAD_TO_0_2.md`
- Decisions: `docs/DECISIONS.md`
- Branch: `opencode/j14c-real-file-move`
