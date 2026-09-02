type target =
  | Origin_label of int
  | Program_complete

type partial_parent_vector = {
  processed_slots : int;
  targets : target option array;
}

type rooted_tree = {
  parent : int array;
  entry : int;
}

type completion = {
  targets : target array;
  slot_nodes : int array;
}

type oracle_stats = {
  completions_considered : int;
}

let target_to_string = function
  | Origin_label label -> string_of_int label
  | Program_complete -> "Complete"

let make_tree ~parent ~entry =
  let size = Array.length parent in
  if size = 0 then Error "empty rooted tree"
  else if entry < 0 || entry >= size then Error "entry is outside the tree"
  else if Array.exists (fun p -> p < -1 || p >= size) parent then
    Error "parent is outside the tree"
  else begin
    let visiting = Array.make size false in
    let finished = Array.make size false in
    let cycle = ref false in
    let rec visit node =
      if visiting.(node) then cycle := true
      else if not finished.(node) then begin
        visiting.(node) <- true;
        begin match parent.(node) with
        | -1 -> ()
        | next -> visit next
        end;
        visiting.(node) <- false;
        finished.(node) <- true
      end
    in
    for node = 0 to size - 1 do visit node done;
    if !cycle then Error "success relation contains a cycle"
    else Ok { parent = Array.copy parent; entry }
  end

let make_prefix ~tree_size ~processed_slots targets =
  if tree_size < 1 then Error "empty parent-vector domain"
  else if processed_slots < 0 || processed_slots > tree_size then
    Error "processed slot count is outside the domain"
  else if List.length targets <> processed_slots then
    Error "prefix length does not equal processed slot count"
  else begin
    let values = Array.make (tree_size + 1) None in
    List.iteri (fun index target -> values.(index + 1) <- Some target) targets;
    Ok { processed_slots; targets = values }
  end

let valid_entry_label tree entry_label =
  entry_label >= 1 && entry_label <= Array.length tree.parent

let valid_target tree source = function
  | Program_complete -> true
  | Origin_label label ->
      label >= 1 && label <= Array.length tree.parent && label <> source

let valid_prefix tree entry_label (prefix : partial_parent_vector) =
  let size = Array.length tree.parent in
  valid_entry_label tree entry_label &&
  Array.length prefix.targets = size + 1 &&
  prefix.processed_slots >= 0 && prefix.processed_slots <= size &&
  (let valid = ref true in
   for source = 1 to size do
     if source <= prefix.processed_slots then
       begin match prefix.targets.(source) with
       | Some target when valid_target tree source target -> ()
       | _ -> valid := false
       end
     else if prefix.targets.(source) <> None then valid := false
   done;
   !valid)

let target_for_assignment tree labels source_node =
  match tree.parent.(source_node) with
  | -1 -> Program_complete
  | parent_node -> Origin_label labels.(parent_node)

let matches_prefix prefix targets =
  let valid = ref true in
  for source = 1 to prefix.processed_slots do
    match prefix.targets.(source) with
    | Some expected when expected = targets.(source) -> ()
    | _ -> valid := false
  done;
  !valid

let brute_force_completable_with_stats tree ~entry_label prefix =
  if not (valid_prefix tree entry_label prefix) then
    false, { completions_considered = 0 }
  else begin
    let size = Array.length tree.parent in
    let slot_nodes = Array.make (size + 1) (-1) in
    let labels = Array.make size 0 in
    slot_nodes.(entry_label) <- tree.entry;
    labels.(tree.entry) <- entry_label;
    let used = Array.make size false in
    used.(tree.entry) <- true;
    let considered = ref 0 in
    let found = ref false in
    let targets = Array.make (size + 1) Program_complete in
    let rec assign_next source =
      if !found then ()
      else if source > size then begin
        incr considered;
        for slot = 1 to size do
          targets.(slot) <- target_for_assignment tree labels slot_nodes.(slot)
        done;
        if matches_prefix prefix targets then found := true
      end
      else if slot_nodes.(source) <> -1 then assign_next (source + 1)
      else begin
        for node = 0 to size - 1 do
          if not used.(node) then begin
            used.(node) <- true;
            slot_nodes.(source) <- node;
            labels.(node) <- source;
            assign_next (source + 1);
            labels.(node) <- 0;
            slot_nodes.(source) <- -1;
            used.(node) <- false
          end
        done
      end
    in
    assign_next 1;
    !found, { completions_considered = !considered }
  end

let brute_force_completable tree ~entry_label prefix =
  fst (brute_force_completable_with_stats tree ~entry_label prefix)

let compare_target left right =
  let bytes target =
    match target with
    | Origin_label label -> "0" ^ string_of_int label ^ ";"
    | Program_complete -> "1;"
  in
  Stdlib.compare (bytes left) (bytes right)

let compare_completion left right size =
  let rec loop source =
    if source > size then 0
    else
      let difference = compare_target left.targets.(source) right.targets.(source) in
      if difference <> 0 then difference else loop (source + 1)
  in
  loop 1

let brute_force_minimum tree ~entry_label =
  if not (valid_entry_label tree entry_label) then None
  else begin
    let size = Array.length tree.parent in
    let slot_nodes = Array.make (size + 1) (-1) in
    let labels = Array.make size 0 in
    slot_nodes.(entry_label) <- tree.entry;
    labels.(tree.entry) <- entry_label;
    let used = Array.make size false in
    used.(tree.entry) <- true;
    let best = ref None in
    let consider () =
      let targets = Array.make (size + 1) Program_complete in
      for slot = 1 to size do
        targets.(slot) <- target_for_assignment tree labels slot_nodes.(slot)
      done;
      let candidate = { targets; slot_nodes = Array.copy slot_nodes } in
      match !best with
      | None -> best := Some candidate
      | Some current when compare_completion candidate current size < 0 ->
          best := Some candidate
      | Some _ -> ()
    in
    let rec assign_next source =
      if source > size then consider ()
      else if slot_nodes.(source) <> -1 then assign_next (source + 1)
      else begin
        for node = 0 to size - 1 do
          if not used.(node) then begin
            used.(node) <- true;
            slot_nodes.(source) <- node;
            labels.(node) <- source;
            assign_next (source + 1);
            labels.(node) <- 0;
            slot_nodes.(source) <- -1;
            used.(node) <- false
          end
        done
      end
    in
    assign_next 1;
    !best
  end

(* This is intentionally only a necessary-condition candidate.  It captures
   local forest validity, root/degree capacity and the fixed entry's terminal
   kind.  It is not called Completable: the B3A task requires the independent
   brute-force oracle to disprove any shortcut before it can become a theorem. *)
let local_capacity_candidate tree ~entry_label prefix =
  if not (valid_prefix tree entry_label prefix) then false
  else begin
    let size = Array.length tree.parent in
    let fixed_children = Array.make (size + 1) 0 in
    let semantic_degree = Array.make (size + 1) 0 in
    let next = Array.make (size + 1) 0 in
    let valid = ref true in
    let root_degree = ref 0 in
    let max_degree = ref 0 in
    for node = 0 to size - 1 do
      let parent = tree.parent.(node) in
      if parent = -1 then begin
        incr root_degree;
        semantic_degree.(0) <- semantic_degree.(0) + 1
      end
      else begin
        semantic_degree.(parent + 1) <- semantic_degree.(parent + 1) + 1;
        max_degree := max !max_degree semantic_degree.(parent + 1)
      end
    done;
    for source = 1 to prefix.processed_slots do
      match prefix.targets.(source) with
      | Some Program_complete ->
          fixed_children.(0) <- fixed_children.(0) + 1
      | Some (Origin_label target) ->
          next.(source) <- target;
          fixed_children.(target) <- fixed_children.(target) + 1;
          if source = target then valid := false
      | None -> valid := false
    done;
    begin match prefix.targets.(entry_label) with
    | Some Program_complete when tree.parent.(tree.entry) <> -1 -> valid := false
    | Some (Origin_label _) when tree.parent.(tree.entry) = -1 -> valid := false
    | _ -> ()
    end;
    for source = 1 to prefix.processed_slots do
      let seen = Array.make (size + 1) false in
      let rec follow current =
        if current = 0 then ()
        else if seen.(current) then valid := false
        else begin
          seen.(current) <- true;
          if next.(current) <> 0 then follow next.(current)
        end
      in
      follow source
    done;
    if fixed_children.(0) > !root_degree then valid := false;
    for target = 1 to size do
      if fixed_children.(target) > !max_degree then valid := false
    done;
    !valid
  end
