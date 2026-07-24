# Worker Note

Task: `J03 four-outcome policy contract`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Lucy/Codex`

Status: `COMPLETE`

Base commit: `643c6ed40e3e8a167afd53eca2c98597c0aa8f24`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Freeze the smallest fail-closed host policy contract for Allow, Ask, Deny and
Unavailable, including the exact one-shot Ask proof and a focused test matrix.
Do not implement runtime behaviour.

## Changes made

- Replaced the legacy completed packet with the completed control-v1 J03 design
  packet.
- Added the J03 policy contract and implementation test matrix to
  `docs/DECISIONS.md`.
- Updated the current goal, task queue and dashboard with the completed design
  state.

## Decisions and assumptions

- Host default is Deny; exact policy rules are name/version scoped and cannot
  bypass schema, scope, binding, or mandatory-confirmation checks.
- Ask is a one-shot host-only approval bound to Action IDs, canonical argument
  digest, manifest digest and provider identity; it is consumed before intent
  preparation and expires on restart.
- An unattempted Action never emits a standard result Anchor.

## Evidence

- Read the current `policy.rs`, `resolver.rs`, `dispatch.rs`, and execution
  boundary in `main.rs`.
- Read the Constitution, capability-bridge confirmation/handoff sections, and
  canonical architecture sections 4.3 through 4.8.
- Ran the packet checker, Git whitespace check, complete documentation diff
  inspection, and final Git status check.

## Discoveries

The existing host already models all four decisions and blocks non-Allow paths
before intent/executor use, but it lacks the complete policy inputs, exact Ask
proof, approval lifecycle, and Trail taxonomy frozen here.

## Remaining risks

J04/J05 must implement the contract without changing the frozen precedence or
approval binding. Durable replay protection is deferred to J09; therefore the
J03 approval record deliberately expires on restart.

## Smallest next action

Compile J04 as a fresh Amber implementation packet against this decision, with
one Copilot worktree owner and the focused matrix cases relevant to effective
policy resolution only.

## References

- `docs/ROAD_TO_0_2.md` J03 and J04
- `docs/DECISIONS.md` J03 Four-Outcome Host Policy Contract
- `tethers-0.1/host-rust/src/policy.rs`
- `tethers-0.1/host-rust/src/dispatch.rs`
