# Current Implementation Task

Control contract: `1`
Task: `J19-P1 - Host Application Seam and 0.2 Parity`
Owner: `Codex Terra High`
Status: `IN_PROGRESS`
Task colour: `Red`
Route: `Codex, bounded Rust crate-ownership extraction and parity proof`
Base branch: `main`
Accepted implementation baseline: `cfdb372ab18c7935c6046faf5cf82da2fe742440`
Frozen architecture base: `a5fd63593a9d9acd397030ecd2e27b4f318c87fd`
Branch: `codex/j19-first-plug-kit`
Worker note: `docs/worker-notes/2026-08-01-j19-p1-host-application-seam.md`

## Control-plane starting rule

Fetch `origin/main`, fast-forward the existing clean programme branch to the
commit containing this control packet, and record that exact control commit in
the worker note.

The accepted implementation baseline remains
`cfdb372ab18c7935c6046faf5cf82da2fe742440`. The control-only commit changes task
authority and is not an implementation change.

## Blocker resolution

The initial P1 attempt proved that `HostExecutionService` and its dependencies
are owned by the binary crate root. `lib.rs` currently exposes only
`child_process`, `cli`, and `engine_stdio`, while `main.rs` owns the wider module
graph and shared execution-boundary helpers.

Two bounded façade attempts failed at the same crate-ownership boundary.

Lucy authorises the smallest real structural correction:

- move shared host module ownership from the binary crate root to the library
  crate root;
- extract binary-root helpers and shared types required by host execution into a
  dedicated library application module;
- make `main.rs` a thin CLI and compatibility dispatcher over the library;
- preserve all released 0.2 behaviour.

This is the accepted P1 seam. It is not a broad host redesign.

## Objective

Create one reusable Rust library application seam around the existing host
execution machinery while preserving byte-level or semantically exact released
0.2 behaviour.

P1 ends when the library owns the shared host module graph, the binary delegates
to it, and all parity evidence passes.

Do not begin P2 Socket operations, discovery pagination, package work, File
Tools, trust, security profiles, credentials, durable stores, new public CLI, or
Tether changes.

## Required structural result

1. `lib.rs` becomes the owner of the shared host modules needed by the existing
   application and execution path.
2. `main.rs` no longer declares a duplicate host module graph.
3. Shared execution-boundary helpers and types currently trapped in `main.rs`
   move into one dedicated library-owned application module or the smallest
   coherent set of library-owned modules.
4. `HostExecutionService` compiles as library code and is reachable through a
   deliberately small public application surface.
5. Existing internal modules remain private or crate-private unless the binary
   or an external integration test genuinely requires public access.
6. The binary remains responsible only for process entry, argument capture,
   calling the library dispatcher, emitting the existing envelope/output, and
   exiting with the existing code. Legacy and debug routes may delegate through
   the application module but must retain their behaviour.
7. No duplicated implementation may remain compiled once in the library and
   again in the binary.
8. Existing file layout may be retained. Moving module declarations into the
   library does not require moving every `.rs` file.

## Authorised extraction scope

Create at most one new production module under:

`tethers-0.1/host-rust/src/`

Recommended name:

`application.rs`

A different single name is allowed only when it more accurately describes the
same boundary. Do not create an abstract framework or directory hierarchy.

The new module may own or expose only the existing application-level seams
needed for:

- command dispatch;
- prepared runtime execution;
- host execution service construction and invocation;
- shared execution-boundary mapping;
- bridge projection;
- outcome, replay, Result Anchor and Trail orchestration already implemented;
- legacy compatibility routing;
- existing debug probe delegation where necessary for parity.

Pure helpers or types may instead move into the existing module that already
owns their concept when that is smaller and avoids a dependency cycle.

## Authorised code paths

Only the minimum necessary subset of these paths may change:

- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/src/main.rs`
- one new application module under `tethers-0.1/host-rust/src/`
- existing Rust host modules whose imports, visibility, crate aliases or tests
  must change solely because ownership moves from binary root to library root
- existing Rust host integration tests solely for parity coverage
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-01-j19-p1-host-application-seam.md`

The potentially affected existing host modules are limited to:

- `approval.rs`
- `check_command.rs`
- `configured_runtime.rs`
- `dispatch.rs`
- `event_admission.rs`
- `event_queue.rs`
- `executor.rs`
- `host_execution.rs`
- `manifest.rs`
- `outcome.rs`
- `policy.rs`
- `provider.rs`
- `replay.rs`
- `replay_runtime.rs`
- `replay_windows.rs`
- `resolver.rs`
- `result_anchor.rs`
- `run_command.rs`
- `run_input.rs`
- `runtime_config.rs`
- `stdio_provider.rs`
- `trail_command.rs`
- `trusted_store.rs`
- `validation.rs`

Do not edit every listed file pre-emptively. Touch only files proven necessary by
compiler errors, dependency direction, tests, or duplicate ownership.

`Cargo.toml` may change only if Rust's automatic library/binary discovery cannot
express the accepted seam. Do not add dependencies, features, build scripts or
workspace changes. `Cargo.lock` must not change.

## Extraction laws

- Move code, do not reinterpret it.
- Preserve function bodies and data semantics unless a visibility-neutral
  wrapper is required.
- Preserve exact outcome classification.
- Preserve replay admission and terminal publication.
- Preserve Result Anchor ordering and suppression rules.
- Preserve Trail contents and ordering.
- Preserve policy, approval and scope decisions.
- Preserve engine and provider lifecycle.
- Preserve request, evaluation, action and execution identities.
- Preserve current CLI JSON envelopes, stdout/stderr placement and exit codes.
- Preserve current debug probes and legacy route unless a test proves they are
  unreachable in the released build.
- Do not make internal modules public merely to silence compiler errors.
- Do not use global mutable state, hidden singletons, new retries or new threads.
- Do not alter serialized schemas or version strings.

## Allowed public surface

Expose the smallest coherent application API.

It may include:

- the existing typed `HostExecutionService` boundary;
- its existing prepared input, result and error types;
- a library command-dispatch result carrying existing output/envelope and exit
  status;
- a small constructor or façade needed by the binary and future Socket seam.

It must not expose:

- raw replay mutation;
- raw Trail mutation;
- provider invocation that bypasses policy and durable intent;
- internal trust-store mutation;
- direct Result Anchor creation without the existing durable gates;
- a generic callback framework;
- package, Plug or Socket v1 types not already implemented.

## Baseline evidence before editing

Record the exact baseline results from the clean accepted implementation commit
for all commands that currently pass.

At minimum run:

```powershell
rustup run 1.89.0 cargo test --manifest-path tethers-0.1\host-rust\Cargo.toml --lib
rustup run 1.89.0 cargo test --manifest-path tethers-0.1\host-rust\Cargo.toml
rustup run 1.89.0 cargo build --locked --manifest-path tethers-0.1\host-rust\Cargo.toml
rustup run 1.89.0 cargo build --locked --release --manifest-path tethers-0.1\host-rust\Cargo.toml
rustup run 1.89.0 cargo fmt --manifest-path tethers-0.1\host-rust\Cargo.toml -- --check
```

Also identify the exact repository scripts for:

- consolidated 0.2 verification;
- OCaml engine tests;
- public check/run/trail proof;
- J14C real file-move proof;
- MCP transcript validation.

Run the strongest applicable existing entry points before and after extraction.
Do not guess script names when repository inspection can establish them.

## Required parity evidence

After extraction require:

1. `cargo test --lib` passes.
2. Full Rust tests pass with no ignored new failure.
3. Debug and release locked builds pass.
4. Formatting passes.
5. `Cargo.lock` hash is unchanged.
6. Existing OCaml engine tests pass.
7. Existing consolidated 0.2 verification passes.
8. Existing public `check`, `run`, and `trail` behaviour passes.
9. Existing J14C real file-move proof passes.
10. Existing MCP transcript tests pass.
11. Existing legacy route and debug probes pass where covered by repository
    tests.
12. No provider call count, Result Anchor count, Trail entry, execution identity,
    replay state, JSON envelope field or exit code changes.

Where baseline output is deterministic, compare exact normalized output before
and after. Where paths, timestamps or UUIDs vary, compare the existing structural
contract and explain the normalization.

## Focused new tests

Add only tests necessary to prove the new seam:

- library compilation owns `HostExecutionService` and dependencies;
- binary calls the library application dispatcher rather than compiling duplicate
  host modules;
- one representative no-action route is parity-identical;
- one representative denied or unavailable route is parity-identical;
- one representative successful execution route preserves intent, terminal
  outcome, Result Anchor and Trail ordering;
- public API cannot directly bypass the accepted execution gates.

Prefer existing fixtures and helpers. Do not duplicate the whole 0.2 harness.

## Forbidden work

Do not:

- implement Socket v1 semantic operations;
- modify MCP protocol behaviour;
- add discovery pagination or catalogue invalidation;
- parse `.tetherplug`;
- add package, candidate, installation, trust or conformance state;
- add File Tools or PDF Tools;
- add credentials or environment sanitation;
- change child-process security behaviour;
- alter replay or external event admission stores;
- change Tether 0.1 syntax or semantics;
- change public CLI syntax;
- remove the legacy path;
- move or recreate `v0.2.0`;
- create a release;
- begin P2.

## Stop conditions

Stop with `BLOCKED` when:

- extracting ownership requires changing a frozen semantic boundary;
- a helper cannot be placed without creating a circular authority dependency;
- CLI parity cannot be retained;
- replay, Result Anchor, Trail or outcome ordering changes;
- a new dependency appears necessary;
- broad unrelated rewrites become necessary;
- two materially similar extraction attempts fail;
- any required baseline or post-change test fails without a bounded understood
  cause.

Do not bypass a test or weaken visibility to continue.

## Git and publication

Use the existing branch:

`codex/j19-first-plug-kit`

No force-push, amend, rebase of published work, or main update.

Create one bounded P1 commit or a small reported stack when compiler-safe
checkpoints materially improve rollback.

Preferred final commit message:

`refactor: extract host application seam`

Do not begin P2 after committing P1.

## Worker note

Create:

`docs/worker-notes/2026-08-01-j19-p1-host-application-seam.md`

Use headings:

- Task
- Initial blocker
- Ownership map before
- Extraction design
- Changed paths
- Public surface
- Ownership map after
- Baseline evidence
- Post-extraction evidence
- Parity comparison
- Frozen-boundary checks
- Rollback
- Discoveries
- Remaining risks
- Next action

Record every compiler failure that materially shaped the seam, but do not paste
large logs.

## Completion report

On success begin exactly:

`P1 COMPLETE - HOST APPLICATION SEAM`

Report:

1. branch and final SHA or stack;
2. exact control-plane commit used;
3. exact changed paths;
4. module ownership before and after;
5. helpers/types moved from binary root;
6. final public application surface;
7. why `main.rs` is now a thin dispatcher;
8. baseline and final Rust totals;
9. OCaml and integration results;
10. exact parity evidence;
11. Cargo.lock hash result;
12. forbidden-work confirmation;
13. rollback commit;
14. clean worktree;
15. ahead/behind `origin/main`;
16. remaining risks;
17. explicit confirmation P2 has not begun.

On failure begin exactly:

`BLOCKED`

Report the exact smallest compiler or parity boundary, rollback state, and one
smallest unresolved question.

Stop after the report.