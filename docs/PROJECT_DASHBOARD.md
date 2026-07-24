# Tethers Project Dashboard

Updated: 2026-07-24

## Current milestone

First vertical runtime slice around the verified manifest store and capability
bridge.

Verified implementation checkpoint: `9ed81b8335c20e6287925b0341dc16da86780508`

Review and planning checkpoint on `main`: `539dc4c908518342f965137ba25dac5f4109b55e`

## Completed gate

- Task: planner-to-dispatch manifest pin correction (J02)
- Verdict: `SIGNED OFF`
- Risk: Red
- Packet: `docs/CURRENT_CLINE_TASK.md`
- Independent review: 2026-07-24
- Legacy worker note: absent by the documented pre-control-loop exception; no
  retrospective note was invented.

The reviewed evidence establishes all of the following:

- approved capability projection is injected before OCaml evaluation;
- bridge Actions copy the opaque digest unchanged from planner input;
- D1-versus-D2 stale plans fail closed before intent recording or executor
  invocation;
- manifest-major to planner-version mapping is explicit and strict;
- non-bridge capabilities and fixtures remain additive-compatible;
- full recorded verification passed: Rust `297 passed; 0 failed`, OCaml build,
  fixture/engine/transcript checks, host denial/failure checks, demo, and
  whitespace check.

## Last accepted result

The planner-to-dispatch manifest-pin milestone is accepted. The previous
configured stdio MCP provider-admission result at baseline `c93d746` remains
accepted.

## Matthew decision required

None. J04a is corrected, verified, and accepted; J05 has not been started.

## Cadence and drift

- Cadence: J00 project-control validation, J02 Red sign-off, J03 policy
  design, J03a's correction, J03b's scope-assessment boundary, and J04 review
  are complete. J04 was rejected on two bounded fail-closed defects; J04a
  corrected both and is accepted. J05 remains unauthorised.
- Cost: reserve Codex for later Red gates, not routine Amber/Green work.
- Process: the legacy milestone is accepted without a worker note only because
  it predates the control loop. Every new task requires evidence and a worker
  note before completion.
- Risk: the manifest-pin chain and the corrected J03/J03a policy contract are
  now the accepted baseline for policy implementation; do not reopen either
  without a demonstrated defect.

## Last signed-off design gate

- Task: J03a one-shot approval correction
- Verdict: `SIGNED OFF`
- State: `ACCEPTED`
- Risk: Red design
- Owner: Lucy/Codex
- Packet history: superseded by the J04 packet below in
  `docs/CURRENT_CLINE_TASK.md`
- Worker note: `docs/worker-notes/2026-07-24-j03a-one-shot-approval-correction.md`
- Review: Codex controller review, 2026-07-24

The corrected contract freezes a default-deny, exact name/version and
scope-aware policy resolver; fail-closed binding precedence; a canonical
one-shot Ask proof that satisfies only its matching mandatory confirmation;
approval invalidation and consumption; and no standard result Anchor for any
unattempted Action.

J03b additionally freezes the scope-assessment boundary: a trusted host/binding
assessor reports `within_scope`, `scope_violation`, or
`scope_not_established`; J04 combines that input and never guesses resource
arguments from the Plan. Missing structured-scope evidence denies safely.

## Current implementation gate

- Task: J04 effective policy resolution
- State: `REJECTED`
- Risk: Amber
- Owner: Copilot (isolated worktree)
- Packet: `docs/CURRENT_CLINE_TASK.md`
- Worker note: `docs/worker-notes/2026-07-24-j04-effective-policy-resolution.md`

Independent review found two contract failures despite otherwise clean test
evidence: J04 does not compare a non-empty Plan digest to the live verified
digest, and the demo supplies `WithinScope` even though it expressly has no
scope assessor. Both can let a structured-scope Action reach Allow. The review
record is `docs/worker-notes/2026-07-24-j04-codex-review.md`.

## Next route

J04a is accepted. Stop here; J05 and all further dispatch/approval work remain
unauthorised until a separate Red approval/resume design packet is compiled.

## Accepted correction

- Task: J04a effective-policy fail-closed correction
- State: `ACCEPTED`
- Risk: Amber
- Owner: Codex
- Worker note: `docs/worker-notes/2026-07-24-j04a-effective-policy-correction.md`

J04a compares every non-empty Action digest to the current verified digest and
returns `unavailable` for a mismatch. It also removes the demo's unsupported
`WithinScope` assertion: without a binding-specific assessor, structured scope
denies before durable intent, executor invocation, or a result Anchor.
