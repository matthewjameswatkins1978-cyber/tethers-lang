type target =
  | Origin_label of int
  | Program_complete

type relation = {
  source : int;
  target : target;
}

type verdict =
  | Proven_feasible
  | Proven_infeasible
  | Unknown_global_packing of string

type stats = {
  candidate_states : int;
  candidate_pairs : int;
  matching_instances : int;
  matching_vertices : int;
  matching_edges : int;
  matching_failures : int;
}

type mutable_stats = {
  mutable candidate_states : int;
  mutable candidate_pairs : int;
  mutable matching_instances : int;
  mutable matching_vertices : int;
  mutable matching_edges : int;
  mutable matching_failures : int;
}

let empty_stats () : mutable_stats = {
  candidate_states = 0;
  candidate_pairs = 0;
  matching_instances = 0;
  matching_vertices = 0;
  matching_edges = 0;
  matching_failures = 0;
}

let freeze_stats (stats : mutable_stats) : stats = {
  candidate_states = stats.candidate_states;
  candidate_pairs = stats.candidate_pairs;
  matching_instances = stats.matching_instances;
  matching_vertices = stats.matching_vertices;
  matching_edges = stats.matching_edges;
  matching_failures = stats.matching_failures;
}

let valid_semantic_tree parent entry =
  let size = Array.length parent in
  if size = 0 then Error "empty semantic tree"
  else if entry < 0 || entry >= size then Error "entry is outside semantic tree"
  else if Array.exists (fun p -> p < -1 || p >= size) parent then
    Error "semantic parent is outside tree"
  else if Array.fold_left (fun count p -> count + if p = -1 then 1 else 0)
      0 parent <> 1 then
    Error "semantic tree must have one ProgramComplete root"
  else begin
    let state = Array.make size 0 in
    let rec visit node =
      match state.(node) with
      | 1 -> Error "semantic tree contains a cycle"
      | 2 -> Ok ()
      | _ ->
          state.(node) <- 1;
          begin match parent.(node) with
          | -1 -> Ok ()
          | next -> visit next
          end |> Result.map (fun () -> state.(node) <- 2)
    in
    let rec visit_all node =
      if node = size then Ok ()
      else match visit node with
        | Error _ as error -> error
        | Ok () -> visit_all (node + 1)
    in
    visit_all 0
  end

let target_label size source = function
  | Program_complete -> Ok None
  | Origin_label label when label >= 1 && label <= size && label <> source ->
      Ok (Some label)
  | Origin_label _ -> Error "prefix target is outside the numeric label domain"

type pattern = {
  entry_label : int;
  next : int option array;
  terminal_source : int option;
  active : bool array;
  children : int list array;
  component_count : int;
}

let build_pattern ~size ~entry_label ~processed_slots relations =
  if size < 1 then Error "empty numeric label domain"
  else if entry_label < 1 || entry_label > size then
    Error "entry label is outside the numeric label domain"
  else if processed_slots < 0 || processed_slots > size then
    Error "processed slot count is outside the numeric label domain"
  else begin
    let next = Array.make (size + 1) None in
    let active = Array.make (size + 1) false in
    let terminal_source = ref None in
    let valid = ref true in
    List.iter (fun { source; target } ->
      if source < 1 || source > processed_slots || next.(source) <> None then
        valid := false
      else begin
        active.(source) <- true;
        match target_label size source target with
        | Error _ -> valid := false
        | Ok None ->
            begin match !terminal_source with
            | Some _ -> valid := false
            | None ->
                next.(source) <- Some 0;
                terminal_source := Some source
            end
        | Ok (Some label) ->
            active.(label) <- true;
            next.(source) <- Some label
      end
    ) relations;
    active.(entry_label) <- true;
    if not !valid then Error "invalid or duplicate partial relation"
    else begin
      let state = Array.make (size + 1) 0 in
      let cycle = ref false in
      for start = 1 to size do
        let rec follow node =
          if node = 0 || not active.(node) then ()
          else match state.(node) with
            | 1 -> cycle := true
            | 2 -> ()
            | _ ->
                state.(node) <- 1;
                begin match next.(node) with
                | None -> ()
                | Some target -> follow target
                end;
                state.(node) <- 2
        in
        follow start
      done;
      if !cycle then Error "partial numeric pattern contains a cycle"
      else begin
        let parent = Array.init (size + 1) (fun index -> index) in
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
          if left_root <> right_root then parent.(right_root) <- left_root
        in
        for source = 1 to size do
          match next.(source) with
          | Some target -> union source target
          | None -> ()
        done;
        let component = Array.make (size + 1) (-1) in
        let component_roots = Hashtbl.create size in
        let component_count = ref 0 in
        for slot = 1 to size do
          if active.(slot) then begin
            let root = find slot in
            let id = match Hashtbl.find_opt component_roots root with
              | Some value -> value
              | None ->
                  let value = !component_count in
                  incr component_count;
                  Hashtbl.add component_roots root value;
                  value
            in
            component.(slot) <- id
          end
        done;
        let children = Array.make (size + 1) [] in
        for source = 1 to size do
          match next.(source) with
          | Some target when target <> 0 ->
              children.(target) <- source :: children.(target)
          | _ -> ()
        done;
        for slot = 1 to size do
          children.(slot) <- List.sort Int.compare children.(slot)
        done;
        Ok {
          entry_label;
          next;
          terminal_source = !terminal_source;
          active;
          children;
          component_count = !component_count;
        }
      end
    end
  end

let evaluate_connected_component ~semantic_parent ~entry_semantic ~entry_label
    ~processed_slots relations =
  let stats = empty_stats () in
  match valid_semantic_tree semantic_parent entry_semantic with
  | Error message -> Error message
  | Ok () ->
      let size = Array.length semantic_parent in
      begin match build_pattern ~size ~entry_label ~processed_slots relations with
      | Error message -> Error message
      | Ok pattern ->
          if pattern.component_count > 1 then
            Ok (Unknown_global_packing
                  "multiple partial components require a global disjoint-placement state",
                freeze_stats stats)
          else begin
            let semantic_children = Array.make size [] in
            Array.iteri (fun node parent ->
              if parent >= 0 then
                semantic_children.(parent) <-
                  node :: semantic_children.(parent)
            ) semantic_parent;
            for node = 0 to size - 1 do
              semantic_children.(node) <-
                List.sort Int.compare semantic_children.(node)
            done;
            let memo = Array.make_matrix (size + 1) size (-1) in
            let rec can_map pattern_node semantic_node =
              stats.candidate_states <- stats.candidate_states + 1;
              if pattern_node = pattern.entry_label &&
                 semantic_node <> entry_semantic then false
              else if memo.(pattern_node).(semantic_node) <> -1 then
                memo.(pattern_node).(semantic_node) = 1
              else begin
                let terminal_ok =
                  match pattern.terminal_source with
                  | Some source when source = pattern_node ->
                      semantic_parent.(semantic_node) = -1
                  | Some _ -> true
                  | None -> true
                in
                if not terminal_ok then begin
                  memo.(pattern_node).(semantic_node) <- 0;
                  false
                end else begin
                  let pattern_children = pattern.children.(pattern_node) in
                  let host_children = semantic_children.(semantic_node) in
                  stats.matching_instances <- stats.matching_instances + 1;
                  stats.matching_vertices <- stats.matching_vertices +
                    List.length pattern_children + List.length host_children;
                  let host_children = Array.of_list host_children in
                  let matches = Array.make (Array.length host_children) (-1) in
                  let rec augment pattern_child seen =
                    let rec try_hosts index =
                      if index = Array.length host_children then false
                      else if seen.(index) then try_hosts (index + 1)
                      else begin
                        stats.candidate_pairs <- stats.candidate_pairs + 1;
                        stats.matching_edges <- stats.matching_edges + 1;
                        if not (can_map pattern_child host_children.(index)) then
                          try_hosts (index + 1)
                        else begin
                          seen.(index) <- true;
                          if matches.(index) = -1 ||
                             augment matches.(index) seen then begin
                            matches.(index) <- pattern_child;
                            true
                          end else
                            try_hosts (index + 1)
                        end
                      end
                    in
                    try_hosts 0
                  in
                  let matched = List.for_all (fun child ->
                    augment child (Array.make (Array.length host_children) false)
                  ) pattern_children in
                  if not matched then stats.matching_failures <-
                    stats.matching_failures + 1;
                  memo.(pattern_node).(semantic_node) <-
                    if matched then 1 else 0;
                  matched
                end
              end
            in
            let roots =
              List.init size (fun index -> index)
              |> List.filter (fun node -> pattern.active.(node + 1) &&
                   (match pattern.next.(node + 1) with
                    | None -> true
                    | Some 0 -> true
                    | Some _ -> false))
            in
            let root = match roots with
              | [value] -> value + 1
              | _ -> pattern.entry_label
            in
            let candidates =
              match pattern.terminal_source with
              | Some source when source = root ->
                  List.init size (fun index -> index)
                  |> List.filter (fun node -> semantic_parent.(node) = -1)
              | _ -> List.init size (fun index -> index)
            in
            let feasible = List.exists (fun semantic_root ->
              can_map root semantic_root
            ) candidates in
            Ok ((if feasible then Proven_feasible else Proven_infeasible),
                freeze_stats stats)
          end
      end
