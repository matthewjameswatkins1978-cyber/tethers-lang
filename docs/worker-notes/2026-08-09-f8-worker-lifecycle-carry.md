# Worker Note — F8-WORKFLOW-CARRY

Task: `F8-WORKFLOW-CARRY — Worker Lifecycle Documentation Carry`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `5e5ec4f6f8afd8aa06ed49569038dd80c8d18940`
Source commit: `30b26d1959138176dbf1481b267adc1791f0bc09`
Implementation checkpoint: `106fb3239a8868c8417d62d3ed5529e602472986`
Implementation branch: `foundation/f8-worker-lifecycle-carry`

## Requested outcome

Carry 7 already-reviewed worker lifecycle documents from the accepted commit
`30b26d` onto the current F8-FMT tip at `5e5ec4f`.

## Changes made

1. Created branch `foundation/f8-worker-lifecycle-carry` from base `5e5ec4f`.
2. Copied exactly 7 files from source commit `30b26d`:
   - `AGENTS.md`
   - `docs/PROJECT_CONTROL.md`
   - `docs/AGENT_WORKFLOW.md`
   - `docs/TASK_PACKET_TEMPLATE.md`
   - `docs/WORKER_NOTE_TEMPLATE.md`
   - `docs/CLINE_HANDOFF.md`
   - `docs/working-guides/DEEPSEEK_PRO_OPENCODE_JOB_PLAYBOOK.md`
3. Wrote a fresh task packet (`docs/CURRENT_CLINE_TASK.md`).
4. Wrote this worker note.
5. Committed implementation checkpoint at `106fb32`.

Did NOT copy `docs/CURRENT_CLINE_TASK.md` from the source commit.
Did NOT copy any old worker notes.
Zero Rust/source/test/build/warning changes.

## Evidence

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `git diff --check` | PASS |
| Packet checker | PASS |
| Diff from base (8 files) | Only the 7 guidance docs + task packet |
| `git status --short` | Clean |

## Remaining risks

None. This is a documentation-only carry with zero semantic change.

## References

- Task packet: `docs/CURRENT_CLINE_TASK.md`
- Source: `30b26d1959138176dbf1481b267adc1791f0bc09`
- Base: `5e5ec4f6f8afd8aa06ed49569038dd80c8d18940`
- Implementation checkpoint: `106fb3239a8868c8417d62d3ed5529e602472986`
- Branch: `foundation/f8-worker-lifecycle-carry`
