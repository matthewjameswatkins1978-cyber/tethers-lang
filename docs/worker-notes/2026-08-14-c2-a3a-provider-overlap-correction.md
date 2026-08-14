# Worker Note — C2-A3a — Final Provider Overlap Correction

Task: `C2-A3a — Final Provider Overlap Correction`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `MiMo Pro`

Status: `IN_PROGRESS`

Base commit: `58aecd0c789802cdfea57d4560b51fd21d5340ae`

Implementation checkpoint: `f6eeb572b2afe42472d2deab6336a1af971f649f`

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

The narrow core leaves intentionally unproved: durable Stage C persistence
under concurrency with real `FileReplayAuthority`; Trail B/A ordering with real
replay persistence; preparation, replay, unattempted/uncertain, intent/G1,
and join-after-terminal matrices; and the full closeout suite. The formal task
must remain `IN_PROGRESS` until those separately assigned proofs complete.

The `TestReplayAuthority` exercises the `ReplayAdmission` contract interface
but does not prove durable filesystem persistence. A later task may require
`FileReplayAuthority` with provisioned replay directories.

## Smallest next action

Lucy review of the five coordinator-observability proofs. If accepted, the
remaining deferred matrices (semantic, replay, unattempted, intent/G1) may be
assigned as separate tasks.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`
- current A3a base commit `58aecd0c789802cdfea57d4560b51fd21d5340ae`
