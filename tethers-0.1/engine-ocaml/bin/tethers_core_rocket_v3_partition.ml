module Model = Tethers_core_rocket_v3_model

type cell = {
  initial : string;
  mutable key : string;
  mutable members : int list;
}

type t = {
  model : Model.t;
  vertex_cells : int array;
  cells : (int, cell) Hashtbl.t;
  mutable cell_order : int list;
  mutable next_cell : int;
  mutable stable : bool;
}

type split_result = {
  changed : bool;
  retained : int;
  parts : int list;
}

let kind_key = function
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

let outcome_key = function
  | Tethers_core.Success -> "success"
  | Tethers_core.Failure -> "failure"
  | Tethers_core.Uncertain -> "uncertain"
  | Tethers_core.Cancelled -> "cancelled"

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

let edge_key direction edge target_key =
  let direction_key = match direction with
    | `Forward -> "forward"
    | `Reverse -> "reverse"
  in
  String.concat "|" [
    direction_key;
    Model.relation_name edge.Model.relation;
    discriminator_key edge.Model.discriminator;
    edge.Model.payload;
    target_key;
  ]

let vertex_key model index =
  let kind = kind_key (Model.vertex_kind model index) in
  let scalar = Model.vertex_scalar model index in
  let describe direction edge =
    let target_kind = kind_key (Model.vertex_kind model edge.Model.target) in
    let target_scalar = Model.vertex_scalar model edge.Model.target in
    edge_key direction edge (kind_key (Model.vertex_kind model edge.Model.target) ^
                            "#" ^ target_scalar) ^
    "#" ^ target_kind
  in
  let forward = Model.forward_edges model index
    |> List.map (describe `Forward) |> List.sort String.compare in
  let reverse = Model.reverse_edges model index
    |> List.map (describe `Reverse) |> List.sort String.compare in
  kind ^ "\000" ^ scalar ^ "\000f[" ^ String.concat ";" forward ^
  "]\000r[" ^ String.concat ";" reverse ^ "]"

let initial_key model index =
  kind_key (Model.vertex_kind model index) ^ "\000" ^
  Model.vertex_scalar model index

let create model =
  let count = Model.vertex_count model in
  let vertex_cells = Array.make count (-1) in
  let groups = Hashtbl.create count in
  for index = 0 to count - 1 do
    let key = initial_key model index in
    let members = match Hashtbl.find_opt groups key with
      | Some members -> members
      | None -> []
    in
    Hashtbl.replace groups key (index :: members)
  done;
  let ordered_groups = Hashtbl.fold (fun key members acc ->
    (key, members) :: acc) groups []
    |> List.sort (fun (a, _) (b, _) -> String.compare a b) in
  let cells = Hashtbl.create count in
  let cell_order = ref [] in
  let next_cell = ref 0 in
  List.iter (fun (key, members) ->
    let id = !next_cell in
    incr next_cell;
    let members = List.sort (fun a b ->
      String.compare (vertex_key model a) (vertex_key model b)) members in
    List.iter (fun vertex -> vertex_cells.(vertex) <- id) members;
    Hashtbl.add cells id { initial = key; key; members };
    cell_order := id :: !cell_order
  ) ordered_groups;
  { model; vertex_cells; cells; cell_order = List.rev !cell_order;
    next_cell = !next_cell; stable = false }

let model partition = partition.model
let vertex_count partition = Array.length partition.vertex_cells
let cell_count partition = Hashtbl.length partition.cells
let cell_ids partition = partition.cell_order
let cell_of_vertex partition vertex = partition.vertex_cells.(vertex)
let cell_members partition cell = (Hashtbl.find partition.cells cell).members
let cell_size partition cell = List.length (cell_members partition cell)
let cell_key partition cell = (Hashtbl.find partition.cells cell).key
let initial_key partition vertex =
  initial_key partition.model vertex
let same_cell partition left right =
  cell_of_vertex partition left = cell_of_vertex partition right
let is_discrete partition =
  List.for_all (fun cell -> cell_size partition cell = 1) (cell_ids partition)
let is_stable partition = partition.stable

let evidence partition =
  cell_ids partition
  |> List.map (fun cell ->
    cell_members partition cell
    |> List.map (vertex_key partition.model)
    |> List.sort String.compare
    |> String.concat ";")
  |> List.sort String.compare
  |> String.concat "\n"

let split_cell partition old_cell groups =
  let cell = Hashtbl.find partition.cells old_cell in
  let groups = List.sort (fun (left_key, left_members) (right_key, right_members) ->
    let size_order = compare (List.length left_members) (List.length right_members) in
    if size_order <> 0 then size_order else String.compare left_key right_key
  ) groups in
  match groups with
  | [_] -> { changed = false; retained = old_cell; parts = [old_cell] }
  | [] -> invalid_arg "Rocket V3 partition split: empty groups"
  | _ ->
      let largest = List.fold_left (fun best candidate ->
        let best_size = List.length (snd best) in
        let candidate_size = List.length (snd candidate) in
        if candidate_size > best_size then candidate
        else if candidate_size < best_size then best
        else if String.compare (fst candidate) (fst best) > 0 then candidate
        else best
      ) (List.hd groups) (List.tl groups) in
      let largest_key, largest_members = largest in
      let ordered_parts = List.sort (fun (left_key, _) (right_key, _) ->
        String.compare left_key right_key) groups in
      let assign_members id members =
        List.iter (fun vertex -> partition.vertex_cells.(vertex) <- id) members
      in
      let part_key key = cell.initial ^ "\000part=" ^ key in
      cell.members <- largest_members;
      cell.key <- part_key largest_key;
      assign_members old_cell largest_members;
      let new_parts = ref [] in
      List.iter (fun (key, members) ->
        if key <> largest_key then begin
          let id = partition.next_cell in
          partition.next_cell <- id + 1;
          Hashtbl.add partition.cells id {
            initial = cell.initial;
            key = part_key key;
            members;
          };
          assign_members id members;
          new_parts := id :: !new_parts
        end
      ) ordered_parts;
      let parts = old_cell :: List.rev !new_parts in
      partition.cell_order <- partition.cell_order @ (List.tl parts);
      partition.stable <- false;
      { changed = true; retained = old_cell; parts }

let mark_stable partition = partition.stable <- true
