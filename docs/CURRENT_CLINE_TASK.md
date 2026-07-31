# Current Implementation Task

Control contract: `1`

Task: `J14C - real local file move capability proof`

Owner: `OpenCode`

Recommended model: `Hy3 High`

Status: `READY`

Task colour: `Amber`

Route: `OpenCode implementation - Lucy independent review`

Base commit: `e5c3328bf8dc54c738190134d4255bdaa9e7181f`

Branch: `opencode/j14c-real-file-move`

Worker note: `docs/worker-notes/2026-07-31-j14c-real-file-move.md`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

## Objective

Prove that Tethers performs one intelligible, externally visible job through the
accepted public Windows route.

A readable Tether must inspect one `folder.received_file` event and immutable
Facts. When the file is a PDF and its name contains `invoice`, Tethers must plan
and authorise `file.move`. A dedicated local stdio MCP provider must then move
one real file from a bounded inbox to a bounded invoices folder.

The proof must use public `check`, `run`, and `trail` commands. It must also prove
that a non-matching image is untouched, an out-of-scope source is denied, exact
replay does not move the file twice, and unsafe destinations fail without an
external effect.

This task is deliberately inserted before J15. J14 proved the runtime with a
fixture capability. J14C proves that the same machinery performs a recognisable
real-world effect. J15 remains the later consolidated release verification entry
point.

## Product demonstration

The committed Tether should read like this, subject only to exact accepted 0.1
syntax:

```text
tether "Sort received invoices"

anchor
    folder.received_file

when
    file.type is "pdf"
    and file.name contains "invoice"

do
    file.move
        source_path: anchor.source_path
        destination_path: anchor.destination_path
```

Do not add `facts.*` Action arguments. Tethers 0.1 resolves Action arguments only
from literals and `anchor.*` references. Conditions continue to read immutable
Facts.

The successful demonstration should visibly produce:

```text
workspace/inbox/invoice-july.pdf
    -> Tethers public run
workspace/invoices/invoice-july.pdf
```

An unrelated `workspace/inbox/holiday-photo.jpg` must remain untouched.

## Required implementation

### 1. Dedicated real local provider

Add:

`tethers-0.1/providers/tethers-local-file-provider.ps1`

This is a dedicated provider, not a new mode in the test fixture.

It must:

- implement the accepted line-delimited stdio MCP protocol version
  `2025-11-25`;
- identify itself exactly as `tethers-local-file-provider`;
- advertise exactly one tool named `file_move`;
- write protocol JSON only to stdout;
- write diagnostics only to stderr;
- support an optional marker file recording exact method names
  `initialize`, `tools/list`, and `tools/call`;
- accept trusted host configuration parameters for one absolute provider root,
  one allowed source prefix, and one allowed destination prefix;
- perform no background watching, scanning, polling, network access, or daemon
  behaviour.

The tool input is exactly:

```json
{
  "source_path": "string",
  "destination_path": "string"
}
```

The successful output is exactly:

```json
{
  "moved": true,
  "source_path": "workspace/inbox/invoice-july.pdf",
  "destination_path": "workspace/invoices/invoice-july.pdf"
}
```

The provider must return JSON-RPC errors for expected validation and filesystem
failures. It must never print a success response unless the source is absent,
the destination exists as a regular file, and the move is known to have
completed.

### 2. Provider filesystem boundary

The provider root is an existing absolute directory supplied by trusted runtime
configuration. Resource paths are relative forward-slash paths beneath that
root.

For both source and destination, reject:

- empty strings;
- absolute, rooted, drive-relative, UNC, or device paths;
- backslashes;
- colons;
- NUL;
- empty segments;
- `.` or `..` segments;
- wildcard interpretation;
- paths outside the canonical provider root;
- any existing reparse point, junction, or symbolic-link component beneath the
  provider root.

Also require:

- source is beneath the configured source prefix;
- destination is beneath the configured destination prefix;
- source exists and is a regular file;
- destination parent already exists and is a regular directory;
- destination does not already exist;
- source and destination are not the same canonical path;
- comparison and root containment are Windows case-insensitive and preserve a
  path-segment boundary;
- no directory is created implicitly;
- no overwrite occurs;
- literal .NET filesystem APIs are used, not wildcard-expanding cmdlets.

Use an operation equivalent to `System.IO.File.Move(source, destination, false)`.
Do not implement copy, delete, rename batches, recursive moves, compensation, or
an overwrite flag.

### 3. Capability manifest

Add:

`tethers-0.1/protocol/capability-manifests/file-move-local.json`

The manifest must:

- declare `file.move`, version `1`;
- bind provider identity and MCP server name to
  `tethers-local-file-provider`;
- bind tool name to `file_move`;
- advertise the exact input and output schemas above;
- declare effect `file.move`;
- use `path_prefix` permission scope with reviewed source prefix
  `workspace/inbox/`;
- allow standing permission for this bounded scenario;
- declare zero automatic retries;
- honestly state that the provider itself has no separate idempotency mechanism;
- use the existing accepted manifest format and enum vocabulary;
- contain the exact RFC 8785/SHA-256 canonical digest expected by the host.

Do not use the raw file hash as the manifest digest. Compute the canonical digest
using the existing manifest rules and prove it through public `check`.

### 4. Scope split

The current host `path_prefix` contract binds one JSON pointer. Do not redesign
it in this task.

The runtime configuration must bind host scope to:

`/source_path`

The host therefore proves that the source is beneath `workspace/inbox/` before
dispatch. The dedicated provider independently enforces both configured source
and destination prefixes and canonical-root confinement.

Record this split explicitly in `docs/DECISIONS.md` and the scenario README. Do
not imply that the host currently evaluates both path arguments.

### 5. Committed scenario

Add:

- `tethers-0.1/scenarios/j14c-real-file-move/README.md`
- `tethers-0.1/scenarios/j14c-real-file-move/runtime.template.json`
- `tethers-0.1/scenarios/j14c-real-file-move/input.invoice.json`
- `tethers-0.1/scenarios/j14c-real-file-move/input.photo.json`
- `tethers-0.1/scenarios/j14c-real-file-move/tethers/sort-invoice.tether`

The runtime template must contain placeholders for:

- the provider script;
- the provider root;
- the provider marker;
- the manifest path;
- the exact pinned manifest digest.

Configure:

- provider root: one generated case-local root;
- provider source prefix: `workspace/inbox/`;
- provider destination prefix: `workspace/invoices/`;
- host scope binding: `/source_path`;
- host policy: Allow only `file.move` version 1.

The invoice input must use:

- event name `folder.received_file`;
- source `workspace/inbox/invoice-july.pdf`;
- destination `workspace/invoices/invoice-july.pdf`;
- Fact `file.type = pdf`;
- Fact `file.name = invoice-july.pdf`.

The photo input must use:

- event name `folder.received_file`;
- source `workspace/inbox/holiday-photo.jpg`;
- destination `workspace/invoices/holiday-photo.jpg`;
- Fact `file.type = jpg`;
- Fact `file.name = holiday-photo.jpg`.

The README must show the human-readable Tether, the before/after directory tree,
the public commands, the safety boundary, and one command that runs the proof.

### 6. Public proof harness

Add:

`tethers-0.1/scripts/test-j14c-real-file-move.ps1`

Use one unique system temporary root containing both a space and a non-ASCII
character. Put every generated config, input, provider root, marker, Trail,
replay root, junction target, and output beneath it. Remove it in `finally` on
success or failure.

The script must report exactly these nine rows, once each and in this order:

| ID | Row |
| --- | --- |
| F01 | public check admits real file provider |
| F02 | non-matching photo remains untouched |
| F03 | matching invoice moves exactly once |
| F04 | public Trail explains the move |
| F05 | exact replay causes no second move |
| F06 | out-of-scope source is denied |
| F07 | traversal destination fails safely |
| F08 | existing destination is never overwritten |
| F09 | junction escape fails safely |

Use explicit row IDs and assert the final sequence is exactly F01 through F09,
with no duplicate or extra row. Print honest row and assertion totals.

## Row contracts

### F01 - public check admits real file provider

Run the real public `check` command with the real OCaml engine and dedicated
provider.

Require:

- one JSON envelope;
- command `check`;
- status `ok` and process/embedded exit 0;
- exactly one provider and one capability;
- provider and capability are available;
- exact `file.move` version 1 identity;
- marker counts: initialize 1, tools/list 1, tools/call 0;
- no file is moved or created by check.

### F02 - non-matching photo remains untouched

Create both the photo and its destination directory. Run the public `run`
command with `input.photo.json`.

Require:

- status `no_actions`, exit 0;
- no execution ID;
- no Result Anchor;
- marker counts: initialize 1, tools/list 1, tools/call 0;
- photo remains byte-identical in the inbox;
- no destination photo exists;
- no other file changes.

### F03 - matching invoice moves exactly once

Create:

- `workspace/inbox/invoice-july.pdf` with deterministic non-empty bytes;
- `workspace/inbox/holiday-photo.jpg` with different deterministic bytes;
- existing empty `workspace/invoices/` directory.

Record hashes before the run. Run the public `run` command with
`input.invoice.json`.

Require:

- one JSON envelope;
- status `completed`, exit 0;
- execution status `completed`;
- one parseable trusted `exec_<UUID>` execution ID;
- exactly one `capability.succeeded` Result Anchor;
- Result Anchor contains no `execution_id`;
- marker counts: initialize 1, tools/list 1, tools/call 1;
- source invoice is absent;
- destination invoice exists as a regular file;
- destination hash equals the original source hash;
- output contains exact source and destination resource paths and `moved: true`;
- unrelated photo remains byte-identical in the inbox;
- no automatic retry.

### F04 - public Trail explains the move

Use the execution ID returned by F03 with the real public `trail` command.

Require:

- one JSON envelope;
- command `trail`, status `ok`, exit 0;
- only entries for the exact F03 execution ID;
- the durable intent precedes the terminal succeeded outcome;
- capability is `file.move` version 1;
- provider identity is `tethers-local-file-provider`;
- exact source and destination arguments are visible in the appropriate durable
  evidence;
- exactly one terminal succeeded outcome;
- the Trail makes no claim that a second call occurred.

Use the accepted J14A Trail helper patterns and exact existing Trail field names.
Do not invent a second Trail schema.

### F05 - exact replay causes no second move

Run the exact F03 public input again against the same replay root and Trail.

Require:

- status `completed`, exit 0;
- execution status `replay_blocked_completed_success`;
- exact same execution ID as F03;
- total marker counts across both runs: initialize 2, tools/list 2, tools/call 1;
- destination invoice remains byte-identical;
- source remains absent;
- unrelated photo remains untouched;
- filtered Trail entries for the execution ID remain structurally identical;
- no second external effect.

### F06 - out-of-scope source is denied

Create `workspace/outside/invoice-secret.pdf`. Generate a matching invoice event
whose source is that path and whose destination is within the invoices prefix.

Require:

- public `run` status `denied`, exit 0;
- no execution ID;
- no Result Anchor;
- marker counts: initialize 1, tools/list 1, tools/call 0;
- outside source remains byte-identical;
- destination is absent.

This row proves the host-side `/source_path` scope binding.

### F07 - traversal destination fails safely

Create a fresh in-scope source and use a destination containing a `..` segment
that would escape the configured invoices prefix if interpreted.

Require:

- host policy allows the source and dispatch reaches the provider;
- public `run` status `failed`, exit 6, machine code `ACTION_FAILED`;
- one trusted execution ID;
- marker counts: initialize 1, tools/list 1, tools/call 1;
- source remains byte-identical;
- no destination or outside file is created;
- Trail records one terminal failed outcome;
- no retry.

### F08 - existing destination is never overwritten

Create a fresh source and a pre-existing destination with different content.

Require:

- public `run` status `failed`, exit 6, `ACTION_FAILED`;
- marker counts: initialize 1, tools/list 1, tools/call 1;
- source remains byte-identical;
- destination remains byte-identical to its original content;
- exactly one terminal failed outcome;
- no retry.

### F09 - junction escape fails safely

Create a provider root and a separate outside directory beneath the case root.
Create a Windows directory junction beneath the configured invoices directory
that points to the outside directory. Use a destination path passing through the
junction.

Require:

- the junction is created successfully on the native Windows environment;
- public `run` status `failed`, exit 6, `ACTION_FAILED`;
- marker counts: initialize 1, tools/list 1, tools/call 1;
- source remains byte-identical;
- no file appears in the outside directory;
- exactly one terminal failed outcome;
- no retry.

Do not skip this row. Return `BLOCKED` with the exact native error if a junction
cannot be created or inspected safely.

## Programme and decision updates

Update `docs/ROAD_TO_0_2.md` to:

- insert J14C between J14 and J15;
- describe J14C as the real local file-move capability proof;
- make J15 depend on accepted J14C;
- change Stage E to cover J12 through J14C;
- add one release-acceptance claim that a non-fixture local provider performs a
  bounded visible file move while non-match and out-of-scope inputs cause no
  effect;
- record the programme update date as 31 July 2026.

Do not rewrite completed job history or update unrelated old checkpoint text.

Add a decision at the top of `docs/DECISIONS.md` recording:

- Matthew authorised J14C after J14 publication;
- fixture ping proved the execution machinery but not an intelligible external
  effect;
- J14C remains inside the existing 0.2 promise of one real local permissioned
  execution loop;
- host scope binds source only because the accepted runtime currently supports
  one JSON pointer;
- the provider independently confines source and destination;
- no watcher, GUI, general filesystem API, overwrite, retry, or production host
  redesign is included.

## Relevant existing components

Read and reuse patterns from:

- `tethers-0.1/scenarios/j14-complete-local/`
- `tethers-0.1/scripts/test-j14a-complete-scenario.ps1`
- `tethers-0.1/scripts/test-j14b-negative-matrix.ps1`
- `tethers-0.1/scripts/tethers-stdio-fixture.ps1`
- `tethers-0.1/protocol/capability-manifests/fixture-ping-standing-allow.json`
- `tethers-0.1/host-rust/src/configured_runtime.rs` path validation and scope
  assessment
- `tethers-0.1/host-rust/src/stdio_provider.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/SPEC.md`
- `docs/CAPABILITY_BRIDGE.md`

## Expected pre-existing changes

None.

The branch contains exactly one planning commit after the published J14 base.
The live implementation worktree must be completely clean before mutation.

## Authorised paths

Modify or add only these twelve paths:

1. `docs/CURRENT_CLINE_TASK.md`
2. `docs/ROAD_TO_0_2.md`
3. `docs/DECISIONS.md`
4. `docs/worker-notes/2026-07-31-j14c-real-file-move.md`
5. `tethers-0.1/providers/tethers-local-file-provider.ps1`
6. `tethers-0.1/protocol/capability-manifests/file-move-local.json`
7. `tethers-0.1/scenarios/j14c-real-file-move/README.md`
8. `tethers-0.1/scenarios/j14c-real-file-move/runtime.template.json`
9. `tethers-0.1/scenarios/j14c-real-file-move/input.invoice.json`
10. `tethers-0.1/scenarios/j14c-real-file-move/input.photo.json`
11. `tethers-0.1/scenarios/j14c-real-file-move/tethers/sort-invoice.tether`
12. `tethers-0.1/scripts/test-j14c-real-file-move.ps1`

No other path is authorised.

## Forbidden changes

Do not modify:

- production Rust;
- Rust tests;
- OCaml;
- existing provider fixtures;
- existing manifests or scenarios;
- Cargo files or `Cargo.lock`;
- public CLI, runtime configuration schema, scope model, Trail schema, replay
  format, Result Anchor schema, language grammar, or protocol version;
- AGENTS.md or workflow/control documents other than the current task packet;
- J15 implementation.

If the accepted public runtime cannot prove this capability without one of those
changes, return `BLOCKED` with the smallest exact missing boundary. Do not widen
the task silently.

## Frozen invariants

- Published base is exactly `e5c3328bf8dc54c738190134d4255bdaa9e7181f`.
- J14 remains accepted and unchanged.
- Public status and exit-code vocabulary remains frozen.
- Trusted execution identity comes only from replay admission.
- Result Anchors never contain `execution_id`.
- Intent is durable before the effectful call.
- At most one provider call occurs per attempted Action.
- No automatic retry or compensation.
- Exact replay does not repeat the move.
- A non-matching Tether performs no action.
- Host scope denial performs no provider call.
- Provider validation failure performs no filesystem effect.
- Existing destinations are never overwritten.
- Repository files are never used as move targets; all effects occur beneath the
  generated system temporary root.
- J15 does not begin in this task.

## Acceptance criteria

1. Mandatory startup report and pre-flight prove exact worktree, refs, clean
   status, twelve authorised paths, stop conditions, and two-failure rule.
2. Dedicated provider implements exactly one real `file_move` tool and no fixture
   modes or unrelated capability.
3. Manifest, live MCP tool schema, provider identity, server name, binding, and
   pinned canonical digest agree exactly.
4. Public `check` admits the provider and performs no effect.
5. Public non-match returns `no_actions` and leaves the photo untouched.
6. Public matching run physically moves the invoice, preserves bytes, leaves the
   photo untouched, returns trusted identity, and emits `capability.succeeded`.
7. Public Trail inspection explains the exact move with intent before success.
8. Exact replay returns the same identity and performs no second move.
9. Out-of-scope source is denied before tools/call.
10. Traversal destination, existing destination, and junction escape each fail
    after exactly one call and cause no filesystem effect.
11. Every row proves one JSON envelope where applicable, matching process and
    embedded exit codes, exact method counts, exact identity and Result Anchor
    rules, durable Trail evidence, and no retry.
12. Harness reports exactly F01 through F09, all PASS, with honest assertion
    totals, Unicode-plus-space root, cleanup, unchanged repository status, and
    unchanged Cargo.lock hash.
13. Scenario README provides a convincing human-readable demonstration from a
    clean built checkout.
14. Programme, decision, worker note, packet checker, focused scenario,
    regressions, toolchains, Cargo.lock, whitespace, branch range, and worktrees
    are reported honestly.
15. No production Rust, Rust test, OCaml, schema, or existing fixture change is
    present.

## Mandatory reading

Read in full before editing:

- `AGENTS.md`
- `docs/PROJECT_CONTROL.md`
- `docs/AGENT_WORKFLOW.md`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`
- `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md`
- `docs/ROAD_TO_0_2.md` J14 through J17
- `docs/DECISIONS.md` J14A and J14B decisions
- `docs/CAPABILITY_BRIDGE.md`
- `tethers-0.1/SPEC.md`
- all relevant components listed above.

## Pre-flight

Run:

```powershell
git rev-parse --show-toplevel
git branch --show-current
git status --porcelain=v1 --untracked-files=all
git fetch origin --prune
git rev-parse HEAD
git rev-parse origin/opencode/j14c-real-file-move
git rev-parse origin/main
git merge-base HEAD origin/main
git rev-list --count origin/main..HEAD
git rev-list --count HEAD..origin/main
git worktree list --porcelain
```

Require:

- worktree `D:\The Next Thing\Tethers Lang - Goose Integration`;
- branch `opencode/j14c-real-file-move`;
- completely clean status;
- local and remote branch identical;
- origin/main exactly `e5c3328bf8dc54c738190134d4255bdaa9e7181f`;
- merge base exactly origin/main;
- branch exactly one planning commit ahead and zero behind;
- original worktree preserved on `cline/j10-result-event-queue` with only
  `M docs/TETHERS_LUCY_NOTES.md`.

Run the non-mutating toolchain preflight:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass `
  -File .\.github\scripts\check-tethers-toolchains.ps1 `
  -OcamlSwitchPath `
    "D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml"
```

## Required verification

Use process-local `RUSTUP_AUTO_INSTALL=0`, restoring its exact previous state in
`finally`.

Run:

```powershell
rustup run 1.89.0 cargo fmt --manifest-path .\tethers-0.1\host-rust\Cargo.toml --check
rustup run 1.89.0 cargo test --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked
rustup run 1.89.0 cargo build --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked
rustup run 1.89.0 cargo build --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked --release
```

Then run:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-j14c-real-file-move.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-j14a-complete-scenario.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-j14b-negative-matrix.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-j13a-check.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-j13b-run.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-j13c-trail.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\check-fixtures.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-mcp-transcripts.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-engine.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\demo.ps1
```

Run the packet and Git checks:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File .\.github\scripts\check-tethers-task-packet.ps1
git diff --check e5c3328bf8dc54c738190134d4255bdaa9e7181f..HEAD
git diff --name-status e5c3328bf8dc54c738190134d4255bdaa9e7181f..HEAD
git status --porcelain=v1 --untracked-files=all
```

Require Cargo.lock SHA-256 to remain exactly:

`d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602`

The pre-existing J13B Ctrl+C interruption test is known to be flaky on native
Windows. One materially identical retry is permitted and must be reported. A
second failure returns `BLOCKED`; do not weaken the assertion.

## Worker note

Write `docs/worker-notes/2026-07-31-j14c-real-file-move.md` with:

- flat checker-compatible metadata;
- exact base and implementation checkpoint;
- exact manifest digest;
- exact F01-F09 results and assertion count;
- exact before/after file hashes for the successful move;
- exact provider method counts for every row;
- exact F03 and replay execution IDs;
- exact public Trail evidence;
- exact unsafe-path and junction evidence;
- Rust and regression totals;
- Cargo.lock hash;
- honest discoveries and remaining risks;
- smallest next action: J15 consolidation.

Do not insert a future closeout commit SHA into the note. Report it externally.

## Commit and publication boundary

Create one implementation commit and at most one documentation closeout commit.
Suggested implementation commit:

`feat: prove real file move capability`

Push only:

`opencode/j14c-real-file-move`

Do not push main. Do not delete branches or worktrees. Do not begin J15.

## Stop conditions

Return `BLOCKED` when:

- any pre-flight ref or worktree differs;
- any unauthorised path changes;
- production Rust, Rust tests, OCaml, schema, grammar, or existing fixtures would
  need modification;
- provider confinement cannot reject traversal, overwrite, or junction escape;
- the manifest cannot be admitted with an honestly computed canonical digest;
- any public row is weaker than its contract;
- the scenario mutates any path outside its generated temporary root;
- Cargo.lock changes;
- two materially similar attempts fail.

## Return contract

Return `COMPLETE` or `BLOCKED` and stop.

For `COMPLETE`, report:

- local and remote branch SHA;
- exact twelve-path branch range;
- exact F01-F09 results and assertion total;
- exact manifest digest;
- exact successful before/after paths and hashes;
- exact execution and replay identity;
- exact provider method counts;
- exact Trail evidence;
- traversal, overwrite, and junction safety evidence;
- Rust totals and regression results;
- packet checker, whitespace, Cargo.lock, branch relationship, and clean status;
- preserved original worktree;
- unresolved risks and honest exceptions.

Stop after reporting. Do not begin J15.
