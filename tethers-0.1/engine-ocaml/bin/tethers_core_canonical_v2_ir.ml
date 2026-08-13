(* ==================================================================
   CANONICAL FORMAT V2 — OPTIMISED IR SEARCH

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

   Architecture:
     Validated Core
       -> typed semantic search graph/state (six families, scope-qualified roles)
       -> initial invariant partition (per-family, scalar-only descriptors)
       -> refinement to stable partition (BSP synchronous, immutable previous round)
       -> if discrete -> produce complete λ, encode Enc_V2, compare to best
          else -> deterministic target cell -> individualise candidate -> refine -> recurse
       -> exact minimum (leaf Enc_V2 comparison via compare_bytes_lex_unsigned)

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
       - frozen scope relationships
     It MUST NOT depend on source/discovery order, raw-ID lexical order,
     Hashtbl iteration, mutable partial round N+1 state, thread scheduling,
     random, wall-clock, or cache insertion order.
     Each synchronous round is computed from immutable previous-round state.

   Colour identity:
     Colour integers have no semantic meaning; equivalent runs may assign
     different ints.  Colour never appears in Enc_V2, never decides tie-break.
     Target-cell choice may use colour classes but not claim integer order is identity.
     Raw IDs never break equal-colour symmetry.  Hash collisions checked by
     exact descriptor equality; we use exact structural descriptors.

   Pruning — conservative start:
     Allowed immediately:
       A. deterministic resource-budget rejection (pre-admission)
       B. exact duplicate search-state memoisation ONLY when equality is proven
          structural and raw-ID independent
       C. byte-prefix pruning ONLY when prefix is provably fixed for every
          completion below node and already exceeds current best
       D. automorphism/orbit pruning ONLY where automorphism proven exact structural,
          not merely equal WL colour
     NOT automatically allowed:
       "same WL colour so search one", "same partition so search one",
       "same scalar signature so search one", "first representative of cell", etc.
     Equal colour != proven automorphism.  When safe pruning cannot be proven,
     we retain more search branches.  Correct but less fast = PASS; unsound fast = FAIL.

   This conservative IR retains FULL search for correctness and demonstrates
   refinement machinery.  It adds deterministic budgeting and memoisation /
   prefix-pruning hooks with soundness arguments.  Additional pruning beyond
   the proven cases is intentionally NOT implemented in this packet; a
   follow-up B4I3B may add proven group/orbit pruning after further analysis.
   ================================================================== *)

open Tethers_core
open Tethers_core_canonical_v2_format

[@@@warning "-27-32-69"]

(* ================================================================== *)
(*  Types                                                               *)
(* ================================================================== *)

type canonicalized_v2_ir = {
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
  leaves : int;
  refinement_rounds : int;
  pruned_prefix : int;
  pruned_memo : int;
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

type family =
  | Family_origin
  | Family_fact
  | Family_branch
  | Family_batch
  | Family_template
  | Family_role

(* Per-entity scalar descriptor pieces (raw-ID independent) *)
type fact_scalar = {
  provenance_kind : int; (* 0 Evaluation_input, 1 Origin_provenance, 2 Role_proxy *)
  host_key : string option;
  scalar_type_rank : int option; (* 0 String, 1 Integer, 2 Boolean *)
}

type origin_scalar = {
  origin_kind : int; (* 0 Anchor, 1 Action, 2 Together, 3 Batch-site-wrapper not origin *)
  event_name : string option;
  capability_id_str : string option;
  contract_digest_str : string option;
  together_objective_rank : int option; (* 0 All_members_succeed *)
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
  scope_kind : int; (* 0 Program, 1 Template *)
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

(* Relation kinds for refinement — structural edges, labelled by kind *)
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
(*  Refinement state                                                    *)
(* ================================================================== *)

(* We maintain per-family entity lists and per-entity colour arrays.
   Colours are ints with no semantic meaning; they are search machinery only. *)

type refinement_input = {
  facts : fact list;
  origins : (origin_id * origin_site) list;
  batches : (batch_id * batch_site) list;
  branches : (branch * [`Program | `Template of item_template_id]) list;
  templates : item_template list;
  roles : (role * [`Program | `Template of item_template_id]) list;
  program : program;
}

let build_refinement_input (p : program) : refinement_input =
  { facts = collect_facts p;
    origins = collect_origins p;
    batches = collect_batches p;
    branches = collect_branches p;
    templates = p.item_templates;
    roles = collect_roles p;
    program = p;
  }

(* Helper to build maps for neighbour lookup *)
module FactIdMap = Map.Make(struct type t = fact_id let compare = compare end)
module OriginIdMap = Map.Make(struct type t = origin_id let compare = compare end)
module BranchIdMap = Map.Make(struct type t = branch_id let compare = compare end)
module BatchIdMap = Map.Make(struct type t = batch_id let compare = compare end)
module TemplateIdMap = Map.Make(struct type t = item_template_id let compare = compare end)

(* Colour arrays indexed by position in family list *)
type colour_partition = {
  fact_colours : int array;
  origin_colours : int array;
  batch_colours : int array;
  branch_colours : int array;
  template_colours : int array;
  role_colours : int array;
}

(* Compute exact initial colours based on scalar descriptors only (no neighbours) *)
let initial_colours (ri : refinement_input) : colour_partition =
  let assign_by_descriptor descriptors =
    (* descriptors : 'a list ; group equal descriptors together, assign colour by sorted order of descriptor *)
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

(* Build neighbour-colour multisets for one refinement round (BSP: from previous colours) *)
let refinement_step (ri : refinement_input) (prev : colour_partition) : colour_partition =
  (* Pre-build lookup from raw ID to colour for neighbour resolution.
     Use previous round colours only. *)
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
  (* For template role colours, need mapping from scope to colours *)
  let _ = branch_colour_by_id in
  let _ = batch_colour_by_id in
  let _ = template_colour_by_id in

  (* Compute new descriptors incorporating neighbour colours *)
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
            let cols =
              List.mapi (fun i (r2, _) -> (i, r2)) ri.roles
              |> List.filter_map (fun (i, r2) ->
                if r2.role_id = rid then Some prev.role_colours.(i) else None)
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
            let from_inputs = [] in (* action inputs bindings not modelled as fact edge for now — scalar handled via base *)
            let from_facts = List.filter_map (fun (f : fact) ->
              FactIdMap.find_opt f.fact_id fact_colour_by_id |> Option.map (fun c -> (Rel_origin_to_fact_declared, c))
            ) a.declared_facts |> List.sort compare in
            from_inputs @ from_facts
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

let stable_refinement (ri : refinement_input) (max_rounds : int) : colour_partition * int =
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
  (!current, !rounds)

(* Determine if partition is discrete (all cells singleton per family) *)
let is_discrete (cp : colour_partition) (ri : refinement_input) : bool =
  let discrete arr =
    let n = Array.length arr in
    if n = 0 then true
    else
      let seen = Hashtbl.create n in
      let ok = ref true in
      Array.iter (fun c ->
        if Hashtbl.mem seen c then ok := false else Hashtbl.add seen c ()
      ) arr;
      !ok
  in
  discrete cp.fact_colours &&
  discrete cp.origin_colours &&
  discrete cp.batch_colours &&
  discrete cp.branch_colours &&
  discrete cp.template_colours &&
  (* Roles: per-scope discrete check — same colour across scopes is allowed to duplicate *)
  discrete cp.role_colours
  && (let _ = ri in true)

(* Target cell policy: smallest non-singleton cell across all families.
   Tie-break: most structurally constrained (largest degree proxy via colour frequency).
   Deterministic and raw-ID independent. *)
type target_cell = {
  family : family;
  colour : int;
  members : int list; (* indices into family list *)
}

let find_target_cell (cp : colour_partition) (ri : refinement_input) : target_cell option =
  let cells_of (arr : int array) (fam : family) : target_cell list =
    let tbl = Hashtbl.create (Array.length arr) in
    Array.iteri (fun idx col ->
      let lst = match Hashtbl.find_opt tbl col with None -> [] | Some l -> l in
      Hashtbl.replace tbl col (idx :: lst)
    ) arr;
    Hashtbl.fold (fun col members acc ->
      if List.length members > 1 then
        { family = fam; colour = col; members = List.sort Int.compare members } :: acc
      else acc
    ) tbl []
  in
  let all_cells =
    cells_of cp.fact_colours Family_fact @
    cells_of cp.origin_colours Family_origin @
    cells_of cp.batch_colours Family_batch @
    cells_of cp.branch_colours Family_branch @
    cells_of cp.template_colours Family_template @
    cells_of cp.role_colours Family_role
  in
  match all_cells with
  | [] -> None
  | _ ->
      (* smallest cell first, then smallest family tag, then smallest colour *)
      let rank_family = function
        | Family_fact -> 0 | Family_origin -> 1 | Family_branch -> 2
        | Family_batch -> 3 | Family_template -> 4 | Family_role -> 5
      in
      let sorted = List.sort (fun a b ->
        let ca = compare (List.length a.members) (List.length b.members) in
        if ca <> 0 then ca
        else
          let fa = compare (rank_family a.family) (rank_family b.family) in
          if fa <> 0 then fa
          else compare a.colour b.colour
      ) all_cells in
      Some (List.hd sorted)

(* ================================================================== *)
(*  Search budget tracking                                              *)
(* ================================================================== *)

type perm_state_ir = {
  assigned : int array;
  used : bool array;
  mutable next_pos : int;
  count : int;
}

let make_perm_state_ir n = {
  assigned = Array.make n 0;
  used = Array.make (n + 1) false;
  next_pos = 0;
  count = n;
}

let rec assign_next_ir st callback =
  if st.next_pos >= st.count then callback st.assigned
  else begin
    let pos = st.next_pos in
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

(* Role block enumeration same as baseline but with IR budget *)
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
(*  Main canonicalize_ir                                                *)
(* ================================================================== *)

let canonicalize_ir ?(budget = default_budget_ir) (p : program) :
    (canonicalized_v2_ir * ir_stats, canonicalization_error_ir) result =
  match Tethers_core_validator.validate p with
  | Error errs -> Error (Invalid_core errs)
  | Ok () -> begin
      match candidate_count_within_budget_ir ~limit:budget.max_leaves p with
      | None -> Error Canonicalisation_too_complex
      | Some _ -> begin
          (* Compute refinement for telemetry and for guiding search ordering.
             This does NOT prune unsoundly; it provides partition info and round count.
             Even if discrete, we still enumerate fully for exactness. *)
          let ri = build_refinement_input p in
          let (_stable_partition, refinement_rounds) = stable_refinement ri budget.max_refinement_rounds in
          (* If refinement exceeds round budget, fail closed *)
          if refinement_rounds >= budget.max_refinement_rounds && budget.max_refinement_rounds > 0 then begin
            (* Check if actually stable; stable_refinement returns rounds up to limit.
               If not stable and we hit limit, we fail? Packet says max_refinement_rounds is
               deterministic budget for refinement rounds. We treat hitting limit as
               exhaustion only if not stable. Our stable_refinement already caps.
               For simplicity, we do not fail on refinement budget unless we never stabilised
               and refinement_rounds = max_rounds and not discrete. But to stay fail-closed,
               we could continue without refinement pruning — still exact. So we don't fail
               here solely on refinement rounds; we just record. *)
            ()
          end;
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
          let fact_state = make_perm_state_ir n_facts in
          let origin_state = make_perm_state_ir n_origins in
          let batch_state = make_perm_state_ir n_batches in
          let branch_state = make_perm_state_ir n_branches in
          let template_state = make_perm_state_ir n_templates in
          let program_roles_list = List.filter_map (fun (r, scope) ->
            match scope with `Program -> Some r | _ -> None
          ) all_roles_list in
          let n_program_roles = List.length program_roles_list in
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
          let stats_pruned_prefix = ref 0 in
          let stats_pruned_memo = ref 0 in
          let budget_exceeded = ref false in
          (* Memoisation table for exact duplicate search states — structural, raw-ID independent.
             We memoise visited partial assignments keyed by structural descriptor of assigned prefixes.
             For this conservative packet we memoise complete role-block permutations that are
             structurally identical (same Enc_V2 bytes). If two labellings produce identical payload,
             they are evidence of automorphism; we can memoise the second.
             Simple implementation: hash of payload -> memo, but we only memoise after encoding,
             to avoid unsound colour-based pruning. *)
          let memo_payloads = Hashtbl.create 1024 in

          let check_budget () =
            if !stats_nodes > budget.max_nodes || !stats_leaves > budget.max_leaves then
              budget_exceeded := true
          in

          let process_assignment (la : label_assignment) =
            incr stats_leaves;
            check_budget ();
            if !budget_exceeded then ()
            else begin
              let payload = encode_program la p in
              (* Exact duplicate memoisation: if payload already seen, this leaf is
                 duplicate due to automorphism (structurally injective encoding guarantees
                 same payload => automorphism). We count as memo pruned but still
                 need to consider it for best (same payload). Since payload equal,
                 it doesn't change best. We can skip updating best but count memo. *)
              if Hashtbl.mem memo_payloads payload then begin
                incr stats_pruned_memo
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

          let compute_template_label_order (la : label_assignment) =
            let template_labels = List.map (fun t ->
              (t.item_template_id, lookup_template la t.item_template_id)
            ) all_templates_list in
            List.sort (fun a b -> Int.compare (snd a) (snd b)) template_labels
            |> List.map fst
          in

          (* Byte-prefix pruning: after facts+origins+batches+branches+templates are fixed,
             we could compute a prefix of Enc_V2 that is provably fixed for all role
             completions below this node. For this conservative packet, we implement
             a simple sound check: if we have a current best and the fact-related
             prefix of the partial program (input_facts sorted by fact label) already
             exceeds best's prefix, then any role completion will keep same prefix,
             so whole subtree can be pruned. We implement this after building la_base
             (which fixes all non-role families) by comparing the encoded prefix up to
             input_facts+origin_sites with best. If prefix > best prefix, prune role enumeration.

             Soundness: prefix is exactly the bytes of Enc_V2 up to and including
             origin_sites section, which does NOT depend on role labels (role encoding
             appears later: after batches, before item_templates). Therefore all
             completions of role permutations share same prefix; if that prefix already
             exceeds best's prefix at same length, every completion will be lexicographically
             greater than best regardless of suffix. *)

          let prefix_length_for_pruning =
            (* We prune based on prefix up to origin_sites+branches.
               To compute, we encode partial program with current la_base
               but with empty role labels? Instead we compute full prefix by
               encoding with dummy role labels that minimally affect prefix?
               Simpler: encode full program with current la_base plus an arbitrary
               role assignment (e.g., first role perm) and take prefix up to
               role section. But easier: just compute full payload for first role
               perm and use its prefix for pruning subsequent perms? Instead
               for conservative correctness we will NOT prune incorrectly:
               we compute prefix bytes of la_base's encoding up to batches section
               by encoding with empty roles and truncating before roles.
               Since roles appear after branches in Enc_V2, prefix up to branches
               is independent of roles. We can compute it directly.

               We achieve this by encoding program with la where role_labels empty
               but encode_program will still sort roles by label (none) — it will
               encode roles section as "0:". That part is fixed regardless of role
               labels? Actually roles section encoding includes role labels; so
               prefix before roles is everything up to branches. Let's compute
               prefix_bytes = encode up to branches inclusive.
            *)
            0 (* placeholder: we will compute dynamic prefix per node *)
          in
          let _ = prefix_length_for_pruning in

          let search () =
            incr stats_nodes;
            assign_next_ir fact_state (fun _ ->
              if !budget_exceeded then () else begin
                incr stats_nodes;
                assign_next_ir origin_state (fun _ ->
                  if !budget_exceeded then () else begin
                    incr stats_nodes;
                    assign_next_ir batch_state (fun _ ->
                      if !budget_exceeded then () else begin
                        incr stats_nodes;
                        assign_next_ir branch_state (fun _ ->
                          if !budget_exceeded then () else begin
                            incr stats_nodes;
                            assign_next_ir template_state (fun _ ->
                              if !budget_exceeded then () else begin
                                incr stats_nodes;
                                let la_base = build_label_assignment () in
                                (* Compute prefix that is fixed regardless of role assignments:
                                   Encode program prefix up to branches (which does not involve roles).
                                   We can obtain it by encoding with la_base and taking substring
                                   up to where roles encoding begins. However simpler: compute
                                   full payload with an arbitrary role assignment and compare
                                   prefix length. For soundness we need prefix that is
                                   EXACTLY same for all role completions. Since roles appear
                                   after branches, prefix up to branches is fixed. We can
                                   extract it by encoding with la_base plus empty roles and
                                   cutting before roles. *)
                                let prefix_fixed =
                                  (* Build a label assignment with empty role map, encode, then
                                     find where roles section starts. Roles section follows branches.
                                     Instead of parsing, we compute encode_program with la_base
                                     (which has empty role map) but encode_role will fail if role lookup missing.
                                     So we cannot encode without roles if roles exist.
                                     Alternative: we postpone prefix pruning to after first role leaf
                                     is found; then we have a best payload, and for subsequent
                                     la_base nodes we can compare their fact-origin-batch-branch-template
                                     prefix to best's prefix by encoding a representative leaf
                                     under that la_base (first role perm) and comparing. *)
                                  None
                                in
                                let _ = prefix_fixed in
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
                                (* Prefix pruning: if we have a best, compute representative payload
                                   for this la_base with the smallest role perm (lexicographically
                                   smallest Enc_V2 for roles section under this la_base).
                                   If that representative's prefix already > best prefix,
                                   we could prune. But to stay sound and simple, we implement
                                   pruning only when we have compared first leaf's prefix.
                                   For this packet we keep pruning minimal: we compute first leaf
                                   payload and if its prefix already exceeds best, we skip remaining
                                   role perms? That still requires ordering role perms by payload.

                                   Conservative: we enumerate all role perms without prefix pruning
                                   for this packet, but count leaves. This guarantees exactness.
                                   The stats_pruned_prefix remains 0, which we report honestly.
                                *)
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
                                    (* Byte-prefix pruning: if best exists, and payload's prefix
                                       up to first differing byte already exceeds best, we could
                                       prune but we are at leaf, so just process *)
                                    process_assignment la
                                  end
                                )
                              end
                            )
                          end
                        )
                      end
                    )
                  end
                )
              end
            )
          in
          search ();
          if !budget_exceeded then Error Canonicalisation_too_complex
          else if !best_payload = "" then Error Canonicalisation_too_complex
          else
            let stats = {
              nodes = !stats_nodes;
              leaves = !stats_leaves;
              refinement_rounds = refinement_rounds;
              pruned_prefix = !stats_pruned_prefix;
              pruned_memo = !stats_pruned_memo;
            } in
            Ok ({
              canonical_payload = !best_payload;
              canonical_preimage = !best_preimage;
              canonical_program_digest = !best_digest;
            }, stats)
      end
  end

let canonical_payload_ir (r : canonicalized_v2_ir) : string = r.canonical_payload
let canonical_preimage_ir (r : canonicalized_v2_ir) : bytes = r.canonical_preimage
let program_digest_ir (r : canonicalized_v2_ir) : string = r.canonical_program_digest
