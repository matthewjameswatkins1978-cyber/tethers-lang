(* ==================================================================
   CANONICAL FORMAT V2 — REFERENCE ENCODER AND SLOW ORACLE

   This module implements:
   1. The frozen Enc_V2 byte encoder (§6 of V2 spec)
   2. A deliberately slow complete Λ(P) oracle (§23 of V2 spec)

   The oracle is test-only and must NOT be used as production
   implementation. It has no WL pruning and is exponentially slower.

   Identity law:
     CanonicalPayload_V2(P) = min { Enc_V2(P, λ) | λ ∈ Λ(P) }
     CanonicalPreimage_V2(P) = DOMAIN_V2 || CanonicalPayload_V2(P)
     ProgramDigest_V2(P) = SHA-256(CanonicalPreimage_V2(P))
   ================================================================== *)

open Tethers_core

(* ================================================================== *)
(*  Domain separation                                                   *)
(* ================================================================== *)

let domain_v2 : bytes =
  Bytes.of_string "TETHERS_CORE_CANON_V2\x00"

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
(*                                                                     *)
(*  Different entity families may legally share raw string IDs:         *)
(*  FactId "X", OriginId "X", BranchId "X" are three unrelated         *)
(*  entities.  We use separate typed maps per family to prevent         *)
(*  cross-family collision.                                             *)
(* ================================================================== *)

module OriginMap = Map.Make (struct type t = origin_id let compare = compare end)
module FactMap = Map.Make (struct type t = fact_id let compare = compare end)
module BranchMap = Map.Make (struct type t = branch_id let compare = compare end)
module BatchMap = Map.Make (struct type t = batch_id let compare = compare end)
module TemplateMap = Map.Make (struct type t = item_template_id let compare = compare end)
module RoleMap = Map.Make (struct type t = role_id let compare = compare end)

(* A scoped role key includes the structural containing scope.
   Program_scope + role_id, or Item_template_scope(tid) + role_id.
   No cross-family or cross-scope key collision is possible. *)
type scoped_role_key =
  | Program_role of role_id
  | Template_role of item_template_id * role_id

module ScopedRoleMap = Map.Make (struct type t = scoped_role_key let compare = compare end)

(* ================================================================== *)
(*  Label assignment: one typed map per family                          *)
(* ================================================================== *)

type label_assignment = {
  origin_labels   : int OriginMap.t;
  fact_labels     : int FactMap.t;
  branch_labels   : int BranchMap.t;
  batch_labels    : int BatchMap.t;
  template_labels : int TemplateMap.t;
  role_labels     : int ScopedRoleMap.t;
}

(* Lookups that fail on missing labels — a missing canonical label after
   validation/full assignment is an INTERNAL ERROR. *)
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

(* Convenience: resolve a role_id within a specific structural scope *)
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
(*  Frozen Enc_V2 encoder (§6.4)                                       *)
(*  Every scoped role reference receives structural containing context.  *)
(* ================================================================== *)

let encode_scalar_type = function
  | String_type -> encode_tag 0
  | Integer_type -> encode_tag 1
  | Boolean_type -> encode_tag 2

let encode_value = function
  | String_value s -> encode_tag 0 ^ encode_string s
  | Integer_value i -> encode_tag 1 ^ encode_int i
  | Boolean_value b -> encode_tag 2 ^ (if b then "1;" else "0;")

(* encode_provenance: Role_proxy resolves using the fact's declaration scope *)
let encode_provenance (la : label_assignment) (fact_scope : role_scope) = function
  | Evaluation_input (Host_snapshot_key k, t) ->
      encode_tag 0 ^ encode_string k ^ encode_scalar_type t
  | Origin_provenance oid ->
      encode_tag 1 ^ encode_int (lookup_origin la oid)
  | Role_proxy rid ->
      encode_tag 2 ^ encode_int (lookup_role_in_scope la fact_scope rid)

(* encode_fact: fact_scope is the structural containing scope *)
let encode_fact (la : label_assignment) ~(fact_scope : role_scope) (f : fact) : string =
  encode_int (lookup_fact la f.fact_id) ^
  encode_provenance la fact_scope f.provenance

let encode_fact_guard (la : label_assignment) (g : fact_guard) : string =
  encode_int (lookup_fact la g.fact_id) ^
  encode_tag (operator_rank g.operator) ^
  encode_value g.expected

(* encode_binding: origin_scope is the structural scope of the containing origin *)
let encode_binding (la : label_assignment) ~(origin_scope : role_scope) = function
  | Literal_value v -> encode_tag 0 ^ encode_value v
  | Fact_from_origin (fid, oid) ->
      encode_tag 1 ^ encode_int (lookup_fact la fid) ^
      encode_int (lookup_origin la oid)
  | Fact_through_role (fid, rid) ->
      (* Resolve using containing origin's structural scope *)
      encode_tag 2 ^ encode_int (lookup_fact la fid) ^
      encode_int (lookup_role_in_scope la origin_scope rid)
  | Anchor_value (oid, path) ->
      encode_tag 3 ^ encode_int (lookup_origin la oid) ^
      encode_list (fun p -> encode_string p) path
  | Batch_item_context tid ->
      encode_tag 4 ^ encode_int (lookup_template la tid)

(* encode_action_input: origin_scope is the structural scope of the containing origin *)
let encode_action_input (la : label_assignment) ~(origin_scope : role_scope) (ai : action_input) : string =
  encode_string (string_of_capability_input_name ai.input_name) ^
  encode_binding la ~origin_scope ai.binding

let encode_constraint = function
  | Deadline s -> encode_tag 0 ^ encode_string s

(* encode_origin_site: origin_scope is the structural scope *)
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
        String.compare (encode_constraint x) (encode_constraint y)
      ) a.execution_constraints in
      encode_tag 1 ^
      encode_int (lookup_origin la a.action_origin_id) ^
      encode_string (string_of_capability_id a.capability_id) ^
      encode_string (string_of_capability_contract_digest a.contract_digest) ^
      encode_list (encode_action_input la ~origin_scope) sorted_inputs ^
      encode_list (encode_fact la ~fact_scope:origin_scope) sorted_facts ^
      encode_list encode_constraint sorted_constraints
  | Together_origin t ->
      (* member_origin_ids: sort by origin label *)
      let member_labels = List.map (fun oid ->
        lookup_origin la oid
      ) t.member_origin_ids |> List.sort Int.compare in
      encode_tag 2 ^
      encode_int (lookup_origin la t.together_origin_id) ^
      (* group_id EXCLUDED — neutral (§6.6) *)
      encode_list encode_int member_labels ^
      encode_tag 0  (* together_objective: All_members_succeed *)
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

(* encode_role: receives the role's physical/declarative scope *)
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

(* encode_item_objective: Required_role resolves in containing item-template scope *)
let encode_item_objective (la : label_assignment) ~(template_scope : item_template_id) = function
  | Required_role rid ->
      encode_tag 0 ^ encode_int (lookup_role_in_scope la (Item_template_scope template_scope) rid)

(* ================================================================== *)
(*  Frozen mixed-origin/Batch sort key (§9.3.1)                        *)
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
  (* schema_description EXCLUDED — neutral (§6.7) *)

(* ================================================================== *)
(*  Top-level encoder                                                   *)
(* ================================================================== *)

let encode_program (la : label_assignment) (p : program) : string =
  (* Sort input_facts by canonical fact label *)
  let sorted_input_facts = List.sort (fun (a : fact) (b : fact) ->
    Int.compare (lookup_fact la a.fact_id) (lookup_fact la b.fact_id)
  ) p.input_facts in

  (* Sort entry_guards by (fact_label, operator_rank, expected) *)
  let sorted_entry_guards = List.sort (fun (a : fact_guard) (b : fact_guard) ->
    let c = Int.compare (lookup_fact la a.fact_id) (lookup_fact la b.fact_id) in
    if c <> 0 then c
    else
      let c2 = Int.compare (operator_rank a.operator) (operator_rank b.operator) in
      if c2 <> 0 then c2
      else String.compare (encode_value a.expected) (encode_value b.expected)
  ) p.entry_guards in

  (* Sort success_continuations by from_origin label *)
  let sorted_success_continuations = List.sort (fun (a : success_continuation) (b : success_continuation) ->
    Int.compare (lookup_origin la a.from_origin) (lookup_origin la b.from_origin)
  ) p.success_continuations in

  (* Sort origin_sites by frozen mixed-site sort key (§9.3.1) *)
  let sorted_origin_sites = sort_origin_sites la p.origin_sites in

  (* Sort branches by canonical branch label *)
  let sorted_branches = List.sort (fun (a : branch) (b : branch) ->
    Int.compare (lookup_branch la a.branch_id) (lookup_branch la b.branch_id)
  ) p.branches in

  (* Sort roles by canonical scoped role label *)
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

  (* Sort item_templates by canonical template label *)
  let sorted_item_templates = List.sort (fun (a : item_template) (b : item_template) ->
    Int.compare (lookup_template la a.item_template_id) (lookup_template la b.item_template_id)
  ) p.item_templates in

  (* Sort capability_contracts by capability_id string (§6.4) *)
  let sorted_capability_contracts = List.sort (fun (a : capability_contract) (b : capability_contract) ->
    String.compare (string_of_capability_id a.capability_id) (string_of_capability_id b.capability_id)
  ) p.capability_contracts in

  (* Frozen top-level field order (§6.4) *)
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

(* ================================================================== *)
(*  SHA-256 digest (§16, §17)                                           *)
(* ================================================================== *)

let sha256_hex (data : bytes) : string =
  Digestif.SHA256.(to_hex (digest_bytes data))

let digest_string_v2 (hex : string) : string =
  "tethers:v2:sha256:" ^ hex

(* ================================================================== *)
(*  Slow complete Λ(P) oracle (§23)                                    *)
(* ================================================================== *)

type oracle_result = {
  payload : string;
  preimage : bytes;
  raw_digest : string;
  digest_string : string;
  candidate_count : int;
}

type oracle_error =
  | Invalid_core of Tethers_core_validator.validation_error list
  | Oracle_too_large

(* Oracle size limits (§23.2) *)
let oracle_total_entities_limit = 16
let oracle_max_family_size = 6
let oracle_max_total_permutations = 720

let count_total_entities (p : program) : int =
  List.length (collect_facts p) +
  List.length (collect_origins p) +
  List.length (collect_batches p) +
  List.length (collect_roles p) +
  List.length (collect_branches p) +
  List.length p.item_templates

(* Generate all permutations of a list *)
let rec perm lst =
  match lst with
  | [] -> [[]]
  | x :: xs ->
      let ps = perm xs in
      List.concat (List.map (fun p ->
        let rec insert_all acc i before = function
          | [] ->
              let after = List.filteri (fun j _ -> j >= i) p in
              let new_item = List.rev_append before (x :: after) in
              new_item :: acc
          | y :: ys ->
              let new_item = List.rev_append before (x :: y :: ys) in
              insert_all (new_item :: acc) (i + 1) (before @ [y]) ys
        in
        insert_all [] 0 [] p
      ) ps)

(* ================================================================== *)
(*  Enumerate all valid canonical labellings Λ(P)                      *)
(*                                                                     *)
(*  Frozen rule (§9.4):                                                 *)
(*  ONE global role range 1..N_roles.                                   *)
(*  Blocks: Program_scope first, then each template scope in            *)
(*  ascending canonical λ_template label order.                         *)
(*  Within each block: enumerate ALL bijections of that scope's         *)
(*  role occurrences onto exactly that block's contiguous interval.     *)
(*  NO cross-block assignment.                                          *)
(*  Candidate factor = product over scopes of factorial(n_roles_in_scope) *)
(* ================================================================== *)

let enumerate_lambda (p : program) : label_assignment list =
  let all_facts_list = collect_facts p in
  let all_origins_list = collect_origins p in
  let all_batches_list = collect_batches p in
  let all_roles_list = collect_roles p in
  let all_branches_list = collect_branches p in
  let all_templates_list = p.item_templates in

  let total = count_total_entities p in
  if total > oracle_total_entities_limit then [] else

  let n_facts = List.length all_facts_list in
  let n_origins = List.length all_origins_list in
  let n_batches = List.length all_batches_list in
  let n_roles = List.length all_roles_list in
  let n_branches = List.length all_branches_list in
  let n_templates = List.length all_templates_list in

  if n_facts > oracle_max_family_size ||
     n_origins > oracle_max_family_size ||
     n_batches > oracle_max_family_size ||
     n_roles > oracle_max_family_size ||
     n_branches > oracle_max_family_size ||
     n_templates > oracle_max_family_size then [] else

  (* Generate all permutations for non-role families *)
  let fact_indices = List.init n_facts (fun i -> i + 1) in
  let origin_indices = List.init n_origins (fun i -> i + 1) in
  let batch_indices = List.init n_batches (fun i -> i + 1) in
  let branch_indices = List.init n_branches (fun i -> i + 1) in
  let template_indices = List.init n_templates (fun i -> i + 1) in

  let fact_perms_list = perm fact_indices in
  let origin_perms_list = perm origin_indices in
  let batch_perms_list = perm batch_indices in
  let branch_perms_list = perm branch_indices in
  let template_perms_list = perm template_indices in

  (* Pre-group roles by scope *)
  let program_roles_list = List.filter_map (fun (r, scope) ->
    match scope with `Program -> Some r | _ -> None
  ) all_roles_list in
  let n_program_roles = List.length program_roles_list in

  (* For template roles, group by template_id *)
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

  (* Check total role permutations: product of factorials per scope *)
  let factorial n =
    let rec fact acc = function 0 -> acc | m -> fact (acc * m) (m - 1) in
    fact 1 n
  in
  let role_total_perms =
    factorial n_program_roles *
    List.fold_left (fun acc (_, roles) -> acc * factorial (List.length roles)) 1 template_roles_groups
  in

  (* Check total permutations across all families *)
  let fact_perms_count = List.length fact_perms_list in
  let origin_perms_count = List.length origin_perms_list in
  let batch_perms_count = List.length batch_perms_list in
  let branch_perms_count = List.length branch_perms_list in
  let template_perms_count = List.length template_perms_list in
  let total_perms = fact_perms_count * origin_perms_count * batch_perms_count *
                    role_total_perms * branch_perms_count * template_perms_count in
  if total_perms > oracle_max_total_permutations then [] else

  (* For each combination of family permutations, build a label_assignment *)
  List.concat (List.map (fun (fact_perm : int list) ->
    let fact_map = List.fold_left2 (fun (m : int FactMap.t) (f : fact) (lbl : int) ->
      FactMap.add f.fact_id lbl m
    ) FactMap.empty all_facts_list fact_perm in

    List.concat (List.map (fun (origin_perm : int list) ->
      let origin_map = List.fold_left2 (fun (m : int OriginMap.t) ((oid, _) : origin_id * origin_site) (lbl : int) ->
        OriginMap.add oid lbl m
      ) OriginMap.empty all_origins_list origin_perm in

      List.concat (List.map (fun (batch_perm : int list) ->
        let batch_map = List.fold_left2 (fun (m : int BatchMap.t) ((bid, _) : batch_id * batch_site) (lbl : int) ->
          BatchMap.add bid lbl m
        ) BatchMap.empty all_batches_list batch_perm in

        List.concat (List.map (fun (branch_perm : int list) ->
          let branch_map = List.fold_left2 (fun (m : int BranchMap.t) ((b, _) : branch * _) (lbl : int) ->
            BranchMap.add b.branch_id lbl m
          ) BranchMap.empty all_branches_list branch_perm in

          List.concat (List.map (fun (template_perm : int list) ->
            let template_map = List.fold_left2 (fun (m : int TemplateMap.t) (t : item_template) (lbl : int) ->
              TemplateMap.add t.item_template_id lbl m
            ) TemplateMap.empty all_templates_list template_perm in

            (* Compute template labelling for this candidate *)
            let template_labels = List.map2 (fun t lbl ->
              (t.item_template_id, lbl)
            ) all_templates_list template_perm in

            (* Sort template_ids by their canonical label for scope ordering *)
            let sorted_template_ids = List.sort (fun a b ->
              Int.compare (List.assoc a template_labels) (List.assoc b template_labels)
            ) (List.map fst template_labels) in

            (* Role-block enumeration (§9.4):
               1. Allocate role-label intervals:
                  Program_scope: 1..n_program_roles
                  Template scopes: in ascending λ_template order
               2. For each scope, independently enumerate all permutations
                  of that scope's roles onto its contiguous interval.
               3. Cross-product of scope-local permutations. *)

            (* Compute interval starts for each scope *)
            let program_start = 1 in
            let program_end = n_program_roles in
            let template_intervals = List.fold_left (fun (acc, next_start) tid ->
              let roles = try List.assoc tid template_roles_groups with Not_found -> [] in
              let n = List.length roles in
              if n = 0 then (acc, next_start) else
              let interval = (tid, next_start, next_start + n - 1) in
              ((interval :: acc), next_start + n)
            ) ([], program_end + 1) sorted_template_ids in
            let template_intervals = List.rev (fst template_intervals) in

            (* Generate scope-local permutations *)
            let program_role_perms = perm (List.init n_program_roles (fun i -> i)) in

            let template_role_perms_per_scope = List.map (fun (tid, _start, _end) ->
              let roles = try List.assoc tid template_roles_groups with Not_found -> [] in
              let n = List.length roles in
              (tid, perm (List.init n (fun i -> i)))
            ) template_intervals in

            (* Cross-product of all scope-local permutations *)
            let rec cross_product = function
              | [] -> [[]]
              | xs :: rest ->
                  let rest_perms = cross_product rest in
                  List.concat (List.map (fun x ->
                    List.map (fun r -> x :: r) rest_perms
                  ) xs)
            in

            let all_scope_perms = cross_product (
              program_role_perms ::
              List.map snd template_role_perms_per_scope
            ) in

            List.concat (List.map (fun scope_perms ->
              match scope_perms with
              | [] -> []
              | program_perm :: template_role_perms ->
                (* Build program role map *)
                let program_role_map = List.fold_left2 (fun m r idx ->
                  let label = program_start + idx in
                  ScopedRoleMap.add (Program_role r.role_id) label m
                ) ScopedRoleMap.empty program_roles_list program_perm in

                (* Build template role maps *)
                let template_role_map = List.fold_left2 (fun m (tid, _start, _end) perm ->
                  let roles = try List.assoc tid template_roles_groups with Not_found -> [] in
                  let (_, start, _) = List.find (fun (t, _, _) -> t = tid) template_intervals in
                  List.fold_left2 (fun m2 r idx ->
                    let label = start + idx in
                    ScopedRoleMap.add (Template_role (tid, r.role_id)) label m2
                  ) m roles perm
                ) ScopedRoleMap.empty template_intervals template_role_perms in

                let role_map = ScopedRoleMap.union (fun _ a _ -> Some a) program_role_map template_role_map in

                [{ origin_labels = origin_map;
                   fact_labels = fact_map;
                   branch_labels = branch_map;
                   batch_labels = batch_map;
                   template_labels = template_map;
                   role_labels = role_map;
                 }]
            ) all_scope_perms)
          ) template_perms_list)
        ) branch_perms_list)
      ) batch_perms_list)
    ) origin_perms_list)
  ) fact_perms_list)

let slow_oracle (p : program) : (oracle_result, oracle_error) result =
  match Tethers_core_validator.validate p with
  | Error errs -> Error (Invalid_core errs)
  | Ok () ->
      let total = count_total_entities p in
      if total > oracle_total_entities_limit then
        Error Oracle_too_large
      else begin
        let labellings = enumerate_lambda p in
        if labellings = [] then
          Error Oracle_too_large
        else begin
          let candidate_count = List.length labellings in

          (* Encode each candidate *)
          let candidates = List.map (fun la ->
            encode_program la p
          ) labellings in

          (* Find the lexicographic minimum *)
          let payload = List.fold_left (fun best candidate ->
            if best = "" || String.compare candidate best < 0 then
              candidate
            else
              best
          ) "" candidates in

          (* Construct preimage *)
          let payload_bytes = Bytes.of_string payload in
          let preimage = Bytes.concat Bytes.empty [domain_v2; payload_bytes] in

          (* Compute digest *)
          let raw_digest = sha256_hex preimage in
          let digest_string = digest_string_v2 raw_digest in

          Ok {
            payload;
            preimage;
            raw_digest;
            digest_string;
            candidate_count;
          }
        end
      end

(* ================================================================== *)
(*  Convenience: run oracle and return just the digest                  *)
(* ================================================================== *)

let compute_digest (p : program) : (string * string, oracle_error) result =
  match slow_oracle p with
  | Error e -> Error e
  | Ok result -> Ok (result.digest_string, result.payload)
