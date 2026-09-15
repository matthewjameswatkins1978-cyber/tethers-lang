# TETHERS x RESOLVE01 - P3 Live Guard Transport & Outcome Delivery

Task: `TETHERS x RESOLVE01 / P3 - Live Guard Transport & Outcome Delivery`

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Red`

Owner: `Codex`

Route: `Implement the bounded Rust-host HTTPS adapter for the accepted Resolve v1 guard-admission contract and explicit durable delivery of already-recorded Tethers provider outcomes. Preserve Tethers authority and accepted P2 ordering. Do not alter Core, syntax, provider execution, replay semantics, or Resolve data access.`

Base commit: `9fb8517acd1192e9a1dbee8fa70c0f520d6c0fad`

Worker note: `docs/worker-notes/2026-09-15-tethers-resolve-p3-live-transport.md`

Suggested branch:

`codex/tethers-resolve-p3-live-transport`

## Objective

Add the production Rust-host transport under the accepted P2 seams. Admission
uses authenticated HTTPS+JSON against Resolve protocol
`resolve.tethers-guard/1`; outcome delivery uses the same protocol after the
existing Tethers provider outcome is durable. The implementation must fail
closed, verify semantic response digests, preserve exact P2 ordering, and make
delivery recovery explicit without adding a generic scheduler.

## Relevant background and existing behaviour

P0/P1/P2 preparation, opaque ScopeKeys, exact current preparation, the closed
`Admitted`/`Rejected`/`Indeterminate` seam, durable intent, replay/G1, provider
execution, Trail, outcomes and Result Anchors are accepted on main. P4 already
provides a transport-neutral outcome journal. Resolve S3 at
`b450305e72815d33357359c6db641d315d6b2975` freezes the admission and outcome
HTTP contract and proves outcome idempotency.

## Required behaviour

1. Implement one small Rust-host HTTPS+JSON client for `/internal/tethers/v1/guard/admit` and `/internal/tethers/v1/outcome`, using the existing JCS/SHA-256 authority and accepted P2 projections.
2. Require an explicit host-owned base URL and bridge key, default to HTTPS, bound timeouts and bodies, disable redirects, redact the key, and keep ordinary unguarded execution unchanged.
3. Serialize exact v1 requests and strictly validate protocol version, digest, closed result vocabulary, UTF-8, duplicate members, unknown fields and bounded responses. Transport inability maps to P2 `Indeterminate` and never triggers an automatic admission retry.
4. Preserve P2 ordering and prove negative admission cases make zero provider calls, while an admitted request reaches the existing provider boundary once.
5. Extend outcome delivery identity with action identity plus preparation digest, persist only bounded delivery evidence, treat `ALREADY_RECORDED` as success, and record `CONFLICT` without rewriting Tethers outcome or retrying automatically.
6. Allow only explicit later redelivery of an already durable outcome; never rerun a provider, mutate Result Anchor taxonomy, or turn notification failure into provider uncertainty.
7. Add strict fake-server/contract tests and, where the local accepted Resolve environment is available, a bounded cross-system smoke against Resolve S3.
8. Document the actual P3 implementation, security properties, recovery states, configuration, verification evidence and explicit exclusions.

## Relevant components

- `tethers-0.1/host-rust/src/resolve_guard.rs`
- `tethers-0.1/host-rust/src/resolve_outcome.rs`
- `tethers-0.1/host-rust/src/resolve_transport.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/tests/resolve_r0_p4_conformance.rs`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P3_LIVE_TRANSPORT.md`
- `docs/worker-notes/2026-09-15-tethers-resolve-p3-live-transport.md`

## Frozen decisions and invariants

- Tethers remains the sole authority for capability, manifest, provider,
  arguments, scope, binding, trust, policy, approval, replay, intent and
  provider outcome.
- Resolve only coordinates admission and records a durable outcome fact. It
  cannot grant Tethers permission or execute a provider.
- Admission is one request per explicit attempt, with no automatic retry.
- Outcome redelivery is permitted only because the accepted Resolve S3 identity
  is action plus preparation plus outcome and is idempotent.
- No raw arguments, provider secrets, raw guard references, Resolve state or
  database data cross the bridge or enter diagnostics.
- No Core, OCaml, Tether syntax, Plan, replay, provider ordering, Result Anchor
  taxonomy, public Plug, Resolve database, or live generic coordination change.

## Acceptance criteria

1. The HTTPS client implements the exact accepted Resolve v1 routes, headers, request shapes and response mappings.
2. Current P2 adapter projections are used directly; request and response digests are independently verified with existing canonicalization authority.
3. TLS, timeout, redirect, body, UTF-8, duplicate-member, unknown-field, unknown-version, unknown-decision, wrong-digest and authentication failures fail closed without provider calls.
4. Default unguarded execution remains unchanged; guarded missing configuration cannot silently execute.
5. Durable outcome delivery is keyed by action plus preparation identity, preserves Tethers outcome truth, supports explicit exact redelivery, and records conflicts safely.
6. Focused admission, outcome, negative-path, secret-redaction, idempotency and no-provider-retry tests pass.
7. Relevant Rust checks, formatting, warning ratchet, packet checker, secret scan, docs checks, whitespace checks and repository verification are run with exact failures recorded.
8. Architecture and worker-note evidence match the implementation; the complete diff is scoped and independently reviewed.
9. The branch is normally pushed and merged only after verification and review; canonical main is then refreshed and proved clean.

## Required verification

Run the repository-owned tool diagnostic and packet checker; `cargo fmt --all --
--check`; locked Cargo check/build/test paths; focused transport, guard and
outcome tests; relevant P2/P4 conformance; warning ratchet; secret scan;
documentation/link checks; `just verify`; `git diff --check`; complete diff
review; and final clean status. Run the accepted Resolve S3 tests or bounded
local smoke when the emulator/service prerequisites are available. Record
unrun or externally blocked checks rather than inheriting historical PASS.

## Forbidden changes

- No OCaml/Core/parser/AST/evaluator/Plan/syntax or policy-vocabulary changes.
- No Resolve SDK, Firestore/database access, MCP/Socket protocol, generic HTTP
  framework, scheduler, lease/claim/commitment model or live guard authority.
- No provider retry, admission retry, execution-order change, replay change,
  new provider outcome or Result Anchor class.
- No raw secret, argument, scoped resource or guard reference in logs, Trail,
  errors, fixtures or reports.
- No force push, history rewrite, direct main update or unrelated cleanup.

## Stop conditions

Stop and report if the accepted Resolve contract cannot be matched exactly;
canonical digest compatibility is unproven; secure endpoint trust cannot be
bounded; Trail/outcome persistence would require a new semantic decision;
transport would grant authority; P3 requires Core/OCaml changes, provider
execution, Resolve database access or a generic coordination framework; or a
second materially similar implementation attempt fails against the same
design issue. Do not merge with a real correctness or contract blocker.

## Expected pre-existing changes

None.

## Implementation scope

- `tethers-0.1/host-rust/Cargo.toml`
- `tethers-0.1/host-rust/Cargo.lock`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/src/resolve_guard.rs`
- `tethers-0.1/host-rust/src/resolve_outcome.rs`
- `tethers-0.1/host-rust/src/resolve_transport.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/tests/resolve_r0_p4_conformance.rs`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P3_LIVE_TRANSPORT.md`
- `docs/worker-notes/2026-09-15-tethers-resolve-p3-live-transport.md`

Do not broaden the packet checker to repository-wide inference.
