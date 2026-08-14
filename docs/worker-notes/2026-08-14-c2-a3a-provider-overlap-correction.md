# Worker Note — C2-A3a — Final Provider Overlap Correction

Task: `C2-A3a — Final Provider Overlap Correction`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `MiMo Pro`

Status: `IN_PROGRESS`

Base commit: `58aecd0c789802cdfea57d4560b51fd21d5340ae`

Implementation checkpoint: `WORKTREE`

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

## Discoveries

The previous `NO ACTIVE IMPLEMENTATION PACKET` state in `docs/CURRENT_CLINE_TASK.md`
blocked Codex correctly: the checker required a full Base commit SHA and
control-v1 packet fields that the stale file did not provide. This activation
resolves that exact defect without changing any implementation.

## Remaining risks

The narrow core leaves intentionally unproved: durable Stage C persistence
under concurrency; Trail B/A ordering and sequencing; preparation, replay,
unattempted/uncertain, intent/G1, and join-after-terminal matrices; and the
full closeout suite. The formal task must remain `IN_PROGRESS` until those
separately assigned proofs complete.

## Smallest next action

Run the narrow focused verification and publish the review commit without
closing the formal C2-A3a task. Lucy's review gate must precede assignment of
the deferred Cline/DeepSeek matrix work.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`
- current A3a base commit `58aecd0c789802cdfea57d4560b51fd21d5340ae`
