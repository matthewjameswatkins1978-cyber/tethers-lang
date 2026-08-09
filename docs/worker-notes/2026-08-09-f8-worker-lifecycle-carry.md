# Worker Note — F8-WORKFLOW-CARRY

Task: `F8-WORKFLOW-CARRY — Worker Lifecycle Documentation Carry`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `5e5ec4f6f8afd8aa06ed49569038dd80c8d18940`
Implementation checkpoint: `106fb3239a8868c8417d62d3ed5529e602472986`

## Requested outcome

Carry 7 already-reviewed worker lifecycle documents from the accepted commit
`30b26d` onto the current F8-FMT tip at `5e5ec4f`.

## Changes made

1. Created branch `foundation/f8-worker-lifecycle-carry` from base `5e5ec4f`.
2. Copied exactly 7 files from source commit `30b26d1959138176dbf1481b267adc1791f0bc09` using `git checkout`.
3. Wrote a fresh task packet for this carry-forward job.
4. Wrote this worker note.

Did NOT copy `docs/CURRENT_CLINE_TASK.md` from the source commit.
Did NOT copy any old worker notes.
Zero Rust/source/test/build/warning changes.

## Decisions and assumptions

- The 7 documents are accepted as-is. Zero redesign applied.
- This is a `NON_RUST` task since no source or test code is touched.

## Evidence

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `git diff --check` | PASS |
| Packet checker | PASS |
| Diff from base | 7 guidance files + task packet + worker note only |
| `git status --short` | Clean |

## Discoveries

None. This is a straightforward `git checkout` carry with no surprises.

## Remaining risks

None. Documentation-only carry with zero semantic or code change.

## Smallest next action

Proceed to F8-T1 (test-only warning cleanup) on the tip of this branch or a
sibling branch from the same base `5e5ec4f`.

## References

- Task packet: `docs/CURRENT_CLINE_TASK.md`
- Source commit: `30b26d1959138176dbf1481b267adc1791f0bc09`
- Base: `5e5ec4f6f8afd8aa06ed49569038dd80c8d18940`
- Implementation checkpoint: `106fb3239a8868c8417d62d3ed5529e602472986`
- Branch: `foundation/f8-worker-lifecycle-carry`
