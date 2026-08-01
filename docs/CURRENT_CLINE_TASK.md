# Current Implementation Task

Control contract: `1`

Task: `J18D - .tetherplug Package Format v1`
Owner: `Luna`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `Luna on OpenCode, architecture transcription and consistency audit`
Base branch: `main`
Base commit: `202abbb79d0095d2e9b4e07cd2d1d67f335f2302`
Branch: `luna/j18d-tetherplug-package-v1`
Worker note: `docs/worker-notes/2026-08-01-j18d-tetherplug-package-v1.md`

## Objective

Define the precise first `.tetherplug` portable package format as documentation
and contract design only.

## Required outcome

Define Plug, Socket, binding, transport, layer ownership, request/result/event
routes, state and outcome boundaries, capability classes, installation/removal,
first implementation envelope, deferred scope, unsuitable systems, and J18H
paper validation without changing language or runtime semantics.

## Relevant background and existing behaviour

Tethers 0.2.0 is released at `b5546411661dcbcb53e1cf2538eaec594c6f76f2`.
J18A aligned the published baseline and opened the Universal Plug programme.
The Core and host separation, deterministic planning, permission boundaries,
canonical outcomes, replay guarantees, and Trails are existing contracts.

## Required behaviour

1. Create the canonical candidate architecture document with all required
   Socket, layer, flow, state, capability, installation, removal, validation,
   and acceptance boundaries.
2. Prepend the dated J18B decision log entry.
3. Align current-state documents with J18B active and J18C next after acceptance.
4. Preserve the unchanged language, runtime, trust, and release boundaries.

## Changes made

The canonical candidate is
`docs/architecture/TETHERS_UNIVERSAL_PLUG_ARCHITECTURE.md`. The decision log and
current-state documents identify J18B as active, architecture-only, pending Lucy
acceptance, with J18C next only after acceptance.

## Frozen decisions and invariants

- Tethers 0.1 syntax and semantics remain unchanged.
- Deterministic planning, Action ordering, permission semantics, Trail ordering,
  replay guarantees, and canonical 0.2 behaviour remain unchanged.
- Core remains unaware of packages and Plugs.
- Socket semantics, protocol binding, and transport remain separate.
- The first intended stack is MCP 2025-11-25 over local stdio.
- Trust, permissions, credentials, bindings, outcomes, lifecycle, and Trail are
  host-owned.
- Vendor-specific translation remains outside the host.
- Action, Query, and Anchor are first-slice candidates; Job, Stream, and Human
  Task remain reserved and unimplemented.
- No implementation is authorised by this packet.

## Relevant components

- `docs/architecture/TETHERS_UNIVERSAL_PLUG_ARCHITECTURE.md`
- `docs/DECISIONS.md`
- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`
- `docs/TASK_QUEUE.md`

## Authorised paths

- `docs/architecture/TETHERS_UNIVERSAL_PLUG_ARCHITECTURE.md`
- `docs/DECISIONS.md`
- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`
- `docs/TASK_QUEUE.md`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-01-j18b-universal-plug-architecture.md`

## Forbidden changes

Do not modify Rust, OCaml, Cargo, Dune, opam, tests, scripts, fixtures,
manifests, protocol transcripts, the Tether specification, Constitution,
released notes, tags, GitHub Releases, or implementation/evidence files. Do
not create JSON schemas, packages, CLI commands, provider code, Socket messages,
installation code, or sandbox code.

## Acceptance criteria

1. The architecture separates Core, host, Socket, binding, transport, provider,
   and outside system.
2. Socket is a semantic contract, not a transport.
3. No vendor-specific behaviour enters Core or host policy.
4. Host authority boundaries remain explicit.
5. Package, installation, provider, capability, and binding are distinct.
6. Action, Query, and Anchor are first-slice candidates.
7. Job, Stream, and Human Task remain reserved and unimplemented.
8. Installation and removal preserve trust and historical Trails.
9. Version axes are independent.
10. J18H paper validation is mandatory before freeze.
11. No implementation or schema files change.
12. Exactly seven authorised paths change and required checks pass.
13. Branch is committed and pushed.

## Required verification

- `git diff --check`
- exact changed-path and clean-worktree checks
- task-packet checker
- architecture heading scan
- required-boundary search
- false-claim search
- published main and peeled tag verification

## Stop conditions

Stop if the base, branch, published refs, worktree, or authorised paths differ;
if a false implementation claim appears; if a required boundary is missing; or
if verification fails. Do not silently redesign the architecture or begin J18C.

## Expected pre-existing changes

None on the new branch before J18B work.

## Commit and publication boundary

Create exactly one commit: `docs: define universal plug architecture`.

Push only `luna/j18b-universal-plug-architecture`. Do not push main, tag,
release, or begin J18C.
