# Worker Note — C2-A3a — Final Provider Overlap Correction

Task: `C2-A3a — Final Provider Overlap Correction`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Mini 2.5 Pro — independent final verification`

Status: `COMPLETE`

Base commit: `58aecd0c789802cdfea57d4560b51fd21d5340ae`

Implementation checkpoint: `65acab1c9dfc7ceab16cf36164a22f94cfc69b17`

## Requested outcome

Implement only the approved C2-A3a narrow core correction at
58aecd0c789802cdfea57d4560b51fd21d5340ae: retain each member's exact C1
ActionStep through terminal state and semantic-order aggregation, use
structural ownership movement without fabricated production placeholders, and
prove real two-member provider-effect overlap. The broader C2-A3a matrix is
explicitly deferred to separately authorised work.

## Changes made

The task is now `IN_PROGRESS`. The concurrent-group state retains complete
`ActionStep` values at every terminal path and uses `step_succeeded` plus
`aggregate_step` in Runtime Plan member order. Stage B/C now move the real
ready action, prepared invocation data, and replay admission through enum
transitions and `Option::take`; the no-op replay guard and dummy executor/test
capability/manifest production helpers were removed. The Stage C mpsc worker
channel remains intact. A child-process barrier fixture and same-provider and
different-provider overlap tests prove two simultaneous `tools/call` effects.

## Decisions and assumptions

The coordinator/worker overlap architecture stays as designed: serial Stage A
preparation, `std::thread::scope` workers, mpsc result delivery, coordinator
ownership of ReplayAdmission and Trail, ephemeral trusted RetainedProviderSession
worker paths, and a preserved sequential non-Together path. Frozen boundaries:
all Together members attempted through fan-out semantics, sibling failure does
not cancel siblings, join waits for all terminal members, first non-success
selected by semantic Runtime Plan member order, ReplayAdmission stays
coordinator-owned and need not become Send, workers own provider invocation
material only, and no Tokio/async or C3 resource scheduling.

## Evidence

Baseline verified before editing: branch `feature/c2-a3a-provider-overlap`,
formal base `58aecd0c789802cdfea57d4560b51fd21d5340ae`, activation HEAD
`671e95931d375424949041a2d35a958dfae5d6ae`, fetched
`origin/main` `1703fb4aadc06980daea8fe5afbeaf3a6218b256`. Device-tool diagnostic
(`scripts/check-dev-tools.ps1`) reported all required tools present. The 16
pre-existing untracked paths are recorded in the packet's Expected pre-existing
changes section and preserved exactly. Packet-checker result
(`.github/scripts/check-tethers-task-packet.ps1`) passed in `IN_PROGRESS`
state. Focused C2-A3a overlap tests and focused host execution tests are the
required narrow-core evidence; the full deferred matrix is not a completion
claim.

Implementation checkpoint: `f6eeb572b2afe42472d2deab6336a1af971f649f`
Remote HEAD: `f6eeb572b2afe42472d2deab6336a1af971f649f`
origin/main: `1703fb4aadc06980daea8fe5afbeaf3a6218b256` (unchanged)

### Focused test results (all PASS)

| Test | Result |
| --- | --- |
| same-provider actual tools/call overlap | ok |
| different-provider actual tools/call overlap | ok |
| prompt Stage C durability while sibling blocked | ok |
| physical Trail order B before A | ok |
| physical Trail order A before B | ok |
| GroupJoin after all terminals | ok |
| worker panic yields uncertain non-success join | ok |
| semantic first non-success preserves exact step | ok |

### Verification results

| Check | Result |
| --- | --- |
| `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check` | PASS |
| `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --locked` | PASS |
| `git diff --check` | PASS |
| `.github/scripts/check-tethers-task-packet.ps1` | PASS (control-v1/IN_PROGRESS) |

### Correction pass changes

1. **Fixture updated** (`tethers-stdio-fixture.ps1`): Added per-member release
   control via `release-member-{tag}` files alongside the shared `release` file.
   Regex extracts member tag from message (`hello-from-a`/`member-a` → `a`).
   Falls back to PID-based token for legacy tests.

2. **Test harness rebuilt** (`host_execution.rs`): Replaced five separate
   `execute_group_concurrent`-based tests with a single reusable
   `C2A3aGroupHarness` struct. All five coordinator-observability tests now
   exercise the real `execute_group_concurrent` → `worker_invoke_provider` →
   `catch_unwind` → Stage C → Trail → GroupJoin path.

3. **Replay authority fixed**: `TestReplayAuthority` (test-only, in-memory)
   replaces `FileReplayAuthority` for the group-level tests. This eliminates
   Windows-specific `ReplayLedger` directory provisioning issues while still
   exercising the full `ReplayAdmission` contract.

4. **Panic injection seam** (`#[cfg(test)] static INJECT_WORKER_PANIC_ACTION_INDEX`):
   Cross-thread atomic replaces thread-local `INJECT_WORKER_PANIC`. Panic is
   injected inside `catch_unwind` boundary in the real worker thread, targeting
   by `action_index`. `PanicGuard` RAII ensures reset even on assertion panic.

5. **Manifest bridge pins**: Actions now include `manifest_digest` extracted
   from verified runtime providers, satisfying `verify_action_bridge_pins`.

6. **Runtime config**: Distinct capability names (`fixture.ping-a`,
   `fixture.ping-b`), `path_prefix` scope binding with `allowed_prefixes:
   ["member/"]`, `standing_permitted: true, per_call_required: false`, policy
   default `deny` with explicit `allow` rules for both capabilities.

## Discoveries

The previous `NO ACTIVE IMPLEMENTATION PACKET` state in `docs/CURRENT_CLINE_TASK.md`
blocked Codex correctly: the checker required a full Base commit SHA and
control-v1 packet fields that the stale file did not provide. This activation
resolves that exact defect without changing any implementation.

## Remaining risks

The deferred matrices named in the earlier correction pass are now complete and
covered by the terminal semantic matrix tests; this note's earlier
`IN_PROGRESS` risk paragraph is superseded.

Two non-blocking residual items remain:

1. **Test replay authority vs. durable filesystem persistence.** The
   group-level and terminal-matrix tests exercise the `ReplayAdmission`
   contract through `TestReplayAuthority` and the test-only
   `ObservingReplayAuthority`. Durable Stage C persistence is proven against a
   real file-backed `FileTrail`, but replay persistence with
   `FileReplayAuthority` (with a provisioned replay root) is not exercised by
   these focused tests. A later task may add a `FileReplayAuthority` variant.

2. **Dead-code warnings in this branch.** `cargo check --all-targets
   --all-features` reports 49 warnings (5 duplicates) in the lib test target
   plus 5 in the production lib: unused fields (`PreparedInvoke.resolved`,
   `.decision`, `.bridge_pins_required`, `.action`; `GroupMemberState`'s
   `action_index`/`semantic_position` in its terminal/transition variants), an
   orphaned `ResolvedCapability::new` constructor (left over from removing
   `test_resolved_capability`), and several unused test helper functions
   (`c2a3a_barrier_runtime`, `c2a3a_establish_sessions`, `c2a3a_actions`,
   `c2a3a_matched_response`, `c2a3a_member_provider`, `poll_until`,
   `entry_kinds`, `timeout_b_ms`). These are non-semantic, non-architectural
   clean-up items that do not affect determinism, trust boundaries, or
   provider behaviour; the production build (`cargo check` without
   `--all-targets`) still finishes with zero errors and the required checks all
   exit 0.

## Smallest next action

Lucy merge review of the published `feature/c2-a3a-provider-overlap` branch.
No further implementation is required before that review; the dead-code
clean-up items above are candidates for a separate bounded follow-up task at
Lucy's discretion.

## Correction pass — terminal semantic matrix hardening

Owner `DeepSeek Flash`. Four review defects were corrected without widening
the accepted architecture.

1. **Unavailable (Defect 1).** `remove_provider_a` deleted member-a's semantic
   Action from the Runtime Plan. Replaced with `provider_a_unavailable`, which
   keeps provider-a configured and member-a's Action present, and only excludes
   provider-a from the host availability snapshot. The exact production
   `ExecutionServiceResult::Unavailable` path is reached, member-b still invokes
   and succeeds, `member_action_ids` holds both members, and `joined == false`.

2. **ReplayBlockedCompletedSuccess (Defect 2).** The prior test permitted a
   weakened "not failure" assertion. Rewritten to require exactly
   `ExecutionServiceResult::Completed`, `joined == true`, member-b success, and
   member-a admitted as recovered `Succeeded` (via an observing replay trace).

3. **Inverse physical completion (Defect 3).** The prior test panicked A before
   any provider completion, so no physical order was actually inverted. Now A
   produces a real provider `Uncertain` and B a real provider `Failed` via
   per-member fixture outcome control, with independent release. Both runs
   assert physical OutcomeEntry order (B→A and A→B) and that the final
   aggregate is always semantic member-a `Uncertain`.

4. **G1-before-effect (Defect 4).** Replaced Trail-order inference with a
   test-only `ObservingReplayAuthority` / `ReplayTrace` that records G0/G1/G2
   per member. G1 is asserted observed before releasing provider effect, and
   G0→G1→G2 ordering is asserted after completion. Replay admission ownership
   remains coordinator-owned and `!Send`.

Panic exact classification now asserts exactly `Uncertain` for member-b; intent
evidence uses mandatory `expect`/`assert` rather than optional `if let`.

### Fixture defect discovered and fixed

`tethers-stdio-fixture.ps1` used
`(Get-ChildItem -Filter 'entered-*').Count`, which throws
`ParentContainsErrorRecordException` under `Set-StrictMode -Version Latest`
when exactly one file matches (scalar `FileInfo` has no `.Count`). This made
member-b fail with `NoFinalResponse` in every single-member test. Fixed to
`@(Get-ChildItem ...).Count`. Added `peer-count` and per-member
`outcome-{tag}` control files.

## Independent final verification

Owner `Mini 2.5 Pro` performed the independent final review and verification
gate. Result: **C2-A3a verified complete**. The architecture matches the
accepted C2-A3 design; no semantic drift was found.

Reviewed directly (not from prior reports):

- **Core ownership.** `execute_group_concurrent` keeps `DispatchReadyAction`,
  `PreparedInvoke`, and `ReplayAdmission` coordinator-owned; workers receive
  only `WorkerInput` (arguments, provider, tool name, remaining). Structural
  movement uses whole-enum `std::mem::replace` and `Option::take`. No
  `DispatchReadyAction::new`, `test_manifest`, `test_resolved_capability`,
  `NoopCapabilityExecutor`, or `NoopReplayAdmissionGuard` remains.
- **Worker path.** `worker_invoke_inner` is
  `RetainedProviderSession::establish` → `refresh_prepared_catalogue` →
  `session.tools_call` → `session.close`; no direct `ManagedProvider` shortcut.
  `catch_unwind` surrounds real worker work and always emits one
  `WorkerResult`, so there is no channel-result hole.
- **Replay.** `ReplayAdmissionGuard`/`ReplayAuthority` have no `Send` bound;
  `FileReplayAuthority` stays `Rc`/`RefCell`. G0 in Stage A, deadline start →
  final deadline check → G1 (`publish_armed`) → launch in Stage B, G2 after
  durable outcome in Stage C. `ObservingReplayAuthority`/`ReplayTrace` are
  test-contained and record actual calls.
- **Trail.** Only the coordinator writes the Trail; physical `OutcomeEntry`
  order follows Stage C arrival; final non-success selection is by Runtime
  Plan member order via `first_non_success_member_step`.
- **Terminal taxonomy.** `map_shared_result` preserves `Denied`,
  `ApprovalRequired`, `Unavailable`, `Unattempted`, `Uncertain`, `Failed`,
  `Completed`, `ReplayBlockedCompletedSuccess`, `ReplayBlockedCompletedFailure`,
  `ReplayRequiresManualResolution`, `ReplayPersistenceUnavailable`, and
  `AuditFailed`. `ReplayBlockedCompletedSuccess` maps to `Boundary(Replay(...))`
  and counts as success in `step_succeeded`.

The 19 focused C2-A3a proofs were read assertion-by-assertion and found to
prove their titles (overlap, durability, physical ordering, GroupJoin, panic,
Denied/ApprovalRequired/Unavailable/ReplayBlocked/Unattempted/Uncertain,
inverse physical completion, intent-before-effect, G1/G2-before/after-effect).

### Verification results (independent run)

| Check | Result |
| --- | --- |
| focused C2-A3a tests (19) | PASS (19/19) |
| `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check` | PASS |
| `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml` | PASS |
| `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --locked` | PASS |
| `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1` | PASS (1512 lib + integration, 0 failed) |
| `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features` | PASS (exit 0; 49 lib-test + 5 lib warnings, dead-code only) |
| `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features -- --test-threads=1` | PASS (0 failed) |
| `git diff --check` | PASS |
| `.github/scripts/check-tethers-task-packet.ps1` | PASS (control-v1/COMPLETE) |

Final feature SHA before closeout: `65acab1c9dfc7ceab16cf36164a22f94cfc69b17`.

`origin/main` remained unchanged at
`1703fb4aadc06980daea8fe5afbeaf3a6218b256`. No force push; the closeout is a
normal descendant commit of the implementation checkpoint.

### PowerShell `@(...).Count` root cause

`tethers-stdio-fixture.ps1` used
`(Get-ChildItem -Filter 'entered-*').Count`, which throws
`ParentContainsErrorRecordException` under `Set-StrictMode -Version Latest`
when exactly one file matches because the scalar `FileInfo` has no `.Count`
member. This made single-member tests fail with `NoFinalResponse`. The fix is
`@(Get-ChildItem ...).Count`, which wraps the result in an array so `.Count`
is always defined for 0, 1, or many matches.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`
- current A3a base commit `58aecd0c789802cdfea57d4560b51fd21d5340ae`
