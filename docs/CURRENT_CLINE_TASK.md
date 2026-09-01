# Rocket V3 — R3-1 Immutable Typed Relational Model

Control contract: `1`

Status: `READY`

Task colour: `Red`

Owner: `Codex`

Route: `Codex implementation in a fresh dedicated worktree; bounded OCaml model construction and tests only`

Base commit: `0fd316083e1b26c3564080dec16d62490116858c`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

OCaml toolchain contract: use this exact external directory switch with explicit `--switch`; run Dune against the current R3-1 worktree source tree. Do not create, copy, move, select globally, or substitute another installed switch. For repository scripts that invoke `opam` without `--switch`, set `OPAMSWITCH` only process-locally to this exact path.

Worker note: `docs/worker-notes/2026-09-01-rocket-v3-r3-1-model.md`

Related issue: `#5 — BUG: Rocket V2 factorial search on simple sequential Action chains`

Design authority: `docs/review/rocket-v3/R3_0_SEMANTIC_RELATION_INVENTORY.md`

Updated: 2026-09-01

## Objective

Implement the first production-quality Rocket V3 component: an immutable typed relational model of validated Tethers Core that exposes every anonymous identity-bearing relation required by frozen `Enc_V2`, in both forward and inverse directions, with exact relation discriminators, multiplicity, scope, scalar descriptors and fixed structural sentinels/terminals.

R3-1 builds the semantic graph only. It MUST NOT implement partition refinement, canonical colour assignment, individualisation/refinement search, canonical labels, Enc_V2 candidate emission, ProgramDigest production, pruning, automorphisms, component recursion, V3 search budgets or production cutover.

The model is a faithful structural input for later Rocket V3 phases, not a second canonicaliser.

## Relevant background and existing behaviour

R3-0 is accepted on `main` and freezes the complete relation inventory. The six anonymous identity families are:

1. `Origin`
2. `Fact`
3. `Branch`
4. `Batch`
5. `ItemTemplate`
6. `ScopedRole`

The Batch taxonomy is frozen:

- `Anchor_origin`, `Action_origin`, and `Together_origin` are structurally `origin_site` constructors and carry anonymous Origin-family identity.
- `Batch_site` is structurally also an `origin_site` constructor, but canonically it is the separate Batch family because it carries `batch_id`, is excluded from `collect_origins`, and is labelled through `BatchMap`.
- No synthetic Origin identity may be created for a Batch site.

R3-0 also established that Rocket V2 refinement omits material Enc_V2-visible relations, notably root/entry structure, success-continuation control flow, Action binding references, role scope/objective relations, complete ownership, inverse directions, exact branch terminal/outcome distinctions and the separate Batch endpoint. Its Together-member relation is also currently misclassified as `Rel_branch_subject`.

Rocket V3 initially changes how the frozen Enc_V2 minimum is found, not what Enc_V2 or ProgramDigest_V2 means.

## Required behaviour

1. Add an immutable Rocket V3 model module that validates a Core program before construction and returns deterministic validation failure without partial model output.
2. Represent every anonymous occurrence as exactly one vertex in one of the six frozen anonymous families. Raw IDs may be used only as construction-time lookup keys and MUST NOT affect structural evidence, ordering decisions or later canonical semantics.
3. Represent `ProgramRoot`, `ProgramScope`, `ProgramComplete`, and branch `Stop` as fixed structural concepts/terminals distinct from anonymous identity families. The compact physical representation may use fixed sentinel vertices or typed fixed endpoints, but the distinctions must be explicit and testable.
4. Implement every relation R01-R29 in the accepted R3-0 inventory, including exact constructor/outcome discriminators, input-name/path/scalar relation payload where Enc_V2 observes it, scope ownership and the corrected Batch endpoint.
5. Provide both forward and reverse adjacency for every semantic directed relation. Each reverse occurrence must correspond exactly to one forward occurrence with the same typed relation identity/discriminator/payload and multiplicity occurrence.
6. Preserve multiplicity exactly where Core permits repeated occurrences. Do not convert relation collections to sets unless Core validation/specification proves set semantics and invalid duplicates are rejected before model construction.
7. Preserve semantic scope explicitly: program-owned versus item-template-owned Origins/Facts/Branches/Batches/Roles, scope-qualified roles, role references, template objective references and Batch/template relationships must resolve to the correct typed endpoints.
8. Produce deterministic scalar descriptors for Enc_V2-visible non-anonymous data while excluding neutral/non-identity fields such as raw `program_id`, schema descriptions and `group_id` from anonymous identity or ordering input.
9. Expose a narrow read-only inspection/evidence API sufficient for tests to compare family counts, vertex kinds, scalar descriptors, edges, reverse edges and a deterministically sorted structural evidence form. Do not expose canonical labels, colours or digest/candidate APIs.
10. Add comprehensive model tests proving the exact R3-0 acceptance properties, including generated 1/10/50/100/250/500/1000 Action chains. These tests inspect graph structure only; they do not perform refinement/search.
11. Cross-check the model against frozen Enc_V2 lookup coverage so a future anonymous lookup/reference cannot silently appear without a corresponding model classification.
12. Integrate the model/test modules minimally into Dune. Do not wire the model into live evaluation, Rocket V2, planner, wire or Rust host.

## Relevant components

Authorised mutation is limited to:

- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-09-01-rocket-v3-r3-1-model.md`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_model.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_model.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_model_test.ml`
- `tethers-0.1/engine-ocaml/bin/dune`

Required read-only authorities include:

- `docs/review/rocket-v3/R3_0_SEMANTIC_RELATION_INVENTORY.md`
- `docs/review/lucy-c-b4s-canonical-v2/CANONICAL_FORMAT_V2_SPEC_DRAFT.md`
- `tethers_core.ml/.mli`
- `tethers_core_validator.ml/.mli`
- `tethers_core_lowerer.ml`
- `tethers_core_canonical_v2_format.ml/.mli`
- `tethers_core_canonical_v2_reference.ml`
- `tethers_core_canonical_v2.ml`
- `tethers_core_canonical_v2_ir.ml/.mli`
- existing V2 tests/builders/corpus helpers where reuse does not couple V3 correctness to Rocket V2 heuristics.

## Frozen decisions and invariants

- Enc_V2 bytes and `tethers:v2:sha256:` ProgramDigest semantics are unchanged.
- R3-0 is the relation-coverage authority. If implementation evidence contradicts it materially, STOP and report rather than silently changing the model.
- Anonymous families remain exactly Origin, Fact, Branch, Batch, ItemTemplate and ScopedRole unless a new Red architectural decision explicitly changes them.
- `Batch_site` is Batch-family identity only. No Batch-as-Origin shortcut.
- `ProgramRoot`, `ProgramScope`, `ProgramComplete` and branch `Stop` are structural/fixed, never anonymous canonical identities.
- Every semantic directed relation must be observable in both directions.
- Edge/relation multiplicity is preserved as occurrences/counts, not silently deduplicated sets.
- Relation types/discriminators are semantically significant. `Rel_together_member` is distinct from `Rel_branch_subject`.
- Branch outcome tags Success/Failure/Uncertain/Cancelled are distinct relation discriminators, and branch `Stop` is distinct from normal `ProgramComplete`.
- Scope is semantic structure. Program and template roles with identical raw role IDs are distinct ScopedRole endpoints.
- Raw IDs, raw vertex numbers, list/array storage order, hash iteration order and construction insertion order are not canonical meaning.
- Internal dense integer vertex IDs are implementation handles only.
- Model construction may use mutable builders internally, but the returned model is immutable. No search-state mutation belongs here.
- Prefer compact arrays/CSR-style adjacency over per-vertex hash tables where practical:
  `vertex_kind`, scalar descriptors, forward offsets/edges, reverse offsets/edges.
  Exact internal representation may vary if it preserves the packet proofs.
- No new external dependency.
- No V1 fallback.
- No production route or replacement of Rocket V2 in this task.

## Acceptance criteria

1. For valid Core, model construction succeeds and every anonymous occurrence maps to exactly one vertex in one frozen family. For invalid Core, construction fails deterministically with validator errors and exposes no partial model.
2. `Anchor_origin`, `Action_origin`, `Together_origin` map to Origin vertices; `Batch_site` maps only to Batch vertices. No synthetic Batch Origin exists.
3. `ProgramRoot`, `ProgramScope`, `ProgramComplete`, and branch `Stop` are structurally distinguishable and are not counted as anonymous family vertices.
4. Every R01-R29 relation from the R3-0 inventory has a typed forward occurrence and exact inverse occurrence, with matching discriminator/payload/multiplicity semantics.
5. Raw-ID renaming, including role-ID collisions across distinct scopes, leaves the model's deterministically sorted structural evidence unchanged.
6. Permuting all representational collections, including origin sites, facts, branches, roles, templates, guards, inputs and continuations, leaves structural evidence unchanged wherever frozen semantics declare order irrelevant; ordered Anchor paths remain observably ordered.
7. Success chains of 1, 10, 50, 100, 250, 500 and 1000 Actions contain one root-to-entry relation, every success-next relation, every inverse success-prev relation and the final ProgramComplete relation, with no refinement/search performed.
8. Each Action input constructor (`Fact_from_origin`, `Fact_through_role`, `Anchor_value`, `Batch_item_context`) exposes all anonymous endpoints and exact constructor/input payload distinctions required by Enc_V2.
9. Together membership uses a dedicated Together-member relation and never the Branch-subject relation. Validator-invalid duplicate/self membership fails before model construction.
10. Every branch outcome Success/Failure/Uncertain/Cancelled is distinguishable, each `Continue_to` endpoint is represented, and branch `Stop` remains distinct from ProgramComplete.
11. Program-scope and template-scope roles with identical raw IDs/payloads resolve to distinct ScopedRole vertices; Role_proxy, Fact_through_role, role contracts, membership and template objective resolve to the correct scope-qualified endpoint.
12. Valid repeated relation occurrences preserve multiplicity; validator-invalid duplicate continuations/outcomes/Together members/role-contract facts are rejected.
13. Scalar descriptors change when Enc_V2-visible scalar payload changes, while neutral `program_id`, schema descriptions and `group_id` do not become anonymous vertices or structural ordering inputs.
14. Randomised construction/insertion/internal numbering produces the same sorted structural evidence after normalisation. Tests must not assert raw internal vertex numbers as canonical values.
15. A machine-checkable coverage fixture/assertion represents every anonymous lookup/reference category from R3-0 Section 6 and fails loudly if the maintained coverage table and model relation taxonomy diverge.
16. For tractable generated programs, the V2 slow oracle may run alongside the model solely to prove all anonymous Enc_V2 references are represented; the V3 model does not emit or choose canonical bytes.
17. `dune build @all`, the focused R3-1 model tests, `dune runtest --force`, `git diff --check` and task-packet consistency all pass.
18. Final diff contains only authorised files, the implementation checkpoint is committed before closeout docs, remote HEAD equals local HEAD, and the worktree is clean.

## Required verification

- Create/use a fresh dedicated worktree tracking `origin/feature/rocket-v3-r3-1-model`.
- Read `AGENTS.md`, mandatory controls, the complete R3-0 inventory and this packet before mutation.
- Run `pwsh -NoProfile -File scripts/check-dev-tools.ps1`.
- Run `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` and require `control-v1/READY`.
- Confirm exact base `0fd316083e1b26c3564080dec16d62490116858c`, expected branch and clean worktree.
- Implement the model and focused tests only within authorised paths.
- Run focused model tests repeatedly while building the relation inventory.
- Run `dune build @all`.
- Run the focused R3-1 model test executable.
- Run `dune runtest --force`.
- Run `git diff --check`.
- Inspect complete base-to-HEAD diff and prove only authorised paths changed.
- Commit the production/test implementation checkpoint and record its exact full 40-character SHA.
- Write the worker note from actual evidence and mark the task `COMPLETE`; no production/test changes are allowed after the recorded implementation checkpoint.
- Run the packet checker again and require `control-v1/COMPLETE`.
- Push normally to `origin/feature/rocket-v3-r3-1-model`.
- Confirm local HEAD equals remote HEAD and worktree is clean.
- Report exact test counts/evidence and STOP.

## Forbidden changes

- No partition refinement, 1-WL, colour refinement, cell splitting or worklists.
- No individualisation/refinement search or canonical label assignment.
- No Enc_V2 candidate generation from the model.
- No canonical payload/preimage/ProgramDigest production.
- No prefix pruning, automorphism/orbit pruning or component recursion.
- No V3 search budgets/resource policy.
- No production adapter/planner/wire/Rust-host integration.
- No modification of Rocket V2 behaviour or frozen V2 format/oracles/vectors.
- No new dependency or graph-canonicalisation library.
- No Human Tether grammar/Core semantic redesign.
- No raw ID/insertion order used as semantic tie-breaker.
- No historical Rocket branch merge/cherry-pick/rebase/copy as implementation authority.
- Do not begin R3-2 automatically after completion.

## Stop conditions

- R3-0 cannot be implemented faithfully without changing one of the six anonymous families or frozen Enc_V2 semantics.
- A relation in R01-R29 cannot be represented bidirectionally without inventing semantic information not present in Core/spec.
- Scope resolution for a valid Core program is ambiguous under current authoritative semantics.
- Correct multiplicity conflicts with current validator/spec assumptions.
- A deterministic structural evidence form cannot be made invariant to raw IDs/storage/insertion order without performing canonical search.
- Two materially similar implementation/test failures recur without a new diagnosis.
- Work requires touching files outside the authorised set, adding dependencies or wiring V3 into production.
- Checkout/branch/base/packet state differs from the authorised task after fetching origin.

## Expected pre-existing changes

- `tethers-0.1/engine-ocaml/bin/dune`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_model.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_model.mli`
