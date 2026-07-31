# Worker Note: J14C Real Local File Move Capability Proof

Task: `J14C - real local file move capability proof`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `e5c3328bf8dc54c738190134d4255bdaa9e7181f`

Implementation checkpoint: `WORKTREE`

**Note:** The implementation is uncommitted; to be committed as `feat: prove real file move capability`. The commit SHA will be reported externally after the commit is created.

## Requested outcome

Prove that Tethers performs one intelligible, externally visible job through the
accepted public Windows route. A readable Tether inspects a `folder.received_file`
event, matches PDF/invoice conditions, and plans `file.move`. A dedicated local
stdio MCP provider moves one real file from a bounded inbox to a bounded invoices
folder. The proof uses public `check`, `run`, and `trail` commands and proves
fifteen boundary conditions across rows F01-F09.

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
| `tethers-0.1/scripts/test-j14c-real-file-move.ps1` | Public proof harness (F01-F09 + non-mutation) |

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

All ten rows PASS (10/10, 158 assertions):

| Row | Result | Key evidence |
| --- | --- | --- |
| F01 | PASS | Check envelope `ok`, provider available, marker 1/1/0, no filesystem effect |
| F02 | PASS | Run `no_actions`, photo untouched, marker 1/1/0 |
| F03 | PASS | Run `completed`, execution ID `exec_<UUIDv4>`, `capability.succeeded` anchor, marker 1/1/1 |
| F04 | PASS | Trail `ok`, intent before success, `file.move` v1 identity, provider identity |
| F05 | PASS | Replay `replay_blocked_completed_success`, same execution ID, marker 2/2/1, no second move |
| F06 | PASS | Run `denied`, no execution ID, marker 1/1/0, source untouched |
| F07 | PASS | Run `failed` exit 6, marker 1/1/1, source untouched, no traversal file |
| F08 | PASS | Run `failed` exit 6, marker 1/1/1, source and dest byte-identical to original |
| F09 | PASS | Run `failed` exit 6, marker 1/1/1, source untouched, no junction-escaped file |
| Non-mutation | PASS | Committed sources and Cargo.lock hashes unchanged |

### Successful move evidence

- Before: `workspace/inbox/invoice-july.pdf` (hash: `281071e7259b57c687d2d8fc5923fcac8b7258920f1969f98b5fb289813b0a10`)
- After: `workspace/invoices/invoice-july.pdf` (hash: `281071e7259b57c687d2d8fc5923fcac8b7258920f1969f98b5fb289813b0a10`)
- Source absent, destination present, byte-identical content
- Unrelated photo `workspace/inbox/holiday-photo.jpg` (hash: `f8ded7b543dceb5f01bedcdcb479b5aad0a0c8367db0bad339d71eb0ce235898`) untouched

### Provider safety boundaries

- Traversal destination (`..` segment): refused by `Test-ResourcePath`, provider returns JSON-RPC error `-32602`, no filesystem effect
- Existing destination (`workspace/invoices/invoice-july.pdf` pre-created with hash `5424cd51f1cc162602a9cd2e64a21079fd21c098849608c969579dddda4dc8f6`): refused, source and destination remain byte-identical
- Junction escape (directory junction `escape-target` -> `outside`): `Get-ReparseEscape` detects reparse point, provider returns error, no file appears in `outside` directory

### Execution identity

- F03 execution ID format: `exec_<UUIDv4>` (verified by regex `^exec_[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$`)
- F05 replay returns the identical execution ID
- Result Anchor serialised JSON does not contain `"execution_id"` (verified by `-notmatch`)

### Trail evidence (F04)

- Trail envelope `ok`, command `trail`, exit 0
- Two entries: durable intent (capability `file.move` v1, provider `tethers-local-file-provider`, digest match) then terminal outcome (status `succeeded`)
- Intent precedes success; exactly one terminal succeeded outcome; no second call claimed

### Provider containment

The provider implements:
- Resource path validation: empty, NUL, backslash, colon, rooted, absolute, drive-relative, UNC, device, empty segment, `.`, `..`, wildcard
- Canonical root containment with segment-boundary enforcement
- Source below configured source prefix, destination below destination prefix
- Reparse-point/junction/symbolic-link inspection in every path component
- `System.IO.File.Move(source, destination, false)` — no overwrite, no directory creation, no wildcard expansion
- JSON-RPC errors for all validation and filesystem failures (`-32602` and `-32603`)

## Discoveries

- The `run` command returns `unavailable` (status and embedded exit code 4) with
  `REPLAY_PERSISTENCE_UNAVAILABLE` when the host data root has not been
  provisioned via `provision-replay`. This was the root cause of the initial F03
  failure; all run cases that produce Actions must call `Provision-ReplayRoot`
  first.
- F06 (`denied`) and F02 (`no_actions`) envelope data objects may not include an
  `execution_id` property. Assertions that check for its absence must use
  `PSObject.Properties["execution_id"]` rather than direct property access to
  avoid `PropertyNotFoundException`.
- The `New-Workspace` function in the harness creates `workspace` as a flat
  directory; each test case must individually create `workspace/inbox`,
  `workspace/invoices`, or `workspace/outsider` subdirectories before writing
  test files.
- The J14C harness had several corruptions in the candidate implementation
  (Unicode corruption of numbers, undefined variable `$ws` in F09, wrong
  execution_id check scope). All were corrected before the first successful run.

## Remaining risks

None known within packet scope. The provider is confined to one canonical root
and does not support symlinks, network paths, or mount points. These are
deliberate scope boundaries, not defects. The junction escape test (F09)
requires `cmd.exe /c mklink /J` with administrator privilege or developer mode
enabled; BLOCKED should be reported if junction creation fails on the target
environment.

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
