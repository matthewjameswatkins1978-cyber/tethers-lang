# Current Implementation Task

Control contract: `1`

Task: `TETHERS-0.4-C1C — Together Execution / Join Correction`

Owner: `OpenCode`

Status: `IN_PROGRESS`

Task colour: `Amber`

Route: `OpenCode implementation + evidence → Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-11-0.4-c1-together-fan-out-join.md`

Base branch: `feature/0.4-c1-together-fan-out-join`

Base commit: `f688954e243f4b61b4e717d367e72772735c3418`

Implementation checkpoint: `92d2a27a1c2f77c0db97cbcbe955a7d99634f83a` (C1C-1 correction; prior C1C checkpoint `6519d92a06b54c64a38f931c65da446dcebd323a`)

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory

Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1` (run; all tools present)

Rust change class: `RUST_CHANGING`

## Packet correction bookkeeping

C1 (the previously accepted planner foundation, checkpoint
`bb860e690e7469dd75d2c02f018ef57a1f8a78ef`) implemented `together` as
deterministic planner semantics and explicitly excluded host scheduling
changes. That narrowing was not authorised by Lucy's original packet: the C1
mission required the reference runtime to respect the semantic group boundary
(attempt every group member, join after all members reach terminal outcomes,
block later Actions on a non-success join). The planner implementation is
retained and remains valid. This correction restores and completes the
originally required host execution / join semantics. This is a scope
correction, not a re-implementation.

### C1C-1 correction bookkeeping

Lucy's review of the pushed C1C result found one bounded acceptance defect in
the plan-level dispatch route (`host_execution.rs::dispatch_matched_plan`):
`plan.get("groups").and_then(Value::as_array)` silently mapped a PRESENT but
non-array `plan.groups` value to `None`, making malformed top-level group
metadata indistinguishable from an absent optional field and allowing
sequential execution. The frozen rule being repaired is: never silently
reinterpret invalid group metadata as sequential execution. C1C-1 corrects
only that decode: absent `plan.groups` remains an ordinary sequential plan, a
present JSON array is passed to `build_plan_schedule` unchanged, and a present
any-other value (null / object / string / number / bool) returns
`ExecutionServiceResult::InvalidData` before any Action dispatch. A focused
production-route regression proves the rejection and that no executor/provider
was invoked. No change to `build_plan_schedule`, group execution semantics,
OCaml, dependencies, or any other C1C behaviour. Status is `IN_PROGRESS`
while the committed correction awaits final verification (full completion
suite deferred by Lucy's instruction).

## Objective

Make the reference host respect the C1 semantic group boundary: once execution
of an authorised `together` group begins, every member is attempted once
regardless of whether an earlier sibling fails; only after every member reaches
a terminal outcome does the group join; a successful join permits later
Actions, any other outcome blocks them. Serial execution remains the valid C1
reference schedule — no physical parallelism is introduced — but serial
behaviour must match what a future genuinely concurrent runtime would observe:
failure stops at the join, not inside the fan-out.

## Relevant background and existing behaviour

- C1 accepted at `bb860e690e7469dd75d2c02f018ef57a1f8a78ef` (branch
  `feature/0.4-c1-together-fan-out-join`, final pushed HEAD
  `f688954e243f4b61b4e717d367e72772735c3418`). The OCaml engine emits flat
  source-order `plan.actions` plus the additive `plan.groups` array
  (`group_id` + `member_action_ids`); a Tether without `together` produces
  byte-identical output to pre-C1; the planner Trail records one
  `group_planned` entry per group.
- The Rust host has no multi-Action execution path today: every production
  dispatch converges on `execute_shared_boundary` / `execute_boundary_impl`
  (`tethers-0.1/host-rust/src/application.rs:1969-2509`), which requires
  exactly one Action in the plan (`application.rs:2055-2058`), and
  `extract_proposed_action` (`application.rs:1150-1194`) reads only
  `plan.actions[0]` for policy. A 2+ Action plan hard-errors.
- The J13B service route (`host_execution.rs::dispatch_matched_response`,
  lines 809-1034) performs per-Action: scope assessment, effective policy
  (Deny / Ask / Unavailable / Allow), exact capability resolution, retained
  MCP session + catalogue refresh, then `execute_shared_boundary` and
  `map_shared_result`. The run CLI requires exactly one service result per
  evaluation (`run_command.rs:175-185`).
- Each Action already has its own replay claim (`LogicalExecutionKey` embeds
  `action_id`), its own `execution_id`, durable `IntentEntry` / `OutcomeEntry`,
  and presentation Trail entries with sequence continuing from the response
  Trail (`application.rs:2190`). `SharedExecutionOutcome` distinguishes
  Completed / Failed / Uncertain / Unattempted / Denied / AuditFailed /
  Replay(...); `ExecutionServiceResult` mirrors those distinctions.
- `dispatch::Trail` (`dispatch.rs:265-288`) is a sealed trait with four
  durable methods (intent, authorisation, outcome, event admission),
  implemented by `FileTrail` and the test-only `RecordingTrail`. The trail
  command parses trail JSONL generically, so a new entry kind is safe.
- The legacy `__legacy` coordinator route and the
  `execute_enabled_installed_action` adapter seams (used by m4/p3 tests) are
  single-Action paths.
- No Rust production code change happened during C1; this correction is the
  first host change for `together`.

## Required behaviour

1. Update `docs/CURRENT_CLINE_TASK.md` to this C1C packet with Status
   `IN_PROGRESS`, including the honest correction bookkeeping above, and pass
   the packet checker (`control-v1/IN_PROGRESS`) before production edits.
2. Extend the host's Plan decoding to consume the optional additive
   `plan.groups` field, keeping flat `plan.actions`, group membership by
   Action ID, source order, and per-Action idempotency keys unchanged. Plans
   with `groups` absent must continue working exactly as ordinary sequential
   plans; `"groups": []` must never be required in old output.
3. Validate group metadata before any execution and fail closed using the
   existing host error pattern (`InvalidData`): unknown member Action ID,
   duplicate Action ID within one group, an Action belonging to more than one
   group, duplicate group IDs, empty or one-member groups, and group members
   whose ordering/structure contradicts the C1 plan contract (members must be
   contiguous in source order). Never silently reinterpret invalid group
   metadata as sequential execution.
4. Build and execute a deterministic schedule from `plan.actions` + optional
   `plan.groups` without a DAG framework: sequential items keep the existing
   stop-on-first-non-success behaviour; a `together` group attempts every
   member once in source order even when an earlier sibling fails; the group
   joins successfully only when every member succeeded; a non-success join
   blocks all later items. No group-wide idempotency keys; no retries,
   rollback, compensation, cancellation, or nested groups.
5. Reuse the current production Action execution function: refactor
   `execute_boundary_impl` / `execute_shared_boundary` / `authorise_and_execute_inner`
   to receive the exact Action to dispatch (removing the
   exactly-one-Action-in-plan gate) and drive every sequential item and every
   group member through that same production boundary. Do not create a
   test-only execution path. Keep the legacy route and the
   `execute_enabled_installed_action` adapter seams fail-closed for
   non-single-Action plans (explicit error, no silent reinterpretation).
6. Record group execution evidence: one durable `GroupJoinEntry` on the host
   Trail (evaluation_id, group_id, member_action_ids, joined, timestamp) via a
   small generic `dispatch::Trail` extension, plus one presentation
   `group_joined` Trail entry in the response (outcome `success` /
   `non_success`). Preserve every per-member outcome distinction (including
   `uncertain`), the planner `group_planned` evidence, and the "all members
   attempted" legibility. The aggregate result for a non-success group must
   preserve the first non-succeeded member's outcome (in source order) rather
   than flattening it.
7. Add the Three Bunny Breakfast production-path crucible driving the real
   execution seam: TB-00 (sequential A B C, B fails: A attempted, B
   attempted→failed, C NOT attempted), TB-01 (together carrot/toast/coffee all
   succeed, join success, report attempted), TB-02 (toast fails: carrot
   attempted, toast failed, coffee attempted, join non-success, report NOT
   attempted), TB-03 (carrot fails: same shape), TB-04 (one member uncertain:
   all siblings attempted, join non-success, later Action blocked, uncertain
   preserved) — TB-04 only if the existing outcome machinery can produce it
   cleanly through the production seam.
8. Add focused host tests proving malformed group metadata fails closed, at
   minimum unknown member Action ID and duplicate membership, plus the other
   validated malformed shapes.
9. Create `docs/ROAD_TO_0_4.md` with the provisional 0.4 roadmap
   (C1 Together semantic foundation; C2 Physical parallel execution;
   C3 Concurrency limits / resource bounds; C4 Adversarial concurrency
   crucible; C5 Fresh-agent concurrency proof), the design principle
   "Concurrency belongs in Tethers semantics. Parallelism mostly belongs in
   the runtime.", and an explicit note that C2–C5 are provisional and not
   started.
10. Run the full Rust completion authority required by the repository guide
    against the committed checkpoint: focused Together/Three Bunny tests,
    `cargo test --all-targets --all-features --locked`, `cargo check` with the
    repository warning policy, `cargo clippy` with the repository warning
    policy, `cargo fmt --check`, `git diff --check`, packet checker
    `control-v1/COMPLETE`. Also rerun the relevant OCaml/engine compatibility
    suites to prove the accepted `together` plan still round-trips into the
    host. Do not modify OCaml code unless a genuine integration defect is
    revealed; if OCaml remains untouched, say so explicitly. No new
    dependency is expected; if none are added, record "No dependency changes."
11. Close out per project control: commit the implementation checkpoint, write
    the worker note at the named path (continuing/updating the C1 worker note
    with the correction record), set the packet to `COMPLETE`, require checker
    `control-v1/COMPLETE`, commit the docs-only closeout, push the branch
    normally to `origin`, resolve the full remote HEAD SHA, confirm local
    `HEAD == remote HEAD`, and confirm a clean worktree.

## Relevant components

- `tethers-0.1/host-rust/src/plan_execution.rs` (new: schedule build/validate, serial group execution loop, join aggregation)
- `tethers-0.1/host-rust/src/dispatch.rs` (Plan `groups` decode types, `GroupJoinEntry`, `Trail::append_group_join`)
- `tethers-0.1/host-rust/src/application.rs` (`execute_boundary_impl` / `execute_shared_boundary` / `authorise_and_execute_inner` Action parameter, `extract_proposed_action_at`, legacy-route and test callers)
- `tethers-0.1/host-rust/src/host_execution.rs` (`execute_one_action` extraction, plan-level dispatch route)
- `tethers-0.1/host-rust/tests/` or module tests (Three Bunny Breakfast crucible, malformed-metadata regressions)
- `docs/ROAD_TO_0_4.md` (new roadmap)
- `docs/CURRENT_CLINE_TASK.md` (packet), `docs/worker-notes/2026-08-11-0.4-c1-together-fan-out-join.md` (continued worker note), `docs/PROJECT_DASHBOARD.md` (closeout)
- Read-only references: `tethers-0.1/engine-ocaml/` (accepted C1 planner; do not modify), `tethers-0.1/protocol/` fixtures

## Frozen decisions and invariants

- Keep the accepted C1 planner foundation: flat `plan.actions`, `plan.groups`
  membership by Action ID, source-order canonicality, stable Action IDs,
  deterministic group IDs, planner `group_planned` Trail evidence, malformed
  group refusal, byte-compatible output for Tethers without `together`.
- C1C establishes execution semantics only: the serial reference schedule is a
  valid C1 schedule; no physical parallel execution, threads, worker pools,
  provider multiplexing, or async-runtime migration.
- Failure stops at the join, not inside the fan-out. Every group member is
  attempted at most once through the group execution path; existing
  idempotency rules remain in force; no group-wide idempotency keys.
- A group succeeds only when every member has a success outcome. A member
  outcome of Completed or replay-blocked completed-success counts as success;
  Failed / Uncertain / Denied / Unattempted / AuditFailed / other replay /
  approval-required are non-success. Non-success distinctions are preserved in
  the aggregate (first non-success in source order), never flattened.
- Existing Action/Effect authorisation remains authoritative per Action; no
  group-level permissions; a Plan remains a request, not permission.
- The Core/host boundary, replay identity material, one-shot approval rules,
  and Trail-ownership semantics are unchanged. The legacy route and adapter
  seams stay single-Action and fail closed explicitly for larger plans.
- No OCaml, dependency, toolchain, Dune, or lockfile changes; no change to
  sequential Tether semantics or existing fixtures; P6 implementation and
  evidence remain untouched.
- New Trail/JSONL entry kinds are additive and legible; planner `group_planned`
  evidence is retained.

## Acceptance criteria

1. Packet is the C1C correction packet, Status `IN_PROGRESS`, checker reports
   `control-v1/IN_PROGRESS` before production edits; the correction
   bookkeeping is present in the packet and final worker note.
2. Host consumes `plan.groups` additively; a plan without `groups` executes as
   an ordinary sequential plan with identical behaviour; no test requires
   `"groups": []`.
3. Host tests prove every listed malformed group shape (unknown member Action
   ID, duplicate membership, duplicate group IDs, empty/one-member group,
   non-contiguous members) is rejected with `InvalidData` (or the equivalent
   existing host error pattern) before any dispatch.
4. Crucible evidence proves: sequential stop-on-first-failure unchanged;
   group members all attempted once in source order regardless of sibling
   failure; join success only when all members succeeded; later Actions
   blocked on non-success join; no retries or cancellation; no group-wide
   idempotency key in any produced record.
5. `execute_shared_boundary` is driven per Action through one refactored
   production path used by both sequential items and group members; legacy and
   adapter seams fail closed explicitly for non-single-Action plans.
6. Durable Trail contains one `GroupJoinEntry` per group with evaluation_id,
   group_id, member_action_ids, joined flag, and timestamp; the response Trail
   contains a `group_joined` presentation entry; per-member outcomes and the
   planner `group_planned` entry remain present; `uncertain` is not flattened.
7. TB-00, TB-01, TB-02, TB-03 pass through the real execution seam; TB-04 is
   implemented if a clean uncertain path exists, otherwise its omission is
   recorded with the reason.
8. Malformed-metadata regression tests exist and pass, covering at minimum
   unknown member Action ID and duplicate membership.
9. `docs/ROAD_TO_0_4.md` exists with the C1–C5 provisional roadmap, the
   concurrency/parallelism design principle, and an explicit not-started note
   for C2–C5.
10. Full Rust gate passes against the committed checkpoint (focused tests,
    full `cargo test --locked`, `cargo check` and `cargo clippy` with the
    repository warning policy, `cargo fmt --check`); OCaml engine suites
    (fixtures, engine cases, MCP transcripts) pass unchanged; OCaml untouched
    (stated explicitly) or a genuine integration defect fixed; "No dependency
    changes." recorded.
11. Closeout evidence: worker note at the named path with the implementation
    checkpoint SHA, checker `control-v1/COMPLETE`, branch pushed normally to
    `origin`, full remote HEAD SHA resolved, local `HEAD == remote HEAD`, and
    `git status --short --branch` clean.

## Required verification

1. Packet checker at start (`control-v1/IN_PROGRESS`) and on closeout
   (`control-v1/COMPLETE`):
   `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`
2. Rust formatter (RUST_CHANGING; run before the implementation checkpoint and
   inspect the immediate diff; STOP if rustfmt touches any file outside the
   authorised Rust paths — `tethers-0.1/host-rust/src/plan_execution.rs`,
   `dispatch.rs`, `application.rs`, `host_execution.rs`, and the test files
   this packet adds):
   `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all`
   then `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`
3. Focused Together/Three Bunny and malformed-metadata tests:
   `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --locked plan_execution`
   (and the full suite below for final authority)
4. Full Rust completion authority:
   `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked`
   `$env:RUSTFLAGS="-D warnings"; cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked`
   `cargo clippy --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked`
5. OCaml/engine compatibility rerun (unchanged source; prove round-trip):
   `opam exec --switch=<OcamlSwitchPath> -- dune build`
   `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\check-fixtures.ps1`
   `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-engine.ps1`
   `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-mcp-transcripts.ps1`
6. `git diff --check`, complete diff inspection, and final
   `git status --short --branch` inspection.

## Formatting and checkpoint sequence

RUST_CHANGING task: run the packet's Cargo formatter command
(`cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all`) before the
implementation checkpoint and inspect its immediate diff. Stop if rustfmt
changes any file outside the authorised Rust paths. The engine has no project
formatter; preserve local OCaml style. The implementation checkpoint commit
precedes all worker-note, packet, and dashboard closeout edits. `docs/ROAD_TO_0_4.md`
is implementation scope and precedes the checkpoint commit; the packet, worker
note, and `docs/PROJECT_DASHBOARD.md` are closeout scope.

## Completion and publication

Commit the implementation/proof checkpoint, continue/update the worker note at
the named path, set this packet to `COMPLETE`, require checker
`control-v1/COMPLETE`, commit the docs-only closeout, then push the named
branch normally and prove `origin/feature/0.4-c1-together-fan-out-join == HEAD`
with a clean worktree. Do not start C2 or any physical-parallel increment.

## Forbidden changes

- No physical parallel execution, Tokio, async-runtime migration, threads for
  group execution, worker pools, provider multiplexing, or simultaneous MCP
  requests.
- No nested `together`, DAG engine, dynamic fan-out, cancellation, retries,
  rollback, compensation, priority scheduling, or resource quotas.
- No C2 work, HQ work, or unrelated cleanup.
- No change to the accepted C1 planner semantics, Action identities,
  idempotency material, existing error contracts, or existing
  fixtures/transcripts; no OCaml change unless a genuine integration defect is
  revealed.
- No dependency, toolchain, Cargo.lock, Dune, or OCaml-version changes.
- No merge, amend, tag, force-push, PR, or direct `main` update.

## Stop conditions

- A real contradiction between the frozen C1 execution semantics and repository
  evidence that cannot be resolved from this packet.
- Evidence that the current host cannot represent a required terminal outcome
  without a consequential redesign (in that case return the architecture
  evidence rather than inventing a new outcome type).
- A protocol/version decision beyond this packet.
- Two materially similar implementation attempts fail on the same unresolved
  underlying problem.
- An unrelated environmental failure prevents trustworthy verification of a
  required check.

## Expected pre-existing changes

None. Base commit is the accepted C1 final pushed HEAD; the C1C branch
continues clean at it.
