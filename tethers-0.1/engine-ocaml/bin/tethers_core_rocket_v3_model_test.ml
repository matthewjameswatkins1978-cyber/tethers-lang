open Tethers_core
open Tethers_core_validator
open Tethers_core_rocket_v3_model

let tests_run = ref 0
let tests_passed = ref 0

let check name condition =
  incr tests_run;
  if condition then incr tests_passed
  else begin
    Printf.eprintf "FAIL: %s\n%!" name;
    exit 1
  end

let check_ok name = function
  | Ok value -> incr tests_run; incr tests_passed; value
  | Error errors ->
      incr tests_run;
      let label = function
        | Duplicate_origin_id _ -> "Duplicate_origin_id"
        | Duplicate_fact_id _ -> "Duplicate_fact_id"
        | Duplicate_role_id _ -> "Duplicate_role_id"
        | Duplicate_capability_id _ -> "Duplicate_capability_id"
        | Duplicate_branch_id _ -> "Duplicate_branch_id"
        | Duplicate_group_id _ -> "Duplicate_group_id"
        | Duplicate_batch_id _ -> "Duplicate_batch_id"
        | Duplicate_item_template_id _ -> "Duplicate_item_template_id"
        | Missing_origin _ -> "Missing_origin"
        | Missing_fact _ -> "Missing_fact"
        | Missing_role _ -> "Missing_role"
        | Missing_capability_contract _ -> "Missing_capability_contract"
        | Missing_branch_target _ -> "Missing_branch_target"
        | Missing_item_template _ -> "Missing_item_template"
        | Missing_entry_origin_for_actions -> "Missing_entry_origin_for_actions"
        | Unknown_entry_origin _ -> "Unknown_entry_origin"
        | Duplicate_success_continuation _ -> "Duplicate_success_continuation"
        | Success_cycle _ -> "Success_cycle"
        | Capability_contract_digest_mismatch _ -> "Capability_contract_digest_mismatch"
        | Duplicate_capability_contract _ -> "Duplicate_capability_contract"
        | Input_fact_not_declared _ -> "Input_fact_not_declared"
        | Input_fact_wrong_provenance _ -> "Input_fact_wrong_provenance"
        | Fact_origin_provenance_missing_origin _ -> "Fact_origin_provenance_missing_origin"
        | Fact_role_provenance_missing_role _ -> "Fact_role_provenance_missing_role"
        | Fact_from_origin_provenance_mismatch _ -> "Fact_from_origin_provenance_mismatch"
        | Fact_role_contract_not_exposed _ -> "Fact_role_contract_not_exposed"
        | Fact_dependency_cycle _ -> "Fact_dependency_cycle"
        | Anchor_origin_not_anchor _ -> "Anchor_origin_not_anchor"
        | Anchor_path_empty -> "Anchor_path_empty"
        | Anchor_path_empty_component _ -> "Anchor_path_empty_component"
        | Branch_duplicate_outcome _ -> "Branch_duplicate_outcome"
        | Branch_subject_missing _ -> "Branch_subject_missing"
        | Together_single_member _ -> "Together_single_member"
        | Together_self_member _ -> "Together_self_member"
        | Together_duplicate_member _ -> "Together_duplicate_member"
        | Together_unknown_member _ -> "Together_unknown_member"
        | Role_fact_contract_invalid_fact _ -> "Role_fact_contract_invalid_fact"
        | Role_scope_missing_item_template _ -> "Role_scope_missing_item_template"
        | Item_objective_missing_role _ -> "Item_objective_missing_role"
        | Item_template_duplicate_origin_id _ -> "Item_template_duplicate_origin_id"
        | Batch_missing_item_template _ -> "Batch_missing_item_template"
        | Role_scope_storage_mismatch _ -> "Role_scope_storage_mismatch"
        | Role_scope_template_mismatch _ -> "Role_scope_template_mismatch"
        | Role_fact_contract_duplicate_fact _ -> "Role_fact_contract_duplicate_fact"
        | Role_proxy_scope_mismatch _ -> "Role_proxy_scope_mismatch"
        | Deadline_empty _ -> "Deadline_empty"
      in
      Printf.eprintf "FAIL: %s (expected valid Core; errors=%s)\n%!" name
        (String.concat "," (List.map label errors));
      exit 1

let check_error name = function
  | Error _ -> incr tests_run; incr tests_passed
  | Ok _ ->
      incr tests_run;
      Printf.eprintf "FAIL: %s (expected validator failure)\n%!" name;
      exit 1

let oid s = origin_id_of_string s
let fid s = fact_id_of_string s
let rid s = role_id_of_string s
let cid s = capability_id_of_string s
let bid s = branch_id_of_string s
let gid s = group_id_of_string s
let batchid s = batch_id_of_string s
let tid s = item_template_id_of_string s
let pid s = program_id_of_string s
let version s = core_version_of_string s
let snapshot s = host_snapshot_key_of_string s
let digest s = capability_contract_digest_of_string s
let input_name s = capability_input_name_of_string s

let reverse_if needed xs = if needed then List.rev xs else xs

let eval_fact name = {
  fact_id = fid name;
  schema_description = "schema is neutral";
  provenance = Evaluation_input (snapshot "snapshot-input", String_type);
}

let origin_fact id provenance = {
  fact_id = fid id;
  schema_description = "schema is neutral";
  provenance;
}

let capability_contract name contract_digest = {
  capability_id = cid name;
  contract_digest = digest contract_digest;
  schema_description = "capability schema is neutral";
}

let rich_program tag reverse =
  let o_anchor = oid ("O_anchor_" ^ tag) in
  let o_action = oid ("O_action_" ^ tag) in
  let o_together = oid ("O_together_" ^ tag) in
  let o_template_action = oid ("O_template_action_" ^ tag) in
  let template = tid ("T_" ^ tag) in
  let batch_program = batchid ("Batch_program_" ^ tag) in
  let batch_template = batchid ("Batch_template_" ^ tag) in
  let program_role = rid ("R_program_" ^ tag) in
  let template_role = rid ("R_template_" ^ tag) in
  let input = eval_fact ("F_input_" ^ tag) in
  let f_anchor = origin_fact ("F_anchor_" ^ tag) (Origin_provenance o_anchor) in
  let f_from = origin_fact ("F_from_" ^ tag) (Origin_provenance o_anchor) in
  let f_action = origin_fact ("F_action_" ^ tag) (Origin_provenance o_action) in
  let f_role = origin_fact ("F_role_" ^ tag) (Role_proxy program_role) in
  let f_batch_program = origin_fact ("F_batch_program_" ^ tag) (Role_proxy program_role) in
  let f_template = origin_fact ("F_template_" ^ tag) (Origin_provenance o_template_action) in
  let f_template_role = origin_fact ("F_template_role_" ^ tag) (Role_proxy template_role) in
  let f_batch_template = origin_fact ("F_batch_template_" ^ tag) (Role_proxy template_role) in
  let anchor = Anchor_origin {
    anchor_origin_id = o_anchor;
    event_name = "event.received";
    declared_facts = [f_anchor; f_from; f_role];
  } in
  let action = Action_origin {
    action_origin_id = o_action;
    capability_id = cid "cap.action";
    contract_digest = digest "digest.action";
    inputs = [
      { input_name = input_name "from";
        binding = Fact_from_origin (f_from.fact_id, o_anchor) };
      { input_name = input_name "through";
        binding = Fact_through_role (f_role.fact_id, program_role) };
      { input_name = input_name "anchor";
        binding = Anchor_value (o_anchor, ["event"; "value"]) };
      { input_name = input_name "batch";
        binding = Batch_item_context template };
    ];
    declared_facts = [f_action];
    execution_constraints = [Deadline "soon"];
  } in
  let together = Together_origin {
    together_origin_id = o_together;
    group_id = gid ("group_" ^ tag);
    member_origin_ids = [o_anchor; o_action];
    objective = All_members_succeed;
  } in
  let program_batch = Batch_site {
    batch_id = batch_program;
    collection_provenance = batch_collection_provenance_of_string "collection.program";
    item_template_id = template;
    traversal_policy = batch_traversal_policy_of_string "ordered";
    composite_objective = batch_objective_of_string "all";
    aggregate_facts = [f_batch_program];
  } in
  let template_action = Action_origin {
    action_origin_id = o_template_action;
    capability_id = cid "cap.template-action";
    contract_digest = digest "digest.template-action";
    inputs = [
      { input_name = input_name "local";
        binding = Fact_through_role (f_template_role.fact_id, template_role) };
    ];
    declared_facts = [f_template];
    execution_constraints = [];
  } in
  let template_batch_site = {
    batch_id = batch_template;
    collection_provenance = batch_collection_provenance_of_string "collection.template";
    item_template_id = template;
    traversal_policy = batch_traversal_policy_of_string "ordered";
    composite_objective = batch_objective_of_string "all";
    aggregate_facts = [f_template_role; f_batch_template];
  } in
  let program_branch = {
    branch_id = bid ("B_program_" ^ tag);
    branch_subject = o_anchor;
    outcome_branches = [
      (Success, Continue_to o_action);
      (Failure, Stop);
      (Uncertain, Continue_to o_together);
      (Cancelled, Stop);
    ];
  } in
  let template_branch = {
    branch_id = bid ("B_template_" ^ tag);
    branch_subject = o_template_action;
    outcome_branches = [(Success, Stop)];
  } in
  let program_role_value = {
    role_id = program_role;
    scope = Program_scope;
    fact_contract = Role_fact_contract [f_role.fact_id; f_batch_program.fact_id];
    eligible_fulfillment = role_fulfillment_of_string "program.fulfilment";
  } in
  let template_role_value = {
    role_id = template_role;
    scope = Item_template_scope template;
    fact_contract = Role_fact_contract [f_template_role.fact_id; f_batch_template.fact_id];
    eligible_fulfillment = role_fulfillment_of_string "template.fulfilment";
  } in
  let item_template = {
    item_template_id = template;
    origin_sites = reverse_if reverse [template_action; Batch_site template_batch_site];
    branches = reverse_if reverse [template_branch];
    roles = reverse_if reverse [template_role_value];
    objective = Required_role template_role;
  } in
  let origins = reverse_if reverse [anchor; action; together; program_batch] in
  let continuations = reverse_if reverse [
    { from_origin = o_anchor; target = Origin_target o_action };
    { from_origin = o_action; target = Program_complete };
  ] in
  {
    program_id = pid ("program_" ^ tag);
    core_version = version "0.1.0";
    input_facts = [input];
    entry_guards = [{ fact_id = input.fact_id; operator = Equals; expected = String_value "ready" }];
    entry_origin = Some o_anchor;
    success_continuations = continuations;
    origin_sites = origins;
    branches = reverse_if reverse [program_branch];
    roles = reverse_if reverse [program_role_value];
    item_templates = reverse_if reverse [item_template];
    capability_contracts = reverse_if reverse [
      capability_contract "cap.action" "digest.action";
      capability_contract "cap.template-action" "digest.template-action";
    ];
  }

let chain_program size reverse =
  let rec ids index acc =
    if index = size then List.rev acc
    else ids (index + 1) ((oid ("chain-origin-" ^ string_of_int index)) :: acc)
  in
  let origin_ids = ids 0 [] in
  let actions = List.mapi (fun index origin_id ->
    Action_origin {
      action_origin_id = origin_id;
      capability_id = cid ("cap.chain-" ^ string_of_int index);
      contract_digest = digest ("digest.chain-" ^ string_of_int index);
      inputs = [];
      declared_facts = [];
      execution_constraints = [];
    }
  ) origin_ids in
  let rec continuations index acc =
    if index = size then List.rev acc
    else
      let source = List.nth origin_ids index in
      let target = if index + 1 = size then Program_complete
        else Origin_target (List.nth origin_ids (index + 1)) in
      continuations (index + 1) ({ from_origin = source; target } :: acc)
  in
  {
    program_id = pid "chain-program";
    core_version = version "0.1.0";
    input_facts = [];
    entry_guards = [];
    entry_origin = Some (List.hd origin_ids);
    success_continuations = reverse_if reverse (continuations 0 []);
    origin_sites = reverse_if reverse actions;
    branches = [];
    roles = [];
    item_templates = [];
    capability_contracts = reverse_if reverse (List.mapi (fun index _ ->
      capability_contract
        ("cap.chain-" ^ string_of_int index)
        ("digest.chain-" ^ string_of_int index)
    ) origin_ids);
  }

let find_edges model relation =
  List.filter (fun edge -> edge.relation = relation) (all_forward_edges model)

let test_rich_family_classification () =
  let model = check_ok "rich program validates" (build (rich_program "alpha" false)) in
  check "four Origin vertices, excluding Batch sites"
    (vertex_family_count model Origin = 4);
  check "two Batch vertices"
    (vertex_family_count model Batch = 2);
  check "two Branch vertices" (vertex_family_count model Branch = 2);
  check "one Fact vertex per fact" (vertex_family_count model Fact = 9);
  check "one ItemTemplate vertex" (vertex_family_count model ItemTemplate = 1);
  check "two ScopedRole vertices" (vertex_family_count model ScopedRole = 2);
  check "four fixed vertices are not anonymous"
    (vertex_count model = 4 + 4 + 2 + 2 + 9 + 1 + 2)

let test_all_relations_and_coverage () =
  let model = check_ok "coverage program validates" (build (rich_program "coverage" false)) in
  List.iter (fun relation ->
    check ("relation present: " ^ relation_name relation)
      (List.mem relation (relation_kinds_present model))
  ) required_relation_kinds;
  List.iter (fun (_family, relations) ->
    List.iter (fun relation ->
      check "lookup coverage relation is in maintained taxonomy"
        (List.mem relation required_relation_kinds)
    ) relations
  ) enc_v2_lookup_coverage;
  check "all six Enc_V2 lookup families are mapped"
    (List.length enc_v2_lookup_coverage = 6);
  check "Together has its dedicated relation"
    (find_edges model Rel_together_member <> []);
  check "Together is not represented as Branch subject"
    (List.for_all (fun edge -> edge.relation = Rel_together_member)
       (find_edges model Rel_together_member));
  let outcomes = List.filter_map (fun edge ->
    match edge.discriminator with
    | Branch_outcome outcome -> Some outcome
    | _ -> None
  ) (find_edges model Rel_branch_target @ find_edges model Rel_branch_stop) in
  List.iter (fun outcome ->
    check "branch outcome discriminator present" (List.mem outcome outcomes)
  ) [Success; Failure; Uncertain; Cancelled];
  check "branch Stop is structurally represented"
    (find_edges model Rel_branch_stop <> [])

let test_forward_inverse_duality () =
  let model = check_ok "duality program validates" (build (rich_program "duality" false)) in
  let forward = ref [] in
  for source = 0 to vertex_count model - 1 do
    List.iter (fun edge -> forward := (source, edge) :: !forward)
      (forward_edges model source)
  done;
  check "forward and reverse edge totals agree"
    (List.length !forward = List.length (all_forward_edges model));
  List.iter (fun (source, edge) ->
    let matches = List.filter (fun reverse ->
      reverse.target = source &&
      reverse.relation = edge.relation &&
      reverse.discriminator = edge.discriminator &&
      reverse.payload = edge.payload
    ) (reverse_edges model edge.target) in
    check "every forward occurrence has an exact inverse" (matches <> [])
  ) !forward

let test_raw_ids_and_storage_order () =
  let first = check_ok "raw-id baseline validates" (build (rich_program "raw-a" false)) in
  let renamed_and_reordered = check_ok "renamed/reordered program validates"
      (build (rich_program "raw-z" true)) in
  let first_evidence = structural_evidence first in
  let second_evidence = structural_evidence renamed_and_reordered in
  check "raw IDs and storage order do not affect structural evidence"
    (first_evidence = second_evidence)

let test_neutral_scalar_boundary () =
  let original = rich_program "neutral" false in
  let changed_input = match original.input_facts with
    | [fact] -> { original with
        program_id = pid "a-different-program-id";
        input_facts = [{ fact with schema_description = "changed schema" }]
      }
    | _ -> assert false
  in
  let first = check_ok "neutral baseline validates" (build original) in
  let second = check_ok "neutral change validates" (build changed_input) in
  check "program_id and schema description are neutral"
    (structural_evidence first = structural_evidence second)

let test_scope_collision () =
  let model = check_ok "scope collision fixture validates" (build (rich_program "scope" false)) in
  check "program and template roles remain distinct"
    (vertex_family_count model ScopedRole = 2);
  check "template role scope edge exists"
    (find_edges model Rel_role_scope_template <> []);
  check "program role scope edge exists"
    (find_edges model Rel_role_scope_program <> []);
  check "Role_proxy relation exists"
    (find_edges model Rel_fact_provenance_role <> []);
  check "Fact_through_role relation exists"
    (find_edges model Rel_action_input_role <> [])

let test_invalid_core_fails_closed () =
  let program = rich_program "invalid" false in
  let duplicate = match program.success_continuations with
    | first :: _ -> { program with success_continuations = first :: program.success_continuations }
    | [] -> assert false
  in
  check_error "invalid Core is rejected before model construction" (build duplicate)

let test_success_chain_sizes () =
  List.iter (fun size ->
    let model = check_ok ("chain size " ^ string_of_int size)
        (build (chain_program size (size mod 2 = 0))) in
    check ("chain Origin count " ^ string_of_int size)
      (vertex_family_count model Origin = size);
    check ("chain root relation " ^ string_of_int size)
      (List.length (find_edges model Rel_root_entry_origin) = 1);
    check ("chain next relation count " ^ string_of_int size)
      (List.length (find_edges model Rel_success_next) = max 0 (size - 1));
    check ("chain complete relation " ^ string_of_int size)
      (List.length (find_edges model Rel_success_complete) = 1)
  ) [1; 10; 50; 100; 250; 500; 1000]

let () =
  test_rich_family_classification ();
  test_all_relations_and_coverage ();
  test_forward_inverse_duality ();
  test_raw_ids_and_storage_order ();
  test_neutral_scalar_boundary ();
  test_scope_collision ();
  test_invalid_core_fails_closed ();
  test_success_chain_sizes ();
  Printf.printf "rocket-v3-model: %d/%d checks passed\n%!" !tests_passed !tests_run
