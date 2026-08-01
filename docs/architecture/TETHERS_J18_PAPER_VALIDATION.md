Status: J18H candidate, pending Lucy paper-validation review
Validation generation: 1
Implementation: Not authorised

# Tethers J18 Universal Plug Paper Validation

This is a paper validation of the accepted J18B through J18G contracts. It is
not an implementation plan, provider approval, safety certification, or claim
that deferred classes are available. Source facts, accepted Tethers rules, and
architectural inferences are labelled in the worker note.

## Disposition Table

| integration | technical possibility | primary class | architectural fit | security mode | first-slice status | verdict |
|---|---|---|---|---|---|---|
| local file tool | yes | Action/Query | first-slice candidate | supervised, no credential | first-slice reference | support bounded |
| PDF processor | conditional | Query | future Query | supervised only for bounded reference use; isolated for production | competition/reference candidate | conditional |
| GitHub service | yes | Query/Action/Anchor | future supported integration | isolated or broker | deferred | support via provider |
| email service | conditional | Query/Action/Anchor | future supported integration | isolated or broker | deferred | support with honest delivery |
| SQL database | yes | Query/Action | future supported integration | isolated or broker | deferred | support with transaction uncertainty |
| cloud drive | yes | Query/Action/Anchor | future supported integration | isolated or broker | deferred | support via change feed |
| remote AI model | yes | Query; reserved Job/Stream | future Query, reserved asynchronous classes | isolated or broker | deferred | support bounded Query |
| local AI model | conditional | Query; reserved Stream | future Query, reserved Stream | isolated; no cloud credential assumed | deferred | support bounded local Query |
| webhook source | conditional | Anchor | gateway mediated | isolated listener gateway | not first-slice | defer |
| long-running video renderer | yes | reserved Job | reserved Job | supervised only for trusted bounded development; isolated production | reserved | defer |
| live sensor stream | conditional | Query snapshot; reserved Stream | reserved Stream, gateway reduction | isolated/gateway | reserved | refuse raw stream |
| printer | conditional | Action/Query | future supported integration | isolated or broker | deferred | physical completion uncertain |
| MIDI instrument | conditional | Action/Query; reserved Stream input | future device integration | isolated device gateway | deferred | bounded device effects only |
| smart lock | conditional | Action/Query | brokered security-sensitive Action | isolated or reviewed gateway | refused first slice | refuse without state proof |
| industrial machine | conditional | Query/Anchor; Action only via gateway | gateway mediated, non-safety | reviewed gateway | refused safety actuation | refuse safety control |
| human approval queue | conditional | reserved Human Task | reserved Human Task | host-owned external queue | reserved | current Ask is insufficient |

## Uniform Validation Fields

Every section answers fields 1-46 in order: possibility and fit; provider/package/
capability identity; class; effects; scopes and resolution; policy; Socket,
binding, transport and translation; discovery/drift; authentication,
credentials, isolation and resource boundaries; attempt and outcomes; partial
work, cancellation, restart, replay and event rules; required class semantics;
Trail/conformance/invalidation/removal; implementation/refusal; revision and
evidence basis. The final range 39-46 is explicitly: 39. Trail evidence; 40.
conformance strategy; 41. conformance invalidation; 42. installation and
removal; 43. first implementation status; 44. refusal boundary; 45.
architecture revision required; 46. evidence basis.

## local file tool

1-4. Technical possibility **yes**; first-slice candidate; local provider; package
`tethers.file-tools`, provider `file-tools`, capabilities
`file.read@1`, `file.move@1`, and `file.metadata@1` are distinct identities.
5-10. `file.read` and metadata are Query; move is Action because it changes
filesystem state. Effects are `data.read`, `metadata.read`, and `data.move`.
Scopes are exact source/destination paths, approved prefixes, overwrite=false,
size and result limits; all resolve before dispatch. Host policy is allow/ask
for move and bounded read, with approval for overwrite-sensitive effects.
11-16. Socket `establish`, `discover`, `invoke`, `observe_result`, `probe`, and
`close`; MCP 2025-11-25; local stdio JSON-RPC; provider owns filesystem
translation. Discovery is complete paginated tools/list; path-capability drift
stales the binding and additions are unavailable.
17-24. Host account/filesystem permissions authenticate; no credential profile;
supervised reference mode; package read-only, per-run scratch, no network, and
finite path/byte/time/message limits. Reparse, junction, overwrite and path
normalisation escape are primary risks.
25-34. Attempt begins only after durable intent and one tools/call. Succeeded
means trusted schema proves the requested bounded read/move; failed means
trusted refusal, invalid output, or proven no-effect cancellation; uncertain
means loss after invocation or ambiguous filesystem state. Unattempted covers
policy, scope, stale binding and pre-boundary failure. Partial reads are success
only when the contract permits bounded partial data; partial moves are failed or
uncertain. Cancellation is bounded and cannot prove physical/filesystem truth;
restart creates a new session and never retries. Replay uses host execution
identity; no automatic retry. No external event/job identity is required.
35-38. No event or job; a later file-change Anchor would need stable source
identity and durable admission. Current Action/Query is truthful.
39-46. Trail records identity, resolved paths as permitted, digest/binding,
policy, approval, attempt, outcome, safe error and replay state. Conformance
tests traversal, overwrite, malformed output, drift, duplicate/restart and
redaction; any payload/binding/scope change invalidates it. Removal stops calls,
removes binding, preserves Trail. First-slice reference. Refuse ambiguous paths,
reparse escapes, unknown effects or unprovable result. Architecture revision: no.
Evidence: accepted J18B-J18G; file/PDF first-envelope inference.

## PDF processor

1-4. **Conditional**; future Query; local provider; package `tethers.pdf-tools`,
provider `pdf-tools`, capability `document.extract_text@1`.
5-10. Query because it observes hostile input without deliberate business
mutation. Effects `data.read`, `compute.consume`, `storage.consume`; scopes are
exact input, bytes, pages, output size, parser features and scratch. Materialise
input before dispatch and resolve limits first; policy is bounded allow/ask.
11-16. Standard Socket operations over MCP stdio; provider owns PDF parsing and
FFI/vendor translation. Discovery is pinned and drift disables the binding.
17-24. No credential for reference use; supervised mode is acceptable only for
bounded development/competition input. Production parsing requires proven
isolated mode. Input and scratch are separate, no network, finite CPU/memory/time
and output limits; malformed PDF/parser escape is the primary risk.
25-34. Attempt starts at parser invocation. Success means schema-valid extraction
within limits, not document safety; failure means proven parse rejection or no
result; uncertainty means process loss or unknown output extent. Unattempted is
rejected materialisation, size/page policy or stale binding. Partial extraction
needs an explicit contract or is failed/uncertain. Cancellation and restart do
not retry. No external identity is required.
35-38. No Job/Stream/Anchor; raw parser progress cannot be admitted as a Stream.
Current Query remains truthful only for bounded extraction.
39-46. Trail records input digest, limits, parser/provider pins, outcome and safe
diagnostics. Conformance uses hostile malformed files, bombs, links, limits and
redaction; any parser or isolation drift invalidates. Removal preserves evidence.
First slice only as bounded reference/competition; production refusal without
isolation. Architecture revision: no. Evidence: accepted contracts, external
FFmpeg/PDF analogy not used for parser guarantees; unresolved parser-specific
behaviour remains provider-owned.

## GitHub service

1-4. **Yes**; future Query/Action/Anchor; remote-facing local provider; package
`tethers.github`, provider `github-rest`, capabilities `github.issue.search@1`,
`github.issue.create@1`, `github.webhook.receive@1`.
5-10. Search is Query, create is Action, webhook is Anchor; effects are read,
create and network communication. Scopes identify account, repository, issue
fields, labels, rate/cost and event types; targets resolve before dispatch or
admission. Host policy and approval govern mutations.
11-16. Socket/MCP stdio/local transport to provider HTTPS; provider owns GitHub
REST/webhook translation. Discovery is manifest plus live endpoint observation;
rate/API/schema drift stales bindings and never changes permission.
17-24. GitHub token/app authentication; host credential profile; isolated
credential-bearing provider or broker; no package secret, restricted filesystem,
HTTPS allowlist, rate/page/byte/time/cost limits. Token leakage, broad repository
scope and rate exhaustion are primary risks.
25-34. Attempt crosses provider request. Query success means validated response;
Action success means GitHub response plus contract evidence, not human outcome;
failure is trusted rejection; uncertain is timeout/connection loss or ambiguous
mutation. Unattempted covers policy/rate preflight and stale binding. Partial
pagination is not success unless contract says so. Cancellation cannot undo a
remote mutation; restart never retries. REST request IDs are not universal event
identity; webhook delivery uses provider stable identity and durable admission.
35-38. Webhook needs Anchor identity, cursor/order/ack rules from source; it is
not first-slice listener implementation. Query/Action are not Job or Stream.
39-46. Trail records endpoint, repository scope, request/response validation,
rate evidence, outcome and event admission. Conformance tests pagination, drift,
auth, redaction and duplicate webhook delivery; token/scope/API drift invalidates.
Removal disables bindings and preserves events. Deferred, provider-mediated;
refuse unknown repository mapping or unstable event identity. Architecture
revision: no. Evidence: accepted contracts and GitHub primary sources.

## email service

1-4. **Conditional**; future Query/Action/Anchor; remote-facing local provider;
package `tethers.email`, provider `smtp-imap`, capabilities
`mail.message.search@1`, `mail.message.send@1`, `mail.inbound.receive@1`.
5-10. Search/read is Query, send is Action, inbound is Anchor. Effects read and
communicate; scopes cover account, folders, sender, recipients, attachment size,
rate and origin. Policy/approval is mandatory for send; targets resolve first.
11-16. Socket/MCP stdio/local transport; provider owns SMTP/IMAP translation.
Discovery is configured and pinned; mailbox/server capability drift disables or
revalidates, never silently broadens.
17-24. Host account auth and credential profile; isolated provider or broker; no
secret in package/message, bounded mailbox/network allowlist, size/rate/cost
limits. Credential leakage and recipient confusion are primary risks.
25-34. Send attempt begins at SMTP submission. Success means only the contract's
accepted SMTP/server evidence; it never means mailbox placement, recipient
delivery or human reading. Failure is trusted rejection; uncertain is loss after
submission or mailbox ambiguity. Unattempted covers policy/connection preflight.
Partial delivery is not inferred. Cancellation cannot retract accepted mail;
restart never retries. Inbound authentication, validation and durable admission
produce an admitted Anchor, not a canonical succeeded operation outcome. Duplicate
delivery is duplicate admission and is not evaluated again; invalid,
unauthenticated, out-of-scope or unstable-identity input is rejected before
evaluation and does not create a canonical failed operation outcome. Same-ID/
different-payload is an identity conflict; unavailable or corrupt durable
admission authority fails closed. Admission uncertainty remains separate from
attempted-operation uncertain. Inbound Anchor requires stable source identity;
folders, UIDs and cursors are source-specific and not interchangeable.
35-38. Anchor needs durable admission and explicit acknowledgement/cursor rules;
current Query/Action cannot represent durable mail workflow. No Job/Stream needed
for bounded operations.
39-46. Trail records message digest, bounded recipients, server evidence and
outcome without content secrets. Conformance uses test mailbox, SMTP acceptance,
IMAP read, duplicate events and loss; credentials/server policy invalidate.
Removal preserves Trail. Deferred; refuse claims of delivery/read or unstable
message identity. Architecture revision: no. Evidence: accepted contracts and
RFC 5321/RFC 9051.

## SQL database

1-4. **Yes**; future Query/Action; remote-facing local provider; package
`tethers.sql`, provider `postgresql`, capabilities `sql.query@1` and
`sql.transaction@1`.
5-10. Read-only statement is Query; mutation/transaction is Action. Effects
`data.read`, `data.create/update/delete`, `compute.consume`; scopes identify
database/server/schema/table/columns, statement class, rows, bytes, time and
transaction. No arbitrary SQL; policy and approval are host-owned.
11-16. Socket/MCP stdio/local transport; provider owns PostgreSQL protocol and
SQL translation. Discovery is reviewed schema/catalogue; schema/privilege drift
stales bindings. Catalogue is not permission.
17-24. Database authentication through host credential profile; isolated or
brokered provider; no broad filesystem, restricted network destinations, row,
timeout, transaction and cost limits. SQL injection, privilege escalation and
data exfiltration are primary risks.
25-34. Attempt begins when statement/transaction may execute. Success requires
trusted result or commit evidence; failure requires final rejection/no commit;
connection loss around commit is uncertain. Unattempted includes policy,
scope, read-only and preflight refusal. Partial transaction is failed only with
rollback/no-effect proof, otherwise uncertain. Cancellation has no invented
rollback guarantee; restart never retries. Query cursors are not event IDs;
database change Anchors need an explicit stable CDC contract.
35-38. No Job/Stream required for bounded calls; continuous change feed would be
Stream/Anchor and is deferred. Query/Action remains truthful when statement
class is reviewed.
39-46. Trail records database identity, statement digest, scope, transaction
boundary, row/byte limits, outcome and redacted error. Conformance uses fixture
DB, read-only enforcement, commit-loss, rollback, injection and drift; schema,
privilege or provider changes invalidate. Removal preserves evidence. Deferred;
refuse arbitrary SQL and uncertain commit claims. Architecture revision: no.
Evidence: accepted contracts and PostgreSQL protocol overview/message-flow
primary sources.

## cloud drive

1-4. **Yes**; future Query/Action/Anchor; remote-facing local provider; package
`tethers.cloud-drive`, provider `google-drive`, capabilities
`drive.file.get@1`, `drive.file.update@1`, `drive.changes.receive@1`.
5-10. File retrieval is Query, mutation is Action, changes are Anchor. Effects
read/update/communicate; scopes identify account, drive, file IDs, fields,
parents, bytes, rate and origin. Exact targets resolve before calls/admission;
OAuth and approval remain host-owned/deferred.
11-16. Socket/MCP stdio/local transport; provider owns Drive REST translation.
Discovery is manifest plus API observation; permission/schema/change-token drift
stales binding. Push notification is only a change hint.
17-24. OAuth host profile (deferred), isolated provider or broker, no package
secret, bounded local scratch and HTTPS allowlist, page/byte/rate/cost limits.
Token leakage and broad shared-drive scope are primary risks.
25-34. Query or Action outcomes remain exactly `succeeded`, `failed`, and
`uncertain` for attempted provider operations. Inbound change admission is a
separate state family: authentication, validation and durable admission produce
an admitted Anchor, not a canonical succeeded operation outcome. Duplicate
redelivery is duplicate admission and is not evaluated again. Invalid,
unauthenticated, out-of-scope or unstable-identity input is rejected before
evaluation and does not create a canonical failed operation outcome. Same-ID/
different-payload is an identity conflict that quarantines or disables the
source binding. Unavailable or corrupt durable admission authority fails closed;
admission uncertainty is not attempted-operation uncertain. Success means
validated file response or mutation evidence, not downstream sync; failure is
final rejection; uncertain is loss around mutation. Unattempted covers
policy/token/scope failure. Partial upload is contract-specific, otherwise
failed/uncertain. Cancellation and restart do not retry. Change feed IDs/token
positions must be kept distinct; acknowledgement follows durable admission.
35-38. Change notification needs Anchor plus authoritative change feed; cursor
is not event identity. No Job/Stream is required for bounded file calls.
39-46. Trail records file ID, change token/event identity, scope, API evidence,
outcome and admission. Conformance tests hints versus feed, duplicates, token
rewind, drift and redaction; OAuth/permission/API changes invalidate. Removal
preserves history. Deferred; refuse treating push as payload or stable event
without source identity. Architecture revision: no. Evidence: accepted contracts
and Google Drive primary sources.

## remote AI model

1-4. **Yes**; future Query, reserved Job/Stream; remote-facing local provider;
package `tethers.remote-ai`, provider `openai`, capabilities
`model.infer@1`, `model.infer_job@1`, `model.infer_stream@1`.
5-10. Bounded synchronous inference is Query; background inference is reserved
Job; token/event streaming is reserved Stream. Effects `model.infer`,
`compute.consume`, `network.communicate`, `privacy.transfer`; scopes model,
data classification, tokens, cost, rate and output. Policy must explicitly
approve remote data transfer and probabilistic output.
11-16. Socket/MCP stdio/local provider transport to HTTPS; provider owns API
translation. Discovery pins model/capability; model/API drift invalidates.
17-24. Host API credential; isolated or brokered provider; no secret in package,
network allowlist, bounded tokens/time/output/cost and scratch. Data leakage,
prompt injection and cost exhaustion are primary risks.
25-34. Query attempt starts at remote request. Success means bounded validated
response, not truth of generated claims; failure is provider/schema refusal;
uncertain is transport loss or unknown billing/effect. Unattempted is policy,
privacy or budget refusal. Partial output is not success unless schema says so.
Cancellation cannot prove remote termination; restart never retries. Webhook/job
IDs are source-specific and require explicit stable identity.
35-38. Background and streaming shapes need reserved Job/Stream; forcing them
into Query is false. Synchronous Query is truthful.
39-46. Trail records model/provider pins, data classification, cost/token limits,
probabilistic flag, output schema and outcome, never prompt secrets. Conformance
uses non-production model/account, limits, drift and redaction; model/policy/
credential changes invalidate. Removal preserves evidence. Deferred; refuse
hidden AI judgement, unbounded remote transfer or Job/Stream pretence.
Architecture revision: no. Evidence: accepted contracts and successfully
inspected OpenAI Responses streaming events and Webhook events pages; the
unavailable Developer quickstart is not evidence. Exact provider guarantees
remain unresolved where those pages do not establish them.

## local AI model

1-4. **Conditional**; future Query, reserved Stream; local provider; package
`tethers.local-ai`, provider `ollama`, capabilities `model.local_infer@1` and
`model.local_stream@1`.
5-10. Bounded response is Query; token stream is reserved Stream. Effects
`model.infer`, `compute.consume`, `storage.consume`; scopes model identity,
input/output bytes, context/tokens, CPU/memory/time and privacy. Targets resolve
before invocation; local does not remove policy.
11-16. Socket/MCP stdio/local transport to local HTTP API; provider owns Ollama
translation. Discovery/model availability drift stales binding; model loading
is not permission.
17-24. No cloud authentication assumed; any local model credential remains
host-owned. Isolated provider, scratch-only filesystem, no network by default,
finite compute/resource/cost limits. Local model compromise and exhaustion are
primary risks.
25-34. Attempt at local API/model execution. Success is schema-valid bounded
output; failure is known process/API refusal; uncertain is process loss or
unknown model state. Unattempted includes unavailable model, policy and limits.
Partial output is contract-specific. Cancellation/restart do not retry. No
external event identity for Query; stream identity would require Stream.
35-38. Streaming remains reserved Stream and cannot be ordinary Query. Loading
or background inference is not silently a Job.
39-46. Trail records model digest/name, resource limits, output validation and
outcome. Conformance tests unavailable model, output bounds, exhaustion, drift
and redaction; model/payload changes invalidate. Removal preserves history.
Deferred; refuse unbounded local process or stream claims. Architecture revision:
no. Evidence: accepted contracts and Ollama primary sources; determinism/cost
remain unresolved provider facts.

## webhook source

1-4. **Conditional**; Anchor; gateway mediated; package `tethers.webhook-gateway`,
provider/gateway identity and capability `webhook.event.receive@1` are distinct.
5-10. Anchor only: effects `network.receive`, `data.read`, `event.admit`; scopes
source, authenticated origin, event types, schema, volume, replay range and
retention. Admission target must resolve stable source/event identity; policy
allows only authenticated configured sources.
11-16. Socket event semantics over a reviewed future listener/gateway binding;
transport is network gateway to local host, not current stdio first slice;
gateway owns HTTP/vendor translation. Discovery and secret/schema drift fail
closed.
17-24. Signature/mTLS or equivalent source authentication; host credential
profile; isolated gateway, no package secret, restricted listener/network and
rate/size limits. Spoofing, replay and admission overload are primary risks.
25-34. Provider-operation outcomes remain exactly `succeeded`, `failed`, and
`uncertain`, and apply only to an attempted host-to-provider operation. Inbound
event admission is a separate state family: authentication, validation and
durable admission produce an admitted Anchor, not a canonical succeeded
operation outcome. Duplicate redelivery is duplicate admission and is not
evaluated again. Invalid, unauthenticated, out-of-scope or unstable-identity
input is rejected before evaluation and does not create a canonical failed
operation outcome. Same-ID/different-payload is an identity conflict and
quarantines or disables the source binding. Unavailable or corrupt durable
admission authority fails closed before evaluation. Uncertainty about
acknowledgement or durable admission is admission uncertainty or indeterminate
admission evidence, not attempted-operation uncertain. Partial payload is
invalid unless schema permits. Cancellation is source-specific; restart never
retries or fabricates event identity. Acknowledgement follows durable admission;
cursor/order are source contracts, not identity.
35-38. Stable source event identity is mandatory; Anchor is correct. Current
runtime has no listener implementation, and notification cannot be assumed
Anchor without gateway admission.
39-46. Trail records source, event ID, payload digest, authentication, admission,
ack and evaluation linkage. Conformance tests spoof, duplicate/conflict, order,
cursor and ack failure; source contract or auth drift invalidates. Removal stops
admission and preserves evidence. Deferred; refuse unstable identity or ack before
durable admission. Architecture revision: no. Evidence: accepted J18F and
GitHub webhook primary sources as representative delivery evidence.

## long-running video renderer

1-4. **Yes**; reserved Job; local provider; package `tethers.video-renderer`,
provider `ffmpeg`, capability `video.render@1`.
5-10. Job because work completes later. Effects `process.execute`,
`storage.consume`, `compute.consume`; scopes input/output paths, codecs, duration,
bytes, CPU/time and scratch. Inputs/outputs resolve before start; policy allows
only bounded render and approval for overwrite.
11-16. Socket/MCP stdio/local transport; provider owns FFmpeg translation.
Discovery pins executable/options; binary/codec drift invalidates.
17-24. No credential for local reference; supervised trusted development or
isolated production; read-only inputs, bounded output/scratch, no network,
process/time/resource limits. Command injection, codec/parser and disk exhaustion
are primary risks.
25-34. Attempt is process start. Start success is not render success; final Job
success requires trusted output/exit evidence, failure requires known no valid
output, uncertainty covers process loss, timeout, cancellation or ambiguous file
completion. Unattempted covers policy/start refusal. Partial output is not final
success. Cancellation is best effort and evidentiary limits remain; restart
creates new Job identity and never retries.
35-38. Stable host Job ID, output identity and progress correlation are required;
progress is not automatically Stream. Current Action cannot truthfully represent
completion, so Job remains reserved.
39-46. Trail records Job identity, command digest, inputs/outputs, progress as
diagnostic, final evidence and outcome. Conformance uses bounded media and
malformed inputs; executable/codec/options/isolation changes invalidate. Removal
stops jobs and preserves evidence. Reserved/deferred; refuse claiming start means
completed render. Architecture revision: no. Evidence: accepted contracts and
architectural inference from the long-running-render scenario; the supplied
FFmpeg documentation retrieval failed with a transport error. FFmpeg-specific
progress, cancellation and process behaviour remain unresolved.

## live sensor stream

1-4. **Conditional**; bounded snapshot Query, continuous data reserved Stream,
threshold event future Anchor; gateway/provider identity `sensor.gateway` and
capability `sensor.snapshot@1`/`sensor.stream@1` remain distinct.
5-10. Snapshot Query reads state; Stream carries continuing observations; Anchor
requires an accepted deterministic reduction. Effects `state.observe`,
`network.receive`, `device.read`; scopes device, measures, rate, time, volume and
quality. Snapshot resolves; unbounded stream does not.
11-16. Reviewed gateway Socket binding; transport is device/gateway network to
host, not current stdio; gateway owns protocol translation. Device catalogue,
calibration and schema drift fail closed.
17-24. Device authentication and host credential profile; isolated gateway;
network/device/filesystem deny by default, bounded rate/storage/CPU. Spoofing,
unbounded ingestion and unsafe control coupling are primary risks.
25-34. Snapshot operation outcomes remain exactly succeeded, failed and uncertain
for an attempted provider read. Continuous stream admission is a separate state
family requiring Stream semantics. Authenticated, validated and durably admitted
reduced sensor input produces an admitted Anchor, not a canonical succeeded
operation outcome. Duplicate input is duplicate admission and is not evaluated
again; invalid, unauthenticated, out-of-scope or unstable-identity input is
rejected before evaluation and does not create a canonical failed operation
outcome. Same-ID/different-payload is an identity conflict; unavailable or
corrupt durable admission authority fails closed. Admission uncertainty is
separate from attempted-read uncertain. Unattempted is policy/unavailable.
Partial sample is not silently success. Cancellation and restart cannot imply
continuity or retry. Stable sequence/event identity, cursor, ordering and ack
are source-specific; acknowledgement follows durable admission.
35-38. Raw continuous sensor data is reserved Stream; threshold Anchor needs a
reviewed reduction. Ordinary Query cannot represent it. Hard-real-time is outside
Tethers.
39-46. Trail records snapshot identity or admitted reduced event, not unbounded
raw stream. Conformance tests identity, gaps, rate, drift and reduction; device/
gateway calibration changes invalidate. Removal stops admission and preserves
history. Refuse raw stream, hard-real-time or unproven threshold semantics.
Architecture revision: no. Evidence: accepted contracts; source-specific facts
remain unresolved.

## printer

1-4. **Conditional**; future Action/Query; local or gateway provider; package
`tethers.printer`, provider identity and `printer.submit@1`, `printer.status@1`.
5-10. Submit is Action, status Query. Effects `data.read`, `device.control`,
`physical.actuate`, `compute.consume`; scopes printer identity, document bytes,
pages, media, copies, queue and rate. Target resolves before dispatch; approval
for physical output.
11-16. Socket/MCP stdio/local transport; provider owns Print Spooler translation.
Discovery pins printer/driver; queue/device drift stales binding.
17-24. Host printer credentials if needed; isolated or broker; input scratch and
no unrelated files, local spool/network bounds, page/byte/time limits. Job
spoofing, data leakage and physical ambiguity are primary risks.
25-34. Attempt begins at spool submission. Success means spooler accepted a job,
not paper emerged; failure means known rejection; uncertain covers loss around
submission or physical/device state. Unattempted is policy/no target. Partial
pages and cancellation require device evidence; otherwise uncertain. Restart
does not retry. Spool job ID is not physical completion identity.
35-38. Job-shaped spool work is not currently a Tethers Job; status Query can be
bounded, physical completion may need Anchor/device evidence. Current Action must
not claim completion.
39-46. Trail records printer/job identity, submitted digest, spooler evidence,
status observations and outcome. Conformance tests duplicate submit, cancellation,
connection loss, queue drift and redaction; driver/device changes invalidate.
Removal preserves history. Deferred; refuse paper-completion claim without
trustworthy evidence. Architecture revision: no. Evidence: accepted contracts
and Microsoft Print Spooler API/StartDocPrinter primary sources.

## MIDI instrument

1-4. **Conditional**; future Action/Query and reserved Stream input; local device
provider/gateway; package `tethers.midi`, provider identity and capabilities
`midi.note@1`, `midi.device.query@1`, `midi.input.stream@1`.
5-10. Bounded note/control is Action with physical/device effect; enumeration and
state are Query; continuous input Stream. Scopes exact device/port, channel,
message, duration/rate and volume; policy approves physical output.
11-16. Socket/MCP stdio/local transport; provider owns Windows/MIDI translation.
Discovery pins device identity and protocol; connection/device drift disables.
17-24. Host-owned device credential if any; isolated device provider/gateway,
filesystem/network deny, bounded message rate/time/compute. Wrong-device routing
and timing/feedback risk are primary.
25-34. Attempt is message dispatch; success means provider/device accepted the
message, not audible result; failure known rejection; uncertain connection loss
or device state. Unattempted is policy/drift refusal. Partial sequence is not
success without contract. Cancellation is best effort; restart never retries.
MIDI input identity/order/cursor are source-specific.
35-38. Continuous input is reserved Stream; no sample-accurate or hard-real-time
promise. Bounded Action/Query is truthful.
39-46. Trail records device/port/message digest, limits and provider evidence.
Conformance tests exact device, drift, rate, duplicate and redaction; device/API
changes invalidate. Removal stops provider and preserves history. Deferred;
refuse hard-real-time or physical success claims. Architecture revision: no.
Evidence: accepted contracts and MIDI UMP/MIDI 2.0, About MIDI, and
Windows.Devices.Midi primary sources.

## smart lock

1-4. **Conditional**; brokered security-sensitive Action/Query; reviewed gateway
or isolated provider; package `tethers.smart-lock`, provider/device identities,
capabilities `lock.set@1`, `lock.state@1`.
5-10. Lock/unlock is physical/security Action; state is Query. Effects
`device.control`, `physical.actuate`, `identity.manage`; scopes exact lock,
operation, time window, account and approval. Target and trustworthy state
evidence must resolve before dispatch; explicit approval is required.
11-16. Socket/MCP stdio to reviewed gateway; gateway owns vendor protocol and
device safety translation. Discovery and lock-state drift fail closed.
17-24. Host credential profile through isolated provider or broker; no broad
filesystem/network, exact device allowlist and rate/time limits. Credential theft,
wrong-device actuation, replay and physical harm are primary risks.
25-34. Attempt begins at gateway command. Success requires contract-defined
trustworthy final state evidence; failed requires rejection/no effect; uncertain
is loss, contradictory state or timeout. Unattempted is denied/approval/stale
device. Partial state is uncertain. Cancellation cannot undo unlock; restart
never retries. Device event identity must be stable and durably admitted.
35-38. Query/Action are possible only through reviewed gateway; no general Job or
Stream is implied. Current Action without final state proof would be false.
39-46. Trail records exact lock identity, approval, time window, gateway evidence,
state observation and outcome. Conformance is test-device only and invalidates
on firmware, gateway, credential or state semantics drift. Removal disables
control and preserves evidence. First slice refused; refuse absent isolation,
approval, exact target or final-state proof. Architecture revision: no. Evidence:
accepted contracts; universal vendor API and safety facts unresolved.

## industrial machine

1-4. **Conditional**; gateway mediated Query/Anchor and non-safety Action only;
reviewed OPC UA gateway; package `tethers.industrial-gateway`, provider/plant
identities, capabilities `machine.observe@1`, `machine.alarm@1`, and a narrowly
bounded non-safety `machine.command@1`.
5-10. Monitoring is Query, alarms Anchor, non-safety bounded command Action.
Effects include state read, network receive and physical actuation; scopes plant,
cell, asset, command set, time, rate and safety classification. Targets must
resolve through gateway; plant policy and approval remain authoritative.
11-16. Socket/MCP stdio/local gateway transport; gateway owns OPC UA/vendor
translation. Discovery, namespace, certificate and plant configuration drift
invalidates bindings.
17-24. Host/gateway authentication and reviewed credential broker; isolated
gateway, explicit network/device boundaries, finite rate/time/resource limits.
Credential, command injection and unsafe bypass are primary risks.
25-34. Query outcomes remain exactly succeeded, failed and uncertain for an
attempted provider read. Action success means only the gateway accepted the
bounded non-safety command, never safe machine operation; failure is known
rejection and uncertain is loss around actuation or state. Inbound authenticated,
validated and durably admitted alarms produce an admitted Anchor, not a canonical
succeeded operation outcome. Duplicate alarm delivery is duplicate admission and
is not evaluated again; invalid, unauthenticated, out-of-scope or unstable-
identity input is rejected before evaluation and does not create a canonical
failed operation outcome. Same-ID/different-payload is an identity conflict;
unavailable or corrupt durable admission authority fails closed. Admission
uncertainty remains separate from operation uncertain. Safety-critical/
unattempted refusal stays unattempted. Partial operation is uncertain.
Cancellation/restart never retry or fabricate identity. Alarm identity, ordering,
cursor and ack must be plant-defined and durable; acknowledgement follows durable
admission.
35-38. Anchor is possible for authenticated alarms; safety control is outside
Tethers. It must not become a safety PLC, certified controller or hard-real-time
loop.
39-46. Trail records plant/asset/command identity, approval, gateway and state
evidence, outcome and admission. Conformance uses simulation/test plant and
cannot prove safety certification; plant/gateway/namespace changes invalidate.
Removal preserves evidence and disables gateway bindings. Refuse direct or
safety-critical actuation, missing plant authority or unstable identity.
Architecture revision: no. Evidence: accepted contracts and OPC UA Part 1 and
security overview primary sources.

## human approval queue

1-4. **Conditional**; reserved Human Task; host-owned external queue or reviewed
gateway; package `tethers.human-queue`, queue/task identities and capability
`human.approval.request@1`.
5-10. A durable assignment, reassignment, deadline and completion is Human Task,
not current Action/Query. Effects `human.judgement`, `notification.emit`,
`data.create`; scopes queue, assignee, deadline, task data classification and
approval authority. Target resolves to a durable task record; host policy governs.
11-16. A future Socket/binding over local or remote queue transport; queue owns
vendor translation. Discovery and task-schema/assignee drift fail closed.
17-24. Host identity and queue credentials; isolated provider/broker; no secret
in package, explicit network and data boundaries, deadline/size/rate limits.
Spoofed approval, stale assignment and privacy leakage are primary risks.
25-34. Attempt is durable task creation, not human completion. Success means task
accepted; failure means rejected; uncertain is loss around creation. Unattempted
is policy/privacy/deadline refusal. Partial completion is not inferred.
Cancellation/reassignment and restart require queue evidence; no retry. Stable
task identity, ordering, acknowledgement and completion evidence are required.
35-38. Human Task remains reserved. Immediate host `Ask` is one-shot approval,
not a general queue, and expressing durable work as current Action would be false.
39-46. Trail records task identity, proof binding, queue evidence, assignment and
completion outcome without sensitive human data. Conformance uses test queue,
reassignment, expiry, duplicate and redaction; queue/policy changes invalidate.
Removal stops new tasks and preserves history. Deferred/reserved; refuse claims
that current Ask implements workflow. Architecture revision: no. Evidence:
accepted J05/J18 contracts; external queue behaviour unresolved.

## Class Pressure

| class | examples | result |
|---|---|---|
| Action | file move, GitHub create, email send, SQL mutation, drive mutation, printer submit, MIDI, smart lock, non-safety machine command | supported only when bounded |
| Query | file/PDF read, GitHub search, mail read, SQL read, drive get, AI response, sensor snapshot, printer status, MIDI state, machine monitor | first candidates/future |
| Anchor | webhook, mail inbound, drive changes, machine alarms, reduced sensor event | representable with identity/admission; deferred |
| Job | video render, background AI | reserved; never forced into Action |
| Stream | sensor/MIDI input, AI token stream | reserved; bounded reduction required |
| Human Task | approval queue | reserved; Ask is not equivalent |

## Security Pressure

| group | no credential | isolated credential | broker required | local filesystem | remote network | physical effect | safety refusal |
|---|---|---|---|---|---|---|---|
| file/PDF | file yes; PDF reference yes | production PDF | possible | yes | no by default | no | parser isolation boundary |
| cloud/API/AI | no | yes | acceptable alternative | scratch only | yes | no | no |
| webhook/SQL | no | yes | acceptable alternative | scratch only | yes | no | no |
| printer/MIDI/lock/machine | device-specific | yes | reviewed gateway | narrow | gateway only | yes | lock/machine safety refusal |
| human queue | no provider secret assumed | queue-specific | host gateway | narrow | possible | no | workflow not implemented |

## Outcome Pressure

| pressure | examples | required truth |
|---|---|---|
| definitive success | bounded file read, validated Query, spool acceptance | only the contract-proven fact |
| definitive failure | trusted refusal, invalid output, known no-effect cancellation | failed only with final evidence |
| uncertainty | SQL commit loss, email submission loss, printer state, lock/machine loss, render timeout | uncertain, never convenient failure |
| partial completion | PDF pages, uploads, renders, machine operation | contract-defined or failed/uncertain |
| unattempted refusal | policy, scope, approval, stale binding, unsafe integration | no canonical outcome or provider call |
| event deduplication | webhook, drive changes, mail inbound, alarms, sensors | stable source identity and durable admission |
| asynchronous completion | render, background AI, human queue | reserved Job/Human Task, not Action success |

## Cross-Example Contradiction Tests

1. Tether syntax change: **PASS**.
2. Vendor knowledge in Core: **PASS**; all translation stays provider/gateway.
3. Vendor-specific parsing in host policy: **PASS**; host uses reviewed bindings.
4. Provider trusted authority: **PASS**.
5. Query concealed mutation: **PASS**.
6. Action concealed Job/Stream: **PASS**.
7. Event without stable identity: **PASS**; such examples are refused/deferred.
8. Cursor mistaken for identity: **PASS**.
9. Success beyond evidence: **PASS**.
10. Timeout converted to failure: **PASS**.
11. Restart implied retry: **PASS**.
12. Idempotency authorised retry: **PASS**.
13. Signature/conformance granted permission: **PASS**.
14. Supervised provider called hostile-code isolated: **PASS**.
15. Credential-bearing supervised production provider: **PASS**; unavailable.
16. Physical integration implied safety certification: **PASS**.
17. Unsuitable integration forced into support: **PASS**.
18. First implementation remains File Tools and PDF-sized: **PASS**.
19. J18I can produce a roadmap without reopening semantics: **PASS**.
20. Final architecture freeze recommended: **PASS**, subject to Lucy accepting this validation.

## Revision Register

None. Every pressure case fits through accepted support, deferral, reservation,
brokering, gateway mediation or refusal. No accepted document or section requires
revision, and J18I remains blocked until Lucy accepts J18H.

## Final Freeze Recommendation

Recommend final architecture freeze after Lucy paper-validation review. Keep
implementation unauthorised, preserve J18G security restrictions, and keep Job,
Stream and Human Task reserved. The first implementation remains credential-free
File Tools and bounded PDF Tools reference work only.

VALIDATED
