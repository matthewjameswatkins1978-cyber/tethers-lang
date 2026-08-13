(* ==================================================================
   CANONICAL FORMAT V2 — REFERENCE ENCODER AND SLOW ORACLE

   This module implements:
   1. The frozen Enc_V2 byte encoder (§6 of V2 spec) — via shared format
   2. A deliberately slow complete Λ(P) oracle (§23 of V2 spec)

   The oracle is test-only and must NOT be used as production
   implementation.  It has no WL pruning and is exponentially slower.

   Identity law:
     CanonicalPayload_V2(P) = min { Enc_V2(P, λ) | λ ∈ Λ(P) }
     CanonicalPreimage_V2(P) = DOMAIN_V2 || CanonicalPayload_V2(P)
     ProgramDigest_V2(P) = SHA-256(CanonicalPreimage_V2(P))
   ================================================================== *)

open Tethers_core
open Tethers_core_canonical_v2_format

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

(* Generate all permutations of a list (test-only helper) *)
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

  let factorial n =
    let rec fact acc = function 0 -> acc | m -> fact (acc * m) (m - 1) in
    fact 1 n
  in
  let role_total_perms =
    factorial n_program_roles *
    List.fold_left (fun acc (_, roles) -> acc * factorial (List.length roles)) 1 template_roles_groups
  in

  let fact_perms_count = List.length fact_perms_list in
  let origin_perms_count = List.length origin_perms_list in
  let batch_perms_count = List.length batch_perms_list in
  let branch_perms_count = List.length branch_perms_list in
  let template_perms_count = List.length template_perms_list in
  let total_perms = fact_perms_count * origin_perms_count * batch_perms_count *
                    role_total_perms * branch_perms_count * template_perms_count in
  if total_perms > oracle_max_total_permutations then [] else

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

            let template_labels = List.map2 (fun t lbl ->
              (t.item_template_id, lbl)
            ) all_templates_list template_perm in

            let sorted_template_ids = List.sort (fun a b ->
              Int.compare (List.assoc a template_labels) (List.assoc b template_labels)
            ) (List.map fst template_labels) in

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

            let program_role_perms = perm (List.init n_program_roles (fun i -> i)) in

            let template_role_perms_per_scope = List.map (fun (tid, _start, _end) ->
              let roles = try List.assoc tid template_roles_groups with Not_found -> [] in
              let n = List.length roles in
              (tid, perm (List.init n (fun i -> i)))
            ) template_intervals in

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
                let program_role_map = List.fold_left2 (fun m r idx ->
                  let label = program_start + idx in
                  ScopedRoleMap.add (Program_role r.role_id) label m
                ) ScopedRoleMap.empty program_roles_list program_perm in

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

          let candidates = List.map (fun la ->
            encode_program la p
          ) labellings in

          let payload = List.fold_left (fun best candidate ->
            if best = "" || String.compare candidate best < 0 then
              candidate
            else
              best
          ) "" candidates in

          let payload_bytes = Bytes.of_string payload in
          let preimage = Bytes.concat Bytes.empty [domain_v2; payload_bytes] in

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
