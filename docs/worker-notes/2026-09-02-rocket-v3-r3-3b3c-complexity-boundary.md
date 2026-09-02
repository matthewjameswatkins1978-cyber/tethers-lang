# Worker Note

Task: `Rocket V3 - R3-3B3C Complexity Boundary`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `Codex`
Status: `COMPLETE`
Base commit: `ef22f861ebfce6ed6341b5e0043baf53b153aab3`
Implementation checkpoint: `9711f8b718712a18f0e0a8aca8fe7e2b600935f9`

## Changes made

- `docs/CURRENT_CLINE_TASK.md`
- `docs/review/rocket-v3/R3_3B3C_COMPLEXITY_BOUNDARY.md`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_complexity.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_complexity.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_complexity_test.ml`
- `tethers-0.1/engine-ocaml/bin/dune`

## Requested outcome

Established the exact Rocket-to-restricted-forest correspondence for an
acyclic partial parent vector.  A prefix over `N` slots plus fixed external
`ProgramComplete` is a spanning directed forest precisely when its supplied
edges are acyclic; completion adds only the missing outgoing parent edges and
must embed the result into the semantic host tree.  The reverse construction
holds for a pinned, externally rooted completion.

The contiguous source-prefix condition is not an obstruction: arbitrary
rooted forests can be relabelled with all non-roots in slots `1..m` and roots
afterwards.  Rocket `Completable` is in NP.  The cited unpinned rooted
spanning-forest hardness results do not prove Rocket NP-hard without a
pin-preserving reduction for the fixed entry image and external terminal.

The exact forest-component parameter including `ProgramComplete` is
`c = N + 1 - m` for an acyclic length-`m` prefix.  The final recommendation is
`FPT COMPLETION RESEARCH`; that task was not started.

## Decisions and assumptions

- Frozen Enc_V2, ProgramDigest V2, Core semantics and all production call paths
  are unchanged.
- B3A and B3B remain read-only authorities.
- No canonical-label solver, graph I/R, SAT/SMT dependency or production
  integration was added.
- Research fixture coordinates are not canonical identity evidence.

## Evidence

- `tethers_core_rocket_v3_tree_complexity_test.ml` — cycle and multiple-terminal
  structural rejection, bounded forest-witness equivalence against B3A, and
  rooted-forest prefix normalization.
- Existing B3A test — local-capacity false positive remains rejected by the
  independent brute-force oracle.
- Existing B3B test — multiple partial components remain explicitly outside
  the connected matching theorem.

## Commands Executed

- `pwsh -NoProfile -File scripts/check-dev-tools.ps1` — `PASS`
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — `PASS control-v1/READY` before mutation; `PASS control-v1/IN_PROGRESS` during work
- Exact switch inspection — `PASS` (OCaml `5.5.0`, Dune `3.24.0`, Yojson `2.2.2`)
- `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune exec bin/tethers_core_rocket_v3_tree_complexity_test.exe` — `PASS 57/57`
- `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune build @all` — `PASS`
- `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune runtest --force` — `PASS`; B3A `47634/47634`, B3B `10/10`, B2 `69/69`, R3-3A `39/39`, R3-1 `214/214`, R3-2 `4807/4807`, V2 corpus `5000 valid / 0 mismatches`
- `git diff --check` — `PASS`
- `git diff --cached --check` — `PASS`
- `git commit -m "research: establish Rocket forest complexity boundary"` — `PASS`, checkpoint `9711f8b718712a18f0e0a8aca8fe7e2b600935f9`

## Unrun Checks and Reason

- `None`.

## Discoveries

- The prefix-source restriction is normalizable for rooted forests and therefore
  is not the smallest reduction failure.
- The exact reduction target is a restricted pinned directed forest extension,
  not generic unary-list ListIso or unrestricted undirected forest completion.
- Unpinned NP-hardness cannot be transferred silently across Rocket's entry
  mapping and fixed `ProgramComplete` semantics.
- The B3C witness checker is research-only and is not in a production call path.

## Remaining risks

- Rocket NP-hardness remains unproved until a pin-preserving hardness theorem or
  gadget is separately established.
- The cited FPT runtime has not been adapted or implemented for Rocket; its
  use remains future research.
- The complexity of directly computing the frozen minimum payload remains a
  separate question from prefix feasibility.

## Smallest next action

Research a Rocket-specific FPT completion adaptation using the exact parameter
`c = N + 1 - m`, while preserving the entry pin, external root and directed
parent constraints.  Do not begin that task from this note without a new
packet.

## References

- `docs/review/rocket-v3/R3_3B3C_COMPLEXITY_BOUNDARY.md`
- `docs/review/rocket-v3/R3_3B3A_PARENT_VECTOR_COMPLETION.md`
- `docs/review/rocket-v3/R3_3B3B_LISTISO_REDUCTION.md`
- `9711f8b718712a18f0e0a8aca8fe7e2b600935f9`
- Garey and Johnson, *Computers and Intractability*, Theorem 4.6,
  [SUBFOREST ISOMORPHISM PDF](https://perso.limos.fr/~palafour/PAPERS/PDF/Garey-Johnson79.pdf)
- Liu, Chen, Zheng, Wang and Shi, *Parameterized algorithms for the spanning
  forest isomorphism and containment on tree*, [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0304397525005894)
