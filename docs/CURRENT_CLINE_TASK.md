# Current Implementation Task

Control contract: `1`
Task: `F3b - Windows persistence primitive evidence`
Owner: `OpenCode`
Model: `DeepSeek Pro`
Status: `IN_PROGRESS`
Task colour: `Red`
Route: `DeepSeek Pro performs the bounded Windows primitive evidence pass; Lucy independently reviews before F3c`
Worker note: `docs/worker-notes/2026-08-07-f3b-windows-persistence-evidence.md`
Base branch: `main`
Base commit: `145a791ceb3f5e3b8855aeadbac83671d9a2b363`
Implementation branch: `foundation/f3b-windows-persistence-evidence`
Parent branch: `main`
Parent tip: `145a791ceb3f5e3b8855aeadbac83671d9a2b363`
Preparation checkpoint: `145a791ceb3f5e3b8855aeadbac83671d9a2b363`
Implementation checkpoint: `145a791ceb3f5e3b8855aeadbac83671d9a2b363`
OCaml switch path: `N/A`
Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory
Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1`

## Objective

Establish direct Windows evidence for the persistence primitives identified
by F3a, without repairing or redesigning the persistence stores. Every
conclusion must distinguish the observed primitive, the directly tested
property, and the remaining uncertainty.

## Relevant background and existing behaviour

F3a at `145a791ceb3f5e3b8855aeadbac83671d9a2b363` classified 14 filesystem-backed
stores and identified gaps in Windows primitive evidence. The inventory marks
every atomic-visibility and directory-durability claim `UNVERIFIED (F3b)`.
The F3a route map records five specific question clusters:

- `sync_all()` + `fs::rename` durability for StoreRoot/Candidate/Local Anchor
- Parent-directory durability feasibility
- Replay `FlushFileBuffers` + `SetFileInformationByHandle` rename semantics
- JSONL line append interruption behaviour (Trail)
- Local Anchor root path reparse-point safety

The central rule for F3b: separate file-data durability, visibility of the
final filename, atomic visibility during rename, persistence of the directory
entry, behaviour after process interruption, behaviour after simulated
incomplete writes, and unsafe-path / reparse-point defence. Do not treat
evidence for one property as evidence for another.

## Required behaviour

1. Characterize `sync_all()` + `fs::rename` with direct test evidence for all
   7 named observable properties (F3b-1).
2. Investigate parent-directory durability feasibility and record what can be
   proven and what remains unverified (F3b-2).
3. Characterize the Replay Windows publish primitive with direct test evidence
   for each of the 6 observable stages (F3b-3).
4. Characterize Trail JSONL interruption behaviour including truncated final-line
   detection and incomplete-line handling (F3b-4).
5. Characterize Local Anchor root reparse-point safety with a bounded
   Windows-only test (F3b-5).

### F3b-1: `sync_all()` + `fs::rename`

Build a minimal private characterization test for the primitive used by
StoreRoot/Candidate/Local Anchor style persistence. Use a temporary directory.

Directly establish what can reasonably be tested on the primary Windows target:

- temporary file is fully written;
- `sync_all()` succeeds;
- rename succeeds;
- final path contains the complete expected bytes;
- temporary path disappears;
- no partial final file is exposed during ordinary execution;
- restart/reopen reads the exact expected bytes.

If true power-loss durability cannot be deterministically established, report
`UNVERIFIED`.

### F3b-2: Parent-directory durability feasibility

Investigate the exact Windows/Rust mechanisms available for flushing or
proving directory-entry durability. Determine from direct platform/API evidence
and a minimal experiment:

- whether Windows permits opening the relevant directory with necessary flags/access;
- whether `FlushFileBuffers` can meaningfully be invoked on that handle;
- whether the current Rust implementation performs such an operation;
- what narrower claim can actually be proven.

Do not change production persistence.

### F3b-3: Replay Windows primitive

Characterize the accepted-main sequence in `publish_new_canonical_file_with_temporary_stem`:

- `CreateFileW(CREATE_NEW | FILE_FLAG_WRITE_THROUGH)` — test observable durability;
- `WriteFile` — test complete write;
- `FlushFileBuffers` before rename — test file-data durability;
- `SetFileInformationByHandle` rename — test rename properties;
- `FlushFileBuffers` on the renamed file handle — test what this proves;
- reopen/re-read exact-byte verification — test what this proves.

Test the observable guarantees individually. Establish exactly what the
post-rename re-read proves and what it does not prove.

### F3b-4: Trail interruption behaviour

Characterize JSONL append using `writeln!`, `flush()`, `sync_data()`:

- complete line survives close/reopen;
- multiple complete lines remain ordered and parseable;
- deliberately truncated final line is detected by the current reader;
- establish current behaviour when the final JSONL entry is incomplete.

If current recovery accepts, ignores, or fails on a partial final line,
record the exact behaviour. Do not redesign Trail or add per-line digests.

### F3b-5: Local Anchor root safety

Characterize the Local Anchor Admission Store root path safety:

- determine whether a reparse point at or within the persistence root can
  redirect admission writes despite hashed safe filenames;
- use a bounded Windows-only test.

If exposure is demonstrated, record it as a confirmed defect and route the
repair to the correct later package. Do not repair root-safety behaviour in F3b.

## Relevant components

- `docs/architecture/TETHERS_FOUNDATION_PASS.md` (F3a/F3b/C boundary)
- `docs/foundation-pass/PERSISTENCE_INVENTORY.md` (F3a deliverable, update with F3b findings)
- `docs/foundation-pass/DEBT_LEDGER.md` (only for directly demonstrated defects/clarifications)
- `docs/worker-notes/2026-08-07-f3a-persistence-vocabulary.md` (F3a evidence)
- `tethers-0.1/host-rust/src/m3_store.rs` (StoreRoot, `create_json`, `verify_chain`, `reject_reparse`)
- `tethers-0.1/host-rust/src/candidate.rs` (write_new, Candidate Registry)
- `tethers-0.1/host-rust/src/local_anchor.rs` (AdmissionStore, atomic_create, safe_filename)
- `tethers-0.1/host-rust/src/replay_windows.rs` (publish_new_canonical_file_with_temporary_stem)
- `tethers-0.1/host-rust/src/dispatch.rs` (FileTrail, JSONL append)
- `docs/worker-notes/2026-08-07-f3b-windows-persistence-evidence.md`
- `docs/CURRENT_GOAL.md` and `docs/CURRENT_CLINE_TASK.md`

## Frozen decisions and invariants

- Accepted main is `145a791ceb3f5e3b8855aeadbac83671d9a2b363` (F3a merged).
  If live `origin/main` differs, record the direct Git evidence and stop.
- F3b is evidence-gathering only. Do not repair, redesign, or change production
  persistence behaviour, write primitives, directory handling, Trail, Replay,
  Local Anchor paths, installation intent/publication, or CLI/protocol/JSON
  output.
- Every conclusion must separate: observed primitive, directly tested property,
  remaining uncertainty. Do not infer a Windows guarantee from API names or
  documentation terminology alone.
- For each property, report one of `PROVEN`, `DISPROVEN`, or `UNVERIFIED` with
  exact source/test evidence.
- A failing characterization test is valuable evidence. Do not change production
  code to make it green.
- Production seams must not be widened merely to make tests easier. Prefer
  private helpers and isolated characterization harnesses.
- Preserve F1 literal fixtures exactly.
- One implementation owner per task. Do not begin F3c.

## Acceptance criteria

1. F3b-1 characterizes `sync_all()` + `fs::rename` with direct test evidence
   for all 7 named observable properties.
2. F3b-2 investigates parent-directory durability and records what can be
   proven and what remains unverified.
3. F3b-3 characterizes all 6 Replay Windows primitive stages with direct test
   evidence for each observable guarantee.
4. F3b-4 characterizes Trail JSONL interruption behaviour including truncated
   final-line detection.
5. F3b-5 characterizes Local Anchor root reparse-point safety with a bounded
   Windows-only test.
6. PERSISTENCE_INVENTORY.md updated with `PROVEN (F3b)`, `DISPROVEN (F3b)`,
   or `UNVERIFIED (F3b)` tags where F3b establishes evidence.
7. DEBT_LEDGER.md updated only for directly demonstrated defects or
   clarifications.
8. F3a/F1 fixtures are byte-identical to accepted main.
9. Complete branch diff contains only characterization tests and documentation;
   no production repair or persistence redesign.
10. F3b worker note records exact evidence, findings, and residual questions.

## Required verification

Run the following serially after the final code change. Record each result as
PASS, FAIL, or NOT RUN; a mandatory NOT RUN blocks COMPLETE.

```powershell
git fetch origin --prune
git rev-parse origin/main
git rev-parse HEAD
git status --short --branch

cargo fmt --all -- --check
cargo check --all-targets --all-features --locked
cargo test --all-targets --all-features --locked
cargo clippy --all-targets --all-features --locked -- -W clippy::all

just verify
just verify-agent

pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1

git diff --exit-code origin/main...HEAD -- docs/foundation-pass/fixtures
git diff --check origin/main...HEAD
git diff --name-only origin/main...HEAD
git status --short --branch
```

Also run every focused F3b characterization test explicitly. Record each
separately.

## Forbidden changes

Do not perform:

- StoreRoot repair (directory flushing, migration);
- Candidate Registry repair;
- Local Anchor path handling or root-safety repair;
- Replay redesign or write-primitive change;
- Trail redesign, per-line digest, or integrity footer addition;
- installation intent/publication repair (F3c);
- immutable/current-state implementation changes (F3d);
- CLI, JSON, protocol, exit-code, compatibility fixture, or replay-digest changes;
- a universal storage abstraction or new dependency;
- beginning F3c, F3d, or F3e;
- changing F1 fixtures;
- weakening an experiment because it is difficult to make pass.

## Stop conditions

Stop and report direct evidence if `origin/main` differs from
`145a791ceb3f5e3b8855aeadbac83671d9a2b363`; the worktree/branch/base is
unexpected; a required property cannot be characterized on the available
target; a finding would require production repair to prove; a required check
fails; or two materially similar evidence attempts fail. Return one smallest
unresolved question.

## Expected pre-existing changes

None.
