(* ==================================================================
   C-B1 REFINEMENT STRATEGY BENCHMARK
   
   Compares the current round-based refinement against a topological
   multi-pass strategy that processes entities in dependency order.
   
   NOT production code. Proof-of-concept only.
   ================================================================== *)

open Tethers_core

(* Use the canonical module's StringMap so colour_map types are compatible *)
module StringMap = Tethers_core_canonical.StringMap
module IntSet = Set.Make(Int)

let safety_cap = 1000

(* Entity collection - copied from production *)
let origin_id_of_site = function
  | Anchor_origin a -> Some a.anchor_origin_id
  | Action_origin a -> Some a.action_origin_id
  | Together_origin t -> Some t.together_origin_id
  | Batch_site _ -> None

let all_origins p =
  let prog =
    List.filter_map (fun s ->
      match origin_id_of_site s with
      | Some id -> Some (id, s, `Program, None)
      | None -> None) p.origin_sites
  in
  let tmpl =
    List.concat_map (fun t ->
      List.filter_map (fun s ->
        match origin_id_of_site s with
        | Some id -> Some (id, s, `Template t.item_template_id, Some t.item_template_id)
        | None -> None) t.origin_sites
    ) p.item_templates
  in
  prog @ tmpl

let all_batches p =
  let prog =
    List.filter_map (fun s ->
      match s with Batch_site b -> Some (b.batch_id, s, `Program, None) | _ -> None
    ) p.origin_sites
  in
  let tmpl =
    List.concat_map (fun t ->
      List.filter_map (fun s ->
        match s with Batch_site b -> Some (b.batch_id, s, `Template t.item_template_id, Some t.item_template_id) | _ -> None
      ) t.origin_sites
    ) p.item_templates
  in
  prog @ tmpl

let all_branches p =
  let prog = List.map (fun b -> (b, `Program)) p.branches in
  let tmpl = List.concat_map (fun t ->
    List.map (fun b -> (b, `Template t.item_template_id)) t.branches
  ) p.item_templates in
  prog @ tmpl

let all_roles p =
  let prog = List.map (fun r -> (r, `Program)) p.roles in
  let tmpl = List.concat_map (fun t ->
    List.map (fun r -> (r, `Template t.item_template_id)) t.roles
  ) p.item_templates in
  prog @ tmpl

let all_facts p =
  let from_origins =
    List.concat_map (fun (_, s, _, _) ->
      match s with
      | Anchor_origin a -> a.declared_facts
      | Action_origin a -> a.declared_facts
      | Together_origin _ -> []
      | Batch_site b -> b.aggregate_facts
    ) (all_origins p)
  in
  let from_batches =
    List.concat_map (fun (_, s, _, _) ->
      match s with
      | Batch_site b -> b.aggregate_facts
      | _ -> []
    ) (all_batches p)
  in
  p.input_facts @ from_origins @ from_batches

let all_origin_sites_flat p =
  p.origin_sites @ List.flatten (List.map (fun (t : item_template) -> t.origin_sites) p.item_templates)

let all_branches_flat p =
  p.branches @ List.flatten (List.map (fun (t : item_template) -> t.branches) p.item_templates)

let scoped_role_id r scope =
  match scope with
  | `Program -> "P:" ^ string_of_role_id r.role_id
  | `Template tid -> "T:" ^ string_of_item_template_id tid ^ ":" ^ string_of_role_id r.role_id

(* Key helpers - copied from production *)
let key_of_value = function
  | String_value s -> "sv:" ^ string_of_int (String.length s) ^ ":" ^ s
  | Integer_value i -> "iv:" ^ string_of_int i
  | Boolean_value b -> "bv:" ^ (if b then "1" else "0")

let key_of_outcome = function
  | Success -> "out_s"
  | Failure -> "out_f"
  | Uncertain -> "out_u"
  | Cancelled -> "out_c"

let key_of_provenance = function
  | Evaluation_input (Host_snapshot_key k, t) ->
      "prov_eval:" ^ k ^ ":" ^ (
        match t with String_type -> "S" | Integer_type -> "I" | Boolean_type -> "B")
  | Origin_provenance _ -> "prov_origin"
  | Role_proxy _ -> "prov_role"

let key_of_together_objective = function
  | All_members_succeed -> "tog_all"

let key_of_role_fulfillment (Role_fulfillment s) =
  "fulfill:" ^ string_of_int (String.length s) ^ ":" ^ s

let key_of_constraint = function
  | Deadline s -> "deadline:" ^ string_of_int (String.length s) ^ ":" ^ s

let key_of_batch_collection_provenance (Batch_collection_provenance s) =
  "bcp:" ^ string_of_int (String.length s) ^ ":" ^ s

let key_of_batch_traversal_policy (Batch_traversal_policy s) =
  "btp:" ^ string_of_int (String.length s) ^ ":" ^ s

let key_of_batch_objective (Batch_objective s) =
  "bo:" ^ string_of_int (String.length s) ^ ":" ^ s

let operator_rank = function
  | Equals -> 0
  | Contains -> 1
  | Greater_than -> 2
  | Greater_than_or_equal -> 3

(* Use the canonical module's colour_map type for compatibility *)
type colour_map = Tethers_core_canonical.colour_map = {
  fact_colours : int StringMap.t;
  origin_colours : int StringMap.t;
  batch_colours : int StringMap.t;
  role_colours : int StringMap.t;
  branch_colours : int StringMap.t;
  item_template_colours : int StringMap.t;
}

(* Compress colours *)
let compress_colours pairs =
  let uniques =
    pairs |> List.map snd |> List.sort_uniq String.compare
  in
  let sig_to_colour =
    List.fold_left (fun m sig_str ->
      let colour = StringMap.cardinal m + 1 in
      StringMap.add sig_str colour m
    ) StringMap.empty uniques
  in
  List.fold_left (fun m (id, sig_str) ->
    StringMap.add id (StringMap.find sig_str sig_to_colour) m
  ) StringMap.empty pairs

(* Partition stability check *)
let int_map_partition_stable prev next =
  let count_unique m =
    StringMap.fold (fun _k v acc -> IntSet.add v acc) m IntSet.empty
    |> IntSet.cardinal
  in
  let n_prev = count_unique prev in
  let n_next = count_unique next in
  if n_prev <> n_next then false
  else
    let prev_classes =
      StringMap.fold (fun id c acc ->
        let key = string_of_int c in
        StringMap.update key (function None -> Some [id] | Some ids -> Some (id :: ids)) acc
      ) prev StringMap.empty
    in
    StringMap.for_all (fun _ members ->
      match members with
      | [] | [_] -> true
      | first :: rest ->
          let first_next = StringMap.find first next in
          List.for_all (fun id -> StringMap.find id next = first_next) rest
    ) prev_classes

let partitions_equal a b =
  int_map_partition_stable a.fact_colours b.fact_colours
  && int_map_partition_stable a.origin_colours b.origin_colours
  && int_map_partition_stable a.batch_colours b.batch_colours
  && int_map_partition_stable a.role_colours b.role_colours
  && int_map_partition_stable a.branch_colours b.branch_colours
  && int_map_partition_stable a.item_template_colours b.item_template_colours

(* Colour lookups *)
let lookup_fact_c colours fid =
  match StringMap.find_opt (string_of_fact_id fid) colours.fact_colours with
  | Some c -> string_of_int c | None -> "0"

let lookup_origin_c colours oid =
  match StringMap.find_opt (string_of_origin_id oid) colours.origin_colours with
  | Some c -> string_of_int c | None -> "0"

let lookup_batch_c colours bid =
  match StringMap.find_opt (string_of_batch_id bid) colours.batch_colours with
  | Some c -> string_of_int c | None -> "0"

let lookup_role_c colours scoped_key =
  match StringMap.find_opt scoped_key colours.role_colours with
  | Some c -> string_of_int c | None -> "0"

let lookup_branch_c colours bid =
  match StringMap.find_opt (string_of_branch_id bid) colours.branch_colours with
  | Some c -> string_of_int c | None -> "0"

let lookup_item_template_c colours tid =
  match StringMap.find_opt (string_of_item_template_id tid) colours.item_template_colours with
  | Some c -> string_of_int c | None -> "0"

let lookup_role_colour_in_scope colours rid = function
  | `Program -> lookup_role_c colours ("P:" ^ string_of_role_id rid)
  | `Template tid -> lookup_role_c colours ("T:" ^ string_of_item_template_id tid ^ ":" ^ string_of_role_id rid)

(* Static reference maps *)
type static_refs = {
  entry_origin_id_str : string option;
  success_out_map : control_target list StringMap.t;
  success_in_map : origin_id list StringMap.t;
  origin_branches : branch_id list StringMap.t;
  together_for_member : origin_id StringMap.t;
  guards_for_fact : (comparison_operator * core_value) list StringMap.t;
  consumers_for_fact_scoped :
    (capability_input_name * input_binding * [`Program | `Template of item_template_id]) list StringMap.t;
  origin_for_fact : origin_id option StringMap.t;
  fact_scope_map : [`Program | `Template of item_template_id] StringMap.t;
}

let build_static_refs p =
  let entry_origin_id_str = Option.map string_of_origin_id p.entry_origin in
  let success_out_map =
    List.fold_left (fun m sc ->
      let from_s = string_of_origin_id sc.from_origin in
      StringMap.update from_s (function
        | None -> Some [sc.target]
        | Some lst -> Some (sc.target :: lst)) m
    ) StringMap.empty p.success_continuations
  in
  let success_in_map =
    List.fold_left (fun m sc ->
      match sc.target with
      | Origin_target tgt ->
        let tgt_s = string_of_origin_id tgt in
        StringMap.update tgt_s (function
          | None -> Some [sc.from_origin]
          | Some lst -> Some (sc.from_origin :: lst)) m
      | Program_complete -> m
    ) StringMap.empty p.success_continuations
  in
  let origin_branches =
    List.fold_left (fun m (b : branch) ->
      let oid_s = string_of_origin_id b.branch_subject in
      StringMap.update oid_s (function
        | None -> Some [b.branch_id]
        | Some lst -> Some (b.branch_id :: lst)) m
    ) StringMap.empty (all_branches_flat p)
  in
  let all_sites = all_origin_sites_flat p in
  let together_for_member =
    List.fold_left (fun m site ->
      match site with
      | Together_origin t ->
        List.fold_left (fun m' member ->
          StringMap.add (string_of_origin_id member) t.together_origin_id m'
        ) m t.member_origin_ids
      | _ -> m
    ) StringMap.empty all_sites
  in
  let guards_for_fact =
    List.fold_left (fun m g ->
      let fid_s = string_of_fact_id g.fact_id in
      StringMap.update fid_s (function
        | None -> Some [(g.operator, g.expected)]
        | Some lst -> Some ((g.operator, g.expected) :: lst)) m
    ) StringMap.empty p.entry_guards
  in
  let consumers_for_fact_scoped =
    List.fold_left (fun m (_oid, site, scope, _) ->
      match site with
      | Action_origin a ->
        List.fold_left (fun m' ai ->
          match ai.binding with
          | Fact_from_origin (fid, _) | Fact_through_role (fid, _) ->
            let fid_s = string_of_fact_id fid in
            StringMap.update fid_s (function
              | None -> Some [(ai.input_name, ai.binding, scope)]
              | Some lst -> Some ((ai.input_name, ai.binding, scope) :: lst)) m'
          | _ -> m'
        ) m a.inputs
      | _ -> m
    ) StringMap.empty (all_origins p)
  in
  let origin_for_fact =
    let m = ref StringMap.empty in
    let add_origin oid (declared : fact list) =
      List.iter (fun (f : fact) ->
        m := StringMap.add (string_of_fact_id f.fact_id) (Some oid) !m
      ) declared
    in
    List.iter (fun site ->
      match site with
      | Anchor_origin a -> add_origin a.anchor_origin_id a.declared_facts
      | Action_origin a -> add_origin a.action_origin_id a.declared_facts
      | _ -> ()
    ) all_sites;
    List.iter (fun (f : fact) ->
      if not (StringMap.mem (string_of_fact_id f.fact_id) !m) then
        m := StringMap.add (string_of_fact_id f.fact_id) None !m
    ) (all_facts p);
    !m
  in
  let fact_scope_map =
    let m = ref StringMap.empty in
    List.iter (fun (_oid, site, scope, _) ->
      let facts = match site with
        | Anchor_origin a -> a.declared_facts
        | Action_origin a -> a.declared_facts
        | _ -> []
      in
      List.iter (fun (f : fact) ->
        m := StringMap.add (string_of_fact_id f.fact_id) scope !m
      ) facts
    ) (all_origins p);
    List.iter (fun (_bid, site, scope, _) ->
      match site with
      | Batch_site b ->
        List.iter (fun (f : fact) ->
          m := StringMap.add (string_of_fact_id f.fact_id) scope !m
        ) b.aggregate_facts
      | _ -> ()
    ) (all_batches p);
    List.iter (fun (f : fact) ->
      if not (StringMap.mem (string_of_fact_id f.fact_id) !m) then
        m := StringMap.add (string_of_fact_id f.fact_id) `Program !m
    ) (all_facts p);
    !m
  in
  { entry_origin_id_str; success_out_map; success_in_map;
    origin_branches; together_for_member; guards_for_fact;
    consumers_for_fact_scoped; origin_for_fact; fact_scope_map }

(* Round 0 signatures *)
let fact_sig_r0 (f : fact) =
  "F:" ^ key_of_provenance f.provenance

let origin_sig_r0 (_oid, site, _scope, _tmpl) =
  match site with
  | Anchor_origin a -> "A:" ^ a.event_name
  | Action_origin a ->
      "Ac:" ^ string_of_capability_id a.capability_id ^ ":"
      ^ string_of_capability_contract_digest a.contract_digest
  | Together_origin t -> "T:" ^ key_of_together_objective t.objective
  | Batch_site b ->
      "Ba:" ^ key_of_batch_collection_provenance b.collection_provenance ^ ":"
      ^ key_of_batch_traversal_policy b.traversal_policy ^ ":"
      ^ key_of_batch_objective b.composite_objective

let role_sig_r0 (r, _scope) =
  let (Role_fact_contract fids) = r.fact_contract in
  "Ro:" ^ key_of_role_fulfillment r.eligible_fulfillment ^ ":"
  ^ string_of_int (List.length fids)

let branch_sig_r0 (b, _scope) =
  "Br:" ^ string_of_int (List.length b.outcome_branches)

let item_template_sig_r0 (t : item_template) =
  let obj = match t.objective with Required_role _ -> "RR" in
  "IT:" ^ obj ^ ":" ^ string_of_int (List.length t.origin_sites) ^ ":"
  ^ string_of_int (List.length t.branches) ^ ":"
  ^ string_of_int (List.length t.roles)

let round_0 p =
  let fact_colours =
    List.map (fun (f : fact) -> (string_of_fact_id f.fact_id, fact_sig_r0 f)) (all_facts p)
    |> compress_colours
  in
  let origin_colours =
    List.map (fun (oid, site, scope, tmpl) ->
      (string_of_origin_id oid, origin_sig_r0 (oid, site, scope, tmpl))) (all_origins p)
    |> compress_colours
  in
  let batch_colours =
    List.map (fun (bid, site, scope, tmpl) ->
      (string_of_batch_id bid, origin_sig_r0 (origin_id_of_string "", site, scope, tmpl))) (all_batches p)
    |> compress_colours
  in
  let role_colours =
    List.map (fun (r, scope) ->
      (scoped_role_id r scope, role_sig_r0 (r, scope))) (all_roles p)
    |> compress_colours
  in
  let branch_colours =
    List.map (fun (b, scope) ->
      (string_of_branch_id b.branch_id, branch_sig_r0 (b, scope))) (all_branches p)
    |> compress_colours
  in
  let item_template_colours =
    List.map (fun (t : item_template) ->
      (string_of_item_template_id t.item_template_id, item_template_sig_r0 t)) p.item_templates
    |> compress_colours
  in
  { fact_colours; origin_colours; batch_colours; role_colours;
    branch_colours; item_template_colours }

(* Round N signatures - copied from production *)
let control_target_cstr colours = function
  | Origin_target oid -> "C:" ^ lookup_origin_c colours oid
  | Program_complete -> "PC"

let fact_sig_rn colours refs (f : fact) =
  let prov_str = match f.provenance with
    | Evaluation_input (Host_snapshot_key k, t) ->
        "E:" ^ k ^ ":" ^ (
          match t with String_type -> "S" | Integer_type -> "I" | Boolean_type -> "B")
    | Origin_provenance oid -> "O:" ^ lookup_origin_c colours oid
    | Role_proxy rid ->
        let scope =
          match StringMap.find_opt (string_of_fact_id f.fact_id) refs.fact_scope_map with
          | Some sc -> sc
          | None -> `Program
        in
        "RP:" ^ lookup_role_colour_in_scope colours rid scope
  in
  let guards_str =
    match StringMap.find_opt (string_of_fact_id f.fact_id) refs.guards_for_fact with
    | None -> ""
    | Some [] -> ""
    | Some guards ->
        let sorted = List.sort (fun (a_op, a_v) (b_op, b_v) ->
          let c = Int.compare (operator_rank a_op) (operator_rank b_op) in
          if c <> 0 then c else String.compare (key_of_value a_v) (key_of_value b_v)
        ) guards in
        ":G=" ^ String.concat "," (List.map (fun (op, v) ->
          string_of_int (operator_rank op) ^ ":" ^ key_of_value v) sorted)
  in
  let consumers_str =
    match StringMap.find_opt (string_of_fact_id f.fact_id) refs.consumers_for_fact_scoped with
    | None -> ""
    | Some [] -> ""
    | Some consumers ->
        let encode_scoped binding scope =
          match binding with
          | Fact_from_origin (_, oid) -> "FO:" ^ lookup_origin_c colours oid
          | Fact_through_role (_, rid) -> "FR:" ^ lookup_role_colour_in_scope colours rid scope
          | _ -> "??"
        in
        let sorted = List.sort (fun (an_a, b_a, sc_a) (an_b, b_b, sc_b) ->
          let c = String.compare (string_of_capability_input_name an_a) (string_of_capability_input_name an_b) in
          if c <> 0 then c else String.compare (encode_scoped b_a sc_a) (encode_scoped b_b sc_b)
        ) consumers in
        ":U=" ^ String.concat "," (List.map (fun (iname, binding, scope) ->
          string_of_capability_input_name iname ^ ":" ^ encode_scoped binding scope) sorted)
  in
  let establisher_str =
    match StringMap.find_opt (string_of_fact_id f.fact_id) refs.origin_for_fact with
    | Some (Some oid) -> ":E=" ^ lookup_origin_c colours oid
    | _ -> ""
  in
  "F:" ^ prov_str ^ guards_str ^ consumers_str ^ establisher_str

let origin_sig_rn colours refs (oid, site, scope, _tmpl) =
  let oid_s = string_of_origin_id oid in
  let entry_tag =
    match refs.entry_origin_id_str with
    | Some s when s = oid_s -> ":entry"
    | _ -> "" in
  let success_out_str =
    match StringMap.find_opt oid_s refs.success_out_map with
    | None -> ""
    | Some targets ->
        ":out=" ^ String.concat "," (List.map (control_target_cstr colours) targets) in
  let success_in_str =
    match StringMap.find_opt oid_s refs.success_in_map with
    | None -> ""
    | Some predecessors ->
        let colour_strs =
          List.map (fun p -> lookup_origin_c colours p) predecessors
          |> List.sort String.compare
        in
        ":in=" ^ String.concat "," colour_strs in
  let branch_tag =
    match StringMap.find_opt oid_s refs.origin_branches with
    | None -> ""
    | Some bids ->
        let colour_strs =
          List.map (fun bid -> lookup_branch_c colours bid) bids
          |> List.sort String.compare
        in
        ":br=" ^ String.concat "," colour_strs in
  let together_tag =
    match StringMap.find_opt oid_s refs.together_for_member with
    | None -> ""
    | Some tog_oid -> ":tg=" ^ lookup_origin_c colours tog_oid in
  let origin_scope = scope in
  match site with
  | Anchor_origin a ->
      let fact_cs =
        List.map (fun (f : fact) -> lookup_fact_c colours f.fact_id) a.declared_facts
        |> List.sort String.compare
      in
      "A:" ^ a.event_name ^ entry_tag ^ success_out_str ^ success_in_str
      ^ branch_tag ^ together_tag
      ^ ":facts=" ^ String.concat "," fact_cs
  | Action_origin a ->
      let fact_cs =
        List.map (fun (f : fact) -> lookup_fact_c colours f.fact_id) a.declared_facts
        |> List.sort String.compare
      in
      let input_keys =
        List.map (fun ai ->
          let name_part = string_of_capability_input_name ai.input_name in
          let binding_key = match ai.binding with
            | Literal_value v -> "L:" ^ key_of_value v
            | Anchor_value (oid', path) ->
                "AV:" ^ lookup_origin_c colours oid' ^ ":" ^ String.concat "/" path
            | Fact_from_origin (fid, oid') ->
                "FO:" ^ lookup_fact_c colours fid ^ ":" ^ lookup_origin_c colours oid'
            | Fact_through_role (fid, rid) ->
                "FT:" ^ lookup_fact_c colours fid ^ ":" ^ lookup_role_colour_in_scope colours rid origin_scope
            | Batch_item_context (Item_template_id tid) ->
                "BIC:" ^ lookup_item_template_c colours (item_template_id_of_string tid)
          in
          name_part ^ "=" ^ binding_key
        ) a.inputs
        |> List.sort String.compare
      in
      let constraint_keys =
        List.map (fun c -> key_of_constraint c) a.execution_constraints
        |> List.sort String.compare
      in
      "Ac:" ^ string_of_capability_id a.capability_id ^ ":"
      ^ string_of_capability_contract_digest a.contract_digest
      ^ entry_tag ^ success_out_str ^ success_in_str
      ^ branch_tag ^ together_tag
      ^ ":facts=" ^ String.concat "," fact_cs
      ^ ":inputs=" ^ String.concat "," input_keys
      ^ ":constraints=" ^ String.concat "," constraint_keys
  | Together_origin t ->
      let member_keys =
        List.map (fun oid' -> lookup_origin_c colours oid') t.member_origin_ids
        |> List.sort String.compare
      in
      "T:" ^ key_of_together_objective t.objective
      ^ entry_tag ^ success_out_str ^ success_in_str
      ^ branch_tag ^ together_tag
      ^ ":members=" ^ String.concat "," member_keys
  | Batch_site b ->
      "Ba:" ^ key_of_batch_collection_provenance b.collection_provenance ^ ":"
      ^ lookup_item_template_c colours b.item_template_id ^ ":"
      ^ key_of_batch_traversal_policy b.traversal_policy ^ ":"
      ^ key_of_batch_objective b.composite_objective
      ^ ":facts=" ^ String.concat ","
          (List.map (fun (f : fact) -> lookup_fact_c colours f.fact_id) b.aggregate_facts
           |> List.sort String.compare)

let role_sig_rn colours _refs (r, scope) =
  let (Role_fact_contract fids) = r.fact_contract in
  let fact_keys_sorted =
    List.map (fun fid -> lookup_fact_c colours fid) fids
    |> List.sort String.compare
  in
  let scope_str = match scope with
    | `Program -> "P"
    | `Template tid -> "T:" ^ lookup_item_template_c colours tid
  in
  "Ro:" ^ scope_str ^ ":"
  ^ key_of_role_fulfillment r.eligible_fulfillment
  ^ ":facts=" ^ String.concat "," fact_keys_sorted

let branch_sig_rn colours _refs (b, _scope) =
  let outcome_keys =
    List.map (fun (outcome, target) ->
      key_of_outcome outcome ^ ":"
      ^ (match target with
         | Continue_to oid -> "C:" ^ lookup_origin_c colours oid
         | Stop -> "S"))
      b.outcome_branches
    |> List.sort String.compare
  in
  "Br:" ^ lookup_origin_c colours b.branch_subject
  ^ ":outcomes=" ^ String.concat "," outcome_keys

let item_template_sig_rn colours _refs (t : item_template) =
  let origin_keys_list =
    List.map (fun site ->
      match origin_id_of_site site with
      | Some oid -> lookup_origin_c colours oid
      | None ->
          (match site with
           | Batch_site b -> lookup_batch_c colours b.batch_id
           | _ -> "0"))
      t.origin_sites
    |> List.sort String.compare
  in
  let branch_keys_list =
    List.map (fun b -> lookup_branch_c colours b.branch_id) t.branches
    |> List.sort String.compare
  in
  let role_keys_list =
    List.map (fun (r : role) ->
      lookup_role_c colours (scoped_role_id r (`Template t.item_template_id))
    ) t.roles
    |> List.sort String.compare
  in
  let obj_key = match t.objective with
    | Required_role rid ->
        "RR:" ^ lookup_role_c colours (scoped_role_id { role_id = rid; scope = Program_scope;
          fact_contract = Role_fact_contract []; eligible_fulfillment = role_fulfillment_of_string "" } (`Template t.item_template_id))
  in
  "IT:" ^ obj_key
  ^ ":origins=" ^ String.concat "," origin_keys_list
  ^ ":branches=" ^ String.concat "," branch_keys_list
  ^ ":roles=" ^ String.concat "," role_keys_list

(* ================================================================== *)
(*  STRATEGY A: Current production refinement (baseline)               *)
(* ================================================================== *)

let refine_round_current prev refs p =
  let fact_colours =
    List.map (fun (f : fact) ->
      (string_of_fact_id f.fact_id, fact_sig_rn prev refs f)) (all_facts p)
    |> compress_colours
  in
  let origin_colours =
    List.map (fun (oid, site, scope, tmpl) ->
      (string_of_origin_id oid, origin_sig_rn prev refs (oid, site, scope, tmpl))) (all_origins p)
    |> compress_colours
  in
  let batch_colours =
    List.map (fun (bid, site, scope, tmpl) ->
      (string_of_batch_id bid, origin_sig_rn prev refs (origin_id_of_string "", site, scope, tmpl))) (all_batches p)
    |> compress_colours
  in
  let role_colours =
    List.map (fun (r, scope) ->
      (scoped_role_id r scope, role_sig_rn prev refs (r, scope))) (all_roles p)
    |> compress_colours
  in
  let branch_colours =
    List.map (fun (b, scope) ->
      (string_of_branch_id b.branch_id, branch_sig_rn prev refs (b, scope))) (all_branches p)
    |> compress_colours
  in
  let item_template_colours =
    List.map (fun (t : item_template) ->
      (string_of_item_template_id t.item_template_id, item_template_sig_rn prev refs t)) p.item_templates
    |> compress_colours
  in
  { fact_colours; origin_colours; batch_colours; role_colours;
    branch_colours; item_template_colours }

let rec refine_until_stable_current n prev refs p =
  if n >= safety_cap then (n, prev)
  else
    let next = refine_round_current prev refs p in
    if partitions_equal prev next then (n, next)
    else refine_until_stable_current (n + 1) next refs p

(* ================================================================== *)
(*  STRATEGY B: Topological multi-pass refinement                      *)
(*                                                                      *)
(*  Process entity types in dependency order within each round:         *)
(*    1. Facts (depend on previous-round origin/role colours)           *)
(*    2. Origins (depend on fact colours + other origin colours)        *)
(*    3. Batches (depend on origin colours)                             *)
(*    4. Branches (depend on origin colours)                            *)
(*    5. Roles (depend on fact colours + template colours)              *)
(*    6. Item templates (depend on origin/branch/role colours)          *)
(*                                                                      *)
(*  Within origins, process in topological order so that each origin    *)
(*  sees the latest colours of its predecessors.                        *)
(* ================================================================== *)

(* Build a topological ordering of origins based on success_continuations.
   Entry origin first, then successors. Back-edges and cycles are handled
   by BFS distance from entry. *)
let build_origin_topo_order p =
  let all_orig = all_origins p in
  let entry_s = Option.map string_of_origin_id p.entry_origin in
  (* Build successor map: origin_id_str -> list of successor origin_id_strs *)
  let succ_map =
    List.fold_left (fun m sc ->
      let from_s = string_of_origin_id sc.from_origin in
      match sc.target with
      | Origin_target tgt ->
        let tgt_s = string_of_origin_id tgt in
        StringMap.update from_s (function
          | None -> Some [tgt_s]
          | Some lst -> Some (tgt_s :: lst)) m
      | Program_complete -> m
    ) StringMap.empty p.success_continuations
  in
  (* BFS from entry to compute distance *)
  let distance = ref StringMap.empty in
  let queue = ref [] in
  (match entry_s with
   | Some s ->
     distance := StringMap.add s 0 !distance;
     queue := [s]
   | None -> ());
  let rec bfs () =
    match !queue with
    | [] -> ()
    | current :: rest ->
      queue := rest;
      let d = StringMap.find current !distance in
      let successors = match StringMap.find_opt current succ_map with
        | Some lst -> lst | None -> []
      in
      List.iter (fun succ ->
        if not (StringMap.mem succ !distance) then begin
          distance := StringMap.add succ (d + 1) !distance;
          queue := !queue @ [succ]
        end
      ) successors;
      bfs ()
  in
  bfs ();
  (* Sort origins by distance (entry first, then by BFS order).
     Origins not reachable from entry get max_int distance. *)
  let get_distance oid_s =
    match StringMap.find_opt oid_s !distance with
    | Some d -> d
    | None -> max_int
  in
  List.sort (fun (oid_a, _, _, _) (oid_b, _, _, _) ->
    let da = get_distance (string_of_origin_id oid_a) in
    let db = get_distance (string_of_origin_id oid_b) in
    let c = Int.compare da db in
    if c <> 0 then c
    else String.compare (string_of_origin_id oid_a) (string_of_origin_id oid_b)
  ) all_orig

let refine_round_topological prev refs p =
  (* Layer 1: Facts (use previous-round colours for origin/role references) *)
  let fact_colours =
    List.map (fun (f : fact) ->
      (string_of_fact_id f.fact_id, fact_sig_rn prev refs f)) (all_facts p)
    |> compress_colours
  in
  (* Layer 2: Origins in topological order, updating colours as we go *)
  let ordered_origins = build_origin_topo_order p in
  (* We accumulate origin signatures (strings) and a running colour map.
     When we process an origin, we compute its signature using the latest
     colours, then immediately assign it a new provisional colour so that
     subsequent origins in topological order can see it. *)
  let origin_sig_acc : string StringMap.t ref = ref StringMap.empty in
  let running_origin_colours : int StringMap.t ref = ref prev.origin_colours in
  List.iter (fun (oid, site, scope, tmpl) ->
    let oid_s = string_of_origin_id oid in
    (* Build a temporary colour_map with the latest running origin colours *)
    let current_colours = { prev with origin_colours = !running_origin_colours; fact_colours } in
    let sig_str = origin_sig_rn current_colours refs (oid, site, scope, tmpl) in
    origin_sig_acc := StringMap.add oid_s sig_str !origin_sig_acc;
    (* Assign a provisional colour: use the signature string's hash as a
       temporary colour value. This isn't the final colour — we compress
       at the end — but it gives subsequent origins a way to distinguish
       this origin from others that haven't been processed yet. *)
    let provisional_colour = Hashtbl.hash sig_str in
    running_origin_colours := StringMap.add oid_s provisional_colour !running_origin_colours
  ) ordered_origins;
  (* Compress the accumulated origin signatures into final colours *)
  let origin_colours =
    StringMap.fold (fun oid_s sig_str acc ->
      (oid_s, sig_str) :: acc
    ) !origin_sig_acc []
    |> compress_colours
  in
  (* Layer 3: Batches (use updated origin colours) *)
  let colours_with_origins = { prev with fact_colours; origin_colours } in
  let batch_colours =
    List.map (fun (bid, site, scope, tmpl) ->
      (string_of_batch_id bid, origin_sig_rn colours_with_origins refs (origin_id_of_string "", site, scope, tmpl))) (all_batches p)
    |> compress_colours
  in
  (* Layer 4: Branches (use updated origin colours) *)
  let branch_colours =
    List.map (fun (b, scope) ->
      (string_of_branch_id b.branch_id, branch_sig_rn colours_with_origins refs (b, scope))) (all_branches p)
    |> compress_colours
  in
  (* Layer 5: Roles (use updated fact colours) *)
  let role_colours =
    List.map (fun (r, scope) ->
      (scoped_role_id r scope, role_sig_rn colours_with_origins refs (r, scope))) (all_roles p)
    |> compress_colours
  in
  (* Layer 6: Item templates (use updated origin/branch/role colours) *)
  let colours_for_templates = { colours_with_origins with batch_colours; branch_colours; role_colours } in
  let item_template_colours =
    List.map (fun (t : item_template) ->
      (string_of_item_template_id t.item_template_id, item_template_sig_rn colours_for_templates refs t)) p.item_templates
    |> compress_colours
  in
  { fact_colours; origin_colours; batch_colours; role_colours;
    branch_colours; item_template_colours }

let rec refine_until_stable_topological n prev refs p =
  if n >= safety_cap then (n, prev)
  else
    let next = refine_round_topological prev refs p in
    if partitions_equal prev next then (n, next)
    else refine_until_stable_topological (n + 1) next refs p

(* ================================================================== *)
(*  Benchmark runner                                                    *)
(* ================================================================== *)

let now_seconds () = Unix.gettimeofday ()

let sort_floats a = Array.sort compare a; a
let median sorted =
  let n = Array.length sorted in
  if n = 0 then 0.0
  else if n mod 2 = 1 then sorted.(n / 2)
  else (sorted.(n / 2 - 1) +. sorted.(n / 2)) /. 2.0

(* Build a benchmark program with n identical sequential actions *)
let make_ping_request ~num_actions =
  let tether_source =
    let buf = Buffer.create 1024 in
    Buffer.add_string buf
      "tether \"benchmark ping\"\n\nanchor\n    fixture.start\n\nwhen\ndo\n";
    for _ = 1 to num_actions do
      Buffer.add_string buf "    fixture.ping\n        message: anchor.message\n"
    done;
    Buffer.contents buf
  in
  let actions_json =
    `List [
      `Assoc [
        ("name", `String "fixture.ping");
        ("version", `String "1.0.0");
        ("inputs", `Assoc [ ("message", `String "string") ]);
        ("effects", `List [ `String "fixture.test" ]);
        ("reversibility", `String "compensatable");
      ];
    ]
  in
  `Assoc [
    ("protocol_version", `String "0.1");
    ("language_version", `String "0.1");
    ("evaluation_id", `String "bench_cb1");
    ("tether", `Assoc [
      ("id", `String "benchmark-ping");
      ("version", `String "1");
      ("source", `String tether_source);
    ]);
    ("event", `Assoc [
      ("id", `String "evt_cb1");
      ("name", `String "fixture.start");
      ("data", `Assoc [ ("message", `String "hello") ]);
    ]);
    ("facts", `Assoc []);
    ("capabilities", actions_json);
    ("core_environment", `Assoc [
      ("program_id", `String "program.benchmark");
      ("core_version", `String "1");
      ("capabilities", `List [
        `Assoc [
          ("source_name", `String "fixture.ping");
          ("capability_id", `String "cap.benchmark.ping");
          ("contract_digest", `String "BENCH-CONTRACT-0");
          ("runtime_name", `String "fixture.ping");
        ];
      ]);
      ("input_facts", `List []);
    ]);
  ]

(* Build a low-symmetry request: each action has a distinct literal *)
let make_distinct_ping_request ~num_actions =
  let tether_source =
    let buf = Buffer.create 4096 in
    Buffer.add_string buf
      "tether \"benchmark ping distinct\"\n\nanchor\n    fixture.start\n\nwhen\ndo\n";
    for i = 1 to num_actions do
      Buffer.add_string buf
        ("    fixture.ping\n        message: \"msg_" ^ string_of_int i
       ^ "\"\n        path: \"projects/bench.txt\"\n")
    done;
    Buffer.contents buf
  in
  let actions_json =
    `List [
      `Assoc [
        ("name", `String "fixture.ping");
        ("version", `String "1.0.0");
        ("inputs", `Assoc [ ("message", `String "string"); ("path", `String "string") ]);
        ("effects", `List [ `String "fixture.test" ]);
        ("reversibility", `String "compensatable");
      ];
    ]
  in
  `Assoc [
    ("protocol_version", `String "0.1");
    ("language_version", `String "0.1");
    ("evaluation_id", `String "bench_cb1_distinct");
    ("tether", `Assoc [
      ("id", `String "benchmark-ping-distinct");
      ("version", `String "1");
      ("source", `String tether_source);
    ]);
    ("event", `Assoc [
      ("id", `String "evt_cb1_dist");
      ("name", `String "fixture.start");
      ("data", `Assoc [ ("message", `String "hello") ]);
    ]);
    ("facts", `Assoc []);
    ("capabilities", actions_json);
    ("core_environment", `Assoc [
      ("program_id", `String "program.benchmark");
      ("core_version", `String "1");
      ("capabilities", `List [
        `Assoc [
          ("source_name", `String "fixture.ping");
          ("capability_id", `String "cap.benchmark.ping");
          ("contract_digest", `String "BENCH-CONTRACT-0");
          ("runtime_name", `String "fixture.ping");
        ];
      ]);
      ("input_facts", `List []);
    ]);
  ]

(* Decode request to program - re-use the benchmark's decode logic *)
let object_fields = function
  | `Assoc fields -> fields
  | _ -> failwith "decode: expected object"

let field_string fields name =
  match List.assoc_opt name fields with
  | Some (`String s) -> s
  | _ -> failwith ("decode: missing string " ^ name)

let field_object fields name =
  match List.assoc_opt name fields with
  | Some (`Assoc _ as v) -> v
  | _ -> failwith ("decode: missing object " ^ name)

let field_list fields name =
  match List.assoc_opt name fields with
  | Some (`List l) -> l
  | _ -> failwith ("decode: missing list " ^ name)

let decode_request request_json =
  let obj = object_fields request_json in
  let source = field_string (object_fields (field_object obj "tether")) "source" in
  let core_fields = object_fields (field_object obj "core_environment") in
  let program_id = Tethers_core.program_id_of_string (field_string core_fields "program_id") in
  let core_version = Tethers_core.core_version_of_string (field_string core_fields "core_version") in
  let cap_binding_jsons = field_list core_fields "capabilities" in
  let top_level_caps =
    List.map (fun cap_json ->
      Tethers_protocol.parse_capability cap_json
    ) (field_list obj "capabilities")
  in
  let capabilities =
    List.map (fun binding ->
      let b = object_fields binding in
      let source_name = field_string b "source_name" in
      let capability_id = Tethers_core.capability_id_of_string (field_string b "capability_id") in
      let contract_digest = Tethers_core.capability_contract_digest_of_string (field_string b "contract_digest") in
      let runtime_name = field_string b "runtime_name" in
      let runtime = List.find (fun (c : Tethers_protocol.capability) -> c.name = runtime_name) top_level_caps in
      ({ source_name; capability_id; contract_digest; runtime } : Tethers_core_evaluation_adapter.capability_binding)
    ) cap_binding_jsons
  in
  let lowerer_env : Tethers_core_lowerer.lowering_environment =
    { program_id; core_version;
      capabilities = List.map (fun (b : Tethers_core_evaluation_adapter.capability_binding) ->
        { Tethers_core_lowerer.source_name = b.source_name;
          capability_id = b.capability_id;
          contract_digest = b.contract_digest }) capabilities;
      input_facts = [] }
  in
  let tether = Tether_parser.parse_tether source in
  Result.get_ok (Tethers_core_lowerer.lower lowerer_env tether)

(* ================================================================== *)
(*  STRATEGY C: Topological with exact colour preservation              *)
(*                                                                      *)
(*  Same as Strategy A (current) but processes origins in topological   *)
(*  order. Since all signatures use `prev` colours (not running         *)
(*  accumulator), the final colour values are identical to Strategy A.  *)
(*  The topological ordering provides no convergence benefit here —     *)
(*  this is a verification that processing order doesn't affect results.*)
(* ================================================================== *)

let refine_round_topo_exact prev refs p =
  let fact_colours =
    List.map (fun (f : fact) ->
      (string_of_fact_id f.fact_id, fact_sig_rn prev refs f)) (all_facts p)
    |> compress_colours
  in
  (* Process origins in topological order but use prev colours for all signatures *)
  let ordered_origins = build_origin_topo_order p in
  let origin_colours =
    List.map (fun (oid, site, scope, tmpl) ->
      (string_of_origin_id oid, origin_sig_rn prev refs (oid, site, scope, tmpl))) ordered_origins
    |> compress_colours
  in
  let batch_colours =
    List.map (fun (bid, site, scope, tmpl) ->
      (string_of_batch_id bid, origin_sig_rn prev refs (origin_id_of_string "", site, scope, tmpl))) (all_batches p)
    |> compress_colours
  in
  let role_colours =
    List.map (fun (r, scope) ->
      (scoped_role_id r scope, role_sig_rn prev refs (r, scope))) (all_roles p)
    |> compress_colours
  in
  let branch_colours =
    List.map (fun (b, scope) ->
      (string_of_branch_id b.branch_id, branch_sig_rn prev refs (b, scope))) (all_branches p)
    |> compress_colours
  in
  let item_template_colours =
    List.map (fun (t : item_template) ->
      (string_of_item_template_id t.item_template_id, item_template_sig_rn prev refs t)) p.item_templates
    |> compress_colours
  in
  { fact_colours; origin_colours; batch_colours; role_colours;
    branch_colours; item_template_colours }

let rec refine_until_stable_topo_exact n prev refs p =
  if n >= safety_cap then (n, prev)
  else
    let next = refine_round_topo_exact prev refs p in
    if partitions_equal prev next then (n, next)
    else refine_until_stable_topo_exact (n + 1) next refs p

(* ================================================================== *)
(*  Colour ordering comparison                                          *)
(* ================================================================== *)

(* Check if the relative ordering of colour classes is preserved.
   For each entity type, sort entities by (colour, id) and compare
   the resulting orderings. *)
let colour_ordering_preserved cur topo p =
  let compare_by_colour cmap =
    List.sort (fun (id_a, _) (id_b, _) ->
      let c_a = StringMap.find id_a cmap in
      let c_b = StringMap.find id_b cmap in
      let c = Int.compare c_a c_b in
      if c <> 0 then c else String.compare id_a id_b
    )
  in
  let origins = List.map (fun (oid, _, _, _) -> string_of_origin_id oid) (all_origins p) in
  let facts = List.map (fun (f : fact) -> string_of_fact_id f.fact_id) (all_facts p) in
  let origin_order_cur =
    List.map (fun id -> (id, StringMap.find id cur.origin_colours)) origins
    |> compare_by_colour cur.origin_colours
    |> List.map fst
  in
  let origin_order_topo =
    List.map (fun id -> (id, StringMap.find id topo.origin_colours)) origins
    |> compare_by_colour topo.origin_colours
    |> List.map fst
  in
  let fact_order_cur =
    List.map (fun id -> (id, StringMap.find id cur.fact_colours)) facts
    |> compare_by_colour cur.fact_colours
    |> List.map fst
  in
  let fact_order_topo =
    List.map (fun id -> (id, StringMap.find id topo.fact_colours)) facts
    |> compare_by_colour topo.fact_colours
    |> List.map fst
  in
  (origin_order_cur = origin_order_topo, fact_order_cur = fact_order_topo)

(* ================================================================== *)
(*  Full canonicalization comparison                                    *)
(* ================================================================== *)

(* Run the full canonicalization pipeline: colours → IDs → program → bytes → digest *)
let full_canonicalize colours p =
  let ids = Tethers_core_canonical.assign_canonical_ids colours p in
  let canon = Tethers_core_canonical.build_canonical_program p ids in
  let bytes = Tethers_core_canonical.make_canonical_bytes canon in
  let hex = Tethers_core_canonical.compute_sha256 bytes in
  let digest = Tethers_core_canonical.make_program_digest hex in
  (canon, bytes, digest)

(* ================================================================== *)
(*  Main benchmark                                                       *)
(* ================================================================== *)

let () =
  Printf.printf "C-B1: Refinement Strategy Benchmark\n%!";
  Printf.printf "====================================\n\n%!";
  
  let sizes = [100; 250; 500; 1000] in
  
  Printf.printf "%-6s  %-8s  %-8s  %-8s  %-8s  %-8s  %-8s  %-10s  %-10s  %-10s\n%!"
    "Size" "CurRnds" "TopoR" "TopoER" "CurMed" "TopoMed" "TopoEMd" "PartEq" "OrdEq" "CanonEq";
  Printf.printf "%-6s  %-8s  %-8s  %-8s  %-8s  %-8s  %-8s  %-10s  %-10s  %-10s\n%!"
    "----" "-------" "------" "-------" "------" "------" "-------" "------" "------" "--------";
  
  List.iter (fun size ->
    (* High-symmetry case *)
    let request = make_ping_request ~num_actions:size in
    let program = decode_request request in
    let refs = build_static_refs program in
    let r0 = round_0 program in
    
    (* Strategy A: Current *)
    let (rounds_current, colours_current) =
      refine_until_stable_current 1 r0 refs program
    in
    
    (* Strategy B: Topological with provisional colours *)
    let (rounds_topo, colours_topo) =
      refine_until_stable_topological 1 r0 refs program
    in
    
    (* Strategy C: Topological with exact colours *)
    let (rounds_topo_exact, colours_topo_exact) =
      refine_until_stable_topo_exact 1 r0 refs program
    in
    
    (* Verify partition equivalence *)
    let part_eq = partitions_equal colours_current colours_topo in
    
    (* Verify colour ordering preservation *)
    let (origin_ord_eq, fact_ord_eq) = colour_ordering_preserved colours_current colours_topo program in
    let ord_eq = origin_ord_eq && fact_ord_eq in
    
    (* Verify canonical bytes + digest equality *)
    let (_canon_cur, bytes_cur, digest_cur) = full_canonicalize colours_current program in
    let (_canon_topo, bytes_topo, digest_topo) = full_canonicalize colours_topo program in
    let canon_eq = (bytes_cur = bytes_topo) in
    let digest_eq = (Tethers_core_canonical.string_of_program_digest digest_cur
                   = Tethers_core_canonical.string_of_program_digest digest_topo) in
    
    (* Verify Strategy C produces exact same canonical output *)
    let (_canon_topo_e, bytes_topo_e, digest_topo_e) = full_canonicalize colours_topo_exact program in
    let _canon_c_eq = (bytes_cur = bytes_topo_e) in
    let _digest_c_eq = (Tethers_core_canonical.string_of_program_digest digest_cur
                     = Tethers_core_canonical.string_of_program_digest digest_topo_e) in
    
    (* Diagnostic: show first few origin colours if ordering differs *)
    if not ord_eq && size <= 100 then begin
      Printf.printf "  DIAGNOSTIC (size %d) — origin colour ordering differs:\n%!" size;
      let all_orig = all_origins program in
      List.iteri (fun i (oid, _, _, _) ->
        if i < 8 then begin
          let oid_s = string_of_origin_id oid in
          let c_cur = StringMap.find oid_s colours_current.origin_colours in
          let c_topo = StringMap.find oid_s colours_topo.origin_colours in
          Printf.printf "    %-12s  cur=%-4d  topo=%-4d%s\n%!"
            oid_s c_cur c_topo (if c_cur <> c_topo then "  DIFF" else "")
        end
      ) all_orig;
      Printf.printf "  Canonical bytes equal: %s, Digest equal: %s\n%!"
        (if canon_eq then "YES" else "NO") (if digest_eq then "YES" else "NO")
    end;
    
    (* Timing *)
    let num_batches = if size <= 250 then 8 else 4 in
    let batch_size = if size <= 250 then 5 else 3 in
    
    let time_strategy refine_fn =
      let _ = refine_fn 1 r0 refs program in  (* warmup *)
      let samples = Array.make num_batches 0.0 in
      for b = 0 to num_batches - 1 do
        let t0 = now_seconds () in
        for _ = 1 to batch_size do
          let _ = refine_fn 1 r0 refs program in
          ()
        done;
        let t1 = now_seconds () in
        samples.(b) <- (t1 -. t0) *. 1_000_000.0 /. float_of_int batch_size
      done;
      median (sort_floats (Array.copy samples))
    in
    
    let time_current = time_strategy refine_until_stable_current in
    let time_topo = time_strategy refine_until_stable_topological in
    let time_topo_exact = time_strategy refine_until_stable_topo_exact in
    
    Printf.printf "%-6d  %-8d  %-8d  %-8d  %-8.0f  %-8.0f  %-8.0f  %-10s  %-10s  %-10s\n%!"
      size rounds_current rounds_topo rounds_topo_exact time_current time_topo time_topo_exact
      (if part_eq then "PASS" else "FAIL")
      (if ord_eq then "PASS" else "FAIL")
      (if canon_eq && digest_eq then "PASS" else "FAIL")
  ) sizes;
  
  Printf.printf "\n%!";
  
  (* Low-symmetry case at 250 *)
  Printf.printf "Low-symmetry (250 distinct actions):\n%!";
  let request_low = make_distinct_ping_request ~num_actions:250 in
  let program_low = decode_request request_low in
  let refs_low = build_static_refs program_low in
  let r0_low = round_0 program_low in
  let (rounds_cur_low, colours_cur_low) = refine_until_stable_current 1 r0_low refs_low program_low in
  let (rounds_topo_low, colours_topo_low) = refine_until_stable_topological 1 r0_low refs_low program_low in
  let part_eq_low = partitions_equal colours_cur_low colours_topo_low in
  let (origin_ord_eq_low, fact_ord_eq_low) = colour_ordering_preserved colours_cur_low colours_topo_low program_low in
  let (_canon_cur_low, bytes_cur_low, digest_cur_low) = full_canonicalize colours_cur_low program_low in
  let (_canon_topo_low, bytes_topo_low, digest_topo_low) = full_canonicalize colours_topo_low program_low in
  let canon_eq_low = (bytes_cur_low = bytes_topo_low) in
  let digest_eq_low = (Tethers_core_canonical.string_of_program_digest digest_cur_low
                     = Tethers_core_canonical.string_of_program_digest digest_topo_low) in
  Printf.printf "  Current: %d rounds, Topological: %d rounds\n%!" rounds_cur_low rounds_topo_low;
  Printf.printf "  Partition: %s, Ordering: %s, Canon: %s, Digest: %s\n%!"
    (if part_eq_low then "PASS" else "FAIL")
    (if origin_ord_eq_low && fact_ord_eq_low then "PASS" else "FAIL")
    (if canon_eq_low then "PASS" else "FAIL")
    (if digest_eq_low then "PASS" else "FAIL");
  
  Printf.printf "\n%!";
  
  (* Small size diagnostic: detailed colour comparison *)
  Printf.printf "Detailed colour comparison (size 10):\n%!";
  let request_10 = make_ping_request ~num_actions:10 in
  let program_10 = decode_request request_10 in
  let refs_10 = build_static_refs program_10 in
  let r0_10 = round_0 program_10 in
  let (_, colours_cur_10) = refine_until_stable_current 1 r0_10 refs_10 program_10 in
  let (_, colours_topo_10) = refine_until_stable_topological 1 r0_10 refs_10 program_10 in
  let all_orig_10 = all_origins program_10 in
  Printf.printf "  %-12s  %-6s  %-6s  %-6s\n%!" "Origin" "Cur" "Topo" "Match";
  Printf.printf "  %-12s  %-6s  %-6s  %-6s\n%!" "------" "---" "----" "-----";
  List.iter (fun (oid, _, _, _) ->
    let oid_s = string_of_origin_id oid in
    let c_cur = StringMap.find oid_s colours_cur_10.origin_colours in
    let c_topo = StringMap.find oid_s colours_topo_10.origin_colours in
    Printf.printf "  %-12s  %-6d  %-6d  %-6s\n%!"
      oid_s c_cur c_topo (if c_cur = c_topo then "YES" else "NO")
  ) all_orig_10;
  
  let (_, bytes_cur_10, digest_cur_10) = full_canonicalize colours_cur_10 program_10 in
  let (_, bytes_topo_10, digest_topo_10) = full_canonicalize colours_topo_10 program_10 in
  Printf.printf "\n  Canonical bytes equal: %s\n%!" (if bytes_cur_10 = bytes_topo_10 then "YES" else "NO");
  Printf.printf "  Digest cur:  %s\n%!" (Tethers_core_canonical.string_of_program_digest digest_cur_10);
  Printf.printf "  Digest topo: %s\n%!" (Tethers_core_canonical.string_of_program_digest digest_topo_10);
  Printf.printf "  Digest equal: %s\n%!"
    (if Tethers_core_canonical.string_of_program_digest digest_cur_10
       = Tethers_core_canonical.string_of_program_digest digest_topo_10 then "YES" else "NO");
  
  Printf.printf "\nC-B1 benchmark complete.\n%!"
