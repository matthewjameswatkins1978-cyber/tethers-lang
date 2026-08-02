# J19 M5 Durable Local Anchor Worker Note

Task: `J19-M5 - Autonomous Durable Local Anchor Vertical Slice`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Status: `IN_PROGRESS`
Owner: `Luna / OpenCode`
Branch: `opencode/j19-m5-durable-local-anchor`
Base commit: `e57bf536fe3d7fb074c00ddac867b5720a15116e`
Accepted M4 baseline: `e57bf536fe3d7fb074c00ddac867b5720a15116e`
Frozen architecture base: `a5fd63593a9d9acd397030ecd2e27b4f318c87fd`
Control commit and starting HEAD: `11dd0ff04da20fa36bdddd19d4132833830194fe`
Implementation checkpoint: `WORKTREE`

Starting branch state: clean `opencode/j19-m5-durable-local-anchor` at the
control commit above. `just tools` passed before implementation.

## Requested outcome

Complete one bounded credential-free local inbound event path from validated
provider notification through durable host admission and one generation-0 root
Anchor, while preserving M3/M4 and released 0.2 behaviour.

## Changes made

Added the host-owned `local_anchor` boundary with strict `file.received@1`
envelope validation, canonical payload/event digests, exact installed/provider/
session/event binding checks, approved source-root confinement, immutable
create-only admission publication, restart reload, duplicate replay, conflict
evidence, and generation-0 root Anchor construction. A coordinator invokes its
acknowledgement callback only after durable admission succeeds.

## Decisions and assumptions

Event identity is the provider-issued `event_id`; occurred time is audit data
only. Admission records are separate from Trail, replay and operation outcomes.
Conflict records are preserved in separate files and never replace the original
admitted identity. The first local event is `file.received@1` with a bounded
payload and optional host-relative source path.

## Evidence

The initial focused checkpoint passes four Rust tests covering restart duplicate,
same-ID/different-digest conflict, generation-zero Anchor creation, and
acknowledgement ordering. Full host integration and regression evidence remains
to be added before completion.

## Discoveries

The authorized packet initially used `AUTHORISED` and omitted checker-required
canonical sections; it was transitioned to `IN_PROGRESS` and those sections
were restored without changing the frozen M5 scope.

## Remaining risks

The durable boundary is not yet wired into the normal external-event execution
route, and the required real Windows source/session/restart scenario is not yet
implemented. Duplicate JSON fields inside nested payload values remain bounded
by canonical JSON parsing but are not interpreted as host authority.

## Smallest next action

Connect `LocalAnchorCoordinator` to the host event/evaluation seam and add the
Windows restart/conflict/disablement integration fixture.

## References

The governing source is `docs/CURRENT_CLINE_TASK.md`; M4 installed Plug and
existing event/evaluation seams are documented in the accepted M4 worker note.

Use this file as the durable M5 implementation ledger. Record:

- exact control commit and starting branch state;
- inbound event contract decisions and schema digests;
- durable admission store schema and recovery rules;
- provider/source implementation commits;
- focused tests and full regression evidence;
- duplicate/conflict/generation/acknowledgement behaviour;
- remaining limitations and deferred work;
- final branch SHA.

Do not use this note to change frozen architecture or expand M5 into networking, credentials, jobs, streams, PDF support, marketplace, release work, or M6.
