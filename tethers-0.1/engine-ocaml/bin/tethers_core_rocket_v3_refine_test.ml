module Core = Tethers_core
module Model = Tethers_core_rocket_v3_model
module Partition = Tethers_core_rocket_v3_partition
module Refine = Tethers_core_rocket_v3_refine

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
let cid value = Core.capability_id_of_string value
let pid value = Core.program_id_of_string value
let tid value = Core.item_template_id_of_string value
let rid value = Core.role_id_of_string value
let fid value = Core.fact_id_of_string value
let digest value = Core.capability_contract_digest_of_string value
let version value = Core.core_version_of_string value
let input_name value = Core.capability_input_name_of_string value

let model_of_program name program =
  match Model.build program with
  | Ok model -> model
  | Error _ ->
      Printf.eprintf "FAIL: %s (valid fixture rejected)\n%!" name;
      exit 1

let empty_program () = {
  Core.program_id = pid "empty-program";
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

let chain_program size reverse =
  let origin_ids = List.init size (fun index -> oid ("chain-origin-" ^ string_of_int index)) in
  let actions = List.map (fun origin_id ->
    Core.Action_origin {
      action_origin_id = origin_id;
      capability_id = cid "chain.capability";
      contract_digest = digest "chain.contract";
      inputs = [];
      declared_facts = [];
      execution_constraints = [];
    }
  ) origin_ids in
  let continuations = List.mapi (fun index from_origin ->
    let target = if index + 1 = size then Core.Program_complete
      else Core.Origin_target (List.nth origin_ids (index + 1))
    in
    { Core.from_origin; target }
  ) origin_ids in
  let reorder values = if reverse then List.rev values else values in
  {
    (empty_program ()) with
    program_id = pid "chain-program";
    entry_origin = Some (List.hd origin_ids);
    success_continuations = reorder continuations;
    origin_sites = reorder actions;
    capability_contracts = [{
      Core.capability_id = cid "chain.capability";
      contract_digest = digest "chain.contract";
      schema_description = "neutral schema";
    }];
  }

let symmetric_twins_program () =
  let template_id = tid "symmetric-template" in
  let action capability_origin = Core.Action_origin {
    action_origin_id = oid capability_origin;
    capability_id = cid "symmetric.capability";
    contract_digest = digest "symmetric.contract";
    inputs = [];
    declared_facts = [];
    execution_constraints = [];
  } in
  let role = {
    Core.role_id = rid "symmetric-role";
    scope = Core.Item_template_scope template_id;
    fact_contract = Core.Role_fact_contract [];
    eligible_fulfillment = Core.role_fulfillment_of_string "symmetric.fulfilment";
  } in
  let template = {
    Core.item_template_id = template_id;
    origin_sites = [action "twin-a"; action "twin-b"];
    branches = [];
    roles = [role];
    objective = Core.Required_role role.role_id;
  } in
  {
    (empty_program ()) with
    program_id = pid "symmetric-program";
    item_templates = [template];
    capability_contracts = [{
      Core.capability_id = cid "symmetric.capability";
      contract_digest = digest "symmetric.contract";
      schema_description = "neutral schema";
    }];
  }

let binding_payload_program () =
  let template_id = tid "binding-template" in
  let role_id = rid "binding-role" in
  let fact = {
    Core.fact_id = fid "binding-fact";
    schema_description = "neutral fact schema";
    provenance = Core.Role_proxy role_id;
  } in
  let anchor = Core.Anchor_origin {
    anchor_origin_id = oid "binding-anchor";
    event_name = "binding.event";
    declared_facts = [fact];
  } in
  let action origin_id name = Core.Action_origin {
    action_origin_id = oid origin_id;
    capability_id = cid "binding.capability";
    contract_digest = digest "binding.contract";
    inputs = [{
      Core.input_name = input_name name;
      binding = Core.Fact_through_role (fact.fact_id, role_id);
    }];
    declared_facts = [];
    execution_constraints = [];
  } in
  let role = {
    Core.role_id = role_id;
    scope = Core.Item_template_scope template_id;
    fact_contract = Core.Role_fact_contract [fact.fact_id];
    eligible_fulfillment = Core.role_fulfillment_of_string "binding.fulfilment";
  } in
  let template = {
    Core.item_template_id = template_id;
    origin_sites = [anchor; action "binding-action-a" "alpha"; action "binding-action-b" "beta"];
    branches = [];
    roles = [role];
    objective = Core.Required_role role_id;
  } in
  {
    (empty_program ()) with
    program_id = pid "binding-program";
    item_templates = [template];
    capability_contracts = [{
      Core.capability_id = cid "binding.capability";
      contract_digest = digest "binding.contract";
      schema_description = "neutral schema";
    }];
  }

let starts_with prefix value =
  let prefix_length = String.length prefix in
  String.length value >= prefix_length &&
  String.sub value 0 prefix_length = prefix

let action_vertices model =
  List.init (Model.vertex_count model) Fun.id
  |> List.filter (fun vertex ->
    match Model.vertex_kind model vertex with
    | Model.Anonymous Model.Origin ->
        starts_with "origin:action:" (Model.vertex_scalar model vertex)
    | _ -> false)

let same_partition left right =
  Array.length left = Array.length right &&
  let equivalent a b =
    Array.for_all (fun i ->
      Array.for_all (fun j -> (a.(i) = a.(j)) = (b.(i) = b.(j)))
        (Array.init (Array.length a) Fun.id)
    ) (Array.init (Array.length a) Fun.id)
  in
  equivalent left right

let model_kind_key = function
  | Model.Anonymous Model.Origin -> "anonymous:Origin"
  | Model.Anonymous Model.Fact -> "anonymous:Fact"
  | Model.Anonymous Model.Branch -> "anonymous:Branch"
  | Model.Anonymous Model.Batch -> "anonymous:Batch"
  | Model.Anonymous Model.ItemTemplate -> "anonymous:ItemTemplate"
  | Model.Anonymous Model.ScopedRole -> "anonymous:ScopedRole"
  | Model.ProgramRoot -> "sentinel:ProgramRoot"
  | Model.ProgramScope -> "sentinel:ProgramScope"
  | Model.ProgramComplete -> "terminal:ProgramComplete"
  | Model.BranchStop -> "terminal:BranchStop"

let model_initial_key model vertex =
  model_kind_key (Model.vertex_kind model vertex) ^ "\000" ^
  Model.vertex_scalar model vertex

let outcome_key = function
  | Core.Success -> "success"
  | Core.Failure -> "failure"
  | Core.Uncertain -> "uncertain"
  | Core.Cancelled -> "cancelled"

let binding_key = function
  | Model.Binding_fact_from_origin -> "Fact_from_origin"
  | Model.Binding_fact_through_role -> "Fact_through_role"
  | Model.Binding_anchor_value -> "Anchor_value"
  | Model.Binding_batch_item_context -> "Batch_item_context"

let discriminator_key = function
  | Model.Discriminator_none -> "none"
  | Model.Origin_anchor -> "origin:anchor"
  | Model.Origin_action -> "origin:action"
  | Model.Origin_together -> "origin:together"
  | Model.Batch_site_aggregate -> "batch-site:aggregate"
  | Model.Provenance_origin -> "provenance:origin"
  | Model.Provenance_role_proxy -> "provenance:role-proxy"
  | Model.Entry_guard -> "entry-guard"
  | Model.Action_binding binding -> "action-binding:" ^ binding_key binding
  | Model.Together_member -> "together-member"
  | Model.Branch_outcome outcome -> "branch-outcome:" ^ outcome_key outcome
  | Model.Branch_continue_to -> "branch:continue-to"
  | Model.Branch_stop_target -> "branch:stop"
  | Model.Success_continuation -> "success-continuation"
  | Model.Success_program_complete -> "success:program-complete"
  | Model.Role_contract -> "role:fact-contract"
  | Model.Role_program_scope -> "role:program-scope"
  | Model.Role_item_template_scope -> "role:item-template-scope"
  | Model.Template_membership -> "template:membership"
  | Model.Template_batch_membership -> "template:batch-membership"
  | Model.Template_objective -> "template:objective"
  | Model.Program_input -> "program:input"
  | Model.Program_origin_membership -> "program:origin-membership"
  | Model.Program_batch_membership -> "program:batch-membership"
  | Model.Program_branch_membership -> "program:branch-membership"
  | Model.Program_role_membership -> "program:role-membership"
  | Model.Program_template_membership -> "program:template-membership"
  | Model.Fact_program_scope -> "fact:program-scope"
  | Model.Fact_template_scope -> "fact:template-scope"
  | Model.Fact_origin_scope -> "fact:origin-scope"
  | Model.Fact_batch_scope -> "fact:batch-scope"
  | Model.Batch_template_context -> "batch:template-context"

let slow_channel_key direction edge target_cell =
  let direction_key = match direction with
    | `Forward -> "forward"
    | `Reverse -> "reverse"
  in
  String.concat "|" [
    direction_key;
    Model.relation_name edge.Model.relation;
    discriminator_key edge.Model.discriminator;
    edge.Model.payload;
    string_of_int target_cell;
  ]

let slow_next_classes model classes =
  let vertex_count = Model.vertex_count model in
  let signature vertex =
    let counts = Hashtbl.create 16 in
    let add direction edge target_cell =
      let key = slow_channel_key direction edge target_cell in
      let count = match Hashtbl.find_opt counts key with
        | Some count -> count
        | None -> 0
      in
      Hashtbl.replace counts key (count + 1)
    in
    List.iter (fun edge -> add `Forward edge classes.(edge.Model.target))
      (Model.forward_edges model vertex);
    List.iter (fun edge -> add `Reverse edge classes.(edge.Model.target))
      (Model.reverse_edges model vertex);
    let entries = Hashtbl.fold (fun key count acc ->
      (key, count) :: acc) counts []
      |> List.sort (fun (left, _) (right, _) -> String.compare left right)
      |> List.map (fun (key, count) -> key ^ "=" ^ string_of_int count) in
    model_initial_key model vertex ^ "{" ^ String.concat ";" entries ^ "}"
  in
  let signatures = Array.init vertex_count signature in
  let unique = Array.to_list signatures |> List.sort_uniq String.compare in
  let ranks = Hashtbl.create (List.length unique) in
  List.iteri (fun index key -> Hashtbl.add ranks key index) unique;
  Array.map (fun key -> Hashtbl.find ranks key) signatures

let slow_refine model =
  let initial = Array.init (Model.vertex_count model) (model_initial_key model) in
  let unique = Array.to_list initial |> List.sort_uniq String.compare in
  let ranks = Hashtbl.create (List.length unique) in
  List.iteri (fun index key -> Hashtbl.add ranks key index) unique;
  let initial = Array.map (fun key -> Hashtbl.find ranks key) initial in
  let rec loop classes rounds =
    let next = slow_next_classes model classes in
    if same_partition classes next then next
    else if rounds > Model.vertex_count model + 2 then begin
      Printf.eprintf "FAIL: slow reference did not converge\n%!";
      exit 1
    end else loop next (rounds + 1)
  in
  loop initial 0

let assert_matches_slow name model partition =
  let slow = slow_refine model in
  for left = 0 to Model.vertex_count model - 1 do
    for right = 0 to Model.vertex_count model - 1 do
      check (name ^ " slow equivalence")
        (Partition.same_cell partition left right = (slow.(left) = slow.(right)))
    done
  done

let stats_equal left right =
  left.Refine.relation_visits = right.Refine.relation_visits &&
  left.splitter_pops = right.splitter_pops &&
  left.cell_splits = right.cell_splits &&
  left.max_worklist = right.max_worklist &&
  left.final_cell_count = right.final_cell_count

let final_signature model partition vertex =
  let counts = Hashtbl.create 16 in
  let add direction edge =
    let key = slow_channel_key direction edge
        (Partition.cell_of_vertex partition edge.Model.target) in
    let count = match Hashtbl.find_opt counts key with
      | Some count -> count
      | None -> 0
    in
    Hashtbl.replace counts key (count + 1)
  in
  List.iter (add `Forward) (Model.forward_edges model vertex);
  List.iter (add `Reverse) (Model.reverse_edges model vertex);
  Hashtbl.fold (fun key count acc -> (key, count) :: acc) counts []
  |> List.sort (fun (left, _) (right, _) -> String.compare left right)

let assert_equitable name model partition =
  let vertices = List.init (Model.vertex_count model) Fun.id in
  List.iter (fun left ->
    List.iter (fun right ->
      if Partition.same_cell partition left right then
        check (name ^ " equitable typed counts")
          (final_signature model partition left =
           final_signature model partition right)
    ) vertices
  ) vertices

let test_initial_and_chains () =
  let count_1000 = ref None in
  List.iter (fun size ->
    let model = model_of_program ("chain " ^ string_of_int size)
        (chain_program size (size mod 2 = 0)) in
    let initial = Partition.create model in
    let actions = action_vertices model in
    let initial_keys = List.map (Partition.initial_key initial) actions in
    check ("chain " ^ string_of_int size ^ " Actions share initial key")
      (List.length (List.sort_uniq String.compare initial_keys) = 1);
    let result = Refine.refine initial in
    check ("chain " ^ string_of_int size ^ " is stable")
      (Partition.is_stable result.partition);
    check ("chain " ^ string_of_int size ^ " is discrete")
      (Partition.is_discrete result.partition);
    assert_equitable ("chain " ^ string_of_int size) model result.partition;
    List.iter (fun action ->
      check ("chain Action singleton " ^ string_of_int size)
        (Partition.cell_size result.partition
           (Partition.cell_of_vertex result.partition action) = 1)
    ) actions;
    if size = 1000 then count_1000 := Some result.stats
  ) [1; 10; 50; 100; 250; 500; 1000];
  match !count_1000 with
  | None -> assert false
  | Some stats ->
      Printf.printf
        "rocket-v3-refine chain-1000: relation_visits=%d splitter_pops=%d cell_splits=%d max_worklist=%d final_cells=%d\n%!"
        stats.relation_visits stats.splitter_pops stats.cell_splits
        stats.max_worklist stats.final_cell_count;
      check "chain 1000 refinement work is bounded" (stats.relation_visits < 100000)

let test_symmetric_twins () =
  let model = model_of_program "symmetric twins" (symmetric_twins_program ()) in
  let result = Refine.run model in
  let actions = action_vertices model in
  check "symmetric fixture has two Actions" (List.length actions = 2);
  let first, second = match actions with
    | [first; second] -> first, second
    | _ -> assert false
  in
  check "symmetric twins share initial key"
    (Partition.initial_key (Partition.create model) first =
     Partition.initial_key (Partition.create model) second);
  check "symmetric twins remain equivalent"
    (Partition.same_cell result.partition first second);
  check "symmetric fixture remains non-discrete" (not (Partition.is_discrete result.partition));
  check "symmetric result is stable" (Partition.is_stable result.partition);
  assert_equitable "symmetric" model result.partition

let test_typed_channel_dimensions () =
  let model = model_of_program "binding payload" (binding_payload_program ()) in
  let initial = Partition.create model in
  let actions = action_vertices model in
  check "binding fixture has two same-scalar Actions" (List.length actions = 2);
  let first, second = match actions with
    | [first; second] -> first, second
    | _ -> assert false
  in
  check "binding Actions initially share semantic key"
    (Partition.initial_key initial first = Partition.initial_key initial second);
  let result = Refine.refine initial in
  check "typed binding channel distinguishes payload-bearing Actions"
    (not (Partition.same_cell result.partition first second));
  let forward = Model.forward_edges model first in
  let reverse =
    List.init (Model.vertex_count model) Fun.id
    |> List.exists (fun vertex ->
      List.exists (fun edge ->
        edge.Model.target = first && edge.Model.relation = Model.Rel_action_input_role)
        (Model.reverse_edges model vertex))
  in
  check "forward and inverse channels are separately exposed"
    (List.exists (fun edge -> edge.Model.relation = Model.Rel_action_input_role) forward && reverse)

let test_storage_order_invariance () =
  let first_model = model_of_program "ordered chain" (chain_program 32 false) in
  let second_model = model_of_program "reversed chain" (chain_program 32 true) in
  let first = Refine.run first_model in
  let second = Refine.run second_model in
  check "reversed representation has identical partition evidence"
    (Partition.evidence first.partition = Partition.evidence second.partition);
  check "reversed representation has identical statistics"
    (stats_equal first.stats second.stats)

let test_repeatability_and_reference_corpus () =
  let corpus =
    [model_of_program "empty" (empty_program ())] @
    List.init 8 (fun index ->
      model_of_program ("corpus chain " ^ string_of_int (index + 1))
        (chain_program (index + 1) (index mod 2 = 0))) @
    [model_of_program "corpus symmetric" (symmetric_twins_program ());
     model_of_program "corpus bindings" (binding_payload_program ())]
  in
  List.iteri (fun index model ->
    let first = Refine.run model in
    let second = Refine.run model in
    check ("corpus repeat evidence " ^ string_of_int index)
      (Partition.evidence first.partition = Partition.evidence second.partition);
    check ("corpus repeat stats " ^ string_of_int index)
      (stats_equal first.stats second.stats);
    assert_equitable ("corpus " ^ string_of_int index) model first.partition;
    assert_matches_slow ("corpus " ^ string_of_int index) model first.partition
  ) corpus

let () =
  test_initial_and_chains ();
  test_symmetric_twins ();
  test_typed_channel_dimensions ();
  test_storage_order_invariance ();
  test_repeatability_and_reference_corpus ();
  Printf.printf "rocket-v3-refine: %d/%d checks passed\n%!"
    !tests_passed !tests_run
