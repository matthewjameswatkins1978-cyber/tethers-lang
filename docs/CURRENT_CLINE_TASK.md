# TETHERS x RESOLVE01 - P4b End-to-End Guarded Lifecycle & Recovery

Task: `TETHERS x RESOLVE01 / P4b - End-to-End Guarded Lifecycle & Recovery`

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Codex`

Route: `Prove the complete guarded lifecycle and recovery composition across the accepted Tethers replay, durable intent, Resolve admission, provider, outcome and delivery authorities. Add only test-only fault injection and focused lifecycle evidence; do not introduce a new recovery engine or change product semantics.`

Base commit: `63aa8e21bfa359ad444d8b32b3007b398fa9002e`

Evidence checkpoint: `00361fd89a381ec48500ff0003e998073e18a347`

Worker note: `docs/worker-notes/2026-09-15-tethers-resolve-p4b-lifecycle-recovery.md`

Suggested branch:

`codex/tethers-resolve-p4b-lifecycle-recovery`

## Objective

Produce direct, deterministic evidence for the P4b guarded lifecycle and crash
recovery matrix. Demonstrate that Tethers remains authoritative, provider
effects are bounded to at most one per execution identity, admission is never
automatically retried, durable outcomes are redelivered exactly, and stale or
conflicting evidence fails closed.

## Relevant background and existing behaviour

P1 preparation proof, P2 guard admission, P3 live transport and P4 outcome
delivery are accepted on `main` at the base commit above. The existing host
already owns durable intent, replay/G1, provider invocation, outcome
classification, Result Anchors, Trail evidence and the transport-neutral
outcome journal. P4b composes and tests those authorities; it does not replace
them.

## Required behaviour

1. Cover no-intent, intent failure, admission rejection/indeterminacy,
   admission-evidence failure, pre-G1 failure, post-G1 uncertainty, provider
   failure/uncertainty, durable-outcome delivery failure, exact redelivery,
   Delivered terminality and conflict.
2. Prove that a guarded provider effect occurs at most once for an
   `ExecutionId`, including replay/restart recovery cases.
3. Prove that admission and provider execution are never automatically retried.
   Only explicit exact outcome redelivery is allowed after a durable outcome.
4. Prove that old admission state is not reusable after restart and that a
   fresh explicit attempt is required.
5. Prove that Tethers provider outcome truth is not rewritten by delivery or
   Resolve coordination failure.
6. Exercise the Together barrier: rejected, indeterminate and pre-stage-B
   failures produce no partial provider effect.
7. Add a named crash-boundary matrix and document which accepted authority
   supplies each recovery rule.
8. Use the accepted Resolve S3 contract for any available bounded smoke; record
   unavailable external prerequisites honestly.

## Relevant components

- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/dispatch.rs`
- `tethers-0.1/host-rust/src/replay_runtime.rs`
- `tethers-0.1/host-rust/src/resolve_guard.rs`
- `tethers-0.1/host-rust/src/resolve_outcome.rs`
- `tethers-0.1/host-rust/src/p4b_lifecycle.rs`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P4B_LIFECYCLE_RECOVERY.md`
- `docs/worker-notes/2026-09-15-tethers-resolve-p4b-lifecycle-recovery.md`

## Frozen decisions and invariants

- Tethers remains the sole authority for capability, manifest, provider,
  arguments, scope, binding, trust, policy, approval, replay, intent and
  provider outcome.
- Resolve coordinates admission and records durable outcome facts. Resolve
  never grants Tethers permission and never executes a provider.
- One explicit admission attempt is made. There is no automatic admission or
  provider retry.
- An outcome may be explicitly redelivered only after Tethers has durably
  recorded it; redelivery never reruns the provider.
- `Delivered` is terminal for the journal identity. `Conflict` is durable and
  does not rewrite Tethers outcome truth.
- Replay recovery is fail-closed. An old admission is not a substitute for a
  fresh current attempt.
- The Together barrier prevents any provider effect before the required guard
  admission and durable admission evidence.
- No new Resolve state machine, transaction protocol, scheduler, retry engine,
  database access, Core/OCaml change, syntax change, policy semantic change,
  provider execution change or Result Anchor taxonomy change.

## Acceptance criteria

1. The named P4b crash matrix covers all packet boundaries and has direct
   focused evidence for the consequential failure classes.
2. Provider effects are zero before admission/G1 and at most one after G1 for
   every tested execution identity.
3. Rejection, indeterminacy, replay recovery, delivery failure, terminal
   delivery, conflict and provider failure/uncertainty have distinct results.
4. Exact outcome redelivery survives Tethers-side restart without provider
   access; Delivered remains terminal.
5. Existing relevant P1/P2/P3/P4/P4a tests remain green.
6. No production retry or authority bypass is introduced.
7. Architecture and worker-note evidence match the implementation.
8. Focused and repository-authoritative verification, packet checks, secret
   scan, docs checks, whitespace checks and complete diff review pass.
9. The branch is normally pushed and merged only after review and green gates.

## Required verification

Run the repository-owned tool diagnostic and packet checker; Cargo formatting;
locked Rust check/build/test paths; focused P4b, P1, P2, P3, P4 and P4a tests;
replay and outcome-journal tests; warning ratchet; secret scan; documentation
checks; `just verify`; `git diff --check`; complete diff review; and final clean
status. Run a bounded real Resolve S3 smoke when its service/emulator
prerequisites are available. Record every unavailable or interrupted check
with its first real error.

## Forbidden changes

- No OCaml/Core/parser/AST/evaluator/Plan/syntax or policy-vocabulary changes.
- No Resolve database, Firestore, SDK, wire-protocol redesign, scheduler,
  distributed transaction, lease/claim/commitment model or generic retry
  framework.
- No automatic admission retry, provider retry, provider-order change, replay
  semantic change, new provider outcome or Result Anchor class.
- No raw secrets, arguments, paths, Resolve state or guard references in logs,
  Trail, diagnostics, fixtures or reports.
- No force push, history rewrite, direct main update or unrelated cleanup.

## Stop conditions

Stop and report if proving the lifecycle requires a new semantic authority;
provider effects cannot be bounded; old admission can be reused; conflict can
rewrite Tethers truth; Together can partially execute before admission; the
accepted P3 contract cannot be used; P4b requires Core/OCaml, syntax, policy,
database, scheduler, provider retry or distributed transaction changes; or two
materially similar implementation attempts fail against the same design issue.

## Expected pre-existing changes

None.

## Implementation scope

- `tethers-0.1/host-rust/src/application.rs` — test-module wiring only.
- `tethers-0.1/host-rust/src/dispatch.rs` — test-only Trail fault injection.
- `tethers-0.1/host-rust/src/p4b_lifecycle.rs` — focused composition evidence.
- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P4B_LIFECYCLE_RECOVERY.md`
- `docs/worker-notes/2026-09-15-tethers-resolve-p4b-lifecycle-recovery.md`

Do not broaden the packet checker to repository-wide inference.

## Remaining risks

The P4b tests use deterministic existing test seams rather than unsafe process
kill hooks. A live two-process Tethers/Resolve restart smoke remains dependent
on the separately provisioned accepted Resolve S3 environment and must be
reported as tested or unavailable, not inferred.
