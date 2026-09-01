# Rocket V3 R3-0: Semantic Relation Inventory

Status: IN_PROGRESS (architectural correction applied)

Owner: Codex

Risk: Red

Task base: `5a1b461dcb95852681f269cd13a63a1e80695795`

READY packet HEAD: `3153dc698ffbb98793e69655f28c0fa80d17ca62`

Canonical authority: frozen `Enc_V2`; Rocket V3 is a new complete search model, not a format change.

## 1. Result and scope

This inventory covers the identity-bearing references in the validated Core model and every anonymous-label lookup/emission in the frozen V2 encoder that can affect canonical bytes.

### Architectural correction recorded

The task packet was corrected after the initial evidence pass. `origin_site` is a Core structural sum type, not a promise that every constructor is canonically an Origin. `Anchor_origin`, `Action_origin`, and `Together_origin` carry `origin_id` and produce Origin-family vertices. `Batch_site` carries `batch_id`, has no `origin_id`, is returned as `None` by `origin_id_of_site`, is excluded by `collect_origins`, is handled by `collect_batches`, and is encoded through `BatchMap` (`tethers_core.ml:124-164`; `tethers_core_canonical_v2_format.ml:162-180, 234-245, 347-355`). There is no separate canonical family or constructor named `Batch_origin`.

The six-family taxonomy is therefore consistent and frozen:

| Family | Core representation | V3 treatment |
| --- | --- | --- |
| Origin | `origin_site` variants (`Anchor_origin`, `Action_origin`, `Together_origin`) | Anonymous vertex family; variant is a discriminator/scalar class. |
| Fact | `fact` | Anonymous vertex family. |
| Branch | `branch` | Anonymous vertex family. |
| Batch | the identity-bearing `batch_id` carried by `Batch_site` | Anonymous vertex family; `Batch_site` is structurally an `origin_site` constructor but canonically Batch-family only. |
| ItemTemplate | `item_template` | Anonymous vertex family. |
| ScopedRole | `role_id` qualified by program/template scope | Anonymous vertex family; scope qualification is structural, not a seventh family. |

The V2 specification’s unqualified `Role` family is the packet’s `ScopedRole`, a role label qualified by `Program_scope` or `Item_template_scope`. The evidence also shows a material completeness gap in Rocket V2 refinement: the shared Enc_V2 encoder emits several references that `tethers_core_canonical_v2_ir.ml` does not expose as refinement relations. V2 also exposes only forward, family-specific signatures and currently tags Together members as `Rel_branch_subject`.

R3-0 changes documentation only. It does not implement the V3 model, refinement, partition state, I/R search, label assignment, pruning, budgets, graph-library integration, or production cutover.

## 2. Evidence inspected

All paths below are relative to the repository root `D:/The Next Thing/Tethers Lang - Rocket V3 R3-0`.

| Evidence | Relevant locations |
| --- | --- |
| Core type definitions and public contract | `tethers-0.1/engine-ocaml/bin/tethers_core.ml:79-236`; `tethers-0.1/engine-ocaml/bin/tethers_core.mli:116-321` |
| Lowering of source references and control flow | `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer.ml:58, 211-272` |
| Core validation of identity references and invariants | `tethers-0.1/engine-ocaml/bin/tethers_core_validator.ml:217-382, 459-616, 623-775` |
| Frozen V2 labels and byte emission | `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_format.ml:1-485`; interface `..._format.mli:1-131` |
| V2 production search and budget admission | `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2.ml` |
| V2 slow reference oracle | `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_reference.ml` |
| V2 refinement graph and heuristics | `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir.ml:286-563, 751-1262` |
| Frozen format specification | `docs/review/lucy-c-b4s-canonical-v2/CANONICAL_FORMAT_V2_SPEC_DRAFT.md:126-317, 429-520, 1722-1841` |
| Accepted V2 production cutover context | `docs/worker-notes/2026-08-12-core-9c-production-cutover.md` |

The relevant test and corpus requirements are retained in the current task packet and are restated as exact R3-1 proofs in Section 8.

## 3. Identity and structural taxonomy

### 3.1 Anonymous identity-bearing entities

The following entities receive dense V3 vertex indices. Raw IDs are construction-time lookup keys only and must not enter refinement, search selection, canonical ordering, or emitted identity decisions.

| Family | Core ID / source | Enc_V2 label | Scope |
| --- | --- | --- | --- |
| Origin | `origin_id`; each `origin_site` variant | `OriginMap` | Program or item-template ownership. |
| Fact | `fact_id` | `FactMap` | Declared/owned by a site or otherwise program-visible through provenance. |
| Branch | `branch_id` | `BranchMap` | Program or item-template collection. |
| Batch | `batch_id` used by `Batch_site` | `BatchMap` | Program or item-template context; no Origin label is assigned. |
| ItemTemplate | `item_template_id` | `TemplateMap` | Program-level template identity. |
| ScopedRole | `role_id` plus `Program_scope` or `Item_template_scope(item_template_id)` | `ScopedRoleMap` | One global label space over scope-qualified roles, per the V2 format specification. |

`group_id`, `program_id`, `role_id` before scope qualification, schemas, capability IDs, event names, contract digests and other scalar identifiers are not additional anonymous families. Where they affect bytes, they are scalar descriptors on the owning vertex or relation; where they are neutral to Enc_V2 identity, they are not model identity inputs.

### 3.2 Structural sentinels and terminal atoms

| Concept | Status | Required V3 use |
| --- | --- | --- |
| `ProgramRoot` | New structural sentinel; not an anonymous family | Source of the entry-origin relation, so entry structure is visible without assigning a special Origin label. |
| `ProgramComplete` | Existing `control_target = Program_complete` terminal | Target of a normal success continuation. It is fixed structural data, not a labelable entity. |
| `ProgramScope` | New structural/scope concept; not an anonymous family | Qualifies program roles and program-owned relations. It may be represented by a fixed scope node or fixed scope discriminator. |
| `EntryGuard` | Existing `fact_guard` structure | A guarded Fact relation with operator and expected scalar, not a new identity family. |
| `Stop` | Existing `branch_target = Stop` terminal atom | An outcome-discriminated terminal relation. It must not be silently conflated with `ProgramComplete`; Enc_V2 distinguishes branch Stop from a continuation target. |

`ProgramRoot` and `ProgramScope` are architectural concepts proposed by the V3 plan, not Core constructors to be invented in R3-0. R3-1 must choose their compact representation while preserving the distinctions above.

## 4. Relation notation and completeness rule

Every row in Section 5 identifies a directed semantic relation. V3 must store the forward relation and make the inverse observable through a compact reverse adjacency index. Physical duplication of objects is unnecessary; semantic direction is not.

`[M]` means relation multiplicity is part of the refinement signature. Collections whose duplicates are invalid must be validated as such; V3 must not collapse a valid repeated relationship into a set. `C` means the Core validator currently treats duplicate occurrence as invalid. `S` identifies scope/discriminator data required to interpret the endpoint. `T` identifies a structural sentinel or terminal atom.

The central rule is:

> If changing an anonymous reference can change Enc_V2 bytes, the V3 model must be able to see that reference.

The model must preserve semantic relation type, direction, endpoint family, discriminator, scope and multiplicity. Storage order is never a semantic tie-breaker.

## 5. Complete semantic relation inventory

The following table is the R3-0 inventory. “V2 refinement status” distinguishes the shared encoder/oracle from the older Rocket V2 refinement IR. `Encoder covered` means Enc_V2 emits the reference. `IR covered` means the V2 refinement model exposes a corresponding typed relation. `Partial` means only a narrower or one-sided relationship is visible.

| ID | Core source/location; owner → referenced family | Enc_V2 lookup / emission | Proposed V3 relation; inverse; discriminator | Multiplicity / scope / sentinel | V2 refinement status | Required R3-1 proof/test |
| --- | --- | --- | --- | --- | --- | --- |
| R01 | `program.entry_origin`, `tethers_core.ml:228-230`; Program → Origin | `encode_program`, `format.ml:474`, `lookup_origin` | `Rel_root_entry_origin`: `ProgramRoot → Origin`; inverse `Rel_origin_entry_root`; discriminator `Entry` | Optional singleton; program scope; `ProgramRoot` | Encoder covered; IR absent, with a V2 heuristic `entry_origin_minimal_label` | Rename and storage-permute programs while preserving entry origin; assert the edge and endpoint are unchanged. |
| R02 | `success_continuation.from_origin`, `ml:174-177`, lowerer `211-231`; Origin → Origin | `encode_program`, `format.ml:441-451`; origin lookups on both endpoints | `Rel_success_next`: source Origin → target Origin; inverse `Rel_success_prev`; discriminator `Success` | Representation collection; duplicate source is invalid; program scope; `[M]` after validation | Encoder covered; IR absent | Build a 1/10/50/100/250/500/1000-origin chain; assert all edges and inverse edges, with no identity dependence on list order. |
| R03 | `success_continuation.target = Program_complete`; `ml:170-177`; Origin → terminal | `encode_program`, `format.ml:443-451` emits terminal tag | `Rel_success_complete`: Origin → `ProgramComplete`; inverse terminal incidence; discriminator `Program_complete` | At most one target per source; terminal is fixed `T` | Encoder covered; IR absent | Assert a chain’s final target is structurally complete and is distinct from branch `Stop`. |
| R04 | `Anchor_origin.declared_facts`, `ml:124-130`; Anchor Origin → Fact | `encode_origin_site`, `format.ml:306-310`, fact labels | `Rel_origin_declared_fact`: Origin → Fact; inverse `Rel_fact_declared_by_origin`; discriminator `Anchor_declared` | Representation list sorted by Fact label; duplicate policy follows Core validation; owning program/template scope; `[M]` | Encoder covered; IR partial as `Rel_origin_to_fact_declared` | Permute declared-fact storage and test endpoint/multiplicity preservation and byte parity. |
| R05 | `Action_origin.declared_facts`, `ml:132-141`; Action Origin → Fact | `encode_origin_site`, `format.ml:317-327`, fact labels | Same relation family with discriminator `Action_declared`; inverse `Rel_fact_declared_by_origin` | Same as R04; action-origin scope | Encoder covered; IR partial as `Rel_origin_to_fact_declared` | Distinguish Anchor and Action by Origin variant scalar class while preserving the common relation type. |
| R06 | `Batch_site.aggregate_facts`, `ml:148-162`; Batch → Fact | `encode_origin_site`, `format.ml:347-355`, fact labels | `Rel_batch_aggregate_fact`: Batch → Fact; inverse `Rel_fact_aggregated_by_batch`; discriminator `Aggregate` | Representation list; multiplicity retained; program/template scope | Encoder covered; IR partial as `Rel_origin_to_fact_aggregate` on the site, with no separate Batch endpoint | Assert the Batch endpoint is retained and aggregate fact edits alter model input. |
| R07 | `fact.provenance = Origin_provenance(origin_id)`, `ml:79-85`; Fact → Origin | `encode_provenance`, `format.ml:266-272`, origin label | `Rel_fact_provenance_origin`: Fact → Origin; inverse `Rel_origin_provenance_fact`; discriminator `Origin_provenance` | Optional singleton provenance; scope must agree with Core fact visibility | Encoder covered; IR partial as `Rel_fact_to_origin` | Change only provenance origin; assert edge changes and raw-ID renames do not. |
| R08 | `fact.provenance = Role_proxy(role_id)`, `ml:79-85`; Fact → ScopedRole | `encode_provenance`, `format.ml:266-272`, `lookup_role_in_scope` | `Rel_fact_provenance_role`: Fact → ScopedRole; inverse `Rel_role_provenance_fact`; discriminator `Role_proxy`, with resolved scope | Optional singleton; role scope is mandatory; invalid/unresolved scope fails validation | Encoder covered; IR partial as `Rel_fact_to_role` but scope ownership is not structurally complete | Same numeric `role_id` in program and template scopes must resolve to distinct endpoints and labels. |
| R09 | `program.entry_guards : fact_guard list`, `ml:228`; EntryGuard → Fact | `encode_fact_guard`, `format.ml:278-281`; fact label plus operator/expected scalar | `Rel_entry_guard_fact`: `ProgramScope/EntryGuard → Fact`; inverse `Rel_fact_entry_guard`; discriminator `EntryGuard` | Representation collection; duplicate guard policy includes scalar guard payload; program scope; `EntryGuard` is existing structure | Encoder covered; IR absent | Change guarded Fact, operator, expected value and guard order independently; assert exact edge/scalar visibility and permutation invariance. |
| R10 | `Action_origin.inputs` binding `Fact_from_origin(fact_id, origin_id)`, `ml:105-110`; Action → Fact | `encode_binding`, `format.ml:283-287`, fact label | `Rel_action_input_fact`: Action Origin → Fact; inverse `Rel_fact_action_input`; discriminator `Fact_from_origin`, with input-name relation attribute | Input list sorted by input name then binding; multiplicity by distinct input occurrence; action scope | Encoder covered; IR absent | Add/remove/swap a Fact_from_origin binding; assert relation, input-name scalar and Enc_V2 oracle all change as expected. |
| R11 | Same `Fact_from_origin` binding; Action → Origin source parameter | `encode_binding`, `format.ml:285-287`, origin label | `Rel_action_input_origin`: Action Origin → Origin; inverse `Rel_origin_action_input`; discriminator `Fact_from_origin` | Same input occurrence; scope must resolve source origin | Encoder covered; IR absent | Keep Fact fixed and change only source Origin; assert the origin endpoint is observable independently of Fact endpoint. |
| R12 | `Action_origin.inputs` binding `Fact_through_role(fact_id, role_id)`, `ml:105-110`; Action → Fact | `encode_binding`, `format.ml:288-290`, fact label | `Rel_action_input_fact`: Action Origin → Fact; discriminator `Fact_through_role` | Same input collection semantics as R10; action scope | Encoder covered; IR absent | Distinguish Fact_from_origin and Fact_through_role with identical endpoint families but different relation discriminators and bytes. |
| R13 | Same `Fact_through_role` binding; Action → ScopedRole | `encode_binding`, `format.ml:288-290`, `lookup_role_in_scope` | `Rel_action_input_role`: Action Origin → ScopedRole; inverse `Rel_role_action_input`; discriminator `Fact_through_role`, scope-qualified | Same occurrence; role scope must be resolved from the owning action/program/template context | Encoder covered; IR absent; V2 role heuristics are not a complete relation | Use program/template roles with colliding raw role IDs; assert role scope is visible and endpoint identity is stable. |
| R14 | `Action_origin.inputs` binding `Anchor_value(origin_id, path)`, `ml:105-110`; Action → Origin | `encode_binding`, `format.ml:291-293`, origin label plus path sequence | `Rel_action_input_anchor`: Action Origin → Anchor Origin; inverse `Rel_anchor_action_input`; discriminator `Anchor_value`, path is a relation payload | Input occurrence; path is an ordered scalar sequence; Anchor endpoint must be an Anchor variant; `S` | Encoder covered; IR absent | Change anchor endpoint and path separately; assert both are represented, with path order preserved and storage order ignored. |
| R15 | `Action_origin.inputs` binding `Batch_item_context(item_template_id)`, `ml:105-110`; Action → ItemTemplate | `encode_binding`, `format.ml:294-295`, template label | `Rel_action_input_template`: Action Origin → ItemTemplate; inverse `Rel_template_action_input`; discriminator `Batch_item_context` | Input occurrence; owning template scope; `[M]` where repeated valid occurrences exist | Encoder covered; IR absent | Change only the item-template endpoint and assert the model/bytes change; preserve input-name ordering semantics. |
| R16 | `Together_origin.member_origin_ids`, `ml:142-147`; Together Origin → Origin | `encode_origin_site`, `format.ml:329-345`, origin labels sorted | `Rel_together_member`: Together Origin → member Origin; inverse `Rel_origin_member_of_together`; discriminator `Together_member` | Semantic set; duplicate members and self-members are invalid; owning scope; endpoint family Origin | Encoder covered; IR misclassified as `Rel_branch_subject` in V2 IR `494`; not a safe relation | Swap members, use repeated member shapes, and assert relation type and member multiplicity; a Together change must not appear as a Branch-subject change. |
| R17 | `branch.branch_subject`, `ml:179-183`; Branch → Origin | `encode_branch`, `format.ml:359-366`, origin label | `Rel_branch_subject`: Branch → Origin; inverse `Rel_origin_branch_subject`; discriminator `Subject` | Singleton; branch owning scope | Encoder covered; IR partial/covered as `Rel_branch_subject` | Change only subject; assert target edges remain unchanged and subject inverse is present. |
| R18 | `branch_target = Continue_to(origin_id)`, `ml:179-183`; Branch → Origin | `encode_branch`, `format.ml:363-368`, outcome rank and origin label | `Rel_branch_target`: Branch → Origin; inverse `Rel_origin_branch_target`; discriminator is exact `Success`, `Failure`, `Uncertain`, or `Cancelled` outcome | One target per outcome; duplicate outcomes invalid; branch scope; `[M]` by outcome tag | Encoder covered; IR partial as undifferentiated `Rel_branch_target` | Permute outcome list, change outcome tag, and change target independently; assert discriminator and endpoint are both visible. |
| R19 | `branch_target = Stop`, `ml:179-183`; Branch → terminal atom | `encode_branch`, `format.ml:363-368`, Stop tag | `Rel_branch_stop`: Branch → `Stop` terminal atom; inverse terminal incidence; outcome discriminator | One target per outcome; duplicate outcomes invalid; `Stop` is `T` and not an anonymous family | Encoder covered; IR only sees no entity for Stop | Change Continue_to to Stop for one outcome; assert structural difference and no accidental `ProgramComplete` equivalence. |
| R20 | `role.fact_contract = Role_fact_contract(fact_id list)`, `ml:193`; ScopedRole → Fact | `encode_role`, `format.ml:377-390`, fact labels sorted | `Rel_role_contract_fact`: ScopedRole → Fact; inverse `Rel_fact_contracted_by_role`; discriminator `Fact_contract` | Semantic set in the format specification; duplicate Fact IDs invalid, though validator gap is documented; role scope | Encoder covered; IR partial as `Rel_role_to_fact_contract` | Assert duplicate-contract rejection, contract permutation invariance, and endpoint changes in the model. |
| R21 | `role.scope = Item_template_scope(item_template_id)`, `ml:189-191`; ScopedRole → ItemTemplate | `encode_role_scope`, `format.ml:372-375`, template label | `Rel_role_scope_template`: ScopedRole → ItemTemplate; inverse `Rel_template_scoped_role`; discriminator `Item_template_scope` | Singleton scope; role is not valid without scope; `S` | Encoder covered; IR absent as a relation; scope is handled indirectly in arrays/heuristics | Same role ID in two templates must produce separate scoped vertices; moving a role between scopes must be observable. |
| R22 | `role.scope = Program_scope`, `ml:189-191`; ScopedRole → ProgramScope | `encode_role_scope`, `format.ml:372-375`, fixed scope tag | `Rel_role_scope_program`: ScopedRole → `ProgramScope`; inverse `Rel_program_scope_role`; discriminator `Program_scope` | Singleton fixed scope; `ProgramScope` is `T/S`, not anonymous | Encoder covered; IR absent as a relation | Program role and template role with otherwise identical payloads must remain distinct. |
| R23 | `item_template.origin_sites`, `ml:200-205`; ItemTemplate → Origin variant | `encode_item_template`, `format.ml:397-416`, mixed origin/batch sort and origin labels | `Rel_template_origin_site`: ItemTemplate → Origin; inverse `Rel_origin_template`; discriminator `Anchor`, `Action`, or `Together` | Representation collection; variant discriminator; template scope; `[M]` | Encoder covered; IR partial as `Rel_template_to_origin` | Move/reorder origins between templates; assert owning template scope and variant remain visible. |
| R24 | `item_template.origin_sites` containing `Batch_site`, `ml:148-162, 200-205`; ItemTemplate → Batch | `encode_item_template`, `format.ml:404-414`, `lookup_batch` in mixed sort; Batch payload emission | `Rel_template_batch_site`: ItemTemplate → Batch; inverse `Rel_batch_template`; discriminator `Batch_site` | Representation collection; Batch identity is separate from the structural `origin_site` sum; template scope | Encoder covered; IR partially treats site as template-origin and does not expose the separate Batch endpoint | Assert changing only `batch_id` changes the Batch endpoint/label; no Origin endpoint is present. |
| R25 | `item_template.branches`, `ml:200-205`; ItemTemplate → Branch | `encode_item_template`, `format.ml:415`, branch labels | `Rel_template_branch`: ItemTemplate → Branch; inverse `Rel_branch_template` | Representation collection; template scope; `[M]` | Encoder covered; IR covered as `Rel_template_to_branch` | Template branch permutation and cross-template movement preserve exact endpoint ownership. |
| R26 | `item_template.roles`, `ml:200-205`; ItemTemplate → ScopedRole | `encode_item_template`, `format.ml:416`, scoped-role labels | `Rel_template_role`: ItemTemplate → ScopedRole; inverse `Rel_role_template`; discriminator `Template_role_membership` | Representation collection; scope is the containing template; `[M]` | Encoder covered; IR partial as `Rel_template_to_role`, but scope ownership is not fully relational | Assert role membership and role scope agree; reject or diagnose a role whose declared scope disagrees with its containing template. |
| R27 | `item_template.objective = Required_role(role_id)`, `ml:197-198`; ItemTemplate → ScopedRole | `encode_item_objective`, `format.ml:393-395`, `lookup_role_in_scope` | `Rel_template_objective_role`: ItemTemplate → ScopedRole; inverse `Rel_role_objective_template`; discriminator `Required_role` | Optional singleton objective; must resolve in template scope; `S` | Encoder covered; IR absent as objective relation | Change only objective role; assert endpoint and scope affect the model and bytes. |
| R28 | Program-level collections (`input_facts`, `origin_sites`, `branches`, `roles`, `item_templates`), `ml:224-236`; ProgramScope → family endpoints | `encode_program`, `format.ml:427-484`, family-specific label lookups and mixed sorting | `Rel_program_fact`, `Rel_program_origin_site`, `Rel_program_branch`, `Rel_program_role`, `Rel_program_template`; inverses for each; fixed `ProgramScope` owner discriminator | Collection semantics are field-specific; top-level scope is structural, not an anonymous family | Encoder covered; V2 IR stores family arrays but does not expose one complete structural ownership relation or inverses | Permute top-level collections and move a semantically owned entity between program/template scopes; assert only semantic changes affect the model. |
| R29 | `fact` declaration/visibility through origin/template/program scope, `ml:79-85, 124-205`; Fact → scope owner | Indirectly affects `Role_proxy` scope resolution and all fact label occurrences | `Rel_fact_scope_owner`: Fact → Origin/ItemTemplate/ProgramScope as appropriate; inverse owner incidence; discriminator `Declared`, `Aggregate`, or `Program_input` | Scope is mandatory for resolving cross-reference legality; multiplicity follows the owning collection | Encoder emits endpoints but V2 IR derives scopes rather than modelling complete ownership | Construct equal payload Facts in different scopes and assert distinct structural context; verify Role_proxy lookup uses this relation. |
| R30 | `origin_site` variant tag, `ml:124-162`; actual variants include `Batch_site` | `encode_origin_site`, `format.ml:304-355`, tags 0–3; `origin_id_of_site` `162-166` | Structural sum discriminator `Origin_kind = Anchor/Action/Together/Batch_site`. Only the first three variants produce Origin-family vertices; `Batch_site` produces a Batch-family vertex. | Exactly one Core variant; Batch has no Origin ID; scope applies | Encoder covered; V2 IR has variant-specific cases but not a complete typed relational signature | Assert the structural variant and the separate Batch identity are visible, with no synthetic Origin endpoint for `Batch_site`. |
| R31 | `batch_site.batch_id` and `item_template_id`, `ml:148-162`; Batch → ItemTemplate | `encode_origin_site`, `format.ml:347-355`, `lookup_batch` and `lookup_template` | `Rel_batch_template_context`: Batch → ItemTemplate; inverse `Rel_template_batch_context`; discriminator `Batch_item_context`/site context | Singleton item-template context; template scope; Batch remains separate family | Encoder covered; V2 IR sees template through site handling but does not retain separate Batch identity and inverse | Rename/reorder Batch IDs and change template context independently; assert both endpoints and labels remain distinct. |
| R32 | `control_target`, `branch_target`, provenance and binding constructors, `ml:79-110, 170-183`; typed union discriminators | Encoded tags in provenance, binding, continuation and branch fields | Typed relation discriminator is mandatory: `Origin_provenance`, `Role_proxy`, `Fact_from_origin`, `Fact_through_role`, `Anchor_value`, `Batch_item_context`, `Success`, `Failure`, `Uncertain`, `Cancelled`, `Continue_to`, `Stop`, `Program_complete` | Discriminator is scalar semantic data; endpoint family alone is insufficient; `T` applies to terminal atoms | V2 encoder covered; V2 IR collapses several constructor distinctions or omits them | Pairwise same-endpoint tests for every constructor collision; assert distinct relation tags and byte parity. |

### 5.1 Why both directions are required

The V2 IR relation set is only:

`Rel_fact_to_origin`, `Rel_fact_to_role`, `Rel_origin_to_fact_declared`, `Rel_origin_to_fact_aggregate`, `Rel_branch_subject`, `Rel_branch_target`, `Rel_role_to_fact_contract`, `Rel_template_to_origin`, `Rel_template_to_branch`, and `Rel_template_to_role` (`tethers_core_canonical_v2_ir.ml:286-295`).

That set is useful evidence, but it is not a complete relational model. It omits or under-specifies R01–R03, R09–R15, R19, R21–R22, R27–R31, relation discriminators, scope ownership and every inverse direction. V3’s reverse adjacency must be built from the same typed edge inventory rather than inferred from colour equality or from the order of an input list.

## 6. Enc_V2 anonymous-label lookup inventory

The format module is the byte authority. The following map makes sure no lookup site is missed by the model inventory.

| Label map / lookup | Encoder sites and semantic uses | V3 coverage requirement |
| --- | --- | --- |
| `OriginMap` / `lookup_origin` (`format.ml:121-124`) | Origin sort keys (`234-245`); provenance (`266-272`); bindings (`285-293`); Anchor/Action/Together/Batch site emission (`304-355`); branch subject/targets (`359-368`); program entry and continuation (`441-480`); mixed template-site sorting (`404-414`) | Every occurrence must resolve to an Origin vertex or a fixed terminal relation; variant, scope and endpoint relation must be visible. |
| `FactMap` / `lookup_fact` (`format.ml:126-129`) | Fact provenance/guard/binding fields; declared and aggregate facts; role contracts; program inputs and entry guards (`274-281`, `427-439`) | Every fact endpoint must be a Fact vertex, with provenance, owner/scope and relation discriminator retained. |
| `BranchMap` / `lookup_branch` (`format.ml:131-134`) | Branch emission (`359-368`); template branch sorting (`415`); program branch sorting (`482`) | Branch endpoint and its owning scope must be visible; subject, target and outcome relations remain separate. |
| `BatchMap` / `lookup_batch` (`format.ml:136-139`) | Mixed origin sort key (`243-245`); Batch-site encoding (`347-355`); template-site sorting (`404-414`) | Batch is a separate anonymous family from the Origin variant; both Batch identity and site variant must be modelled. |
| `TemplateMap` / `lookup_template` (`format.ml:141-144`) | Batch item context (`294-295`); Batch-site context (`352`); role scope (`375`); item objective (`393-395`); template sorting (`464`) | Every template endpoint must retain containing scope and relation purpose. |
| `ScopedRoleMap` / `lookup_scoped_role` (`format.ml:146-149`) | Role emission and template/program role lists (`386`, `411`, `456`); scoped sorting | Scoped role identity is one global map over scope-qualified keys; no raw role ID or unqualified role family may substitute. |
| `lookup_role_in_scope` (`format.ml:151-156`) | Fact `Role_proxy` provenance (`270-272`); `Fact_through_role` binding (`288-290`); item objective (`395`) | V3 must resolve role references through explicit scope relations/discriminators, not a hidden array index. |

The encoder also emits scalar-only data. These fields require deterministic scalar descriptors but do not create anonymous relation endpoints: `core_version`, `event_name`, `capability_id`, `contract_digest`, `execution_constraints`, `host_snapshot_key`, `core_scalar_type`, input names, Anchor paths, literal values, Together objective tags, Batch collection/traversal/objective values, branch outcome tags, role fulfilment and capability-contract scalar fields. `group_id` is intentionally neutral in Enc_V2 and must not be promoted to identity.

## 7. Rocket V2 coverage findings

### 7.1 Covered or partially covered

V2 refinement does expose parts of provenance, declared/aggregate facts, branch subject/Continue_to target, template membership and role contract relationships (`tethers_core_canonical_v2_ir.ml:457-563`). These relations must be retained as typed V3 edges, not discarded merely because V2 already used them.

### 7.2 Omitted or unsafe for completeness

The following are emitted by Enc_V2 but are absent from the V2 refinement relation signature or only handled by local heuristics:

1. Entry structure: `entry_origin` and its relation to a program root.
2. Normal success control flow: every `success_continuation` edge and the `Program_complete` terminal.
3. Action input references: `Fact_from_origin`, `Fact_through_role`, `Anchor_value`, and `Batch_item_context`, including their constructor discriminators.
4. Role scope ownership and objective references; V2 derives some scope facts in arrays and uses role heuristics, but does not expose the complete relation.
5. Program/template ownership and the separate Batch endpoint inside `Batch_site`.
6. Inverse relations, which are needed for propagation from a successor/reference back to its owner.
7. Branch `Stop` as a typed terminal distinction and exact outcome discriminator.
8. Complete multiplicity/cardinality semantics and invalid-duplicate assertions.

The V2 IR’s Together case (`tethers_core_canonical_v2_ir.ml:494`) uses `Rel_branch_subject` while iterating Together members. This is a concrete misclassification: a Together member is not a Branch subject. R3-1 must introduce a distinct `Rel_together_member` discriminator.

The V2 IR also includes `entry_origin_minimal_label`, dependency-closure checks, role fast paths and other encoder-sensitive heuristics (`..._ir.ml:751-946`). Those may improve V2 performance, but they are not evidence that the complete relationship is represented. The V3 baseline must expose the relation first and leave optimization to later tasks.

## 8. Bounded proposed R3-1 implementation surface

R3-1 should be limited to constructing and testing the complete immutable typed relational model. It must not start refinement or search.

### 8.1 Proposed files and responsibilities

| File | Bounded responsibility |
| --- | --- |
| `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_model.ml` | Translate validated Core into dense vertex arrays, typed forward edges, reverse adjacency, relation discriminators, scalar descriptors, scope markers and fixed terminal/sentinel representation. Build once; no search-state mutation. |
| `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_model.mli` | Expose only model construction, immutable inspection and deterministic evidence/statistics needed by tests. Do not expose label assignment or canonical output. |
| One narrowly scoped model test module under the existing OCaml test layout | Execute the proofs below against hand-built and generated Core values. Reuse existing builders/oracles; do not add a V3 production route. |
| Existing build metadata, only if required to compile the two model files/tests | Minimal additive registration. Any build-file change requires a later packet explicitly authorizing it; it is not authorized by R3-0. |

The table is a proposal for the next packet, not an authorization to edit these paths in R3-0. The only R3-0 implementation artifact is this document.

### 8.2 Exact proofs and tests required

1. **Family cardinality and classification.** Every Core occurrence maps to exactly one of Origin, Fact, Branch, Batch, ItemTemplate or scope-qualified ScopedRole. `Action_origin`, `Anchor_origin` and `Together_origin` are Origin-family identities; `Batch_site` is structurally an `origin_site` constructor but its `batch_id` is the separate Batch-family identity. No synthetic Origin identity is introduced.
2. **Raw-ID invariance.** Renaming every nominal ID, including colliding role IDs across scopes, leaves the typed model’s canonical structural description unchanged.
3. **Storage-order invariance.** Shuffle `origin_sites`, facts, branches, roles, templates, guards, inputs, continuations and all other representation collections. The model edge multiset and scalar descriptors remain equal.
4. **Forward/inverse duality.** For every edge, the reverse index contains exactly one corresponding inverse edge with the same relation ID, discriminator, multiplicity occurrence and endpoint scope. No inverse is inferred from a colour or hash.
5. **Reference coverage assertion.** A machine-checkable inventory maps each Enc_V2 anonymous lookup site in Section 6 to one model vertex endpoint or fixed terminal relation. The assertion fails if a new lookup site is not classified.
6. **Success-chain structure.** Generate chains of 1, 10, 50, 100, 250, 500 and 1000 Actions. The model contains the root edge, every success-next edge, every inverse success-prev edge and the final ProgramComplete edge. No search/refinement is run in this test.
7. **Entry and terminal distinctions.** Test `ProgramRoot`, `ProgramComplete` and branch `Stop` as distinct structural values. Entry and normal success completion must not be represented as anonymous labels.
8. **Input constructor coverage.** For each binding constructor, change only its anonymous endpoint and then only its discriminator/payload. Prove that `Fact_from_origin`, `Fact_through_role`, `Anchor_value` and `Batch_item_context` are independently visible; Anchor paths remain ordered scalar sequences.
9. **Together relation correctness.** A Together member produces `Rel_together_member`, never `Rel_branch_subject`; duplicate/self members are rejected according to Core validation. Member multiplicity is not silently set-collapsed.
10. **Branch outcome coverage.** For each of Success, Failure, Uncertain and Cancelled, test both `Continue_to` and `Stop`; change outcome and target independently. Exact discriminator and terminal behavior must be visible.
11. **Scope resolution.** Test program roles and template roles with identical raw IDs and identical payloads. Prove distinct ScopedRole endpoints, correct `Role_proxy`/`Fact_through_role` resolution and correct template/program scope relations.
12. **Multiplicity and invalid input.** Test all valid repeated relation occurrences permitted by Core, plus duplicate success sources, duplicate branch outcomes, duplicate Together members and duplicate role contract facts. Valid multiplicity is preserved; invalid structures are rejected before model construction or reported deterministically.
13. **Scalar/neutral boundary.** Prove that scalar payloads affect descriptors where Enc_V2 emits them, while neutral `program_id`, schema descriptions and `group_id` do not become anonymous identity vertices or ordering inputs.
14. **Deterministic construction.** Randomize internal insertion/index order and hash-map insertion order. Dense vertex numbering may differ internally, but the immutable model’s sorted structural evidence must be identical.
15. **V2 oracle bridge without production wiring.** For tractable values, build the model alongside the frozen V2 slow oracle and assert that every anonymous reference used by the oracle’s Enc_V2 output is covered. Do not make the model produce ProgramDigest in R3-1.

## 9. R3-0 non-goals and stop boundary

The following were deliberately not implemented or designed beyond the bounded relation surface:

- Rocket V3 production modules or production routing;
- ordered-partition refinement, 1-WL or smaller-half worklists;
- individualisation/refinement search or canonical label assignment;
- certified prefix pruning;
- automorphism/orbit pruning;
- component recursion;
- V3 search budgets or new resource counters;
- bliss, nauty or any other graph-library integration;
- changing Enc_V2 or ProgramDigest semantics;
- fallback to V1 or V2 as a production identity authority.

R3-0 must stop if later implementation evidence contradicts the six-family taxonomy, the frozen classifications, or the exact Enc_V2 lookup inventory. Such a contradiction requires an architectural finding and a new decision; it must not be resolved by silently adding a family or weakening a relation.

## 10. Evidence conclusion

Rocket V3 has a bounded, testable R3-1 starting point: a dense immutable typed relational model with forward and inverse edges, explicit endpoint families, constructor/outcome discriminators, multiplicity, scope and fixed structural terminals. The primary structural defect implicated by issue #5 is now represented directly: normal success continuation is an explicit Origin-to-Origin chain ending at `ProgramComplete`, rooted at `ProgramRoot`.

The exact next task is R3-1 model construction and its proofs. No R3-1 work was started in this task.
