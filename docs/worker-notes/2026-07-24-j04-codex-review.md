# J04 Codex Review

Task reviewed: `J04 effective policy resolution`

Review date: `2026-07-24`

Verdict: `CORRECTION REQUIRED` — not signed off.

## Evidence inspected

- Live base and HEAD: `643c6ed40e3e8a167afd53eca2c98597c0aa8f24`; all J04
  changes are uncommitted in the primary worktree.
- Complete J04 diff, current packet, worker note, J03/J03a/J03b decision,
  capability-bridge scope/binding contract, resolver, policy, validation, and
  host call path.
- Independent verification on 2026-07-24: `cargo fmt --check`; Rust `316
  passed; 0 failed`; fixture, engine, MCP-transcript, host-denial,
  host-execution-failure, and demo scripts; and `opam exec -- dune build`.
  Packet consistency and `git diff --check` also pass.

## Blocking defects

1. `evaluate_effective_policy()` validates only that the Plan digest is
   non-empty. It resolves by capability name/version/provider but never
   compares `ProposedAction.manifest_digest` to the resolved verified digest.
   A stale non-empty Plan digest can therefore receive `Allow`, contrary to
   J03's required `Unavailable` binding result.
2. The reference host declares structured `path_prefix` scope, states that no
   binding-specific assessor exists, then supplies `WithinScope` anyway. J03b
   classifies the absence of an assessor as `ScopeNotEstablished`, which must
   deny before local Allow or Ask. The passing completion demo currently
   exercises this unsupported assertion.

## Non-blocking process note

The implementation was made in the primary checkout rather than the packet's
isolated worktree. No overlapping work was observed. This is recorded as
process drift, not the basis for rejection.

## Required next action

J04a is compiled as a separate `PROPOSED` correction packet. It is limited to
verified digest comparison and fail-closed demo scope assessment; it does not
authorise J05.
