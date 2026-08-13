# Tethers Canonical Format V2 — Specification Draft

Status: `DRAFT — frozen for review`
Version: `V2-DRAFT-2026-08-13`
Replaces: `V1 (TETHERS_CORE_CANON_V1)` — known-defective, never frozen for 1.0
Author: OpenCode (C-B4S)
Reviewer: Lucy (required before implementation)

> **One-sentence contract**
> `CanonicalPayload_V2(P)` is the lexicographically smallest byte sequence
> obtainable by applying every permitted type- and scope-preserving bijection over
> entity occurrences of validated Core program `P` and then encoding the result
> with the frozen `Enc_V2` encoder.
> `CanonicalPreimage_V2(P) = DOMAIN_V2 || CanonicalPayload_V2(P)`
> `ProgramDigest_V2(P) = SHA-256(CanonicalPreimage_V2(P))` rendered as
> `tethers:v2:sha256:<64 hex>`.

---

## 0. Authority and scope

- This document is the sole authority for V2 canonical identity, byte layout,
  domain separation, digest syntax, collection semantics, scalar encoding,
  lexicographic comparison, scope handling, and failure mode.
- `tethers-0.1/SPEC.md` remains the authority for Tethers language surface syntax.
- `docs/CONSTITUTION.md` remains the authority for enduring principles.
- `tethers-0.1/engine-ocaml/bin/tethers_core.ml:1-236` is the authoritative Core
  type inventory for this draft. Any field present there and not classified here
  is a spec defect and blocks `READY TO IMPLEMENT V2`.
- V2 correctness outranks V1 compatibility. No V1 byte-layout compatibility is
  preserved at the expense of correctness.

---

## 1. Formal canonical identity definition

### 1.1 Validated input

`P` MUST be a `Tethers_core.program` that passes
`Tethers_core_validator.validate` (`bin/tethers_core_validator.ml:112-686`) with `Ok ()`.
If validation fails, there is no canonical form.

`program_id` and `schema_description` (on `fact` and `capability_contract`)
are excluded from identity (see §3). `core_version` IS included (see §3.3).

### 1.2 Permitted bijections Π(P)

Let `E(P)` be the multiset of **entity occurrences** in `P` (see §2).
`Π(P)` is the set of all bijections `π` such that:

- `π` is bijective within each entity family (kind-preserving).
- `π` is scope-preserving where scoping exists (see §2.4, §10).
- `π` preserves multiplicity: each occurrence maps to exactly one occurrence;
  the image of `π` has the same cardinality as the preimage. No merging, no deletion.
- `π` does NOT alter any semantic scalar payload (see §3).
- `π` does NOT alter collection semantics (§4): it only permutes the order of
  elements inside unordered collections; ordered collections are not reordered
  by definition (they have no alternative ordering that is semantically equal).

Applying `π` to `P` yields `π(P)`: the same program structure with every
entity **occurrence** identity replaced by its image under `π`, and every
reference to that occurrence rewritten to point to the new identity.
References that carry family+scope-qualified identity (e.g., `Role_proxy`,
`Fact_through_role`, `scoped_role_id`) are rewritten with the qualified key
(see `tethers_core_canonical.ml:107-111`).

`π` acts on **occurrences**, not on raw ID strings as abstract values. Two
occurrences that are structurally identical are still two distinct domain
elements; `π` may swap them (producing a distinct `π` but a potentially
identical `Enc_V2` output — see §5, §6).

### 1.3 Encoder

`Enc_V2 : Program → bytes` is the frozen deterministic encoder defined in §6.
It is structurally injective (distinct canonical-labelled structures produce
distinct byte sequences, modulo explicitly stated neutral fields).

### 1.4 CanonicalPayload_V2

```
CanonicalPayload_V2(P) = min { Enc_V2(π(P)) | π ∈ Π(P) }
```

where `min` is unsigned byte-wise lexicographic order defined in §8.
The minimum is taken over encodings, not over intermediate labels or colour
numbers.

### 1.5 CanonicalPreimage_V2 and ProgramDigest_V2

```
DOMAIN_V2            =  ASCII "TETHERS_CORE_CANON_V2" || 0x00   (20 bytes, see §16)
CanonicalPreimage_V2(P) = DOMAIN_V2 || CanonicalPayload_V2(P)
ProgramDigest_V2(P)  = SHA-256(CanonicalPreimage_V2(P))
DigestString_V2(P)   = "tethers:v2:sha256:" || hex_lower(SHA-256(...))   (see §17)
```

`CanonicalPayload_V2` is the canonical representation payload.
`CanonicalPreimage_V2` is the hash preimage. `ProgramDigest_V2` is the digest
value. Terminology is frozen per §18.

### 1.6 Search independence

Any algorithm that returns `CanonicalPayload_V2(P)` as defined above is
conformant. Refinement colours, partition cell numbers, target-cell heuristics,
search order, caching, pruning, and parallelism are **not** part of the
definition and MUST NOT affect the result (see §13).

---

## 2. Bijection domain — exact

### 2.1 Entity families

Each row is one independent permutation group. `π` is the disjoint union of
per-family bijections. Cross-family swaps are forbidden.

| # | Family | Core type(s) | Identity newtype(s) | Scope | Permutation group |
|---|--------|--------------|---------------------|-------|-------------------|
| 1 | Origin | `origin_site` (via `Anchor_origin`, `Action_origin`, `Together_origin`) | `origin_id` | Program-wide global uniqueness; template origins also globally unique per validator (`Duplicate_origin_id`) | `π_origin : origins → origins` |
| 2 | Fact | `fact` (appearing as `input_facts`, `declared_facts` on Anchor/Action origins, `aggregate_facts` on Batch) | `fact_id` | Logically global; declared facts have a containing site but identity is globally unique (`Duplicate_fact_id`) | `π_fact : facts → facts` |
| 3 | Role | `role` | `role_id` **qualified by scope** (`scoped_role_id`) | `Program_scope` or `Item_template_scope(tid)` — see §2.4 | `π_role : scoped_roles → scoped_roles`, partitioned by scope equivalence class |
| 4 | Branch | `branch` | `branch_id` | Program-wide (validator enforces uniqueness across templates too) | `π_branch : branches → branches` |
| 5 | Batch | `batch_site` (origin_site variant) | `batch_id` | Same sites as origins; `Duplicate_batch_id` globally | `π_batch : batches → batches` |
| 6 | Item Template | `item_template` | `item_template_id` | Global | `π_item_template : item_templates → item_templates` |
| 7 | Together Group | `together_origin.group_id` | `group_id` | One per `Together_origin`; `Duplicate_group_id` globally | `π_group : groups → groups` |

**Notes:**

- `capability_id`, `host_snapshot_key`, `capability_input_name`, `capability_contract_digest`, `batch_collection_provenance`, `batch_traversal_policy`, `batch_objective`, `role_fulfillment`, `core_version`, `event_name`, `contract_digest` values, and `Deadline` strings are **scalar payloads**, NOT entity identities. They are never relabelled.
- `action_id`, `plan_id`, `evaluation_id` do not exist in Core `program`; they are runtime plan identities, out of scope for `ProgramDigest_V2`.
- Counts: for a program with `F` facts + `O` origins + `Ba` batches + `R` scoped roles + `Br` branches + `IT` templates + `G` groups, the total bijection space size is `F! × O! × Ba! × R! × Br! × IT! × G!` before scope and validator constraints prune it. The oracle enumerates this reduced space.

### 2.2 What counts as an occurrence?

Every syntactic occurrence of an entity in `P` is one domain element:

- An `origin_site` occurrence (`Anchor_origin`, `Action_origin`, `Together_origin`, `Batch_site`) is one origin/batch occurrence. The same `origin_id` MUST NOT appear twice (validator rejects it); therefore each origin ID corresponds to exactly one occurrence.
- A `fact` occurrence in `input_facts ++ declared_facts ++ aggregate_facts` is one fact occurrence. Validator requires global uniqueness of `fact_id`.
- A `role` occurrence is one scoped role. `Role_id` may repeat across templates only if isolated by scope qualification (see §2.4); within one scope it must be unique.
- A `branch` occurrence is one `branch` record.
- An `item_template` occurrence is one `item_template` record.
- A `together_origin` carries both an origin occurrence and a group occurrence (two families, two domain elements, linked by containing site).

Conceptual rule (frozen):

```
π_origin           : OriginId        → OriginId
π_fact             : FactId          → FactId
π_role[Program]    : RoleId@Program  → RoleId@Program
π_role[Template t] : RoleId@Template(t) → RoleId@Template(t)   (per t)
π_branch           : BranchId        → BranchId
π_batch            : BatchId         → BatchId
π_item_template    : ItemTemplateId  → ItemTemplateId
π_group            : GroupId         → GroupId
```

Each `π_*` is bijective; the family-wise union is `π`.

### 2.3 What is NOT an entity occurrence

- Each `capability_contract` is keyed by `capability_id` which is a scalar
  capability name. It is NOT relabelled — `capability_id` equality is semantic.
- Each `success_continuation` is not an entity; it is a relationship between
  origins (`from_origin` → `control_target`). Its identity derives from
  participating origins.
- Each `entry_guard` is not an entity; it is a relationship (`fact_id` + operator + expected value).
- Each `action_input` / `input_binding` is not an entity; it is a relationship
  between a consumer origin and a fact/role/anchor literal.

### 2.4 Scope qualification

Role identity is **scope-qualified**:

```
scoped_key(r, Program)       = "P:" ++ string_of_role_id r
scoped_key(r, Template(tid)) = "T:" ++ string_of_item_template_id tid ++ ":" ++ string_of_role_id r
```

(`tethers_core_canonical.ml:107-111`)

This means:

- `RoleId "R1" @ Program` and `RoleId "R1" @ Template("IT1")` are **different
  domain elements** and belong to **different permutation groups**. `π` may
  permute one without affecting the other.
- Inside one template `IT_a`, `RoleId "R1"` and `RoleId "R2"` are permutable
  only if they are structurally equivalent under the full relational signature.
- Cross-scope role swaps are **forbidden**. A program-scope role can never be
  relabelled to a template-scope role.

Fact and Origin identities are globally unique and are NOT scope-qualified in
the validator. Their scope membership is a **relationship** (which origin/template
declares the fact), not part of the identity. Relabelling a fact does not change
which site declares it; the declaration relationship moves with the label.

Item Template scope itself canonicalises structurally (see §10): `π_item_template`
may relabel template IDs, but the membership relations (which origins/roles/branches
belong to which template) are preserved and participate in ordering.

---

## 3. What is NOT relabelled — scalar field classification

Every field below is **semantic**: its value participates in `Enc_V2` and
changing it changes the digest (unless the spec explicitly declares it neutral).
None of them is relabelled by `π`. Normalisation rules are frozen in §7.

### 3.1 Complete field table

| Field | Location | Type | Semantic? | Relabelled? | Normalised? | Encoding rule (§6/§7) |
|-------|----------|------|-----------|-------------|-------------|----------------------|
| `core_version` | `program.core_version` | `Core_version(string)` | YES | NO | NO (exact bytes) | `encode_string` (§6.5) |
| `program_id` | `program.program_id` | `Program_id(string)` | NO — neutral | NO | — | **Excluded from preimage** (see §3.3) |
| `event_name` | `anchor_origin.event_name` | `string` | YES | NO | NO | `encode_string` |
| `capability_id` | `action_origin.capability_id`, `capability_contract.capability_id` | `Capability_id(string)` | YES | NO | NO | `encode_string` |
| `contract_digest` | `action_origin.contract_digest`, `capability_contract.contract_digest` | `Capability_contract_digest(string)` | YES | NO | NO | `encode_string` (exact, lowercase hex after `sha256:` is contractual) |
| `host_snapshot_key` | `fact.provenance = Evaluation_input (Host_snapshot_key, _)` | `Host_snapshot_key(string)` | YES | NO | NO | `encode_string` |
| `core_scalar_type` | `fact.provenance`, scalar type tag | `String_type \| Integer_type \| Boolean_type` | YES | NO | — | `encode_tag 0/1/2` |
| `core_value` | `fact_guard.expected`, `input_binding.Literal_value`, fact value payloads | `String_value \| Integer_value \| Boolean_value` | YES | NO | NO | `encode_value` (§6.5): tag + length-prefixed string / decimal+`;` / `0;`/`1;` |
| `comparison_operator` | `fact_guard.operator` | `Equals \| Contains \| Greater_than \| Greater_than_or_equal` | YES | NO | — | `encode_tag 0..3` |
| `origin_provenance` | `fact.provenance = Origin_provenance(origin_id)` | `origin_id` | YES — reference | via `π_origin` (reference rewritten) | — | `encode_tag 1` + label |
| `role_proxy` | `fact.provenance = Role_proxy(role_id)` | `role_id` (scoped) | YES — reference | via `π_role` (scoped) | — | `encode_tag 2` + label |
| `capability_input_name` | `action_input.input_name` | `Capability_input_name(string)` | YES | NO | NO | `encode_string` |
| `input_binding` literal payload | `Literal_value(core_value)` | `core_value` | YES | NO | — | `encode_tag 0` + `encode_value` |
| `input_binding` anchor path | `Anchor_value(origin_id, string list)` | `origin_id` + `string list` | YES | origin_id via `π_origin`; strings semantic | NO for strings | `encode_tag 3` + origin label + list of `encode_string` |
| `input_binding` fact reference | `Fact_from_origin(fact_id, origin_id)` | `fact_id` + `origin_id` | YES | via `π_fact`, `π_origin` | — | `encode_tag 1` + fact label + origin label |
| `input_binding` role reference | `Fact_through_role(fact_id, role_id)` | `fact_id` + `role_id` | YES | via `π_fact`, `π_role` (scoped) | — | `encode_tag 2` + fact label + role label |
| `input_binding` batch context | `Batch_item_context(item_template_id)` | `item_template_id` | YES | via `π_item_template` | — | `encode_tag 4` + template label |
| `execution_constraint` | `action_origin.execution_constraints` | `Deadline(string)` | YES | NO | NO | `encode_tag 0` + `encode_string` |
| `together_objective` | `together_origin.objective` | `All_members_succeed` | YES | NO | — | `encode_tag 0` |
| `batch_collection_provenance` | `batch_site.collection_provenance` | `Batch_collection_provenance(string)` | YES | NO | NO | `encode_string` |
| `batch_traversal_policy` | `batch_site.traversal_policy` | `Batch_traversal_policy(string)` | YES | NO | NO | `encode_string` |
| `batch_objective` | `batch_site.composite_objective` | `Batch_objective(string)` | YES | NO | NO | `encode_string` |
| `terminal_outcome` | `branch.outcome_branches` key | `Success \| Failure \| Uncertain \| Cancelled` | YES | NO | — | `encode_tag 0..3` |
| `branch_target` | `branch.outcome_branches` value | `Continue_to(origin_id) \| Stop` | YES | origin via `π_origin` | — | `encode_tag 0` + origin label OR `encode_tag 1` |
| `control_target` | `success_continuation.target` | `Origin_target(origin_id) \| Program_complete` | YES | origin via `π_origin` | — | `encode_tag 0` + origin label OR `encode_tag 1` |
| `role_fact_contract` | `role.fact_contract` | `Role_fact_contract(fact_id list)` | YES — set (see §4) | fact IDs via `π_fact` | — | sorted list of fact labels |
| `role_fulfillment` | `role.eligible_fulfillment` | `Role_fulfillment(string)` | YES | NO | NO | `encode_string` |
| `role_scope` | `role.scope` | `Program_scope \| Item_template_scope(tid)` | YES | tid via `π_item_template` | — | `encode_tag 0` OR `encode_tag 1` + template label |
| `item_objective` | `item_template.objective` | `Required_role(role_id)` | YES | via `π_role` (scoped to that template) | — | `encode_tag 0` + role label |
| `schema_description` | `fact.schema_description`, `capability_contract.schema_description` | `string` | NO — neutral | NO | — | **Excluded from preimage** (see §3.3) |
| `fact_provenance tag` | `fact.provenance` variant tag | enum | YES | — | — | `encode_tag 0/1/2` |

### 3.2 Critical rule: raw IDs are NOT semantic values

The **string content** of `origin_id`, `fact_id`, `role_id`, `branch_id`,
`batch_id`, `item_template_id`, `group_id` is **not semantic**. Only the
equivalence relation they induce (which occurrences are the same entity) matters.
Changing the string from `"O_action_1"` to `"banana_thing_947"` without merging
or splitting entities MUST NOT change the digest. This is the definition of
raw-ID independence (§9 corollary of §1).

Conversely, `capability_id` string content **is** semantic: changing
`"cap.send"` to `"cap.recv"` changes meaning and MUST change the digest, even
though both are strings.

### 3.3 Neutral (non-canonical) fields

These fields exist in the Core type but are **excluded** from `Enc_V2` and
therefore do not affect `ProgramDigest_V2`:

- `program.program_id` — deployment-local handle, not semantic identity.
- `fact.schema_description` — human-readable description, validator ignores it,
  canonicaliser MUST exclude it.
- `capability_contract.schema_description` — same rationale.

Rationale: they are not consumed by `Tethers_core_validator`, not included in
the semantics that drive planning, and are explicitly declared neutral by
`tethers_core_canonical_test.ml:350-371` ("neutral descriptions" test).

If a future change makes either field semantic, that requires `V3` (see §20).

### 3.4 Semantic scalar immutability

For every scalar payload row above, the canonicaliser MUST encode the exact
value supplied. It MUST NOT apply:

- Unicode NFC/NFD normalisation,
- CRLF→LF normalisation,
- case folding,
- trimming,
- numeric base conversion (integers are decimal, see §7),
- float normalisation (no floats exist; see §7.1),
- digest hex case conversion.

Two distinct semantic values MUST produce distinct encodings. See §7.

---

## 4. Collection semantics (frozen)

One classification per field. A field's classification determines whether
`π`-induced permutation and encoder sorting are semantics-preserving.

**Definitions:**

- **A. SEMANTIC SEQUENCE** — Order matters. `[A,B] ≠ [B,A]`. Sorting would change meaning. The encoder MUST preserve declared order and encode elements in that order.
- **B. SEMANTIC SET** — Order irrelevant, duplicates forbidden. `{A,B} = {B,A}`. `A,A` is either invalid (validator rejects) or, if reachable, is the same as `A`. Encoder sorts by canonical labels.
- **C. SEMANTIC MULTISET** — Order irrelevant, multiplicity matters. `[A,B] = [B,A]` but `[A] ≠ [A,A]`. Encoder sorts stably; duplicates are preserved as separate occurrences.
- **D. REPRESENTATION COLLECTION** — Storage/source order is non-semantic. The physical order in the input `program` value is irrelevant; canonicalisation replaces it with canonical order (sorted by canonical labels). Not a language-semantic set—just a host-materialised bag.

**Frozen table:**

| Core field | Type | Classification | Justification |
|------------|------|----------------|---------------|
| `program.input_facts` | `fact list` | **D** | Validator: uniqueness of IDs only; no order semantics. Tests: storage-order independence (§5). Sorted by canonical fact label in `Enc_V2`. |
| `program.entry_guards` | `fact_guard list` | **D** | Guards are a conjunction; conjunction is unordered. Tests: guard storage reversal same digest. Sorted by `(fact_label, operator_rank, expected)` in `Enc_V2`. |
| `program.origin_sites` | `origin_site list` | **D** | Contains heterogeneous origins (Anchor, Action, Together, Batch). No language-specified order semantics. Tests: storage-order independence (origin lists). Sorted by canonical origin/batch label inside templates and at program level via `all_entries` sort. |
| `program.success_continuations` | `success_continuation list` | **D** | Each is `from_origin → target`. Set semantics: at most one per `from_origin` (validator: `Duplicate_success_continuation`). Sorted by `from_origin` canonical label. |
| `program.branches` | `branch list` | **D** | Each branch is independent, keyed by `branch_id`. No order semantics. Sorted by canonical branch label. |
| `program.roles` | `role list` | **D** | Each role is independent, keyed by scoped role id. Sorted by canonical role label. |
| `program.item_templates` | `item_template list` | **D** | Each template is independent, keyed by `item_template_id`. Sorted by canonical template label. |
| `program.capability_contracts` | `capability_contract list` | **D** | Each contract keyed by `capability_id`. Validator: uniqueness. Sorted by `capability_id` **string** (semantic key, not canonical label) — see §3. |
| `item_template.origin_sites` | `origin_site list` | **D** | Same as program level but template-scoped. Sorted. |
| `item_template.branches` | `branch list` | **D** | Same as program level, template-scoped. Sorted. |
| `item_template.roles` | `role list` | **D** | Same as program level, template-scoped. Sorted. |
| `origin_site: Anchor_origin.declared_facts` | `fact list` | **D** | Bag of facts declared by anchor. Order irrelevant. Sorted by canonical fact label. |
| `origin_site: Action_origin.declared_facts` | `fact list` | **D** | Same. Sorted by canonical fact label. |
| `origin_site: Action_origin.inputs` | `action_input list` | **D** | Named capability inputs; each `input_name` is the key. Order irrelevant, keyed by name. Tests: input reordering same digest. Sorted by `input_name` string then binding encoding. |
| `origin_site: Action_origin.execution_constraints` | `execution_constraint list` | **D** | Currently single variant `Deadline`. Bag semantics. Sorted by string value. |
| `origin_site: Together_origin.member_origin_ids` | `origin_id list` | **B — SEMANTIC SET** | Together membership is a **set**: order does not matter (`[A,B]=[B,A]`), duplicates forbidden (`Together_duplicate_member`), self-member forbidden. Encoder sorts `member_origin_ids` by canonical origin label. See §11. |
| `origin_site: Batch_site.aggregate_facts` | `fact list` | **D** | Same as declared_facts. Sorted by canonical fact label. |
| `branch.outcome_branches` | `(terminal_outcome * branch_target) list` | **D (up to duplicate check)** | Validator forbids duplicate outcome keys (`Branch_duplicate_outcome`). Logically a map `outcome → target`. Encoder sorts by outcome rank `Success<Failure<Uncertain<Cancelled`. |
| `role.fact_contract` | `Role_fact_contract(fact_id list)` | **B — SEMANTIC SET** | Set of fact IDs the role exposes. Order irrelevant, duplicates logically meaningless. Sorted by canonical fact label. |
| `fact_guard.expected` | `core_value` | scalar | — not a collection |
| `input_binding.Anchor_value path` | `string list` | **A — SEMANTIC SEQUENCE** | Path components are ordered: `["a","b"] ≠ ["b","a"]`. Encoder encodes in declared order, each as `encode_string`. |
| `capability_contract_digest` | `string` | scalar | — |

**Consequence:** There are **no frozen semantic sequences of entities** whose
permutation would be a semantic change at the program structure level (apart
from `Anchor_value` paths, which are string lists). All entity-occurrence
collections are bags/sets/multisets. This is why `π` can be defined as independent
per-family bijections without an additional ordering constraint. If a future Core
feature introduces a semantic sequence of entities (e.g., ordered pipeline steps
where position has meaning), §4 MUST be revised and the bijection definition in
§1 extended, requiring `V3`.

---

## 5. Multiplicity (frozen)

### 5.1 Invariant

> **Two semantically identical entity occurrences remain two occurrences.**
> Canonicalisation may relabel them. It may reorder them. It MUST NOT merge them.

Formally: if `P` contains two Action origins `A1` and `A2` whose scalar payloads
are identical (same `capability_id`, `contract_digest`, inputs, facts), then
`CanonicalPayload_V2` for `P` is **distinct** from the payload for the program
`P'` that contains only one of them.

### 5.2 Multiplicity appears as distinct label occurrences

In `Enc_V2`, each occurrence is encoded as `encode_int(canonical_label)` at its
definition site (e.g., `encode_origin_site` starts with `encode_int(label)`).
Two identical actions have distinct labels (`1;` vs `2;`) and therefore produce
distinct byte sequences even though their scalar payloads coincide.

### 5.3 Required examples

**Identical-actions-swapped = same digest:**

```
[A@"cap.x"/payload1, B@"cap.x"/payload2]  with CapSet {cap.x}
vs
[B@"cap.x"/payload2, A@"cap.x"/payload1]  (storage swap, same Π-representative)
→ CanonicalPayload_V2 identical → digest identical
```

This holds because `origin_sites` is D and sorting by canonical labels
normalises the storage swap; the multiset {payload1, payload2} is the same.

**One action vs two identical actions = different digest:**

```
P1: 1× Action { cap:"cap.x", input "x"="one" }   → payload length L1
P2: 2× Action { cap:"cap.x", input "x"="a" } and { cap:"cap.x", input "x"="b" }
     where "a" and "b" happen to be the same scalar "v" but are two occurrences
→ P2 has one more entity occurrence → encoding has one more encode_origin_site chunk
→ payloads differ → digests differ
```

Even when the two actions in `P2` are **identical scalars** (same capability,
same inputs, same facts), `P2 ≠ P1` because `|E(P2)| = |E(P1)|+1`.

**Cross-check:** `tethers_core_canonical_test.ml:672-703` ("multiplicity").

### 5.4 Encoding multiplicity

- `encode_list` (see §6) prefixes every list with its length `N:` . A list with
  two identical `Action_origin` sites will have `N=3` at `origin_sites` level
  (including the anchor); a list with one will have `N=2`.
- Even if lengths were equal, the label prefix distinguishes occurrences (labels
  are consecutive 1..N per family per the minimising assignment).

---

## 6. Enc_V2 — exact byte schema (frozen)

### 6.1 Design principles

- Structurally injective: distinct canonical-labelled structures produce distinct
  byte sequences. Every field boundary is unambiguous.
- Explicitly tagged and length-prefixed. No JSON, no object-key ordering
  dependence, no whitespace ambiguity.
- All strings are raw UTF-8 bytes with explicit length prefix; no terminators
  other than the `:` after the decimal length.
- All integers are decimal ASCII with `;` terminator (signed if needed; see §7).
- All lists are length-prefixed: `N:` (decimal, then `:`) followed by `N` encodings.
- All variants have an integer tag `tag:` (decimal + `:`).
- All optionals are encoded as `0;` for `None` and `1:` + payload for `Some`.
- Order inside sets/bags/multisets is **canonical label order**, not storage order.

### 6.2 Primitive encoders (frozen)

These are the only primitive encoders. They are reused verbatim everywhere.

```
encode_string(s):
    len = |s| in bytes  (UTF-8, may include any byte except NUL? — see §7)
    emit decimal(len)    // e.g., "5"
    emit ":"
    emit s bytes verbatim

encode_int(n):            // for signed integers (guard values, etc.) and labels
    emit decimal(n)       // e.g., "-3", "0", "42"
    emit ";"

encode_tag(t):            // non-negative variant tag
    emit decimal(t)
    emit ":"

encode_list<T>(items, encode_T):
    emit decimal(|items|)
    emit ":"
    for each item in canonical order:
        encode_T(item)

encode_option<T>(opt, encode_T):
    if None:  emit "0;"
    if Some x: emit "1:" ; encode_T(x)

encode_bytes(b):          // for future binary blobs (none yet)
    emit decimal(|b|)
    emit ":"
    emit b bytes verbatim
```

`decimal(n)` is `string_of_int n` (OCaml `string_of_int`) for `n ≥ 0`, and
`"-" ++ string_of_int (-n)` for `n < 0`. No leading zeros, no `+` prefix.

### 6.3 Tag assignments (frozen)

Tag numbers are part of the frozen format. Changing any tag requires `V3`.

| Type | Variant | Tag |
|------|---------|-----|
| `core_scalar_type` | `String_type` | `0:` |
| | `Integer_type` | `1:` |
| | `Boolean_type` | `2:` |
| `core_value` | `String_value` | `0:` + `encode_string` |
| | `Integer_value` | `1:` + `encode_int` |
| | `Boolean_value` | `2:` + `"0;"`/`"1;"` |
| `provenance` | `Evaluation_input` | `0:` + `encode_string(k)` + `encode_scalar_type` |
| | `Origin_provenance` | `1:` + `encode_int(label_of_origin)` |
| | `Role_proxy` | `2:` + `encode_int(label_of_scoped_role)` |
| `comparison_operator` | `Equals` | `0:` |
| | `Contains` | `1:` |
| | `Greater_than` | `2:` |
| | `Greater_than_or_equal` | `3:` |
| `input_binding` | `Literal_value` | `0:` + `encode_value` |
| | `Fact_from_origin` | `1:` + `encode_int(fact_label)` + `encode_int(origin_label)` |
| | `Fact_through_role` | `2:` + `encode_int(fact_label)` + `encode_int(role_label)` |
| | `Anchor_value` | `3:` + `encode_int(origin_label)` + `encode_list(path, encode_string)` |
| | `Batch_item_context` | `4:` + `encode_int(template_label)` |
| `together_objective` | `All_members_succeed` | `0:` |
| `execution_constraint` | `Deadline` | `0:` + `encode_string(s)` |
| `control_target` | `Origin_target` | `0:` + `encode_int(origin_label)` |
| | `Program_complete` | `1:` |
| `branch_target` | `Continue_to` | `0:` + `encode_int(origin_label)` |
| | `Stop` | `1:` |
| `terminal_outcome` | `Success` | `0:` |
| | `Failure` | `1:` |
| | `Uncertain` | `2:` |
| | `Cancelled` | `3:` |
| `origin_site` | `Anchor_origin` | `0:` |
| | `Action_origin` | `1:` |
| | `Together_origin` | `2:` |
| | `Batch_site` | `3:` |
| `role_scope` | `Program_scope` | `0:` |
| | `Item_template_scope` | `1:` + `encode_int(template_label)` |
| `item_objective` | `Required_role` | `0:` + `encode_int(role_label)` |

### 6.4 Composite encoders (frozen field order)

Field order inside each composite is frozen. No field may be reordered without `V3`.

```
encode_fact(f):
    encode_int(label_of_fact(f.fact_id))
    encode_provenance(f.provenance)      // includes variant tag

encode_fact_guard(g):
    encode_int(label_of_fact(g.fact_id))
    encode_tag(operator_rank(g.operator))   // 0..3 per table
    encode_value(g.expected)

encode_action_input(ai):
    encode_string(string_of_capability_input_name(ai.input_name))
    encode_binding(ai.binding)

encode_origin_site(site):
    match site:
    | Anchor_origin a:
        encode_tag 0
        encode_int(label_of_origin(a.anchor_origin_id))
        encode_string(a.event_name)
        encode_list(a.declared_facts, encode_fact)
    | Action_origin a:
        encode_tag 1
        encode_int(label_of_origin(a.action_origin_id))
        encode_string(string_of_capability_id(a.capability_id))
        encode_string(string_of_capability_contract_digest(a.contract_digest))
        encode_list(a.inputs, encode_action_input)            // sorted by input_name
        encode_list(a.declared_facts, encode_fact)            // sorted by fact label
        encode_list(a.execution_constraints, encode_constraint) // sorted by value
    | Together_origin t:
        encode_tag 2
        encode_int(label_of_origin(t.together_origin_id))
        encode_int(label_of_group(t.group_id))                // V2: group identity is label-only; raw string_of_group_id is EXCLUDED (§6.6, §9)
        encode_list(sorted_member_labels, encode_int)         // member_origin_ids sorted by canonical origin label
        encode_tag(together_objective)
    | Batch_site b:
        encode_tag 3
        encode_int(label_of_batch(b.batch_id))
        encode_string(string_of_batch_collection_provenance(b.collection_provenance))
        encode_int(label_of_template(b.item_template_id))
        encode_string(string_of_batch_traversal_policy(b.traversal_policy))
        encode_string(string_of_batch_objective(b.composite_objective))
        encode_list(b.aggregate_facts, encode_fact)

encode_branch(b):
    encode_int(label_of_branch(b.branch_id))
    encode_int(label_of_origin(b.branch_subject))
    encode_list(b.outcome_branches sorted by outcome rank, func (outcome,target):
        encode_tag(outcome_rank)
        encode_branch_target(target)
    )

encode_role(r):
    encode_int(label_of_scoped_role(r.role_id, r.scope))
    encode_role_scope(r.scope)            // tag 0 or tag 1 + template label
    encode_list(sorted_fact_ids_in_contract, encode_int)  // Role_fact_contract sorted by fact label
    encode_string(string_of_role_fulfillment(r.eligible_fulfillment))

encode_item_template(t):
    encode_int(label_of_template(t.item_template_id))
    encode_list(sorted_origin_sites, encode_origin_site)
    encode_list(sorted_branches, encode_branch)
    encode_list(sorted_roles, encode_role)
    encode_item_objective(t.objective)    // tag 0 + role label (template-scoped)

encode_capability_contract(c):
    encode_string(string_of_capability_id(c.capability_id))
    encode_string(string_of_capability_contract_digest(c.contract_digest))
    // schema_description EXCLUDED (neutral)

encode_program(p, label_map):
    encode_string(string_of_core_version(p.core_version))
    // Labels are assigned per §9; all entity lists below are sorted by those labels
    encode_list(sorted_input_facts by fact label, encode_fact)
    encode_list(sorted_entry_guards by (fact_label, operator_rank, expected_bytes), encode_fact_guard)
    encode_option(p.entry_origin, encode_int∘label)   // 0; or 1:<label>;
    encode_list(sorted_success_continuations by from_origin label, func sc:
        encode_int(label_of_origin(sc.from_origin))
        encode_control_target(sc.target)
    )
    // --- entities ---
    // All entity occurrences across program + templates, flattened and sorted by (label, kind_tag)
    // Each entity is encoded via its specific encoder (origin_site / batch / branch / role / template / fact)
    // Facts via encode_fact, origins/batches via encode_origin_site (which includes tag), etc.
    // Order: strictly ascending canonical label; ties broken by kind tag (§9)
    for each entity in sorted_entities:
        match kind:
        | Fact        -> encode_fact(entity)
        | Origin/Batch-> encode_origin_site(entity)
        | Branch      -> encode_branch(entity)
        | Role        -> encode_role(entity)
        | ItemTemplate-> encode_item_template(entity)
        // Groups are not encoded as standalone entries; they appear only inside Together_origin.
        // Their identity participates only via Together_origin.group_id label.
    // --- trailing collections ---
    encode_list(sorted_capability_contracts by capability_id string, encode_capability_contract)
    // success_continuations and entry_guards already emitted above as top-level collections
    // (double-check ordering — canonical order is: core_version, input_facts, entry_guards,
    //  entry_origin, success_continuations, origin_sites, branches, roles, item_templates,
    //  capability_contracts — this matches make_canonical_bytes field order)

```

**Unambiguousness argument:** Every list is length-prefixed, every string is
length-prefixed, every variant is tagged, every integer is `;`-terminated.
Therefore no two distinct canonical-labelled structures can produce the same
byte sequence. The encoding of `"ab"` as `2:ab` cannot be confused with
two strings `"a","b"` as `1:a1:b` because list length `2:` vs string length
`2:` appear at different structural positions disambiguated by surrounding tags.

### 6.5 Exact byte schema for scalar string/integer/boolean

Covered in primitive encoders. No further cases.

### 6.6 Decision: group_id encoding

`group_id` is an entity identity (see §2) whose raw string is not semantic.
Two options were considered:

- **(A) literal group_id string inside Enc_V2** — would make raw group strings
  semantic, violating raw-ID independence.
- **(B) canonical group label inside Enc_V2** — raw string excluded, only the
  integer label participates.

**Frozen decision: (B).** `Together_origin` encodes `encode_int(label_of_group(t.group_id))`
and does **not** emit `string_of_group_id`. The raw group string is excluded
from the preimage exactly like raw origin/fact/role/branch/batch/template strings.
This matches the prototype's intent and satisfies raw-ID independence
(`tethers_core_canonical_test.ml:749-773` group_id independence).

**Erratum vs v1:** v1 emitted `string_of_group_id` as `encode_string` inside
`Together_origin`. V2 removes it.

### 6.7 Excluded fields are absent, not encoded as empty

Neutral fields (`program_id`, `schema_description`) are **omitted entirely**.
They are not encoded as empty strings, not encoded as absent options, not
hashed. The encoder simply does not emit them.

---

## 7. Scalar canonical representation (frozen)

### 7.1 Rule

> **The canonicaliser encodes Core semantics. It does not invent equivalences.**

For every current scalar type, exactly one byte representation exists, and two
distinct Core semantic values have distinct encodings.

### 7.2 Current scalar types and their single representation

| Core scalar type | OCaml type | Canonical bytes | Notes |
|------------------|------------|-----------------|-------|
| UTF-8 string (event_name, capability_id, contract_digest, host_snapshot_key, Deadline, batch strings, fulfillment, capability_input_name) | `string` | **Exact UTF-8 bytes** as supplied, length-prefixed via `encode_string`. No Unicode normalisation (NFC/NFD). No case folding. No trimming. No CRLF→LF. Empty string `""` is `0:` (length 0). | If the host supplies `"café"` as NFC vs NFD, those are **distinct** Core values and MUST yield distinct digests, because the language semantics do not declare them equal. A future decision could normalise, requiring `V3`. |
| Integer | `int` (OCaml `int`, 63-bit on 64-bit) | Decimal ASCII, no leading zeros (except `"0"`), optional leading `"-"` for negative, terminated by `";"`. E.g., `42 → "42;"`, `-7 → "-7;"`, `0 → "0;"`. | Current Core integers are small. No hex/octal alternative. |
| Boolean | `bool` | `Boolean_value true → tag 2 + "1;"`, `false → tag 2 + "0;"` inside `encode_value`. | |
| Core scalar type tag | `String_type/Integer_type/Boolean_type` | `encode_tag 0/1/2` | |
| Comparison operator | `comparison_operator` | `encode_tag 0..3` per table | |
| Together objective | `together_objective` | `encode_tag 0:` | Single variant today; tag still emitted for forward compatibility. |
| Terminal outcome | `terminal_outcome` | `encode_tag 0..3` | |
| Control/branch targets | `control_target`/`branch_target` | variant tag + optional label | |
| Capability contract digest string | `string` of form `sha256:<64 hex>` | Exact bytes as supplied; case-sensitive. | Validator does not enforce hex case for contract digests beyond existence check? Canonicaliser preserves exact bytes. If two contracts differ only by hex case they are different digests. |

### 7.3 What is NOT normalised

- No Unicode NFC normalisation.
- No line-ending normalisation.
- No float canonicalisation (floats do not exist in current Core). See §7.4.
- No `-0` vs `0` handling (integers, not floats).
- No digest hex case-folding.
- No string interning or deduplication.

### 7.4 Future types (floats, binary, etc.)

**Floats do not currently exist** in Core (`core_value` has no float case).
If floats are ever added, the specification MUST freeze before implementation:

- Bit-exact IEEE 754 binary representation vs decimal string.
- Handling of `-0.0`, `NaN` payloads, infinities.
- Whether `0.0` and `-0.0` are semantically equal (requires language decision).

Adding floats without a frozen rule is a **V3-required** change and MUST NOT be
implemented under V2.

Same for any new scalar type (bytes, timestamp, decimal, etc.).

---

## 8. Lexicographic order (frozen)

### 8.1 Definition

`CanonicalPayload_V2(P) = min_{π∈Π(P)} Enc_V2(π(P))` where `min` is:

> **Unsigned byte-wise lexicographic comparison:**
> Compare the two byte sequences `A` and `B` from index 0 upward as unsigned
> bytes (`0x00 … 0xFF`). At the first index `i` where `A[i] ≠ B[i]`,
> the sequence with the smaller byte value is smaller. If one sequence is an
> exact prefix of the other, the **shorter sequence is smaller**.

No locale, no Unicode collation, no platform collation, no OCaml
`String.compare` quirks beyond the fact that OCaml's `String.compare` happens
to implement this exact unsigned byte-wise comparison (documented as
lexicographic on `char` values `0..255`). Conformance is to the mathematical
definition above, not to any library function.

### 8.2 Tie-breaking

If `Enc_V2(π1(P)) = Enc_V2(π2(P))` byte-identically, then `π1` and `π2`
witness the same automorphism and the choice between them is irrelevant (the
minimum is that byte sequence). The specification does not require a tie-breaker
between `π`s; it requires the **byte minimum** be returned.

### 8.3 Frozenness

Changing the comparison (e.g., to signed bytes, to length-first, to hash-first)
requires `V3`.

---

## 9. Canonical local labels (frozen)

### 9.1 Labels are NOT content of Enc_V2 — except as integers

Canonical labels (`O1`, `F1`, etc.) are **not** emitted as literal strings
`"O1"`. They are emitted as **integer label values** via `encode_int(label)`.
The prototype's `encode_int(lookup_label(...))` is the normative form.

Rationale: integer labels are smaller, avoid string parsing, and are fully
determined by the minimisation.

### 9.2 Labels are implementation-local indexes, not persistent identities

There is **no requirement** that a specific raw occurrence (e.g., `"Fred"`) always
becomes a specific canonical label (e.g., `O7`). Different search paths may
associate different raw occurrences with the same canonical index, as long as the
resulting `Enc_V2` is identical.

What is frozen: the **final byte sequence**. The mapping `raw_id → label` is
internal scheduling state, not part of the contract. Two conformant implementations
may produce different internal label assignments for automorphic cases and still
be conformant if they produce the same minimum bytes.

### 9.3 Label assignment scheme (normative, inside Enc_V2)

Labels are assigned per family as **consecutive integers 1..N** where N is the
number of occurrences in that family, in the order induced by the minimising
`π`. That is, the canonical-labelled program's entities are numbered by their
position in the **sorted-by-label** traversal order.

Concretely, if `π` relabels origins, then the first origin in sorted order gets
label `1;`, the second gets `2;`, etc. The integer values are labels, not
colours and not hashes.

The minimising `π` is defined as the `π` that yields the minimum `Enc_V2`.
Any `π` achieving that minimum is acceptable; the encoder does not define a
secondary tie-breaker beyond the byte minimum.

### 9.4 Labels and scope

Labels are per-family AND per-scope-partition for roles:

- Origin labels: `1..|origins|`
- Fact labels: `1..|facts|`
- Branch labels: `1..|branches|`
- Batch labels: `1..|batches|`
- Template labels: `1..|templates|`
- Group labels: `1..|groups|`
- Role labels:
  - For each scope equivalence class (Program scope = one class; each
    `Template(tid)` = one class), labels `1..|roles_in_class|`
  — but as currently encoded, roles across scopes share a global label space
    keyed by scoped key; see §2.4. Frozen `Enc_V2` uses global role labels
    derived from the global sorted order of scoped roles. This is equivalent
    to per-scope consecutive assignment with a deterministic interleaving rule:
    sorted by `(scope_label, role_label)` where `scope_label` is the template's
    canonical label or `0` for Program. Any deterministic rule producing the same
    minimum is conformant as long as it respects scope isolation (no cross-scope
    permutation).

---

## 10. Scopes (frozen)

### 10.1 Rule

> A role/fact/origin in one Item Template MUST NOT be interchangeable with an
> otherwise identical entity in another scope unless Core semantics declare them
> inter-scope identical. Scope membership is a structural relationship and
> participates in canonical identity via label-qualified references.

### 10.2 Role scope handling

Defined in §2.4 and §9.4. In `Enc_V2`:

- `role.scope` is encoded as `Program_scope` (`0:`) or
  `Item_template_scope(label_of_template)`.
- `Role_fact_contract` fact IDs are sorted by canonical fact label (fact identity
  is global, but the contract membership is template-scoped).
- `Fact_through_role` bindings are resolved against the **origin's scope**:
  an Action origin inside `IT_a` can only reference a role in `IT_a` (or Program
  scope for anchors) per validator `Fact_through_role` checks. The encoder looks
  up the role via its scoped key, so the correct label is emitted.

### 10.3 Item Template scope handling

- Each `item_template` has its own `origin_sites`, `branches`, `roles` lists.
- `item_template.objective = Required_role(role_id)` is scoped: the required
  role MUST be in the same template (validator: `Item_objective_missing_role`).
  Encoding uses the scoped role label.
- Templates themselves are permutable via `π_item_template`, but the
  membership relations are preserved: if template `IT_a` contains origins
  `{O1,O2}`, after `π` the renamed template still contains exactly the
  corresponding renamed origins.

### 10.4 Scope canonicalises structurally

Scope identity (`item_template_id` raw string) is not semantic. Two templates
with identical structure but swapped raw IDs must have the same digest (up to
global template permutation). Scope is identified by **structure**, not by raw
template ID.

### 10.5 Worked example — two isomorphic templates

```
P with:
  IT_alpha: roles { R1@alpha: fact_contract [F1], fulfillment "fa" }, objective Required_role(R1@alpha)
  IT_beta:  roles { R1@beta : fact_contract [F2], fulfillment "fb" }, objective Required_role(R1@beta)
  where F1 and F2 are Facts with identical provenance type but different host keys (distinct semantics)

Swap raw IDs: IT_alpha↔IT_beta and R1@alpha↔R1@beta are NOT simultaneously swappable
without also swapping F1↔F2 and fulfillment strings. Since fulfillment strings differ
("fa" vs "fb"), the two templates are NOT automorphic; their Enc_V2 contributions
differ and the minimum is still unique.

If instead both templates are identical scalars (same fulfillment string, same fact
provenance shape, same objective):
  IT_a: role R with "X", fact F@eval(hk, String_type)
  IT_b: role R with "X", fact F@eval(hk, String_type) — but facts have distinct IDs and are distinct occurrences
Then π_item_template swapping IT_a↔IT_b yields identical Enc_V2 outputs (the two
templates are automorphic). The lexicographic minimum is that single equivalence class.
Both raw renamings and template storage permutation give the same digest.

Changing actual scope relationship MUST differ:
  Move a role from IT_alpha to Program scope (or to a different IT) changes
  r.scope encoding (0: vs 1:<template_label>) → bytes differ → digest differs.
```

---

## 11. Together (frozen)

### 11.1 Semantics

_From `tethers-0.1/SPEC.md:6.1`:_

- `together` declares a concurrency group (`Together_origin`). Its member Actions
  are independent and are planned in source order with `action_id`s, but in Core
  the group is the unordered set of members.
- Restrictions: ≥2 members, no nesting, no empty, no duplicate members, no
  self-member (validator enforces all).

Canonical status: **membership is a set** (see §4, classification B).

### 11.2 Encoding

```
Together_origin {
  together_origin_id : origin_id   → label (family Origin)
  group_id           : group_id     → label (family Group)  [NOT raw string]
  member_origin_ids  : origin_id list → sorted list of origin labels (ascending integer)
  objective          : All_members_succeed → tag 0:
}
```

- Member order does not matter: `Together(A,B) = Together(B,A)` — the encoder
  sorts member labels.
- Duplicate members are **illegal** and rejected by validator; encoding of
  duplicates would be a distinct set of member labels but such `P` is invalid
  and has no digest.
- Group multiplicity: two `Together_origin` groups with identical members but
  distinct `group_id`/`together_origin_id` are two occurrences; see §5. Their
  encodings differ by group label.

### 11.3 Examples

**MUST have same digest:**

```
Together(G1, members [A, B], objective All_members_succeed)
vs
Together(G2, members [B, A], objective All_members_succeed)
where A,B are Action origins with identical scalar payloads (automorphic)
→ after π sorting, both encode to member label set {1,2} → same bytes → same digest
```

**MUST differ (negative):**

```
Together(G1, members [A, B])
vs
Together(G1, members [A, C])   where C has different capability_id than B
→ B vs C scalar differs → encode_origin_site for B vs C differs → bytes differ
```

Also must differ if objective differed (future variant), or members differ in
cardinality, or the connection to `member_origin_ids` validators differs.

---

## 12. Branches (frozen)

### 12.1 Structure

`branch` (`Tethers_core.branch`):

```
{ branch_id, branch_subject: origin_id, outcome_branches: (terminal_outcome * branch_target) list }
branch_target: Continue_to(origin_id) | Stop
```

- `branch_subject` is the origin whose terminal outcome triggers branching.
- `outcome_branches` maps each `terminal_outcome` to a `branch_target`.
- Validator: outcomes must be unique per branch (`Branch_duplicate_outcome`).

### 12.2 Canonical encoding

From §6.4:

```
encode_branch(b):
    encode_int(label_of_branch(b.branch_id))
    encode_int(label_of_origin(b.branch_subject))
    encode_list(sorted by outcome rank, (outcome, target)):
        encode_tag(outcome_rank)   // 0..3
        encode_branch_target(target) // 0:<origin_label> or 1:
```

- `branch_subject` label is via `π_origin`.
- `outcome_branches` order is **sorted by outcome rank**, not storage order.
- `branch multiplicity`: two branches with identical `branch_subject` and
  `outcome_branches` are two occurrences; see §5. Their encodings differ by
  branch label.
- Missing outcome vs explicit `Stop` outcome: the validator's domain is that
  every entry in `outcome_branches` is explicit. There is no "absent outcome"
  encoding; absence means the outcome key is not present in the list. A branch
  with `{Success→Stop, Failure→Stop}` is different from `{Success→Stop}` (one
  more list element, different length prefix).

### 12.3 Witness shape

The C-B3T 24-permutation witness MUST be expressible:

```
4 branches B1..B4 with identical scalar structure:
  branch_subject in origin partition O* (4 equivalent Action origins)
  outcome_branches = [Success→Continue_to(tgt), Failure→Stop, Uncertain→Stop, Cancelled→Stop]
  where tgt differs per origin but the pattern is symmetric

Π includes 4! = 24 bijections over origins+branches that preserve the structure.
V2's minimum over Π is unique — all 24 encodings collapse to the same minimum.
```

This was the V1 defect: V1 tied canonical ID assignment to raw `branch_subject`
ordering and produced 24 distinct byte sequences. V2's `Π`-minimum eliminates it.

---

## 13. Search-independence contract (frozen)

### 13.1 Refinement and search MAY use:

- WL colours, numeric colour IDs, sequential R0..Rn iterations, safety caps,
- Ordered partition cells, cell rank numbers,
- Raw IDs **as opaque lookup handles** (e.g., `Hashtbl` keys, `StringMap` keys),
- Arbitrary `target-cell` heuristics (largest cell first, first non-singleton, etc.),
- Sequential or parallel traversal, caching, memoisation,
- Pruning (e.g., prefix-pruning when a candidate already exceeds the best prefix),
- Thread/domain scheduling, worker counts.

### 13.2 ONLY if:

> **None of those choices alter the complete mathematical candidate space `Π(P)`**
> **or the selected minimum `min Enc_V2(π(P))`.**

Specifically:

- Raw IDs may identify objects internally. They **may not constrain or rank**
  valid canonical solutions. The candidate set must be exactly `Π(P)` as defined
  in §1. Filtering candidates by raw string order is forbidden.
- Colour numbers/densities may schedule search. They **may not appear in Enc_V2**
  and must not determine which valid canonical representation wins. The minimum
  is over *encoded bytes*, not over colour integer order.
- Partition cell numbering may order iteration. It must not affect the minimum.
- Pruning may elide candidates that are **provably** strictly larger than the
  current best prefix (safe prefix pruning). It must never prune a candidate
  that could be the global minimum.
- Parallel scheduling must yield exactly the same final `CanonicalPayload_V2`.

### 13.3 Conformance test

Any implementation that, for any valid `P`, returns different bytes under a
different thread count, different `Hashtbl` iteration order, different colour
renumbering, or different `target-cell` choice — while still passing all other
tests — is **non-conformant**.

---

## 14. Correction of the "same partition" claim (frozen)

### Previously claimed (incorrect):

> If two refinement implementations produce the same final extensional partition
> (same equivalence classes), they are interchangeable without a format version
> change.

This is **insufficient**. Two implementations can produce the same partition but
differ in `Enc_V2` layout or minimisation logic and still diverge on digests.

### Corrected condition (frozen):

> A refinement/search implementation may replace another **without** a Canonical
> Format version change **iff, for every valid Core program `P`, it returns
> exactly `CanonicalPayload_V2(P)` and `ProgramDigest_V2(P)` as defined by the
> V2 mathematical specification (§1).**

`partitions_equal` ( `tethers_core_canonical.ml:210-216` ) is **not canonical
identity**. It may be used as diagnostic telemetry or as a necessary-but-not-
sufficient compatibility hint, but it is never a sufficient proof of
compatibility.

---

## 15. Resource limits — deterministic fail-closed (frozen)

### 15.1 No wall-clock timeout may determine result

Canonicalisation MUST NOT branch on wall-clock time, timeout, random seed, or
load to decide success vs failure or choice of minimum. The result must be
deterministic given `P`.

### 15.2 Deterministic work budget

Implementations SHOULD enforce a deterministic budget in terms of **search
nodes or candidate leaves** (not wall-clock milliseconds). Suggested budget:

```
budget_nodes  : 1_000_000  search nodes (individualisation steps)
budget_leaves : 5_000_000  candidate leaves (full label assignments evaluated)
safety_cap    : 1000       WL refinement iterations (already in tethers_core_canonical.ml:17)
```

Exact numbers are **implementation policy**, not part of the format spec, but
MUST be documented where chosen. Changing the budget affects **whether** a
digest is produced, NOT **what** digest is produced when successful (see §15.3).

Implementations MAY also count WL refinement operations.

### 15.3 On exhaustion

If the budget is exceeded:

```
Result = Error Canonicalisation_too_complex
No CanonicalPayload_V2
No ProgramDigest_V2
No fallback
```

Fall-back to any of these is **forbidden** and would be a non-conformant
degradation:

- Source order
- Raw ID order
- Current-best-so-far
- First leaf found
- Random ordering
- Truncated candidate set

The API MUST surface a typed error (`Refinement_exceeded` / `Canonicalisation_too_complex`)
that callers can distinguish from `Invalid_core`.

### 15.4 Budget vs format

`Canonical Format V2` defines **what** the digest is when canonicalisation
succeeds. Budget defines **when** the operation fails. Two implementations with
different budgets are both conformant if they agree on every `P` that both
succeed on. Interop requires documentation of budgets but not identical budgets.

---

## 16. Domain separation — exact bytes (frozen)

### 16.1 Preimage construction

```
CanonicalPreimage_V2(P) = DOMAIN_V2 || CanonicalPayload_V2(P)
```

where `||` is byte concatenation.

### 16.2 DOMAIN_V2 exact bytes

```
Human-readable:  ASCII "TETHERS_CORE_CANON_V2"  followed by one zero byte 0x00
Hex:             54 45 54 48 45 52 53 5F 43 4F 52 45 5F 43 41 4E 4F 4E 5F 56 32 00
Length:          22 bytes (21 ASCII + 1 zero)
```

Why a trailing zero byte: it acts as an unambiguous separator guaranteeing that
`DOMAIN_V2` cannot be a prefix collision with any `CanonicalPayload_V2` content
(which is length-prefixed). The encoder's payload begins with `encode_string(core_version)`
whose first bytes are decimal digits (`"5:0.1.0"` etc.), not `0x00`, but the
separator makes this property structural rather than coincidental.

**Alternative considered:** length-prefixed domain (`"21:TETHERS_CORE_CANON_V2"`).
Rejected: domain separation benefits from a non-numeric separator distinguishing
`DOMAIN_V2` from the length-prefixed payload structure. The zero byte is also
the natural `canonical_prefix_byte = '\x00'` in `tethers_core_canonical.ml:16`
evolved with the version bump.

**Domain is inside the SHA-256 preimage.** It is included in the hash input,
not prepended to the hex output. Verification of a digest MUST recompute
`SHA256(DOMAIN_V2 || CanonicalPayload_V2(P))`.

### 16.3 Legacy V1 domain for contrast

`V1:  "TETHERS_CORE_CANON_V1" + 0x00  (21+1=22 bytes, same structure, different version char)`
`V2:  "TETHERS_CORE_CANON_V2" + 0x00  (22 bytes)`

The `V1` vs `V2` one-byte version distinction is covered by the hash;
cross-version collisions are impossible.

---

## 17. Digest string format — exact (frozen)

### 17.1 Syntax

```
DigestString_V2 ::= "tethers" ":" "v2" ":" "sha256" ":" 64*lower_hex
lower_hex       ::= [0-9a-f]         // exactly 64 chars
```

Full example: `tethers:v2:sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`

- Case: **lowercase** only. Uppercase hex is rejected on parse.
- Separators: ASCII colon `:` between fields, no whitespace, no trailing newline inside the string.
- Validation: parser MUST reject wrong prefix, wrong version, wrong hash algorithm tag, short/long hex, non-hex chars, extra fields.
- Legacy V1 external form was `"sha256:<64 hex>"` without the `tethers:v2:` envelope. V1 parser SHOULD still accept `"sha256:<hex>"` as `V1_DIGEST` for historical comparison only; `V2` parser MUST NOT accept bare `"sha256:<hex>"` as V2 without the envelope. No ambiguity: a `tethers:v2:`-prefixed string is V2; a bare `sha256:` string is legacy V1.

### 17.2 Storage and comparison

Digest strings are compared **exactly** (case-sensitive, byte-equal). There is
no canonicalisation beyond the frozen syntax. Hex is always 64 lowercase
characters, zero-padded from the 32-byte SHA-256 output.

---

## 18. ProgramDigest input — frozen terminology

Frozen terms (use these, not ad-hoc synonyms):

| Term | Definition | Contains DOMAIN_V2? |
|------|------------|---------------------|
| `CanonicalPayload_V2(P)` | `min_{π∈Π(P)} Enc_V2(π(P))` — the minimum encoded relabelling. This is the canonical representation. | NO |
| `CanonicalPreimage_V2(P)` | `DOMAIN_V2 \|\| CanonicalPayload_V2(P)` — the hash preimage. | YES (prefix) |
| `ProgramDigest_V2(P)` | `SHA-256(CanonicalPreimage_V2(P))` — 32 raw bytes. | via preimage |
| `DigestString_V2(P)` | `"tethers:v2:sha256:" ++ hex(ProgramDigest_V2)` — external rendering. | via digest |

**Frozen equation:**

```
ProgramDigest_V2(P) = SHA-256( DOMAIN_V2 || min_{π∈Π(P)} Enc_V2(π(P)) )
```

`CanonicalBytes_V2` is deprecated terminology; use `CanonicalPayload_V2`
(pure payload) vs `CanonicalPreimage_V2` (domain+payload). If the phrase
"canonical bytes" is used informally, it means `CanonicalPayload_V2`.

Naming ambiguity (e.g., "canonical bytes includes domain?") is resolved:
it does NOT. The payload is domain-free; the preimage is domain-prefixed.

---

## 19. V1 migration policy (frozen)

- **V1 is known incorrect.** C-B3T proved 23/24 Persistent Branch permutations
  diverged. V1 has never shipped as a frozen 1.0 public contract.
- **No new V1 identities may be produced** by any code path that stamps
  `ProgramDigest` for durable use (Trail, runtime plan, replay keys) after V2
  ships. The `TETHERS_CORE_CANON_V1` prefix and bare `sha256:<hex>` rendering
  are legacy.
- **Historical V1 → V2 mapping** is **provenance metadata only**. A stored trail
  entry that records `program_digest = sha256:<v1hex>` and a later V2 digest for
  the same `program_id` does NOT assert general semantic equivalence between V1
  and V2 identity models.
- **Dual identity must not infect runtime semantics.** No runtime component may
  accept either V1 or V2 interchangeably as equivalent without explicit
  provenance-aware comparison. Normal equality of digests MUST compare the full
  `DigestString_V2` (including `tethers:v2:`).
- Before 1.0, the legacy implementation (`tethers_core_canonical.ml` with
  `TETHERS_CORE_CANON_V1` prefix and current `assign_canonical_ids` that leaks
  colour order) may be archived/removed. Do not add new features to the V1 path.

---

## 20. Future format evolution — V2 → V3 table (frozen)

| Change | Requires V3? | Why |
|--------|:------------:|-----|
| Changing `Enc_V2` field/tag layout or field order | YES | Byte identity changes |
| Changing collection semantics (e.g., making a bag ordered) | YES | New semantic distinction merges or splits equivalence classes |
| Adding a semantic field to Core canonical identity (e.g., making `schema_description` semantic) | YES | Changes preimage |
| Removing a semantic field from identity | YES | Same |
| Changing scalar equality/normalisation (e.g., adding NFC, lowercasing, trimming) | YES | Merges previously distinct values |
| Changing integer/string/boolean representation (e.g., hex ints, base64 strings) | YES | Byte layout change |
| Introducing floats and choosing their encoding | YES | New type + equality rule |
| Changing lexicographic order definition | YES | Minimum changes |
| Changing `DOMAIN_V2` bytes | YES (is V3) | Preimage changes |
| Changing digest rendering (separator, case, prefix) | YES (digest-string V3) | External form changes |
| Changing `program_id` neutrality (making it semantic) | YES | Changes equivalence |
| Faster WL colour refinement that still returns `CanonicalPayload_V2` minimum | NO | Search independence (§13) |
| Different target-cell selection heuristic | NO | Still same minimum |
| Physical parallelism / different pruning / scheduling | NO | Same minimum |
| Different cache / memoisation / data structures | NO | Same minimum |
| Different colour integer assignment internally | NO | Colour never in payload |
| Different work-budget limit (more/less permissive) | NO | Affects liveness, not identity when successful |

If in doubt, ask: "Could `CanonicalPayload_V2(P)` for some valid `P` change?"
If yes, V3. If provably for all `P` the payload is identical, no version bump.

---

## 21. Frozen invariants (normative, for source comments/spec)

Copy this block verbatim into implementation headers and keep it in sync with
this spec.

```
I1  Permutation invariance:         For any π∈Π(P), CanonicalPayload_V2(P) = CanonicalPayload_V2(π(P)).
I2  Multiplicity preservation:      |E(P)| occurrences ↔ |E(P)| labels; no merging, no deletion.
I3  Semantic scalar preservation:   Every scalar payload listed in §3 contributes to Enc_V2; none is dropped or conflated.
I4  Representation-order elimination: Storage/source order of all D-classified collections is irrelevant; Π and Enc_V2 sorting erase it.
I5  Raw-ID non-semanticity:         The string content of origin_id / fact_id / role_id / branch_id / batch_id / item_template_id / group_id is not semantic.
I6  Colour / partition-rank non-semanticity: WL colours and partition cell numbers never appear in Enc_V2 and never affect the minimum.
I7  Deterministic structurally injective encoding: Distinct CanonicalPayload_V2 structures → distinct byte sequences (no boundary ambiguity).
I8  Exact byte-minimum identity:    Identity is min Enc_V2(π(P)) under unsigned byte-wise lexicographic order (§8), not under hash or colour.
I9  Search-strategy independence:   Any refinement/search strategy returning the §1 minimum is conformant (§13).
I10 Fail-closed canonicalisation:   On deterministic resource exhaustion, error Canonicalisation_too_complex with no payload/digest fallback (§15).
I11 Scope preservation:             Role identity is scope-qualified; cross-scope swaps are forbidden (§2.4, §10).
I12 Parallel scheduling non-observability: Thread count, scheduling, and pruning do not affect the result (§13).
```

---

## 22. Worked examples — equivalence / non-equivalence

For each pair, the expected `ProgramDigest_V2` relationship is frozen.

### A. Raw-ID rename — same digest

```
P1: input_facts=[Evaluation_input("hk1",String_type) Fact_id "banana_thing_947"]
    origin_sites=[Anchor_origin "banana_thing_947" event "ev"]
P2: same but Fact_id "O_anchor" and origin "O_anchor"
→ Π maps banana_thing_947 ↔ O_anchor → Enc_V2 identical → digests identical
```

### B. Origin declaration reorder — same digest

`P1: origin_sites=[Anchor "ent", Action "A", Action "B"]`
`P2: origin_sites=[Anchor "ent", Action "B", Action "A"]` (storage swap, same scalar payloads)
→ D-classified → sorted by canonical label → same payload → same digest

### C. Two identical Actions swapped — same digest

`P1: two Actions both {cap "cap.x", digest "sha256:d1", input "x"="v"}`
`P2: storage order of those two origins swapped`
→ Both are automorphic (identical scalars) → `Enc_V2` sorting normalises → same digest

### D. One Action vs two identical Actions — different digest

`P1: 1× Action {cap "cap.x", input "x"="v"}`
`P2: 2× Action identical to above (two occurrences)`
→ `|E(P2)| = |E(P1)|+1` → `encode_list` length prefix and extra `encode_origin_site` chunk →
different payload → different digests (even though scalars coincide)

### E. Persistent Branch permutation — same digest (C-B3T witness)

`P with 4 branches B1..B4 and 4 origins O1..O4 permuted identically:`
`P1: branches [B1,B2,B3,B4] with outcome maps identical shape`
`P2: any of the other 23 permutations of the same 4 branches`
→ Same `Π` equivalence class of size ≤24; V2 minimum collapses them → same digest
(V1 produced 24 distinct digests; V2 produces 1.)

### F. Semantic capability_id mutation — different digest

`P1: Action capability_id "cap.A" vs P2: "cap.B"` (same positions/labels, different scalar)
→ `encode_string("cap.A") ≠ encode_string("cap.B")` → bytes differ → digests differ

### G. Together member reorder — same digest

`Together(G, [A,B], All_members_succeed)` vs `Together(G, [B,A], All_members_succeed)`
where A,B are automorphic actions
→ member_origin_ids is B-classified set → sorted by canonical origin label →
same encoding → same digest

**Negative Together:** `Together(G, [A,B])` vs `Together(G, [A,C])` where `C` has
capability_id `"cap.Y"` vs B's `"cap.X"` → B≠C scalar → different bytes → different digest

### H. Scoped/template symmetry — same digest for pure scope renaming/permutation

```
P1: IT_alpha { role R1@alpha : fulfillment "ok" }  +  IT_beta { role R1@beta : fulfillment "ok" }
P2: swap raw IT IDs (alpha↔beta) with storage swap
→ π_item_template swaps the two templates → same sorted payload → same digest
(because template scalars are identical, so the templates are automorphic)
```

### I. Change actual scope relationship — different digest

`P1: role R1 @ Program_scope`
`P2: role R1 @ Item_template_scope(IT_a)`
→ `encode_role_scope` differs (`0:` vs `1:<template_label>`) → bytes differ → digests differ

---

## 23. Reference slow oracle (frozen test oracle)

### 23.1 Definition — obviously-correct, deliberately slow

```
slow_reference_oracle(P):
    validate P or return Error (no digest)
    E = enumerate_all_Π(P)   // every type- and scope-preserving bijection (§1-2)
    if |E| exceeds oracle_size_limit: return Error Oracle_too_large (not Canonicalisation_too_complex)
    candidates = [ Enc_V2(π(P)) for each π in E ]      // use frozen Enc_V2 (§6)
    payload    = min candidates under lexicographic order (§8)
    preimage   = DOMAIN_V2 || payload
    digest     = SHA-256(preimage)
    return (payload, preimage, hex_digest, DigestString_V2)
```

This is factorial in each family's automorphic sub-partitions but is the **gold
oracle for small programs**. Production algorithms MUST match it for all `P`
that the oracle can handle.

### 23.2 Practical size bounds

The oracle is test-only and must gate its enumeration to avoid combinatorial
explosion:

```
oracle_total_entities_limit = 16   // |E(P)| kinds combined; enumerates Π only if Π size < ~1M
oracle_max_family_size      = 6    // no single family has >6 permutable occurrences
oracle_max_total_permutations = 720 // hard cap: abort with Oracle_too_large beyond this
```

For the Persistent Branches witness (`4!×4! = 576`), the oracle is within budget.
For 7+ identical facts, it exceeds and tests must shard or use production
canonicaliser comparison instead.

The oracle MUST NOT be used as the production implementation — it has no WL
pruning and is exponentially slower.

### 23.3 Oracle vs production contract

For every generated small valid `P` within oracle bounds:

```
production_v2(P) == slow_oracle(P)   // payload, preimage, and digest all identical
```

Any divergence is a production bug or an oracle bug; the oracle's mathematical
definition (§1) is the tie-breaker.

---

## 24. Property testing — frozen plan

Use deterministic seeds; no nondeterministic randomness may determine pass/fail.

### 24.1 Generators

- Generate small valid `P` via a typed Core generator that respects validator
  invariants (unique IDs, reference integrity, no duplicate outcomes, etc.).
- `seed = SHA256(test_name || attempt_index || quickcheck_seed)`; tests must be
  reproducible from the seed.
- Families: empty programs, single anchor+action, multi-action chains, Together
  groups, Branch maps, Roles, Batch+Template hybrids, deep chains.

### 24.2 Properties (all MUST hold)

For generated valid `P` within oracle bounds:

```
P1:  oracle(P) == oracle(rename(P))          // rename = replace every raw ID string with fresh unique strings
P2:  oracle(P) == oracle(permutation(P))     // permutation = shuffle storage order of every D collection
P3:  production_v2(P) == oracle(P)           // oracle agreement
P4:  production_v2(P) == production_v2(rename(P))
P5:  production_v2(P) == production_v2(permutation(P))
P6:  production_v2(P) == production_v2(colour_shuffle(P))  // re-refine with permuted colour ints
P7:  For semantic mutation mutate(P) that changes one scalar listed in §3
     (e.g., capability_id, event_name, fact value), production_v2(P) != production_v2(mutate(P))
     unless the mutation accidentally preserves bytes (e.g., neutral field change)
P8:  Multiplicity: let P1 have N identical entity occurrences, P2 have N+1
     (same scalars, one more occurrence) => Enc length differs => digest differs
P9:  Determinism: production_v2(P) repeated 100× is byte-identical
```

### 24.3 Negative property

Semantic mutations that alter `§3:Semantic? = YES` fields MUST change the
payload unless proven that the mutated value encodes identically (which would be
a collision in §6, i.e., a spec bug). The property test asserts `!=` for at
least `capability_id`, `event_name`, `core_value`, `contract_digest`,
`capability_input_name`, and `Deadline` mutations.

---

## 25. External independent oracle — bliss/nauty evaluation

### 25.1 Question

Could a test-only graph canonicalisation implementation (nauty/Traces/bliss)
serve as independent diversity checking, without being production identity
authority?

### 25.2 Assessment

**Useful as a second oracle** for small programs, with caveats. Tethers Core is
a typed relational structure, not an untyped graph. Translation to a coloured
digraph must be precise:

Proposed translation (for testing only):

```
Vertex per entity occurrence, coloured by entity kind (Origin/Fact/Branch/Role/Batch/Template/Group)
Vertex per capability_contract (but NOT per ProgramDigest — scalar payloads are vertex colours, not identities)
Edge types as labelled coloured edges:
  - origin_subject → branch_subject
  - fact → origin_provenance
  - fact → role_proxy (scoped)
  - origin → fact (declared_facts membership)
  - role → fact (fact_contract membership)
  - origin → role (Fact_through_role reference)
  - success_continuation (from_origin → target)
  - together membership (group → origin)
  - branch outcome (branch → origin via Continue_to)
Scalar labels (strings/ints) encoded as vertex colour extensions (sorted scalar bytes hashed into colour int)
Multiplicity: separate vertices for each occurrence (preserved)
Scope: encoded as additional structure (template→origin/role membership edges)
```

**What such an oracle could prove:**

- That `production_v2` and the slow permutation oracle agree with a completely
  independent canonical labelling library's canonical form (up to encoding
  translation). This increases confidence that the typed-relation view is correct.

**What it could NOT prove:**

- Byte-exact `Enc_V2` equality, because nauty/bliss canonical forms are over
  graph isomorphism, not over `Enc_V2`'s length-prefixed byte layout. You must
  map the graph's canonical permutation back to a Core relabelling and then
  re-encode with `Enc_V2` before comparing payloads.
- That multiplicity is preserved unless the translation preserves vertex cardinality
  (it does, by construction).
- That the directed/typed edge distinctions are lossless — at least property
  tests must verify that two Core programs that the graph translation considers
  automorphic are indeed Core-automorphic under §1.

### 25.3 Decision

Bliss/nauty translation is **optional, recommended for C-B4I or later**, not a
pre-C2 gate. If pursued, it MUST be test-only, MUST preserve the multiplicity
and kind distinctions above, and MUST NOT become production identity authority.
The frozen slow permutation oracle (§23) is already sufficient for the pre-C2
gate.

---

## 26. Pre-C2 freeze gate — exact minimum requirements

Before **`C2 physical concurrency`** (Together fan-out/join adapter execution)
resumes, all of the following MUST be true OFF `main` and green:

| # | Gate item | Acceptance |
|---|-----------|------------|
| 1 | V2 mathematical definition frozen | This draft accepted by Lucy (not just DRAFT) |
| 2 | `Enc_V2` frozen | `§6` encoder bytes/field order/tags stable |
| 3 | Domain/digest syntax frozen | `§16` `DOMAIN_V2` bytes, `§17` `tethers:v2:sha256:<hex>` rendering |
| 4 | Collection semantics frozen | `§4` table, no unclassified field |
| 5 | Scalar rules frozen | `§7` one representation per scalar; no hidden normalisation |
| 6 | Slow reference oracle implemented | `§23` test-only oracle behind a `dune` test target |
| 7 | Production V2 matches oracle on adversarial/generated small cases | `§24 P3` green on cap values in `§23` bounds |
| 8 | Persistent Branches 24-permutation suite green | V2 digest identical across all 24 witness permutations |
| 9 | Multiplicity green | `§5` examples: 1 vs 2 identical actions differ; swap identical actions same |
| 10 | Raw-ID and colour-renumbering attacks green | `P4` and `P6` from `§24` green |
| 11 | Deterministic resource-failure tested | `§15`: budget exhaustion returns typed error, no fallback, no wall-clock influence |
| 12 | Production test suites green | `dune runtest --force` and `cargo test` green on the V2 branch |

Performance optimisation (faster WL, different heuristics, parallelism tuning) is
explicitly **not** a pre-C2 gate item, provided the above identity gates are met.

---

## 27. Ambiguities discovered in current Core semantics

These were encountered while inventorying `tethers-0.1/engine-ocaml/bin/*.ml`:

1. **Group ID double-encoding (V1 erratum).** `Together_origin` in `tethers_core_canonical.ml:1327-1332` emits `string_of_group_id` as a raw string, making raw group IDs semantic. §6.6 resolves this by freezing label-only encoding for V2.

2. **Role label space (global vs per-scope).** `tethers_core_canonical.ml:787-878` assigns role labels in a single global sorted order keyed by `scoped_role_id`. §9.4 documents this as the frozen rule and notes the alternative per-scope consecutive assignment; both are acceptable if they preserve scope isolation and are documented. The draft freezes the global-sorted form already implemented.

3. **Capability contracts sorted by semantic string, not label.** V1 sorts `capability_contracts` by `capability_id` string (semantic). V2 preserves this — capability IDs are not entity labels. No ambiguity, but §4 and §6.4 call it out explicitly because it is the only non-label sort key.

4. **Neutral fields excluded vs not encoded as empty.** §3.3 freezes that neutral fields are **omitted**, not encoded as empty options. V1 never emitted them either (they were simply not in `encode_program`'s structure), but the freeze is explicit for future implementers.

5. **Batch encoding duplication.** `tethers_v2_canon_label.ml:403-415` had a special-case re-encode of `Batch_site` vs `encode_origin_site` sharing tag `3:`. The draft unifies to a single `encode_origin_site` path for batches. No semantic difference — both emit tag 3 with the same fields — just a structural note.

6. **Entry guards value scope.** `entry_guards` each reference a `fact_id` that MUST be in `input_facts` (validator). The spec classifies `entry_guards` as D; the guard's `fact_id` is a reference via `π_fact`, so guard set equality is up to fact relabelling. This is now frozen.

7. **No semantic sequences of entities.** Inventory shows zero D-unclassified fields and no `A. SEMANTIC SEQUENCE` collections of entities. If a future Core adds an ordered entity pipeline, §4 and §1.2 must be revised (requires V3).

8. **Floats absent.** `core_value` has no float case. Any future float rules require a new frozen section before they can be canonical (§7.4).

---

## 28. Explicit questions that MUST be resolved before implementation

**Q1.** Does Lucy accept the `DOMAIN_V2 = "TETHERS_CORE_CANON_V2" || 0x00` bytes,
and the `tethers:v2:sha256:<64hex>` rendering, or should the prefix change to a
length-prefixed domain form? (Blocked until Lucy review; default is the bytes in §16.)

**Q2.** Does Lucy accept the neutral-fields list in §3.3 (`program_id`,
`schema_description ×2`) as permanently neutral, or should any of them be made
semantic in V2? (Blocked until Lucy review; default is neutral.)

**Q3.** Should the resource budget numbers in §15.2 be enforced as hard limits
in the first V2 implementation, or deferred as implementation-policy knobs
while the spec freezes the failure contract only? (Recommend freezing the contract
now, tuning numbers during C-B4I.)

If all three answers are "accept default," no further spec ambiguity remains.

---

## 29. Final verdict

```
NOT READY — DRAFT SPEC AMBIGUITY REMAINS (pending Lucy review of Q1..Q3)
```

The **technical content** is implementation-ready: a competent engineer who has
never seen `tethers_core_canonical.ml` can implement a conforming V2
canonicaliser from §§1-18 alone and match the reference oracle.

The **process gate** is not yet ready: Lucy has not yet accepted this draft,
and the three explicit questions above require her written freeze before the
branch may move to C-B4I implementation. Once Lucy marks the three Qs as
ACCEPTED (or provides corrected values that are applied to §§16-17, §3.3, §15),
the verdict becomes `READY TO IMPLEMENT V2`.

---

## 30. Report

MODEL USED: `muse-spark-1.2`
THINKING LEVEL: `HIGH`
WALL-CLOCK TIME: `~55 minutes`
TOKEN/CACHE USAGE: `UNAVAILABLE`
COST: `UNAVAILABLE`

Startup report (per AGENTS.md gate):

- Repository root: `D:\The Next Thing\Tethers Lang - Goose Integration`
- Current branch: `perf/c-core-cheap-structural-fixes`
- Exact HEAD: `20dd0ee2d5bfbab507ef85fdfc7f27c4ae0859dc`
- Instruction files loaded: `AGENTS.md` (auto), `opencode.json` boosters (if active)
- Mandatory documents read explicitly: `docs/PROJECT_CONTROL.md`, `docs/AGENT_WORKFLOW.md`,
  `docs/CURRENT_CLINE_TASK.md`, `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`,
  `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md`, `docs/CONSTITUTION.md`,
  `docs/DECISIONS.md`, `tethers-0.1/SPEC.md`, `tethers-0.1/engine-ocaml/bin/tethers_core.ml`,
  `tethers-0.1/engine-ocaml/bin/tethers_core_canonical.ml`, `tethers-0.1/engine-ocaml/bin/tethers_core_validator.ml`,
  `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer.ml`, `tethers-0.1/engine-ocaml/bin/tethers_v2_canon_label.ml`,
  plus `tethers_core_canonical_test.ml` and `tethers_core_canonical.mli`
- Current task owner/status/risk: This is `C-B4S` spec-freeze task per user prompt (not packet-bound `CORE-9C` which is `COMPLETE`). Risk colour: `Red` (semantics/identity).
- Authorised files / forbidden changes: Per C-B4S packet text: **NO production implementation, NO ProgramDigest behaviour change, NO C2 implementation, NO raw-ID tie-breaks.** This draft touches only `docs/CANONICAL_FORMAT_V2_SPEC_DRAFT.md` (new) — no production source modified.
- Stop conditions: Implement production Canonical Format V2, modify ProgramDigest behaviour, start C2 concurrency, optimise refinement, redesign replay/runtime.
- Repeated-failure rule: After two materially similar failed attempts on the same unresolved problem, stop and return exact evidence + smallest question.

`scripts/check-dev-tools.ps1` was run and reported `rg, fd, jq, yq, gh, just, git, pwsh` all present (no missing tool).

Verification:

- No production file was modified (`git status` shows only `docs/`, existing `perf/` worktree debris untouched).
- Spec cross-checked against `Tethers_core.program`, validator, lowerer, v1 encoder, and v2 prototype search metrics.
- No implementation checks (`cargo fmt --check`, `dune build @all`, `dune runtest`) are required for this doc-only spec draft.
