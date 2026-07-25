Task: `J05 exact one-shot Ask approval and resume`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `d7962642fc85a433a7d4257de73a9f2417f4418f`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Implement host-owned one-shot approval for an exact fresh `Ask`, with complete
proof binding, truthful authorisation Trail records, atomic consumption before
intent, and no change to planner or existing structured-scope demo semantics.

## Changes made

- Added `host-rust/src/approval.rs`: RFC 8785/JCS proof digests, all six record
  states, deterministic in-memory approval identities, exact comparison,
  invalidation, and one-shot consume.
- Added the host production orchestration seam in `main.rs`. It reruns the
  current effective-policy inputs internally, invalidates live records on fresh
  non-Ask gates or proof drift, and obtains dispatch permission only through
  `policy::allow_after_exact_approval` after consumption.
- Added a dedicated unrestricted, mandatory-confirmation test fixture. It is
  test-only and leaves the existing structured-scope demo fail-closed.
- Added durable authorisation Trail entry support and test error injection.

## Decisions and assumptions

Approval identities are process-local monotonic host identities; a fresh store
on restart contains no approval records. Terminal records are never reused as
pending: a later request receives a new identity. Credentials are absent from
proofs and Trail records; only the canonical argument digest is retained.

## Evidence

- `cargo fmt --check` and `cargo test`: 323 passed, 0 failed after the J05
  focused tests were added.
- `pwsh -NoProfile -File scripts/check-fixtures.ps1`, `test-engine.ps1`,
  `test-mcp-transcripts.ps1`, `test-host-denial.ps1`,
  `test-host-execution-failure.ps1`, and `demo.ps1`: passed.
- `opam exec -- dune build`: passed.
- J05 focused tests cover pending request de-duplication, approval, consume
  before intent, replay prevention, fresh Deny invalidation, terminal human
  denial/cancellation, deterministic proof construction, and terminal-state
  separation. Existing policy/dispatch tests cover fresh unavailable, schema,
  scope, no-intent, no-provider-call, and no-result-anchor fail-closed paths.

## Discoveries

The ordinary demonstration host deliberately returns `ScopeNotEstablished` for
its structured manifest. J05 correctly preserves that denial; the dedicated
unrestricted fixture reaches Ask through the manifest's mandatory confirmation.

## Remaining risks

Approval records are intentionally not durable across restart, and the demo has
no GUI or remote human-decision endpoint; both are explicitly deferred by J05.

## Smallest next action

Independent Red review of the complete J05 diff and evidence; do not begin J06.

## References

- `docs/J05_EXACT_ASK_APPROVAL_DESIGN.md`
- `tethers-0.1/host-rust/src/approval.rs`
- `tethers-0.1/host-rust/src/main.rs`
- `safety/preserve-local-main-20260725` (reference only; not merged or changed)
