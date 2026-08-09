# F8-WORKFLOW — Worker Formatting and Publication Defaults

Task: `F8-WORKFLOW — Worker Formatting and Publication Defaults`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `bfb47ced813d8ec227f8828bbf66c7ecd1110d2f`

Implementation checkpoint: `30b26d1959138176dbf1481b267adc1791f0bc09`

## Requested outcome

Make the standard task packet and authoritative worker guidance automatically
distinguish Rust-changing work from non-Rust work, and make normal branch push
and remote-head reporting part of every completed task. Preserve the existing
implementation-checkpoint then documentation-only closeout sequence.

## Changes made

- `AGENTS.md` — added the formatter split, immediate rustfmt-diff stop rule,
  mandatory normal push, remote-SHA equality check, and report fields.
- `docs/PROJECT_CONTROL.md` and `docs/AGENT_WORKFLOW.md` — made normal push and
  remote equality part of completion and retained checkpoint/closeout ordering.
- `docs/TASK_PACKET_TEMPLATE.md` — added Rust change classification, formatting
  rules, and default publication/reporting requirements.
- `docs/WORKER_NOTE_TEMPLATE.md`, `docs/CLINE_HANDOFF.md`, and
  `docs/working-guides/DEEPSEEK_PRO_OPENCODE_JOB_PLAYBOOK.md` — aligned worker
  and handoff instructions with the same rules.
- `docs/CURRENT_CLINE_TASK.md` — recorded this bounded documentation-only task.

## Decisions and assumptions

- A Rust-changing packet must name its authorised Rust paths and formatter
  command; rustfmt output outside those paths stops the task rather than
  importing pre-existing formatting debt.
- A non-Rust or evidence-only packet uses formatter check mode only and never
  changes Rust source.
- The required normal push applies only to `COMPLETE` branches. It never
  authorises force-push, merge, rebase, direct `main` update, or a pull request.

## Evidence

- `pwsh -NoProfile -File scripts/check-dev-tools.ps1` — PASS; required tools
  resolved.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` before
  the implementation checkpoint — PASS (`control-v1/IN_PROGRESS`, base
  `bfb47ce`, HEAD `bfb47ce`).
- `git diff --cached --check` before the implementation checkpoint — PASS.
- Implementation checkpoint `30b26d1959138176dbf1481b267adc1791f0bc09` —
  documentation/template paths only.
- `cargo fmt --all -- --check` in `tethers-0.1/host-rust` — FAIL, unchanged
  pre-existing formatting-only diff at `src/replay_windows.rs:3277`. Per this
  non-Rust packet, no mutating formatter was run and no Rust source changed.
- `git diff --check 30b26d1^..30b26d1` — PASS.
- `git diff --name-status bfb47ced813d8ec227f8828bbf66c7ecd1110d2f..30b26d1`
  — only the authorised documentation/template paths.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` at the
  implementation checkpoint — PASS (`control-v1/IN_PROGRESS`, base `bfb47ce`,
  HEAD `30b26d1`).

The final closeout commit and normal push are recorded in the completion report.

## Publication evidence

Branch to push: `foundation/f8-worker-lifecycle`.

The final completion report resolves the full remote HEAD SHA after the normal
push, confirms local `HEAD == remote HEAD`, and records clean
`git status --short --branch` output.

## Discoveries

The existing `replay_windows.rs:3277` rustfmt failure remains on the separate
`foundation/f8-fmt` branch. This task neither incorporated nor changed it.

## Remaining risks

The separate F8-FMT task remains responsible for its Rust formatting repair and
its own completion evidence. No risk within this packet's documentation scope
remains unresolved.

## Smallest next action

Independently review the pushed `foundation/f8-worker-lifecycle` documentation
diff, then decide whether to accept it. Do not start F8 warning cleanup from
this task.

## References

- Base: `bfb47ced813d8ec227f8828bbf66c7ecd1110d2f`
- Implementation checkpoint: `30b26d1959138176dbf1481b267adc1791f0bc09`
- Task packet: `docs/CURRENT_CLINE_TASK.md`
