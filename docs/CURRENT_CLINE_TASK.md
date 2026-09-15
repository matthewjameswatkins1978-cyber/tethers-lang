# TETHERS x RESOLVE01 - P3 Resolve Guard Transport and Durable Coordination

Task: `TETHERS x RESOLVE01 / P3 - Resolve Guard Transport & Durable Coordination`

Control contract: `1`

Status: `BLOCKED`

Task colour: `Red`

Owner: `Codex`

Route: `Implement the real Resolve guard transport and bounded outcome-delivery coordination only after Gate 0 proves that Resolve has published an accepted, versioned and sufficiently specific S3 protocol contract. Until then, preserve the accepted P2 boundary and make no production changes.`

Base commit: `ccbb567b938d4fc1edaeba48d249f684b0e790bb`

Implementation checkpoint: `WORKTREE`

Worker note: `docs/worker-notes/2026-09-15-tethers-resolve-p3.md`

Suggested branch:

`codex/tethers-resolve-p3-transport-coordination`

## Objective

Connect the accepted P2 `ResolveGuardAdapter` to the real Resolve01 guard
protocol and, only where the accepted protocol requires it, add bounded durable
coordination and outcome delivery. This packet is blocked at Gate 0 because no
accepted Resolve S3 contract could be located. P3 must not invent that contract.

## Relevant background and existing behaviour

P2 is accepted on `main` at `ccbb567b938d4fc1edaeba48d249f684b0e790bb`.
Tethers currently owns preparation, permission, replay, durable intent,
provider execution, provider outcomes and Trail evidence. P2 provides the
closed `Admitted`, `Rejected`, `Indeterminate` adapter boundary, but deliberately
contains no live transport or outcome delivery.

The Resolve repositories inspected were `C:\dev\resolve-ai` at
`c684ee60d563d6cb0a03ce5066aad8e518978070` and
`D:\The Next Thing\resolve-ai` at
`3e302e7d33c06219d5ac2f56373e64005bcdc83d`. Neither contains an accepted S3
guard protocol contract, and neither has an S3/guard/transport contract branch
or pull request in its fetched repository state.

## Required behaviour

1. Do not write live transport, response parsing, timeout handling, durable
   coordination, or outcome delivery until Gate 0 is satisfied by an accepted
   Resolve-owned S3 contract.
2. Once Gate 0 is satisfied in a future packet, implement exactly that
   contract below the existing P2 adapter, fail closed on all undefined or
   malformed protocol states, preserve durable Tethers truth before any
   delivery, and keep provider calls at zero for admission failure.

## Frozen decisions and invariants

- Resolve owns its guard validity, Claim/Commitment, worker, lease, recovery,
  wire protocol and outcome application semantics.
- Tethers remains the sole authority for permission, scope, approval, replay,
  durable intent, provider execution, provider outcomes and Trail.
- No guessed transport, protocol version, request/response shape, timeout,
  authentication/trust boundary, malformed-response or delivery-idempotency
  rule is valid.
- No automatic admission or outcome-delivery retry is allowed without an exact
  Resolve-owned idempotency contract and separate proof.
- No Resolve database access, generic RPC/HTTP framework, Core change, syntax
  change, policy change, new scope form, provider outcome, or replay authority.
- Gate 0 failure is a clean block, not a Tethers product failure.

## Acceptance criteria

1. Gate 0 records an accepted Resolve repository, contract file, accepted SHA,
   protocol version, request/response identities, timeout semantics and
   outcome-delivery/idempotency rules before implementation begins.
2. No production transport or coordination code is added while Gate 0 is
   unsatisfied; the blocked result and smallest required next action are
   recorded in the worker note.

## Relevant components

- `tethers-0.1/host-rust/src/resolve_guard.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/host-rust/src/dispatch.rs`
- `tethers-0.1/host-rust/src/replay.rs`
- `tethers-0.1/host-rust/src/replay_runtime.rs`
- `tethers-0.1/host-rust/src/outcome.rs`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P3_TRANSPORT_COORDINATION.md`
- `docs/worker-notes/2026-09-15-tethers-resolve-p3.md`

## Required verification

For this Gate 0 audit, run the repository tool diagnostic, inspect fetched
Resolve repository refs and contract candidates, run the task checker, verify
the branch contains no production changes, and run `git diff --check`. A future
unblocked P3 must run the full transport, malformed-input, timeout, bounded-I/O,
crash, delivery, Rust, cross-language, compatibility, secret, documentation and
authoritative verification route.

## Forbidden changes

- No Rust production implementation while Gate 0 is blocked.
- No OCaml/Core/parser/AST/evaluator/Plan/Tether syntax changes.
- No Resolve transport or protocol guessed from Tethers or old plans.
- No Resolve SQLite/database access, generic client framework, scheduler,
  retry framework, distributed transaction, public Plug, or provider change.
- No direct `main` update, force-push, history rewrite, or unrelated cleanup.

## Stop conditions

Stop until Resolve S3 is accepted, versioned, Resolve-owned and specific enough
to implement without guessing. Stop again if the contract leaves identity,
timeout, authentication/trust, malformed-response, or delivery-idempotency
semantics undefined. After two materially similar failed approaches to the same
contract/design issue, report the exact evidence and one smallest unresolved
question.

## Implementation scope

While blocked: task packet and worker note only. When Gate 0 is later passed,
the bounded scope is the Rust-host adapter implementation, its focused tests,
bounded coordination evidence, the P3 architecture note, this packet and the
named worker note. The packet checker must remain task-scoped.

## Expected pre-existing changes

None.
