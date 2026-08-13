(* ==================================================================
   CANONICAL FORMAT V2 — PRODUCTION CANONICALISER

   This module implements the first non-oracle production
   implementation of Canonical Format V2.  It searches the complete
   Λ(P) using streaming permutation traversal (not materialising
   factorial candidate lists) and enforces a deterministic work
   budget.

   Identity law:
     CanonicalPayload_V2(P) = min { Enc_V2(P, λ) | λ ∈ Λ(P) }
     CanonicalPreimage_V2(P) = DOMAIN_V2 || CanonicalPayload_V2(P)
     ProgramDigest_V2(P) = SHA-256(CanonicalPreimage_V2(P))
   ================================================================== *)

open Tethers_core
open Tethers_core_canonical_v2_format

(* ================================================================== *)
(*  Types                                                               *)
(* ================================================================== *)

type canonicalized_v2 = {
  canonical_payload : string;
  canonical_preimage : bytes;
  canonical_program_digest : string;
}

type canonicalization_error =
  | Invalid_core of Tethers_core_validator.validation_error list
  | Canonicalisation_too_complex

type search_budget = {
  max_candidates : int;
}

let default_budget = { max_candidates = 5_000_000 }

(* ================================================================== *)
(*  Overflow-safe candidate counting                                    *)
(*                                                                     *)
(*  All arithmetic is relative to the caller's budget limit.            *)
(*  Multiplication aborts immediately if product would exceed limit,    *)
(*  without computing the product first (safe when limit = max_int).    *)
(* ================================================================== *)

(* Overflow-safe multiply: returns Some product iff product <= limit *)
let safe_mul a b limit =
  if a = 0 || b = 0 then Some 0
  else if a > limit / b then None
  else Some (a * b)

(* Overflow-safe factorial: returns Some n! iff n! <= limit *)
let safe_fact n limit =
  let rec go i acc =
    if i > n then Some acc
    else match safe_mul acc i limit with
    | None -> None
    | Some acc' -> go (i + 1) acc'
  in
  if n <= 0 then Some 1 else go 2 1

(* Compute exact Λ(P) candidate count relative to a budget limit.
   Returns Some exact_count iff count <= limit, None otherwise. *)
let candidate_count_within_budget ~limit (p : program) : int option =
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

  (* Factorial per family *)
  let* fact_perms = safe_fact n_facts limit in
  let* origin_perms = safe_fact n_origins limit in
  let* batch_perms = safe_fact n_batches limit in
  let* branch_perms = safe_fact n_branches limit in
  let* template_perms = safe_fact n_templates limit in

  (* Role permutations: product of factorials per scope *)
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

  (* Total = fact_perms * origin_perms * ... * role_perms *)
  let* total = safe_mul fact_perms origin_perms limit in
  let* total = safe_mul total batch_perms limit in
  let* total = safe_mul total branch_perms limit in
  let* total = safe_mul total template_perms limit in
  safe_mul total role_perms limit

(* ================================================================== *)
(*  Streaming permutation traversal (backtracking)                     *)
(*                                                                     *)
(*  We do NOT materialise permutation lists.  Instead we use           *)
(*  recursive backtracking to assign labels one entity at a time.      *)
(*  For each family, we assign labels 1..N to N entities in all        *)
(*  possible orders (permutations).                                    *)
(* ================================================================== *)

(* A permutation state tracks which labels have been assigned *)
type perm_state = {
  assigned : int array;
  used : bool array;
  mutable next_pos : int;
  count : int;
}

let make_perm_state n = {
  assigned = Array.make n 0;
  used = Array.make (n + 1) false;
  next_pos = 0;
  count = n;
}

(* Assign next entity in permutation, calling callback for each complete assignment *)
let rec assign_next st callback =
  if st.next_pos >= st.count then begin
    callback st.assigned
  end else begin
    let pos = st.next_pos in
    for label = 1 to st.count do
      if not st.used.(label) then begin
        st.assigned.(pos) <- label;
        st.used.(label) <- true;
        st.next_pos <- pos + 1;
        assign_next st callback;
        st.used.(label) <- false
      end
    done;
    st.next_pos <- pos
  end

(* ================================================================== *)
(*  Role block permutation traversal                                    *)
(* ================================================================== *)

(* A role block scope and its assigned permutation *)
type role_block = {
  n_roles : int;
  start_label : int;
  mutable perm : int array;
}

(* State for role block enumeration *)
type role_block_state = {
  blocks : role_block array;
  mutable block_index : int;
  block_perms : perm_state array;
}

let make_role_block_state blocks =
  let block_perms = Array.map (fun b ->
    make_perm_state b.n_roles
  ) blocks in
  { blocks; block_index = 0; block_perms }

(* Enumerate all cross-block role permutations *)
let rec enumerate_role_blocks st callback =
  if st.block_index >= Array.length st.blocks then begin
    callback st.blocks
  end else begin
    let idx = st.block_index in
    st.block_index <- idx + 1;
    let bp = st.block_perms.(idx) in
    assign_next bp (fun perm ->
      st.blocks.(idx).perm <- perm;
      enumerate_role_blocks st callback
    );
    st.block_index <- idx
  end

(* ================================================================== *)
(*  Complete Λ(P) streaming search                                      *)
(* ================================================================== *)

let canonicalize ?(budget = default_budget) (p : program) :
    (canonicalized_v2, canonicalization_error) result =
  (* Validate first *)
  match Tethers_core_validator.validate p with
  | Error errs -> Error (Invalid_core errs)
  | Ok () -> begin
    (* Pre-admission: compute candidate space relative to budget *)
    match candidate_count_within_budget ~limit:budget.max_candidates p with
    | None -> Error Canonicalisation_too_complex
    | Some _ -> begin
      (* Collect all entity families *)
      let all_facts_list = collect_facts p in
      let all_origins_list = collect_origins p in
      let all_batches_list = collect_batches p in
      let all_roles_list = collect_roles p in
      let all_branches_list = collect_branches p in
      let all_templates_list = p.item_templates in

      let n_facts = List.length all_facts_list in
      let n_origins = List.length all_origins_list in
      let n_batches = List.length all_batches_list in
      let n_branches = List.length all_branches_list in
      let n_templates = List.length all_templates_list in

      (* Create permutation states for non-role families *)
      let fact_state = make_perm_state n_facts in
      let origin_state = make_perm_state n_origins in
      let batch_state = make_perm_state n_batches in
      let branch_state = make_perm_state n_branches in
      let template_state = make_perm_state n_templates in

      (* Pre-group roles by scope *)
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

      (* Mutable state for current best *)
      let best_payload = ref "" in
      let best_preimage = ref Bytes.empty in
      let best_digest = ref "" in
      let candidate_counter = ref 0 in
      let budget_exceeded = ref false in

      (* Encode a complete label assignment and update best if smaller *)
      let process_assignment (la : label_assignment) =
        incr candidate_counter;
        if !candidate_counter > budget.max_candidates then
          budget_exceeded := true
        else begin
          let payload = encode_program la p in
          if !best_payload = "" ||
             compare_bytes_lex_unsigned payload !best_payload < 0 then begin
            best_payload := payload;
            let payload_bytes = Bytes.of_string payload in
            best_preimage := Bytes.concat Bytes.empty [domain_v2; payload_bytes];
            best_digest := digest_string_v2 (sha256_hex !best_preimage)
          end
        end
      in

      (* Build label assignment from current permutation states *)
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

      (* Compute template ordering by canonical label for role-block intervals *)
      let compute_template_label_order (la : label_assignment) =
        let template_labels = List.map (fun t ->
          (t.item_template_id, lookup_template la t.item_template_id)
        ) all_templates_list in
        List.sort (fun a b -> Int.compare (snd a) (snd b)) template_labels
        |> List.map fst
      in

      (* Main search: enumerate all label assignments via streaming *)
      let search () =
        (* 1. Enumerate fact permutations *)
        assign_next fact_state (fun _ ->
          if !budget_exceeded then () else
          (* 2. Enumerate origin permutations *)
          assign_next origin_state (fun _ ->
            if !budget_exceeded then () else
            (* 3. Enumerate batch permutations *)
            assign_next batch_state (fun _ ->
              if !budget_exceeded then () else
              (* 4. Enumerate branch permutations *)
              assign_next branch_state (fun _ ->
                if !budget_exceeded then () else
                (* 5. Enumerate template permutations *)
                assign_next template_state (fun _ ->
                  if !budget_exceeded then () else
                  let la_base = build_label_assignment () in

                  (* Compute template ordering for this candidate *)
                  let sorted_template_ids = compute_template_label_order la_base in

                  (* Build role blocks *)
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
                  let role_state = make_role_block_state all_blocks in

                  (* 6. Enumerate role block permutations *)
                  enumerate_role_blocks role_state (fun blocks ->
                    if !budget_exceeded then () else begin
                      (* Build complete role map from block permutations *)
                      let role_map = ref ScopedRoleMap.empty in

                      (* Program roles *)
                      List.iteri (fun idx r ->
                        let label = blocks.(0).start_label + blocks.(0).perm.(idx) - 1 in
                        role_map := ScopedRoleMap.add (Program_role r.role_id) label !role_map
                      ) program_roles_list;

                      (* Template roles *)
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
                )
              )
            )
          )
        )
      in

      search ();

      if !budget_exceeded then
        Error Canonicalisation_too_complex
      else if !best_payload = "" then
        Error Canonicalisation_too_complex
      else
        Ok {
          canonical_payload = !best_payload;
          canonical_preimage = !best_preimage;
          canonical_program_digest = !best_digest;
        }
    end
  end

let canonical_payload (r : canonicalized_v2) : string = r.canonical_payload
let canonical_preimage (r : canonicalized_v2) : bytes = r.canonical_preimage
let program_digest (r : canonicalized_v2) : string = r.canonical_program_digest
