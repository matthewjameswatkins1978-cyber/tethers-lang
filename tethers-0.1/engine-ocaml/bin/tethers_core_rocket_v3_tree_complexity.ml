type target =
  | Origin_label of int
  | Program_complete

type prefix = {
  size : int;
  entry_label : int;
  targets : target option array;
}

type prefix_analysis = {
  acyclic : bool;
  terminal_edges : int;
  component_count : int;
  specified_sources : int;
}

type relabelled_forest = {
  slot_of_vertex : int array;
  processed_sources : int;
  targets : target list;
}

let make_prefix ~size ~entry_label targets =
  let processed_sources = List.length targets in
  if size < 1 then Error "empty numeric domain"
  else if entry_label < 1 || entry_label > size then
    Error "entry label is outside the numeric domain"
  else if processed_sources > size then Error "prefix is too long"
  else begin
    let values = Array.make (size + 1) None in
    List.iteri (fun index target -> values.(index + 1) <- Some target) targets;
    Ok { size; entry_label; targets = values }
  end

let analyse_prefix prefix =
  let vertices = prefix.size + 1 in
  let parent = Array.init vertices (fun index -> index) in
  let rec find node =
    if parent.(node) = node then node
    else begin
      parent.(node) <- find parent.(node);
      parent.(node)
    end
  in
  let union left right =
    let left_root = find left in
    let right_root = find right in
    if left_root = right_root then false
    else begin
      parent.(right_root) <- left_root;
      true
    end
  in
  let acyclic = ref true in
  let terminal_edges = ref 0 in
  let specified_sources = ref 0 in
  for source = 1 to prefix.size do
    match prefix.targets.(source) with
    | None -> ()
    | Some Program_complete ->
        incr specified_sources;
        incr terminal_edges;
        if not (union source prefix.size) then acyclic := false
    | Some (Origin_label target) ->
        incr specified_sources;
        if target < 1 || target > prefix.size ||
           not (union source target) then
          acyclic := false
  done;
  let roots = Hashtbl.create vertices in
  for vertex = 0 to vertices - 1 do
    Hashtbl.replace roots (find vertex) ()
  done;
  {
    acyclic = !acyclic;
    terminal_edges = !terminal_edges;
    component_count = Hashtbl.length roots;
    specified_sources = !specified_sources;
  }

let relabel_non_roots_first ~parent =
  let size = Array.length parent in
  if size < 1 then Error "empty forest"
  else if Array.exists (fun value -> value < -1 || value >= size) parent then
    Error "forest parent is outside the vertex domain"
  else begin
    let state = Array.make size 0 in
    let acyclic = ref true in
    let rec visit node =
      if state.(node) = 1 then acyclic := false
      else if state.(node) = 0 then begin
        state.(node) <- 1;
        begin match parent.(node) with
        | -1 -> ()
        | next -> visit next
        end;
        state.(node) <- 2
      end
    in
    for node = 0 to size - 1 do visit node done;
    if not !acyclic then Error "forest contains a cycle"
    else begin
      let non_roots = ref [] in
      let roots = ref [] in
      for node = 0 to size - 1 do
        if parent.(node) = -1 then roots := node :: !roots
        else non_roots := node :: !non_roots
      done;
      let vertices = Array.of_list
          ((List.rev !non_roots) @ (List.rev !roots)) in
      let slot_of_vertex = Array.make size 0 in
      Array.iteri (fun index vertex -> slot_of_vertex.(vertex) <- index + 1)
        vertices;
      let processed_sources = size - List.length !roots in
      let targets = List.init processed_sources (fun index ->
        let vertex = vertices.(index) in
        Origin_label slot_of_vertex.(parent.(vertex))) in
      Ok { slot_of_vertex; processed_sources; targets }
    end
  end

let valid_semantic_tree parent entry =
  let size = Array.length parent in
  size > 0 && entry >= 0 && entry < size &&
  Array.for_all (fun value -> value >= -1 && value < size) parent &&
  Array.fold_left (fun count value -> count + if value = -1 then 1 else 0)
    0 parent = 1 &&
  let state = Array.make size 0 in
  let rec visit node =
    if state.(node) = 1 then false
    else if state.(node) = 2 then true
    else begin
      state.(node) <- 1;
      let valid = match parent.(node) with
        | -1 -> true
        | next -> visit next
      in
      state.(node) <- 2;
      valid
    end
  in
  let valid = ref true in
  for node = 0 to size - 1 do
    if not (visit node) then valid := false
  done;
  !valid

let spanning_forest_witness ~semantic_parent ~semantic_entry ~entry_label
    ~prefix =
  if not (valid_semantic_tree semantic_parent semantic_entry) then false
  else if prefix.entry_label <> entry_label then false
  else if entry_label < 1 || entry_label > prefix.size ||
          prefix.size <> Array.length semantic_parent then false
  else begin
    let size = prefix.size in
    let slot_nodes = Array.make (size + 1) (-1) in
    let used = Array.make size false in
    slot_nodes.(entry_label) <- semantic_entry;
    used.(semantic_entry) <- true;
    let target_matches source node =
      match prefix.targets.(source) with
      | None -> true
      | Some Program_complete -> semantic_parent.(node) = -1
      | Some (Origin_label target) when target >= 1 && target <= size ->
          let target_node = slot_nodes.(target) in
          target_node < 0 || semantic_parent.(node) = target_node
      | Some (Origin_label _) -> false
    in
    let complete_matches () =
      let valid = ref true in
      for source = 1 to size do
        if !valid then
          match prefix.targets.(source) with
          | None -> ()
          | Some Program_complete ->
              if semantic_parent.(slot_nodes.(source)) <> -1 then valid := false
          | Some (Origin_label target) ->
              if target < 1 || target > size ||
                 semantic_parent.(slot_nodes.(source)) < 0 ||
                 semantic_parent.(slot_nodes.(source)) <> slot_nodes.(target)
              then valid := false
      done;
      !valid
    in
    let rec assign source =
      if source > size then complete_matches ()
      else if slot_nodes.(source) >= 0 then
        if target_matches source slot_nodes.(source) then assign (source + 1)
        else false
      else begin
        let found = ref false in
        for node = 0 to size - 1 do
          if not !found && not used.(node) then begin
            used.(node) <- true;
            slot_nodes.(source) <- node;
            if target_matches source node && assign (source + 1) then
              found := true;
            if not !found then begin
              slot_nodes.(source) <- -1;
              used.(node) <- false
            end
          end
        done;
        !found
      end
    in
    assign 1
  end
