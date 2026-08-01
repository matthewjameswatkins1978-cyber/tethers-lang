# Current Implementation Task

Control contract: `1`

Task: `J17A2 - draft the Tethers 0.2.0 release notes`
Owner: `Luna`
Status: `COMPLETE`
Task colour: `Green`
Route: `Luna on OpenCode - documentation-only release preparation`
Branch: `luna/j17a-release-notes`
Base commit: `7179087ed82a9d2055f4958d23b1e38ac366ebb1`
Worker note: `docs/worker-notes/2026-08-01-j17a-release-notes.md`

## Outcome

Draft the 0.2.0 release candidate notes and identify the candidate in README.
This is documentation-only preparation. J17 sign-off, main publication, and
tagging remain deferred.

## Objective

Create a factual 0.2.0 release-candidate document and add a concise README
release-candidate pointer without claiming that the release is signed off.

## Relevant background and existing behaviour

J16 clean native Windows verification and J17A1 product identity work are
accepted project records. The 0.2 language semantics remain the signed-off 0.1
semantics, and the active implementation directory remains `tethers-0.1/`.

## Required behaviour

1. Create `docs/releases/v0.2.0.md` with the requested candidate structure and
   accepted verification totals.
2. Update only the requested README release-candidate section and worker route.
3. Keep J17 sign-off, main publication, and tag creation explicitly pending.

## Relevant components

- `README.md`
- `docs/releases/v0.2.0.md`
- Accepted J16 and J17A1 worker notes named by the task.

## Frozen decisions and invariants

- The document is a release candidate, not a signed-off release.
- No implementation or release evidence changes.
- Do not imply support beyond native Windows and configured local stdio MCP
  providers.

## Frozen Scope

- Release notes are a candidate document only.
- The signed-off 0.1 language and `tethers-0.1/` tree remain unchanged.
- No implementation, tests, scripts, fixtures, manifests, locks, or product
  version strings change.
- Main and tags remain untouched.

## Authorised Paths

- `docs/releases/v0.2.0.md`
- `README.md`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-01-j17a-release-notes.md`

## Verification

- Release status and tag wording remain pending.
- `SIGNED OFF FOR 0.2.0` appears only as the future J17 condition.
- The packet checker, `git diff --check`, changed-path inspection, and clean
  worktree check pass.

## Acceptance criteria

1. The release note contains only supported candidate claims and the required
   sections.
2. README contains the requested release-candidate section and route.
3. The phrase `SIGNED OFF FOR 0.2.0` appears only as the future condition.
4. Exactly the four authorised paths change and all required checks pass.

## Required verification

- Run the phrase and version searches requested by the task.
- Run `check-tethers-task-packet.ps1`, `git diff --check`, changed-path, and
  final status checks.

## Stop conditions

Stop if any unsupported release claim, unauthorised path, failed check, or
conflicting source record is found. After two materially similar failures,
return the exact evidence and one smallest unresolved question.

## Expected pre-existing changes

None in the new branch before this task.

## Forbidden Changes

Do not perform J17 sign-off, publish `main`, create or modify a tag, or change
any path outside the four authorised paths.

## Completion Record

Release notes are a candidate document only. J17 sign-off remains pending.
Main and tags remain untouched. README now identifies the release candidate and
current worker route. No implementation or release evidence changed.
