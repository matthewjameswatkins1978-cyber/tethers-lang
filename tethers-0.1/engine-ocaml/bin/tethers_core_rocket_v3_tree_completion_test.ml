module Completion = Tethers_core_rocket_v3_tree_completion
module Core = Tethers_core
module Format = Tethers_core_canonical_v2_format

let checks = ref 0
let passed = ref 0
let partial_prefixes_checked = ref 0
let brute_force_completions_considered = ref 0
let feasibility_states = ref 0
let candidate_targets_considered = ref 0
let complete_permutations_enumerated = ref 0
let max_state_width = ref 0

let check name condition =
  incr checks;
  if condition then incr passed
  else begin
    Printf.eprintf "FAIL: %s\n%!" name;
    exit 1
  end

let oid value = Core.origin_id_of_string value
let pid value = Core.program_id_of_string value
let cid value = Core.capability_id_of_string value
let digest value = Core.capability_contract_digest_of_string value
let version value = Core.core_version_of_string value

let action id = Core.Action_origin {
  action_origin_id = id;
  capability_id = cid "tree-completion-capability";
  contract_digest = digest "tree-completion-digest";
  inputs = [];
  declared_facts = [];
  execution_constraints = [];
}

(* parent.(node) is the success target; -1 is ProgramComplete.  Dense node
   positions are fixture coordinates only.  The completion module never uses
   them as canonical evidence. *)
let program_of_parents parents =
  let size = Array.length parents in
  let ids = Array.init size (fun index -> oid ("completion-origin-" ^ string_of_int index)) in
  let sites = Array.to_list (Array.map action ids) in
  let continuations = Array.to_list (Array.mapi (fun source parent -> {
    Core.from_origin = ids.(source);
    target = if parent = -1 then Core.Program_complete
      else Core.Origin_target ids.(parent);
  }) parents) in
  {
    Core.program_id = pid ("tree-completion-program-" ^ string_of_int size);
    core_version = version "0.1.0";
    input_facts = [];
    entry_guards = [];
    entry_origin = Some ids.(0);
    success_continuations = List.rev continuations;
    origin_sites = List.rev sites;
    branches = [];
    roles = [];
    item_templates = [];
    capability_contracts = [{
      Core.capability_id = cid "tree-completion-capability";
      contract_digest = digest "tree-completion-digest";
      schema_description = "tree completion fixture";
    }];
  }

let labels_for_slots parents slots =
  let labels = Array.make (Array.length parents) 0 in
  for slot = 1 to Array.length parents do
    labels.(slots.(slot)) <- slot
  done;
  labels

let targets_for_slots parents slots =
  let labels = labels_for_slots parents slots in
  let targets = Array.make (Array.length parents + 1) Completion.Program_complete in
  for slot = 1 to Array.length parents do
    let node = slots.(slot) in
    targets.(slot) <- if parents.(node) = -1 then Completion.Program_complete
      else Completion.Origin_label labels.(parents.(node))
  done;
  targets

let payload_for_slots parents slots =
  let program = program_of_parents parents in
  let ids = Array.init (Array.length parents)
      (fun index -> oid ("completion-origin-" ^ string_of_int index)) in
  let origin_labels = ref Format.OriginMap.empty in
  for slot = 1 to Array.length parents do
    origin_labels := Format.OriginMap.add ids.(slots.(slot)) slot !origin_labels
  done;
  let assignment = {
    Format.origin_labels = !origin_labels;
    fact_labels = Format.FactMap.empty;
    branch_labels = Format.BranchMap.empty;
    batch_labels = Format.BatchMap.empty;
    template_labels = Format.TemplateMap.empty;
    role_labels = Format.ScopedRoleMap.empty;
  } in
  Format.encode_program assignment program

let first_difference left right =
  let limit = min (String.length left) (String.length right) in
  let rec loop index =
    if index = limit then
      if String.length left = String.length right then None else Some index
    else if left.[index] <> right.[index] then Some index
    else loop (index + 1)
  in
  loop 0

let target_at targets slot = Completion.target_to_string targets.(slot)

let historical_counterexample () =
  let parents = [| 6; 2; 3; 6; 5; 6; -1 |] in
  let tree = match Completion.make_tree ~parent:parents ~entry:0 with
    | Ok tree -> tree
    | Error message -> failwith message
  in
  let oracle = match Completion.brute_force_minimum tree ~entry_label:1 with
    | Some completion -> completion
    | None -> failwith "historical tree has no completion"
  in
  let historical_slots = [| -1; 0; 6; 5; 3; 4; 2; 1 |] in
  let historical_targets = targets_for_slots parents historical_slots in
  let oracle_vector = Array.to_list (Array.sub oracle.targets 1 (Array.length parents)) in
  let historical_vector = Array.to_list (Array.sub historical_targets 1 (Array.length parents)) in
  let first_vector_difference =
    let rec find slot =
      if slot > Array.length parents then None
      else if oracle.targets.(slot) <> historical_targets.(slot) then Some slot
      else find (slot + 1)
    in
    find 1
  in
  let historical_payload = payload_for_slots parents historical_slots in
  let oracle_payload = payload_for_slots parents oracle.slot_nodes in
  begin match first_vector_difference with
  | Some slot ->
      Printf.printf
        "historical-B3 tree=[6,2,3,6,5,6,-1] rank_vector=[%s] oracle_vector=[%s] first_continuation_slot=%d rank_target=%s oracle_target=%s\n%!"
        (String.concat "," (List.map Completion.target_to_string historical_vector))
        (String.concat "," (List.map Completion.target_to_string oracle_vector))
        slot (target_at historical_targets slot) (target_at oracle.targets slot)
  | None -> failwith "historical candidate unexpectedly equals oracle"
  end;
  begin match first_difference historical_payload oracle_payload with
  | Some offset ->
      Printf.printf "historical-B3 first_payload_byte=%d rank=0x%02X oracle=0x%02X\n%!"
        offset (Char.code historical_payload.[offset]) (Char.code oracle_payload.[offset])
  | None -> failwith "historical payload unexpectedly equals oracle"
  end;
  check "historical rank candidate has target 6" (target_at historical_targets 7 = "6");
  check "historical oracle has target 5" (target_at oracle.targets 7 = "5");
  check "historical vector differs from oracle" (historical_vector <> oracle_vector);
  check "historical payload differs from oracle" (historical_payload <> oracle_payload)

let prefix tree_size targets = match Completion.make_prefix ~tree_size
    ~processed_slots:(List.length targets) targets with
  | Ok value -> value
  | Error message -> failwith message

let test_oracle_and_local_counterexample () =
  let parents = [| 1; 2; -1 |] in
  let tree = match Completion.make_tree ~parent:parents ~entry:0 with
    | Ok tree -> tree
    | Error message -> failwith message
  in
  let impossible = prefix 3 [Completion.Origin_label 2; Completion.Program_complete] in
  check "brute-force rejects impossible local-capacity prefix"
    (not (Completion.brute_force_completable tree ~entry_label:1 impossible));
  check "local capacity candidate exposes false positive"
    (Completion.local_capacity_candidate tree ~entry_label:1 impossible)

let permutations values =
  let rec insert value = function
    | [] -> [[value]]
    | head :: tail as list ->
        (value :: list) :: List.map (fun suffix -> head :: suffix) (insert value tail)
  in
  List.fold_left (fun all value ->
    List.concat_map (insert value) all
  ) [[]] values

let valid_parent_arrays size =
  let rec build node suffix =
    if node = size then
      if List.length (List.filter (( = ) (-1)) suffix) = 1 then [Array.of_list (List.rev suffix)]
      else []
    else
      List.concat_map (fun parent -> build (node + 1) (parent :: suffix))
        (List.init (size - node) (fun offset ->
           if offset = size - node - 1 then -1 else node + offset + 1))
  in
  build 0 []

let vector_prefix parents slots processed =
  let targets = targets_for_slots parents slots in
  prefix (Array.length parents)
    (List.init processed (fun index -> targets.(index + 1)))

let exhaustive_reachable_prefixes parents entries =
  let tree = match Completion.make_tree ~parent:parents ~entry:entries with
    | Ok tree -> tree
    | Error message -> failwith message
  in
  let size = Array.length parents in
  let values = List.init size (fun index -> index) in
  let all_slots = List.map (fun rest ->
    let slots = Array.make (size + 1) (-1) in
    slots.(1) <- entries;
    List.iteri (fun index node -> slots.(index + 2) <- node) rest;
    slots
  ) (permutations (List.filter (fun node -> node <> entries) values)) in
  List.iter (fun slots ->
    for processed = 0 to size do
      let p = vector_prefix parents slots processed in
      incr partial_prefixes_checked;
      max_state_width := max !max_state_width processed;
      candidate_targets_considered := !candidate_targets_considered + processed;
      let brute_force, stats = Completion.brute_force_completable_with_stats
          tree ~entry_label:1 p in
      brute_force_completions_considered :=
        !brute_force_completions_considered + stats.completions_considered;
      complete_permutations_enumerated :=
        !complete_permutations_enumerated + stats.completions_considered;
      incr feasibility_states;
      check "reachable prefix accepted by brute-force oracle" brute_force;
      check "reachable prefix accepted by local candidate"
        (Completion.local_capacity_candidate tree ~entry_label:1 p)
    done
  ) all_slots

let test_exhaustive_small_corpus () =
  Printf.printf "testing independent completion oracle on exhaustive reachable prefixes\n%!";
  List.iter (fun size ->
    List.iter (fun parents ->
      for entry = 0 to size - 1 do
        exhaustive_reachable_prefixes parents entry
      done
    ) (valid_parent_arrays size)
  ) [2; 3; 4; 5];
  exhaustive_reachable_prefixes [| 6; 2; 3; 6; 5; 6; -1 |] 0

let () =
  historical_counterexample ();
  test_oracle_and_local_counterexample ();
  test_exhaustive_small_corpus ();
  Printf.printf
    "rocket-v3-tree-completion: %d/%d checks passed; partial_prefixes_checked=%d brute_force_completions_considered=%d feasibility_states=%d isomorphism_checks=0 exact_tie_states=0 candidate_targets_considered=%d committed_targets=0 complete_permutations_enumerated=%d max_state_width=%d\n%!"
    !passed !checks !partial_prefixes_checked
    !brute_force_completions_considered !feasibility_states
    !candidate_targets_considered !complete_permutations_enumerated
    !max_state_width
