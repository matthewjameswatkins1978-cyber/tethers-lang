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

None. The next task is a bounded Red design decision, not a product-direction
decision.

## Next route

J03 — freeze the four-outcome policy contract (`allow`, `ask`, `deny`,
`unavailable`). Lucy/Codex designs and records the contract; no implementation
increment starts until that design is accepted and a fresh live task packet is
compiled.

## Cadence and drift

- Cadence: J00 project-control validation and J02 Red sign-off are complete.
  The programme advances to J03; J04 is not yet authorised.
- Cost: reserve Codex for J03 and later Red gates, not routine Green work.
- Process: the legacy milestone is accepted without a worker note only because
  it predates the control loop. Every new task requires evidence and a worker
  note before completion.
- Risk: the manifest-pin chain is now the accepted baseline for later policy,
  dispatch, and result-Anchor work; do not reopen it without a demonstrated
  defect.
