# Current Implementation Task

Control contract: `1`
Task: `F3a - Persistence inventory and vocabulary`
Owner: `Codex`
Status: `READY`
Task colour: `Red`
Route: `Codex performs the bounded documentation and evidence pass; Lucy performs independent architecture review before F3b`
Worker note: `docs/worker-notes/2026-08-07-f3a-persistence-vocabulary.md`
Base branch: `main`
Base commit: `83eec98a0f33f964623f4cbbf4548a76bbdf5255`
Implementation branch: `foundation/f3a-persistence-vocabulary`
Parent branch: `main`
Parent tip: `83eec98a0f33f964623f4cbbf4548a76bbdf5255`
Preparation checkpoint: `WORKTREE`
OCaml switch path: `N/A`
Rust toolchain: `Not required; this is documentation and evidence work only`
Toolchain preflight: `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`

## Objective

Produce one evidence-backed vocabulary and complete inventory for every
filesystem-backed persistence store in the accepted F2 mainline. The result
must distinguish what the current implementation proves from what remains
unverified, without repairing any persistence behaviour or beginning F3b.

## Relevant background and existing behaviour

F1 established the initial inventory at
`24428139807cac0adeb0b62264547e61ca809d16`; F2 was accepted and merged as
`83eec98a0f33f964623f4cbbf4548a76bbdf5255`. F1 records four vocabulary
classes in `docs/foundation-pass/PERSISTENCE_INVENTORY.md`, but its claims must
now be reconciled against the accepted F2 mainline and direct source/test
evidence.

The Foundation Pass explicitly separates F3a vocabulary from F3b Windows
primitive evidence. In particular, no current store has a confirmed
directory-entry durability claim. `m3_store.rs` is shared infrastructure, not
an independent store; Trail appends JSONL rather than writing atomic records;
and the installation recovery plan is a reader/planner, not a store.

## Required behaviour

1. Inspect the accepted mainline and the Foundation Pass, persistence, debt,
   module/dependency, test-inventory, and F1/F2 worker-note evidence named in
   this packet. Identify every filesystem-backed store and every shared write
   primitive without treating a historical inventory row as proof.
2. Classify each store exactly once as an immutable atomic record, replaceable
   current-state record, append-only causal log, or multi-step intent/recovery
   journal. Explicitly record any non-durable in-memory state in the appendix;
   do not call it a persistence store.
3. For every classified store, record the current write primitive, atomic
   visibility statement, file-durability statement, directory-durability
   statement, recovery reader, corruption classification, unsafe-path
   protection, and one or more direct tests. Cite concrete module/function and
   test names; use `UNVERIFIED (F3b)` where the evidence does not establish a
   Windows guarantee.
4. Reconcile contradictions, overclaims, duplicate rows, and category errors
   in the persistence inventory. Record an evidence-backed correction in the
   debt ledger only when a ledger statement itself is inaccurate; do not turn
   an unverified guarantee into a defect or attempt a repair.
5. Finish with a bounded F3a worker note and documentation-only verification.
   Stop after the packet deliverables: F3b primitive experiments, installation
   intent/recovery repair, Trail/replay redesign, and all production/test work
   remain out of scope.

## Relevant components

- `docs/architecture/TETHERS_FOUNDATION_PASS.md` (F3a/F3b boundary)
- `docs/foundation-pass/PERSISTENCE_INVENTORY.md` (primary F3a deliverable)
- `docs/foundation-pass/DEBT_LEDGER.md` (only evidence-backed inventory corrections)
- `docs/foundation-pass/MODULE_DEPENDENCY_MAP.md` and `docs/foundation-pass/TEST_INVENTORY.md`
- `docs/worker-notes/2026-08-06-f1-baseline.md` and `docs/worker-notes/2026-08-07-f2-operational-correctness.md`
- `tethers-0.1/host-rust/src/m3_store.rs`, `replay_windows.rs`, `installed.rs`,
  `installation_publication_intent.rs`, `dispatch.rs`, `local_anchor.rs`, and
  the source/tests reached directly from those modules
- `docs/worker-notes/2026-08-07-f3a-persistence-vocabulary.md`
- `docs/CURRENT_GOAL.md` and `docs/CURRENT_CLINE_TASK.md`

## Frozen decisions and invariants

- The accepted mainline/base is `83eec98a0f33f964623f4cbbf4548a76bbdf5255`.
  If live `origin/main` differs before F3a begins, record the direct Git
  evidence and stop for a packet correction.
- Directory-entry durability is unverified unless F3a discovers direct,
  accepted-main evidence that proves a narrower statement. F3a may clarify
  wording but must route primitive validation to F3b.
- The four persistence classes are vocabulary, not an instruction to make
  stores share an implementation.
- Preserve F1 literal fixtures exactly. They are independent compatibility
  evidence and are not a persistence-inventory output.
- Do not treat historical `PackageTrustEvidence` or any historical inventory
  statement as proof of current trust or current persistence behaviour.
- Every claim must be supported by the accepted-main source or a direct test;
  uncertainty is reported honestly as `UNVERIFIED`, not inferred from a nearby
  API, a passing test, or Windows terminology.

## Acceptance criteria

1. The inventory names every filesystem-backed store reachable in accepted
   main and classifies each once using the frozen four-class vocabulary.
2. Each row records all nine required evidence fields: write primitive, atomic
   visibility, file durability, directory durability, recovery reader,
   corruption classification, unsafe-path protection, and direct tests, plus
   its class.
3. Every durability statement distinguishes proven file data, atomic
   visibility, and directory-entry durability; unsupported claims are marked
   `UNVERIFIED (F3b)`.
4. The in-memory appendix is complete and does not misclassify process-local
   state as durable persistence.
5. F1 fixtures are byte-identical to accepted main, and the complete branch
   diff contains documentation only.
6. The F3a worker note states actual source/test evidence, corrections made,
   residual F3b questions, and no unrun command as passed.
7. Packet checker, whitespace check, documentation-only diff review, and
   final Git status pass with the exact results recorded in the worker note.

## Required verification

Run after the final documentation edit and record each result as `PASS`,
`FAIL`, or `NOT RUN` in the worker note. A mandatory `NOT RUN` blocks
`COMPLETE`.

```powershell
git fetch origin --prune
git rev-parse origin/main
git rev-parse HEAD
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
git diff --exit-code origin/main...HEAD -- docs/foundation-pass/fixtures
git diff --check origin/main...HEAD
git diff --name-only origin/main...HEAD
git diff --name-only origin/main...HEAD -- ':!docs/**'
git status --short --branch
```

Before claiming complete, inspect each inventory row against its named source
and direct test. No Rust, OCaml, or integration test run is required because
F3a changes documentation only; record those suites as `NOT RUN (not required
for documentation-only F3a)`.

## Forbidden changes

Do not perform:

- production, test, fixture, dependency, Cargo.lock, OCaml, protocol, CLI, or
  compatibility-output changes;
- persistence repairs, write-primitive changes, directory flushes, migration,
  or recovery behaviour changes;
- Windows primitive experiments or interruption/fault-injection tests (F3b);
- installation intent/publication repair (F3c);
- immutable/current-state implementation changes (F3d) or Trail/replay
  redesign (F3e);
- a universal storage framework, extraction, speculative renaming, or a new
  persistence abstraction;
- starting F3b or any later Foundation package;
- changing F1 fixtures.

## Stop conditions

Stop and report direct evidence if `origin/main` differs from the frozen base;
the branch/base is unexpected; a required claim cannot be tied to a concrete
accepted-main source or direct test; a classification requires a semantic or
recovery design decision; a correction needs production/test/fixture changes;
a required check fails; or two materially similar evidence attempts fail.
Return one smallest unresolved question. Do not weaken the packet, invent a
durability guarantee, or proceed into F3b to bypass a stop.

## Expected pre-existing changes

None.
