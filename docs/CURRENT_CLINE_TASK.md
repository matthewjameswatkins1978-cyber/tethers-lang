# Current Implementation Task

Control contract: `1`

Task: `J14A-RV -- run missing regressions and correct final evidence`

Owner: `OpenCode`

Status: `COMPLETE`

Task colour: `Green`

Route: `OpenCode verification-only closeout — Lucy independent review`

Base commit: `0c64b48d860ce2178858c4c5d8a0af38708bc7cc`

Branch: `opencode/j14a-closeout-bootstrap`

Candidate under verification: `735988b9ef212c18cdc33cc233fc9be99287219c`

Worker note: `docs/worker-notes/2026-07-30-j14a-complete-local-scenario.md`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

## Objective

Finish J14A-R verification honestly. Run the five regressions omitted from
J14A-R. Consolidate duplicated worker-note commit evidence. No production code
change.

## Relevant background and existing behaviour

The J14A implementation and J14A-R engineering repair are accepted and pushed.
The five regressions (host denial, execution failure, result follow-up, engine,
demo) were not verified in the J14A-R repair task because the OCaml switch
resides in a separate worktree. The worker note metadata contains duplicated
fields and stale implementation-checkpoint labels from the evidence closeout.

## Required behaviour

1. Complete the mandatory startup gate.
2. Run test-host-denial, test-host-execution-failure, test-host-result-follow-up,
   test-engine, and demo regressions using the accepted process-local external
   OCaml switch.
3. Consolidate the worker note metadata: one block at the top, no duplicated
   fields, each implementation SHA exactly once in the entire file.
4. Record exact regression results in the worker note.

## Relevant components

- `docs/CURRENT_CLINE_TASK.md` — this task packet
- `docs/worker-notes/2026-07-30-j14a-complete-local-scenario.md` — evidence note

## Expected pre-existing changes

None at the verification start. Branch is 8 commits ahead of origin/main with
J14A implementation, J14A-R repair, evidence closeout, and OpenCode bootstrap.

## Frozen decisions and invariants

No production code change. No OCaml, Rust, PowerShell, or scenario mutation.

## Acceptance criteria

1. All five regressions PASS.
2. Worker note has one metadata block with no duplicated fields.
3. Each implementation SHA occurs exactly once in the worker note.
4. Packet checker passes with COMPLETE status.
5. Only the two authorised files change.
6. Git clean, whitespace clean, origin/main unchanged.

## Forbidden changes

No Rust, PowerShell, OCaml, scenarios, opencode.json, AGENTS.md, control docs,
DECISIONS.md, Cargo files, manifests, or fixtures.

## Stop conditions

Return BLOCKED when: pre-flight mismatch, worktree dirty, regression failure
after two materially similar attempts, non-authorised file changed, or
origin/main differs.

## Required verification

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File .\.github\scripts\check-tethers-task-packet.ps1
git diff --check
git diff --stat
git diff
git status --porcelain=v1 --untracked-files=all
```

SHA occurrence counts must be exactly 1 each.
