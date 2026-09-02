# Worker Note

Task: `Rocket V3 — R3-3B3A Exact Parent-Vector Completion Theorem`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `Codex`
Status: `BLOCKED`
Base commit: `64d1557603366f2b8b934f987bfdef87e2b4ec0e`
Implementation checkpoint: `2234438f8fa03811bc33788d26744023bef6495e`
Branch / Worktree: `feature/rocket-v3-r3-3b3a-parent-vector-completion` / `D:\The Next Thing\Tethers Lang - Rocket V3 R3-3B3A Parent Vector Completion`

## Requested outcome

Establish an exact partial parent-vector completion theorem for the bounded
Origin-only connected rooted success-tree shape, and build a left-to-right
constructor only if the theorem is proved.

## Changes made

- Added a research-only dense rooted-tree abstraction and independent
  brute-force `Completable` oracle.
- Added an explicit local-capacity candidate predicate for the first bounded
  recurrence investigation.  It is intentionally not named or treated as
  `Completable`.
- Added the required historical B3 counterexample reconstruction, frozen
  payload comparison, and partial-prefix evidence.
- Added the authorised review document and Dune test stanza.
- Changed the packet state to `IN_PROGRESS` during the investigation and then
  to `BLOCKED` after the committed evidence checkpoint.
- Did not add a parent-vector constructor, revive subtree ranking, modify
  frozen V2/Core/R3-2 authorities, or begin B3B/R3-3C/R3-4 work.

## Decisions and assumptions

The exact predicate tested is existence of a bijection from semantic Origins
to numeric labels with fixed entry label and an exactly matching supplied
parent-vector prefix.  The independent oracle enumerates all remaining dense
label assignments and compares only the supplied prefix at complete leaves.
Dense fixture positions are construction coordinates, never canonical order.

The first candidate compact state retains fixed-prefix edge validity,
acyclicity, external-root capacity, maximum Origin child-degree capacity and
the entry edge terminal/non-terminal kind.  These conditions are necessary but
not sufficient, so they are retained only as a disproved research candidate.

## Evidence

- Fresh worktree started at the requested remote HEAD
  `00559f5d605c62a4840569d7dd679af8937a1d17`; merge-base with the required
  base was `64d1557603366f2b8b934f987bfdef87e2b4ec0e`; initial worktree was
  clean and the packet checker reported `control-v1/READY`.
- Exact switch verified at
  `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`; OCaml `5.5.0`,
  Dune `3.24.0`; `scripts/check-dev-tools.ps1` passed.
- Historical B3 tree `[6,2,3,6,5,6,-1]` reproduced independently.  The
  historical rank vector was `[2,Complete,2,2,3,4,6]`; the exact oracle vector
  was `[2,Complete,2,2,3,4,5]`.  First differing continuation slot was `7`,
  target `6` versus `5`.  Frozen payload byte offset `55` was `0x36` versus
  `0x35`.
- B3A focused suite: `rocket-v3-tree-completion: 47634/47634 checks passed`.
  It checked `partial_prefixes_checked=23814`,
  `brute_force_completions_considered=1465731`,
  `feasibility_states=23814`, `candidate_targets_considered=64878`,
  `complete_permutations_enumerated=1465731`, and `max_state_width=7`.
- The candidate recurrence is disproved by
  `parents=[1,2,-1]`, entry `0`, prefix `[2,ProgramComplete]`: the local
  candidate accepts it, while the independent brute-force oracle rejects it.
- Inherited focused suites passed: B2 `69/69`, R3-3A `39/39`, R3-1
  `214/214`, R3-2 `4807/4807`.
- Full `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune build @all` passed.
- Full `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune runtest --force` passed with exit code `0`, including the V2 reference, production and IR suites, the generated differential corpus (`valid=5000`, `mismatches=0`), all inherited Rocket suites, and B3A `47634/47634`.
- `git diff --check` and staged diff inspection passed before the evidence
  checkpoint.  Only packet/review/note/Dune and the three authorised
  `tree_completion` paths are present.

## Discoveries

The failed B3 subtree-rank theorem loses global placement information: a local
subtree shape does not determine whether its parent/source slots can satisfy
the earlier fixed vector.  The three-node false positive shows that local
degree and component-style facts cannot express the coupled constraint that a
label fixed as the entry's parent must itself have the original semantic
parent kind.  Complete rooted-subtree isomorphism and interchangeability under
the partial global label objective remain separate claims.

The brute-force oracle is exact for the stated finite domain, but the bounded
investigation did not establish a compact exact polynomial/bounded recurrence.
Treating the oracle itself as a scalable predicate would simply reintroduce
complete assignment enumeration and would violate the task boundary.

## Remaining risks

No exact compact `Completable` state or parent-vector constructor exists in this
checkpoint.  The blocked result does not show that no tree-specific theorem is
possible; it shows that the investigated local-capacity recurrence is false and
that the required replacement theorem was not established in this task.

## Smallest next action

Specify and independently review an exact tree-automaton or matching state
that retains the coupled placement information, then rerun the same exhaustive
prefix differential corpus.  Do not start B3B or production integration from
this blocked evidence without that separate decision.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/review/rocket-v3/R3_3B3A_PARENT_VECTOR_COMPLETION.md`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_completion.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_completion_test.ml`
- `2234438f8fa03811bc33788d26744023bef6495e`
