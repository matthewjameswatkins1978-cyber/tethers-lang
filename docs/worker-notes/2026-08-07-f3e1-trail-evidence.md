# Worker Note

Task: `F3e1 - Trail evidence harvest`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `DeepSeek Pro HIGH`

Status: `COMPLETE`

Base commit: `c9332bab072ce273db3aecc367faf64be71a8586`

Implementation checkpoint: `fb07c607a5c938d326489a03a7e1b474d6e88461`

## Requested outcome

Audit Trail/FileTrail only as the append-only causal-log persistence store.
Map every evidence dimension to PROVEN/DISPROVEN/UNVERIFIED with exact test
citations and hard assertions. Close remaining F3b UNVERIFIED gaps with
characterization tests. Do not change production code.

## Changes made

- Added 3 characterization tests (no production changes):
  1. `f3e1_truncated_final_line_maps_to_audit_failed` (trail_command.rs) —
     production reader classifies truncated final line as TRAIL_INVALID
     (fail-closed). Was F3b UNVERIFIED.
  2. `f3e1_file_trail_open_has_no_path_validation` (dispatch.rs) —
     records that FileTrail::open() has no root/reparse/chain validation.
  3. `f3e1_file_trail_open_accepts_relative_path` (dispatch.rs) —
     proves FileTrail::open() accepts relative paths without validation.
     Path validation inside FileTrail::open is DISPROVEN.

- Updated `docs/CURRENT_CLINE_TASK.md` to describe F3e1 (owner, base, scope,
  findings, acceptance criteria).
- Updated `docs/foundation-pass/PERSISTENCE_INVENTORY.md` with F3e1 Trail
  evidence section including the full evidence summary table.
- Created this worker note.

## Decisions and assumptions

- Trail is append-only causal log — no conversion to atomic records.
- No production code redesign. Tests only.
- F3b UNVERIFIED platform properties preserved (never upgrade).

## Evidence

| # | Property | Status | Test |
|---|---|---|---|
| 1 | Append order | PROVEN | `trail_multiple_complete_lines_ordered_and_parseable` |
| 2 | One JSONL record per write | PROVEN | `trail_complete_line_survives_close_and_reopen` |
| 3 | Flush/sync accepted | PROVEN (F3b) | F3b-1 primitive evidence |
| 4 | Close/reopen readback | PROVEN | 4 tests in dispatch.rs |
| 5 | Truncated line: raw bytes present and non-parseable | PROVEN (F3b) | `trail_truncated_final_line_present_and_non_parseable` |
| 6 | Production reader: truncated final line → TRAIL_INVALID | PROVEN | `f3e1_truncated_final_line_maps_to_audit_failed` (NEW) |
| 7 | Malformed complete line → TRAIL_INVALID | PROVEN | 6 `j13c_*_maps_to_audit_failed` tests |
| 8 | Fail-closed: later malformed → nothing returned | PROVEN | `j13c_malformed_later_prevents_all_output` |
| 9 | Execution_id filtering | PROVEN | 5 tests in trail_command.rs |
| 10 | FileTrail::open accepts relative paths | PROVEN | `f3e1_file_trail_open_accepts_relative_path` (NEW) |
| 11 | Path validation inside FileTrail::open | DISPROVEN | `f3e1_file_trail_open_accepts_relative_path` (NEW) |

## Discoveries

- Production reader fail-closes on truncated final line. Was F3b UNVERIFIED; now PROVEN.
- FileTrail::open() provides no root/reparse/chain/absolute-path validation; callers enforce path safety.
- No defect found.

## Remaining risks

- Power-loss durability: UNVERIFIED (F3b) — never upgrade
- Directory-entry durability: UNVERIFIED (F3b) — never upgrade
- Parent-directory flush in production: DISPROVEN (F3b)

## Smallest next action

Lucy reviews F3e1 evidence. F3e2 (Replay) is the next bounded subtask.

## References

- `docs/CURRENT_CLINE_TASK.md` — F3e1 task packet
- `docs/foundation-pass/PERSISTENCE_INVENTORY.md` — F3e1 Trail evidence section
- `docs/foundation-pass/DEBT_LEDGER.md` — unchanged
