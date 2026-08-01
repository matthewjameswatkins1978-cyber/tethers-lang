# Current Implementation Task

Control contract: `1`
Task: `J19-M1 - Autonomous Socket Parity Programme`
Owner: `Codex Terra High`
Status: `IN_PROGRESS`
Task colour: `Red`
Route: `Codex, autonomous Rust restructuring and Socket parity implementation`
Base branch: `main`
Accepted implementation baseline: `cfdb372ab18c7935c6046faf5cf82da2fe742440`
Frozen architecture base: `a5fd63593a9d9acd397030ecd2e27b4f318c87fd`
Branch: `codex/j19-first-plug-kit`
Worker note: `docs/worker-notes/2026-08-01-j19-m1-socket-parity.md`

## Control-plane starting rule

Fetch `origin/main`, fast-forward the existing clean programme branch to the
commit containing this control packet, record the control commit in the worker
note, and continue on `codex/j19-first-plug-kit`.

The accepted implementation baseline remains
`cfdb372ab18c7935c6046faf5cf82da2fe742440`. Control-only commits after that SHA
change task authority, not runtime semantics.

## Objective

Complete Milestone 1 from the accepted J18I roadmap:

1. P1-SOCKET-PARITY;
2. P2-SOCKET-BOUNDARY;
3. P3-DISCOVERY-CATALOGUE.

Codex owns ordinary source-layout, module, visibility, test-placement and commit
choices required to complete this milestone. Lucy will review the finished
milestone diff and evidence. Do not stop merely because the safest implementation
is larger than an earlier guessed file count or requires a different Rust module
layout.

Return only at:

`M1 COMPLETE - SOCKET PARITY`

or on a genuine stop condition defined below.

## Blocker resolution and autonomy rule

The initial P1 attempts proved that the useful host execution machinery is
binary-crate-root coupled. `main.rs` owns the host module graph, shared
execution-boundary helpers and a large body of legacy tests, while `lib.rs`
exports only a small foundation.

The following are explicitly authorised and are not blockers:

- move shared host module ownership from `main.rs` to `lib.rs`;
- make `main.rs` a thin process-entry and CLI compatibility dispatcher;
- create as many coherent Rust modules as are reasonably needed for the
  extraction, rather than forcing everything into one file;
- split application dispatch, legacy compatibility, event-drain helpers and
  debug probes into separate concept-owned modules when that reduces coupling;
- move unit tests out of `main.rs` into the module that owns the tested code;
- move broad parity tests into `tests/` integration-test files;
- preserve tests byte-for-byte where useful, or refactor test helpers when the
  assertions and covered behaviour remain equivalent or stronger;
- change `pub`, `pub(crate)` and private visibility as needed while keeping the
  external public API deliberately small;
- change import paths, crate aliases and module declarations;
- change `Cargo.toml` only when required to declare or configure existing
  library/binary/test targets, with no new dependency unless a genuine blocker
  is reported;
- create a bounded commit stack and reorganise it before first publication;
- choose the exact extraction order and temporary intermediate layout;
- use compiler errors and tests to guide the dependency untangling;
- make ordinary engineering decisions without requesting Lucy approval.

`Cargo.lock` must remain unchanged unless a separately reported dependency
change is genuinely required. No dependency change is currently authorised.

A source-layout decision, number of touched Rust files, movement of existing
tests, visibility adjustment, or temporary compilation break during the local
refactor is not grounds for a BLOCKED report.

## Required P1 result

P1 is complete when:

1. the library crate owns the shared host module graph;
2. `HostExecutionService` and its required types compile as library code;
3. one deliberately small public application surface exists;
4. the binary delegates to the library rather than compiling duplicate host
   implementations;
5. `main.rs` contains only process entry, argument capture, library dispatch,
   output emission and process exit, plus the smallest unavoidable platform
   glue;
6. legacy and debug routes still behave as before, regardless of which module
   now owns them;
7. existing 0.2 CLI behaviour, envelopes, exit codes and ordering remain
   compatible;
8. no P2 Socket semantics have been introduced merely to finish P1;
9. parity and full regression evidence pass.

Codex decides the concrete module map. A reasonable shape may include
`application`, `legacy`, `event_flow` or test modules, but these names and counts
are not requirements.

## P2 authorised scope

After P1 passes locally, continue automatically to P2.

Place retained MCP stdio provider sessions behind a semantic Socket boundary
covering the accepted Socket v1 operations:

- establish;
- discover;
- invoke;
- observe_result;
- observe_catalogue_change;
- probe;
- close.

The Socket returns observations only. It does not own trust, policy, approval,
credentials, canonical outcomes, replay, Result Anchors, Trail or retries.

The first binding remains MCP `2025-11-25` over local stdio. Standard MCP
methods remain the wire mapping. Do not invent custom Socket wire methods.

Preserve:

- one active invocation per session;
- monotonically unique session-local JSON-RPC request IDs;
- no batching;
- no parallel invocation;
- no hidden restart queue;
- no automatic retry;
- exact executable/argument launch behaviour currently in the legacy path;
- bounded close and child cleanup;
- protocol stdout separate from diagnostic stderr.

Codex owns the exact trait, structs, enums and module layout needed to express
this boundary.

## P3 authorised scope

After P2 passes locally, continue automatically to P3.

Implement the accepted discovery and catalogue behaviour:

- consume every `tools/list` page;
- treat cursors as opaque;
- detect repeated or looping cursors and fail closed;
- reject duplicate operation names;
- validate live input and output schemas against trusted binding expectations;
- preserve provider descriptions and annotations only as untrusted observations;
- represent a complete catalogue snapshot;
- make catalogue-change notification mark discovery stale;
- prevent affected invocation while stale;
- perform bounded rediscovery through the host-owned lifecycle;
- retain exact unchanged bindings;
- make missing, changed or incompatible bindings unavailable or quarantined as
  required by the accepted contracts;
- leave unapproved additions unavailable;
- never convert catalogue notifications into Tethers Anchors.

Codex owns the exact catalogue types, pagination implementation and transcript or
fixture layout.

## Frozen decisions and invariants

- Tethers Core remains deterministic and application-agnostic.
- The Rust host owns trust, policy, approval, credentials, dispatch, outcomes,
  replay, event admission, conformance and Trail.
- Providers own vendor-specific translation.
- Socket semantics, protocol binding and byte transport remain distinct.
- Attempted operation outcomes remain exactly `succeeded`, `failed` and
  `uncertain`.
- Unattempted remains a disposition, not a fourth outcome.
- No automatic retry or restart retry exists.
- Idempotency does not authorise retry.
- Event admission remains separate from operation outcomes.
- Durable outcome and replay-terminal publication precede Result Anchor
  creation.
- No Result Anchor exists for unattempted work.
- Supervised execution is not hostile-code isolation.
- Tether language syntax and semantics remain `0.1`.
- Released `v0.2.0` and its history remain unchanged.

## Relevant components

Codex may inspect and modify the minimum coherent set under:

- `tethers-0.1/host-rust/src/`;
- `tethers-0.1/host-rust/tests/`;
- `tethers-0.1/host-rust/Cargo.toml` only under the target-configuration rule;
- existing MCP fixtures/transcripts and test scripts required for P2/P3 evidence;
- `docs/CURRENT_CLINE_TASK.md` for status only;
- `docs/worker-notes/2026-08-01-j19-m1-socket-parity.md`.

Existing runtime configuration, capability manifests, providers and scenarios may
be used unchanged as regression evidence. Do not reinterpret them as
`.tetherplug` v1.

## Required behaviour

1. Preserve released 0.2 behaviour while moving shared host ownership into the
   library.
2. Complete P1, P2 and P3 without asking for ordinary source-layout approval.
3. Keep every commit reviewable and the legacy route recoverable.
4. Run focused tests during development and the complete required evidence before
   reporting M1.
5. Record architectural discoveries and deliberate layout choices in the worker
   note.
6. Do not begin package inspection, trust, File Tools packaging or Milestone 2.

## Acceptance criteria

M1 is accepted for review only when all of the following are true:

1. P1, P2 and P3 each have identifiable commits or a clearly mapped commit
   stack.
2. The library owns the reusable application and Socket seams.
3. The binary is thin and has no duplicate host implementation.
4. Existing `check`, `run`, `trail`, legacy, replay and debug-probe behaviour is
   compatible.
5. Existing policy, approval, dispatch, outcome, replay, Result Anchor, event
   admission and Trail ordering remain intact.
6. Socket operations preserve serial invocation and no retry.
7. Discovery proves complete pagination, repeated-cursor refusal, duplicate-name
   refusal, schema drift handling and catalogue invalidation.
8. Catalogue notifications are not Anchors.
9. Full Rust and OCaml regression passes.
10. Existing real file-move and public 0.2 verification passes.
11. Formatting, locked builds, packet checks and whitespace checks pass.
12. `Cargo.lock` remains unchanged.
13. Worktree is clean and the programme branch is pushed normally.
14. No Milestone 2 implementation has begun.

## Required verification

Use focused tests freely during implementation. Before the M1 report, run at
least:

- `rustup run 1.89.0 cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --lib`;
- complete Rust tests for the host crate;
- locked debug build;
- locked release build;
- `cargo fmt --check`;
- existing MCP transcript and provider tests;
- existing public `check`, `run` and `trail` verification;
- existing real local file-move proof;
- full OCaml engine tests and demo through the established Windows/opam route;
- repository JSON/JSONL fixture checks;
- task-packet checker;
- `git diff --check`;
- `Cargo.lock` hash comparison against the recorded baseline.

Add focused parity, Socket and discovery tests where existing suites do not prove
the acceptance criteria.

Do not suppress, ignore or delete a failing regression merely to complete the
milestone.

## Forbidden changes

Do not begin or implement:

- `.tetherplug` parsing or extraction;
- quarantine or installed registries;
- signatures, publisher trust or revocation;
- clean-environment security profiles beyond preserving existing launch
  behaviour;
- credentials or credential delivery;
- conformance stores;
- durable external-event admission;
- packaged File Tools or PDF Tools;
- new public Plug lifecycle CLI;
- Jobs, Streams or Human Tasks;
- network providers or listeners;
- Tether syntax or semantic changes;
- release tags or GitHub Releases.

Do not move or rewrite `v0.2.0` history.

## Stop conditions

Do not stop for ordinary refactoring uncertainty.

Continue, choose a reasonable implementation, test it, and record the choice when
the issue concerns:

- module names or count;
- exact source-file boundaries;
- movement of existing tests;
- visibility and import paths;
- helper placement;
- whether a parity test is unit or integration level;
- intermediate commit structure;
- compiler-guided dependency untangling;
- a larger-than-estimated but still bounded P1 diff.

Stop and begin `BLOCKED` only when one of these remains after at least two
materially different, evidence-based attempts:

- the frozen architecture must change;
- released 0.2 semantics cannot be preserved;
- a required regression fails for reasons not caused by ordinary test relocation
  or harness adjustment;
- a new third-party dependency appears necessary;
- public CLI or machine schemas must change;
- outcome, replay, admission or Result Anchor truth would change;
- a security or durable-state boundary outside M1 must be implemented;
- source evidence reveals existing corruption or unsafe behaviour that cannot be
  contained within M1;
- work would cross into Milestone 2;
- Git history would require force, rebase of published work or release-ref
  mutation.

A BLOCKED report must include the exact failing command, both attempted
approaches, smallest relevant diff or compiler evidence, external effects, safe
rollback and one concrete decision that cannot reasonably be made by the
implementation owner.

## Expected pre-existing changes

The programme branch is expected to be clean and equal to the prior control
commit before fast-forwarding to this packet. No P1 implementation commit is
expected yet.

## Worker note

Create and maintain:

`docs/worker-notes/2026-08-01-j19-m1-socket-parity.md`

Record:

- control commit and branch base;
- baseline commands and totals;
- chosen source layout and why;
- P1, P2 and P3 commit map;
- moved tests and preserved assertions;
- public library surface;
- compatibility evidence;
- Socket identity and lifecycle decisions;
- discovery pagination and invalidation evidence;
- regressions encountered and resolutions;
- exact final checks;
- remaining risks;
- rollback points;
- confirmation that Milestone 2 did not begin.

## Commit and publication boundary

Use bounded commits with descriptive messages. A suggested stack is:

- `refactor: extract host application library seam`;
- `refactor: add semantic socket boundary`;
- `feat: complete socket discovery catalogue`;
- optional focused test or correction commits.

This wording is not mandatory. The packet-to-commit mapping must be clear.

Push only:

`codex/j19-first-plug-kit`

Do not push `main`, tags or releases.

## Completion report

Begin exactly:

`M1 COMPLETE - SOCKET PARITY`

Report:

1. branch and final SHA;
2. control commit;
3. P1/P2/P3 commit map;
4. changed paths by packet;
5. final source layout;
6. public library/application/Socket surfaces;
7. test commands and exact totals;
8. 0.2 compatibility evidence;
9. Socket lifecycle evidence;
10. pagination, cursor, duplicate, drift and invalidation evidence;
11. frozen-boundary checks;
12. `Cargo.lock` hash result;
13. rollback points;
14. remaining risks;
15. clean worktree;
16. ahead/behind main;
17. confirmation Milestone 2 did not begin.

On a genuine stop condition begin exactly:

`BLOCKED`

Stop after the report.