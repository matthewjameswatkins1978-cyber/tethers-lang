module Core = Tethers_core
module Validator = Tethers_core_validator
module Format = Tethers_core_canonical_v2_format

type target =
  | Origin_target of int
  | Program_complete

type partial_successor = {
  source : int;
  target : target;
}

type choice_order =
  | Encoded_ascending
  | Numeric_ascending
  | Numeric_descending

type stats = {
  path_size : int;
  successor_slots_processed : int;
  candidate_targets_considered : int;
  feasibility_checks : int;
  rejected_infeasible_choices : int;
  committed_choices : int;
  complete_permutations_enumerated : int;
  max_partial_components : int;
}

type mutable_stats = {
  mutable successor_slots_processed : int;
  mutable candidate_targets_considered : int;
  mutable feasibility_checks : int;
  mutable rejected_infeasible_choices : int;
  mutable committed_choices : int;
  mutable max_partial_components : int;
}

type result = {
  payload : string;
  labels : (Core.origin_id * int) list;
  stats : stats;
}

type error =
  | Invalid_core of Validator.validation_error list
  | Unsupported_success_path of string
  | No_legal_success_path

type raw_target =
  | Raw_origin of int
  | Raw_complete

type rollback_dsu = {
  parent : int array;
  size : int array;
}

type solver_state = {
  path_size : int;
  entry_label : int;
  next : raw_target option array;
  predecessor_used : bool array;
  dsu : rollback_dsu;
  mutable components : int;
  mutable terminal_source : int option;
}

let make_dsu size = {
  parent = Array.init (size + 1) (fun index -> index);
  size = Array.make (size + 1) 1;
}

let rec dsu_find dsu node =
  if dsu.parent.(node) = node then node
  else dsu_find dsu dsu.parent.(node)

let dsu_union dsu left right =
  let left_root = dsu_find dsu left in
  let right_root = dsu_find dsu right in
  if left_root = right_root then None
  else if dsu.size.(left_root) < dsu.size.(right_root) then begin
    dsu.parent.(left_root) <- right_root;
    dsu.size.(right_root) <- dsu.size.(right_root) + dsu.size.(left_root);
    Some (left_root, right_root, dsu.size.(left_root))
  end else begin
    dsu.parent.(right_root) <- left_root;
    dsu.size.(left_root) <- dsu.size.(left_root) + dsu.size.(right_root);
    Some (right_root, left_root, dsu.size.(right_root))
  end

let dsu_undo dsu = function
  | None -> ()
  | Some (child, parent, child_size) ->
      dsu.parent.(child) <- child;
      dsu.size.(parent) <- dsu.size.(parent) - child_size

let target_bytes = function
  | Raw_origin label -> Format.encode_tag 0 ^ Format.encode_int label
  | Raw_complete -> Format.encode_tag 1

let compare_targets left right =
  Format.compare_bytes_lex_unsigned (target_bytes left) (target_bytes right)

let target_of_public = function
  | Origin_target label -> Raw_origin label
  | Program_complete -> Raw_complete

let valid_label path_size label = label >= 1 && label <= path_size

(* With sources processed in increasing numeric order, every assigned origin
   edge is an edge from the open end of one partial path to the open start of
   another.  The rollback DSU therefore gives an exact cycle test and exact
   component count.  The only remaining completion obstruction is a closed
   ProgramComplete path rooted at entry while other components remain. *)
let partial_state_feasible state processed_slots =
  if processed_slots < 0 || processed_slots > state.path_size then false
  else if state.entry_label < 1 || state.entry_label > state.path_size then false
  else if state.terminal_source <> None &&
          dsu_find state.dsu (Option.get state.terminal_source) =
          dsu_find state.dsu state.entry_label && state.components > 1 then false
  else if processed_slots = state.path_size then
    state.terminal_source <> None &&
    state.components = 1 &&
    dsu_find state.dsu state.entry_label = dsu_find state.dsu 1
  else
    true

let feasible_partial ~path_size ~entry_label ~processed_slots partial =
  if path_size < 1 || entry_label < 1 || entry_label > path_size ||
     processed_slots < 0 || processed_slots > path_size then false
  else begin
    let state = {
      path_size;
      entry_label;
      next = Array.make (path_size + 1) None;
      predecessor_used = Array.make (path_size + 1) false;
      dsu = make_dsu path_size;
      components = path_size;
      terminal_source = None;
    } in
    let valid = ref true in
    List.iter (fun { source; target } ->
      if source < 1 || source > path_size || source > processed_slots ||
         state.next.(source) <> None then valid := false
      else match target_of_public target with
        | Raw_complete ->
            if state.terminal_source <> None then valid := false
            else begin
              state.next.(source) <- Some Raw_complete;
              state.terminal_source <- Some source
            end
        | Raw_origin target ->
            if not (valid_label path_size target) || target = entry_label ||
               state.predecessor_used.(target) || source = target ||
               dsu_find state.dsu source = dsu_find state.dsu target then
              valid := false
            else begin
              match dsu_union state.dsu source target with
              | None -> valid := false
              | Some _ ->
                  state.next.(source) <- Some (Raw_origin target);
                  state.predecessor_used.(target) <- true;
                  state.components <- state.components - 1
            end
    ) partial;
    if not !valid then false
    else begin
      for source = 1 to processed_slots do
        if state.next.(source) = None then valid := false
      done;
      for source = processed_slots + 1 to path_size do
        if state.next.(source) <> None then valid := false
      done;
      if state.predecessor_used.(entry_label) then valid := false;
      if not !valid then false else partial_state_feasible state processed_slots
    end
  end

let minimum_encoded_label path_size =
  let labels = List.init path_size (fun index -> index + 1) in
  match labels with
  | [] -> None
  | first :: rest ->
      Some (List.fold_left (fun best label ->
        if Format.compare_bytes_lex_unsigned (Format.encode_int label)
             (Format.encode_int best) < 0 then label else best
      ) first rest)

let ordered_candidates state choice_order =
  let labels = List.init state.path_size (fun index -> index + 1) in
  let labels = List.filter (fun label ->
    label <> state.entry_label && not state.predecessor_used.(label)
  ) labels in
  let labels = match choice_order with
    | Encoded_ascending ->
        List.sort (fun left right ->
          Format.compare_bytes_lex_unsigned (Format.encode_int left)
            (Format.encode_int right)
        ) labels
    | Numeric_ascending -> labels
    | Numeric_descending -> List.rev labels
  in
  match state.terminal_source with
  | Some _ -> List.map (fun label -> Raw_origin label) labels
  | None -> List.map (fun label -> Raw_origin label) labels @ [Raw_complete]

let try_candidate state source candidate =
  match candidate with
  | Raw_complete ->
      if state.terminal_source <> None then None
      else begin
        state.next.(source) <- Some Raw_complete;
        state.terminal_source <- Some source;
        Some None
      end
  | Raw_origin target ->
      if not (valid_label state.path_size target) ||
         target = state.entry_label ||
         state.predecessor_used.(target) ||
         source = target then None
      else
        let source_root = dsu_find state.dsu source in
        let target_root = dsu_find state.dsu target in
        if source_root = target_root then None
        else begin
          match dsu_union state.dsu source target with
          | None -> None
          | Some undo ->
              state.next.(source) <- Some (Raw_origin target);
              state.predecessor_used.(target) <- true;
              state.components <- state.components - 1;
              Some (Some undo)
        end

let undo_candidate state source candidate undo =
  state.next.(source) <- None;
  begin match candidate with
  | Raw_complete -> state.terminal_source <- None
  | Raw_origin target ->
      state.predecessor_used.(target) <- false;
      state.components <- state.components + 1;
      dsu_undo state.dsu undo
  end

let follow_numeric_path next entry path_size =
  let visited = Array.make (path_size + 1) false in
  let rec loop label remaining acc =
    if remaining = 0 then Some (List.rev acc)
    else if not (valid_label path_size label) || visited.(label) then None
    else begin
      visited.(label) <- true;
      match next.(label) with
      | Some (Raw_origin target) -> loop target (remaining - 1) (label :: acc)
      | Some Raw_complete when remaining = 1 -> Some (List.rev (label :: acc))
      | _ -> None
    end
  in
  loop entry path_size []

let origin_sites program =
  List.filter_map (fun site ->
    match Format.origin_id_of_site site with
    | Some oid -> Some (oid, site)
    | None -> None
  ) program.Core.origin_sites

let find_origin origins oid =
  List.find_opt (fun (candidate, _) -> candidate = oid) origins

let build_path program =
  let origins = origin_sites program in
  if origins = [] then Error (Unsupported_success_path "empty Origin domain")
  else if List.exists (function Core.Batch_site _ -> true | _ -> false)
      program.Core.origin_sites then
    Error (Unsupported_success_path "Batch site is outside the path crucible")
  else if program.Core.input_facts <> [] || program.Core.entry_guards <> [] ||
          program.Core.branches <> [] || program.Core.roles <> [] ||
          program.Core.item_templates <> [] then
    Error (Unsupported_success_path "non-Origin fields are outside the path crucible")
  else begin
    let successors = List.map (fun (continuation : Core.success_continuation) ->
      continuation.Core.from_origin, continuation.Core.target
    ) program.Core.success_continuations in
    let duplicate_source =
      List.exists (fun (source, _) ->
        List.length (List.filter (fun (candidate, _) -> candidate = source) successors) > 1
      ) successors
    in
    if duplicate_source then
      Error (Unsupported_success_path "duplicate success continuation source")
    else match program.Core.entry_origin with
      | None -> Error (Unsupported_success_path "missing entry_origin")
      | Some entry ->
          if find_origin origins entry = None then
            Error (Unsupported_success_path "entry is outside program Origins")
          else begin
            let rec walk current seen path =
              if List.mem current seen then
                Error (Unsupported_success_path "success path repeats an Origin")
              else
                let seen = current :: seen in
                let path = current :: path in
                match List.assoc_opt current successors with
                | None -> Error (Unsupported_success_path "path has missing continuation")
                | Some Core.Program_complete ->
                    let path = List.rev path in
                    if List.length path = List.length origins then Ok path
                    else Error (Unsupported_success_path
                                  "ProgramComplete occurs before every Origin")
                | Some (Core.Origin_target next) ->
                    if find_origin origins next = None then
                      Error (Unsupported_success_path
                               "success target is outside program Origins")
                    else walk next seen path
            in
            match walk entry [] [] with
            | Error _ as error -> error
            | Ok path when List.length path = List.length origins -> Ok (origins, path)
            | Ok _ -> Error (Unsupported_success_path "disconnected Origin")
          end
  end

let encode_result program path next entry_label stats =
  match follow_numeric_path next entry_label (Array.length next - 1) with
  | None -> Error No_legal_success_path
  | Some numeric_path ->
      let labels = List.map2 (fun oid label -> oid, label) path numeric_path in
      let origin_labels = List.fold_left (fun map (oid, label) ->
        Format.OriginMap.add oid label map
      ) Format.OriginMap.empty labels in
      let assignment = {
        Format.origin_labels;
        fact_labels = Format.FactMap.empty;
        branch_labels = Format.BranchMap.empty;
        batch_labels = Format.BatchMap.empty;
        template_labels = Format.TemplateMap.empty;
        role_labels = Format.ScopedRoleMap.empty;
      } in
      let payload = Format.encode_program assignment program in
      Ok { payload; labels; stats }

let canonicalise ?(choice_order = Encoded_ascending) program =
  match Validator.validate program with
  | Error errors -> Error (Invalid_core errors)
  | Ok () ->
      begin match build_path program with
      | Error error -> Error error
      | Ok (_, path) ->
          let path_size = List.length path in
          match minimum_encoded_label path_size with
          | None -> Error No_legal_success_path
          | Some entry_label ->
              let state = {
                path_size;
                entry_label;
                next = Array.make (path_size + 1) None;
                predecessor_used = Array.make (path_size + 1) false;
                dsu = make_dsu path_size;
                components = path_size;
                terminal_source = None;
              } in
              let statistics = {
                successor_slots_processed = 0;
                candidate_targets_considered = 0;
                feasibility_checks = 0;
                rejected_infeasible_choices = 0;
                committed_choices = 0;
                max_partial_components = path_size;
              } in
              let rec process source =
                if source > path_size then
                  let stats = {
                    path_size;
                    successor_slots_processed = statistics.successor_slots_processed;
                    candidate_targets_considered = statistics.candidate_targets_considered;
                    feasibility_checks = statistics.feasibility_checks;
                    rejected_infeasible_choices = statistics.rejected_infeasible_choices;
                    committed_choices = statistics.committed_choices;
                    complete_permutations_enumerated = 0;
                      max_partial_components = statistics.max_partial_components;
                  } in
                  encode_result program path state.next entry_label stats
                else begin
                  let candidates = ordered_candidates state choice_order in
                  let best = ref None in
                  let rec inspect = function
                    | [] -> ()
                    | candidate :: rest ->
                        statistics.candidate_targets_considered <-
                          statistics.candidate_targets_considered + 1;
                        begin match try_candidate state source candidate with
                        | None ->
                            statistics.feasibility_checks <-
                              statistics.feasibility_checks + 1;
                            statistics.rejected_infeasible_choices <-
                              statistics.rejected_infeasible_choices + 1;
                            inspect rest
                        | Some undo ->
                            statistics.feasibility_checks <-
                              statistics.feasibility_checks + 1;
                            let feasible = partial_state_feasible state source in
                            if feasible then begin
                              statistics.max_partial_components <-
                                max statistics.max_partial_components state.components;
                              begin match !best with
                              | None -> best := Some candidate
                              | Some current when compare_targets candidate current < 0 ->
                                  best := Some candidate
                              | Some _ -> ()
                              end
                            end else
                              statistics.rejected_infeasible_choices <-
                                statistics.rejected_infeasible_choices + 1;
                            undo_candidate state source candidate undo;
                            if choice_order = Encoded_ascending && feasible then ()
                            else inspect rest
                        end
                  in
                  inspect candidates;
                  match !best with
                  | None -> Error No_legal_success_path
                  | Some candidate ->
                      begin match try_candidate state source candidate with
                      | None -> Error No_legal_success_path
                      | Some undo ->
                          ignore undo;
                          statistics.successor_slots_processed <-
                            statistics.successor_slots_processed + 1;
                          statistics.committed_choices <-
                            statistics.committed_choices + 1;
                          process (source + 1)
                      end
                end
              in
              process 1
      end
