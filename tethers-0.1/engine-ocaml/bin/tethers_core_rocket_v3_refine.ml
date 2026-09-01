module Model = Tethers_core_rocket_v3_model
module Partition = Tethers_core_rocket_v3_partition

type direction =
  | Forward
  | Reverse

type stats = {
  relation_visits : int;
  splitter_pops : int;
  cell_splits : int;
  max_worklist : int;
  final_cell_count : int;
}

type result = {
  partition : Partition.t;
  stats : stats;
}

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

let direction_key = function
  | Forward -> "forward"
  | Reverse -> "reverse"

let channel_key (direction, relation, discriminator, payload) =
  String.concat "|" [direction_key direction; Model.relation_name relation;
                     discriminator_key discriminator; payload]

let signature_key partition counts vertex channels =
  let entries = List.sort (fun left right ->
    String.compare (channel_key left) (channel_key right)) channels
    |> List.map (fun channel ->
      let count = match Hashtbl.find_opt counts (vertex, channel) with
        | Some count -> count
        | None -> 0
      in
      channel_key channel ^ "=" ^ string_of_int count
    )
  in
  let target_cell_context = Partition.cell_key partition (Partition.cell_of_vertex partition vertex) in
  target_cell_context ^ "{" ^ String.concat ";" entries ^ "}"

let choose_best_work_item partition first rest =
  let rec choose best before = function
    | [] -> best, List.rev before
    | item :: tail ->
        if String.compare (Partition.cell_key partition item)
             (Partition.cell_key partition best) < 0 then
          choose item (best :: before) tail
        else choose best (item :: before) tail
  in
  choose first [] rest

let refine partition =
  let model = Partition.model partition in
  let vertex_count = Partition.vertex_count partition in
  let queued = Hashtbl.create (Partition.cell_count partition * 2 + 1) in
  let worklist = ref [] in
  let max_worklist = ref 0 in
  let enqueue cell =
    if not (Hashtbl.mem queued cell) then begin
      Hashtbl.add queued cell ();
      worklist := cell :: !worklist;
      max_worklist := max !max_worklist (List.length !worklist)
    end
  in
  let remove_queued cell =
    if Hashtbl.mem queued cell then begin
      Hashtbl.remove queued cell;
      worklist := List.filter (fun item -> item <> cell) !worklist
    end
  in
  List.iter enqueue (Partition.cell_ids partition);
  let relation_visits = ref 0 in
  let splitter_pops = ref 0 in
  let cell_splits = ref 0 in
  while !worklist <> [] do
    let first = List.hd !worklist in
    let splitter, remainder = choose_best_work_item partition first (List.tl !worklist) in
    worklist := remainder;
    Hashtbl.remove queued splitter;
    incr splitter_pops;
    let counts = Hashtbl.create 32 in
    let channels_by_vertex = Hashtbl.create 32 in
    let affected_vertices = ref [] in
    let marked_vertex = Array.make vertex_count false in
    let add_incidence source channel =
      let key = source, channel in
      let had = Hashtbl.mem counts key in
      let count = match Hashtbl.find_opt counts key with
        | Some count -> count
        | None -> 0
      in
      Hashtbl.replace counts key (count + 1);
      if not had then begin
        let channels = match Hashtbl.find_opt channels_by_vertex source with
          | Some channels -> channels
          | None -> []
        in
        Hashtbl.replace channels_by_vertex source (channel :: channels)
      end;
      if not marked_vertex.(source) then begin
        marked_vertex.(source) <- true;
        affected_vertices := source :: !affected_vertices
      end
    in
    let visit direction edge source =
      incr relation_visits;
      add_incidence source
        (direction, edge.Model.relation, edge.Model.discriminator, edge.Model.payload)
    in
    List.iter (fun vertex ->
      List.iter (fun edge -> visit Forward edge edge.Model.target)
        (Model.reverse_edges model vertex);
      List.iter (fun edge -> visit Reverse edge edge.Model.target)
        (Model.forward_edges model vertex)
    ) (Partition.cell_members partition splitter);
    let affected_cells = Hashtbl.create 16 in
    List.iter (fun vertex ->
      Hashtbl.replace affected_cells (Partition.cell_of_vertex partition vertex) ()
    ) !affected_vertices;
    let affected_cells = Hashtbl.fold (fun cell () acc -> cell :: acc) affected_cells []
      |> List.sort (fun left right ->
        String.compare (Partition.cell_key partition left)
          (Partition.cell_key partition right)) in
    List.iter (fun cell ->
      let groups = Hashtbl.create (Partition.cell_size partition cell) in
      List.iter (fun vertex ->
        let channels = match Hashtbl.find_opt channels_by_vertex vertex with
          | Some channels -> channels
          | None -> []
        in
        let key = signature_key partition counts vertex channels in
        let members = match Hashtbl.find_opt groups key with
          | Some members -> members
          | None -> []
        in
        Hashtbl.replace groups key (vertex :: members)
      ) (Partition.cell_members partition cell);
      let groups = Hashtbl.fold (fun key members acc -> (key, members) :: acc) groups [] in
      if List.length groups > 1 then begin
        let was_queued = Hashtbl.mem queued cell in
        let result = Partition.split_cell partition cell groups in
        if result.changed then begin
          incr cell_splits;
          if was_queued then begin
            remove_queued cell;
            List.iter enqueue result.parts
          end else
            List.iter (fun part -> if part <> result.retained then enqueue part) result.parts
        end
      end
    ) affected_cells
  done;
  Partition.mark_stable partition;
  { partition;
    stats = {
      relation_visits = !relation_visits;
      splitter_pops = !splitter_pops;
      cell_splits = !cell_splits;
      max_worklist = !max_worklist;
      final_cell_count = Partition.cell_count partition;
    } }

let run model = refine (Partition.create model)
