# Current Implementation Task

Control contract: `1`
Task packet: `F8-WORKFLOW-CARRY — Worker Lifecycle Documentation Carry`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode copies accepted lifecycle docs onto current F8 tip`
Worker note: `docs/worker-notes/2026-08-09-f8-worker-lifecycle-carry.md`
Base branch: `foundation/f8-fmt`
Base commit: `5e5ec4f6f8afd8aa06ed49569038dd80c8d18940`
Implementation branch: `foundation/f8-worker-lifecycle-carry`
Implementation checkpoint: `106fb3239a8868c8417d62d3ed5529e602472986`
Source commit: `30b26d1959138176dbf1481b267adc1791f0bc09`

## Objective

Carry the already-reviewed worker formatting/publication rules from commit
`30b26d1959138176dbf1481b267adc1791f0bc09` onto the current accepted
F8-FMT lineage at `5e5ec4f6f8afd8aa06ed49569038dd80c8d18940`.

## Relevant background and existing behaviour

The 7 guidance/template files were reviewed and accepted in commit `30b26d`. The F8-FMT
tip at `5e5ec4f` needs them carried forward to ensure task packets, worker notes, and
agent workflows follow the updated lifecycle rules. No redesign is needed.

## Required behaviour

1. Copy exactly these 7 files from source commit `30b26d`:
   - `AGENTS.md`
   - `docs/PROJECT_CONTROL.md`
   - `docs/AGENT_WORKFLOW.md`
   - `docs/TASK_PACKET_TEMPLATE.md`
   - `docs/WORKER_NOTE_TEMPLATE.md`
   - `docs/CLINE_HANDOFF.md`
   - `docs/working-guides/DEEPSEEK_PRO_OPENCODE_JOB_PLAYBOOK.md`
2. Do NOT copy `docs/CURRENT_CLINE_TASK.md` from the source commit.
3. Do NOT copy the old lifecycle worker note.
4. Create a fresh task packet and worker note.
5. No Rust/source/test/build/warning changes.
6. Do not redesign the documents.

## Frozen decisions and invariants

- The 7 documents are accepted as-is from the source commit.
- Zero redesign, rewording, or editorial changes.
- No Rust or test file changes.
- The carry is documentation-only.

## Acceptance criteria

1. Diff from base contains only the 7 guidance files + CURRENT_CLINE_TASK.md + worker note
2. `cargo fmt --all -- --check` passes
3. `git diff --check` passes
4. Packet checker passes
5. Clean git status
6. No Rust/source/test/build/warning changes in diff

## Required verification

- `cargo fmt --all -- --check`: PASS
- `git diff --check`: PASS
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`: PASS

## Relevant components

### DOCUMENTATION
- `AGENTS.md`
- `docs/PROJECT_CONTROL.md`
- `docs/AGENT_WORKFLOW.md`
- `docs/TASK_PACKET_TEMPLATE.md`
- `docs/WORKER_NOTE_TEMPLATE.md`
- `docs/CLINE_HANDOFF.md`
- `docs/working-guides/DEEPSEEK_PRO_OPENCODE_JOB_PLAYBOOK.md`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-worker-lifecycle-carry.md`

## Forbidden changes

- No `docs/CURRENT_CLINE_TASK.md` from source commit
- No old lifecycle worker note
- No Rust/source/test/build/warning changes
- No redesign of the 7 documents
- No F8-T1 work

## Stop conditions

STOP if any file not in the authorised set appears in the diff.
STOP if `cargo fmt --check` fails.
STOP if packet checker fails.

## Expected pre-existing changes

None.
