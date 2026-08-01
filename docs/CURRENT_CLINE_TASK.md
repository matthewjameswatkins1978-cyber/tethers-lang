# Current Implementation Task

Control contract: `1`

Task: `J18F - Lifecycle, Outcomes, Events and Conformance v1`
Owner: `Luna`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `Luna on OpenCode, lifecycle and evidence architecture audit`
Base branch: `main`
Base commit: `eb6548ca61a2c8b108e675f01f3512f0598bc6b6`
Branch: `luna/j18f-lifecycle-events-conformance-v1`
Worker note: `docs/worker-notes/2026-08-01-j18f-lifecycle-events-conformance-v1.md`

## Objective

Define the canonical J18 contract for installation/provider lifecycle, sessions,
health, binding freshness, operation attempts, canonical outcomes, Result
Anchors, restart/replay, inbound Plug Anchors, event identity/admission,
acknowledgement/cursors, and Plug conformance evidence. Documentation only.

## Relevant background and existing behaviour

J18B, J18C, J18D, and J18E are accepted documentation contracts. Released
Tethers 0.2.0 is peeled by `v0.2.0` to `b5546411661dcbcb53e1cf2538eaec594c6f76f2`.
J06 defines the attempt boundary and outcomes; J09 defines durable replay;
J10/J11 define serial Result Anchor delivery and event admission.

## Required behaviour

1. Define separate lifecycle, health, catalogue, binding, operation, outcome,
   replay, event, and conformance state families.
2. Preserve canonical outcomes, Result Anchors, replay, event, and causal
   guarantees.
3. Define stable external event identity, durable admission, acknowledgement,
   cursors, and conformance evidence without implementation claims.
4. Update J18E status, decision log, current-state documents, and worker note.

## Relevant components

- `docs/architecture/TETHERS_LIFECYCLE_OUTCOMES_EVENTS_CONFORMANCE_V1.md`
- `docs/J06_DEADLINE_OUTCOME_DESIGN.md`
- `docs/J09_DURABLE_REPLAY_DESIGN.md`
- `tethers-0.1/host-rust/src/outcome.rs`
- `tethers-0.1/host-rust/src/replay_runtime.rs`
- `tethers-0.1/host-rust/src/result_anchor.rs`
- `tethers-0.1/host-rust/src/event_admission.rs`
- `tethers-0.1/host-rust/src/event_queue.rs`
- `tethers-0.1/host-rust/src/stdio_provider.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`

## Frozen decisions and invariants

- Core remains deterministic and application-agnostic.
- Unattempted is not an execution outcome.
- Canonical outcomes remain succeeded, failed, and uncertain.
- Durable replay authority remains separate from Trail and never retries.
- Conformance is evidence and does not grant permission or enablement.
- J18F authorises no implementation, schema, provider, or Tether change.

## Frozen boundaries

- Tethers 0.1 syntax and semantics remain unchanged.
- Existing 0.2 outcome, replay, Result Anchor, event queue, admission, Trail,
  generation 0 through 8, and no-retry guarantees remain unchanged.
- Unattempted is not an execution outcome; outcomes remain succeeded, failed,
  and uncertain.
- Replay authority remains separate from Trail and never authorises retry.
- Conformance does not grant permission, approve, install, or enable a Plug.
- J18F authorises no implementation.

## Required outcome

Create the canonical architecture document, accept J18E in its status block,
prepend the J18F decision entry, align current-state documents with J18F active
and J18G next, and create the worker note. Preserve all runtime and trust
boundaries.

## Required inspection

Inspect the accepted Universal Plug, Socket, MCP binding, package, and J18E
documents; J06 outcome and J09 replay designs; J10/J11 worker notes; and
`outcome.rs`, `replay_runtime.rs`, `result_anchor.rs`, `event_admission.rs`,
`event_queue.rs`, `stdio_provider.rs`, and `host_execution.rs` without changing
them.

## Authorised paths

- `docs/architecture/TETHERS_LIFECYCLE_OUTCOMES_EVENTS_CONFORMANCE_V1.md`
- `docs/architecture/TETHERS_CAPABILITIES_EFFECTS_SCOPES_V1.md`
- `docs/DECISIONS.md`
- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`
- `docs/TASK_QUEUE.md`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-01-j18f-lifecycle-events-conformance-v1.md`

## Forbidden changes

Do not modify Rust, OCaml, Cargo, Dune, opam, scripts, tests, fixtures,
manifests, runtime configuration, schemas, packages, providers, MCP
transcripts, replay/event-admission storage, Tether specification, Constitution,
release notes, tags, or GitHub Releases.

## Acceptance criteria

1. Exactly eight authorised documentation paths change.
2. J18E is marked accepted.
3. Lifecycle state families and capability-specific readiness remain separate.
4. Existing 0.2 outcome, replay, Result Anchor, event, and causal contracts are
   preserved.
5. External event identity/admission and acknowledgement/cursors are honest.
6. Conformance is host-orchestrated evidence without authority.
7. No retry, schema, runtime, provider, or Tether change is introduced.
8. Required checks pass; worktree is clean after commit; branch is pushed.

## Required verification

- `git diff --check`
- exact changed-path and clean-worktree checks
- task-packet checker
- required outcome, event, conformance, forbidden-conflation, and artifact
  searches from the task request
- published main and peeled `v0.2.0` verification

## Stop conditions

Stop on base, branch, published-ref, worktree, ownership, authorised-path, or
required-boundary mismatch; any false implementation claim; any failed check; or
any need to redesign frozen semantics. After two materially similar failed
attempts, stop with exact evidence and one smallest unresolved question.

## Expected pre-existing changes

None on the new J18F branch before this task.

## Commit and publication boundary

Create exactly one commit: `docs: define plug lifecycle and conformance`.
Push only `luna/j18f-lifecycle-events-conformance-v1`. Do not push main, tags,
releases, or begin J18G.
