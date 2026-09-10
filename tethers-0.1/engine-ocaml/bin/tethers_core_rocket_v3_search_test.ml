module Core = Tethers_core
module Model = Tethers_core_rocket_v3_model
module Partition = Tethers_core_rocket_v3_partition
module Refine = Tethers_core_rocket_v3_refine
module Encode = Tethers_core_rocket_v3_encode
module Format = Tethers_core_canonical_v2_format
module Oracle = Tethers_core_canonical_v2_reference

let tests_run = ref 0
let tests_passed = ref 0

let check name condition =
  incr tests_run;
  if condition then incr tests_passed
  else begin
    Printf.eprintf "FAIL: %s\n%!" name;
    exit 1
  end

let oid value = Core.origin_id_of_string value
let fid value = Core.fact_id_of_string value
let rid value = Core.role_id_of_string value
let bid value = Core.branch_id_of_string value
let batchid value = Core.batch_id_of_string value
let tid value = Core.item_template_id_of_string value
let pid value = Core.program_id_of_string value
let cid value = Core.capability_id_of_string value
let digest value = Core.capability_contract_digest_of_string value
let input_name value = Core.capability_input_name_of_string value
let version value = Core.core_version_of_string value

let fact id provenance = {
      Core.fact_id = fid id;
  schema_description = "neutral fact schema";
  provenance;
}

let contract capability contract_digest = {
  Core.capability_id = cid capability;
  contract_digest = digest contract_digest;
  schema_description = "neutral capability schema";
}

let empty_program () = {
  Core.program_id = pid "empty";
  core_version = version "0.1.0";
  input_facts = [];
  entry_guards = [];
  entry_origin = None;
  success_continuations = [];
  origin_sites = [];
  branches = [];
  roles = [];
  item_templates = [];
  capability_contracts = [];
}

let chain_program size =
  let origins = List.init size (fun index -> oid ("chain-origin-" ^ string_of_int index)) in
  let sites = List.map (fun origin_id ->
    Core.Action_origin {
      action_origin_id = origin_id;
      capability_id = cid "chain.capability";
      contract_digest = digest "chain.contract";
      inputs = [];
      declared_facts = [];
      execution_constraints = [];
    }
  ) origins in
  let continuations = List.mapi (fun index from_origin ->
    let target = if index + 1 = size then Core.Program_complete
      else Core.Origin_target (List.nth origins (index + 1))
    in
    { Core.from_origin; target }
  ) origins in
  {
    (empty_program ()) with
    program_id = pid "chain-program";
    entry_origin = Some (List.hd origins);
    success_continuations = continuations;
    origin_sites = sites;
    capability_contracts = [contract "chain.capability" "chain.contract"];
  }

let _stage_a_program tag reverse =
  let reverse_if values = if reverse then List.rev values else values in
  let p_anchor = oid ("p-anchor-" ^ tag) in
  let p_action = oid ("p-action-" ^ tag) in
  let p_together = oid ("p-together-" ^ tag) in
  let t1_anchor = oid ("t1-anchor-" ^ tag) in
  let t1_action = oid ("t1-action-" ^ tag) in
  let t2_anchor = oid ("t2-anchor-" ^ tag) in
  let t2_action = oid ("t2-action-" ^ tag) in
  let template1 = tid ("template-one-" ^ tag) in
  let template2 = tid ("template-two-" ^ tag) in
  let batch_program = batchid ("batch-program-" ^ tag) in
  let batch1 = batchid ("batch-one-" ^ tag) in
  let batch2 = batchid ("batch-two-" ^ tag) in
  let program_role = rid ("shared-role-" ^ tag) in
  let template_role1 = rid ("shared-role-" ^ tag) in
  let template_role2 = rid ("shared-role-" ^ tag) in
  let input_text = fact ("input-text-" ^ tag)
      (Core.Evaluation_input
         (Core.host_snapshot_key_of_string "snapshot-text", Core.String_type)) in
  let input_number = fact ("input-number-" ^ tag)
      (Core.Evaluation_input
         (Core.host_snapshot_key_of_string "snapshot-number", Core.Integer_type)) in
  let p_anchor_fact = fact ("p-anchor-fact-" ^ tag) (Core.Origin_provenance p_anchor) in
  let p_from_fact = fact ("p-from-fact-" ^ tag) (Core.Origin_provenance p_anchor) in
  let p_role_fact = fact ("p-role-fact-" ^ tag) (Core.Role_proxy program_role) in
  let p_batch_fact = fact ("p-batch-fact-" ^ tag) (Core.Role_proxy program_role) in
  let p_action_fact = fact ("p-action-fact-" ^ tag) (Core.Origin_provenance p_action) in
  let t1_anchor_fact = fact ("t1-anchor-fact-" ^ tag) (Core.Origin_provenance t1_anchor) in
  let t1_role_fact = fact ("t1-role-fact-" ^ tag) (Core.Role_proxy template_role1) in
  let t1_batch_fact = fact ("t1-batch-fact-" ^ tag) (Core.Role_proxy template_role1) in
  let t1_action_fact = fact ("t1-action-fact-" ^ tag) (Core.Origin_provenance t1_action) in
  let t2_anchor_fact = fact ("t2-anchor-fact-" ^ tag) (Core.Origin_provenance t2_anchor) in
  let t2_role_fact = fact ("t2-role-fact-" ^ tag) (Core.Role_proxy template_role2) in
  let t2_batch_fact = fact ("t2-batch-fact-" ^ tag) (Core.Role_proxy template_role2) in
  let t2_action_fact = fact ("t2-action-fact-" ^ tag) (Core.Origin_provenance t2_action) in
  let p_anchor_site = Core.Anchor_origin {
    anchor_origin_id = p_anchor;
    event_name = "program.event";
    declared_facts = reverse_if [p_anchor_fact; p_role_fact];
  } in
  let p_action_site = Core.Action_origin {
    action_origin_id = p_action;
    capability_id = cid "program.action";
    contract_digest = digest "program.action.contract";
    inputs = reverse_if [
      { Core.input_name = input_name "from";
        binding = Core.Fact_from_origin (p_from_fact.fact_id, p_anchor) };
      { Core.input_name = input_name "through";
        binding = Core.Fact_through_role (p_role_fact.fact_id, program_role) };
      { Core.input_name = input_name "anchor";
        binding = Core.Anchor_value (p_anchor, ["event"; "value"]) };
      { Core.input_name = input_name "batch";
        binding = Core.Batch_item_context template1 };
    ];
    declared_facts = [p_action_fact];
    execution_constraints = [Core.Deadline "soon"];
  } in
  let p_together_site = Core.Together_origin {
    together_origin_id = p_together;
    group_id = Core.group_id_of_string ("group-" ^ tag);
    member_origin_ids = reverse_if [p_anchor; p_action];
    objective = Core.All_members_succeed;
  } in
  let p_batch_site = Core.Batch_site {
    batch_id = batch_program;
    collection_provenance = Core.batch_collection_provenance_of_string "program.collection";
    item_template_id = template1;
    traversal_policy = Core.batch_traversal_policy_of_string "ordered";
    composite_objective = Core.batch_objective_of_string "all";
    aggregate_facts = [p_batch_fact];
  } in
  let template_site template_id anchor_id action_id
      (anchor_facts : Core.fact list) (action_facts : Core.fact list)
      (role_fact : Core.fact) role_id =
    let anchor = Core.Anchor_origin {
      anchor_origin_id = anchor_id;
      event_name = if template_id = template1 then "template.one.event" else "template.two.event";
      declared_facts = anchor_facts;
    } in
    let action = Core.Action_origin {
      action_origin_id = action_id;
      capability_id = if template_id = template1 then cid "template.one.action"
        else cid "template.two.action";
      contract_digest = if template_id = template1 then digest "template.one.contract"
        else digest "template.two.contract";
      inputs = [{
        Core.input_name = input_name "local";
        binding = Core.Fact_through_role (role_fact.Core.fact_id, role_id);
      }];
      declared_facts = action_facts;
      execution_constraints = [];
    } in
    anchor, action
  in
  let t1_anchor_site, t1_action_site = template_site template1 t1_anchor t1_action
      [t1_anchor_fact] [t1_action_fact] t1_role_fact template_role1 in
  let t2_anchor_site, t2_action_site = template_site template2 t2_anchor t2_action
      [t2_anchor_fact] [t2_action_fact] t2_role_fact template_role2 in
  let t1_batch_site = Core.Batch_site {
    batch_id = batch1;
    collection_provenance = Core.batch_collection_provenance_of_string "template.one.collection";
    item_template_id = template1;
    traversal_policy = Core.batch_traversal_policy_of_string "ordered";
    composite_objective = Core.batch_objective_of_string "all";
    aggregate_facts = [t1_batch_fact];
  } in
  let t2_batch_site = Core.Batch_site {
    batch_id = batch2;
    collection_provenance = Core.batch_collection_provenance_of_string "template.two.collection";
    item_template_id = template2;
    traversal_policy = Core.batch_traversal_policy_of_string "ordered";
    composite_objective = Core.batch_objective_of_string "all";
    aggregate_facts = [t2_batch_fact];
  } in
  let program_role_value = {
    Core.role_id = program_role;
    scope = Core.Program_scope;
    fact_contract = Core.Role_fact_contract [p_role_fact.fact_id; p_batch_fact.fact_id];
    eligible_fulfillment = Core.role_fulfillment_of_string "program.fulfilment";
  } in
  let template_role role_id = {
    Core.role_id = role_id;
    scope = Core.Item_template_scope (if role_id = template_role1 then template1 else template2);
    fact_contract = Core.Role_fact_contract (
      if role_id = template_role1
      then [t1_role_fact.fact_id; t1_batch_fact.fact_id]
      else [t2_role_fact.fact_id; t2_batch_fact.fact_id]);
    eligible_fulfillment = Core.role_fulfillment_of_string
      (if role_id = template_role1 then "template.one.fulfilment"
       else "template.two.fulfilment");
  } in
  let t1_role_value = template_role template_role1 in
  let t2_role_value = template_role template_role2 in
  let template_branch subject branch_id = {
    Core.branch_id = branch_id;
    branch_subject = subject;
    outcome_branches = [
      (Core.Success, Core.Stop);
    ];
  } in
  let t1 = {
    Core.item_template_id = template1;
    origin_sites = reverse_if [t1_anchor_site; t1_action_site; t1_batch_site];
    branches = [template_branch t1_anchor (bid ("branch-one-" ^ tag))];
    roles = [t1_role_value];
    objective = Core.Required_role template_role1;
  } in
  let t2 = {
    Core.item_template_id = template2;
    origin_sites = reverse_if [t2_anchor_site; t2_action_site; t2_batch_site];
    branches = [template_branch t2_anchor (bid ("branch-two-" ^ tag))];
    roles = [t2_role_value];
    objective = Core.Required_role template_role2;
  } in
  {
    Core.program_id = pid ("stage-a-program-" ^ tag);
    core_version = version "0.1.0";
    input_facts = reverse_if [input_text; input_number];
    entry_guards = reverse_if [
      { Core.fact_id = input_text.fact_id; operator = Core.Equals;
        expected = Core.String_value "ready" };
      { Core.fact_id = input_number.fact_id; operator = Core.Greater_than;
        expected = Core.Integer_value 0 };
    ];
    entry_origin = Some p_anchor;
    success_continuations = reverse_if [
      { Core.from_origin = p_anchor; target = Core.Origin_target p_action };
      { Core.from_origin = p_action; target = Core.Program_complete };
    ];
    origin_sites = reverse_if [p_anchor_site; p_action_site; p_together_site; p_batch_site];
    branches = [{
      Core.branch_id = bid ("program-branch-" ^ tag);
      branch_subject = p_anchor;
      outcome_branches = [
        (Core.Success, Core.Continue_to p_action);
        (Core.Failure, Core.Stop);
        (Core.Uncertain, Core.Continue_to p_together);
        (Core.Cancelled, Core.Stop);
      ];
    }];
    roles = [program_role_value];
    item_templates = reverse_if [t1; t2];
    capability_contracts = reverse_if [
      contract "program.action" "program.action.contract";
      contract "template.one.action" "template.one.contract";
      contract "template.two.action" "template.two.contract";
    ];
  }

let model_of_program name program =
  match Model.build program with
  | Ok model -> model
  | Error _ ->
      Printf.eprintf "FAIL: %s (valid fixture rejected)\n%!" name;
      exit 1

(* A compact all-features fixture kept within the slow-oracle domain.  It
   covers all four Action bindings, all Branch outcomes, Batch sites,
   templates, and three legal role blocks without making the oracle
   candidate product exceed its deterministic test limit. *)
let compact_certificate_program tag reverse =
  let reverse_if values = if reverse then List.rev values else values in
  let p_anchor = oid ("compact-anchor-" ^ tag) in
  let p_action = oid ("compact-action-" ^ tag) in
  let t1_anchor = oid ("compact-template-anchor-" ^ tag) in
  let template1 = tid ("compact-template-one-" ^ tag) in
  let template2 = tid ("compact-template-two-" ^ tag) in
  let p_batch = batchid ("compact-program-batch-" ^ tag) in
  let t1_batch = batchid ("compact-template-batch-" ^ tag) in
  let p_role = rid ("compact-program-role-" ^ tag) in
  let t1_role = rid ("compact-template-one-role-" ^ tag) in
  let t2_role = rid ("compact-template-two-role-" ^ tag) in
  let input = fact ("compact-input-" ^ tag)
      (Core.Evaluation_input
         (Core.host_snapshot_key_of_string "compact-snapshot", Core.String_type)) in
  let anchor_fact = fact ("compact-anchor-fact-" ^ tag)
      (Core.Origin_provenance p_anchor) in
  let program_role_fact = fact ("compact-program-role-fact-" ^ tag)
      (Core.Role_proxy p_role) in
  let template_role_fact = fact ("compact-template-role-fact-" ^ tag)
      (Core.Role_proxy t1_role) in
  let p_anchor_site = Core.Anchor_origin {
    anchor_origin_id = p_anchor;
    event_name = "compact.event";
    declared_facts = [anchor_fact];
  } in
  let p_action_site = Core.Action_origin {
    action_origin_id = p_action;
    capability_id = cid "compact.action";
    contract_digest = digest "compact.action.contract";
    inputs = reverse_if [
      { Core.input_name = input_name "from";
        binding = Core.Fact_from_origin (anchor_fact.Core.fact_id, p_anchor) };
      { Core.input_name = input_name "role";
        binding = Core.Fact_through_role (program_role_fact.Core.fact_id, p_role) };
      { Core.input_name = input_name "anchor";
        binding = Core.Anchor_value (p_anchor, ["event"; "value"]) };
      { Core.input_name = input_name "batch";
        binding = Core.Batch_item_context template1 };
    ];
    declared_facts = [];
    execution_constraints = [Core.Deadline "soon"];
  } in
  let t1_anchor_site = Core.Anchor_origin {
    anchor_origin_id = t1_anchor;
    event_name = "compact.template.event";
    declared_facts = [];
  } in
  let p_batch_site = Core.Batch_site {
    batch_id = p_batch;
    collection_provenance = Core.batch_collection_provenance_of_string "compact.program.collection";
    item_template_id = template1;
    traversal_policy = Core.batch_traversal_policy_of_string "ordered";
    composite_objective = Core.batch_objective_of_string "all";
    aggregate_facts = [program_role_fact];
  } in
  let t1_batch_site = Core.Batch_site {
    batch_id = t1_batch;
    collection_provenance = Core.batch_collection_provenance_of_string "compact.template.collection";
    item_template_id = template1;
    traversal_policy = Core.batch_traversal_policy_of_string "ordered";
    composite_objective = Core.batch_objective_of_string "all";
    aggregate_facts = [template_role_fact];
  } in
  let program_role = {
    Core.role_id = p_role;
    scope = Core.Program_scope;
    fact_contract = Core.Role_fact_contract [program_role_fact.Core.fact_id];
    eligible_fulfillment = Core.role_fulfillment_of_string "compact.program";
  } in
  let template_one_role = {
    Core.role_id = t1_role;
    scope = Core.Item_template_scope template1;
    fact_contract = Core.Role_fact_contract [template_role_fact.Core.fact_id];
    eligible_fulfillment = Core.role_fulfillment_of_string "compact.template.one";
  } in
  let template_two_role = {
    Core.role_id = t2_role;
    scope = Core.Item_template_scope template2;
    fact_contract = Core.Role_fact_contract [];
    eligible_fulfillment = Core.role_fulfillment_of_string "compact.template.two";
  } in
  let branch = {
    Core.branch_id = bid ("compact-branch-" ^ tag);
    branch_subject = p_anchor;
    outcome_branches = [
      (Core.Success, Core.Continue_to p_action);
      (Core.Failure, Core.Stop);
      (Core.Uncertain, Core.Stop);
      (Core.Cancelled, Core.Stop);
    ];
  } in
  {
    Core.program_id = pid ("compact-program-" ^ tag);
    core_version = version "0.1.0";
    input_facts = [input];
    entry_guards = [{ Core.fact_id = input.Core.fact_id;
                      operator = Core.Equals;
                      expected = Core.String_value "ready" }];
    entry_origin = Some p_anchor;
    success_continuations = [
      { Core.from_origin = p_anchor; target = Core.Origin_target p_action };
      { Core.from_origin = p_action; target = Core.Program_complete };
    ];
    origin_sites = reverse_if [p_anchor_site; p_action_site; p_batch_site];
    branches = [branch];
    roles = [program_role];
    item_templates = reverse_if [
      { Core.item_template_id = template1;
        origin_sites = [t1_anchor_site; t1_batch_site];
        branches = [];
        roles = [template_one_role];
        objective = Core.Required_role t1_role };
      { Core.item_template_id = template2;
        origin_sites = [];
        branches = [];
        roles = [template_two_role];
        objective = Core.Required_role t2_role };
    ];
    capability_contracts = [contract "compact.action" "compact.action.contract"];
  }

let first_difference left right =
  let limit = min (String.length left) (String.length right) in
  let rec find index =
    if index = limit then
      if String.length left = String.length right then None
      else Some index
    else if left.[index] <> right.[index] then Some index
    else find (index + 1)
  in
  find 0

let hex_at value index =
  if index >= String.length value then "<end>"
  else Printf.sprintf "%02X" (Char.code value.[index])

let run_stage_a name program =
  let model = model_of_program name program in
  let refined = Refine.run model in
  check (name ^ " is root-discrete") (Partition.is_discrete refined.partition);
  match Encode.encode program model refined.partition with
  | Error _ ->
      Printf.eprintf "FAIL: %s leaf encoding rejected\n%!" name;
      exit 1
  | Ok leaf ->
      begin match Oracle.slow_oracle program with
      | Error _ ->
          Printf.eprintf "FAIL: %s slow oracle rejected tractable fixture\n%!" name;
          exit 1
      | Ok oracle ->
          begin match first_difference leaf.payload oracle.payload with
          | None -> check (name ^ " Enc_V2 payload parity") true
          | Some index ->
              Printf.eprintf
                "FAIL: %s first payload difference at byte %d: V3=%s oracle=%s\n%!"
                name index (hex_at leaf.payload index) (hex_at oracle.payload index);
              exit 1
          end;
          check (name ^ " digest parity") (leaf.digest = oracle.digest_string)
      end;
      leaf

let test_stage_a_fixtures () =
  List.iter (fun size ->
    ignore (run_stage_a ("chain-" ^ string_of_int size) (chain_program size))
  ) [1; 2; 3; 4; 5; 6];
  let chain3 = run_stage_a "chain-3-certificate-evidence" (chain_program 3) in
  check "chain-3 entry Origin receives exact V2-minimal label"
    (Format.OriginMap.find (oid "chain-origin-0") chain3.labels.origin_labels = 1);
  let compact = run_stage_a "compact-branch-batch-role"
      (compact_certificate_program "base" false) in
  check "compact certificate covers all family maps"
    (Format.OriginMap.cardinal compact.labels.origin_labels = 3 &&
     Format.FactMap.cardinal compact.labels.fact_labels = 4 &&
     Format.BatchMap.cardinal compact.labels.batch_labels = 2 &&
     Format.BranchMap.cardinal compact.labels.branch_labels = 1 &&
     Format.TemplateMap.cardinal compact.labels.template_labels = 2 &&
     Format.ScopedRoleMap.cardinal compact.labels.role_labels = 3)

let test_frozen_integer_boundaries () =
  let compare left right =
    Format.compare_bytes_lex_unsigned (Format.encode_int left)
      (Format.encode_int right)
  in
  check "frozen Enc_V2 orders 8 before 9" (compare 8 9 < 0);
  check "frozen Enc_V2 orders 10 before 9" (compare 9 10 > 0);
  check "frozen Enc_V2 orders 10 before 11" (compare 10 11 < 0);
  check "frozen Enc_V2 orders 11 before 12" (compare 11 12 < 0)

let test_stage_a_metamorphic () =
  let base = run_stage_a "metamorphic-base"
      (compact_certificate_program "base" false) in
  let renamed = run_stage_a "metamorphic-renamed"
      (compact_certificate_program "renamed" true) in
  check "renamed/reordered Stage-A payload parity" (base.payload = renamed.payload);
  check "renamed/reordered Stage-A digest parity" (base.digest = renamed.digest)

let test_stage_a_rejects_unproven_leaf () =
  let program = compact_certificate_program "reject" false in
  let model = model_of_program "reject fixture" program in
  let partition = Partition.create model in
  match Encode.encode program model partition with
  | Error Encode.Partition_not_stable -> check "unstable leaf rejected" true
  | Error _ -> check "unstable leaf rejected with expected error" false
  | Ok _ -> check "unstable leaf rejected" false

let () =
  test_stage_a_fixtures ();
  test_frozen_integer_boundaries ();
  test_stage_a_metamorphic ();
  test_stage_a_rejects_unproven_leaf ();
  Printf.printf "rocket-v3-stage-a: %d/%d checks passed\n%!"
    !tests_passed !tests_run
