# Rocket V3 — R3-3 Exact Unpruned I/R Search

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Codex`

Route: `Codex implementation in a fresh dedicated worktree; exact discrete-leaf bridge + unpruned I/R search only`

Base commit: `21bb7442fa9f8442db98e193eb4954096f356678`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

OCaml toolchain contract: use this exact external directory switch with explicit `--switch`; run Dune against the current R3-3 worktree source tree. Do not create, copy, move, select globally, or substitute another installed switch. For repository scripts that invoke opam without `--switch`, set `OPAMSWITCH` only process-locally to this exact path.

Worker note: `docs/worker-notes/2026-09-01-rocket-v3-r3-3-exact-search.md`

Related issue: `#5 — BUG: Rocket V2 factorial search on simple sequential Action chains`

Accepted inputs:

- R3-0 complete semantic relation inventory
- R3-1 immutable typed model
- R3-2 stable typed partition refinement

Updated: 2026-09-01

## Objective

Implement the first exact Rocket V3 canonical search engine over the accepted R3-1/R3-2 machinery.

R3-3 has two strictly ordered stages:

### Stage A — discrete leaf certificate / Enc_V2 bridge

Prove that a discrete stable V3 partition can be converted into one legal frozen V2 `label_assignment`, then encoded using the existing frozen `Tethers_core_canonical_v2_format.encode_program`.

This stage MUST establish exact byte parity against the slow V2 oracle on tractable root-discrete fixtures before broader search is implemented.

If discrete-partition ordering cannot be mapped to exact V2 minimum bytes, STOP as a Red architectural finding. Do not paper over the mismatch with a greedy encoder special case.

### Stage B — complete unpruned individualisation/refinement search

Only after Stage A is green, implement complete I/R traversal:

`initial partition -> stable refinement -> select non-singleton cell -> individualize each member -> stable refinement -> recurse -> Enc_V2 at every discrete leaf -> unsigned-byte minimum`.

Search may use refinement to expose structure and reduce ambiguity, but the winning certificate remains frozen `Enc_V2(P, λ)` bytes. Colours/cell keys never become ProgramDigest identity themselves.

No prefix pruning, automorphism/orbit pruning, component recursion, memo pruning, greedy family forcing, search budget or production cutover is authorised in R3-3.

## Relevant background and existing behaviour

Frozen identity remains:

`CanonicalPayload_V2(P) = min { Enc_V2(P, λ) | λ ∈ Λ(P) }`

`ProgramDigest_V2(P) = SHA-256(domain_v2 || CanonicalPayload_V2(P))`

R3-2 proved that a homogeneous 1000-Action sequential chain reaches a discrete stable partition with:

- `relation_visits=6999`
- `splitter_pops=1004`
- `cell_splits=998`
- `max_worklist=6`

No search was required to distinguish those Actions structurally. R3-3 must now prove that this discrete result can be translated into the exact frozen canonical bytes, rather than assuming refinement cell order is canonical label order.

The frozen role rule MUST be preserved exactly:

- one global role label range;
- Program-scope role block first;
- template-role blocks follow in ascending assigned canonical template-label order;
- roles may permute only inside their own scope block;
- no cross-scope role assignment.

## Required behaviour

1. Add the narrowest possible model identity bridge needed only for leaf encoding:
   - raw Origin ID -> model vertex lookup;
   - Fact ID -> vertex;
   - Branch ID -> vertex;
   - Batch ID -> vertex;
   - ItemTemplate ID -> vertex;
   - scope-qualified Role ID -> vertex.
   Lookups return options/fail-closed results. Raw IDs remain construction/encoding handles only and MUST NOT be consumed by refinement, cell selection, individualisation priority or canonical comparison.
2. Preserve existing R3-1 structural evidence exactly after adding the lookup bridge. The lookup tables are not included in `structural_evidence` and do not alter vertex/scalar/edge construction.
3. Add a dedicated Rocket V3 leaf-encoder module that accepts the exact Core program, its paired V3 model and a discrete stable partition and constructs a legal frozen V2 `label_assignment`.
4. For Origin, Fact, Branch, Batch and ItemTemplate, derive one label order from the discrete partition's invariant leaf ordering, never raw IDs/internal vertex numbers/storage order. The implementation must document the exact ordering certificate used.
5. Preserve the frozen scoped-role block rule exactly. Program roles occupy the first interval. Template-role intervals are determined by the leaf's assigned template labels. Roles are ordered only within their legal scope block.
6. Leaf encoding MUST call the existing frozen `encode_program`; it must not reimplement Enc_V2.
7. Stage A must compare the discrete-leaf payload byte-for-byte with the V2 slow oracle on tractable root-discrete fixtures covering at minimum:
   - homogeneous success chains sizes 1 through 6;
   - distinct scalar facts;
   - entry/root + ProgramComplete structure;
   - Batch/template structure;
   - program and template scoped roles including more than one template role block;
   - Branch/Stop and outcome discriminators;
   - Action binding/scoped-role references.
8. Stage A must include raw-ID renaming and representation-order permutations and prove identical leaf payload.
9. Stage A STOP rule: any oracle mismatch on a valid root-discrete fixture blocks Stage B. Record the first exact differing payload/fixture and do not add heuristic special cases.
10. After Stage A passes, add a complete unpruned I/R search module. At each non-discrete stable node, select one non-singleton cell and create one child for every member of that cell.
11. Individualisation must be represented as an explicit artificial ordered distinction local to the search branch, then followed by ordinary R3-2 refinement. It must not modify the R3-1 model or frozen Core meaning.
12. Every selected-cell member must be explored. Branch traversal order may affect operational traversal only; because R3-3 has no pruning or budget, it must not affect the final minimum payload.
13. Provide at least three deterministic cell selector strategies based only on partition semantic evidence/cell size, for example:
    - smallest non-singleton cell;
    - largest non-singleton cell;
    - semantic-first non-singleton cell.
    Exact names may vary. No selector may use raw IDs or internal vertex handles as semantic tie-breakers.
14. All selector strategies must return identical canonical payload/preimage/digest for the same program. Search statistics may differ by selector; identity must not.
15. A discrete search leaf must be encoded only through the Stage-A leaf encoder. The search module must not assign labels by a second route.
16. Compare Rocket V3 payload/digest exactly against:
    - the slow V2 oracle wherever the oracle accepts the case;
    - V2 production/exhaustive where tractable;
    - Rocket V2 IR as regression evidence where appropriate.
    V2 engines are test oracles only and MUST NOT be called by the Rocket V3 implementation.
17. Add deterministic generated differential coverage including storage permutation and complete raw-ID renaming. Any payload or digest mismatch is a hard failure.
18. Add unresolved-symmetry fixtures that require real branching. Include at least:
    - two symmetric twins;
    - three symmetric anonymous entities;
    - a tractable high-symmetry case with multiple leaves.
    Prove the minimum payload matches the slow oracle.
19. Add the 1000 homogeneous-Action chain as a V3 search test. Required result:
    - root refinement discrete;
    - `search_nodes = 1`;
    - `individualisations = 0`;
    - `encoded_leaves = 1`;
    - exact repeated/metamorphic V3 payload/digest stability.
    No V2 factorial oracle/baseline is required for this 1000-Action case.
20. Record deterministic search statistics at minimum:
    - `refinement_relation_visits`;
    - `search_nodes`;
    - `individualisations`;
    - `encoded_leaves`;
    - `max_depth`.
21. Invalid Core must fail closed through the existing validation/model path with no payload/digest.
22. Return an abstract Rocket V3 canonicalized result containing the validated program association, frozen V2 payload, preimage and `tethers:v2:sha256:` digest. This is an experimental V3 engine API only; do not wire it into production.
23. Integrate only the new/extended V3 modules and focused tests into Dune. Rocket V2 production behaviour remains untouched.

## Search-state implementation boundary

Correctness comes before state optimisation in R3-3.

The accepted R3-2 partition is mutable and has no search undo API. R3-3 MAY rebuild a child partition from the immutable model and replay that child's individualisation path before refinement. This is intentionally allowed for the first correct search.

Do not add a per-node graph/model copy, external graph library, or broad partition redesign.

An in-place compact partition + undo trail is deferred until exact search parity is established.

## Relevant components

Authorised mutation is limited to:

- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-09-01-rocket-v3-r3-3-exact-search.md`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_model.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_model.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_model_test.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_encode.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_encode.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_search.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_search.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_search_test.ml`
- `tethers-0.1/engine-ocaml/bin/dune`

Read-only authorities:

- `tethers_core_rocket_v3_partition.ml/.mli`
- `tethers_core_rocket_v3_refine.ml/.mli`
- `tethers_core_canonical_v2_format.ml/.mli`
- `tethers_core_canonical_v2_reference.ml/.mli`
- `tethers_core_canonical_v2.ml/.mli`
- `tethers_core_canonical_v2_ir.ml/.mli`
- R3-0 inventory and accepted R3-1/R3-2 worker notes/tests.

## Frozen decisions and invariants

- Enc_V2 bytes and ProgramDigest V2 semantics are unchanged.
- The only winning certificate is exact frozen Enc_V2 bytes under unsigned-byte lexicographic comparison.
- Refinement colours, cell IDs, cell keys and search selectors are search machinery, not digest identity.
- Raw IDs are permitted only to bridge a leaf's vertex ordering back to the frozen typed label maps.
- Search selection/individualisation must not inspect raw IDs.
- Batch remains a separate Batch family.
- Role block constraints are part of Λ(P), not an optimisation.
- Every leaf assignment must be a complete legal V2 assignment.
- No leaf may be skipped in R3-3 because of similarity, hash, prefix, orbit, automorphism or prior payload.
- No greedy assignment of a non-singleton cell.
- No assumption that a discrete refined partition is exact until Stage A byte parity proves the leaf mapping.
- No external dependency.
- No wall-clock decision.
- No V1 fallback.
- No production cutover.

## Acceptance criteria

1. Model identity lookups round-trip every anonymous Core occurrence to exactly its existing model vertex and do not change R3-1 structural evidence or R3-2 refinement results/statistics.
2. Search/refinement code contains no raw-ID-based selector, ordering or tie-break logic; raw-ID lookup usage is confined to leaf-label construction/tests.
3. The leaf encoder rejects a non-discrete or non-stable partition deterministically.
4. The leaf encoder produces a complete legal V2 `label_assignment` covering all six anonymous families.
5. Scoped-role intervals exactly match the frozen Program-first/template-by-canonical-template-label block rule.
6. Stage A root-discrete fixtures match the slow V2 oracle byte-for-byte and digest-for-digest.
7. Stage A renamed/reordered metamorphic variants produce identical payload/digest.
8. Any Stage A mismatch stops the task before full I/R search implementation.
9. Search visits every individualisation branch in the selected non-singleton cell and encodes every discrete leaf reached; no pruning exists.
10. At least three deterministic selector strategies produce identical final payload/preimage/digest across focused and generated tests.
11. Slow oracle, V2 baseline and V2 IR differential comparisons have zero accepted-case payload/digest mismatches.
12. Symmetric twin/three-way/high-symmetry fixtures require branching and match the oracle minimum exactly.
13. Search traversal/internal vertex/storage perturbation does not change final payload or digest.
14. 1000 homogeneous Actions produce one root search node, zero individualisations, one encoded leaf and stable V2 digest bytes across metamorphic variants.
15. Search statistics report refinement relation visits, nodes, individualisations, leaves and max depth deterministically for repeated same-selector runs.
16. Invalid Core returns deterministic validation error and no payload/preimage/digest.
17. Public V3 search API exposes no prefix pruning, automorphism pruning, component recursion, budget, production-routing or V1 fallback operation.
18. Existing R3-1 `214/214`, R3-2 `4807/4807`, V2 suites and existing 5,000-case V2 corpus remain green.
19. Focused R3-3 tests, `dune build @all`, `dune runtest --force`, `git diff --check` and packet checker pass.
20. Final diff contains only authorised paths, implementation/test checkpoint precedes closeout docs, local HEAD equals remote HEAD and worktree is clean.
21. Leaf encoding delegates to the existing frozen `Tethers_core_canonical_v2_format.encode_program`; no duplicate/reimplemented Enc_V2 encoder or alternate leaf-label emission path exists.
22. Individualisation is branch-local search state only: it adds the explicit artificial distinction required for that child, then ordinary R3-2 refinement runs without mutating Core meaning or R3-1 model construction/structural evidence.
23. Dune integration registers only the authorised Rocket V3 modules/focused tests, and regression evidence confirms Rocket V2 production behaviour remains unchanged.

## Required verification

- Use a fresh dedicated worktree tracking `origin/feature/rocket-v3-r3-3-exact-search`.
- Read `AGENTS.md`, OCaml guide, complete packet, R3-0 inventory, R3-1/R3-2 public interfaces and frozen V2 format/oracle role enumeration before mutation.
- Confirm exact base `21bb7442fa9f8442db98e193eb4954096f356678`, branch, READY state and clean initial worktree.
- Run `pwsh -NoProfile -File scripts/check-dev-tools.ps1`.
- Run packet checker and require `control-v1/READY`.
- Verify the exact authorised OCaml switch.
- Implement model lookup bridge + Stage-A leaf encoder/tests first.
- Run Stage-A focused tests and oracle differential.
- If any Stage-A valid-case mismatch occurs, STOP. Do not proceed to search.
- Only after Stage A passes, implement unpruned I/R search and selector tests.
- Run focused R3-3 test executable and record exact check count and key search stats.
- Run `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune build @all`.
- Run `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune runtest --force`.
- Run `git diff --check`.
- Inspect full base-to-HEAD diff and prove only authorised paths changed.
- Commit implementation/tests and capture exact 40-character implementation checkpoint.
- Write worker note from evidence; mark packet COMPLETE. No implementation/test mutation after checkpoint.
- Run packet checker and require `control-v1/COMPLETE`.
- Push normally.
- Confirm local HEAD == remote HEAD and clean worktree.
- Report evidence and STOP.

## Forbidden changes

- No edit to R3-2 partition/refinement modules.
- No Core/validator/lowerer change.
- No change to frozen V2 format, V2 oracle, V2 production or Rocket V2 IR.
- No prefix-byte pruning.
- No automorphism/orbit pruning.
- No component decomposition/recursion.
- No duplicate-payload memo pruning.
- No branch-and-bound.
- No search budget/pre-admission factorial gate.
- No wall-clock timeout.
- No greedy forced labels from refinement colour/cell key.
- No new dependency.
- No production planner/wire/Rust-host integration.
- No V1 fallback.
- No release/version work.
- Do not begin R3-4 automatically.

## Stop conditions

- Stage-A discrete leaf mapping differs from the slow oracle on any valid accepted fixture.
- A legal V2 scoped-role block assignment cannot be derived from the V3 leaf without changing frozen semantics.
- Exact I/R result differs from oracle/baseline on any accepted differential case.
- Selector strategies produce different final payloads/digests for the same valid program.
- Search correctness requires skipping branches or inventing an unproved pruning rule.
- The accepted R3-1 model lacks identity information that cannot be supplied by the narrow lookup bridge without changing semantic construction.
- Work requires editing R3-2, frozen V2, Core, adding dependencies or production integration.
- Two materially similar failures recur without a new diagnosis.
- Checkout/branch/base/packet state differs after fetch.

## Expected pre-existing changes

- `tethers-0.1/engine-ocaml/bin/dune`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_model.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_model.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_encode.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_encode.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_search_test.ml`
