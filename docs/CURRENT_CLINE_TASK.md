# Current Implementation Task

Control contract: `1`

Task: `J17A3 - align current project state for 0.2.0 sign-off`
Owner: `Luna`
Status: `COMPLETE`
Task colour: `Green`
Route: `Luna on OpenCode - documentation-only current-state alignment`
Branch: `luna/j17a-current-state`
Base commit: `58affc8c30ddfa9284933a5e38f598dad573f4dd`
Worker note: `docs/worker-notes/2026-08-01-j17a-current-state.md`

## Objective

Align the current-state documents with the actual completed 0.2.0 candidate and
leave J17 as the only remaining release gate.

## Relevant background and existing behaviour

J17A1 established product identity `0.2.0`; J17A2 drafted the candidate release
notes. J05 through J16 are accepted project work. J17 verification and sign-off
remain pending.

## Required behaviour

1. Install the accepted candidate checkpoint in the three current-state documents.
2. Mark J05 through J17A3 complete and leave only the J17 release queue.
3. Update the current Luna, DeepSeek, and Codex route without changing release
   notes or historical worker notes.
4. Preserve frozen boundaries and authoritative-reference links.

## Relevant components

- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`
- `docs/TASK_QUEUE.md`
- `docs/releases/v0.2.0.md` (read-only)

## Frozen decisions and invariants

- No J17 verification or sign-off is performed here.
- Product identity is `0.2.0`; language semantics remain `0.1`.
- Main and tags remain untouched.
- No implementation, tests, scripts, fixtures, manifests, locks, or version
  strings change.

## Acceptance criteria

1. The three current-state documents record candidate `58affc8c30ddfa9284933a5e38f598dad573f4dd`.
2. J17 is the only remaining release gate and no feature work is authorised.
3. Stale J04a, J05-future, and Cline route claims are absent from those documents.
4. Exactly the five authorised paths change and required checks pass.

## Required verification

- Run the four stale-state audits requested by the task.
- Run the packet checker, `git diff --check`, changed-path, and status checks.

## Forbidden changes

Do not modify release notes or historical worker notes, perform J17 verification
or sign-off, publish main, create or modify tags, or change any unauthorised path.

## Stop conditions

Stop on any conflicting source statement, stale audit result, failed check, or
unauthorised path. After two materially similar failures, return exact evidence
and the smallest unresolved issue.

## Expected pre-existing changes

None in the new branch before this task.
