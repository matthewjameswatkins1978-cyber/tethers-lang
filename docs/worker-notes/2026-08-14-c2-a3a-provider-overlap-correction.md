# Worker Note — C2-A3a — Final Provider Overlap Correction

Task: `C2-A3a — Final Provider Overlap Correction`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `READY`

Base commit: `58aecd0c789802cdfea57d4560b51fd21d5340ae`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Activate the already-reviewed C2-A3a correction as a formal READY Red task
assigned to Codex. Codex must surgically correct the existing C2-A3a
implementation at 58aecd0c789802cdfea57d4560b51fd21d5340ae without rewriting
it from scratch, proving real provider-effect overlap and exact C1 terminal
semantics. This note records task activation only; implementation is not yet
started.

## Changes made

Implementation changes have not started yet; only task activation has been
recorded. The control packet in `docs/CURRENT_CLINE_TASK.md` was replaced with
the control-v1 READY packet, `docs/PROJECT_DASHBOARD.md` was updated to record
the active correction, and this worker note was created. No production code was
touched.

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
local HEAD `58aecd0c789802cdfea57d4560b51fd21d5340ae`, fetched
`origin/main` `1703fb4aadc06980daea8fe5afbeaf3a6218b256`. Device-tool diagnostic
(`scripts/check-dev-tools.ps1`) reported all required tools present. The 16
pre-existing untracked paths are recorded in the packet's Expected pre-existing
changes section and preserved exactly. Packet-checker result
(`.github/scripts/check-tethers-task-packet.ps1`) is recorded after activation.

## Discoveries

The previous `NO ACTIVE IMPLEMENTATION PACKET` state in `docs/CURRENT_CLINE_TASK.md`
blocked Codex correctly: the checker required a full Base commit SHA and
control-v1 packet fields that the stale file did not provide. This activation
resolves that exact defect without changing any implementation.

## Remaining risks

Active implementation risks for Codex: exact terminal-result algebra must not
flatten Denied, ApprovalRequired, Unavailable, Uncertain, Unattempted,
AuditFailed, replay classifications, or Completed/Failed; Rust ownership
transitions must use structural state movement (Option::take / whole-enum
transitions) with no fabricated semantic objects; and the provider-overlap
crucible must prove real simultaneous provider effects deterministically.

## Smallest next action

Codex executes the authorised correction packet: surgical correction of the
C2-A3a implementation at base `58aecd0c789802cdfea57d4560b51fd21d5340ae`,
then the required verification set, worker note closeout, and a normal branch
push.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`
- current A3a base commit `58aecd0c789802cdfea57d4560b51fd21d5340ae`
