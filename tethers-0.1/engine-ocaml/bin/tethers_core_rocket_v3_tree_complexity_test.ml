module Complexity = Tethers_core_rocket_v3_tree_complexity
module Completion = Tethers_core_rocket_v3_tree_completion

let checks = ref 0
let passed = ref 0

let check name condition =
  incr checks;
  if condition then incr passed
  else begin
    Printf.eprintf "FAIL: %s\n%!" name;
    exit 1
  end

let make_prefix size targets =
  match Complexity.make_prefix ~size ~entry_label:1 targets with
  | Ok prefix -> prefix
  | Error message -> failwith message

let completion_prefix parents slots processed =
  let size = Array.length parents in
  let labels = Array.make size 0 in
  for slot = 1 to size do labels.(slots.(slot)) <- slot done;
  let targets = List.init processed (fun index ->
    let node = slots.(index + 1) in
    if parents.(node) = -1 then Complexity.Program_complete
    else Complexity.Origin_label labels.(parents.(node))) in
  make_prefix size targets

let b3a_prefix parents slots processed =
  let targets = List.init processed (fun index ->
    let node = slots.(index + 1) in
    if parents.(node) = -1 then Completion.Program_complete
    else Completion.Origin_label
      (let labels = Array.make (Array.length parents) 0 in
       for slot = 1 to Array.length parents do labels.(slots.(slot)) <- slot done;
       labels.(parents.(node)))) in
  match Completion.make_prefix ~tree_size:(Array.length parents)
      ~processed_slots:processed targets with
  | Ok prefix -> prefix
  | Error message -> failwith message

let test_forest_properties () =
  let prefix = make_prefix 5 [Complexity.Origin_label 3;
                             Complexity.Origin_label 4] in
  let analysis = Complexity.analyse_prefix prefix in
  check "acyclic prefix is spanning forest" analysis.acyclic;
  check "forest includes external root component" (analysis.component_count = 4);
  check "processed source prefix is counted" (analysis.specified_sources = 2);
  let self = make_prefix 3 [Complexity.Origin_label 1] in
  check "self edge is not a forest" (not (Complexity.analyse_prefix self).acyclic);
  let terminals = make_prefix 3 [Complexity.Program_complete;
                                 Complexity.Program_complete] in
  check "multiple external terminals are recorded"
    ((Complexity.analyse_prefix terminals).terminal_edges = 2)

let test_bounded_equivalence () =
  let fixtures = [
    [| 1; 2; -1 |], [0; 2];
    [| 3; 3; 3; -1 |], [0; 3];
    [| 2; 2; -1; 2; 3; 3 |], [0; 2];
    [| 6; 2; 3; 6; 5; 6; -1 |], [0; 6]
  ] in
  List.iter (fun (parents, entries) ->
    let size = Array.length parents in
    List.iter (fun entry ->
      let slots = Array.make (size + 1) (-1) in
      slots.(1) <- entry;
      let next_slot = ref 2 in
      for node = 0 to size - 1 do
        if node <> entry then begin
          slots.(!next_slot) <- node;
          incr next_slot
        end
      done;
      for processed = 0 to size do
      let tree = match Completion.make_tree ~parent:parents ~entry with
        | Ok tree -> tree
        | Error message -> failwith message
      in
      let rocket_prefix = completion_prefix parents slots processed in
      let b3a = b3a_prefix parents slots processed in
      let expected = Completion.brute_force_completable tree
          ~entry_label:1 b3a in
      let actual = Complexity.spanning_forest_witness
          ~semantic_parent:parents ~semantic_entry:entry ~entry_label:1
          ~prefix:rocket_prefix in
      check "forest witness agrees with B3A on bounded reachable prefix"
        (expected = actual)
      done
    ) entries
  ) fixtures

let test_normalisation () =
  let parent = [| 1; -1; 3; -1; 3 |] in
  let result = match Complexity.relabel_non_roots_first ~parent with
    | Ok value -> value
    | Error message -> failwith message
  in
  let analysis = Complexity.analyse_prefix
      (make_prefix (Array.length parent) result.targets) in
  check "arbitrary forest non-roots occupy the prefix"
    (result.processed_sources = 3 && analysis.specified_sources = 3);
  check "normalised arbitrary forest remains acyclic" analysis.acyclic

let test_storage_permutation () =
  let original = [| 1; 2; -1 |] in
  let permutation = [| 2; 0; 1 |] in
  let inverse = Array.make 3 0 in
  Array.iteri (fun index old_node -> inverse.(old_node) <- index) permutation;
  let renamed = Array.init 3 (fun index ->
    let old_parent = original.(permutation.(index)) in
    if old_parent = -1 then -1 else inverse.(old_parent)) in
  let original_forest = match Complexity.relabel_non_roots_first ~parent:original with
    | Ok value -> value
    | Error message -> failwith message
  in
  let renamed_forest = match Complexity.relabel_non_roots_first ~parent:renamed with
    | Ok value -> value
    | Error message -> failwith message
  in
  let original_analysis = Complexity.analyse_prefix
      (make_prefix 3 original_forest.targets) in
  let renamed_analysis = Complexity.analyse_prefix
      (make_prefix 3 renamed_forest.targets) in
  check "semantic storage permutation preserves forest component count"
    (original_analysis.component_count = renamed_analysis.component_count);
  check "semantic storage permutation preserves forest acyclicity"
    (original_analysis.acyclic = renamed_analysis.acyclic)

let () =
  test_forest_properties ();
  test_bounded_equivalence ();
  test_normalisation ();
  test_storage_permutation ();
  Printf.printf "rocket-v3-tree-complexity: %d/%d checks passed\n%!"
    !passed !checks
