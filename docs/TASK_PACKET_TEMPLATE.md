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

Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory

Toolchain preflight: `<required or N/A>`

Rust change class: `<RUST_CHANGING or NON_RUST>`

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

## Formatting and checkpoint sequence

For `RUST_CHANGING` tasks, list the authorised Rust paths and the exact Cargo
formatter command. Before the implementation checkpoint, the worker must run
that command and inspect the immediate diff. STOP if rustfmt changes any file
outside the authorised Rust paths; do not fold unrelated formatting debt into
the task.

For `NON_RUST` tasks, require `cargo fmt --all -- --check` only. The worker must
not run a mutating formatter or modify Rust source, even if the check reports a
pre-existing formatting failure.

## Completion and publication

For every `COMPLETE` task, require a normal push of the finished branch to
`origin` after the closeout commit. The completion report must state the remote
branch, full remote HEAD SHA, confirmation that local `HEAD` equals that remote
SHA, and clean `git status --short --branch` output. This authorises neither a
force-push nor a direct update to `main`.

## Forbidden changes

- No scope expansion.
- No merge, amend, tag, force-push, direct `main` update, or other publication
  unless explicitly authorised here. The required normal push of a completed
  branch is already authorised by the Completion and publication section.

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
