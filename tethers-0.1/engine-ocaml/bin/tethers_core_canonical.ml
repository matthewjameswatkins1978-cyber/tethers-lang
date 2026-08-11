open Tethers_core

type canonicalization_error =
  | Invalid_core of Tethers_core_validator.validation_error list

type program_digest = Program_digest of string

type canonicalized = {
  c_program : program;
  c_bytes : string;
  c_digest : program_digest;
}

let canonical_prefix = "TETHERS_CORE_CANON_V1"
let canonical_prefix_byte = '\x00'

(* ------------------------------------------------------------------ *)
(*  Entity collection helpers                                           *)
(* ------------------------------------------------------------------ *)

let origin_id_of_site = function
  | Anchor_origin a -> Some a.anchor_origin_id
  | Action_origin a -> Some a.action_origin_id
  | Together_origin t -> Some t.together_origin_id
  | Batch_site _ -> None

type origin_scope = [ `Program | `Template of item_template_id ]

let all_origins p =
  let prog = List.filter_map (fun s ->
    match origin_id_of_site s with
    | Some id -> Some (id, s, (`Program :> origin_scope))
    | None -> None) p.origin_sites
  in
  let tmpl = List.concat_map (fun t ->
    List.filter_map (fun s ->
      match origin_id_of_site s with
      | Some id -> Some (id, s, (`Template t.item_template_id :> origin_scope))
      | None -> None) t.origin_sites
  ) p.item_templates in
  prog @ tmpl

type branch_scope = [ `Program | `Template of item_template_id ]

let all_branches p =
  let prog = List.map (fun b -> (b, (`Program :> branch_scope))) p.branches in
  let tmpl = List.concat_map (fun t ->
    List.map (fun b -> (b, (`Template t.item_template_id :> branch_scope))) t.branches
  ) p.item_templates in
  prog @ tmpl

type role_scope_tag = [ `Program | `Template of item_template_id ]

let all_roles p =
  let prog = List.map (fun r -> (r, (`Program :> role_scope_tag))) p.roles in
  let tmpl = List.concat_map (fun t ->
    List.map (fun r -> (r, (`Template t.item_template_id :> role_scope_tag))) t.roles
  ) p.item_templates in
  prog @ tmpl

let all_facts p =
  let from_origins = List.concat_map (fun (_, s, _) ->
    match s with
    | Anchor_origin a -> a.declared_facts
    | Action_origin a -> a.declared_facts
    | Together_origin _ -> []
    | Batch_site b -> b.aggregate_facts
  ) (all_origins p) in
  p.input_facts @ from_origins

(* ------------------------------------------------------------------ *)
(*  Structural key types                                                *)
(* ------------------------------------------------------------------ *)

module StringMap = Map.Make(String)

type entity_keys = {
  fact_keys : string StringMap.t;
  origin_keys : string StringMap.t;
  role_keys : string StringMap.t;
  branch_keys : string StringMap.t;
  item_template_keys : string StringMap.t;
}

(* ------------------------------------------------------------------ *)
(*  Key helpers: key of semantic scalar fields                          *)
(* ------------------------------------------------------------------ *)

let key_of_value = function
  | String_value s -> "sv:" ^ string_of_int (String.length s) ^ ":" ^ s
  | Integer_value i -> "iv:" ^ string_of_int i
  | Boolean_value b -> "bv:" ^ (if b then "1" else "0")

let key_of_outcome = function
  | Success -> "out_success"
  | Failure -> "out_failure"
  | Uncertain -> "out_uncertain"
  | Cancelled -> "out_cancelled"

let key_of_provenance = function
  | Evaluation_input (Host_snapshot_key k, t) ->
      "prov_eval:" ^ k ^ ":" ^ (
        match t with String_type -> "S" | Integer_type -> "I" | Boolean_type -> "B")
  | Origin_provenance _ -> "prov_origin"
  | Role_proxy _ -> "prov_role"

let key_of_together_objective = function
  | All_members_succeed -> "tog_all"

let key_of_role_scope_scalar = function
  | Program_scope -> "scope_prog"
  | Item_template_scope _ -> "scope_tmpl"

let key_of_role_fulfillment (Role_fulfillment s) = "fulfill:" ^ string_of_int (String.length s) ^ ":" ^ s

let key_of_item_objective_scalar = function
  | Required_role _ -> "item_obj_rr"

let key_of_constraint = function
  | Deadline s -> "deadline:" ^ string_of_int (String.length s) ^ ":" ^ s

let key_of_batch_collection_provenance (Batch_collection_provenance s) =
  "bcp:" ^ string_of_int (String.length s) ^ ":" ^ s

let key_of_batch_traversal_policy (Batch_traversal_policy s) =
  "btp:" ^ string_of_int (String.length s) ^ ":" ^ s

let key_of_batch_objective (Batch_objective s) =
  "bo:" ^ string_of_int (String.length s) ^ ":" ^ s

(* ------------------------------------------------------------------ *)
(*  Round-0 keys: scalar fields only                                    *)
(* ------------------------------------------------------------------ *)

let fact_key_r0 f =
  "fact_r0:" ^ key_of_provenance f.provenance

let origin_key_r0 (_oid, site, _scope) =
  match site with
  | Anchor_origin a ->
      "anchor_r0:" ^ a.event_name
  | Action_origin a ->
      "action_r0:" ^ string_of_capability_id a.capability_id ^ ":"
      ^ string_of_capability_contract_digest a.contract_digest
  | Together_origin t ->
      "tog_r0:" ^ string_of_group_id t.group_id ^ ":"
      ^ key_of_together_objective t.objective
  | Batch_site b ->
      "batch_r0:" ^ key_of_batch_collection_provenance b.collection_provenance ^ ":"
      ^ key_of_batch_traversal_policy b.traversal_policy ^ ":"
      ^ key_of_batch_objective b.composite_objective

let role_key_r0 (r, _scope) =
  let (Role_fact_contract fids) = r.fact_contract in
  "role_r0:" ^ key_of_role_scope_scalar r.scope ^ ":"
  ^ key_of_role_fulfillment r.eligible_fulfillment ^ ":"
  ^ string_of_int (List.length fids)

let branch_key_r0 (b, _scope) =
  "branch_r0:" ^ string_of_int (List.length b.outcome_branches)

let item_template_key_r0 (t : item_template) =
  "it_r0:" ^ key_of_item_objective_scalar t.objective ^ ":"
  ^ string_of_int (List.length t.origin_sites) ^ ":"
  ^ string_of_int (List.length t.branches) ^ ":"
  ^ string_of_int (List.length t.roles)

(* ------------------------------------------------------------------ *)
(*  Round-0 construction                                                *)
(* ------------------------------------------------------------------ *)

let round_0 p =
  let fk0 = all_facts p in
  let fact_keys =
    List.fold_left (fun m (f : fact) ->
      StringMap.add (string_of_fact_id f.fact_id) (fact_key_r0 f) m
    ) StringMap.empty fk0
  in
  let origin_keys =
    List.fold_left (fun m (oid, site, scope) ->
      StringMap.add (string_of_origin_id oid) (origin_key_r0 (oid, site, scope)) m
    ) StringMap.empty (all_origins p)
  in
  let role_keys =
    List.fold_left (fun m (r, scope) ->
      StringMap.add (string_of_role_id r.role_id) (role_key_r0 (r, scope)) m
    ) StringMap.empty (all_roles p)
  in
  let branch_keys =
    List.fold_left (fun m (b, scope) ->
      StringMap.add (string_of_branch_id b.branch_id) (branch_key_r0 (b, scope)) m
    ) StringMap.empty (all_branches p)
  in
  let item_template_keys =
    List.fold_left (fun m t ->
      StringMap.add (string_of_item_template_id t.item_template_id) (item_template_key_r0 t) m
    ) StringMap.empty p.item_templates
  in
  { fact_keys; origin_keys; role_keys; branch_keys; item_template_keys }

(* ------------------------------------------------------------------ *)
(*  Key lookups by entity ID                                            *)
(* ------------------------------------------------------------------ *)

let lookup_fact keys fid =
  match StringMap.find_opt (string_of_fact_id fid) keys.fact_keys with
  | Some k -> k | None -> "fact_unknown"

let lookup_origin keys oid =
  match StringMap.find_opt (string_of_origin_id oid) keys.origin_keys with
  | Some k -> k | None -> "origin_unknown"

let lookup_role keys rid =
  match StringMap.find_opt (string_of_role_id rid) keys.role_keys with
  | Some k -> k | None -> "role_unknown"

let lookup_branch keys bid =
  match StringMap.find_opt (string_of_branch_id bid) keys.branch_keys with
  | Some k -> k | None -> "branch_unknown"

let lookup_item_template keys tid =
  match StringMap.find_opt (string_of_item_template_id tid) keys.item_template_keys with
  | Some k -> k | None -> "it_unknown"

(* ------------------------------------------------------------------ *)
(*  Round-N key computation                                             *)
(* ------------------------------------------------------------------ *)

let fact_key_rn keys f =
  let prov_key = match f.provenance with
    | Evaluation_input (Host_snapshot_key k, t) ->
        "E:" ^ k ^ ":" ^ (
          match t with String_type -> "S" | Integer_type -> "I" | Boolean_type -> "B")
    | Origin_provenance oid -> "O:" ^ lookup_origin keys oid
    | Role_proxy rid -> "R:" ^ lookup_role keys rid
  in
  "F:" ^ prov_key

let origin_key_rn keys (_oid, site, _scope) =
  match site with
  | Anchor_origin a ->
      let fact_keys_sorted =
        List.map (fun (f : fact) -> lookup_fact keys f.fact_id) a.declared_facts
        |> List.sort String.compare
      in
      "A:" ^ a.event_name ^ ":facts:" ^ String.concat "," fact_keys_sorted
  | Action_origin a ->
      let fact_keys_sorted =
        List.map (fun (f : fact) -> lookup_fact keys f.fact_id) a.declared_facts
        |> List.sort String.compare
      in
      let input_keys =
        List.map (fun ai ->
          match ai.binding with
          | Literal_value v -> "L:" ^ key_of_value v
          | Anchor_value (oid', path) ->
              "AV:" ^ string_of_origin_id oid' ^ ":" ^ String.concat "/" path
          | Fact_from_origin (fid, oid') ->
              "FO:" ^ string_of_fact_id fid ^ ":" ^ lookup_origin keys oid'
          | Fact_through_role (fid, rid) ->
              "FT:" ^ string_of_fact_id fid ^ ":" ^ lookup_role keys rid
          | Batch_item_context (Item_template_id tid) ->
              "BIC:" ^ lookup_item_template keys (item_template_id_of_string tid)
        ) a.inputs
        |> List.sort String.compare
      in
      let constraint_keys =
        List.map (fun c -> key_of_constraint c) a.execution_constraints
        |> List.sort String.compare
      in
      "Ac:" ^ string_of_capability_id a.capability_id ^ ":"
      ^ string_of_capability_contract_digest a.contract_digest
      ^ ":facts:" ^ String.concat "," fact_keys_sorted
      ^ ":inputs:" ^ String.concat "," input_keys
      ^ ":constraints:" ^ String.concat "," constraint_keys
  | Together_origin t ->
      let member_keys =
        List.map (fun oid' -> lookup_origin keys oid') t.member_origin_ids
        |> List.sort String.compare
      in
      "T:" ^ string_of_group_id t.group_id ^ ":"
      ^ key_of_together_objective t.objective
      ^ ":members:" ^ String.concat "," member_keys
  | Batch_site b ->
      "Ba:" ^ key_of_batch_collection_provenance b.collection_provenance ^ ":"
      ^ lookup_item_template keys b.item_template_id ^ ":"
      ^ key_of_batch_traversal_policy b.traversal_policy ^ ":"
      ^ key_of_batch_objective b.composite_objective
      ^ ":facts:" ^ String.concat ","
          (List.map (fun (f : fact) -> lookup_fact keys f.fact_id) b.aggregate_facts
           |> List.sort String.compare)

let role_key_rn keys (r, _scope) =
  let (Role_fact_contract fids) = r.fact_contract in
  let fact_keys_sorted =
    List.map (fun fid -> lookup_fact keys fid) fids
    |> List.sort String.compare
  in
  let scope_key = match r.scope with
    | Program_scope -> "P"
    | Item_template_scope tid -> "T:" ^ lookup_item_template keys tid
  in
  "Ro:" ^ scope_key ^ ":"
  ^ key_of_role_fulfillment r.eligible_fulfillment
  ^ ":facts:" ^ String.concat "," fact_keys_sorted

let branch_key_rn keys (b, _scope) =
  let outcome_keys =
    List.map (fun (outcome, target) ->
      key_of_outcome outcome ^ ":"
      ^ (match target with
         | Continue_to oid -> "C:" ^ lookup_origin keys oid
         | Stop -> "S"))
      b.outcome_branches
    |> List.sort String.compare
  in
  "Br:" ^ string_of_origin_id b.branch_subject
  ^ ":outcomes:" ^ String.concat "," outcome_keys

let item_template_key_rn keys (t : item_template) =
  let origin_keys_list =
    List.map (fun site ->
      match origin_id_of_site site with
      | Some oid -> lookup_origin keys oid
      | None -> "none")
      t.origin_sites
    |> List.sort String.compare
  in
  let branch_keys_list =
    List.map (fun b -> lookup_branch keys b.branch_id) t.branches
    |> List.sort String.compare
  in
  let role_keys_list =
    List.map (fun r -> lookup_role keys r.role_id) t.roles
    |> List.sort String.compare
  in
  let obj_key = match t.objective with
    | Required_role rid -> "RR:" ^ lookup_role keys rid
  in
  "IT:" ^ obj_key
  ^ ":origins:" ^ String.concat "," origin_keys_list
  ^ ":branches:" ^ String.concat "," branch_keys_list
  ^ ":roles:" ^ String.concat "," role_keys_list

(* ------------------------------------------------------------------ *)
(*  Refinement round                                                    *)
(* ------------------------------------------------------------------ *)

let refine_keys prev p =
  let fact_keys =
    List.fold_left (fun m (f : fact) ->
      StringMap.add (string_of_fact_id f.fact_id) (fact_key_rn prev f) m
    ) StringMap.empty (all_facts p)
  in
  let origin_keys =
    List.fold_left (fun m (oid, site, scope) ->
      StringMap.add (string_of_origin_id oid) (origin_key_rn prev (oid, site, scope)) m
    ) StringMap.empty (all_origins p)
  in
  let role_keys =
    List.fold_left (fun m (r, scope) ->
      StringMap.add (string_of_role_id r.role_id) (role_key_rn prev (r, scope)) m
    ) StringMap.empty (all_roles p)
  in
  let branch_keys =
    List.fold_left (fun m (b, scope) ->
      StringMap.add (string_of_branch_id b.branch_id) (branch_key_rn prev (b, scope)) m
    ) StringMap.empty (all_branches p)
  in
  let item_template_keys =
    List.fold_left (fun m (t : item_template) ->
      StringMap.add (string_of_item_template_id t.item_template_id) (item_template_key_rn prev t) m
    ) StringMap.empty p.item_templates
  in
  { fact_keys; origin_keys; role_keys; branch_keys; item_template_keys }

let keys_equal a b =
  StringMap.equal (=) a.fact_keys b.fact_keys
  && StringMap.equal (=) a.origin_keys b.origin_keys
  && StringMap.equal (=) a.role_keys b.role_keys
  && StringMap.equal (=) a.branch_keys b.branch_keys
  && StringMap.equal (=) a.item_template_keys b.item_template_keys

let rec refine_until_stable n max_n prev p =
  if n >= max_n then prev
  else
    let next = refine_keys prev p in
    if keys_equal prev next then next
    else refine_until_stable (n + 1) max_n next p

let final_keys p =
  let r0 = round_0 p in
  refine_until_stable 1 20 r0 p

(* ------------------------------------------------------------------ *)
(*  Canonical ID assignment                                             *)
(* ------------------------------------------------------------------ *)

type canonical_ids = {
  origin_order : (origin_id * origin_id) list;
  fact_order : (fact_id * fact_id) list;
  role_order : (role_id * role_id) list;
  branch_order : (branch_id * branch_id) list;
  group_order : (group_id * group_id) list;
  batch_order : (batch_id * batch_id) list;
  item_template_order : (item_template_id * item_template_id) list;
}

let assign_canonical_ids keys p =
  let all_orig = all_origins p in
  let all_fact = all_facts p in
  let all_rl = all_roles p in
  let all_br = all_branches p in
  let all_it = p.item_templates in

  let sorted_origins =
    List.map (fun (oid, _, _) ->
      let k = StringMap.find (string_of_origin_id oid) keys.origin_keys in
      (string_of_origin_id oid, k)) all_orig
    |> List.sort (fun (_, a) (_, b) -> String.compare a b)
  in
  let origin_order =
    List.mapi (fun i (oid_s, _) ->
      (origin_id_of_string oid_s,
       origin_id_of_string ("O" ^ string_of_int (i + 1)))) sorted_origins
  in

  let sorted_facts =
    List.map (fun (f : fact) ->
      let k = StringMap.find (string_of_fact_id f.fact_id) keys.fact_keys in
      (string_of_fact_id f.fact_id, k)) all_fact
    |> List.sort (fun (_, a) (_, b) -> String.compare a b)
  in
  let fact_order =
    List.mapi (fun i (fid_s, _) ->
      (fact_id_of_string fid_s,
       fact_id_of_string ("F" ^ string_of_int (i + 1)))) sorted_facts
  in

  let sorted_roles =
    List.map (fun (r, _) ->
      let k = StringMap.find (string_of_role_id r.role_id) keys.role_keys in
      (string_of_role_id r.role_id, k)) all_rl
    |> List.sort (fun (_, a) (_, b) -> String.compare a b)
  in
  let role_order =
    List.mapi (fun i (rid_s, _) ->
      (role_id_of_string rid_s,
       role_id_of_string ("R" ^ string_of_int (i + 1)))) sorted_roles
  in

  let sorted_branches =
    List.map (fun (b, _) ->
      let k = StringMap.find (string_of_branch_id b.branch_id) keys.branch_keys in
      (string_of_branch_id b.branch_id, k)) all_br
    |> List.sort (fun (_, a) (_, b) -> String.compare a b)
  in
  let branch_order =
    List.mapi (fun i (bid_s, _) ->
      (branch_id_of_string bid_s,
       branch_id_of_string ("B" ^ string_of_int (i + 1)))) sorted_branches
  in

  let sorted_item_templates =
    List.map (fun t ->
      let k = StringMap.find (string_of_item_template_id t.item_template_id) keys.item_template_keys in
      (string_of_item_template_id t.item_template_id, k)) all_it
    |> List.sort (fun (_, a) (_, b) -> String.compare a b)
  in
  let item_template_order =
    List.mapi (fun i (tid_s, _) ->
      (item_template_id_of_string tid_s,
       item_template_id_of_string ("IT" ^ string_of_int (i + 1)))) sorted_item_templates
  in

  let group_ids = List.filter_map (fun (_, s, _) ->
    match s with Together_origin t -> Some t.group_id | _ -> None) all_orig
  in
  let sorted_groups =
    List.map (fun gid -> (gid, string_of_group_id gid)) group_ids
    |> List.sort (fun (_, a) (_, b) -> String.compare a b)
  in
  let group_order =
    List.mapi (fun i (gid, _) ->
      (gid, group_id_of_string ("G" ^ string_of_int (i + 1)))) sorted_groups
  in

  let batch_ids = List.filter_map (fun (_, s, _) ->
    match s with Batch_site b -> Some b.batch_id | _ -> None) all_orig
  in
  let sorted_batches =
    List.map (fun bid -> (bid, string_of_batch_id bid)) batch_ids
    |> List.sort (fun (_, a) (_, b) -> String.compare a b)
  in
  let batch_order =
    List.mapi (fun i (bid, _) ->
      (bid, batch_id_of_string ("BA" ^ string_of_int (i + 1)))) sorted_batches
  in

  { origin_order; fact_order; role_order; branch_order;
    group_order; batch_order; item_template_order }

(* ------------------------------------------------------------------ *)
(*  Reference rewriting                                                 *)
(* ------------------------------------------------------------------ *)

let canonical_origin ids oid =
  match List.assoc_opt oid ids.origin_order with Some c -> c | None -> oid

let canonical_fact ids fid =
  match List.assoc_opt fid ids.fact_order with Some c -> c | None -> fid

let canonical_role ids rid =
  match List.assoc_opt rid ids.role_order with Some c -> c | None -> rid

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
    | Role_proxy rid -> Role_proxy (canonical_role ids rid)
  in
  { fact_id = canonical_fact ids f.fact_id;
    schema_description = f.schema_description;
    provenance }

let rewrite_fact_guard ids g =
  { fact_id = canonical_fact ids g.fact_id;
    operator = g.operator;
    expected = g.expected }

let rewrite_input_binding ids = function
  | Literal_value v -> Literal_value v
  | Fact_from_origin (fid, oid) ->
      Fact_from_origin (canonical_fact ids fid, canonical_origin ids oid)
  | Fact_through_role (fid, rid) ->
      Fact_through_role (canonical_fact ids fid, canonical_role ids rid)
  | Anchor_value (oid, path) -> Anchor_value (canonical_origin ids oid, path)
  | Batch_item_context tid -> Batch_item_context (canonical_item_template ids tid)

let rewrite_action_input ids ai =
  { input_name = ai.input_name;
    binding = rewrite_input_binding ids ai.binding }

let rewrite_origin_site ids = function
  | Anchor_origin a ->
      Anchor_origin { anchor_origin_id = canonical_origin ids a.anchor_origin_id;
                       event_name = a.event_name;
                       declared_facts = List.map (rewrite_fact ids) a.declared_facts }
  | Action_origin a ->
      Action_origin { action_origin_id = canonical_origin ids a.action_origin_id;
                       capability_id = a.capability_id;
                       contract_digest = a.contract_digest;
                       inputs = List.map (rewrite_action_input ids) a.inputs;
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
  let scope =
    match r.scope with
    | Program_scope -> Program_scope
    | Item_template_scope tid -> Item_template_scope (canonical_item_template ids tid)
  in
  { role_id = canonical_role ids r.role_id;
    scope;
    fact_contract = Role_fact_contract (List.map (canonical_fact ids) fids);
    eligible_fulfillment = r.eligible_fulfillment }

let rewrite_item_objective ids = function
  | Required_role rid -> Required_role (canonical_role ids rid)

let rewrite_item_template ids t =
  { item_template_id = canonical_item_template ids t.item_template_id;
    origin_sites = List.map (rewrite_origin_site ids) t.origin_sites;
    branches = List.map (rewrite_branch ids) t.branches;
    roles = List.map (rewrite_role ids) t.roles;
    objective = rewrite_item_objective ids t.objective }

(* ------------------------------------------------------------------ *)
(*  Collection sorting                                                  *)
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
  List.sort (fun a b -> compare_input_name a.input_name b.input_name) inputs

let sort_member_origin_ids ids =
  List.sort compare_origin_id ids

let sort_origin_sites sites =
  let origin_of = function
    | Anchor_origin a -> a.anchor_origin_id
    | Action_origin a -> a.action_origin_id
    | Together_origin t -> t.together_origin_id
    | Batch_site _ -> origin_id_of_string ""
  in
  List.sort (fun a b -> compare_origin_id (origin_of a) (origin_of b)) sites

let sort_success_continuations (scs : success_continuation list) =
  List.sort (fun (a : success_continuation) (b : success_continuation) -> compare_origin_id a.from_origin b.from_origin) scs

let sort_entry_guards (guards : fact_guard list) =
  List.sort (fun (a : fact_guard) (b : fact_guard) -> compare_fact_id a.fact_id b.fact_id) guards

let sort_roles (roles : role list) =
  List.sort (fun (a : role) (b : role) -> compare_role_id a.role_id b.role_id) roles

let sort_branches (branches : branch list) =
  List.sort (fun (a : branch) (b : branch) -> compare_branch_id a.branch_id b.branch_id) branches

let sort_item_templates (templates : item_template list) =
  List.sort (fun (a : item_template) (b : item_template) -> compare_item_template_id a.item_template_id b.item_template_id) templates

let sort_capability_contracts (contracts : capability_contract list) =
  List.sort (fun (a : capability_contract) (b : capability_contract) -> compare_capability_id a.capability_id b.capability_id) contracts

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
(*  Canonical byte encoding                                             *)
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
      let keys = final_keys p in
      let ids = assign_canonical_ids keys p in
      let canon = build_canonical_program p ids in
      let bytes = make_canonical_bytes canon in
      let hex = compute_sha256 bytes in
      let digest = make_program_digest hex in
      Ok { c_program = canon; c_bytes = bytes; c_digest = digest }

let canonical_program c = c.c_program

let canonical_bytes c = c.c_bytes

let program_digest c = c.c_digest

let string_of_program_digest (Program_digest s) = s
