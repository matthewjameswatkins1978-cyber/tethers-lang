# J19 M5 Durable Local Anchor Worker Note

Task: `J19-M5 - Autonomous Durable Local Anchor Vertical Slice`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Status: `COMPLETE`
Owner: `Luna / OpenCode`
Branch: `opencode/j19-m5-durable-local-anchor`
Base commit: `e57bf536fe3d7fb074c00ddac867b5720a15116e`
Accepted M4 baseline: `e57bf536fe3d7fb074c00ddac867b5720a15116e`
Frozen architecture base: `a5fd63593a9d9acd397030ecd2e27b4f318c87fd`
Control commit and starting HEAD: `11dd0ff04da20fa36bdddd19d4132833830194fe`
Implementation checkpoint: `9e3e1ebe21c2540be2ad30b2db7082facabd8c8e`

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
acknowledgement callback only after durable admission, ordered Trail
publication, existing host evaluation, and append-only evaluation completion.
The application integration seam submits the root Anchor through J11 admission
and the existing `process_one_event` coordinator path.

## Decisions and assumptions

Event identity is the provider-issued `event_id`; occurred time is audit data
only. Admission records are separate from Trail, replay and operation outcomes.
Conflict records are preserved in separate files and never replace the original
admitted identity. The first local event is `file.received@1` with a bounded
payload and optional host-relative source path.

## Evidence

The final focused checkpoint passes seven `local_anchor` unit tests and one
native Windows `m5_local_anchor` integration test. The integration test proves
durable restart duplicate handling, conflict evidence without acknowledgement,
host scope binding, and generation-zero Anchor identity. The coordinator offers
Trail publication before acknowledgement; no notification is itself treated as
an Anchor.

## Discoveries

The authorized packet initially used `AUTHORISED` and omitted checker-required
canonical sections; it was transitioned to `IN_PROGRESS` and those sections
were restored without changing the frozen M5 scope.

## Remaining risks

The local source/session adapter remains a bounded host integration seam rather
than a general filesystem watcher or network listener. Installed Plug trust and
session credentials must be supplied by the M4 host adapter; this module does
not invent authority from provider claims. Duplicate JSON fields inside nested
payload values are treated as payload data and do not become host authority.

## Smallest next action

Independent Lucy review of the pushed M5 correction branch. Do not begin M6.

## References

The governing source is `docs/CURRENT_CLINE_TASK.md`; M4 installed Plug and
existing event/evaluation seams are documented in the accepted M4 worker note.

## Implementation Ledger

- Contract/admission commit: `1b9e27f` (`feat: add durable local event admission`).
- Refusal-boundary tests: `d33da54` (`test: cover local anchor refusal boundaries`).
- Windows restart/conflict integration: `5dac35c` (`test: prove durable local anchor restart flow`).
- Trail-before-ack ordering: `9e8970e` (`feat: order local anchor trail before acknowledgement`).
- Integration correction: `e127521` (`fix: route durable anchors through host evaluation`).
- Evaluation completion persistence and source-test compatibility: `3af3e61`.
- Event identity is the stable provider-issued `event_id`; cursor, timestamp and
  transport position are never used as identity.
- Durable records use `tethers.local-event-admission.v1`, canonical SHA-256
  covered bytes, host-generated `anchor/<event-id>/0` identity, create-new temp
  publication, flush/sync, atomic rename, strict reload, and separate hashed
  conflict files.
- The `file.received@1` envelope contains exact format, provider, installed Plug,
  session, event, occurred time, payload, payload digest, optional source path,
  and generation fields. The envelope rejects unknown/duplicate fields and
  generation above eight.
- `AdmissionBinding` requires exact installed Plug/provider/session/event
  identity and canonical source-root confinement before durable admission.
- `LocalAnchorCoordinator` publishes admission, then optional Trail evidence,
  then existing host evaluation, then append-only evaluation completion, then
  invokes provider acknowledgement. Duplicate delivery returns the original
  root identity without a new evaluation when terminal; conflict returns no
  success.

## Verification

- `just tools`: PASS.
- `just fmt`: PASS.
- `just check`: PASS; existing M3/J13 warnings only.
- Focused M5 unit tests: PASS; 7 tests, including terminal evaluation restart state.
- Focused M5 native Windows integration: PASS; 1 test.
- `just test-m3`: PASS; 6 trust tests and 13 lifecycle tests.
- `just test-m4`: PASS; 5 contract/provider tests and 4 integration tests.
- `just test-rust`: PASS; 812 tests plus all target test binaries.
- `just verify`: PASS; same 812 Rust tests and M3/M4/M5 integration.
- OCaml `dune build`: PASS; OCaml `dune runtest`: PASS, sequentially.
- Locked debug and release Rust builds: PASS.
- Complete `tethers-0.1/scripts/verify-0.2.ps1`: PASS; all 6 suites, including
  J13A 25, J13B 10, J13C 19, J14A 5/95, J14B 11/243, J14C 9/196.
- Packet checker and `git diff --check`: PASS. Native integration leaves no
  provider process behind.

## Final State

No credentials, network listener, watcher, recursive crawl, provider
notification-as-Anchor shortcut, cursor identity, automatic retry, M6 feature,
main movement, tag, release or force-push was added.
Final branch SHA: `9e3e1ebe21c2540be2ad30b2db7082facabd8c8e`.

Correction verification reran `just verify` successfully after the source-test
compatibility fix; one earlier parallel full-matrix run had a transient Windows
M3 permission failure, and the isolated test plus serialized rerun passed.
M6 remains absent.

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
