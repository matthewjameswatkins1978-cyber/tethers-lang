# GitHub Copilot Trial

## Goal

Determine whether Copilot reduces total implementation plus architecture-review
effort per accepted Tethers change without weakening scope, correctness, or
trust boundaries.

Do not create make-work for the trial. Select real queued tasks when they match
the categories below.

## Sequence

1. **Repository comprehension:** explain a completed moderately complex
   component without changing files. First candidate: executor output
   validation through known-outcome Result Anchor emission.
2. **Bounded feature:** implement an accepted two-or-three-file change following
   an existing pattern.
3. **Fiddly implementation:** implement one contained Rust or OCaml mechanism
   involving ownership, traits, error propagation, parsing, or process state.
4. **Bug repair:** reproduce a real failure, diagnose it, make the smallest
   correction, add a regression test, and verify it.
5. **Autonomous background task:** complete an accepted Amber task in an
   isolated worktree and return a full evidence report.

The configured local provider binding crosses a capability trust boundary. Use
the Tethers Explorer to inspect and plan it, but do not use it as an autonomous
implementation trial until its design and invariants have been reviewed.

## Evidence log

Add one row only after a real task is accepted or rejected.

| Date | Task | Colour | Agent/model | Worktree | First pass | Correction loops | Verification complete | Architecture review | Usage | Outcome | Notes |
| --- | --- | --- | --- | --- | --- | ---: | --- | --- | --- | --- | --- |
| 2026-07-23 | Host-owned stdio provider binding fixture | Green | Cline + Codex | main | no | 1 | yes | required (capability-trust boundary), Codex milestone sign-off complete | unavailable | corrected | Initial implementation was corrected after Codex review; final signed-off evidence: 21 focused provider tests, 283/283 Rust tests, 15 MCP transcript cases, fixture checks, host denial, host execution failure, demo, OCaml build, fmt, and git diff --check all passed. |

Use `accepted`, `corrected`, or `rejected` for Outcome. Record usage from the
Copilot status dashboard when available; do not estimate missing numbers.

## Review points

- After five representative tasks, identify immediate safety or routing
  failures.
- After approximately ten real tasks or one month, compare Copilot with the
  previous Cline/DeepSeek plus architecture-review workflow.
- If Copilot becomes reliable on a class of Green tasks, sample rather than
  automatically reviewing every change.
- Continue reviewing every Red change and any task that crosses a trust,
  permission, persistence, compatibility, or determinism boundary.

## Codex milestone cadence

Cadence baseline: `bc4e077` — Result Anchor checkpoint and agent-workflow
transition starting point. This is a counting marker, not a fresh technical
sign-off.

Ask for a Codex milestone review:

- after three `accepted` or `corrected` evidence rows following this baseline;
- before the next meaningful push or release checkpoint;
- immediately for any Red task, agent disagreement, or unresolved validation.

Only Codex may reset the cadence baseline after completing a milestone review.

When Copilot detects a milestone gate, it must explicitly tell Matthew to ask
Codex for sign-off and stop task continuation. Copilot must not treat its own
review, a green test suite, or Cline's report as milestone sign-off.
