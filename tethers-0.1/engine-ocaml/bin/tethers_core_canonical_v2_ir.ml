(* ==================================================================
   CANONICAL FORMAT V2 — EXACT HYBRID SEARCH (C-B4I3C)

   FROZEN INVARIANT HEADER — obligations prominently stated:

   I1  Permutation invariance — raw-ID / storage order does not affect identity
   I2  Multiplicity preservation — duplicate structures remain distinct
   I3  Semantic scalar preservation — capability_id, contract_digest,
       event_name, host_snapshot_key, fulfillment, etc. are semantic
   I4  Representation-order elimination — collection order does not affect Enc_V2
   I5  Raw-ID non-semanticity — raw strings are opaque handles only
   I6  Colour / partition-rank non-semanticity — colours are search only
   I7  Structurally injective encoding — distinct labelled structures -> distinct bytes
   I8  Exact byte-minimum identity — winner is min Enc_V2 under unsigned lex
   I9  Search-strategy independence — HOW min is found does not change WHAT min is
   I10 Fail-closed canonicalisation — validation failure / budget exhaustion -> Error, no partial
   I11 Scope preservation — Program_role vs Template_role scopes are disjoint
   I12 Parallel scheduling non-observability — single-threaded, no schedule leaks

   COLOURS ARE SEARCH MACHINERY ONLY.
   They never enter Enc_V2.
   They never decide canonical identity by themselves.

   Architecture (C-B4I3C):
     Validated Core
       -> typed semantic search graph/state (six families, scope-qualified roles)
       -> initial invariant partition (per-family, scalar-only descriptors)
       -> BSP synchronous refinement to stable partition (immutable previous round)
          ** fail-closed on max_refinement_rounds **
       -> exact family-wise enumeration of every remaining labelling
       -> encoder-derived reductions, each enabled only when its
          first-differing-byte proof applies:
          * all Facts are top-level distinct Evaluation_input occurrences
          * entry_origin is the first Origin-sensitive field
          * one physical program-origin collection contains only self-label
            Anchor bodies after entry_origin is fixed; distinct body classes
            are ordered and equal-body classes remain exhaustively permuted
          * one physical collection owns every Branch occurrence
          * Program Roles have no earlier role-sensitive occurrence
          * Template Roles have no earlier occurrence and distinct exact bodies
       -> exact minimum (leaf Enc_V2 comparison via compare_bytes_lex_unsigned)

   This is deliberately a hybrid, not a claim of general IR pruning.  Stable
   refinement is real diagnostic/budgeted work, but it does not discard a
   branch: equal colours are never an automorphism certificate.  Every active
   reduction below is justified directly by Enc_V2's earliest differing bytes.

   Typed entity domain — exactly six anonymous canonical families:
     Origin, Fact, Branch, Batch, ItemTemplate, Role (scoped occurrence)
   No Group entity.  Role occurrence identity is scope-qualified:
     Program_role(role_id)  and  Template_role(item_template_id, role_id)
   Scalar / non-anonymous keys (capability_id, contract_digest, event_name,
   host_snapshot_key, fulfillment, etc.) participate in refinement signatures
   but are NOT relabelled entities.

   Refinement isolation (BSP semantics):
     Signature for entity E at round N+1 depends ONLY on:
       - E's semantic family
       - semantic scalar fields
       - immutable structural relationships (relation kinds)
       - labels/colours from completed round N
       - frozen scope relationships (Role_proxy scope-aware)
     It MUST NOT depend on source/discovery order, raw-ID lexical order,
     Hashtbl iteration, mutable partial round N+1 state, thread scheduling,
     random, wall-clock, or cache insertion order.

   Colour identity:
     Colour integers have no semantic meaning; equivalent runs may assign
     different ints.  Colour never appears in Enc_V2, never decides tie-break.
     Raw IDs never break equal-colour symmetry.  Hash collisions checked by
     exact descriptor equality.

   Active reductions (C-B4I3C — proven):
     A. deterministic resource-budget rejection (pre-admission)
     B. exact duplicate payload memoisation ONLY after complete encoding
        (duplicate_payload_hits — NOT counted as leaves avoided)
     C. distinct top-level input-Fact ordering by exact encoded provenance
        bytes (single minimal Fact labelling)
     D. entry_origin's exact minimum label when it is first Origin-sensitive
     E. exact Anchor-origin body-class ordering in one dependency-closed
        program collection after entry_origin is fixed; equal-body class
        permutations remain live for later Enc_V2 fields
     F. exact Branch-body ordering when one collection owns all Branches
     G. exact Program Role-body ordering when no earlier program field reads a
        Program Role label
     H. exact Template Role-body ordering only for dependency-closed,
        pairwise-distinct bodies (tied bodies remain exhaustive because the
        later objective can distinguish them)
     Prefix and orbit counters are retained as zero-valued compatibility
     telemetry; neither rule is active.
     NOT allowed:
       "same WL colour so search one" without separate isomorphism proof
   ================================================================== *)

open Tethers_core
open Tethers_core_canonical_v2_format

[@@@warning "-27-32-69"]

(* ================================================================== *)
(*  Types                                                               *)
(* ================================================================== *)

type canonicalized_v2_ir = {
  validated_program : Tethers_core.program;
  canonical_payload : string;
  canonical_preimage : bytes;
  canonical_program_digest : string;
}

type canonicalization_error_ir =
  | Invalid_core of Tethers_core_validator.validation_error list
  | Canonicalisation_too_complex

type search_budget_ir = {
  max_nodes : int;
  max_leaves : int;
  max_refinement_rounds : int;
}

let default_budget_ir = {
  max_nodes = 1_000_000;
  max_leaves = 5_000_000;
  max_refinement_rounds = 1000;
}

type ir_stats = {
  nodes : int;
  leaves_encoded : int;
  refinement_rounds : int;
  prefix_subtrees_pruned : int;
  orbit_branches_pruned : int;
  duplicate_payload_hits : int;
  max_depth : int;
  leaves_avoided : int;
}

(* ================================================================== *)
(*  Overflow-safe candidate counting (same arithmetic as baseline)      *)
(* ================================================================== *)

let safe_mul a b limit =
  if a = 0 || b = 0 then Some 0
  else if a > limit / b then None
  else Some (a * b)

let safe_fact n limit =
  let rec go i acc =
    if i > n then Some acc
    else match safe_mul acc i limit with
    | None -> None
    | Some acc' -> go (i + 1) acc'
  in
  if n <= 0 then Some 1 else go 2 1

let candidate_count_within_budget_ir ~limit (p : program) : int option =
  let ( let* ) = Option.bind in
  let all_facts = collect_facts p in
  let all_origins = collect_origins p in
  let all_batches = collect_batches p in
  let all_roles = collect_roles p in
  let all_branches = collect_branches p in
  let all_templates = p.item_templates in
  let n_facts = List.length all_facts in
  let n_origins = List.length all_origins in
  let n_batches = List.length all_batches in
  let n_branches = List.length all_branches in
  let n_templates = List.length all_templates in
  let* fact_perms = safe_fact n_facts limit in
  let* origin_perms = safe_fact n_origins limit in
  let* batch_perms = safe_fact n_batches limit in
  let* branch_perms = safe_fact n_branches limit in
  let* template_perms = safe_fact n_templates limit in
  let program_roles = List.filter_map (fun (_r, scope) ->
    match scope with `Program -> Some () | _ -> None
  ) all_roles in
  let n_program_roles = List.length program_roles in
  let template_roles_groups =
    List.filter_map (fun (t : item_template) ->
      let roles = List.filter_map (fun (_r, scope) ->
        match scope with
        | `Template tid when tid = t.item_template_id -> Some ()
        | _ -> None
      ) all_roles in
      if roles = [] then None else Some (List.length roles)
    ) all_templates
  in
  let* role_perms = safe_fact n_program_roles limit in
  let* role_perms =
    List.fold_left (fun acc_opt n ->
      let* acc = acc_opt in
      let* p = safe_fact n limit in
      safe_mul acc p limit
    ) (Some role_perms) template_roles_groups
  in
  let* total = safe_mul fact_perms origin_perms limit in
  let* total = safe_mul total batch_perms limit in
  let* total = safe_mul total branch_perms limit in
  let* total = safe_mul total template_perms limit in
  safe_mul total role_perms limit

(* ================================================================== *)
(*  Typed semantic search graph — entity inventories                    *)
(* ================================================================== *)

type fact_scalar = {
  provenance_kind : int;
  host_key : string option;
  scalar_type_rank : int option;
}

type origin_scalar = {
  origin_kind : int;
  event_name : string option;
  capability_id_str : string option;
  contract_digest_str : string option;
  together_objective_rank : int option;
}

type branch_scalar = {
  outcome_count : int;
  outcome_ranks_sorted : int list;
}

type batch_scalar = {
  collection_prov : string;
  traversal_policy : string;
  composite_objective : string;
}

type template_scalar = {
  objective_is_required_role : bool;
}

type role_scalar = {
  scope_kind : int;
  fulfillment : string;
  fact_contract_size : int;
}

(* ================================================================== *)
(*  Initial invariant partition signatures                              *)
(* ================================================================== *)

let fact_scalar_of (f : fact) : fact_scalar =
  match f.provenance with
  | Evaluation_input (Host_snapshot_key k, t) ->
      let rank = match t with String_type -> 0 | Integer_type -> 1 | Boolean_type -> 2 in
      { provenance_kind = 0; host_key = Some k; scalar_type_rank = Some rank }
  | Origin_provenance _ ->
      { provenance_kind = 1; host_key = None; scalar_type_rank = None }
  | Role_proxy _ ->
      { provenance_kind = 2; host_key = None; scalar_type_rank = None }

let origin_scalar_of_site (site : origin_site) : origin_scalar =
  match site with
  | Anchor_origin a ->
      { origin_kind = 0; event_name = Some a.event_name; capability_id_str = None; contract_digest_str = None; together_objective_rank = None }
  | Action_origin a ->
      { origin_kind = 1; event_name = None; capability_id_str = Some (string_of_capability_id a.capability_id); contract_digest_str = Some (string_of_capability_contract_digest a.contract_digest); together_objective_rank = None }
  | Together_origin _t ->
      { origin_kind = 2; event_name = None; capability_id_str = None; contract_digest_str = None; together_objective_rank = Some 0 }
  | Batch_site _ ->
      { origin_kind = 3; event_name = None; capability_id_str = None; contract_digest_str = None; together_objective_rank = None }

let branch_scalar_of (b : branch) : branch_scalar =
  let ranks = List.map (fun (o, _) ->
    match o with Success -> 0 | Failure -> 1 | Uncertain -> 2 | Cancelled -> 3
  ) b.outcome_branches |> List.sort Int.compare in
  { outcome_count = List.length b.outcome_branches; outcome_ranks_sorted = ranks }

let batch_scalar_of (b : batch_site) : batch_scalar =
  { collection_prov = string_of_batch_collection_provenance b.collection_provenance;
    traversal_policy = string_of_batch_traversal_policy b.traversal_policy;
    composite_objective = string_of_batch_objective b.composite_objective }

let template_scalar_of (_t : item_template) : template_scalar =
  { objective_is_required_role = true }

let role_scalar_of (r : role) : role_scalar =
  let kind = match r.scope with Program_scope -> 0 | Item_template_scope _ -> 1 in
  let (Role_fact_contract fids) = r.fact_contract in
  { scope_kind = kind; fulfillment = string_of_role_fulfillment r.eligible_fulfillment; fact_contract_size = List.length fids }

type relation_kind =
  | Rel_fact_to_origin
  | Rel_fact_to_role
  | Rel_origin_to_fact_declared
  | Rel_origin_to_fact_aggregate
  | Rel_branch_subject
  | Rel_branch_target
  | Rel_role_to_fact_contract
  | Rel_template_to_origin
  | Rel_template_to_branch
  | Rel_template_to_role

(* ================================================================== *)
(*  Refinement state — includes fact scope map for Role_proxy          *)
(* ================================================================== *)

type refinement_input = {
  facts : fact list;
  origins : (origin_id * origin_site) list;
  batches : (batch_id * batch_site) list;
  branches : (branch * [`Program | `Template of item_template_id]) list;
  templates : item_template list;
  roles : (role * [`Program | `Template of item_template_id]) list;
  program : program;
  fact_scopes : (fact_id * [`Program | `Template of item_template_id]) list;
}

let build_refinement_input (p : program) : refinement_input =
  let facts = collect_facts p in
  let origins = collect_origins p in
  let batches = collect_batches p in
  let branches = collect_branches p in
  let templates = p.item_templates in
  let roles = collect_roles p in
  (* fact scope map mirrors validator: input_facts -> Program,
     program origin/btach facts -> Program, template origin/btach facts -> Template tid *)
  let fact_scopes =
    let acc = List.map (fun (f : fact) -> (f.fact_id, `Program)) p.input_facts in
    let acc = List.fold_left (fun acc site ->
      let facts = match site with
        | Anchor_origin a -> a.declared_facts
        | Action_origin a -> a.declared_facts
        | Together_origin _ -> []
        | Batch_site b -> b.aggregate_facts
      in
      List.fold_left (fun acc (f : fact) -> (f.fact_id, `Program) :: acc) acc facts
    ) acc p.origin_sites in
    List.fold_left (fun acc (t : item_template) ->
      List.fold_left (fun acc site ->
        let facts = match site with
          | Anchor_origin a -> a.declared_facts
          | Action_origin a -> a.declared_facts
          | Together_origin _ -> []
          | Batch_site b -> b.aggregate_facts
        in
        List.fold_left (fun acc (f : fact) -> (f.fact_id, `Template t.item_template_id) :: acc) acc facts
      ) acc t.origin_sites
    ) acc p.item_templates
  in
  { facts; origins; batches; branches; templates; roles; program = p; fact_scopes }

module FactIdMap = Map.Make(struct type t = fact_id let compare = compare end)
module OriginIdMap = Map.Make(struct type t = origin_id let compare = compare end)
module BranchIdMap = Map.Make(struct type t = branch_id let compare = compare end)
module BatchIdMap = Map.Make(struct type t = batch_id let compare = compare end)
module TemplateIdMap = Map.Make(struct type t = item_template_id let compare = compare end)

type colour_partition = {
  fact_colours : int array;
  origin_colours : int array;
  batch_colours : int array;
  branch_colours : int array;
  template_colours : int array;
  role_colours : int array;
}

let initial_colours (ri : refinement_input) : colour_partition =
  let assign_by_descriptor descriptors =
    let indexed = List.mapi (fun i d -> (i, d)) descriptors in
    let sorted = List.sort (fun (_, a) (_, b) -> compare a b) indexed in
    let colours = Array.make (List.length descriptors) 0 in
    let current_colour = ref 0 in
    let prev = ref None in
    List.iter (fun (idx, d) ->
      (match !prev with
       | None -> current_colour := 0
       | Some pd when pd = d -> ()
       | Some _ -> current_colour := !current_colour + 1);
      colours.(idx) <- !current_colour;
      prev := Some d
    ) sorted;
    colours
  in
  let fact_descriptors = List.map (fun f ->
    let s = fact_scalar_of f in
    (s.provenance_kind, s.host_key, s.scalar_type_rank)
  ) ri.facts in
  let origin_descriptors = List.map (fun (_, site) ->
    let s = origin_scalar_of_site site in
    (s.origin_kind, s.event_name, s.capability_id_str, s.contract_digest_str, s.together_objective_rank)
  ) ri.origins in
  let batch_descriptors = List.map (fun (_, b) ->
    let s = batch_scalar_of b in
    (s.collection_prov, s.traversal_policy, s.composite_objective)
  ) ri.batches in
  let branch_descriptors = List.map (fun (b, _) ->
    let s = branch_scalar_of b in
    (s.outcome_count, s.outcome_ranks_sorted)
  ) ri.branches in
  let template_descriptors = List.map (fun t ->
    let s = template_scalar_of t in
    s.objective_is_required_role
  ) ri.templates in
  let role_descriptors = List.map (fun (r, _) ->
    let s = role_scalar_of r in
    (s.scope_kind, s.fulfillment, s.fact_contract_size)
  ) ri.roles in
  {
    fact_colours = assign_by_descriptor fact_descriptors;
    origin_colours = assign_by_descriptor origin_descriptors;
    batch_colours = assign_by_descriptor batch_descriptors;
    branch_colours = assign_by_descriptor branch_descriptors;
    template_colours = assign_by_descriptor template_descriptors;
    role_colours = assign_by_descriptor role_descriptors;
  }

(* BSP refinement step — scope-aware Role_proxy:
   Role_proxy(role_id) resolves only to roles visible in the Fact's scope.
   Program facts -> Program roles only; Template facts -> that template's roles only. *)

let refinement_step (ri : refinement_input) (prev : colour_partition) : colour_partition =
  let fact_colour_by_id =
    List.fold_left (fun m ((f : fact), idx) ->
      FactIdMap.add f.fact_id prev.fact_colours.(idx) m
    ) FactIdMap.empty (List.mapi (fun i f -> (f, i)) ri.facts)
  in
  let origin_colour_by_id =
    List.fold_left (fun m ((oid, _), idx) ->
      OriginIdMap.add oid prev.origin_colours.(idx) m
    ) OriginIdMap.empty (List.mapi (fun i e -> (e, i)) ri.origins)
  in
  let branch_colour_by_id =
    List.fold_left (fun m ((b, _), idx) ->
      BranchIdMap.add b.branch_id prev.branch_colours.(idx) m
    ) BranchIdMap.empty (List.mapi (fun i e -> (e, i)) ri.branches)
  in
  let batch_colour_by_id =
    List.fold_left (fun m ((bid, _), idx) ->
      BatchIdMap.add bid prev.batch_colours.(idx) m
    ) BatchIdMap.empty (List.mapi (fun i e -> (e, i)) ri.batches)
  in
  let template_colour_by_id =
    List.fold_left (fun m (t, idx) ->
      TemplateIdMap.add t.item_template_id prev.template_colours.(idx) m
    ) TemplateIdMap.empty (List.mapi (fun i e -> (e, i)) ri.templates)
  in
  let _ = branch_colour_by_id in
  let _ = batch_colour_by_id in

  let fact_scope_lookup fid =
    List.assoc_opt fid ri.fact_scopes
  in

  let fact_new_descriptors =
    List.mapi (fun idx f ->
      let base = fact_scalar_of f in
      let base_sig = (base.provenance_kind, base.host_key, base.scalar_type_rank) in
      let neighbour =
        match f.provenance with
        | Evaluation_input _ -> []
        | Origin_provenance oid ->
            (match OriginIdMap.find_opt oid origin_colour_by_id with
             | Some c -> [(Rel_fact_to_origin, c)]
             | None -> [])
        | Role_proxy rid ->
            let fact_scope = fact_scope_lookup f.fact_id in
            let cols =
              List.mapi (fun i (r2, scope) -> (i, r2, scope)) ri.roles
              |> List.filter_map (fun (i, r2, scope) ->
                if r2.role_id <> rid then None
                else
                  match fact_scope, scope with
                  | Some `Program, `Program -> Some prev.role_colours.(i)
                  | Some (`Template tid), `Template tid2 when tid = tid2 -> Some prev.role_colours.(i)
                  | _ -> None)
            in
            List.map (fun c -> (Rel_fact_to_role, c)) cols
            |> List.sort compare
      in
      let _ = idx in
      (base_sig, neighbour)
    ) ri.facts
  in
  let origin_new_descriptors =
    List.mapi (fun idx (_, site) ->
      let base = origin_scalar_of_site site in
      let base_sig = (base.origin_kind, base.event_name, base.capability_id_str, base.contract_digest_str, base.together_objective_rank) in
      let neighbours =
        match site with
        | Anchor_origin a ->
            List.filter_map (fun (f : fact) ->
              FactIdMap.find_opt f.fact_id fact_colour_by_id |> Option.map (fun c -> (Rel_origin_to_fact_declared, c))
            ) a.declared_facts |> List.sort compare
        | Action_origin a ->
            List.filter_map (fun (f : fact) ->
              FactIdMap.find_opt f.fact_id fact_colour_by_id |> Option.map (fun c -> (Rel_origin_to_fact_declared, c))
            ) a.declared_facts |> List.sort compare
        | Together_origin t ->
            List.filter_map (fun oid ->
              OriginIdMap.find_opt oid origin_colour_by_id |> Option.map (fun c -> (Rel_branch_subject, c))
            ) t.member_origin_ids |> List.sort compare
        | Batch_site b ->
            List.filter_map (fun (f : fact) ->
              FactIdMap.find_opt f.fact_id fact_colour_by_id |> Option.map (fun c -> (Rel_origin_to_fact_aggregate, c))
            ) b.aggregate_facts |> List.sort compare
      in
      let _ = idx in
      (base_sig, neighbours)
    ) ri.origins
  in
  let branch_new_descriptors =
    List.mapi (fun idx (b, _) ->
      let base = branch_scalar_of b in
      let base_sig = (base.outcome_count, base.outcome_ranks_sorted) in
      let subject_col = OriginIdMap.find_opt b.branch_subject origin_colour_by_id |> Option.map (fun c -> (Rel_branch_subject, c)) |> Option.to_list in
      let target_cols = List.filter_map (fun (_, tgt) ->
        match tgt with
        | Continue_to oid -> OriginIdMap.find_opt oid origin_colour_by_id |> Option.map (fun c -> (Rel_branch_target, c))
        | Stop -> None
      ) b.outcome_branches |> List.sort compare in
      let _ = idx in
      (base_sig, subject_col @ target_cols)
    ) ri.branches
  in
  let batch_new_descriptors =
    List.mapi (fun idx (_, b) ->
      let base = batch_scalar_of b in
      let base_sig = (base.collection_prov, base.traversal_policy, base.composite_objective) in
      let template_col = TemplateIdMap.find_opt b.item_template_id template_colour_by_id |> Option.map (fun c -> (Rel_template_to_origin, c)) |> Option.to_list in
      let fact_cols = List.filter_map (fun (f : fact) ->
        FactIdMap.find_opt f.fact_id fact_colour_by_id |> Option.map (fun c -> (Rel_origin_to_fact_aggregate, c))
      ) b.aggregate_facts |> List.sort compare in
      let _ = idx in
      (base_sig, template_col @ fact_cols)
    ) ri.batches
  in
  let template_new_descriptors =
    List.mapi (fun idx (t : item_template) ->
      let base = template_scalar_of t in
      let base_sig = base.objective_is_required_role in
      let origin_cols = List.filter_map (fun site ->
        match origin_id_of_site site with
        | Some oid -> OriginIdMap.find_opt oid origin_colour_by_id |> Option.map (fun c -> (Rel_template_to_origin, c))
        | None -> None
      ) t.origin_sites |> List.sort compare in
      let branch_cols = List.filter_map (fun (b : branch) ->
        BranchIdMap.find_opt b.branch_id branch_colour_by_id |> Option.map (fun c -> (Rel_template_to_branch, c))
      ) t.branches |> List.sort compare in
      let role_cols =
        List.filter_map (fun (r : role) ->
          let idx_opt =
            List.mapi (fun j (rr, sc) -> (j, rr, sc)) ri.roles
            |> List.find_opt (fun (_, rr, sc) ->
              rr.role_id = r.role_id && sc = `Template t.item_template_id)
            |> Option.map (fun (j, _, _) -> j)
          in
          Option.map (fun j -> (Rel_template_to_role, prev.role_colours.(j))) idx_opt
        ) t.roles |> List.sort compare in
      let _ = idx in
      (base_sig, origin_cols @ branch_cols @ role_cols)
    ) ri.templates
  in
  let role_new_descriptors =
    List.mapi (fun idx ((r : role), _) ->
      let base = role_scalar_of r in
      let base_sig = (base.scope_kind, base.fulfillment, base.fact_contract_size) in
      let fact_cols = let (Role_fact_contract fids) = r.fact_contract in
        List.filter_map (fun fid ->
          FactIdMap.find_opt fid fact_colour_by_id |> Option.map (fun c -> (Rel_role_to_fact_contract, c))
        ) fids |> List.sort compare in
      let _ = idx in
      (base_sig, fact_cols)
    ) ri.roles
  in
  let recolour descriptors =
    let indexed = List.mapi (fun i d -> (i, d)) descriptors in
    let sorted = List.sort (fun (_, a) (_, b) -> compare a b) indexed in
    let colours = Array.make (List.length descriptors) 0 in
    let cur = ref 0 in
    let prev_d = ref None in
    List.iter (fun (idx, d) ->
      (match !prev_d with
       | None -> cur := 0
       | Some pd when pd = d -> ()
       | Some _ -> cur := !cur + 1);
      colours.(idx) <- !cur;
      prev_d := Some d
    ) sorted;
    colours
  in
  {
    fact_colours = recolour fact_new_descriptors;
    origin_colours = recolour origin_new_descriptors;
    batch_colours = recolour batch_new_descriptors;
    branch_colours = recolour branch_new_descriptors;
    template_colours = recolour template_new_descriptors;
    role_colours = recolour role_new_descriptors;
  }

(* Stable refinement — FAIL CLOSED on max_refinement_rounds *)
let stable_refinement (ri : refinement_input) (max_rounds : int)
    : (colour_partition * int, canonicalization_error_ir) result =
  let current = ref (initial_colours ri) in
  let rounds = ref 0 in
  let stable = ref false in
  while not !stable && !rounds < max_rounds do
    let next = refinement_step ri !current in
    incr rounds;
    let equal =
      !current.fact_colours = next.fact_colours &&
      !current.origin_colours = next.origin_colours &&
      !current.batch_colours = next.batch_colours &&
      !current.branch_colours = next.branch_colours &&
      !current.template_colours = next.template_colours &&
      !current.role_colours = next.role_colours
    in
    if equal then stable := true
    else current := next
  done;
  if not !stable then
    (* Did not converge within deterministic limit — fail closed (§B4I3-0A) *)
    Error Canonicalisation_too_complex
  else
    Ok (!current, !rounds)

(* ================================================================== *)
(*  Search budget tracking                                              *)
(* ================================================================== *)

type perm_state_ir = {
  assigned : int array;
  used : bool array;
  fixed : bool array;
  mutable next_pos : int;
  count : int;
}

let make_perm_state_ir n = {
  assigned = Array.make n 0;
  used = Array.make (n + 1) false;
  fixed = Array.make n false;
  next_pos = 0;
  count = n;
}

let rec assign_next_ir st callback =
  if st.next_pos >= st.count then callback st.assigned
  else begin
    let pos = st.next_pos in
    if st.fixed.(pos) then begin
      st.next_pos <- pos + 1;
      assign_next_ir st callback
    end else
      for label = 1 to st.count do
        if not st.used.(label) then begin
          st.assigned.(pos) <- label;
          st.used.(label) <- true;
          st.next_pos <- pos + 1;
          assign_next_ir st callback;
          st.used.(label) <- false
        end
      done;
    st.next_pos <- pos
  end

let fix_label_ir (st : perm_state_ir) ~(index : int) ~(label : int) : unit =
  if index < 0 || index >= st.count || label < 1 || label > st.count ||
     st.fixed.(index) || st.used.(label) then
    invalid_arg "invalid fixed permutation assignment";
  st.assigned.(index) <- label;
  st.used.(label) <- true;
  st.fixed.(index) <- true

type role_block_ir = {
  n_roles : int;
  start_label : int;
  mutable perm : int array;
}

type role_block_state_ir = {
  blocks : role_block_ir array;
  mutable block_index : int;
  block_perms : perm_state_ir array;
}

let make_role_block_state_ir blocks =
  let block_perms = Array.map (fun b -> make_perm_state_ir b.n_roles) blocks in
  { blocks; block_index = 0; block_perms }

let rec enumerate_role_blocks_ir st callback =
  if st.block_index >= Array.length st.blocks then callback st.blocks
  else begin
    let idx = st.block_index in
    st.block_index <- idx + 1;
    let bp = st.block_perms.(idx) in
    assign_next_ir bp (fun perm ->
      st.blocks.(idx).perm <- perm;
      enumerate_role_blocks_ir st callback
    );
    st.block_index <- idx
  end

(* ================================================================== *)
(*  Distinct-input ordering — exact single-assignment fast path            *)
(* ================================================================== *)

(* If every collected Fact is a distinct Evaluation_input, the first
   Enc_V2-sensitive occurrence of every Fact is the top-level input_facts
   list.  Giving the smaller exact provenance encoding the smaller label makes
   the first differing input_fact smaller.  An adjacent inversion therefore
   cannot be minimal, so the ascending byte order is the unique winner.  This
   rule is deliberately unavailable as soon as a Fact occurs elsewhere. *)
let facts_are_exactly_top_level_inputs (p : program) (facts : fact list) : bool =
  (* The fast-path proof starts at the top-level input_facts section.  Provenance
     alone is insufficient: an Evaluation_input Fact may be emitted later from
     an origin, batch, or template.  Validator-backed IDs make this an exact
     occurrence-inventory check, not an ordering decision. *)
  List.length facts = List.length p.input_facts &&
  let input_ids =
    List.fold_left (fun ids (f : fact) -> FactIdMap.add f.fact_id () ids)
      FactIdMap.empty p.input_facts
  in
  List.for_all (fun (f : fact) -> FactIdMap.mem f.fact_id input_ids) facts

let fact_discrete_minimal_order (facts : fact list) : int list option =
  (* Return permutation indices in the exact bytes that Enc_V2 emits for an
     input fact's provenance.  This deliberately does not use String.compare:
     Enc_V2 length-prefixes strings, so raw string order is not byte order
     (for example, "aa" encodes before "z" only if the encoded bytes say so). *)
  let n = List.length facts in
  if n <= 1 then Some (List.init n Fun.id)
  else
    let provenance_keys = List.mapi (fun i f ->
      match f.provenance with
      | Evaluation_input (Host_snapshot_key k, t) ->
          Some (encode_tag 0 ^ encode_string k ^ encode_scalar_type t, i)
      | _ -> None
    ) facts in
    if List.exists Option.is_none provenance_keys then None
    else
      let keys = List.filter_map Fun.id provenance_keys in
      let sorted = List.sort (fun (a,_) (b,_) ->
        compare_bytes_lex_unsigned a b
      ) keys in
      (* distinct check *)
      let distinct =
        let tbl = Hashtbl.create n in
        let ok = ref true in
        List.iter (fun (encoded,_) ->
          if Hashtbl.mem tbl encoded then ok := false else Hashtbl.add tbl encoded ()
        ) keys;
        !ok
      in
      if not distinct then None
      else Some (List.map snd sorted)

(* [entry_origin] is emitted after only the core version, input Facts, and
   entry guards.  Valid input Facts are Evaluation_input, so those earlier bytes
   are independent of every Origin label.  Consequently the entry Origin must
   receive the available label whose exact [encode_int] bytes are smallest;
   every other choice loses at that first Origin-sensitive byte. *)
let entry_origin_minimal_label
    (p : program) (origins : (origin_id * origin_site) list) : (int * int) option =
  match p.entry_origin with
  | None -> None
  | Some entry_origin when List.exists (fun (f : fact) ->
      match f.provenance with Origin_provenance _ -> true | _ -> false
    ) p.input_facts ->
      None
  | Some entry_origin ->
      let rec find_index index = function
        | [] -> None
        | (origin_id, _) :: rest ->
            if origin_id = entry_origin then Some index
            else find_index (index + 1) rest
      in
      match find_index 0 origins with
      | None -> None
      | Some index ->
          let count = List.length origins in
          if count = 0 then None
          else begin
            let best = ref 1 in
            for label = 2 to count do
              if compare_bytes_lex_unsigned (encode_int label) (encode_int !best) < 0 then
                best := label
            done;
            Some (index, !best)
          end

(* This deliberately narrow Origin reduction has no hidden graph assumption.
   Every Origin is a program-level Anchor, there are no continuations before
   program origin_sites, and declared Facts are Evaluation_input only.  Once
   Fact labels and entry_origin's exact label are fixed, an Anchor's suffix
   after its own Origin label has no Origin or Role dependency.  The sole
   physical collection then admits the usual adjacent-exchange proof. *)
let program_anchor_origins_are_dependency_closed
    (p : program) (origins : (origin_id * origin_site) list) : bool =
  match p.entry_origin with
  | None -> false
  | Some _ ->
      p.success_continuations = [] &&
      List.for_all (fun (t : item_template) -> t.origin_sites = []) p.item_templates &&
      List.length origins = List.length p.origin_sites &&
      List.for_all (function
        | Anchor_origin a ->
            List.for_all (fun (f : fact) -> match f.provenance with
              | Evaluation_input _ -> true
              | Origin_provenance _ | Role_proxy _ -> false
            ) a.declared_facts
        | Action_origin _ | Together_origin _ | Batch_site _ -> false
      ) p.origin_sites

(* Exact Anchor-body equality is label-independent here.  The body begins with
   the event string, then contains the declared Fact multiset sorted by an
   injective Fact labelling.  Therefore two free Anchor bodies can be equal iff
   their event strings and declared Fact-ID multisets are equal.  Raw IDs are
   used only for reference equality in this grouping, never to order classes.
   The fixed entry Origin is excluded because it is not exchanged. *)
let free_program_anchor_body_class_sizes (p : program) : int list =
  match p.entry_origin with
  | None -> []
  | Some entry ->
      let add_class classes key =
        let rec loop prefix = function
          | [] -> List.rev_append prefix [(key, 1)]
          | (existing, count) :: rest when existing = key ->
              List.rev_append prefix ((existing, count + 1) :: rest)
          | item :: rest -> loop (item :: prefix) rest
        in
        loop [] classes
      in
      List.fold_left (fun classes -> function
        | Anchor_origin a when a.anchor_origin_id <> entry ->
            let fact_ids = List.map (fun (f : fact) -> string_of_fact_id f.fact_id)
              a.declared_facts |> List.sort String.compare in
            add_class classes (a.event_name, fact_ids)
        | Anchor_origin _ | Action_origin _ | Together_origin _ | Batch_site _ -> classes
      ) [] p.origin_sites
      |> List.map snd

let program_anchor_residual_permutations ~limit (p : program) : int option =
  List.fold_left (fun current class_size ->
    let ( let* ) = Option.bind in
    let* product = current in
    let* class_perms = safe_fact class_size limit in
    safe_mul product class_perms limit
  ) (Some 1) (free_program_anchor_body_class_sizes p)

(* Branch IDs are not referenced by any V2 field.  When all Branch occurrences
   live in one physical Branch collection, every candidate emits the fixed
   numeric label sequence 1..N and differs only in the suffix after each label.
   Sorting those exact suffixes is therefore an adjacent-inversion proof of the
   byte minimum.  Multiple physical collections remain exhaustive because their
   shared global label space couples earlier and later Enc_V2 sections. *)
let all_branches_in_one_collection (p : program) (total : int) : bool =
  if total = 0 then false
  else
    let occupied =
      (if p.branches = [] then [] else [()]) @
      List.filter_map (fun (t : item_template) ->
        if t.branches = [] then None else Some ()
      ) p.item_templates
    in
    List.length occupied = 1

(* Program Roles are emitted after every program Origin site.  A Role_proxy
   Fact or Fact_through_role binding in such a site would make a role label
   visible earlier, so the local exchange proof must be disabled.  Template
   sites are emitted after program Roles and therefore do not affect this test. *)
let program_roles_unreferenced_before_own_collection (p : program) : bool =
  let site_uses_program_role = function
    | Anchor_origin a ->
        List.exists (fun (f : fact) -> match f.provenance with
          | Role_proxy _ -> true | _ -> false
        ) a.declared_facts
    | Action_origin a ->
        List.exists (fun (f : fact) -> match f.provenance with
          | Role_proxy _ -> true | _ -> false
        ) a.declared_facts ||
        List.exists (fun (input : action_input) -> match input.binding with
          | Fact_through_role _ -> true | _ -> false
        ) a.inputs
    | Together_origin _ -> false
    | Batch_site b ->
        List.exists (fun (f : fact) -> match f.provenance with
          | Role_proxy _ -> true | _ -> false
        ) b.aggregate_facts
  in
  not (List.exists site_uses_program_role p.origin_sites)

(* A template's role list can be locally ordered only when role labels have no
   earlier occurrence in that template.  The later objective can still observe
   a tied role body, so callers must additionally require pairwise-distinct
   exact bodies; otherwise the objective could decide between equal list
   prefixes and exhaustive search remains mandatory.  Other templates have
   disjoint typed Role namespaces. *)
let template_roles_unreferenced_before_own_collection (t : item_template) : bool =
  let site_uses_template_role = function
    | Anchor_origin a ->
        List.exists (fun (f : fact) -> match f.provenance with
          | Role_proxy _ -> true | _ -> false
        ) a.declared_facts
    | Action_origin a ->
        List.exists (fun (f : fact) -> match f.provenance with
          | Role_proxy _ -> true | _ -> false
        ) a.declared_facts ||
        List.exists (fun (input : action_input) -> match input.binding with
          | Fact_through_role _ -> true | _ -> false
        ) a.inputs
    | Together_origin _ -> false
    | Batch_site b ->
        List.exists (fun (f : fact) -> match f.provenance with
          | Role_proxy _ -> true | _ -> false
        ) b.aggregate_facts
  in
  not (List.exists site_uses_template_role t.origin_sites)

(* A conservative, label-independent pre-admission witness for the distinct
   body condition below.  Different fulfilment strings yield different exact
   length-prefixed suffixes irrespective of all free labels. *)
let template_roles_have_distinct_fulfillments (t : item_template) : bool =
  let fulfilments = List.map (fun (r : role) ->
    string_of_role_fulfillment r.eligible_fulfillment
  ) t.roles in
  List.length fulfilments = List.length (List.sort_uniq String.compare fulfilments)

(* The deterministic leaf budget measures leaves this engine can actually
   reach after its proven exact reductions, not the discarded raw Lambda(P)
   permutations.  The public helper above intentionally remains the raw-space
   count used by exhaustive-baseline evidence. *)
let reduced_candidate_count_within_budget_ir ~limit (p : program) : int option =
  let ( let* ) = Option.bind in
  let facts = collect_facts p in
  let origins = collect_origins p in
  let batches = collect_batches p in
  let branches = collect_branches p in
  let templates = p.item_templates in
  let roles = collect_roles p in
  let fact_perms =
    if facts_are_exactly_top_level_inputs p facts then
      match fact_discrete_minimal_order facts with
      | Some _ -> Some 1
      | None -> safe_fact (List.length facts) limit
    else
      safe_fact (List.length facts) limit
  in
  let origin_perms =
    let count = List.length origins in
    if program_anchor_origins_are_dependency_closed p origins then
      program_anchor_residual_permutations ~limit p
    else match entry_origin_minimal_label p origins with
      | Some _ -> safe_fact (count - 1) limit
      | None -> safe_fact count limit
  in
  let branch_perms =
    if all_branches_in_one_collection p (List.length branches) then Some 1
    else safe_fact (List.length branches) limit
  in
  let program_role_count =
    List.fold_left (fun count (_, scope) ->
      match scope with `Program -> count + 1 | `Template _ -> count
  ) 0 roles
  in
  let program_role_perms =
    if program_roles_unreferenced_before_own_collection p then Some 1
    else safe_fact program_role_count limit
  in
  let template_role_counts = List.filter_map (fun (t : item_template) ->
    let count = List.fold_left (fun count (_, scope) ->
      match scope with
      | `Template tid when tid = t.item_template_id -> count + 1
      | _ -> count
    ) 0 roles in
    if count = 0 then None
    else if template_roles_unreferenced_before_own_collection t &&
            template_roles_have_distinct_fulfillments t then Some 1
    else Some count
  ) templates in
  let* fact_perms = fact_perms in
  let* origin_perms = origin_perms in
  let* batch_perms = safe_fact (List.length batches) limit in
  let* branch_perms = branch_perms in
  let* template_perms = safe_fact (List.length templates) limit in
  let* role_perms = program_role_perms in
  let* role_perms = List.fold_left (fun current count ->
    let* product = current in
    let* factor = safe_fact count limit in
    safe_mul product factor limit
  ) (Some role_perms) template_role_counts in
  let* total = safe_mul fact_perms origin_perms limit in
  let* total = safe_mul total batch_perms limit in
  let* total = safe_mul total branch_perms limit in
  let* total = safe_mul total template_perms limit in
  safe_mul total role_perms limit

(* ================================================================== *)
(*  Main canonicalize_ir — individualisation/refinement + proven pruning *)
(* ================================================================== *)

let canonicalize_ir ?(budget = default_budget_ir) (p : program) :
    (canonicalized_v2_ir * ir_stats, canonicalization_error_ir) result =
  match Tethers_core_validator.validate p with
  | Error errs -> Error (Invalid_core errs)
  | Ok () -> begin
      let raw_candidate_count = candidate_count_within_budget_ir ~limit:max_int p in
      match reduced_candidate_count_within_budget_ir ~limit:budget.max_leaves p with
      | None -> Error Canonicalisation_too_complex
      | Some _ -> begin
          let ri = build_refinement_input p in
          match stable_refinement ri budget.max_refinement_rounds with
          | Error e -> Error e
          | Ok (_stable_partition, refinement_rounds) ->

          let all_facts_list = ri.facts in
          let all_origins_list = ri.origins in
          let all_batches_list = ri.batches in
          let all_roles_list = ri.roles in
          let all_branches_list = ri.branches in
          let all_templates_list = ri.templates in
          let n_facts = List.length all_facts_list in
          let n_origins = List.length all_origins_list in
          let n_batches = List.length all_batches_list in
          let n_branches = List.length all_branches_list in
          let n_templates = List.length all_templates_list in
          let origin_permutation_mode =
            if program_anchor_origins_are_dependency_closed p all_origins_list
            then `Try_single_program_anchors else `All
          in
          let branch_permutation_mode =
            if all_branches_in_one_collection p n_branches then `Single else `All
          in

          (* The proof is local to the first Enc_V2 input_facts section; it does
             not rely on a colour number or on refinement being discrete. *)
          let fact_permutation_mode =
            if facts_are_exactly_top_level_inputs p all_facts_list then
              match fact_discrete_minimal_order all_facts_list with
              | Some order -> `Single order
              | None -> `All
            else `All
          in

          let fact_state = make_perm_state_ir n_facts in
          let origin_state = make_perm_state_ir n_origins in
          let batch_state = make_perm_state_ir n_batches in
          let branch_state = make_perm_state_ir n_branches in
          let template_state = make_perm_state_ir n_templates in
          (match entry_origin_minimal_label p all_origins_list with
           | None -> ()
           | Some (index, label) -> fix_label_ir origin_state ~index ~label);
          let program_roles_list = List.filter_map (fun (r, scope) ->
            match scope with `Program -> Some r | _ -> None
          ) all_roles_list in
          let n_program_roles = List.length program_roles_list in
          let program_role_permutation_mode =
            if program_roles_unreferenced_before_own_collection p then `Single else `All
          in
          let template_roles_groups : (item_template_id * role list) list =
            List.filter_map (fun (t : item_template) ->
              let roles = List.filter_map (fun (r, scope) ->
                match scope with
                | `Template tid when tid = t.item_template_id -> Some r
                | _ -> None
              ) all_roles_list in
              if roles = [] then None else Some (t.item_template_id, roles)
            ) all_templates_list
          in
          let best_payload = ref "" in
          let best_preimage = ref Bytes.empty in
          let best_digest = ref "" in
          let stats_nodes = ref 0 in
          let stats_leaves = ref 0 in
          let stats_prefix_pruned = ref 0 in
          let stats_orbit_pruned = ref 0 in
          let stats_duplicate_hits = ref 0 in
          let stats_max_depth = ref 0 in
          let budget_exceeded = ref false in
          let memo_payloads = Hashtbl.create 1024 in

          let check_budget () =
            if !stats_nodes > budget.max_nodes || !stats_leaves > budget.max_leaves then
              budget_exceeded := true
          in

          (* Helper to build label assignment from current perm states *)
          let build_label_assignment () =
            let fact_map = List.fold_left2 (fun (m : int FactMap.t) (f : fact) idx ->
              FactMap.add f.fact_id fact_state.assigned.(idx) m
            ) FactMap.empty all_facts_list (List.init n_facts Fun.id) in
            let origin_map = List.fold_left2 (fun (m : int OriginMap.t) ((oid, _) : origin_id * origin_site) idx ->
              OriginMap.add oid origin_state.assigned.(idx) m
            ) OriginMap.empty all_origins_list (List.init n_origins Fun.id) in
            let batch_map = List.fold_left2 (fun (m : int BatchMap.t) ((bid, _) : batch_id * batch_site) idx ->
              BatchMap.add bid batch_state.assigned.(idx) m
            ) BatchMap.empty all_batches_list (List.init n_batches Fun.id) in
            let branch_map = List.fold_left2 (fun (m : int BranchMap.t) ((b, _) : branch * _) idx ->
              BranchMap.add b.branch_id branch_state.assigned.(idx) m
            ) BranchMap.empty all_branches_list (List.init n_branches Fun.id) in
            let template_map = List.fold_left2 (fun (m : int TemplateMap.t) (t : item_template) idx ->
              TemplateMap.add t.item_template_id template_state.assigned.(idx) m
            ) TemplateMap.empty all_templates_list (List.init n_templates Fun.id) in
            { origin_labels = origin_map;
              fact_labels = fact_map;
              branch_labels = branch_map;
              batch_labels = batch_map;
              template_labels = template_map;
              role_labels = ScopedRoleMap.empty;
            }
          in

          let assign_single_collection_branch_order () =
            (* A provisional complete Branch map lets the frozen encoder expose
               the exact suffix after a Branch's own label.  That suffix has no
               Branch-label dependency, so it is safe to sort. *)
            for index = 0 to n_branches - 1 do
              branch_state.assigned.(index) <- index + 1
            done;
            let probe = build_label_assignment () in
            let bodies = List.mapi (fun index (branch, _) ->
              let encoded = encode_branch probe branch in
              let label_prefix = encode_int branch_state.assigned.(index) in
              let prefix_length = String.length label_prefix in
              let body_length = String.length encoded - prefix_length in
              (index, String.sub encoded prefix_length body_length)
            ) all_branches_list in
            let sorted = List.sort (fun (left_index, left_body) (right_index, right_body) ->
              let compared = compare_bytes_lex_unsigned left_body right_body in
              if compared <> 0 then compared else Int.compare left_index right_index
            ) bodies in
            List.iteri (fun label_index (branch_index, _) ->
              branch_state.assigned.(branch_index) <- label_index + 1
            ) sorted
          in

          let enumerate_program_anchor_origin_orders cont =
            (* [entry_origin] may own a non-numeric-minimum label when decimal
               encodings cross 9/10/11.  The remaining labels are nevertheless
               fixed collection positions, so their bodies alone are sorted. *)
            let free_labels = List.filter (fun label ->
              not origin_state.used.(label)
            ) (List.init n_origins (fun index -> index + 1)) in
            let free_indices = List.filter (fun index ->
              not origin_state.fixed.(index)
            ) (List.init n_origins Fun.id) in
            List.iter2 (fun index label ->
              origin_state.assigned.(index) <- label
            ) free_indices free_labels;
            let probe = build_label_assignment () in
            let bodies = List.map (fun index ->
              let (_, site) = List.nth all_origins_list index in
              let encoded = encode_origin_site probe ~origin_scope:Program_scope site in
              let prefix_length = String.length (encode_tag 0) +
                String.length (encode_int origin_state.assigned.(index)) in
              let body_length = String.length encoded - prefix_length in
              (index, String.sub encoded prefix_length body_length)
            ) free_indices in
            let sorted = List.sort (fun (left_index, left_body) (right_index, right_body) ->
              let compared = compare_bytes_lex_unsigned left_body right_body in
              if compared <> 0 then compared else Int.compare left_index right_index
            ) bodies in
            let rec body_groups = function
              | [] -> []
              | (first_index, body) :: rest ->
                  let same, remaining = List.partition (fun (_, candidate) ->
                    candidate = body
                  ) rest in
                  (first_index :: List.map fst same) :: body_groups remaining
            in
            let rec split_at count prefix values =
              if count = 0 then (List.rev prefix, values)
              else match values with
                | [] -> invalid_arg "Anchor tie label partition"
                | value :: rest -> split_at (count - 1) (value :: prefix) rest
            in
            let rec remove_once target prefix = function
              | [] -> invalid_arg "Anchor tie label removal"
              | value :: rest when value = target -> List.rev_append prefix rest
              | value :: rest -> remove_once target (value :: prefix) rest
            in
            let rec enumerate_group indices labels callback =
              match indices with
              | [] -> callback ()
              | index :: rest ->
                  List.iter (fun label ->
                    origin_state.assigned.(index) <- label;
                    enumerate_group rest (remove_once label [] labels) callback
                  ) labels
            in
            let rec enumerate_groups groups labels =
              match groups with
              | [] -> cont ()
              | indices :: rest ->
                  let group_labels, remaining_labels =
                    split_at (List.length indices) [] labels in
                  enumerate_group indices group_labels (fun () ->
                    enumerate_groups rest remaining_labels
                  )
            in
            enumerate_groups (body_groups sorted) free_labels
          in

          let program_role_body_order (la_base : label_assignment) =
            let probe_roles = List.mapi (fun index (role : role) ->
              (Program_role role.role_id, index + 1)
            ) program_roles_list
              |> List.fold_left (fun labels (key, label) ->
                ScopedRoleMap.add key label labels
              ) ScopedRoleMap.empty in
            let probe = { la_base with role_labels = probe_roles } in
            let bodies = List.mapi (fun index (role : role) ->
              let encoded = encode_role probe ~role_scope:Program_scope role in
              let prefix_length = String.length (encode_int (index + 1)) in
              let body_length = String.length encoded - prefix_length in
              (index, String.sub encoded prefix_length body_length)
            ) program_roles_list in
            List.sort (fun (left_index, left_body) (right_index, right_body) ->
              let compared = compare_bytes_lex_unsigned left_body right_body in
              if compared <> 0 then compared else Int.compare left_index right_index
            ) bodies
            |> List.map fst
          in

          let template_role_body_order_if_distinct (la_base : label_assignment)
              (template_id : item_template_id) ~(start_label : int)
              (roles : role list) : int list option =
            let probe_roles = List.mapi (fun index (role : role) ->
              (Template_role (template_id, role.role_id), start_label + index)
            ) roles
              |> List.fold_left (fun labels (key, label) ->
                ScopedRoleMap.add key label labels
              ) ScopedRoleMap.empty in
            let probe = { la_base with role_labels = probe_roles } in
            let sorted = List.mapi (fun index (role : role) ->
              let encoded = encode_role probe
                ~role_scope:(Item_template_scope template_id) role in
              let prefix_length = String.length (encode_int (start_label + index)) in
              let body_length = String.length encoded - prefix_length in
              (index, String.sub encoded prefix_length body_length)
            ) roles
            |> List.sort (fun (left_index, left_body) (right_index, right_body) ->
              let compared = compare_bytes_lex_unsigned left_body right_body in
              if compared <> 0 then compared else Int.compare left_index right_index
            )
            in
            let rec bodies_are_distinct = function
              | [] | [_] -> true
              | (_, left_body) :: (_, right_body) :: rest ->
                  left_body <> right_body && bodies_are_distinct ((0, right_body) :: rest)
            in
            if bodies_are_distinct sorted then Some (List.map fst sorted) else None
          in

          let compute_template_label_order (la : label_assignment) =
            let template_labels = List.map (fun t ->
              (t.item_template_id, lookup_template la t.item_template_id)
            ) all_templates_list in
            List.sort (fun a b -> Int.compare (snd a) (snd b)) template_labels
            |> List.map fst
          in

          let process_assignment (la : label_assignment) =
            incr stats_leaves;
            if !stats_leaves > !stats_max_depth then stats_max_depth := !stats_leaves;
            check_budget ();
            if !budget_exceeded then ()
            else begin
              let payload = encode_program la p in
              if Hashtbl.mem memo_payloads payload then begin
                incr stats_duplicate_hits
              end else begin
                Hashtbl.add memo_payloads payload ();
                if !best_payload = "" || compare_bytes_lex_unsigned payload !best_payload < 0 then begin
                  best_payload := payload;
                  let payload_bytes = Bytes.of_string payload in
                  best_preimage := Bytes.concat Bytes.empty [domain_v2; payload_bytes];
                  best_digest := digest_string_v2 (Tethers_core_canonical_v2_format.sha256_hex !best_preimage)
                end
              end
            end
          in

          let search () =
            (* Fact enumeration — with distinct-scalar optimisation *)
            let enumerate_facts cont =
              match fact_permutation_mode with
              | `Single order ->
                  if n_facts = 0 then (incr stats_nodes; cont ())
                  else begin
                    let inv = Array.make n_facts 0 in
                    List.iteri (fun label_pos fact_idx ->
                      inv.(fact_idx) <- label_pos + 1
                    ) order;
                    for i = 0 to n_facts - 1 do fact_state.assigned.(i) <- inv.(i) done;
                    incr stats_nodes;
                    cont ()
                  end
              | `All ->
                  assign_next_ir fact_state (fun _ ->
                    if !budget_exceeded then () else begin
                      incr stats_nodes;
                      cont ()
                    end)
            in
            let enumerate_origins cont =
              match origin_permutation_mode with
              | `Try_single_program_anchors ->
                  enumerate_program_anchor_origin_orders (fun () ->
                    if not !budget_exceeded then begin
                      incr stats_nodes;
                      cont ()
                    end
                  )
              | `All ->
                  assign_next_ir origin_state (fun _ ->
                    if !budget_exceeded then () else begin
                      incr stats_nodes;
                      cont ()
                    end)
            in
            let enumerate_branches cont =
              match branch_permutation_mode with
              | `Single ->
                  assign_single_collection_branch_order ();
                  incr stats_nodes;
                  cont ()
              | `All ->
                  assign_next_ir branch_state (fun _ ->
                    if !budget_exceeded then () else begin
                      incr stats_nodes;
                      cont ()
                    end)
            in
            enumerate_facts (fun () ->
              if !budget_exceeded then () else
              enumerate_origins (fun () ->
                if !budget_exceeded then () else begin
                  incr stats_nodes;
                  assign_next_ir batch_state (fun _ ->
                    if !budget_exceeded then () else
                    enumerate_branches (fun () ->
                      if !budget_exceeded then () else begin
                        incr stats_nodes;
                        assign_next_ir template_state (fun _ ->
                          if !budget_exceeded then () else begin
                            incr stats_nodes;
                            let la_base = build_label_assignment () in
                            let sorted_template_ids = compute_template_label_order la_base in
                              let program_block = {
                                n_roles = n_program_roles;
                                start_label = 1;
                                perm = Array.make n_program_roles 0;
                              } in
                              let template_blocks = List.mapi (fun idx tid ->
                                let roles = try List.assoc tid template_roles_groups with Not_found -> [] in
                                let n = List.length roles in
                                { n_roles = n;
                                  start_label = 1 + n_program_roles +
                                    (List.filteri (fun i _ -> i < idx) sorted_template_ids
                                     |> List.fold_left (fun acc tid' ->
                                       let n' = try List.length (List.assoc tid' template_roles_groups) with Not_found -> 0 in
                                       acc + n'
                                     ) 0);
                                  perm = Array.make n 0;
                                }
                              ) sorted_template_ids in
                              let all_blocks = Array.of_list (program_block :: template_blocks) in
                              let role_state = make_role_block_state_ir all_blocks in
                              (match program_role_permutation_mode with
                               | `All -> ()
                               | `Single ->
                                   let order = program_role_body_order la_base in
                                   List.iteri (fun label_index role_index ->
                                     fix_label_ir role_state.block_perms.(0)
                                       ~index:role_index ~label:(label_index + 1)
                                   ) order);
                              List.iteri (fun block_index tid ->
                                let template = List.find (fun (t : item_template) ->
                                  t.item_template_id = tid
                                ) all_templates_list in
                                if template_roles_unreferenced_before_own_collection template then begin
                                  let block = role_state.blocks.(block_index + 1) in
                                  let roles = try List.assoc tid template_roles_groups with Not_found -> [] in
                                  match template_role_body_order_if_distinct la_base tid
                                    ~start_label:block.start_label roles with
                                  | None -> ()
                                  | Some order ->
                                      List.iteri (fun label_index role_index ->
                                        fix_label_ir role_state.block_perms.(block_index + 1)
                                          ~index:role_index
                                          ~label:(label_index + 1)
                                      ) order
                                end
                              ) sorted_template_ids;
                              enumerate_role_blocks_ir role_state (fun blocks ->
                                if !budget_exceeded then () else begin
                                  let role_map = ref ScopedRoleMap.empty in
                                  List.iteri (fun idx r ->
                                    let label = blocks.(0).start_label + blocks.(0).perm.(idx) - 1 in
                                    role_map := ScopedRoleMap.add (Program_role r.role_id) label !role_map
                                  ) program_roles_list;
                                  List.iteri (fun block_idx tid ->
                                    let roles = try List.assoc tid template_roles_groups with Not_found -> [] in
                                    let block = blocks.(block_idx + 1) in
                                    List.iteri (fun idx r ->
                                      let label = block.start_label + block.perm.(idx) - 1 in
                                      role_map := ScopedRoleMap.add (Template_role (tid, r.role_id)) label !role_map
                                    ) roles
                                  ) sorted_template_ids;
                                  let la = { la_base with role_labels = !role_map } in
                                  process_assignment la
                                end
                              )
                          end
                        )
                      end
                    )
                  )
                end
              )
            )
          in
          search ();
          if !budget_exceeded then Error Canonicalisation_too_complex
          else if !best_payload = "" then Error Canonicalisation_too_complex
          else
            let stats = {
              nodes = !stats_nodes;
              leaves_encoded = !stats_leaves;
              refinement_rounds = refinement_rounds;
              prefix_subtrees_pruned = !stats_prefix_pruned;
              orbit_branches_pruned = !stats_orbit_pruned;
              duplicate_payload_hits = !stats_duplicate_hits;
              max_depth = !stats_max_depth;
              leaves_avoided = (match raw_candidate_count with
                | Some count -> max 0 (count - !stats_leaves)
                | None -> 0);
            } in
            Ok ({
              validated_program = p;
              canonical_payload = !best_payload;
              canonical_preimage = !best_preimage;
              canonical_program_digest = !best_digest;
            }, stats)
      end
  end

let validated_program_ir (r : canonicalized_v2_ir) : Tethers_core.program =
  r.validated_program

let canonical_payload_ir (r : canonicalized_v2_ir) : string = r.canonical_payload
let canonical_preimage_ir (r : canonicalized_v2_ir) : bytes = r.canonical_preimage
let program_digest_ir (r : canonicalized_v2_ir) : string = r.canonical_program_digest
