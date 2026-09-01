module Core = Tethers_core
module Validator = Tethers_core_validator
module Format = Tethers_core_canonical_v2_format

type branch_order =
  | Numeric_ascending
  | Numeric_descending
  | Semantic_first

type decision =
  | NeedLabel of Core.origin_id
  | NeedOwnerOfNumericSlot of int

type stats = {
  emitted_bytes : int;
  forced_assignments : int;
  decision_points : int;
  branches_explored : int;
  prefix_prunes : int;
  completed_candidates : int;
  max_depth : int;
}

type result = {
  payload : string;
  stats : stats;
}

type error =
  | Invalid_core of Validator.validation_error list
  | Empty_origin_domain
  | Unsupported_origin_projection of string
  | No_legal_origin_assignment

type target =
  | Target_origin of int
  | Target_complete

type context = {
  core_version : string;
  fixed_suffix : string;
  origins : (Core.origin_id * Core.origin_site) array;
  entry : int option;
  continuations : target option array;
  sites : Core.origin_site option array;
  site_shapes : string array;
}

type state = {
  labels : int option array;
  owners : int option array;
}

type mutable_stats = {
  mutable emitted_bytes : int;
  mutable forced_assignments : int;
  mutable decision_points : int;
  mutable branches_explored : int;
  mutable prefix_prunes : int;
  mutable completed_candidates : int;
  mutable max_depth : int;
}

let empty_labels = {
  Format.origin_labels = Format.OriginMap.empty;
  fact_labels = Format.FactMap.empty;
  branch_labels = Format.BranchMap.empty;
  batch_labels = Format.BatchMap.empty;
  template_labels = Format.TemplateMap.empty;
  role_labels = Format.ScopedRoleMap.empty;
}

let origin_sites program =
  List.filter_map (fun site ->
    match Format.origin_id_of_site site with
    | Some oid -> Some (oid, site)
    | None -> None
  ) program.Core.origin_sites
  |> Array.of_list

let site_shape = function
  | Core.Anchor_origin anchor ->
      "anchor|" ^ anchor.Core.event_name ^ "|" ^
      string_of_int (List.length anchor.Core.declared_facts)
  | Core.Action_origin action ->
      "action|" ^ Core.string_of_capability_id action.Core.capability_id ^ "|" ^
      Core.string_of_capability_contract_digest action.Core.contract_digest ^ "|" ^
      string_of_int (List.length action.Core.inputs) ^ "|" ^
      string_of_int (List.length action.Core.declared_facts) ^ "|" ^
      string_of_int (List.length action.Core.execution_constraints)
  | Core.Together_origin together ->
      "together|" ^ string_of_int (List.length together.Core.member_origin_ids)
        ^ "|" ^ (match together.Core.objective with
          | Core.All_members_succeed -> "all-members-succeed")
  | Core.Batch_site _ -> "batch-site-excluded"

let index_of_origin origins oid =
  let rec find index =
    if index = Array.length origins then None
    else if fst origins.(index) = oid then Some index
    else find (index + 1)
  in
  find 0

let validate_projection origins =
  let rec validate_sites index =
    if index = Array.length origins then Ok ()
    else
      match snd origins.(index) with
      | Core.Anchor_origin anchor when anchor.Core.declared_facts <> [] ->
          Error (Unsupported_origin_projection "Anchor declared Facts")
      | Core.Action_origin action when
          action.Core.inputs <> [] || action.Core.declared_facts <> [] ->
          Error (Unsupported_origin_projection "Action non-Origin fields")
      | Core.Together_origin together ->
          let unknown = List.exists (fun oid ->
            index_of_origin origins oid = None) together.Core.member_origin_ids in
          if unknown then
            Error (Unsupported_origin_projection "Together member outside program Origin slice")
          else validate_sites (index + 1)
      | Core.Anchor_origin _
      | Core.Action_origin _ -> validate_sites (index + 1)
      | Core.Batch_site _ ->
          Error (Unsupported_origin_projection "Batch site")
  in
  validate_sites 0

let build_context program =
  let origins = origin_sites program in
  if Array.length origins = 0 then Error Empty_origin_domain
  else if program.Core.input_facts <> [] || program.Core.entry_guards <> [] ||
          program.Core.branches <> [] || program.Core.roles <> [] ||
          program.Core.item_templates <> [] then
    Error (Unsupported_origin_projection
             "non-Origin top-level semantic fields are outside this crucible")
  else
    let ( let* ) = Result.bind in
    let* () = validate_projection origins in
    let* entry = match program.Core.entry_origin with
      | None -> Ok None
      | Some oid ->
          begin match index_of_origin origins oid with
          | Some index -> Ok (Some index)
          | None ->
              (* The projection cannot represent an entry outside the
                 program Origin family.  Do not silently turn it into an
                 unrooted projection. *)
              Error (Unsupported_origin_projection
                       "entry outside Origin projection")
          end
    in
    let continuations = Array.make (Array.length origins) None in
    let rec add_continuations = function
      | [] -> Ok ()
      | continuation :: rest ->
          begin match index_of_origin origins continuation.Core.from_origin with
          | None -> Error (Unsupported_origin_projection
                             "success continuation outside program Origin slice")
          | Some source ->
              begin match continuation.Core.target with
              | Core.Program_complete ->
                  if continuations.(source) <> None then
                    Error (Unsupported_origin_projection
                             "duplicate success continuation source")
                  else begin
                    continuations.(source) <- Some Target_complete;
                    add_continuations rest
                  end
              | Core.Origin_target oid ->
                  begin match index_of_origin origins oid with
                  | None -> Error (Unsupported_origin_projection
                                     "success target outside Origin projection")
                  | Some target ->
                      if continuations.(source) <> None then
                        Error (Unsupported_origin_projection
                                 "duplicate success continuation source")
                      else begin
                        continuations.(source) <- Some (Target_origin target);
                        add_continuations rest
                      end
                  end
              end
          end
    in
    let* () = add_continuations program.Core.success_continuations in
    let sites = Array.make (Array.length origins) None in
    Array.iteri (fun index (_, site) -> sites.(index) <- Some site) origins;
    let site_shapes = Array.map (fun (_, site) -> site_shape site) origins in
    let sorted_capability_contracts = List.sort (fun left right ->
      String.compare
        (Core.string_of_capability_id left.Core.capability_id)
        (Core.string_of_capability_id right.Core.capability_id)
    ) program.Core.capability_contracts in
    let fixed_suffix =
      Format.encode_list (fun _ -> "") program.Core.branches ^
      Format.encode_list (fun _ -> "") program.Core.roles ^
      Format.encode_list (fun _ -> "") program.Core.item_templates ^
      Format.encode_list (Format.encode_capability_contract empty_labels)
        sorted_capability_contracts
    in
    Ok {
      core_version = Core.string_of_core_version program.Core.core_version;
      fixed_suffix;
      origins; entry; continuations; sites; site_shapes
    }

let initial_decision program =
  match Validator.validate program with
  | Error errors -> Error (Invalid_core errors)
  | Ok () ->
      begin match build_context program with
      | Error error -> Error error
      | Ok context ->
          begin match context.entry with
          | Some index -> Ok (NeedLabel (fst context.origins.(index)))
          | None -> Ok (NeedOwnerOfNumericSlot 1)
          end
      end

let encoded_label_compare left right =
  Format.compare_bytes_lex_unsigned (Format.encode_int left)
    (Format.encode_int right)

let remaining_labels state =
  Array.to_list state.owners
  |> List.mapi (fun label owner -> label, owner)
  |> List.filter_map (fun (label, owner) ->
       if label = 0 || owner <> None then None else Some label)

let minimum_encoded_remaining_label state =
  match remaining_labels state with
  | [] -> None
  | labels -> Some (List.fold_left (fun best label ->
      if encoded_label_compare label best < 0 then label else best
    ) (List.hd labels) (List.tl labels))

let assign state origin label =
  if state.labels.(origin) <> None || state.owners.(label) <> None then false
  else begin
    state.labels.(origin) <- Some label;
    state.owners.(label) <- Some origin;
    true
  end

let unassign state origin label =
  state.labels.(origin) <- None;
  state.owners.(label) <- None

let label_of state origin = state.labels.(origin)

let add_full_origin_labels state context =
  let labels = Array.fold_left (fun map (oid, _) ->
    match label_of state (Option.get (index_of_origin context.origins oid)) with
    | Some label -> Format.OriginMap.add oid label map
    | None -> map
  ) Format.OriginMap.empty context.origins in
  { empty_labels with Format.origin_labels = labels }

let encode_site labels = function
  | (Core.Anchor_origin _ as site) ->
      Format.encode_origin_site labels ~origin_scope:Core.Program_scope site
  | (Core.Action_origin _ as site) ->
      Format.encode_origin_site labels ~origin_scope:Core.Program_scope site
  | (Core.Together_origin _ as site) ->
      Format.encode_origin_site labels ~origin_scope:Core.Program_scope site
  | Core.Batch_site _ -> ""

let continuation_prefix state context =
  let total = Array.length context.origins in
  let buffer = Buffer.create (4 * total + 2) in
  Buffer.add_string buffer (string_of_int
    (Array.fold_left (fun count target ->
      match target with Some _ -> count + 1 | None -> count
    ) 0 context.continuations));
  Buffer.add_char buffer ':';
  let rec visit slot =
    if slot > total then true
    else
      match state.owners.(slot) with
      | None -> false
      | Some origin ->
          begin match context.continuations.(origin) with
          | None -> visit (slot + 1)
          | Some Target_complete ->
              Buffer.add_string buffer (Format.encode_int slot);
              Buffer.add_string buffer (Format.encode_tag 1);
              visit (slot + 1)
          | Some (Target_origin target) ->
              Buffer.add_string buffer (Format.encode_int slot);
              Buffer.add_string buffer (Format.encode_tag 0);
              begin match label_of state target with
              | None -> false
              | Some label ->
                  Buffer.add_string buffer (Format.encode_int label);
                  visit (slot + 1)
              end
          end
  in
  let complete = visit 1 in
  Buffer.contents buffer, complete

let guaranteed_prefix state context =
  let buffer = Buffer.create (8 * Array.length context.origins + 8) in
  (* The projection is Origin-only, but the certificate still compares the
     complete frozen payload.  These fields are fixed for this crucible and
     are emitted with the frozen primitive encoders so byte offsets remain
     the real Enc_V2 offsets. *)
  Buffer.add_string buffer (Format.encode_string context.core_version);
  Buffer.add_string buffer (Format.encode_list (fun _ -> "") []);
  Buffer.add_string buffer (Format.encode_list (fun _ -> "") []);
  begin match context.entry with
  | None -> Buffer.add_string buffer "0;"
  | Some entry ->
      begin match label_of state entry with
      | None -> ()
          | Some label ->
              Buffer.add_string buffer "1:";
              Buffer.add_string buffer (Format.encode_int label)
      end
  end;
  let continuations, complete = continuation_prefix state context in
  Buffer.add_string buffer continuations;
  if complete then begin
    Buffer.add_string buffer (string_of_int
      (Array.fold_left (fun count site ->
        match site with Some _ -> count + 1 | None -> count
      ) 0 context.sites));
    Buffer.add_char buffer ':';
    if Array.for_all (fun label -> label <> None) state.labels then begin
      let labels = add_full_origin_labels state context in
      let index_for_site site = match Format.origin_id_of_site site with
        | Some oid -> Option.get (index_of_origin context.origins oid)
        | None -> 0
      in
      let ordered_sites = Array.to_list context.sites
        |> List.filter_map (fun site -> site)
        |> List.sort (fun left right ->
             Int.compare
               (Option.get (label_of state (index_for_site left)))
               (Option.get (label_of state (index_for_site right))))
      in
      List.iter (fun site -> Buffer.add_string buffer (encode_site labels site))
        ordered_sites
    end;
    if Array.for_all (fun label -> label <> None) state.labels then
      Buffer.add_string buffer context.fixed_suffix
  end;
  Buffer.contents buffer

let prefix_greater prefix incumbent =
  let limit = min (String.length prefix) (String.length incumbent) in
  let rec compare index =
    if index = limit then String.length prefix > String.length incumbent
    else
      let left = Char.code prefix.[index] in
      let right = Char.code incumbent.[index] in
      if left = right then compare (index + 1) else left > right
  in
  compare 0

let semantic_origin_order context =
  let indices = List.init (Array.length context.origins) Fun.id in
  let distances = Array.make (Array.length context.origins)
      (Array.length context.origins + 1) in
  let rec walk current distance =
    if distance < distances.(current) then begin
      distances.(current) <- distance;
      match context.continuations.(current) with
      | Some (Target_origin target) -> walk target (distance + 1)
      | Some Target_complete
      | None -> ()
    end
  in
  begin match context.entry with
  | Some entry -> walk entry 0
  | None -> ()
  end;
  List.sort (fun left right ->
    let c = Int.compare distances.(left) distances.(right) in
    if c <> 0 then c
    else String.compare context.site_shapes.(left) context.site_shapes.(right)
  ) indices

let branch_candidates context order unassigned =
  let semantic = semantic_origin_order context in
  let ordered = match order with
    | Semantic_first -> semantic
    | Numeric_ascending -> semantic
    | Numeric_descending -> List.rev semantic
  in
  List.filter (fun origin -> unassigned origin) ordered

let walk ?(branch_order = Semantic_first) program =
  match Validator.validate program with
  | Error errors -> Error (Invalid_core errors)
  | Ok () ->
      begin match build_context program with
      | Error error -> Error error
      | Ok context ->
          let count = Array.length context.origins in
          let state = {
            labels = Array.make count None;
            owners = Array.make (count + 1) None;
          } in
          begin match context.entry, minimum_encoded_remaining_label state with
          | Some entry, Some label ->
              ignore (assign state entry label)
          | None, _ -> ()
          | Some _, None -> ()
          end;
          let statistics = {
            emitted_bytes = 0;
            forced_assignments = (match context.entry with Some _ -> 1 | None -> 0);
            decision_points = 0;
            branches_explored = 0;
            prefix_prunes = 0;
            completed_candidates = 0;
            max_depth = 0;
          } in
          let best = ref None in
          let consider () =
            let payload = guaranteed_prefix state context in
            statistics.emitted_bytes <- statistics.emitted_bytes +
              String.length payload;
            statistics.completed_candidates <- statistics.completed_candidates + 1;
            match !best with
            | None -> best := Some payload
            | Some current ->
                if Format.compare_bytes_lex_unsigned payload current < 0 then
                  best := Some payload
          in
          let rec force_pending_targets ~record_statistics ~on_assign () =
            (* The continuation list is emitted in numeric source-label order.
               Once a source slot is occupied, an unresolved Origin target is
               the next label-dependent byte.  Its owner is therefore forced
               to the smallest remaining encoded integer; this is a frozen
               byte-law consequence, not a semantic or source-order rule. *)
            let changed = ref false in
            for slot = 1 to count do
              match state.owners.(slot) with
              | Some origin ->
                  begin match context.continuations.(origin) with
                  | Some (Target_origin target) when label_of state target = None ->
                      begin match minimum_encoded_remaining_label state with
                      | None -> ()
                      | Some label ->
                          if assign state target label then begin
                            changed := true;
                            if record_statistics then
                              statistics.forced_assignments <-
                                statistics.forced_assignments + 1;
                            on_assign (target, label)
                          end
                      end
                  | Some Target_complete
                  | None
                  | Some (Target_origin _) -> ()
                  end
              | None -> ()
            done;
            if !changed then
              force_pending_targets ~record_statistics ~on_assign ()
          in
          force_pending_targets ~record_statistics:true ~on_assign:(fun _ -> ()) ();
          let greedy_assigned = ref [] in
          let rec greedy slot =
            if slot > count then consider ()
            else if state.owners.(slot) <> None then greedy (slot + 1)
            else
              let candidates = branch_candidates context branch_order
                  (fun origin -> label_of state origin = None) in
              match candidates with
              | [] -> ()
              | origin :: _ ->
                  if assign state origin slot then begin
                    greedy_assigned := (origin, slot) :: !greedy_assigned;
                    force_pending_targets ~record_statistics:false
                      ~on_assign:(fun assignment ->
                        greedy_assigned := assignment :: !greedy_assigned) ();
                  end;
                  greedy (slot + 1)
          in
          greedy 1;
          List.iter (fun (origin, label) -> unassign state origin label)
            !greedy_assigned;
          (* The greedy completion is only an incumbent.  The recursive walk
             below still explores every legal residual alternative. *)
          let rec search slot depth =
            statistics.max_depth <- max statistics.max_depth depth;
            if slot > count then consider ()
            else if state.owners.(slot) <> None then search (slot + 1) depth
            else begin
              statistics.decision_points <- statistics.decision_points + 1;
              let candidates = branch_candidates context branch_order
                  (fun origin -> label_of state origin = None) in
              List.iter (fun origin ->
                statistics.branches_explored <- statistics.branches_explored + 1;
                if assign state origin slot then begin
                  let assigned = ref [ (origin, slot) ] in
                  force_pending_targets ~record_statistics:true
                    ~on_assign:(fun assignment ->
                      assigned := assignment :: !assigned) ();
                  let prefix = guaranteed_prefix state context in
                  statistics.emitted_bytes <- statistics.emitted_bytes +
                    String.length prefix;
                  let pruned = match !best with
                    | Some incumbent -> prefix_greater prefix incumbent
                    | None -> false
                  in
                  if pruned then
                    statistics.prefix_prunes <- statistics.prefix_prunes + 1
                  else
                    search (slot + 1) (depth + 1);
                  List.iter (fun (assigned_origin, assigned_label) ->
                    unassign state assigned_origin assigned_label)
                    !assigned
                end
              ) candidates
            end
          in
          search 1 0;
          match !best with
          | None -> Error No_legal_origin_assignment
          | Some payload -> Ok { payload; stats = {
              emitted_bytes = statistics.emitted_bytes;
              forced_assignments = statistics.forced_assignments;
              decision_points = statistics.decision_points;
              branches_explored = statistics.branches_explored;
              prefix_prunes = statistics.prefix_prunes;
              completed_candidates = statistics.completed_candidates;
              max_depth = statistics.max_depth;
            } }
      end
