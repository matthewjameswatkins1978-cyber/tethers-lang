# C5 — Fresh-Agent Concurrency Authoring Proof

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Red`

Owner: `Fresh Tethers Agent`

Route: `Fresh-agent authoring proof — awaiting Lucy review`

Base commit: `7c9f846cf5c7681a919f321faf42657c386d99ca`

Worker note: `docs/worker-notes/2026-08-15-c5-fresh-agent-concurrency-proof.md`

Updated: 2026-08-15

## Objective

Test whether Tethers concurrency can be understood and used by a newcomer from the repository's intended documentation, examples and normal user-facing surfaces.

## Relevant background and existing behaviour

- C1–C4 are treated as frozen.
- This task tests authoring usability, not implementation architecture.
- No production semantic changes are authorised.
- A documentation/usability failure is a legitimate C5 BLOCKER.
- Do not secretly repair the language or runtime.

## Required behaviour

1. Author ONE real Tether source file with one trigger/event, any necessary simple Condition, one `together` group, exactly TWO independent Actions with TWO DIFFERENT capability names.
2. Process the Tether through the normal source-language path.
3. Capture/assert the generated Runtime Plan proving two Actions, different capabilities, one Together group, deterministic membership.
4. Run the authored Tether using safe existing fixture providers proving both members attempted, both terminalise, GroupJoin occurs, Trail contains truthful evidence.
5. Run the same authored Tether 3+ times proving semantic identity/shape preservation.
6. Write a human-readability assessment.

## Relevant components

- Tethers language source and normal authoring surface
- Existing fixture/test provider infrastructure
- Normal integration-test path for source-to-host execution

## Frozen decisions and invariants

- No production semantic changes authorised.
- No scheduler redesign.
- C1–C4 frozen.
- This is a Red proof — authoring usability, not implementation.

## Acceptance criteria

1. Fresh agent understood the syntax from ordinary docs/examples.
2. A real source Tether was authored.
3. Exactly two different capabilities are used.
4. Both belong to one Together group.
5. Plan generated through normal language path.
6. Plan group membership is deterministic.
7. Normal runtime executes both members.
8. Both terminalise.
9. GroupJoin succeeds.
10. Trail/result evidence is inspectable.
11. Repeated runs preserve semantic identity/result.
12. No production semantic change was required.

## Stop conditions

- If ordinary docs are insufficient to author the Tether.
- If the only way to make the Tether work is to change production semantics.

## Expected pre-existing changes

- `WORKTREE.md`
- `docs/CANONICAL_FORMAT_V2_SPEC_DRAFT.md`
- `docs/performance/CORE_PHASE_A_IMPLEMENTATION_PACKET.md`
- `docs/performance/R1_PERFORMANCE_PROOF.md`
- `docs/performance/core-phase-a/`
- `docs/performance/r1/`
- `docs/worker-notes/2026-08-12-c-core-cheap-structural-fixes.md`
- `docs/worker-notes/2026-08-14-c2a1-together-semantic-bridge.md`
- `scripts/assert-worktree.ps1`
- `tethers-0.1/engine-ocaml/bin/tethers_cb3t_tie_audit.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_rank_avalanche.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_v2_canon_label.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_v2_canon_label_test.ml`
