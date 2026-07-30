# Current Implementation Task

Control contract: `1`

Status: `PROPOSED`

Task colour: `Green`

Owner: `<one implementation owner>`

Route: `<current worker and model/tool>`

Worker note: `docs/worker-notes/YYYY-MM-DD-short-task-name.md`

Base branch: `main`

Base commit: `<40-character implementation checkpoint>`

OCaml switch path: `<absolute directory-switch root or N/A>`

Rust toolchain: `1.89.0` (use `rustup run 1.89.0`; `--locked` mandatory)

Toolchain preflight: `<required or N/A>`

## Objective

State one independently testable outcome.

## Relevant background and existing behaviour

Include only facts that can change implementation of this task.

## Required behaviour

1. State one required behaviour.
2. Add separately testable failure or mismatch branches as separate items.

## Relevant components

- `exact/path`

## Frozen decisions and invariants

- State decisions the worker must not reopen.
- State architectural, determinism, permission, compatibility, and safety
  boundaries relevant to this task.

## Acceptance criteria

1. Pair this criterion with Required behaviour 1 and name observable evidence.
2. Pair this criterion with Required behaviour 2 and name observable evidence.

The checker requires at least as many numbered acceptance criteria as numbered
required behaviours. Do not combine several required failure branches into one
representative check.

## Required verification

List exact focused and regression commands in the order they must run. Include
complete diff and final Git status inspection.

## Forbidden changes

- No scope expansion.
- No commit, push, merge, amend, tag, or publication unless explicitly
  authorised here.

## Stop conditions

Stop on a missing architectural or safety decision, conflicting requirements,
unrelated failure that prevents trustworthy verification, or two materially
similar failed attempts. Return exact evidence and one smallest unresolved
question.

## Expected pre-existing changes

None.

If the captured pre-work tree was dirty, replace `None` with one exact path per
line:

```text
- `path/to/file`
```
