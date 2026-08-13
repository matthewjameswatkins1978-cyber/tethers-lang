(* ==================================================================
   CANONICAL FORMAT V2 — SHARED FROZEN FORMAT LAYER

   This module contains the frozen Enc_V2 byte encoder, domain
   separation, primitive encoders, typed label maps, entity
   collection helpers, and the unsigned-byte lexicographic comparator.

   Both the reference oracle and the production canonicaliser import
   this module.  Search implementations remain independent.
   ================================================================== *)

open Tethers_core

(* ================================================================== *)
(*  Domain separation                                                   *)
(* ================================================================== *)

let domain_v2 : bytes =
  Bytes.of_string "TETHERS_CORE_CANON_V2\x00"

(* ================================================================== *)
(*  SHA-256 digest (§16, §17)                                           *)
(* ================================================================== *)

let sha256_hex (data : bytes) : string =
  Digestif.SHA256.(to_hex (digest_bytes data))

let digest_string_v2 (hex : string) : string =
  "tethers:v2:sha256:" ^ hex

(* ================================================================== *)
(*  Unsigned byte lexicographic comparator                              *)
(* ================================================================== *)

let compare_bytes_lex_unsigned (a : string) (b : string) : int =
  let len_a = String.length a in
  let len_b = String.length b in
  let min_len = min len_a len_b in
  let rec loop i =
    if i >= min_len then
      Int.compare len_a len_b
    else
      let ca = Char.code (String.get a i) in
      let cb = Char.code (String.get b i) in
      if ca <> cb then ca - cb
      else loop (i + 1)
  in
  loop 0

(* ================================================================== *)
(*  Primitive encoders (frozen §6.2)                                    *)
(* ================================================================== *)

let encode_string (s : string) : string =
  string_of_int (String.length s) ^ ":" ^ s

let encode_int (n : int) : string =
  string_of_int n ^ ";"

let encode_tag (n : int) : string =
  string_of_int n ^ ":"

let encode_list (f : 'a -> string) (items : 'a list) : string =
  string_of_int (List.length items) ^ ":" ^ String.concat "" (List.map f items)

let encode_option (f : 'a -> string) (opt : 'a option) : string =
  match opt with
  | None -> "0;"
  | Some x -> "1:" ^ f x

(* ================================================================== *)
(*  Tag assignments (frozen §6.3)                                       *)
(* ================================================================== *)

let operator_rank = function
  | Equals -> 0
  | Contains -> 1
  | Greater_than -> 2
  | Greater_than_or_equal -> 3

let outcome_rank = function
  | Success -> 0
  | Failure -> 1
  | Uncertain -> 2
  | Cancelled -> 3

(* ================================================================== *)
(*  Family-safe typed label maps                                        *)
(* ================================================================== *)

module OriginMap = Map.Make (struct type t = origin_id let compare = compare end)
module FactMap = Map.Make (struct type t = fact_id let compare = compare end)
module BranchMap = Map.Make (struct type t = branch_id let compare = compare end)
module BatchMap = Map.Make (struct type t = batch_id let compare = compare end)
module TemplateMap = Map.Make (struct type t = item_template_id let compare = compare end)
module RoleMap = Map.Make (struct type t = role_id let compare = compare end)

type scoped_role_key =
  | Program_role of role_id
  | Template_role of item_template_id * role_id

module ScopedRoleMap = Map.Make (struct type t = scoped_role_key let compare = compare end)

(* ================================================================== *)
(*  Label assignment                                                    *)
(* ================================================================== *)

type label_assignment = {
  origin_labels   : int OriginMap.t;
  fact_labels     : int FactMap.t;
  branch_labels   : int BranchMap.t;
  batch_labels    : int BatchMap.t;
  template_labels : int TemplateMap.t;
  role_labels     : int ScopedRoleMap.t;
}

(* ================================================================== *)
(*  Label lookups                                                       *)
(* ================================================================== *)

let lookup_origin (la : label_assignment) (oid : origin_id) : int =
  match OriginMap.find_opt oid la.origin_labels with
  | Some l -> l
  | None -> failwith "INTERNAL ERROR: missing origin label"

let lookup_fact (la : label_assignment) (fid : fact_id) : int =
  match FactMap.find_opt fid la.fact_labels with
  | Some l -> l
  | None -> failwith "INTERNAL ERROR: missing fact label"

let lookup_branch (la : label_assignment) (bid : branch_id) : int =
  match BranchMap.find_opt bid la.branch_labels with
  | Some l -> l
  | None -> failwith "INTERNAL ERROR: missing branch label"

let lookup_batch (la : label_assignment) (bid : batch_id) : int =
  match BatchMap.find_opt bid la.batch_labels with
  | Some l -> l
  | None -> failwith "INTERNAL ERROR: missing batch label"

let lookup_template (la : label_assignment) (tid : item_template_id) : int =
  match TemplateMap.find_opt tid la.template_labels with
  | Some l -> l
  | None -> failwith "INTERNAL ERROR: missing template label"

let lookup_scoped_role (la : label_assignment) (key : scoped_role_key) : int =
  match ScopedRoleMap.find_opt key la.role_labels with
  | Some l -> l
  | None -> failwith "INTERNAL ERROR: missing scoped role label"

let lookup_role_in_scope (la : label_assignment) (scope : role_scope) (rid : role_id) : int =
  let key = match scope with
    | Program_scope -> Program_role rid
    | Item_template_scope tid -> Template_role (tid, rid)
  in
  lookup_scoped_role la key

(* ================================================================== *)
(*  Entity collection from program                                      *)
(* ================================================================== *)

let origin_id_of_site = function
  | Anchor_origin a -> Some a.anchor_origin_id
  | Action_origin a -> Some a.action_origin_id
  | Together_origin t -> Some t.together_origin_id
  | Batch_site _ -> None

let collect_origins (p : program) : (origin_id * origin_site) list =
  let prog = List.filter_map (fun s ->
    match origin_id_of_site s with
    | Some id -> Some (id, s)
    | None -> None
  ) p.origin_sites in
  let tmpl = List.concat (List.map (fun (t : item_template) ->
    List.filter_map (fun s ->
      match origin_id_of_site s with
      | Some id -> Some (id, s)
      | None -> None
    ) t.origin_sites
  ) p.item_templates) in
  prog @ tmpl

let collect_batches (p : program) : (batch_id * batch_site) list =
  let prog = List.filter_map (fun s ->
    match s with
    | Batch_site b -> Some (b.batch_id, b)
    | _ -> None
  ) p.origin_sites in
  let tmpl = List.concat (List.map (fun (t : item_template) ->
    List.filter_map (fun s ->
      match s with
      | Batch_site b -> Some (b.batch_id, b)
      | _ -> None
    ) t.origin_sites
  ) p.item_templates) in
  prog @ tmpl

let collect_facts (p : program) : fact list =
  let from_input = p.input_facts in
  let from_origins =
    List.concat (List.map (fun (_, site) ->
      match site with
      | Anchor_origin a -> a.declared_facts
      | Action_origin a -> a.declared_facts
      | Together_origin _ -> []
      | Batch_site b -> b.aggregate_facts
    ) (collect_origins p))
  in
  let from_batches =
    List.concat (List.map (fun (_, b) -> b.aggregate_facts) (collect_batches p))
  in
  from_input @ from_origins @ from_batches

let collect_branches (p : program) : (branch * [`Program | `Template of item_template_id]) list =
  let prog = List.map (fun b -> (b, `Program)) p.branches in
  let tmpl = List.concat (List.map (fun t ->
    List.map (fun b -> (b, `Template t.item_template_id)) t.branches
  ) p.item_templates) in
  prog @ tmpl

let collect_roles (p : program) : (role * [`Program | `Template of item_template_id]) list =
  let prog = List.map (fun r -> (r, `Program)) p.roles in
  let tmpl = List.concat (List.map (fun t ->
    List.map (fun r -> (r, `Template t.item_template_id)) t.roles
  ) p.item_templates) in
  prog @ tmpl

(* ================================================================== *)
(*  Mixed-origin/Batch sort key (§9.3.1)                                *)
(* ================================================================== *)

type sort_key = int * int

let origin_sort_key (la : label_assignment) (site : origin_site) : sort_key =
  match site with
  | Anchor_origin a ->
      (0, lookup_origin la a.anchor_origin_id)
  | Action_origin a ->
      (0, lookup_origin la a.action_origin_id)
  | Together_origin t ->
      (0, lookup_origin la t.together_origin_id)
  | Batch_site b ->
      (1, lookup_batch la b.batch_id)

let sort_origin_sites (la : label_assignment) (sites : origin_site list) : origin_site list =
  List.sort (fun a b ->
    let ka = origin_sort_key la a in
    let kb = origin_sort_key la b in
    compare ka kb
  ) sites

(* ================================================================== *)
(*  Frozen Enc_V2 encoder (§6.4)                                       *)
(* ================================================================== *)

let encode_scalar_type = function
  | String_type -> encode_tag 0
  | Integer_type -> encode_tag 1
  | Boolean_type -> encode_tag 2

let encode_value = function
  | String_value s -> encode_tag 0 ^ encode_string s
  | Integer_value i -> encode_tag 1 ^ encode_int i
  | Boolean_value b -> encode_tag 2 ^ (if b then "1;" else "0;")

let encode_provenance (la : label_assignment) (fact_scope : role_scope) = function
  | Evaluation_input (Host_snapshot_key k, t) ->
      encode_tag 0 ^ encode_string k ^ encode_scalar_type t
  | Origin_provenance oid ->
      encode_tag 1 ^ encode_int (lookup_origin la oid)
  | Role_proxy rid ->
      encode_tag 2 ^ encode_int (lookup_role_in_scope la fact_scope rid)

let encode_fact (la : label_assignment) ~(fact_scope : role_scope) (f : fact) : string =
  encode_int (lookup_fact la f.fact_id) ^
  encode_provenance la fact_scope f.provenance

let encode_fact_guard (la : label_assignment) (g : fact_guard) : string =
  encode_int (lookup_fact la g.fact_id) ^
  encode_tag (operator_rank g.operator) ^
  encode_value g.expected

let encode_binding (la : label_assignment) ~(origin_scope : role_scope) = function
  | Literal_value v -> encode_tag 0 ^ encode_value v
  | Fact_from_origin (fid, oid) ->
      encode_tag 1 ^ encode_int (lookup_fact la fid) ^
      encode_int (lookup_origin la oid)
  | Fact_through_role (fid, rid) ->
      encode_tag 2 ^ encode_int (lookup_fact la fid) ^
      encode_int (lookup_role_in_scope la origin_scope rid)
  | Anchor_value (oid, path) ->
      encode_tag 3 ^ encode_int (lookup_origin la oid) ^
      encode_list (fun p -> encode_string p) path
  | Batch_item_context tid ->
      encode_tag 4 ^ encode_int (lookup_template la tid)

let encode_action_input (la : label_assignment) ~(origin_scope : role_scope) (ai : action_input) : string =
  encode_string (string_of_capability_input_name ai.input_name) ^
  encode_binding la ~origin_scope ai.binding

let encode_constraint = function
  | Deadline s -> encode_tag 0 ^ encode_string s

let encode_origin_site (la : label_assignment) ~(origin_scope : role_scope) (site : origin_site) : string =
  match site with
  | Anchor_origin a ->
      let sorted_facts = List.sort (fun (x : fact) (y : fact) ->
        Int.compare (lookup_fact la x.fact_id) (lookup_fact la y.fact_id)
      ) a.declared_facts in
      encode_tag 0 ^
      encode_int (lookup_origin la a.anchor_origin_id) ^
      encode_string a.event_name ^
      encode_list (encode_fact la ~fact_scope:origin_scope) sorted_facts
  | Action_origin a ->
      let sorted_inputs = List.sort (fun (x : action_input) (y : action_input) ->
        let c = String.compare
          (string_of_capability_input_name x.input_name)
          (string_of_capability_input_name y.input_name)
        in
        if c <> 0 then c
        else String.compare
          (encode_binding la ~origin_scope x.binding)
          (encode_binding la ~origin_scope y.binding)
      ) a.inputs in
      let sorted_facts = List.sort (fun (x : fact) (y : fact) ->
        Int.compare (lookup_fact la x.fact_id) (lookup_fact la y.fact_id)
      ) a.declared_facts in
      let sorted_constraints = List.sort (fun x y ->
        match (x, y) with
        | Deadline s1, Deadline s2 -> String.compare s1 s2
      ) a.execution_constraints in
      encode_tag 1 ^
      encode_int (lookup_origin la a.action_origin_id) ^
      encode_string (string_of_capability_id a.capability_id) ^
      encode_string (string_of_capability_contract_digest a.contract_digest) ^
      encode_list (encode_action_input la ~origin_scope) sorted_inputs ^
      encode_list (encode_fact la ~fact_scope:origin_scope) sorted_facts ^
      encode_list encode_constraint sorted_constraints
  | Together_origin t ->
      let member_labels = List.map (fun oid ->
        lookup_origin la oid
      ) t.member_origin_ids |> List.sort Int.compare in
      encode_tag 2 ^
      encode_int (lookup_origin la t.together_origin_id) ^
      encode_list encode_int member_labels ^
      encode_tag 0
  | Batch_site b ->
      let sorted_facts = List.sort (fun (x : fact) (y : fact) ->
        Int.compare (lookup_fact la x.fact_id) (lookup_fact la y.fact_id)
      ) b.aggregate_facts in
      encode_tag 3 ^
      encode_int (lookup_batch la b.batch_id) ^
      encode_string (string_of_batch_collection_provenance b.collection_provenance) ^
      encode_int (lookup_template la b.item_template_id) ^
      encode_string (string_of_batch_traversal_policy b.traversal_policy) ^
      encode_string (string_of_batch_objective b.composite_objective) ^
      encode_list (encode_fact la ~fact_scope:origin_scope) sorted_facts

let encode_branch (la : label_assignment) (b : branch) : string =
  let sorted_outcomes = List.sort (fun (o1, _) (o2, _) ->
    Int.compare (outcome_rank o1) (outcome_rank o2)
  ) b.outcome_branches in
  encode_int (lookup_branch la b.branch_id) ^
  encode_int (lookup_origin la b.branch_subject) ^
  encode_list (fun (outcome, target) ->
    encode_tag (outcome_rank outcome) ^
    (match target with
     | Continue_to oid -> encode_tag 0 ^ encode_int (lookup_origin la oid)
     | Stop -> encode_tag 1)
  ) sorted_outcomes

let encode_role_scope (la : label_assignment) = function
  | Program_scope -> encode_tag 0
  | Item_template_scope tid ->
      encode_tag 1 ^ encode_int (lookup_template la tid)

let encode_role (la : label_assignment) ~(role_scope : role_scope) (r : role) : string =
  let scoped_key = match role_scope with
    | Program_scope -> Program_role r.role_id
    | Item_template_scope tid -> Template_role (tid, r.role_id)
  in
  let (Role_fact_contract fids) = r.fact_contract in
  let sorted_fids = List.sort (fun a b ->
    Int.compare (lookup_fact la a) (lookup_fact la b)
  ) fids in
  encode_int (lookup_scoped_role la scoped_key) ^
  encode_role_scope la r.scope ^
  encode_list (fun fid ->
    encode_int (lookup_fact la fid)
  ) sorted_fids ^
  encode_string (string_of_role_fulfillment r.eligible_fulfillment)

let encode_item_objective (la : label_assignment) ~(template_scope : item_template_id) = function
  | Required_role rid ->
      encode_tag 0 ^ encode_int (lookup_role_in_scope la (Item_template_scope template_scope) rid)

let encode_item_template (la : label_assignment) (t : item_template) : string =
  let sorted_origin_sites = sort_origin_sites la t.origin_sites in
  let sorted_branches = List.sort (fun (a : branch) (b : branch) ->
    Int.compare (lookup_branch la a.branch_id) (lookup_branch la b.branch_id)
  ) t.branches in
  let sorted_roles = List.sort (fun (a : role) (b : role) ->
    let key_a = match a.scope with
      | Program_scope -> Program_role a.role_id
      | Item_template_scope tid -> Template_role (tid, a.role_id)
    in
    let key_b = match b.scope with
      | Program_scope -> Program_role b.role_id
      | Item_template_scope tid -> Template_role (tid, b.role_id)
    in
    Int.compare (lookup_scoped_role la key_a) (lookup_scoped_role la key_b)
  ) t.roles in
  encode_int (lookup_template la t.item_template_id) ^
  encode_list (encode_origin_site la ~origin_scope:(Item_template_scope t.item_template_id)) sorted_origin_sites ^
  encode_list (encode_branch la) sorted_branches ^
  encode_list (encode_role la ~role_scope:(Item_template_scope t.item_template_id)) sorted_roles ^
  encode_item_objective la ~template_scope:t.item_template_id t.objective

let encode_capability_contract (_la : label_assignment) (c : capability_contract) : string =
  encode_string (string_of_capability_id c.capability_id) ^
  encode_string (string_of_capability_contract_digest c.contract_digest)

(* ================================================================== *)
(*  Top-level encoder                                                   *)
(* ================================================================== *)

let encode_program (la : label_assignment) (p : program) : string =
  let sorted_input_facts = List.sort (fun (a : fact) (b : fact) ->
    Int.compare (lookup_fact la a.fact_id) (lookup_fact la b.fact_id)
  ) p.input_facts in

  let sorted_entry_guards = List.sort (fun (a : fact_guard) (b : fact_guard) ->
    let c = Int.compare (lookup_fact la a.fact_id) (lookup_fact la b.fact_id) in
    if c <> 0 then c
    else
      let c2 = Int.compare (operator_rank a.operator) (operator_rank b.operator) in
      if c2 <> 0 then c2
      else String.compare (encode_value a.expected) (encode_value b.expected)
  ) p.entry_guards in

  let sorted_success_continuations = List.sort (fun (a : success_continuation) (b : success_continuation) ->
    Int.compare (lookup_origin la a.from_origin) (lookup_origin la b.from_origin)
  ) p.success_continuations in

  let sorted_origin_sites = sort_origin_sites la p.origin_sites in

  let sorted_branches = List.sort (fun (a : branch) (b : branch) ->
    Int.compare (lookup_branch la a.branch_id) (lookup_branch la b.branch_id)
  ) p.branches in

  let sorted_roles = List.sort (fun (a : role) (b : role) ->
    let key_a = match a.scope with
      | Program_scope -> Program_role a.role_id
      | Item_template_scope tid -> Template_role (tid, a.role_id)
    in
    let key_b = match b.scope with
      | Program_scope -> Program_role b.role_id
      | Item_template_scope tid -> Template_role (tid, b.role_id)
    in
    Int.compare (lookup_scoped_role la key_a) (lookup_scoped_role la key_b)
  ) p.roles in

  let sorted_item_templates = List.sort (fun (a : item_template) (b : item_template) ->
    Int.compare (lookup_template la a.item_template_id) (lookup_template la b.item_template_id)
  ) p.item_templates in

  let sorted_capability_contracts = List.sort (fun (a : capability_contract) (b : capability_contract) ->
    String.compare (string_of_capability_id a.capability_id) (string_of_capability_id b.capability_id)
  ) p.capability_contracts in

  encode_string (string_of_core_version p.core_version) ^
  encode_list (encode_fact la ~fact_scope:Program_scope) sorted_input_facts ^
  encode_list (encode_fact_guard la) sorted_entry_guards ^
  encode_option (fun oid -> encode_int (lookup_origin la oid)) p.entry_origin ^
  encode_list (fun (sc : success_continuation) ->
    encode_int (lookup_origin la sc.from_origin) ^
    (match sc.target with
     | Origin_target oid -> encode_tag 0 ^ encode_int (lookup_origin la oid)
     | Program_complete -> encode_tag 1)
  ) sorted_success_continuations ^
  encode_list (encode_origin_site la ~origin_scope:Program_scope) sorted_origin_sites ^
  encode_list (encode_branch la) sorted_branches ^
  encode_list (encode_role la ~role_scope:Program_scope) sorted_roles ^
  encode_list (encode_item_template la) sorted_item_templates ^
  encode_list (encode_capability_contract la) sorted_capability_contracts
