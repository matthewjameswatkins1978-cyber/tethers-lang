open Tethers_core

type canonicalization_error =
  | Invalid_core of Tethers_core_validator.validation_error list
  | Refinement_exceeded

type program_digest = Program_digest of string

type canonicalized = {
  c_program : program;
  c_bytes : string;
  c_digest : program_digest;
}

let canonical_prefix = "TETHERS_CORE_CANON_V1"
let canonical_prefix_byte = '\x00'
let safety_cap = 1000

(* ------------------------------------------------------------------ *)
(*  Entity collection                                                   *)
(* ------------------------------------------------------------------ *)

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

(* ------------------------------------------------------------------ *)
(*  Scoped identity maps                                                *)
(* ------------------------------------------------------------------ *)

module StringMap = Map.Make(String)

let scoped_role_id r scope =
  match scope with
  | `Program -> "P:" ^ string_of_role_id r.role_id
  | `Template tid -> "T:" ^ string_of_item_template_id tid ^ ":" ^ string_of_role_id r.role_id

(* ------------------------------------------------------------------ *)
(*  Key helpers                                                         *)
(* ------------------------------------------------------------------ *)

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

(* ------------------------------------------------------------------ *)
(*  Colour map: collision-free partition refinement                     *)
(* ------------------------------------------------------------------ *)

type colour_map = {
  fact_colours : int StringMap.t;
  origin_colours : int StringMap.t;
  batch_colours : int StringMap.t;
  role_colours : int StringMap.t;
  branch_colours : int StringMap.t;
  item_template_colours : int StringMap.t;
}

(* Compress: given (entity_id_string, full_signature) pairs,
   assign consecutive colours 1..N by sorted unique signatures *)
let compress_colours pairs =
  let uniques =
    pairs |> List.map snd |> List.sort_uniq String.compare
  in
  let sig_to_colour =
    List.mapi (fun i sig_str -> (sig_str, i + 1)) uniques
  in
  List.fold_left (fun m (id, sig_str) ->
    StringMap.add id (List.assoc sig_str sig_to_colour) m
  ) StringMap.empty pairs

let int_map_partition_stable prev next =
  let count_unique m =
    StringMap.fold (fun _ v acc -> if List.mem v acc then acc else v :: acc) m [] |> List.length
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

(* ------------------------------------------------------------------ *)
(*  Colour lookups                                                      *)
(* ------------------------------------------------------------------ *)

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

(* Scope-aware role colour lookup *)
let lookup_role_colour_in_scope colours rid = function
  | `Program -> lookup_role_c colours ("P:" ^ string_of_role_id rid)
  | `Template tid -> lookup_role_c colours ("T:" ^ string_of_item_template_id tid ^ ":" ^ string_of_role_id rid)

(* ------------------------------------------------------------------ *)
(*  Static reference maps (built once, used every refinement round)     *)
(* ------------------------------------------------------------------ *)

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

  (* Scoped consumers: track the scope of the consuming Action origin *)
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

  (* Direct derivation of Fact scope from containing site/template, including Batch *)
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

(* ------------------------------------------------------------------ *)
(*  Round 0: scalar-only signatures                                     *)
(* ------------------------------------------------------------------ *)

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

(* ------------------------------------------------------------------ *)
(*  Round N: signatures using previous-round colours                     *)
(* ------------------------------------------------------------------ *)

(* Control-target colour string for origin signatures *)
let control_target_cstr colours = function
  | Origin_target oid -> "C:" ^ lookup_origin_c colours oid
  | Program_complete -> "PC"

(* Fact signature round N *)
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

(* Origin signature round N *)
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

(* Role signature round N *)
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

(* Branch signature round N *)
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

(* Item Template signature round N *)
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

(* ------------------------------------------------------------------ *)
(*  Single refinement round                                             *)
(* ------------------------------------------------------------------ *)

let refine_round prev refs p =
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

(* ------------------------------------------------------------------ *)
(*  Refinement to stability                                             *)
(* ------------------------------------------------------------------ *)

let rec refine_until_stable n prev refs p =
  if n >= safety_cap then raise Exit
  else
    let next = refine_round prev refs p in
    if partitions_equal prev next then next
    else refine_until_stable (n + 1) next refs p

let final_colours p =
  let refs = build_static_refs p in
  let r0 = round_0 p in
  try refine_until_stable 1 r0 refs p
  with Exit -> failwith "refinement exceeded safety cap"

(* ------------------------------------------------------------------ *)
(*  Canonical ID assignment                                             *)
(* ------------------------------------------------------------------ *)

type canonical_ids = {
  origin_order : (origin_id * origin_id) list;
  fact_order : (fact_id * fact_id) list;
  role_order : (string * role_id) list;
  origin_scope_map : [`Program | `Template of item_template_id] StringMap.t;
  fact_scope_map : [`Program | `Template of item_template_id] StringMap.t;
  branch_order : (branch_id * branch_id) list;
  group_order : (group_id * group_id) list;
  batch_order : (batch_id * batch_id) list;
  item_template_order : (item_template_id * item_template_id) list;
}

let assign_canonical_ids colours p =
  let all_orig = all_origins p in
  let all_fact = all_facts p in
  let all_rl = all_roles p in
  let all_br = all_branches p in
  let all_it = p.item_templates in
  let all_bat = all_batches p in

  let sorted_origins =
    List.map (fun (oid, _, _, _) ->
      let c = StringMap.find (string_of_origin_id oid) colours.origin_colours in
      (string_of_origin_id oid, c)) all_orig
    |> List.sort (fun (_, a) (_, b) -> Int.compare a b)
  in
  let origin_order =
    List.mapi (fun i (oid_s, _) ->
      (origin_id_of_string oid_s,
       origin_id_of_string ("O" ^ string_of_int (i + 1)))) sorted_origins
  in

  let sorted_facts =
    List.map (fun (f : fact) ->
      let c = StringMap.find (string_of_fact_id f.fact_id) colours.fact_colours in
      (string_of_fact_id f.fact_id, c)) all_fact
    |> List.sort (fun (_, a) (_, b) -> Int.compare a b)
  in
  let fact_order =
    List.mapi (fun i (fid_s, _) ->
      (fact_id_of_string fid_s,
       fact_id_of_string ("F" ^ string_of_int (i + 1)))) sorted_facts
  in

  (* Roles: sort by colour; do NOT sort_uniq — different scopes with same colour are distinct *)
  let sorted_roles =
    List.map (fun (r, scope) ->
      let c = StringMap.find (scoped_role_id r scope) colours.role_colours in
      (scoped_role_id r scope, c)) all_rl
    |> List.sort (fun (_, a) (_, b) -> Int.compare a b)
  in
  let role_order =
    List.mapi (fun i (scoped_key, _) ->
      (scoped_key,
       role_id_of_string ("R" ^ string_of_int (i + 1)))) sorted_roles
  in

  (* Maps for scoped rewriting: origin -> scope, fact -> scope (direct) *)
  let origin_scope_map =
    List.fold_left (fun m (oid, _, scope, _) ->
      StringMap.add (string_of_origin_id oid) scope m
    ) StringMap.empty all_orig
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
    ) all_orig;
    List.iter (fun (_bid, site, scope, _) ->
      match site with
      | Batch_site b ->
        List.iter (fun (f : fact) ->
          m := StringMap.add (string_of_fact_id f.fact_id) scope !m
        ) b.aggregate_facts
      | _ -> ()
    ) all_bat;
    List.iter (fun (f : fact) ->
      if not (StringMap.mem (string_of_fact_id f.fact_id) !m) then
        m := StringMap.add (string_of_fact_id f.fact_id) `Program !m
    ) all_fact;
    !m
  in

  let sorted_branches =
    List.map (fun (b, _) ->
      let c = StringMap.find (string_of_branch_id b.branch_id) colours.branch_colours in
      (string_of_branch_id b.branch_id, c)) all_br
    |> List.sort (fun (_, a) (_, b) -> Int.compare a b)
  in
  let branch_order =
    List.mapi (fun i (bid_s, _) ->
      (branch_id_of_string bid_s,
       branch_id_of_string ("B" ^ string_of_int (i + 1)))) sorted_branches
  in

  let sorted_item_templates =
    List.map (fun (t : item_template) ->
      let c = StringMap.find (string_of_item_template_id t.item_template_id) colours.item_template_colours in
      (string_of_item_template_id t.item_template_id, c)) all_it
    |> List.sort (fun (_, a) (_, b) -> Int.compare a b)
  in
  let item_template_order =
    List.mapi (fun i (tid_s, _) ->
      (item_template_id_of_string tid_s,
       item_template_id_of_string ("IT" ^ string_of_int (i + 1)))) sorted_item_templates
  in

  (* Batches *)
  let sorted_batches =
    List.map (fun (bid, _, _, _) ->
      let c = StringMap.find (string_of_batch_id bid) colours.batch_colours in
      (string_of_batch_id bid, c)) all_bat
    |> List.sort (fun (_, a) (_, b) -> Int.compare a b)
  in
  let batch_order =
    List.mapi (fun i (bid_s, _) ->
      (batch_id_of_string bid_s,
       batch_id_of_string ("BA" ^ string_of_int (i + 1)))) sorted_batches
  in

  (* Groups *)
  let origin_pos =
    List.mapi (fun i (oid_s, _) -> (oid_s, i)) sorted_origins
  in
  let together_origins =
    List.filter_map (fun (_, site, _, _) ->
      match site with
      | Together_origin t -> Some (t.group_id, string_of_origin_id t.together_origin_id)
      | _ -> None
    ) all_orig
  in
  let get_pos oid_s =
    match List.assoc_opt oid_s origin_pos with Some p -> p | None -> (-1)
  in
  let sorted_tog =
    List.sort (fun (_, a) (_, b) ->
      Int.compare (get_pos a) (get_pos b)) together_origins
  in
  let group_order =
    List.mapi (fun i (gid, _) ->
      (gid, group_id_of_string ("G" ^ string_of_int (i + 1)))) sorted_tog
  in

  { origin_order; fact_order; role_order;
    origin_scope_map; fact_scope_map; branch_order;
    group_order; batch_order; item_template_order }

(* ------------------------------------------------------------------ *)
(*  Reference rewriting                                                  *)
(* ------------------------------------------------------------------ *)

let canonical_origin ids oid =
  match List.assoc_opt oid ids.origin_order with Some c -> c | None -> oid

let canonical_fact ids fid =
  match List.assoc_opt fid ids.fact_order with Some c -> c | None -> fid

let canonical_role_in_scope ids rid scope =
  let scoped_key = match scope with
    | Program_scope -> "P:" ^ string_of_role_id rid
    | Item_template_scope tid -> "T:" ^ string_of_item_template_id tid ^ ":" ^ string_of_role_id rid
  in
  match List.assoc_opt scoped_key ids.role_order with Some c -> c | None -> rid

let canonical_role_for_fact ids fid rid =
  let scope =
    match StringMap.find_opt (string_of_fact_id fid) ids.fact_scope_map with
    | Some (`Program) -> Program_scope
    | Some (`Template tid) -> Item_template_scope tid
    | None -> Program_scope
  in
  canonical_role_in_scope ids rid scope

let canonical_role_for_origin ids rid origin_scope =
  let scope = match origin_scope with
    | `Program -> Program_scope
    | `Template tid -> Item_template_scope tid
  in
  canonical_role_in_scope ids rid scope

let canonical_branch ids bid =
  match List.assoc_opt bid ids.branch_order with Some c -> c | None -> bid

let canonical_group ids gid =
  match List.assoc_opt gid ids.group_order with Some c -> c | None -> gid

let canonical_batch ids bid =
  match List.assoc_opt bid ids.batch_order with Some c -> c | None -> bid

let canonical_item_template ids tid =
  match List.assoc_opt tid ids.item_template_order with Some c -> c | None -> tid

let rewrite_fact ids f =
  let provenance =
    match f.provenance with
    | Evaluation_input _ -> f.provenance
    | Origin_provenance oid -> Origin_provenance (canonical_origin ids oid)
    | Role_proxy rid -> Role_proxy (canonical_role_for_fact ids f.fact_id rid)
  in
  { fact_id = canonical_fact ids f.fact_id;
    schema_description = f.schema_description;
    provenance }

let rewrite_fact_guard ids g =
  { fact_id = canonical_fact ids g.fact_id;
    operator = g.operator;
    expected = g.expected }

let rewrite_input_binding ids origin_scope = function
  | Literal_value v -> Literal_value v
  | Fact_from_origin (fid, oid) ->
      Fact_from_origin (canonical_fact ids fid, canonical_origin ids oid)
  | Fact_through_role (fid, rid) ->
      let canon_rid =
        match origin_scope with
        | None -> canonical_role_for_fact ids fid rid
        | Some sc -> canonical_role_for_origin ids rid sc
      in
      Fact_through_role (canonical_fact ids fid, canon_rid)
  | Anchor_value (oid, path) -> Anchor_value (canonical_origin ids oid, path)
  | Batch_item_context tid -> Batch_item_context (canonical_item_template ids tid)

let rewrite_action_input ids origin_scope ai =
  { input_name = ai.input_name;
    binding = rewrite_input_binding ids origin_scope ai.binding }

let rewrite_origin_site ids = function
  | Anchor_origin a ->
      Anchor_origin { anchor_origin_id = canonical_origin ids a.anchor_origin_id;
                       event_name = a.event_name;
                       declared_facts = List.map (rewrite_fact ids) a.declared_facts }
  | Action_origin a ->
      let origin_scope =
        StringMap.find_opt (string_of_origin_id a.action_origin_id) ids.origin_scope_map
      in
      Action_origin { action_origin_id = canonical_origin ids a.action_origin_id;
                       capability_id = a.capability_id;
                       contract_digest = a.contract_digest;
                       inputs = List.map (rewrite_action_input ids origin_scope) a.inputs;
                       declared_facts = List.map (rewrite_fact ids) a.declared_facts;
                       execution_constraints = a.execution_constraints }
  | Together_origin t ->
      Together_origin { together_origin_id = canonical_origin ids t.together_origin_id;
                         group_id = canonical_group ids t.group_id;
                         member_origin_ids = List.map (canonical_origin ids) t.member_origin_ids;
                         objective = t.objective }
  | Batch_site b ->
      Batch_site { batch_id = canonical_batch ids b.batch_id;
                    collection_provenance = b.collection_provenance;
                    item_template_id = canonical_item_template ids b.item_template_id;
                    traversal_policy = b.traversal_policy;
                    composite_objective = b.composite_objective;
                    aggregate_facts = List.map (rewrite_fact ids) b.aggregate_facts }

let rewrite_control_target ids = function
  | Origin_target oid -> Origin_target (canonical_origin ids oid)
  | Program_complete -> Program_complete

let rewrite_success_continuation ids sc =
  { from_origin = canonical_origin ids sc.from_origin;
    target = rewrite_control_target ids sc.target }

let rewrite_branch_target ids = function
  | Continue_to oid -> Continue_to (canonical_origin ids oid)
  | Stop -> Stop

let rewrite_branch ids b =
  { branch_id = canonical_branch ids b.branch_id;
    branch_subject = canonical_origin ids b.branch_subject;
    outcome_branches = List.map (fun (outcome, target) ->
      (outcome, rewrite_branch_target ids target)) b.outcome_branches }

let rewrite_role ids r =
  let (Role_fact_contract fids) = r.fact_contract in
  let scope = match r.scope with
    | Program_scope -> Program_scope
    | Item_template_scope tid -> Item_template_scope (canonical_item_template ids tid)
  in
  { role_id = canonical_role_in_scope ids r.role_id r.scope;
    scope;
    fact_contract = Role_fact_contract (List.map (canonical_fact ids) fids);
    eligible_fulfillment = r.eligible_fulfillment }

let rewrite_item_template ids t =
  let scope = Item_template_scope t.item_template_id in
  { item_template_id = canonical_item_template ids t.item_template_id;
    origin_sites = List.map (rewrite_origin_site ids) t.origin_sites;
    branches = List.map (rewrite_branch ids) t.branches;
    roles = List.map (rewrite_role ids) t.roles;
    objective = (match t.objective with
      | Required_role rid -> Required_role (canonical_role_in_scope ids rid scope)) }

(* ------------------------------------------------------------------ *)
(*  Collection sorting                                                   *)
(* ------------------------------------------------------------------ *)

let compare_origin_id a b =
  String.compare (string_of_origin_id a) (string_of_origin_id b)
let compare_fact_id a b =
  String.compare (string_of_fact_id a) (string_of_fact_id b)
let compare_role_id a b =
  String.compare (string_of_role_id a) (string_of_role_id b)
let compare_branch_id a b =
  String.compare (string_of_branch_id a) (string_of_branch_id b)
let compare_item_template_id a b =
  String.compare (string_of_item_template_id a) (string_of_item_template_id b)
let compare_capability_id a b =
  String.compare (string_of_capability_id a) (string_of_capability_id b)
let compare_input_name a b =
  String.compare (string_of_capability_input_name a) (string_of_capability_input_name b)

let compare_outcome_key = function
  | Success -> 0 | Failure -> 1 | Uncertain -> 2 | Cancelled -> 3

let sort_outcome_branches branches =
  List.sort (fun (a, _) (b, _) ->
    Int.compare (compare_outcome_key a) (compare_outcome_key b)) branches

let sort_facts (facts : fact list) =
  List.sort (fun (a : fact) (b : fact) -> compare_fact_id a.fact_id b.fact_id) facts

let sort_action_inputs inputs =
  List.sort (fun a b ->
    let c = compare_input_name a.input_name b.input_name in
    if c <> 0 then c
    else
      let encode_binding ai =
        match ai.binding with
        | Literal_value v -> "0:" ^ key_of_value v
        | Fact_from_origin (fid, oid) ->
            "1:" ^ string_of_fact_id fid ^ ":" ^ string_of_origin_id oid
        | Fact_through_role (fid, rid) ->
            "2:" ^ string_of_fact_id fid ^ ":" ^ string_of_role_id rid
        | Anchor_value (oid, path) ->
            "3:" ^ string_of_origin_id oid ^ ":" ^ String.concat "/" path
        | Batch_item_context tid ->
            "4:" ^ string_of_item_template_id tid
      in
      String.compare (encode_binding a) (encode_binding b)
    ) inputs

let sort_member_origin_ids ids =
  List.sort compare_origin_id ids

let sort_origin_sites sites =
  let rank_of = function
    | Anchor_origin a -> (0, string_of_origin_id a.anchor_origin_id)
    | Action_origin a -> (1, string_of_origin_id a.action_origin_id)
    | Together_origin t -> (2, string_of_origin_id t.together_origin_id)
    | Batch_site b -> (3, string_of_batch_id b.batch_id)
  in
  List.sort (fun a b ->
    let (ra, sa) = rank_of a in
    let (rb, sb) = rank_of b in
    let c = Int.compare ra rb in
    if c <> 0 then c else String.compare sa sb
  ) sites

let sort_success_continuations scs =
  List.sort (fun a b -> compare_origin_id a.from_origin b.from_origin) scs

let sort_entry_guards guards =
  List.sort (fun a b ->
    let c = compare_fact_id a.fact_id b.fact_id in
    if c <> 0 then c
    else
      let d = Int.compare (operator_rank a.operator) (operator_rank b.operator) in
      if d <> 0 then d
      else String.compare (key_of_value a.expected) (key_of_value b.expected)
  ) guards

let sort_roles (roles : role list) =
  List.sort (fun (a : role) (b : role) -> compare_role_id a.role_id b.role_id) roles

let sort_branches (branches : branch list) =
  List.sort (fun (a : branch) (b : branch) -> compare_branch_id a.branch_id b.branch_id) branches

let sort_item_templates (templates : item_template list) =
  List.sort (fun (a : item_template) (b : item_template) ->
    compare_item_template_id a.item_template_id b.item_template_id) templates

let sort_capability_contracts (contracts : capability_contract list) =
  List.sort (fun (a : capability_contract) (b : capability_contract) ->
    compare_capability_id a.capability_id b.capability_id) contracts

let sort_execution_constraints constraints =
  List.sort (fun a b ->
    match a, b with
    | Deadline da, Deadline db -> String.compare da db) constraints

let sort_fact_contract_ids fids =
  List.sort compare_fact_id fids

let normalize_origin_site = function
  | Anchor_origin a ->
      Anchor_origin { a with declared_facts = sort_facts a.declared_facts }
  | Action_origin a ->
      Action_origin { a with
        inputs = sort_action_inputs a.inputs;
        declared_facts = sort_facts a.declared_facts;
        execution_constraints = sort_execution_constraints a.execution_constraints }
  | Together_origin t ->
      Together_origin { t with member_origin_ids = sort_member_origin_ids t.member_origin_ids }
  | Batch_site b ->
      Batch_site { b with aggregate_facts = sort_facts b.aggregate_facts }

let normalize_branch (b : branch) =
  { b with outcome_branches = sort_outcome_branches b.outcome_branches }

let normalize_role (r : role) =
  let (Role_fact_contract fids) = r.fact_contract in
  { r with fact_contract = Role_fact_contract (sort_fact_contract_ids fids) }

let normalize_item_template (t : item_template) =
  { t with
    origin_sites = (List.map normalize_origin_site t.origin_sites |> sort_origin_sites);
    branches = (List.map normalize_branch t.branches |> sort_branches);
    roles = (List.map normalize_role t.roles |> sort_roles) }

(* ------------------------------------------------------------------ *)
(*  Build the canonical program                                         *)
(* ------------------------------------------------------------------ *)

let build_canonical_program p ids =
  let rewritten_input_facts = List.map (rewrite_fact ids) p.input_facts in
  let rewritten_entry_guards = List.map (rewrite_fact_guard ids) p.entry_guards in
  let rewritten_entry_origin =
    match p.entry_origin with Some oid -> Some (canonical_origin ids oid) | None -> None
  in
  let rewritten_scs = List.map (rewrite_success_continuation ids) p.success_continuations in
  let rewritten_origin_sites = List.map (rewrite_origin_site ids) p.origin_sites in
  let rewritten_branches = List.map (rewrite_branch ids) p.branches in
  let rewritten_roles = List.map (rewrite_role ids) p.roles in
  let rewritten_item_templates = List.map (rewrite_item_template ids) p.item_templates in
  let rewritten_capability_contracts = p.capability_contracts in

  { program_id = p.program_id;
    core_version = p.core_version;
    input_facts = sort_facts rewritten_input_facts;
    entry_guards = sort_entry_guards rewritten_entry_guards;
    entry_origin = rewritten_entry_origin;
    success_continuations = sort_success_continuations rewritten_scs;
    origin_sites = (List.map normalize_origin_site rewritten_origin_sites |> sort_origin_sites);
    branches = (List.map normalize_branch rewritten_branches |> sort_branches);
    roles = (List.map normalize_role rewritten_roles |> sort_roles);
    item_templates = (List.map normalize_item_template rewritten_item_templates |> sort_item_templates);
    capability_contracts = sort_capability_contracts rewritten_capability_contracts }

(* ------------------------------------------------------------------ *)
(*  Canonical byte encoding                                              *)
(* ------------------------------------------------------------------ *)

let encode_string buf s =
  let len = String.length s in
  Buffer.add_string buf (string_of_int len);
  Buffer.add_char buf ':';
  Buffer.add_string buf s

let encode_int buf n =
  Buffer.add_string buf (string_of_int n);
  Buffer.add_char buf ';'

let encode_tag buf n =
  Buffer.add_string buf (string_of_int n);
  Buffer.add_char buf ':'

let encode_list buf f items =
  Buffer.add_string buf (string_of_int (List.length items));
  Buffer.add_char buf ':';
  List.iter f items

let encode_value buf = function
  | String_value s -> encode_tag buf 0; encode_string buf s
  | Integer_value i -> encode_tag buf 1; encode_int buf i
  | Boolean_value b -> encode_tag buf 2;
      (if b then Buffer.add_string buf "1;" else Buffer.add_string buf "0;")

let encode_scalar_type buf = function
  | String_type -> encode_tag buf 0
  | Integer_type -> encode_tag buf 1
  | Boolean_type -> encode_tag buf 2

let encode_provenance buf = function
  | Evaluation_input (Host_snapshot_key k, t) ->
      encode_tag buf 0; encode_string buf k; encode_scalar_type buf t
  | Origin_provenance (Origin_id oid) ->
      encode_tag buf 1; encode_string buf oid
  | Role_proxy (Role_id rid) ->
      encode_tag buf 2; encode_string buf rid

let encode_fact buf (f : fact) =
  encode_string buf (string_of_fact_id f.fact_id);
  encode_provenance buf f.provenance

let encode_operator buf = function
  | Equals -> encode_tag buf 0
  | Contains -> encode_tag buf 1
  | Greater_than -> encode_tag buf 2
  | Greater_than_or_equal -> encode_tag buf 3

let encode_fact_guard buf (g : fact_guard) =
  encode_string buf (string_of_fact_id g.fact_id);
  encode_operator buf g.operator;
  encode_value buf g.expected

let encode_binding buf = function
  | Literal_value v -> encode_tag buf 0; encode_value buf v
  | Fact_from_origin (Fact_id fid, Origin_id oid) ->
      encode_tag buf 1; encode_string buf fid; encode_string buf oid
  | Fact_through_role (Fact_id fid, Role_id rid) ->
      encode_tag buf 2; encode_string buf fid; encode_string buf rid
  | Anchor_value (Origin_id oid, path) ->
      encode_tag buf 3; encode_string buf oid;
      encode_list buf (fun p -> encode_string buf p) path
  | Batch_item_context (Item_template_id tid) ->
      encode_tag buf 4; encode_string buf tid

let encode_action_input buf (ai : action_input) =
  encode_string buf (string_of_capability_input_name ai.input_name);
  encode_binding buf ai.binding

let encode_constraint buf = function
  | Deadline s -> encode_tag buf 0; encode_string buf s

let encode_together_objective buf = function
  | All_members_succeed -> encode_tag buf 0

let encode_origin_site buf = function
  | Anchor_origin a ->
      encode_tag buf 0;
      encode_string buf (string_of_origin_id a.anchor_origin_id);
      encode_string buf a.event_name;
      encode_list buf (fun (f : fact) -> encode_fact buf f) a.declared_facts
  | Action_origin a ->
      encode_tag buf 1;
      encode_string buf (string_of_origin_id a.action_origin_id);
      encode_string buf (string_of_capability_id a.capability_id);
      encode_string buf (string_of_capability_contract_digest a.contract_digest);
      encode_list buf (fun ai -> encode_action_input buf ai) a.inputs;
      encode_list buf (fun (f : fact) -> encode_fact buf f) a.declared_facts;
      encode_list buf (fun c -> encode_constraint buf c) a.execution_constraints
  | Together_origin t ->
      encode_tag buf 2;
      encode_string buf (string_of_origin_id t.together_origin_id);
      encode_string buf (string_of_group_id t.group_id);
      encode_list buf (fun (Origin_id oid) -> encode_string buf oid) t.member_origin_ids;
      encode_together_objective buf t.objective
  | Batch_site b ->
      encode_tag buf 3;
      encode_string buf (string_of_batch_id b.batch_id);
      encode_string buf (string_of_batch_collection_provenance b.collection_provenance);
      encode_string buf (string_of_item_template_id b.item_template_id);
      encode_string buf (string_of_batch_traversal_policy b.traversal_policy);
      encode_string buf (string_of_batch_objective b.composite_objective);
      encode_list buf (fun (f : fact) -> encode_fact buf f) b.aggregate_facts

let encode_control_target buf = function
  | Origin_target (Origin_id oid) -> encode_tag buf 0; encode_string buf oid
  | Program_complete -> encode_tag buf 1

let encode_success_continuation buf (sc : success_continuation) =
  encode_string buf (string_of_origin_id sc.from_origin);
  encode_control_target buf sc.target

let encode_outcome buf = function
  | Success -> encode_tag buf 0
  | Failure -> encode_tag buf 1
  | Uncertain -> encode_tag buf 2
  | Cancelled -> encode_tag buf 3

let encode_branch_target buf = function
  | Continue_to (Origin_id oid) -> encode_tag buf 0; encode_string buf oid
  | Stop -> encode_tag buf 1

let encode_branch buf (b : branch) =
  encode_string buf (string_of_branch_id b.branch_id);
  encode_string buf (string_of_origin_id b.branch_subject);
  encode_list buf (fun (outcome, target) ->
    encode_outcome buf outcome;
    encode_branch_target buf target) b.outcome_branches

let encode_role_scope buf = function
  | Program_scope -> encode_tag buf 0
  | Item_template_scope (Item_template_id tid) -> encode_tag buf 1; encode_string buf tid

let encode_role buf (r : role) =
  encode_string buf (string_of_role_id r.role_id);
  encode_role_scope buf r.scope;
  let (Role_fact_contract fids) = r.fact_contract in
  encode_list buf (fun (Fact_id fid) -> encode_string buf fid) fids;
  encode_string buf (string_of_role_fulfillment r.eligible_fulfillment)

let encode_item_objective buf = function
  | Required_role (Role_id rid) -> encode_tag buf 0; encode_string buf rid

let encode_capability_contract buf (c : capability_contract) =
  encode_string buf (string_of_capability_id c.capability_id);
  encode_string buf (string_of_capability_contract_digest c.contract_digest)

let encode_item_template buf (t : item_template) =
  encode_string buf (string_of_item_template_id t.item_template_id);
  encode_list buf (fun site -> encode_origin_site buf site) t.origin_sites;
  encode_list buf (fun b -> encode_branch buf b) t.branches;
  encode_list buf (fun r -> encode_role buf r) t.roles;
  encode_item_objective buf t.objective

let encode_core_version buf (Core_version v) =
  encode_string buf v

let encode_program buf (p : program) =
  encode_core_version buf p.core_version;
  encode_list buf (fun (f : fact) -> encode_fact buf f) p.input_facts;
  encode_list buf (fun g -> encode_fact_guard buf g) p.entry_guards;
  (match p.entry_origin with
   | Some (Origin_id oid) ->
       Buffer.add_string buf "1:"; encode_string buf oid
   | None -> Buffer.add_string buf "0;");
  encode_list buf (fun sc -> encode_success_continuation buf sc) p.success_continuations;
  encode_list buf (fun site -> encode_origin_site buf site) p.origin_sites;
  encode_list buf (fun b -> encode_branch buf b) p.branches;
  encode_list buf (fun r -> encode_role buf r) p.roles;
  encode_list buf (fun t -> encode_item_template buf t) p.item_templates;
  encode_list buf (fun c -> encode_capability_contract buf c) p.capability_contracts

let make_canonical_bytes p =
  let buf = Buffer.create 8192 in
  Buffer.add_string buf canonical_prefix;
  Buffer.add_char buf canonical_prefix_byte;
  encode_program buf p;
  Buffer.contents buf

(* ------------------------------------------------------------------ *)
(*  SHA-256 and ProgramDigest                                           *)
(* ------------------------------------------------------------------ *)

let compute_sha256 bytes =
  let hash = Digestif.SHA256.digest_string bytes in
  Digestif.SHA256.to_hex hash

let make_program_digest hex =
  Program_digest ("sha256:" ^ hex)

(* ------------------------------------------------------------------ *)
(*  Public API                                                          *)
(* ------------------------------------------------------------------ *)

let canonicalize p =
  match Tethers_core_validator.validate p with
  | Error errors -> Error (Invalid_core errors)
  | Ok () ->
      (try
         let colours = final_colours p in
         let ids = assign_canonical_ids colours p in
         let canon = build_canonical_program p ids in
         let bytes = make_canonical_bytes canon in
         let hex = compute_sha256 bytes in
         let digest = make_program_digest hex in
         Ok { c_program = canon; c_bytes = bytes; c_digest = digest }
       with Failure msg when msg = "refinement exceeded safety cap" ->
         Error Refinement_exceeded)

let canonical_program c = c.c_program

let canonical_bytes c = c.c_bytes

let program_digest c = c.c_digest

let string_of_program_digest (Program_digest s) = s
