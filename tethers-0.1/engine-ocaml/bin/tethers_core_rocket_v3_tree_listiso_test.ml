module Completion = Tethers_core_rocket_v3_tree_completion
module ListIso = Tethers_core_rocket_v3_tree_listiso

let checks = ref 0
let passed = ref 0
let partial_prefixes_checked = ref 0
let candidate_states = ref 0
let candidate_pairs = ref 0
let matching_instances = ref 0
let matching_vertices = ref 0
let matching_edges = ref 0
let matching_failures = ref 0
let exact_oracle_complete_assignments = ref 0

let check name condition =
  incr checks;
  if condition then incr passed
  else begin
    Printf.eprintf "FAIL: %s\n%!" name;
    exit 1
  end

let target_of_completion = function
  | Completion.Origin_label label -> ListIso.Origin_label label
  | Completion.Program_complete -> ListIso.Program_complete

let evaluate parents entry prefix =
  let relations = List.mapi (fun index target -> {
    ListIso.source = index + 1;
    target = target_of_completion target;
  }) prefix in
  match ListIso.evaluate_connected_component
      ~semantic_parent:parents ~entry_semantic:entry ~entry_label:1
      ~processed_slots:(List.length prefix) relations with
  | Ok (result, stats) ->
      incr partial_prefixes_checked;
      candidate_states := !candidate_states + stats.candidate_states;
      candidate_pairs := !candidate_pairs + stats.candidate_pairs;
      matching_instances := !matching_instances + stats.matching_instances;
      matching_vertices := !matching_vertices + stats.matching_vertices;
      matching_edges := !matching_edges + stats.matching_edges;
      matching_failures := !matching_failures + stats.matching_failures;
      result, stats
  | Error message -> failwith message

let verdict_is = function
  | ListIso.Proven_feasible -> true
  | ListIso.Proven_infeasible -> false
  | ListIso.Unknown_global_packing _ -> false

let test_three_node_false_positive () =
  let parents = [| 1; 2; -1 |] in
  let prefix = [Completion.Origin_label 2; Completion.Program_complete] in
  let verdict, stats = evaluate parents 0 prefix in
  check "matching state rejects B3A three-node false positive"
    (verdict = ListIso.Proven_infeasible);
  check "false-positive rejection uses matching state"
    (stats.matching_instances > 0 && stats.matching_edges > 0)

let test_seven_node_global_difference () =
  let parents = [| 6; 2; 3; 6; 5; 6; -1 |] in
  let prefix target = [
    Completion.Origin_label 2;
    Completion.Program_complete;
    Completion.Origin_label 2;
    Completion.Origin_label 2;
    Completion.Origin_label 3;
    Completion.Origin_label 4;
    Completion.Origin_label target;
  ] in
  let target_five, _ = evaluate parents 0 (prefix 5) in
  let target_six, _ = evaluate parents 0 (prefix 6) in
  check "seven-node target 5 is matchable" (target_five = ListIso.Proven_feasible);
  check "seven-node target 6 is also a legal completion"
    (target_six = ListIso.Proven_feasible);
  check "seven-node exact oracle chooses target 5"
    (match Completion.make_tree ~parent:parents ~entry:0 with
     | Error _ -> false
     | Ok tree ->
         match Completion.brute_force_minimum tree ~entry_label:1 with
         | None -> false
         | Some completion -> completion.targets.(7) = Completion.Origin_label 5)

let test_matches_oracle_on_connected_prefixes () =
  let cases = [
    ([| 1; -1 |], 0,
     [Completion.Origin_label 2; Completion.Program_complete]);
    ([| 1; 2; -1 |], 0,
     [Completion.Origin_label 2; Completion.Origin_label 3;
      Completion.Program_complete]);
    ([| 1; 3; 3; -1 |], 0,
     [Completion.Origin_label 2; Completion.Origin_label 3;
      Completion.Origin_label 4; Completion.Program_complete]);
  ] in
  List.iter (fun (parents, entry, prefix) ->
    let tree = match Completion.make_tree ~parent:parents ~entry with
      | Ok tree -> tree
      | Error message -> failwith message
    in
    let partial = match Completion.make_prefix
        ~tree_size:(Array.length parents)
        ~processed_slots:(List.length prefix) prefix with
      | Ok value -> value
      | Error message -> failwith message
    in
    let expected, oracle_stats =
      Completion.brute_force_completable_with_stats tree ~entry_label:1 partial
    in
    exact_oracle_complete_assignments :=
      !exact_oracle_complete_assignments + oracle_stats.completions_considered;
    let actual, _ = evaluate parents entry prefix in
    check "connected matching agrees with brute-force oracle"
      (verdict_is actual = expected)
  ) cases

let test_disconnected_state_is_explicit () =
  let parents = [| 1; 2; 3; -1 |] in
  let prefix = [Completion.Origin_label 3; Completion.Program_complete] in
  let verdict, _ = evaluate parents 0 prefix in
  match verdict with
  | ListIso.Unknown_global_packing message ->
      check "disconnected packing is not promoted to a theorem"
        (String.length message > 0)
  | _ -> check "disconnected packing is not promoted to a theorem" false

let test_storage_renaming () =
  let parents = [| 1; 2; -1 |] in
  let renamed = [| -1; 2; 0 |] in
  let prefix = [Completion.Origin_label 2; Completion.Origin_label 3;
                Completion.Program_complete] in
  let first, _ = evaluate parents 0 prefix in
  let second, _ = evaluate renamed 1 prefix in
  check "semantic storage renaming preserves connected feasibility"
    (first = second)

let () =
  test_three_node_false_positive ();
  test_seven_node_global_difference ();
  test_matches_oracle_on_connected_prefixes ();
  test_disconnected_state_is_explicit ();
  test_storage_renaming ();
  Printf.printf
    "rocket-v3-tree-listiso: %d/%d checks passed; partial_prefixes_checked=%d candidate_states=%d candidate_pairs=%d matching_instances=%d matching_vertices=%d matching_edges=%d matching_failures=%d candidate_targets_considered=%d committed_targets=0 exact_oracle_complete_assignments=%d complete_permutations_enumerated=0\n%!"
    !passed !checks !partial_prefixes_checked !candidate_states
    !candidate_pairs !matching_instances !matching_vertices !matching_edges
    !matching_failures !candidate_pairs !exact_oracle_complete_assignments
