# Tethers repository reconciliation and deterministic plan integration

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Codex`

Route: `Fresh integration worktree from current origin/main; publish by normal non-force branch push and fast-forward/merge, then verify the remote before retiring the dirty GARY worker worktree.`

Base commit: `5cce71f8f93be26a0dfd1a0e50935f9419a5c284`

Implementation checkpoint: `d144d7f1acacd556daa4471e6cef6c93e0b64ed0`

Worker note: `docs/worker-notes/2026-09-10-tethers-main-plan-integration.md`

Suggested branch:

`integrate/tethers-main-plan-20260910`

Source branch:

`origin/main`

Updated: 2026-09-10

## Objective

Reconcile the recovered Tethers 0.5 implementation ancestry into the current
authoritative main line and ship a deterministic, machine-readable
side-effect-free `tethers plan` command. Preserve the existing Core/host
boundary, prove the plan path with the real OCaml engine fixture, publish by
normal Git operations, and remove only the obsolete dirty worker state after
remote verification.

## Relevant background and existing behaviour

The supplied GARY worker contains intentional plan-related Rust changes mixed
with disposable underscore-prefixed scratch files. The repository tag
`tethers-v0.5.8` contains the implementation ancestry that must remain
reachable. Current `origin/main` is newer than the stale SHA in the request,
so this task starts from the fetched live `origin/main` and records that exact
base.

The existing `preview` command is read-only. This task adds the stable
machine-facing `plan` contract while retaining `preview`, `run`, policy,
provider, replay, and Trail semantics unchanged.

## Required behaviour

1. Preserve both the current main ancestry and the tagged Tethers 0.5 ancestry
   without force-pushing or rewriting either line.
2. Expose `tethers plan --config --engine --input --host-data-root` with stable
   `tethers.cli/1` wrapping `tethers.plan/1` data and explicit zero-effect
   execution metadata.
3. Evaluate the selected Tether through the real Core engine while never
   requesting authority, entering policy/execution, launching a provider,
   mutating replay state, or writing a Trail execution entry.
4. Keep success, no-actions, planner-error, unavailable, interrupted, and
   invalid-input outcomes machine-readable and distinguishable.
5. Update living current-truth documentation and this control packet so they
   describe the reconciled ancestry and shipped plan surface.
6. Verify the integration, push the integration branch, publish main through a
   normal non-force fast-forward/merge, verify GitHub, and only then clean the
   supplied obsolete GARY worktree and branch.

## Relevant components

- `tethers-0.1/host-rust/src/cli.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/host-rust/src/plan_command.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/engine-ocaml/`
- `README.md`, `QUICKSTART.md`, and `docs/`
- `.github/scripts/check-tethers-task-packet.ps1`

## Frozen decisions and invariants

- `tethers-v0.5.8` remains intact and reachable; do not rewrite or retag it.
- Core plans remain requests, not permission. Host policy and provider
  execution remain outside the plan path.
- The plan command must not claim execution merely because planning succeeded.
- Provider invocation count must be zero for every plan result.
- No plan result may create a Trail execution entry or second persistence store.
- Existing public commands and Rust/OCaml trust boundaries remain compatible.
- No force push, destructive reset, or unrelated cleanup is permitted.

## Acceptance criteria

1. A merge commit preserves both `origin/main` at the recorded base and the
   peeled `tethers-v0.5.8` commit as ancestors.
2. Focused plan tests and the complete Rust verification matrix pass, with
   pre-existing warnings classified rather than hidden.
3. The real OCaml engine fixture returns a completed `tethers.plan/1` response
   with the expected action, provider marker absent, and Trail absent.
4. Invalid input returns the stable schema and nonzero structured error without
   execution effects.
5. Living docs, the worker note, and this packet contain no stale claim that
   the reconciled implementation ancestry is absent from main.
6. The integration branch and authoritative `origin/main` contain the same
   verified final history; GitHub content confirms the plan implementation.
7. The supplied GARY worker worktree and its obsolete branch are removed only
   after the final remote proof, with its intentional changes accounted for
   and scratch debris discarded.

## Required verification

- toolchain gate and task-packet checker;
- `cargo fmt`, locked check, clippy, complete locked tests, and release/build
  smoke checks;
- explicit OCaml switch `dune build @all`, `dune runtest --force`, and the
  repository fixture/MCP/demo checks supported by the live workstation;
- focused `tethers plan` unit tests, invalid-input smoke, and the real
  `j14-complete` OCaml-engine fixture;
- ancestry, diff, branch, tag, remote, and GitHub-content verification;
- final clean-worktree and branch-removal proof.

## Forbidden changes

- no provider, policy, replay, Trail, or Core semantic redesign;
- no force push, reset, checkout-overwrite, or blanket conflict resolution;
- no unrelated refactor or dependency addition;
- no committing underscore-prefixed worker scratch files;
- no deletion of the worker worktree before remote verification.

## Stop conditions

Stop and report if the current remote main moves in a way that requires
reconciliation beyond a normal merge/fast-forward, if the real engine cannot
be built or exercised after bounded diagnosis, if provider/policy/Trail effects
appear in a plan run, if a required verification fails repeatedly without new
evidence, or if the intentional worker changes cannot be accounted for safely.

## Expected pre-existing changes

None. All implementation and documentation changes for this task are made in
the fresh integration worktree; the supplied dirty GARY worker is evidence to
reconcile, not a second source of unrelated changes.
