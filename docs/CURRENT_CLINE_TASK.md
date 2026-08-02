# Current Implementation Task

Control contract: `1`
Task: `J19-M5 - Autonomous Durable Local Anchor Vertical Slice`
Owner: `Luna / OpenCode`
Status: `COMPLETE`
Task colour: `Amber`
Route: `Luna in one continuous autonomous Milestone 5 implementation`
Base branch: `main`
Base commit: `e57bf536fe3d7fb074c00ddac867b5720a15116e`
Accepted M4 baseline: `e57bf536fe3d7fb074c00ddac867b5720a15116e`
Frozen architecture base: `a5fd63593a9d9acd397030ecd2e27b4f318c87fd`
Branch: `opencode/j19-m5-durable-local-anchor`
Worker note: `docs/worker-notes/2026-08-02-j19-m5-durable-local-anchor.md`

## Start rule

Use Luna in OpenCode. Fetch the remote branch above, switch to it, require a clean worktree, and record the exact branch HEAD containing this control packet in the worker note before implementation.

Read `AGENTS.md`, this task, the accepted M4 worker note, and the governing architecture documents. Run:

```powershell
just tools
```

Use normal commits and normal pushes only. Do not amend or rebase published work, force-push, move `main`, move tags, create a release, or begin M6.

## Mission

Complete Milestone 5 as one coherent autonomous run:

1. freeze one bounded local inbound event contract;
2. implement host-owned durable event admission;
3. convert accepted events into canonical generation-0 Anchors;
4. prove restart-safe duplicate handling, conflict quarantine, acknowledgement ordering, evaluation, Result Anchors and Trail;
5. complete the full regression matrix.

Return only with:

`M5 COMPLETE - DURABLE LOCAL ANCHOR`

or a genuine major blocked report after two materially different evidence-based attempts.

## Objective

Allow one enabled local reference Plug or bounded local source to report one stable event back to Tethers safely.

The provider notification is never automatically an Anchor. The host must authenticate the active Plug/session, validate the exact event contract and scope, durably admit or refuse the external event, and only then create one canonical root Anchor for evaluation.

The accepted flow is:

outside/local source -> provider notification -> Socket/binding -> host validation -> durable external-event admission -> canonical root Anchor generation 0 -> J11 generation checks -> Trail -> evaluation -> ordinary Action/Query path -> acknowledgement.

## Relevant background and existing behaviour

M4 provides the enabled local File Tools Plug, supervised Windows launch,
host-owned scope binding, policy, durable intent, replay, canonical outcomes,
Result Anchors and Trail ordering. Existing event admission only covers
process-local causal checks and the existing follow-up queue handles generated
Result Anchors. M5 must add a separate durable external-event authority and
host-created generation-0 root Anchor without replacing those M4 seams or the
released 0.2 runtime.

## Required behaviour

1. Freeze one small local event, preferably `file.received@1` or the smallest equivalent, with strict schema and stable provider-issued event identity.
2. Accept events only from one exact enabled installed Plug identity, active session, provider identity, capability/event binding, and host-approved source scope.
3. Persist admission before evaluation or acknowledgement.
4. Treat same event ID plus same canonical digest as a duplicate: no second evaluation, no second root Anchor, return the original durable admission result.
5. Treat same event ID plus different canonical digest as a conflict: refuse evaluation, preserve conflict evidence, quarantine the source/session where appropriate, and do not acknowledge success.
6. Create exactly one host-owned root Anchor at generation 0 only after durable admission succeeds.
7. Preserve J11 causal limits: generation 0 through 8 accepted; generation 9 and above rejected.
8. Acknowledge the provider only after durable admission. A cursor, sequence, timestamp or transport offset is not event identity.
9. Preserve Trail and evaluation ordering and all M4 operational Plug behaviour.

## Relevant components

- Existing `event_admission.rs`, `event_queue.rs`, `dispatch.rs`, `application.rs`,
  `result_anchor.rs`, and Trail implementations.
- M4 installed/enablement/session and File Tools host boundaries.
- New strict inbound event contract, durable admission store, host Anchor
  creation path, local source fixture, and Windows integration evidence.

## Acceptance criteria

1. One strict stable local event contract and binding passes positive and negative validation tests.
2. Durable admission survives restart, corruption refuses safely, and acknowledgement follows durable publication.
3. Same-ID/same-digest delivery is a duplicate without reevaluation; same-ID/different-digest is quarantined conflict evidence.
4. Accepted admission creates exactly one generation-0 host root Anchor and preserves generation 8/9 boundaries.
5. Real Windows end-to-end evidence proves enablement, admission, evaluation, Trail ordering, restart deduplication, conflict refusal, and disablement.
6. M3, M4, Rust, OCaml, 0.2, formatting, process-cleanup, and packet checks pass.
7. Invalid identity, binding, schema, digest, scope, reparse, payload, and generation inputs are refused without evaluation or success acknowledgement.
8. Admission publication failure creates no Anchor and no acknowledgement; recovery after admission does not duplicate evaluation.
9. No provider notification is treated as an Anchor, no cursor/timestamp substitutes for identity, and no M6 or external Anchor behaviour is added.

## Frozen architectural truths

- Tethers coordinates; the host owns trust, admission and execution.
- A provider notification is untrusted input, not an Anchor.
- Admission state is separate from attempted provider-operation outcomes.
- Stable external event ID and canonical event digest are both required.
- Duplicate same-ID/same-digest is not a new event and must not evaluate again.
- Same-ID/different-digest is a conflict, never last-write-wins.
- Durable admission precedes acknowledgement and Anchor creation.
- Root Anchors are host-created at generation 0.
- No Result Anchor is created merely for inbound event admission.
- Any Actions/Queries caused by evaluation retain normal intent, replay, outcome, Result Anchor and Trail semantics.
- No automatic retry of provider operations.
- Cursor/offset/timestamp is not event identity.
- No network listener, credential delivery, PDF feature, marketplace, updater or M6 work.

## Frozen decisions and invariants

The frozen architectural truths, event contract, durable admission rules, source
boundary, and host Anchor ordering above are binding implementation invariants.

## Event contract

Freeze the smallest direct v1 contract before implementation.

Use a strict event envelope containing at least:

- event format version;
- stable provider-issued event ID;
- exact event name and version;
- provider identity and installed Plug identity;
- active session identity;
- occurred time as presentation/audit data only;
- canonical payload;
- canonical payload/event digest;
- optional source-relative path bound to host-approved scope;
- causal generation/source metadata required by the accepted lifecycle contract.

Reject duplicate JSON fields, unknown fields, malformed IDs, unsupported versions, wrong provider/session/installed identity, wrong event binding, schema drift, invalid digest, path escape, reparse/junction scope violations, oversized payloads and secret-bearing diagnostics.

The package or provider cannot define host admission authority. Event schemas and bindings must be part of exact installed and enabled evidence.

## Durable admission store

Implement a separate host-owned durable external-event admission authority. Do not reuse Trail, replay, Result Anchor queue, enablement store or operation outcome records as the authority.

The store must preserve at least:

- stable external event ID;
- canonical event digest;
- exact installed Plug, provider, session and event binding identity;
- admission state and reason;
- durable root Anchor identity where admitted;
- first-seen/admitted audit time;
- schema version and record-integrity digest;
- conflict evidence without replacing the original admitted record.

Required properties:

- create-only or explicit append-only transitions;
- crash-aware atomic publication;
- strict reload and corruption refusal;
- reparse-safe host-owned root;
- deterministic restart reconstruction;
- no forked authority;
- no acknowledgement before durable publication;
- no evaluation if publication fails;
- no second evaluation on duplicate after restart.

## Source and Socket behaviour

Use one bounded local source only. It may be the File Tools reference Plug or a dedicated credential-free local fixture provider.

The source must:

- run through the accepted installed/enabled Plug lifecycle where applicable;
- use the accepted Socket/MCP local stdio boundary;
- emit only the reviewed notification/event shape;
- have no network access or credentials;
- use one host-approved disposable source root;
- generate stable event IDs independent of transport cursor or process restart;
- tolerate host duplicate responses without inventing a new event identity;
- remain removable/disableable without affecting Tethers core.

Do not add general filesystem watching, arbitrary directory recursion, networking, webhook listeners or a broad event bus. One deterministic local event is enough.

## Host admission and Anchor creation

Before admission, require:

- exact installed Plug and current enablement;
- current trust/developer approval and installed-byte validity;
- active authenticated provider session;
- exact event name/version/schema binding;
- exact host-approved source scope;
- canonical digest verification;
- payload bound and secret-safe diagnostics;
- generation and causal checks.

After durable admission:

1. create one host-owned canonical root Anchor at generation 0;
2. record admission and Anchor relationship durably;
3. append ordered Trail evidence;
4. submit the Anchor through the existing event/evaluation path;
5. acknowledge the provider only after durable admission and required host publication succeed.

If durable Anchor publication fails after admission, fail closed and do not acknowledge success. Recovery must resume deterministically without creating a second evaluation.

## Required evidence

Provide a real Windows end-to-end scenario proving:

- package/install/trust/conformance/approval/enablement remains valid;
- event source unavailable while Plug is disabled;
- explicit enablement starts one accepted local source session;
- one valid event is durably admitted;
- one generation-0 root Anchor is created;
- one evaluation occurs;
- resulting Action/Query uses the existing M4 host path;
- Trail ordering is correct;
- acknowledgement happens after durable admission;
- restart preserves admission and prevents duplicate evaluation;
- disablement stops new event admission without affecting Tethers core.

Also prove:

- same ID and same digest is a duplicate with no second evaluation;
- same ID and different digest is a conflict with no evaluation;
- malformed/unknown/schema-drift events refuse;
- wrong Plug/provider/session/binding refuses;
- path escape/reparse/scope mismatch refuses;
- generation 8 accepted and generation 9 rejected where the existing causal path applies;
- admission-store torn write/corruption refuses safely;
- failure before durable admission produces no Anchor and no acknowledgement;
- failure after durable admission but before evaluation recovers once without duplicate evaluation;
- provider process loss does not corrupt admission authority;
- no provider process survives the test;
- M4 Query/Action behaviour remains unchanged.

## Autonomy and judgement

Luna may choose:

- exact module/type/store names;
- event name during the contract-freeze checkpoint;
- exact strict schema spelling and committed fixture layout;
- durable store layout and recovery representation;
- session identity representation;
- acknowledgement response representation;
- deterministic local fixture/source design;
- thin `justfile` recipes;
- conservative refactoring needed to connect durable admission to the existing event/evaluation path.

Prefer reuse over a second event engine. Avoid broad rewrites. Keep commits coherent and reviewable.

## Forbidden changes

No general event marketplace, network listener, webhook server, credentials, arbitrary filesystem watcher, recursive directory crawl, PDF support, jobs, streams, Human Tasks, auto-update, release work or M6 behaviour.

Do not conflate:

- notification with Anchor;
- admission with operation outcome;
- cursor with event identity;
- Trail with admission authority;
- enablement with event permission;
- duplicate delivery with a new event.

## Major stop rule

Stop only when one of these remains after two materially different evidence-based attempts:

- stable identity/deduplication cannot be guaranteed across restart;
- durable admission and acknowledgement ordering cannot be proven;
- the existing event/evaluation path cannot accept a host-created generation-0 Anchor without architectural contradiction;
- same-ID/different-digest conflict cannot be contained safely;
- current trust/session/enablement cannot be bound to admission;
- released 0.2 or accepted M4 behaviour regresses and cannot be isolated;
- a frozen security boundary must weaken;
- repository corruption, missing baseline or unavailable required toolchain prevents progress.

Ordinary compiler errors, fixtures, store schemas, Windows API details, test harness work and cross-module integration are not major blockers.

## Stop conditions

The major stop rule above is exhaustive for this task. Stop only when one of
those conditions remains after two materially different evidence-based
attempts.

## Expected pre-existing changes

None beyond the accepted M4 baseline and the authorized M5 control/ledger
commits.

## Required verification

Run at minimum:

```powershell
just tools
just fmt
just check
just test-m3
just test-m4
just test-rust
just verify
```

Add and run focused M5 contract, admission, restart, conflict and Windows end-to-end tests.

Also run:

- complete OCaml build and tests;
- complete `verify-0.2.ps1`;
- locked debug and release Rust builds;
- packet/control validation;
- real Windows process-survivor checks;
- `git diff --check`;
- final clean-worktree check after the completion ledger commit.

Record exact counts and commands in the worker note. Local test claims are evidence, not CI; state that honestly.

## Completion response

Return only:

`M5 COMPLETE - DURABLE LOCAL ANCHOR`

plus the branch name and final SHA if OpenCode requires additional text.

Do not begin M6. Do not move `main`. Do not create a release.
