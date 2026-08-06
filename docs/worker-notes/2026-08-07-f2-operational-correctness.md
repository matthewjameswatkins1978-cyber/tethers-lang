# Worker Note

Task: `F2 - Operational correctness defects`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `IN_PROGRESS`

Base commit: `f295daa288f4d3dc48181888d6655df798675033`

Implementation checkpoint: `WORKTREE`

## Checkpoints

| Checkpoint | SHA | Purpose |
|---|---|---|
| F2 packet | (pending) | Task definition and documentation |
| F2a regression | (pending) | Failing live-stderr regression test |
| F2a repair | (pending) | Stderr capture fix and passing tests |
| F2b characterisation | (pending) | M3 handle allow-list nondeterminism evidence |
| F2b repair | (pending) | Handle test fix, if directly proven |
| Final documentation | (pending) | Worker note completion |

## Requested outcome

Repair two F1-confirmed operational defects: truthful live stderr capture in
`child_process.rs` (F2a) and nondeterministic M3 handle allow-list test
behaviour (F2b). Preserve all public contracts.

## Changes made

See checkpoint sections below.

## F2a: Live stderr and child cleanup

### Defect analysis

(stderr capture thread details to be filled as work progresses)

### Regression test

(F2a regression checkpoint to be filled)

### Repair

(F2a repair checkpoint to be filled)

## F2b: M3 handle allow-list

### Characterisation

(M3 handle test characterisation results to be filled)

### Root cause

(to be determined)

### Repair

(F2b repair to be filled or BLOCKED)

## Decisions and assumptions

(to be filled as work progresses)

## Evidence

### Final verification matrix

Run after final code/test change. See externally reported results.

| # | Command | Result | Notes |
|---|---|---|---|
| 1 | `git fetch origin --prune` | (pending) | |
| 2 | `git rev-parse origin/main` | (pending) | |
| 3 | `git rev-parse HEAD` | (pending) | |
| 4 | `git status --short --branch` | (pending) | |
| 5 | `rustup show` | (pending) | |
| 6 | `cargo --version` | (pending) | |
| 7 | `cargo fmt --all -- --check` | (pending) | |
| 8 | `cargo check --all-targets --all-features --locked` | (pending) | |
| 9 | `cargo test --all-targets --all-features --locked` | (pending) | |
| 10 | `cargo clippy --all-targets --all-features --locked -- -W clippy::all` | (pending) | |
| 11 | `just verify` | (pending) | |
| 12 | `just verify-agent` | (pending) | |
| 13 | `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` | (pending) | |
| 14 | `git diff --exit-code origin/main...HEAD -- docs/foundation-pass/fixtures` | (pending) | |
| 15 | `git diff --check origin/main...HEAD` | (pending) | |
| 16 | `git diff --name-only origin/main...HEAD` | (pending) | |
| 17 | `git status --short --branch` | (pending) | |

## Discoveries

(to be filled as work progresses)

## Remaining risks

(to be filled as work progresses)

## Smallest next action

Execute F2a: create the failing-regression checkpoint for live stderr visibility.

## References

- Base: `f295daa288f4d3dc48181888d6655df798675033` (`origin/main`)
- F1 worker note: `docs/worker-notes/2026-08-06-f1-baseline.md`
- Debt ledger: `docs/foundation-pass/DEBT_LEDGER.md`
- Task packet: `docs/CURRENT_CLINE_TASK.md`
