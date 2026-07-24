# Worker Note

Task: `J04a effective-policy fail-closed correction`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `643c6ed40e3e8a167afd53eca2c98597c0aa8f24`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Correct J04's stale-manifest and unassessed-structured-scope fail-open paths
without changing the frozen four-outcome contract, scope mapping boundary, or
approval work reserved for J05.

## Changes made

- `policy.rs` now compares the required Action digest to the resolved verified
  digest and returns `Unavailable` with `ManifestDigestMismatch` on mismatch.
  A focused regression proves a non-empty stale digest cannot bypass exact
  local Allow.
- `main.rs` supplies `ScopeNotEstablished` for the demo's structured scope,
  because no binding-specific assessor exists. It therefore denies before
  intent preparation or executor invocation.
- The demo and host execution-failure PowerShell checks now prove that this
  unassessed path produces no durable intent/outcome, executor call, or result
  Anchor. Existing Rust tests retain direct authorised executor-success and
  executor-failure coverage.

## Decisions and assumptions

- Digest mismatch is a binding fact and therefore returns `Unavailable`, as
  J03 specifies; it is not recast as a host-policy Deny.
- No `project`-to-`path_prefix` mapping was invented. The reference host
  treats absent binding-specific scope proof as `ScopeNotEstablished`.
- The existing engine-response and dispatch boundaries remain unchanged.

## Evidence

- `cargo fmt --check`: passed.
- `cargo test`: `317 passed; 0 failed`.
- `scripts/check-fixtures.ps1`: passed, `46 JSON files, 30 JSONL files`.
- `scripts/test-engine.ps1`: passed all fixture cases and deterministic repeat.
- `scripts/test-mcp-transcripts.ps1`: passed, `15 cases`.
- `scripts/test-host-denial.ps1`: passed.
- `scripts/test-host-execution-failure.ps1`: passed; the failing executor is
  blocked by the scope gate with `execution_status: denied`.
- `scripts/demo.ps1`: passed; unassessed structured scope is denied before
  execution with no Result Anchor.
- `opam exec -- dune build`: passed.
- Packet consistency and `git diff --check`: passed before commit.

## Discoveries

The old end-to-end successful demo depended on an explicit comment admitting
that its `WithinScope` value was only a placeholder. It was not a valid proof
under J03b, so a completion or executor-failure process test cannot safely run
through that demo binding until a later binding-specific assessor is designed.

## Remaining risks

J04 now provides the policy boundary, but concrete extraction for path,
repository, or calendar scopes remains intentionally deferred. A future
binding/adapter task must implement one before a structured-scope capability
can be automatically allowed by the reference host.

## Smallest next action

Stop. J05 remains unauthorised; its one-shot approval/resume design requires a
separate Red packet and review.

## References

- `docs/DECISIONS.md` — J03, J03a, and J03b
- `docs/worker-notes/2026-07-24-j04-codex-review.md`
- `tethers-0.1/host-rust/src/policy.rs`
- `tethers-0.1/host-rust/src/main.rs`
- `tethers-0.1/scripts/demo.ps1`
- `tethers-0.1/scripts/test-host-execution-failure.ps1`
