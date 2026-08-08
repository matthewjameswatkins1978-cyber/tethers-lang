# Worker Note

Task: `Control — Permanent Worker-Evidence Hardening`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `a9c2862adfd3bca5c7c253609c397ad9a59c5ac8`

Implementation checkpoint: `e4f7f098e89053fa2fb4f61b1a0b94147b22ebe4`

## Requested outcome

Fix the demonstrated process defect where COMPLETE tasks could pass the packet checker with `WORKTREE` as the implementation checkpoint, and the workflow permitted worker notes to be written before a committed implementation checkpoint existed.

## Changes made

- `.github/scripts/check-tethers-task-packet.ps1` — Added COMPLETE/ACCEPTED/REJECTED enforcement: WORKTREE rejected, implementation checkpoint must be a real commit, base must be ancestor of checkpoint, checkpoint must be ancestor of HEAD, only closeout paths (packet, worker note, PROJECT_DASHBOARD.md) may differ after checkpoint.
- `.github/scripts/test-check-tethers-task-packet.ps1` — NEW: 7 independent tests (A-G) using isolated temp Git repositories, covering WORKTREE rejection, nonexistent SHA rejection, ancestry validation, production-after-checkpoint rejection, arbitrary-doc-after-checkpoint rejection, and valid closeout-only acceptance.
- `docs/AGENT_WORKFLOW.md` — Normal Work Sequence rewritten: implement, commit checkpoint, verify against checkpoint, write worker note from final results, commit closeout docs only. Added explicit rules against writing COMPLETE evidence before the checkpoint exists and using stale check results.
- `docs/PROJECT_CONTROL.md` — COMPLETE definition tightened: requires committed implementation checkpoint, verification against that checkpoint, worker note recording actual results, only closeout docs after checkpoint. Added BLOCKED/COMPLETE WORKTREE distinction.
- `docs/WORKER_NOTE_TEMPLATE.md` — Updated checkpoint instructions: full SHA required for COMPLETE; WORKTREE allowed only for BLOCKED. Added Evidence-section instructions: commit before recording, run verification against committed checkpoint, do not carry forward stale results.

## Decisions and assumptions

- `docs/PROJECT_DASHBOARD.md` is included as a closeout-allowed path alongside the packet and worker note.
- BLOCKED + WORKTREE remains legal for intentionally uncommitted evidence.
- The checker reads the worker note anew for the checkpoint field to avoid stale variable scope issues.
- No Rust or OCaml build/test dependencies required for control-only changes.

## Evidence

Verification run against committed implementation checkpoint `e4f7f098e89053fa2fb4f61b1a0b94147b22ebe4`:

- `pwsh -NoProfile -File .github/scripts/test-check-tethers-task-packet.ps1` — 7/7 passed (A: COMPLETE+WORKTREE rejected, B: BLOCKED+WORKTREE allowed, C: nonexistent SHA rejected, D: valid checkpoint+closeout-only passes, E: production-after-checkpoint rejected, F: arbitrary-doc-after-checkpoint rejected, G: packet+worker-note closeout passes)
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — PASS (control-v1/IN_PROGRESS at checkpoint; destined for COMPLETE after closeout docs)
- `git diff --check` — PASS (whitespace clean)
- No Rust or OCaml product code changed

## Discoveries

None.

## Remaining risks

None known within packet scope.

## Smallest next action

Lucy review and acceptance of the control hardening. DO NOT BEGIN F4b or F5.

## References

- `.github/scripts/check-tethers-task-packet.ps1` — checker
- `.github/scripts/test-check-tethers-task-packet.ps1` — test suite
- `docs/AGENT_WORKFLOW.md` — workflow
- `docs/PROJECT_CONTROL.md` — project control
- `docs/WORKER_NOTE_TEMPLATE.md` — template
- `docs/CURRENT_CLINE_TASK.md` — this control task packet
- Branch: `foundation/control-worker-evidence-finalization`
- Base: `a9c2862adfd3bca5c7c253609c397ad9a59c5ac8`
