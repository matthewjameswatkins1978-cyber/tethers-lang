# J14C Real Local File Move

This scenario proves that Tethers performs one intelligible, externally visible
job through the accepted public Windows route. Every earlier accepted row ended
in an echoed fixture string; this one moves a real file.

## The Tether

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

One Anchor, two Conditions over immutable Facts, one Action. Action arguments
resolve only from `anchor.*` references, which is all Tethers 0.1 permits.

## Before and after

```text
Before                                 After
workspace/                             workspace/
  inbox/                                 inbox/
    invoice-july.pdf                       holiday-photo.jpg
    holiday-photo.jpg                    invoices/
  invoices/                                invoice-july.pdf
```

The invoice matches both Conditions and is moved. The holiday photo matches
neither `file.type is "pdf"` nor `file.name contains "invoice"`, so the Tether
proposes no Action and nothing on disk changes.

## Public commands

```powershell
tethers-reference-host check --config <runtime.json> --engine <engine.exe>

tethers-reference-host run   --config <runtime.json> --engine <engine.exe> `
                             --input  <input.invoice.json> `
                             --trail  <trail.jsonl> `
                             --host-data-root <replay-root>

tethers-reference-host trail --trail <trail.jsonl> --execution-id <exec_UUID>
```

`check` admits the provider and performs no effect. `run` plans, authorises,
records durable intent, makes exactly one `tools/call`, validates the output,
and returns a `capability.succeeded` Result Anchor. `trail` explains what
happened, with intent recorded before the successful outcome.

## Files

- `tethers/sort-invoice.tether` - the scenario Tether
- `input.invoice.json` - the matching `folder.received_file` event
- `input.photo.json` - the non-matching `folder.received_file` event
- `runtime.template.json` - runtime configuration with five placeholders:
  `__PROVIDER_SCRIPT__`, `__PROVIDER_ROOT__`, `__PROVIDER_MARKER__`,
  `__MANIFEST_PATH__`, and `__MANIFEST_DIGEST__`

The reviewed capability manifest is
`tethers-0.1/protocol/capability-manifests/file-move-local.json`, pinned by its
RFC 8785/SHA-256 canonical digest
`sha256:6a99459d4f01bca270ae7453757bcab9ce6b8fd4634f0be185a07ae13a34ac4e`.

The provider is `tethers-0.1/providers/tethers-local-file-provider.ps1`, a
dedicated local stdio MCP provider exposing exactly one `file_move` tool. It is
not the test fixture and has no failure-injection modes.

## Safety boundary

Two independent boundaries protect this Action, and they do not check the same
thing.

**Host scope.** The accepted runtime binds `path_prefix` scope through exactly
one JSON pointer per capability. This scenario binds `/source_path`. The host
therefore proves before dispatch that the source lies beneath
`workspace/inbox/`. It does **not** currently evaluate `destination_path`.

**Provider confinement.** The provider independently enforces everything the
host cannot express through a single pointer:

- both paths must be relative forward-slash paths beneath one canonical root;
- empty, rooted, absolute, drive-relative, UNC, device, backslash, colon, NUL,
  empty-segment, `.` and `..` paths are refused;
- the source must lie beneath `workspace/inbox/` and the destination beneath
  `workspace/invoices/`;
- no component of either path may be a reparse point, junction, or symbolic
  link;
- the source must exist as a regular file and the destination must not exist;
- the destination parent must already exist; no directory is ever created;
- nothing is ever overwritten;
- the move uses `System.IO.File.Move(source, destination, false)`, never a
  wildcard-expanding cmdlet.

Beyond that: a Plan is a request, not permission; intent is durable before the
call; at most one `tools/call` occurs per attempted Action; there is no
automatic retry and no compensation; and exact replay of a completed execution
returns the same identity without moving the file a second time.

## Running the proof

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass `
  -File .\tethers-0.1\scripts\test-j14c-real-file-move.ps1
```

The harness reports rows F01 through F09: check admission, the untouched photo,
the successful move, the public Trail, blocked replay, out-of-scope denial,
traversal refusal, overwrite refusal, and junction-escape refusal. Every file it
creates lives beneath one unique system temporary directory that is removed
afterwards. No repository file is mutated.
