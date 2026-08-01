# Current Implementation Task

Control contract: `1`
Task: `J18H - Universal Plug Paper Validation Matrix`
Owner: `Luna`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `Luna on OpenCode, adversarial architecture validation`
Base branch: `main`
Base commit: `41235a3093ed73b3d58533bcfad45ef490211560`
Accepted architecture base: `8f1f2c685fb9f700cf7c1dfe3d877958b8bea6f7`
Branch: `luna/j18h-paper-validation`
Worker note: `docs/worker-notes/2026-08-01-j18h-paper-validation.md`

## Control-plane starting rule

Fetch `origin/main`, switch to `main`, and fast-forward to the commit containing
this J18H control contract. Create `luna/j18h-paper-validation` from that updated
`main`.

The accepted architecture being validated remains exactly
`8f1f2c685fb9f700cf7c1dfe3d877958b8bea6f7`. The control-plane commit changes
only task authority and is not a new architecture decision. Final J18H changed-
path verification is measured against the accepted architecture base above.

## Objective

Paper-test the accepted J18 Universal Plug architecture against all sixteen
representative integrations required by J18B. Determine whether each example is
honestly supported, deferred, reserved, brokered, gateway-mediated, unsuitable,
or refused without changing Tether 0.1 semantics or inserting vendor logic into
Core.

This is documentation and paper validation only. Do not implement Plugs,
providers, schemas, packages, protocols, listeners, credentials, sandboxes,
storage, Jobs, Streams, Human Tasks, hardware control, or Tether syntax.

## Accepted contracts under test

- J18B Universal Plug Architecture.
- J18C Socket v1 and MCP stdio binding.
- J18D `.tetherplug` package v1.
- J18E capability classes, effects and scopes.
- J18F lifecycle, outcomes, events and conformance.
- J18G security, trust, credentials and sandbox.

J18B through J18G are accepted. Plug implementation remains unauthorised.
Language semantics remain `0.1`.

## Frozen laws

- Tethers coordinates; the host owns trust, policy, credentials, dispatch,
  replay, event admission and canonical outcomes.
- Providers own vendor-specific translation.
- Core remains application-agnostic.
- Class, effects, scope, policy and outcome remain distinct.
- Action, Query and Anchor are the first-programme classes.
- Job, Stream and Human Task remain reserved.
- Attempted outcomes remain exactly `succeeded`, `failed`, and `uncertain`.
- Unattempted is not an execution outcome.
- No automatic retry exists.
- Replay authority remains separate from Trail.
- External Anchors require stable identity and durable duplicate admission.
- Signing, conformance and installation do not grant operational permission.
- Supervised execution is not hostile-code isolation.
- Credential-bearing production providers require proven isolation or a reviewed
  host broker.
- Unsupported or unsafe systems are refused honestly.

## Required integrations

Validate all sixteen without removing, merging or renaming them:

1. local file tool;
2. PDF processor;
3. GitHub service;
4. email service;
5. SQL database;
6. cloud drive;
7. remote AI model;
8. local AI model;
9. webhook source;
10. long-running video renderer;
11. live sensor stream;
12. printer;
13. MIDI instrument;
14. smart lock;
15. industrial machine;
16. human approval queue.

## Required analysis for every integration

For every example record:

- technical possibility and architectural fit;
- provider, package and capability identities;
- exact class or reserved class and why;
- effects, structured scopes, policy and approval;
- Socket operation, protocol binding, transport and translation owner;
- discovery, drift, authentication, credential and isolation boundary;
- filesystem, network, resource, privacy and cost boundaries;
- attempt boundary and exact meanings of success, failure and uncertainty;
- unattempted cases, partial completion, cancellation and restart behaviour;
- replay, stable event/job identity, ordering, cursor and acknowledgement where
  relevant;
- Trail evidence, conformance, invalidation, install/removal consequences;
- first implementation status and exact refusal boundary;
- whether accepted architecture requires revision;
- evidence basis: accepted Tethers contract, primary source, inference, or
  unresolved fact.

No required field may be silently omitted. A compact structured template may be
used, but it must preserve every required question.

## Required pressure boundaries

- File Tools and PDF Tools remain the first-slice references.
- PDF input is hostile parser input.
- GitHub, email, SQL and cloud drive separate Query, Action and Anchor shapes.
- SMTP acceptance is not proof of recipient delivery or reading.
- SQL connection loss around commit may be uncertain.
- Cloud-drive notification is a change hint, not automatically the change data.
- Remote/local AI streaming is Stream-shaped; background inference is Job-shaped.
- Webhook admission requires authentication, stable identity and durable dedup.
- Long renders are Job-shaped; progress is not automatically a Stream.
- Continuous sensors and MIDI input are Stream-shaped.
- A spooler job ID does not prove paper emerged correctly.
- Smart-lock and industrial effects require physical/security refusal boundaries.
- Tethers is not a safety PLC, certified controller or hard-real-time loop.
- Immediate host `Ask` is not a general Human Task implementation.

## Relevant background and existing behaviour

J18B through J18G are accepted architecture contracts. Tethers 0.2.0 is the
published baseline. Action, Query and Anchor are first-programme candidates;
Job, Stream and Human Task remain reserved. The accepted architecture is
validated against representative integrations without implementation.

## Required behaviour

1. Validate all sixteen representative integrations without changing frozen
   semantics.
2. Record class, effects, scopes, policy, Socket/binding/transport, security,
   lifecycle, outcomes, events, Trail, conformance and refusal for each.
3. Test cross-example contradictions and produce one final verdict.
4. Update only the eight authorised Markdown paths and preserve implementation
   unauthorised.

## Relevant components

- `docs/architecture/TETHERS_J18_PAPER_VALIDATION.md`
- Accepted J18B through J18G architecture documents.
- `docs/DECISIONS.md`, current-state documents and J18H worker note.

## Frozen decisions and invariants

- Tethers coordinates; the host owns trust, policy, credentials, dispatch and
  outcomes; providers own vendor translation.
- Core remains application-agnostic and Tether 0.1 semantics remain unchanged.
- Job, Stream and Human Task remain reserved and unsupported.
- Unknown effects/scopes fail closed; no automatic retry exists.
- Stable event identity, durable admission, truthful outcomes and physical safety
  boundaries remain mandatory.

## Acceptance criteria

1. Exactly eight authorised Markdown paths differ from accepted architecture base.
2. All sixteen integrations are present and fully analysed.
3. Every integration has an honest class, security mode, lifecycle and refusal.
4. Summary tables, contradiction tests and revision register are complete.
5. Exactly one final verdict is present and consistent with the register.
6. J18G is marked accepted; J18H remains pending Lucy review; J18I remains
   blocked; implementation remains unauthorised.
7. No implementation, schema, provider, credential, sandbox or Tether change.

## Required verification

- Run whitespace, exact changed-path, staged-diff and task-packet checks.
- Run integration presence, class/disposition, truth-boundary, security and
  forbidden-conflation searches.
- Verify exactly one verdict, refs, clean topology and no implementation artifact.

## Forbidden changes

Do not modify Rust, OCaml, build files, scripts, tests, fixtures, manifests,
runtime configuration, schemas, packages, providers, credentials, trust stores,
keys, signatures, sandboxes, event/replay storage, Tether specification,
Constitution, release material, or begin J18I. Do not modify J18B through J18F.

## Stop conditions

Stop on branch, base, ownership, authorised-path or boundary mismatch; an
unresolved semantic contradiction; failed required checks; or a source claim
that cannot be supported. After two materially similar failed attempts, stop
with exact evidence and one smallest unresolved question.

## Expected pre-existing changes

The control-only J18H packet commit is expected on `main`; no working-tree
changes are expected before this task.

## Canonical output

Create:

`docs/architecture/TETHERS_J18_PAPER_VALIDATION.md`

Begin:

Status: J18H candidate, pending Lucy paper-validation review
Validation generation: 1
Implementation: Not authorised

End with exactly one verdict:

`VALIDATED`

or:

`REVISION_REQUIRED`

`VALIDATED` means all examples fit honestly through support, reservation,
deferral, brokering, gateway mediation or refusal without changing frozen
semantics.

`REVISION_REQUIRED` means at least one example exposes a genuine contradiction
or missing authority boundary. Record the affected document and section, the
smallest correction, affected examples and whether J18I remains blocked.

Do not silently edit accepted J18B through J18F contracts in this task.

## Required summary material

Include:

- integration disposition table;
- class pressure table;
- security pressure table;
- outcome pressure table;
- cross-example contradiction results;
- revision register;
- final-freeze recommendation.

The cross-example review must explicitly test vendor leakage into Core/host
policy, concealed mutation, false Action/Query use, unstable event identity,
excess success claims, timeout-to-failure conversion, retry after restart,
permission from signatures/conformance, false supervised isolation, unsafe
credential delivery, physical safety claims and first-slice size.

## Source discipline

Use accepted repository documents for Tethers facts and official primary sources
for external integration facts. Distinguish source fact, accepted Tethers rule,
architectural inference and unresolved fact. Record exact source title, final URL
and access date in the worker note. Do not invent delivery guarantees,
identifiers, cancellation, transaction, ordering or safety behaviour.

## J18G acceptance alignment

Update the J18G status block only to:

Status: Accepted J18G security contract
Accepted by Lucy: 2026-08-01
Final architecture freeze: Requires J18H paper validation
Implementation: Not authorised

Do not otherwise redesign J18G.

## Current-state and decision updates

Update the decision log and current-state documents to record:

- J18B through J18G accepted;
- J18H active and pending Lucy review;
- actual J18H verdict;
- architecture not finally frozen until Lucy accepts J18H;
- J18I blocked until J18H acceptance;
- implementation remains unauthorised;
- refusal and deferral count as valid validation results;
- no implementation or Tether semantic change.

Do not claim Lucy accepted J18H.

## Authorised changed paths

Exactly eight paths relative to accepted architecture base
`8f1f2c685fb9f700cf7c1dfe3d877958b8bea6f7`:

1. `docs/architecture/TETHERS_J18_PAPER_VALIDATION.md`
2. `docs/architecture/TETHERS_SECURITY_TRUST_CREDENTIALS_SANDBOX_V1.md`
3. `docs/DECISIONS.md`
4. `docs/CURRENT_GOAL.md`
5. `docs/PROJECT_DASHBOARD.md`
6. `docs/TASK_QUEUE.md`
7. `docs/CURRENT_CLINE_TASK.md`
8. `docs/worker-notes/2026-08-01-j18h-paper-validation.md`

No other path may change.

## Worker note

Create:

`docs/worker-notes/2026-08-01-j18h-paper-validation.md`

Use headings:

- Task
- Changes
- Validation method
- Repository contracts inspected
- External primary sources inspected
- Integration findings
- Cross-example findings
- Revision candidates
- Tool bootstrap
- Evidence
- Discoveries
- Remaining risks
- Final verdict
- Next action
- References

Record resolved tool paths and versions, every source and redirect/failure, the
exact verdict, and confirmation that no implementation or schema changed.

## Acceptance criteria

1. Exactly eight authorised Markdown paths differ from accepted architecture
   base `8f1f2c685fb9f700cf7c1dfe3d877958b8bea6f7`.
2. All sixteen integrations are present and fully analysed.
3. Every example has an honest class, reserved class, mediation or refusal.
4. No example changes Tether syntax or introduces vendor logic into Core.
5. Canonical outcome, uncertainty, replay and no-retry laws remain intact.
6. Event identities, cursors and acknowledgements are not conflated.
7. Credential and isolation claims match accepted J18G.
8. Physical and industrial integrations do not imply safety certification.
9. Summary tables, contradiction tests and revision register are complete.
10. Exactly one final verdict is present and consistent with the register.
11. J18G is marked accepted; J18H is not falsely marked accepted.
12. J18I remains blocked until Lucy accepts J18H.
13. No implementation, schema, package, provider, credential or Tether change.
14. Diff, staged-diff, task-packet and content checks pass.
15. Worktree is clean and branch is pushed normally.

## Forbidden changes

Do not modify Rust, OCaml, Cargo, Dune, opam, scripts, tests, fixtures,
manifests, runtime configuration, JSON Schema, packages, ZIPs, providers,
protocol transcripts, credentials, trust stores, keys, signatures, AppContainer
profiles, event/replay storage, Tether specification, Constitution, release
notes, tags, GitHub Releases, or an implementation roadmap.

Do not begin J18I.

## Commit and publication boundary

Create one J18H work commit:

`docs: paper-validate universal plug architecture`

Push only:

`luna/j18h-paper-validation`

Do not push `main`, tags or releases.

## Completion report

Begin exactly:

`COMPLETE - READY_FOR_LUCY_PAPER_VALIDATION_REVIEW`

Report branch/commit, tools, exact paths, final verdict, disposition and class
coverage, first-slice/deferred/refused conclusions, outcome/event/security/safety
findings, revision register, freeze recommendation, J18G status update, sources,
checks, refs, clean topology, confirmation of no implementation change, and the
smallest next action.

On failure begin exactly:

`BLOCKED`
