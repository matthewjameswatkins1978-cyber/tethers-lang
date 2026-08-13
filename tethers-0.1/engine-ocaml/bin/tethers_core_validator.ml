open Tethers_core

type validation_error =
  | Duplicate_origin_id of origin_id
  | Duplicate_fact_id of fact_id
  | Duplicate_role_id of role_id
  | Duplicate_capability_id of capability_id
  | Duplicate_branch_id of branch_id
  | Duplicate_group_id of group_id
  | Duplicate_batch_id of batch_id
  | Duplicate_item_template_id of item_template_id
  | Missing_origin of origin_id
  | Missing_fact of fact_id
  | Missing_role of role_id
  | Missing_capability_contract of capability_id
  | Missing_branch_target of origin_id
  | Missing_item_template of item_template_id
  | Missing_entry_origin_for_actions
  | Unknown_entry_origin of origin_id
  | Duplicate_success_continuation of origin_id
  | Success_cycle of origin_id list
  | Capability_contract_digest_mismatch of capability_id
  | Duplicate_capability_contract of capability_id
  | Input_fact_not_declared of fact_id
  | Input_fact_wrong_provenance of fact_id
  | Fact_origin_provenance_missing_origin of fact_id
  | Fact_role_provenance_missing_role of fact_id
  | Fact_from_origin_provenance_mismatch of fact_id * origin_id
  | Fact_role_contract_not_exposed of fact_id * role_id
  | Fact_dependency_cycle of fact_id list
  | Anchor_origin_not_anchor of origin_id
  | Anchor_path_empty
  | Anchor_path_empty_component of origin_id * string list
  | Branch_duplicate_outcome of branch_id
  | Branch_subject_missing of branch_id
  | Together_single_member of group_id
  | Together_self_member of group_id
  | Together_duplicate_member of group_id
  | Together_unknown_member of group_id * origin_id
  | Role_fact_contract_invalid_fact of role_id * fact_id
  | Role_scope_missing_item_template of role_id
  | Item_objective_missing_role of item_template_id * role_id
  | Item_template_duplicate_origin_id of item_template_id * origin_id
  | Batch_missing_item_template of batch_id
  | Role_scope_storage_mismatch of role_id
  | Role_scope_template_mismatch of role_id * item_template_id * item_template_id
  | Role_fact_contract_duplicate_fact of role_id * fact_id
  | Role_proxy_scope_mismatch of fact_id * role_id
  | Deadline_empty of origin_id

(* ------------------------------------------------------------------ *)
(*  Helpers                                                            *)
(* ------------------------------------------------------------------ *)

let origin_id_of_site = function
  | Anchor_origin a -> Some a.anchor_origin_id
  | Action_origin a -> Some a.action_origin_id
  | Together_origin t -> Some t.together_origin_id
  | Batch_site _ -> None

let origin_ids_of_sites sites =
  List.filter_map (fun s -> origin_id_of_site s) sites

let find_duplicates (ids : 'a list) =
  let rec aux acc = function
    | [] -> []
    | x :: xs ->
        if List.mem x acc then x :: aux acc xs
        else aux (x :: acc) xs
  in
  aux [] ids

let find_duplicates_unique ids =
  let all = find_duplicates ids in
  let dedup = List.sort_uniq compare all in
  dedup

let all_origin_sites program =
  let program_sites = program.origin_sites in
  let item_sites =
    List.concat_map (fun (t : item_template) -> t.origin_sites) program.item_templates
  in
  program_sites @ item_sites

let all_branches program =
  let program_branches = program.branches in
  let item_branches =
    List.concat_map (fun (t : item_template) -> t.branches) program.item_templates
  in
  program_branches @ item_branches

let all_roles program =
  let program_roles = program.roles in
  let item_roles =
    List.concat_map (fun (t : item_template) -> t.roles) program.item_templates
  in
  program_roles @ item_roles

let all_item_template_ids program =
  List.map (fun (t : item_template) -> t.item_template_id) program.item_templates

let origin_by_id sites oid =
  List.find_opt (fun s -> origin_id_of_site s = Some oid) sites

(* Declared Facts from an Origin site *)
let declared_facts_of = function
  | Anchor_origin a -> a.declared_facts
  | Action_origin a -> a.declared_facts
  | Together_origin _ -> []
  | Batch_site b -> b.aggregate_facts

(* ------------------------------------------------------------------ *)
(*  Deterministic validator error accumulator                          *)
(* ------------------------------------------------------------------ *)

let validate program =
  let errors = ref [] in
  let add e = errors := e :: !errors in

  let all_sites = all_origin_sites program in
  let all_branches_list = all_branches program in
  let all_roles_list = all_roles program in
  let program_sites = program.origin_sites in

  (* Collect identity sets *)
  let all_origin_ids = origin_ids_of_sites all_sites in
  let all_branch_ids = List.map (fun (b : branch) -> b.branch_id) all_branches_list in
  let all_role_ids = List.map (fun (r : role) -> r.role_id) all_roles_list in
  let all_cap_ids = List.map (fun (c : capability_contract) -> c.capability_id) program.capability_contracts in
  let all_input_fact_ids = List.map (fun (f : fact) -> f.fact_id) program.input_facts in
  let all_declared_fact_ids =
    List.concat_map declared_facts_of all_sites
    |> List.map (fun (f : fact) -> f.fact_id)
  in
  let all_item_template_ids = all_item_template_ids program in

  let all_fact_ids = all_input_fact_ids @ all_declared_fact_ids in

  (* ------------------------------------------------------------------ *)
  (*  1. Identity uniqueness                                             *)
  (* ------------------------------------------------------------------ *)

  (* Global origin ID uniqueness across program and all item templates *)
  let dup_all_origins = find_duplicates_unique all_origin_ids in
  List.iter (fun oid -> add (Duplicate_origin_id oid)) dup_all_origins;

  (* input_fact IDs *)
  let dup_input_facts = find_duplicates_unique all_input_fact_ids in
  List.iter (fun fid -> add (Duplicate_fact_id fid)) dup_input_facts;

  (* All declared fact IDs *)
  let dup_declared_facts = find_duplicates_unique all_declared_fact_ids in
  List.iter (fun fid -> add (Duplicate_fact_id fid)) dup_declared_facts;

  (* Also check cross-collection fact ID conflicts *)
  let dup_all_facts = find_duplicates_unique all_fact_ids in
  List.iter (fun fid ->
    if not (List.mem fid dup_input_facts || List.mem fid dup_declared_facts) then
      add (Duplicate_fact_id fid)
  ) dup_all_facts;

  (* Branch IDs *)
  let dup_branches = find_duplicates_unique all_branch_ids in
  List.iter (fun bid -> add (Duplicate_branch_id bid)) dup_branches;

  (* Role IDs - program-scoped *)
  let program_role_ids = List.map (fun (r : role) -> r.role_id) program.roles in
  let dup_program_roles = find_duplicates_unique program_role_ids in
  List.iter (fun rid -> add (Duplicate_role_id rid)) dup_program_roles;

  (* Capability contract IDs *)
  let dup_cap_contracts = find_duplicates_unique all_cap_ids in
  List.iter (fun cid -> add (Duplicate_capability_contract cid)) dup_cap_contracts;

  (* Group IDs from Together origins *)
  let group_ids =
    List.filter_map (function Together_origin t -> Some t.group_id | _ -> None) all_sites
  in
  let dup_groups = find_duplicates_unique group_ids in
  List.iter (fun gid -> add (Duplicate_group_id gid)) dup_groups;

  (* Batch IDs *)
  let batch_ids =
    List.filter_map (function Batch_site b -> Some b.batch_id | _ -> None) all_sites
  in
  let dup_batches = find_duplicates_unique batch_ids in
  List.iter (fun bid -> add (Duplicate_batch_id bid)) dup_batches;

  (* Item Template IDs *)
  let dup_item_templates = find_duplicates_unique all_item_template_ids in
  List.iter (fun tid -> add (Duplicate_item_template_id tid)) dup_item_templates;

  (* 1a. Item Template scoped identity *)
  List.iter (fun (t : item_template) ->
    let item_origin_ids = origin_ids_of_sites t.origin_sites in
    let dup_item_origins = find_duplicates_unique item_origin_ids in
    List.iter (fun oid -> add (Item_template_duplicate_origin_id (t.item_template_id, oid))) dup_item_origins;

    let item_branch_ids = List.map (fun (b : branch) -> b.branch_id) t.branches in
    let dup_item_branches = find_duplicates_unique item_branch_ids in
    List.iter (fun bid -> add (Duplicate_branch_id bid)) dup_item_branches;

    let item_role_ids = List.map (fun (r : role) -> r.role_id) t.roles in
    let dup_item_roles = find_duplicates_unique item_role_ids in
    List.iter (fun rid -> add (Duplicate_role_id rid)) dup_item_roles
  ) program.item_templates;

  (* ------------------------------------------------------------------ *)
  (*  2. Reference integrity                                             *)
  (* ------------------------------------------------------------------ *)

  let is_known_origin oid = List.mem oid all_origin_ids in
  let is_known_fact fid = List.mem fid all_fact_ids in
  let is_known_role rid = List.mem rid all_role_ids in
  let is_known_item_template tid = List.mem tid all_item_template_ids in

  (* Origin references from entry_origin *)
  (match program.entry_origin with
   | Some oid when not (is_known_origin oid) -> add (Unknown_entry_origin oid)
   | _ -> ());

  (* Origin references from success_continuations *)
  List.iter (fun (sc : success_continuation) ->
    if not (is_known_origin sc.from_origin) then
      add (Missing_origin sc.from_origin);
    match sc.target with
    | Origin_target oid when not (is_known_origin oid) -> add (Missing_origin oid)
    | _ -> ()
  ) program.success_continuations;

  (* Origin references from Branches *)
  List.iter (fun (b : branch) ->
    if not (is_known_origin b.branch_subject) then
      add (Branch_subject_missing b.branch_id);
    List.iter (fun (_, target) ->
      match target with
      | Continue_to oid when not (is_known_origin oid) -> add (Missing_branch_target oid)
      | _ -> ()
    ) b.outcome_branches
  ) all_branches_list;

  (* Origin references from Together *)
  List.iter (fun site ->
    match site with
    | Together_origin t ->
        List.iter (fun oid ->
          if not (is_known_origin oid) then
            add (Together_unknown_member (t.group_id, oid))
        ) t.member_origin_ids
    | _ -> ()
  ) all_sites;

  (* Origin references from input_bindings (Anchor_value, Fact_from_origin) *)
  List.iter (fun site ->
    let inputs =
      match site with
      | Action_origin a -> a.inputs
      | _ -> []
    in
    List.iter (fun (ai : action_input) ->
      match ai.binding with
      | Anchor_value (origin_id, _path) ->
          if not (is_known_origin origin_id) then
            add (Missing_origin origin_id)
      | Fact_from_origin (_, origin_id) ->
          if not (is_known_origin origin_id) then
            add (Missing_origin origin_id)
      | _ -> ()
    ) inputs
  ) all_sites;

  (* Fact references from entry_guards *)
  List.iter (fun (g : fact_guard) ->
    if not (List.mem g.fact_id all_input_fact_ids) then
      add (Input_fact_not_declared g.fact_id)
  ) program.entry_guards;

  (* Role references from Item Template objectives *)
  List.iter (fun (t : item_template) ->
    match t.objective with
    | Required_role rid ->
        let item_role_ids =
          List.map (fun (r : role) -> r.role_id) t.roles
        in
        if not (List.mem rid item_role_ids) then
          add (Item_objective_missing_role (t.item_template_id, rid))
  ) program.item_templates;

  (* Role references in Fact_through_role input bindings *)
  (* Scope-aware: program origins resolve program roles only;
     item-template origins resolve roles from their own template only. *)
  let program_role_ids =
    List.map (fun (r : role) -> r.role_id) program.roles
  in
  let item_template_role_map =
    List.map (fun (t : item_template) ->
      (t.item_template_id, List.map (fun (r : role) -> r.role_id) t.roles))
      program.item_templates
  in
  let item_origin_to_template =
    List.concat_map (fun (t : item_template) ->
      List.filter_map (fun site ->
        match origin_id_of_site site with
        | Some oid -> Some (oid, t.item_template_id)
        | None -> None
      ) t.origin_sites
    ) program.item_templates
  in
  let program_origin_ids_set = origin_ids_of_sites program_sites in

  let role_visible_from_origin origin_id =
    if List.mem origin_id program_origin_ids_set then
      program_role_ids
    else
      match List.assoc_opt origin_id item_origin_to_template with
      | Some tid ->
          (match List.assoc_opt tid item_template_role_map with
           | Some rids -> rids
           | None -> [])
      | None -> []
  in

  let find_role_in_scope origin_id role_id =
    let visible = role_visible_from_origin origin_id in
    if List.mem role_id visible then Some role_id else None
  in

  List.iter (fun site ->
    let inputs =
      match site with
      | Action_origin a -> (origin_id_of_site site, a.inputs)
      | _ -> (None, [])
    in
    match inputs with
    | (Some action_origin_id, bindings) ->
        List.iter (fun (ai : action_input) ->
          match ai.binding with
          | Fact_through_role (_, role_id) ->
              if not (is_known_role role_id) then
                add (Missing_role role_id)
              else
                (match find_role_in_scope action_origin_id role_id with
                 | None -> add (Missing_role role_id)
                 | Some _ -> ())
          | _ -> ()
        ) bindings
    | _ -> ()
  ) all_sites;

  (* Item Template references from Batch sites *)
  List.iter (fun site ->
    match site with
    | Batch_site b ->
        if not (is_known_item_template b.item_template_id) then
          add (Batch_missing_item_template b.batch_id)
    | _ -> ()
  ) all_sites;

  (* ------------------------------------------------------------------ *)
  (*  3. Entry integrity                                                 *)
  (* ------------------------------------------------------------------ *)

  let has_actions =
    List.exists (fun s -> match s with Action_origin _ -> true | _ -> false) program_sites
  in
  (match program.entry_origin with
   | None when has_actions -> add Missing_entry_origin_for_actions
   | _ -> ());

  (* ------------------------------------------------------------------ *)
  (*  4. Success continuation integrity & cycle detection                *)
  (* ------------------------------------------------------------------ *)

  (* Duplicate success continuations *)
  let sc_from_ids = List.map (fun (sc : success_continuation) -> sc.from_origin) program.success_continuations in
  let dup_scs = find_duplicates_unique sc_from_ids in
  List.iter (fun oid -> add (Duplicate_success_continuation oid)) dup_scs;

  (* Cycle detection using DFS *)
  if dup_scs = [] then begin
    let sc_targets = List.map (fun (sc : success_continuation) ->
      (sc.from_origin, sc.target)) program.success_continuations
    in
    let successor from_oid =
      match List.assoc_opt from_oid sc_targets with
      | Some (Origin_target oid) -> Some oid
      | _ -> None
    in
    let visited = ref [] in
    let rec dfs stack oid =
      if List.mem oid stack then begin
        let rec cycle_from target = function
          | x :: _ when x = target -> List.rev (target :: stack)
          | _ :: rest -> cycle_from target rest
          | [] -> []
        in
        add (Success_cycle (cycle_from oid (List.rev stack)))
      end else if not (List.mem oid !visited) then begin
        visited := oid :: !visited;
        (match successor oid with
         | Some next -> dfs (oid :: stack) next
         | None -> ())
      end
    in
    List.iter (fun oid -> dfs [] oid) all_origin_ids
  end;

  (* ------------------------------------------------------------------ *)
  (*  5. Capability contract integrity                                   *)
  (* ------------------------------------------------------------------ *)

  List.iter (fun site ->
    match site with
    | Action_origin a ->
        (match List.find_opt
                 (fun (c : capability_contract) -> c.capability_id = a.capability_id)
                 program.capability_contracts
         with
         | Some c when c.contract_digest = a.contract_digest -> ()
         | Some _ -> add (Capability_contract_digest_mismatch a.capability_id)
         | None -> add (Missing_capability_contract a.capability_id))
    | _ -> ()
  ) all_sites;

  (* ------------------------------------------------------------------ *)
  (*  6. Input Fact integrity                                            *)
  (* ------------------------------------------------------------------ *)

  List.iter (fun (f : fact) ->
    match f.provenance with
    | Evaluation_input _ -> ()
    | _ -> add (Input_fact_wrong_provenance f.fact_id)
  ) program.input_facts;

  (* ------------------------------------------------------------------ *)
  (*  7. Fact provenance integrity                                       *)
  (* ------------------------------------------------------------------ *)

  let all_facts = program.input_facts @
    List.concat_map declared_facts_of all_sites
  in

  List.iter (fun (f : fact) ->
    match f.provenance with
    | Origin_provenance oid ->
        if not (is_known_origin oid) then
          add (Fact_origin_provenance_missing_origin f.fact_id)
    | Role_proxy rid ->
        if not (is_known_role rid) then
          add (Fact_role_provenance_missing_role f.fact_id)
    | _ -> ()
  ) all_facts;

  (* ------------------------------------------------------------------ *)
  (*  8. Fact dependency DAG                                             *)
  (* ------------------------------------------------------------------ *)
  (*
     Correct v0 dependency rule:
     Dependencies are derived from Action_origin input/output relationships.
     If an Action_origin OA declares output Facts {F_out} and consumes
     input Facts {F_in} (via Fact_from_origin or Fact_through_role
     bindings), then each output Fact depends on each input Fact:
         F_out -> F_in
     Literal_value and Anchor_value introduce no Fact dependency edge.
     Origin_provenance alone does not create a dependency.
     Batch aggregate placeholders are not interpreted.
  *)

  let fact_input_of_binding (ai : action_input) =
    match ai.binding with
    | Fact_from_origin (fid, _) -> Some fid
    | Fact_through_role (fid, _) -> Some fid
    | _ -> None
  in

  let action_declared_fact_ids a =
    List.map (fun (f : fact) -> f.fact_id) a.declared_facts
  in

  let action_input_fact_ids a =
    List.filter_map fact_input_of_binding a.inputs
  in

  let fact_dep_pairs =
    List.concat_map (fun site ->
      match site with
      | Action_origin a ->
          let outputs = action_declared_fact_ids a in
          let inputs = action_input_fact_ids a in
          List.concat_map (fun out ->
            List.map (fun inp -> (out, inp)) inputs
          ) outputs
      | _ -> []
    ) all_sites
  in

  let fact_dep_map =
    let rec insert key new_dep = function
      | [] -> [(key, [new_dep])]
      | (k, deps) :: rest when k = key -> (k, new_dep :: deps) :: rest
      | pair :: rest -> pair :: insert key new_dep rest
    in
    List.fold_left (fun acc (out, inp) -> insert out inp acc) [] fact_dep_pairs
  in

  let fact_visited = ref [] in

  let rec fact_dfs stack fid =
    if List.mem fid stack then begin
      let rec cycle_from target = function
        | x :: _ when x = target -> List.rev (target :: stack)
        | _ :: rest -> cycle_from target rest
        | [] -> []
      in
      add (Fact_dependency_cycle (cycle_from fid (List.rev stack)))
    end else if not (List.mem fid !fact_visited) then begin
      fact_visited := fid :: !fact_visited;
      let deps =
        match List.assoc_opt fid fact_dep_map with
        | Some ds -> ds
        | None -> []
      in
      List.iter (fun d -> fact_dfs (fid :: stack) d) deps
    end
  in
  List.iter (fun fid -> fact_dfs [] fid) all_fact_ids;

  (* ------------------------------------------------------------------ *)
  (*  9. Anchor binding integrity                                        *)
  (* ------------------------------------------------------------------ *)

  List.iter (fun site ->
    match site with
    | Action_origin a ->
        List.iter (fun (ai : action_input) ->
          match ai.binding with
          | Anchor_value (oid, path) ->
              (match origin_by_id all_sites oid with
               | Some (Anchor_origin _) -> ()
               | Some _ -> add (Anchor_origin_not_anchor oid)
               | None -> ());
              if path = [] then add Anchor_path_empty;
              if List.mem "" path then add (Anchor_path_empty_component (oid, path))
          | _ -> ()
        ) a.inputs
    | _ -> ()
  ) all_sites;

  (* ------------------------------------------------------------------ *)
  (*  10. Fact_from_origin provenance mismatch                            *)
  (* ------------------------------------------------------------------ *)

  let find_fact_provenance fid =
    match List.find_opt (fun (f : fact) -> f.fact_id = fid) all_facts with
    | Some f -> Some f.provenance
    | None -> None
  in

  List.iter (fun site ->
    match site with
    | Action_origin a ->
        List.iter (fun (ai : action_input) ->
          match ai.binding with
          | Fact_from_origin (fid, oid) ->
              if not (is_known_fact fid) then
                add (Missing_fact fid)
              else
                (match find_fact_provenance fid with
                 | Some (Origin_provenance oid') when oid = oid' -> ()
                 | Some (Origin_provenance _) ->
                     add (Fact_from_origin_provenance_mismatch (fid, oid))
                 | _ -> ())
          | _ -> ()
        ) a.inputs
    | _ -> ()
  ) all_sites;

  (* ------------------------------------------------------------------ *)
  (*  11. Fact_through_role integrity                                    *)
  (* ------------------------------------------------------------------ *)

  (* Scope-aware helper: find a role record visible from the given origin *)
  let find_role_record_in_scope origin_id role_id =
    if List.mem origin_id program_origin_ids_set then
      List.find_opt (fun (r : role) -> r.role_id = role_id) program.roles
    else
      match List.assoc_opt origin_id item_origin_to_template with
      | Some tid ->
          (match List.find_opt (fun (t : item_template) -> t.item_template_id = tid) program.item_templates with
           | Some tmpl ->
               List.find_opt (fun (r : role) -> r.role_id = role_id) tmpl.roles
           | None -> None)
      | None -> None
  in

  List.iter (fun site ->
    match site with
    | Action_origin a ->
        (match origin_id_of_site site with
         | Some origin_id ->
             List.iter (fun (ai : action_input) ->
               match ai.binding with
               | Fact_through_role (fid, rid) ->
                   if not (is_known_fact fid) then
                     add (Missing_fact fid)
                   else
                     (match find_role_record_in_scope origin_id rid with
                      | Some r ->
                          let (Role_fact_contract fact_ids) = r.fact_contract in
                          if not (List.mem fid fact_ids) then
                            add (Fact_role_contract_not_exposed (fid, rid))
                      | None -> ())
               | _ -> ()
             ) a.inputs
         | None -> ())
    | _ -> ()
  ) all_sites;

  (* ------------------------------------------------------------------ *)
  (*  12. Branch integrity                                                *)
  (* ------------------------------------------------------------------ *)

  List.iter (fun (b : branch) ->
    let outcomes = List.map fst b.outcome_branches in
    let dup_outcomes = find_duplicates outcomes in
    if dup_outcomes <> [] then
      add (Branch_duplicate_outcome b.branch_id)
  ) all_branches_list;

  (* ------------------------------------------------------------------ *)
  (*  13. Role integrity                                                  *)
  (* ------------------------------------------------------------------ *)

  List.iter (fun (r : role) ->
    (match r.scope with
     | Item_template_scope tid ->
         if not (is_known_item_template tid) then
           add (Role_scope_missing_item_template r.role_id)
     | Program_scope -> ());
    let (Role_fact_contract fact_ids) = r.fact_contract in
    List.iter (fun fid ->
      if not (is_known_fact fid) then
        add (Role_fact_contract_invalid_fact (r.role_id, fid))
    ) fact_ids
  ) all_roles_list;

  (* A1: Role physical/declarative scope consistency *)
  (* A program role must declare Program_scope *)
  List.iter (fun (r : role) ->
    match r.scope with
    | Item_template_scope _ ->
        if List.mem r.role_id (List.map (fun (r : role) -> r.role_id) program.roles) then
          add (Role_scope_storage_mismatch r.role_id)
    | Program_scope -> ()
  ) program.roles;

  (* A template role must declare Item_template_scope of the same template *)
  List.iter (fun (t : item_template) ->
    List.iter (fun (r : role) ->
      match r.scope with
      | Program_scope ->
          add (Role_scope_storage_mismatch r.role_id)
      | Item_template_scope declared_tid ->
          if declared_tid <> t.item_template_id then
            add (Role_scope_template_mismatch (r.role_id, t.item_template_id, declared_tid))
    ) t.roles
  ) program.item_templates;

  (* A2: Role_fact_contract duplicates *)
  List.iter (fun (r : role) ->
    let (Role_fact_contract fact_ids) = r.fact_contract in
    let dup_facts = find_duplicates fact_ids in
    List.iter (fun fid -> add (Role_fact_contract_duplicate_fact (r.role_id, fid))) dup_facts
  ) all_roles_list;

  (* A3: Role_proxy scope consistency *)
  (* Build a map: fact_id -> scope of its containing declaration site *)
  let fact_scope_map =
    let add_fact_scope fid scope acc =
      (fid, scope) :: acc
    in
    let acc = [] in
    (* input_facts are Program_scope *)
    let acc = List.fold_left (fun acc (f : fact) ->
      add_fact_scope f.fact_id `Program acc
    ) acc program.input_facts in
    (* program origin sites -> Program_scope *)
    let acc = List.fold_left (fun acc site ->
      let facts = declared_facts_of site in
      List.fold_left (fun acc (f : fact) -> add_fact_scope f.fact_id `Program acc) acc facts
    ) acc program_sites in
    (* template origin sites -> Item_template_scope(tid) *)
    List.fold_left (fun acc (t : item_template) ->
      List.fold_left (fun acc site ->
        let facts = declared_facts_of site in
        List.fold_left (fun acc (f : fact) ->
          add_fact_scope f.fact_id (`Template t.item_template_id) acc
        ) acc facts
      ) acc t.origin_sites
    ) acc program.item_templates
  in

  (* For each Role_proxy, check that the role is visible in the fact's scope *)
  List.iter (fun site ->
    let facts = declared_facts_of site in
    List.iter (fun (f : fact) ->
      match f.provenance with
      | Role_proxy rid ->
          let fact_scope =
            List.assoc_opt f.fact_id fact_scope_map
          in
          (match fact_scope with
           | Some `Program ->
               if not (List.mem rid program_role_ids) then
                 add (Role_proxy_scope_mismatch (f.fact_id, rid))
           | Some (`Template tid) ->
               let template_role_ids =
                 match List.find_opt (fun (t : item_template) -> t.item_template_id = tid) program.item_templates with
                 | Some t -> List.map (fun (r : role) -> r.role_id) t.roles
                 | None -> []
               in
               if not (List.mem rid template_role_ids) then
                 add (Role_proxy_scope_mismatch (f.fact_id, rid))
           | None -> ())
      | _ -> ()
    ) facts
  ) all_sites;

  (* ------------------------------------------------------------------ *)
  (*  14. Together integrity                                               *)
  (* ------------------------------------------------------------------ *)

  List.iter (fun site ->
    match site with
    | Together_origin t ->
        if List.length t.member_origin_ids < 2 then
          add (Together_single_member t.group_id);
        if List.mem t.together_origin_id t.member_origin_ids then
          add (Together_self_member t.group_id);
        let mem_dup = find_duplicates t.member_origin_ids in
        if mem_dup <> [] then
          add (Together_duplicate_member t.group_id)
    | _ -> ()
  ) all_sites;

  (* ------------------------------------------------------------------ *)
  (*  15. Deadline structural validation                                  *)
  (* ------------------------------------------------------------------ *)

  List.iter (fun site ->
    let constraints =
      match site with
      | Action_origin a -> a.execution_constraints
      | _ -> []
    in
    List.iter (fun c ->
      match c with
      | Deadline s when s = "" ->
          (match origin_id_of_site site with
           | Some oid -> add (Deadline_empty oid)
           | None -> ())
      | _ -> ()
    ) constraints
  ) all_sites;

  (* ------------------------------------------------------------------ *)
  (*  Return result                                                       *)
  (* ------------------------------------------------------------------ *)

  match !errors with
  | [] -> Ok ()
  | errs -> Error (List.rev errs)
