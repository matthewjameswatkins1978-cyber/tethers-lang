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
  (* ASCII "TETHERS_CORE_CANON_V2" followed by 0x00 *)
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
(*  Label maps                                                          *)
(* ================================================================== *)

module StringMap = Map.Make (String)

type label_map = int StringMap.t

let lookup_label (lbl_map : label_map) (key : string) : int =
  match StringMap.find_opt key lbl_map with
  | Some l -> l
  | None -> 0

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
(*  Build fact_scope_map for Role_proxy validation (§10.2.1)           *)
(* ================================================================== *)

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

let encode_provenance (lbl_map : label_map) = function
  | Evaluation_input (Host_snapshot_key k, t) ->
      encode_tag 0 ^ encode_string k ^ encode_scalar_type t
  | Origin_provenance oid ->
      encode_tag 1 ^ encode_int (lookup_label lbl_map (string_of_origin_id oid))
  | Role_proxy rid ->
      encode_tag 2 ^ encode_int (lookup_label lbl_map (string_of_role_id rid))

let encode_fact (lbl_map : label_map) (f : fact) : string =
  encode_int (lookup_label lbl_map (string_of_fact_id f.fact_id)) ^
  encode_provenance lbl_map f.provenance

let encode_fact_guard (lbl_map : label_map) (g : fact_guard) : string =
  encode_int (lookup_label lbl_map (string_of_fact_id g.fact_id)) ^
  encode_tag (operator_rank g.operator) ^
  encode_value g.expected

let encode_binding (lbl_map : label_map) = function
  | Literal_value v -> encode_tag 0 ^ encode_value v
  | Fact_from_origin (fid, oid) ->
      encode_tag 1 ^ encode_int (lookup_label lbl_map (string_of_fact_id fid)) ^
      encode_int (lookup_label lbl_map (string_of_origin_id oid))
  | Fact_through_role (fid, rid) ->
      encode_tag 2 ^ encode_int (lookup_label lbl_map (string_of_fact_id fid)) ^
      encode_int (lookup_label lbl_map (string_of_role_id rid))
  | Anchor_value (oid, path) ->
      encode_tag 3 ^ encode_int (lookup_label lbl_map (string_of_origin_id oid)) ^
      encode_list (fun p -> encode_string p) path
  | Batch_item_context (Item_template_id tid) ->
      encode_tag 4 ^ encode_int (lookup_label lbl_map tid)

let encode_action_input (lbl_map : label_map) (ai : action_input) : string =
  encode_string (string_of_capability_input_name ai.input_name) ^
  encode_binding lbl_map ai.binding

let encode_constraint = function
  | Deadline s -> encode_tag 0 ^ encode_string s

let encode_origin_site (lbl_map : label_map) (site : origin_site) : string =
  match site with
  | Anchor_origin a ->
      encode_tag 0 ^
      encode_int (lookup_label lbl_map (string_of_origin_id a.anchor_origin_id)) ^
      encode_string a.event_name ^
      encode_list (encode_fact lbl_map) a.declared_facts
  | Action_origin a ->
      encode_tag 1 ^
      encode_int (lookup_label lbl_map (string_of_origin_id a.action_origin_id)) ^
      encode_string (string_of_capability_id a.capability_id) ^
      encode_string (string_of_capability_contract_digest a.contract_digest) ^
      encode_list (encode_action_input lbl_map) a.inputs ^
      encode_list (encode_fact lbl_map) a.declared_facts ^
      encode_list encode_constraint a.execution_constraints
  | Together_origin t ->
      let member_labels = List.map (fun oid ->
        lookup_label lbl_map (string_of_origin_id oid)
      ) t.member_origin_ids |> List.sort Int.compare in
      encode_tag 2 ^
      encode_int (lookup_label lbl_map (string_of_origin_id t.together_origin_id)) ^
      (* group_id EXCLUDED — neutral (§6.6) *)
      encode_list encode_int member_labels ^
      encode_tag 0  (* together_objective: All_members_succeed *)
  | Batch_site b ->
      encode_tag 3 ^
      encode_int (lookup_label lbl_map (string_of_batch_id b.batch_id)) ^
      encode_string (string_of_batch_collection_provenance b.collection_provenance) ^
      encode_int (lookup_label lbl_map (string_of_item_template_id b.item_template_id)) ^
      encode_string (string_of_batch_traversal_policy b.traversal_policy) ^
      encode_string (string_of_batch_objective b.composite_objective) ^
      encode_list (encode_fact lbl_map) b.aggregate_facts

let encode_branch (lbl_map : label_map) (b : branch) : string =
  encode_int (lookup_label lbl_map (string_of_branch_id b.branch_id)) ^
  encode_int (lookup_label lbl_map (string_of_origin_id b.branch_subject)) ^
  encode_list (fun (outcome, target) ->
    encode_tag (outcome_rank outcome) ^
    (match target with
     | Continue_to oid -> encode_tag 0 ^ encode_int (lookup_label lbl_map (string_of_origin_id oid))
     | Stop -> encode_tag 1)
  ) b.outcome_branches

let encode_role_scope (lbl_map : label_map) = function
  | Program_scope -> encode_tag 0
  | Item_template_scope (Item_template_id tid) ->
      encode_tag 1 ^ encode_int (lookup_label lbl_map tid)

let encode_role (lbl_map : label_map) (r : role) : string =
  let (Role_fact_contract fids) = r.fact_contract in
  encode_int (lookup_label lbl_map (string_of_role_id r.role_id)) ^
  encode_role_scope lbl_map r.scope ^
  encode_list (fun (Fact_id fid) -> encode_int (lookup_label lbl_map fid)) fids ^
  encode_string (string_of_role_fulfillment r.eligible_fulfillment)

let encode_item_objective (lbl_map : label_map) = function
  | Required_role rid ->
      encode_tag 0 ^ encode_int (lookup_label lbl_map (string_of_role_id rid))

let encode_item_template (lbl_map : label_map) (t : item_template) : string =
  encode_int (lookup_label lbl_map (string_of_item_template_id t.item_template_id)) ^
  encode_list (encode_origin_site lbl_map) t.origin_sites ^
  encode_list (encode_branch lbl_map) t.branches ^
  encode_list (encode_role lbl_map) t.roles ^
  encode_item_objective lbl_map t.objective

let encode_capability_contract (_lbl_map : label_map) (c : capability_contract) : string =
  encode_string (string_of_capability_id c.capability_id) ^
  encode_string (string_of_capability_contract_digest c.contract_digest)
  (* schema_description EXCLUDED — neutral (§6.7) *)

(* ================================================================== *)
(*  Frozen mixed-origin/Batch sort key (§9.3.1)                        *)
(* ================================================================== *)

type sort_key = int * int

let origin_sort_key (lbl_map : label_map) (site : origin_site) : sort_key =
  match site with
  | Anchor_origin a ->
      (0, lookup_label lbl_map (string_of_origin_id a.anchor_origin_id))
  | Action_origin a ->
      (0, lookup_label lbl_map (string_of_origin_id a.action_origin_id))
  | Together_origin t ->
      (0, lookup_label lbl_map (string_of_origin_id t.together_origin_id))
  | Batch_site b ->
      (1, lookup_label lbl_map (string_of_batch_id b.batch_id))

let sort_origin_sites (lbl_map : label_map) (sites : origin_site list) : origin_site list =
  List.sort (fun a b ->
    let ka = origin_sort_key lbl_map a in
    let kb = origin_sort_key lbl_map b in
    compare ka kb
  ) sites

(* ================================================================== *)
(*  Top-level encoder                                                   *)
(* ================================================================== *)

let encode_program (lbl_map : label_map) (p : program) : string =
  (* Sort input_facts by canonical fact label *)
  let sorted_input_facts = List.sort (fun (a : fact) (b : fact) ->
    Int.compare
      (lookup_label lbl_map (string_of_fact_id a.fact_id))
      (lookup_label lbl_map (string_of_fact_id b.fact_id))
  ) p.input_facts in

  (* Sort entry_guards by (fact_label, operator_rank, expected) *)
  let sorted_entry_guards = List.sort (fun (a : fact_guard) (b : fact_guard) ->
    let c = Int.compare
      (lookup_label lbl_map (string_of_fact_id a.fact_id))
      (lookup_label lbl_map (string_of_fact_id b.fact_id))
    in
    if c <> 0 then c
    else
      let c2 = Int.compare (operator_rank a.operator) (operator_rank b.operator) in
      if c2 <> 0 then c2
      else String.compare (encode_value a.expected) (encode_value b.expected)
  ) p.entry_guards in

  (* Sort success_continuations by from_origin label *)
  let sorted_success_continuations = List.sort (fun (a : success_continuation) (b : success_continuation) ->
    Int.compare
      (lookup_label lbl_map (string_of_origin_id a.from_origin))
      (lookup_label lbl_map (string_of_origin_id b.from_origin))
  ) p.success_continuations in

  (* Sort origin_sites by frozen mixed-site sort key (§9.3.1) *)
  let sorted_origin_sites = sort_origin_sites lbl_map p.origin_sites in

  (* Sort branches by canonical branch label *)
  let sorted_branches = List.sort (fun (a : branch) (b : branch) ->
    Int.compare
      (lookup_label lbl_map (string_of_branch_id a.branch_id))
      (lookup_label lbl_map (string_of_branch_id b.branch_id))
  ) p.branches in

  (* Sort roles by canonical scoped role label *)
  let sorted_roles = List.sort (fun (a : role) (b : role) ->
    Int.compare
      (lookup_label lbl_map (string_of_role_id a.role_id))
      (lookup_label lbl_map (string_of_role_id b.role_id))
  ) p.roles in

  (* Sort item_templates by canonical template label *)
  let sorted_item_templates = List.sort (fun (a : item_template) (b : item_template) ->
    Int.compare
      (lookup_label lbl_map (string_of_item_template_id a.item_template_id))
      (lookup_label lbl_map (string_of_item_template_id b.item_template_id))
  ) p.item_templates in

  (* Sort capability_contracts by capability_id string (§6.4) *)
  let sorted_capability_contracts = List.sort (fun (a : capability_contract) (b : capability_contract) ->
    String.compare (string_of_capability_id a.capability_id) (string_of_capability_id b.capability_id)
  ) p.capability_contracts in

  (* Frozen top-level field order (§6.4) *)
  encode_string (string_of_core_version p.core_version) ^
  encode_list (encode_fact lbl_map) sorted_input_facts ^
  encode_list (encode_fact_guard lbl_map) sorted_entry_guards ^
  encode_option (fun oid -> encode_int (lookup_label lbl_map (string_of_origin_id oid))) p.entry_origin ^
  encode_list (fun (sc : success_continuation) ->
    encode_int (lookup_label lbl_map (string_of_origin_id sc.from_origin)) ^
    (match sc.target with
     | Origin_target oid -> encode_tag 0 ^ encode_int (lookup_label lbl_map (string_of_origin_id oid))
     | Program_complete -> encode_tag 1)
  ) sorted_success_continuations ^
  encode_list (encode_origin_site lbl_map) sorted_origin_sites ^
  encode_list (encode_branch lbl_map) sorted_branches ^
  encode_list (encode_role lbl_map) sorted_roles ^
  encode_list (encode_item_template lbl_map) sorted_item_templates ^
  encode_list (encode_capability_contract lbl_map) sorted_capability_contracts

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
        let rec insert acc before = function
          | [] -> List.rev_append (List.rev (x :: before)) [] :: acc
          | y :: ys ->
              let new_item = List.rev_append (List.rev before) (x :: y :: ys) in
              insert (new_item :: acc) (before @ [y]) ys
        in
        insert [] [] p
      ) ps)

(* Enumerate all valid canonical labellings Λ(P) *)
let enumerate_lambda (p : program) : label_map list =
  let all_facts_list = collect_facts p in
  let all_origins_list = collect_origins p in
  let all_batches_list = collect_batches p in
  let all_roles_list = collect_roles p in
  let all_branches_list = collect_branches p in
  let all_templates_list = p.item_templates in

  (* Check size limits *)
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

  (* Generate all permutations for each family *)
  let fact_indices = List.init n_facts (fun i -> i + 1) in
  let origin_indices = List.init n_origins (fun i -> i + 1) in
  let batch_indices = List.init n_batches (fun i -> i + 1) in
  let role_indices = List.init n_roles (fun i -> i + 1) in
  let branch_indices = List.init n_branches (fun i -> i + 1) in
  let template_indices = List.init n_templates (fun i -> i + 1) in

  (* Check total permutations *)
  let fact_perms_count = List.length (perm fact_indices) in
  let origin_perms_count = List.length (perm origin_indices) in
  let batch_perms_count = List.length (perm batch_indices) in
  let role_perms_count = List.length (perm role_indices) in
  let branch_perms_count = List.length (perm branch_indices) in
  let template_perms_count = List.length (perm template_indices) in

  let total_perms = fact_perms_count * origin_perms_count * batch_perms_count * role_perms_count * branch_perms_count * template_perms_count in
  if total_perms > oracle_max_total_permutations then [] else

  (* Generate all labellings *)
  let fact_perms_list = perm fact_indices in
  let origin_perms_list = perm origin_indices in
  let batch_perms_list = perm batch_indices in
  let role_perms_list = perm role_indices in
  let branch_perms_list = perm branch_indices in
  let template_perms_list = perm template_indices in

  (* For each combination of family permutations, build a label map *)
  List.concat (List.map (fun (fact_perm : int list) ->
    let fact_map = List.fold_left2 (fun (m : label_map) (f : fact) (lbl : int) ->
      StringMap.add (string_of_fact_id f.fact_id) lbl m
    ) StringMap.empty all_facts_list fact_perm in

    List.concat (List.map (fun (origin_perm : int list) ->
      let origin_map = List.fold_left2 (fun (m : label_map) ((oid, _) : origin_id * origin_site) (lbl : int) ->
        StringMap.add (string_of_origin_id oid) lbl m
      ) StringMap.empty all_origins_list origin_perm in

      List.concat (List.map (fun (batch_perm : int list) ->
        let batch_map = List.fold_left2 (fun (m : label_map) ((bid, _) : batch_id * batch_site) (lbl : int) ->
          StringMap.add (string_of_batch_id bid) lbl m
        ) StringMap.empty all_batches_list batch_perm in

        List.concat (List.map (fun (branch_perm : int list) ->
          let branch_map = List.fold_left2 (fun (m : label_map) ((b, _) : branch * [`Program | `Template of item_template_id]) (lbl : int) ->
            StringMap.add (string_of_branch_id b.branch_id) lbl m
          ) StringMap.empty all_branches_list branch_perm in

          List.concat (List.map (fun (template_perm : int list) ->
            let template_map = List.fold_left2 (fun (m : label_map) (t : item_template) (lbl : int) ->
              StringMap.add (string_of_item_template_id t.item_template_id) lbl m
            ) StringMap.empty all_templates_list template_perm in

            (* For role permutations, we need to respect scope blocks.
               For now, enumerate all role permutations and filter by scope. *)
            List.concat (List.map (fun (role_perm : int list) ->
              (* Compute scope blocks based on template labelling *)
              let template_labels = List.map2 (fun t lbl ->
                (t.item_template_id, lbl)
              ) all_templates_list template_perm in

              (* Group roles by scope *)
              let program_roles_list = List.filter_map (fun (r, scope) ->
                match scope with `Program -> Some r | _ -> None
              ) all_roles_list in
              let template_roles_by_tid : (string, role list) Hashtbl.t = Hashtbl.create 16 in
              List.iter (fun (r, scope) ->
                match scope with
                | `Template tid ->
                    let key = string_of_item_template_id tid in
                    let existing = try Hashtbl.find template_roles_by_tid key with Not_found -> [] in
                    Hashtbl.replace template_roles_by_tid key (r :: existing)
                | `Program -> ()
              ) all_roles_list;

              let n_program_roles = List.length program_roles_list in

              (* Assign program scope labels: 1..n_program_roles *)
              let program_role_map = List.fold_left2 (fun m r lbl ->
                StringMap.add (string_of_role_id r.role_id) lbl m
              ) StringMap.empty program_roles_list (List.filteri (fun i _ -> i < n_program_roles) role_perm) in

              (* Assign template scope labels *)
              let sorted_template_ids = List.sort (fun a b ->
                Int.compare (List.assoc a template_labels) (List.assoc b template_labels)
              ) (List.map fst template_labels) in

              let template_role_offset = ref (n_program_roles + 1) in
              let template_role_map = List.fold_left (fun m tid ->
                let template_roles = try Hashtbl.find template_roles_by_tid (string_of_item_template_id tid) with Not_found -> [] in
                let n = List.length template_roles in
                let start_lbl = !template_role_offset in
                template_role_offset := start_lbl + n;
                List.fold_left2 (fun m2 r lbl ->
                  StringMap.add (string_of_role_id r.role_id) lbl m2
                ) m template_roles (List.init n (fun i -> start_lbl + i))
              ) StringMap.empty sorted_template_ids in

              let role_map = StringMap.merge (fun _key a b ->
                match a, b with
                | Some x, _ -> Some x
                | _, Some x -> Some x
                | None, None -> None
              ) program_role_map template_role_map in

              (* Merge all maps *)
              let merged = StringMap.empty in
              let merged = StringMap.fold StringMap.add fact_map merged in
              let merged = StringMap.fold StringMap.add origin_map merged in
              let merged = StringMap.fold StringMap.add batch_map merged in
              let merged = StringMap.fold StringMap.add branch_map merged in
              let merged = StringMap.fold StringMap.add template_map merged in
              let merged = StringMap.fold StringMap.add role_map merged in

              [merged]
            ) role_perms_list)
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
          let candidates = List.map (fun lbl_map ->
            encode_program lbl_map p
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
