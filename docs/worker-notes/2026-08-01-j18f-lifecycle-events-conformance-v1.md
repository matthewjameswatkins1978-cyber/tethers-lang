# J18F Worker Note

## Task

J18F - Lifecycle, Outcomes, Events and Conformance v1.

## Changes

Added the lifecycle, outcome, event, replay, Result Anchor, and conformance
candidate contract. Accepted J18E, added the J18F decision entry, and aligned
current-state documents with J18F active and J18G next after Lucy acceptance.
No implementation is authorised.

## Decisions and assumptions

State families remain separate. Unattempted is not an execution outcome; the
only attempted outcomes are succeeded, failed, and uncertain. Durable outcome
precedes Result Anchor. Replay is separate from Trail and never authorises
retry. Plug Anchors require stable identity and durable admission distinct from
J11. Conformance is host-orchestrated evidence, not permission.

## Existing outcome implementation inspected

Inspected `docs/J06_DEADLINE_OUTCOME_DESIGN.md` and `outcome.rs`: monotonic
deadline, invocation boundary, redaction, three attempted outcomes, and no
Result Anchor for unattempted work.

## Existing replay implementation inspected

Inspected `docs/J09_DURABLE_REPLAY_DESIGN.md` and `replay_runtime.rs`: separate
host replay authority, terminal blocking, manual resolution, durable ordering,
and no retry after restart.

## Existing event-admission implementation inspected

Inspected J10/J11 notes plus `event_queue.rs` and `event_admission.rs`: serial
FIFO follow-up delivery, admission before evaluation, duplicate rejection, and
causal generations 0 through 8.

## Existing provider lifecycle inspected

Inspected `stdio_provider.rs` and `host_execution.rs`: retained host-owned
sessions, initialize/version/provider checks, discovery, serial calls, typed
transport errors, and bounded close.

## Tool bootstrap

- `rg` 15.2.0
- `fd` 10.4.2
- `jq` 1.8.2
- `gh` 2.97.0
- `yq` 4.53.3

Resolved from existing WinGet installations process-locally. No installation,
upgrade, replacement, or permanent configuration occurred.

## Evidence

Base: `eb6548ca61a2c8b108e675f01f3512f0598bc6b6`. Released tag peels to
`b5546411661dcbcb53e1cf2538eaec594c6f76f2`. No Rust, OCaml, schema, manifest,
runtime, test, fixture, provider, package, or protocol file changed.

## Discoveries

Retained provider code supplies implementation evidence for session phases
without constituting a lifecycle state machine. J09 and J11 must remain distinct
from future durable external-event admission.

The staged whitespace check found two intentional-looking Markdown hard breaks
in the J18F status block. Lucy authorised a follow-up correction commit rather
than history rewriting. The two trailing-space sequences were removed. No
normative wording or behaviour changed, and no implementation or schema changed.

## Remaining risks

J18G must define credential secrecy and sandbox enforcement. J18H must paper
validate representative integrations and resolve lifecycle ambiguity before
freeze. Anchor subscription, cursor, and external-admission mechanics remain
future implementation planning.

## Next action

Lucy reviews J18F. Do not begin J18G or implementation before acceptance.

## References

- `docs/architecture/TETHERS_LIFECYCLE_OUTCOMES_EVENTS_CONFORMANCE_V1.md`
- `docs/J06_DEADLINE_OUTCOME_DESIGN.md`
- `docs/J09_DURABLE_REPLAY_DESIGN.md`
- `docs/worker-notes/2026-07-27-j10-result-event-queue.md`
- `docs/worker-notes/2026-07-28-j11-event-admission-foundation.md`
- `docs/worker-notes/2026-07-28-j11-event-trail-final.md`
- `tethers-0.1/host-rust/src/outcome.rs`
- `tethers-0.1/host-rust/src/replay_runtime.rs`
- `tethers-0.1/host-rust/src/result_anchor.rs`
- `tethers-0.1/host-rust/src/event_admission.rs`
- `tethers-0.1/host-rust/src/event_queue.rs`
- `tethers-0.1/host-rust/src/stdio_provider.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
