# Current Implementation Task

Control contract: `1`

Task: `J18A - post-release reset`
Owner: `Luna`
Status: `COMPLETE`
Task colour: `Green`
Route: `Luna on OpenCode - documentation-only project-state alignment`
Branch: `luna/j18a-post-release-reset`
Base commit: `b5546411661dcbcb53e1cf2538eaec594c6f76f2`
Worker note: `docs/worker-notes/2026-08-01-j18a-post-release-reset.md`

## Objective

Close Tethers 0.2.0 as independently signed off and published, then open the
J18 Universal Plug Architecture and Plug Kit programme without implementing
Plug functionality.

## Relevant background and existing behaviour

The exact accepted release commit is `b5546411661dcbcb53e1cf2538eaec594c6f76f2`.
Remote `main` and peeled tag `v0.2.0` identify that commit. J17 returned
`SIGNED OFF FOR 0.2.0`; all 17 release claims were proven. Language semantics
remain `0.1`.

## Required behaviour

1. Mark the 0.2.0 release published and signed off in release notes and README.
2. Align current goal, dashboard, and queue with J18 and the published baseline.
3. Replace this packet with the completed J18A packet after verification.
4. Create the J18A worker note and preserve all runtime and release boundaries.

## Relevant components

- `README.md`
- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`
- `docs/TASK_QUEUE.md`
- `docs/releases/v0.2.0.md`

## Frozen decisions and invariants

- No Plug implementation, package schema, Socket code, CLI command, or provider
  change is authorised.
- Tethers Core remains application-agnostic and plugs remain outside the core.
- Permissions, credentials, canonical outcomes, and Trails remain host-owned.
- Tethers 0.1 language semantics and 0.2 runtime boundaries are unchanged.
- `ROAD_TO_0_2.md`, release evidence, tag, and implementation files are read-only.

## Acceptance criteria

1. All current documents state that 0.2.0 is published at the exact accepted SHA.
2. J17 is complete and J18 is the only active programme.
3. Plug implementation remains explicitly unauthorised.
4. Exactly seven authorised paths change and all required checks pass.

## Required verification

- Verify local and remote published refs and peeled tag target.
- Run stale release-state search across current documents.
- Run `git diff --check`, path/status checks, and the packet checker.

## Forbidden changes

Do not modify Rust, OCaml, tests, scripts, fixtures, manifests, Cargo files,
protocol transcripts, version strings, `docs/ROAD_TO_0_2.md`, tag objects,
GitHub Releases, or release evidence directories. Do not begin Plug
implementation.

## Stop conditions

Stop if published refs differ, a stale release claim remains, an unauthorised
path changes, or any required check fails.

## Expected pre-existing changes

None in the new branch before this task.
