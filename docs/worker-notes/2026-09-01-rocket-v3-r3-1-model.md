# Rocket V3 R3-1 — Immutable Typed Relational Model

Task: `Rocket V3 R3-1 immutable typed relational model`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `Codex`
Status: `COMPLETE`
Base commit: `0fd316083e1b26c3564080dec16d62490116858c`
Implementation checkpoint: `73f3421e03ac78a9357b0cfac86708c4e0a9f975`

## Requested outcome

Implement and test the immutable typed Rocket V3 semantic relation model over validated Tethers Core, covering the six frozen anonymous families and the complete R3-0 relation inventory, without implementing refinement, search, canonical emission or production integration.

## Changes made

- Added `tethers_core_rocket_v3_model.ml/.mli` with validation-first construction, six typed anonymous families, fixed `ProgramRoot`, `ProgramScope`, `ProgramComplete` and `BranchStop` concepts, deterministic scalar descriptors, typed discriminated edges, exact reverse adjacency and read-only evidence accessors.
- Modelled `Anchor_origin`, `Action_origin` and `Together_origin` as Origin vertices. Modelled `Batch_site` as a Batch vertex only; no synthetic Origin is created.
- Covered the R3-0 R01-R29 inventory, including success continuation, entry, declared/aggregate facts, provenance, guards, all Action bindings, Together membership, Branch outcomes, role scope/contracts/objective, template/program ownership, fact scope and Batch/template context. The maintained taxonomy expands the inventory’s family-specific ownership relations and direct Batch context into 33 typed relation kinds.
- Added focused tests covering family cardinality, relation/lookup coverage, forward/reverse duality, raw-ID and storage-order invariance, neutral scalar boundaries, scope collisions, fail-closed validation, all four branch outcomes and 1/10/50/100/250/500/1000 Action-chain structures.
- Added only the minimal Dune test stanza. No Rocket V2, Enc_V2, ProgramDigest, planner, wire or Rust production path was changed.

## Decisions and assumptions

- Validation is delegated to the existing Core validator before any returned model exists; invalid Core returns validator errors.
- Raw IDs and construction order are lookup/indexing aids only. Structural evidence sorts semantic descriptors and edge evidence, so internal indices and storage order do not define identity.
- Relation multiplicity is retained by emitting one edge occurrence per Core occurrence; validator-invalid duplicate structures fail before construction.
- Role lookup is scope-qualified. Program and template roles with the same raw role ID are separate ScopedRole vertices.
- The returned model uses immutable arrays. Mutable lists and hash tables exist only during construction and are not exposed.

## Evidence

- Startup controls passed on the dedicated worktree and branch `feature/rocket-v3-r3-1-model`; exact base was `0fd316083e1b26c3564080dec16d62490116858c`.
- Authorized external switch verified as `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`; `opam switch show` and `opam var prefix` resolved to that path, with OCaml `5.5.0` and Dune `3.24.0`.
- Focused command: `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune exec ./bin/tethers_core_rocket_v3_model_test.exe` — `rocket-v3-model: 214/214 checks passed`.
- Full build: `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune build @all` — passed.
- Full tests: `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune runtest --force` — passed: lowerer 52/52, validator 51/51, plan bridge 188/188, adapter 46/46, request adapter 89/89, wire T1/T2/T3, Rocket V3 model 214/214, V2 reference oracle, V2 production canonicaliser and V2 exact hybrid/IR suites including the 5,000-case generated corpus.
- `git diff --cached --check` and the final diff review passed; Git reported only expected LF-to-CRLF warnings for Windows working-copy normalization.

## Discoveries

The implementation confirmed the R3-0 architectural correction: `origin_site` is a structural sum type, not an Origin-family promise. Batch sites carry `batch_id`, resolve through BatchMap, and must remain Batch-family vertices with Batch/template and Batch/fact relations. The V2 relation IR omitted several Enc_V2-visible relations and did not provide complete inverse structure; the R3-1 model exposes those relations without changing the frozen encoder.

## Remaining risks

Rocket V3 refinement, canonical search, canonical label assignment, prefix pruning, automorphism pruning, component recursion, resource budgets and production cutover remain unimplemented by design. The model’s construction uses straightforward immutable output arrays but is not yet a CSR/worklist refinement engine; that belongs to later R3 work. No unresolved R3-1 blocker remains.

## Smallest next action

Review the committed R3-1 checkpoint as input to the separately authorized R3-2 refinement task. Do not begin R3-2 from this task.

## References

- Design authority: `docs/review/rocket-v3/R3_0_SEMANTIC_RELATION_INVENTORY.md`
- Implementation checkpoint: `73f3421e03ac78a9357b0cfac86708c4e0a9f975`
- Authorized source files: `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_model.ml`, `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_model.mli`
- Focused tests: `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_model_test.ml`
