module Core = Tethers_core
module Format = Tethers_core_canonical_v2_format
module Path = Tethers_core_rocket_v3_success_path

let tests_run = ref 0
let tests_passed = ref 0

let check name condition =
  incr tests_run;
  if condition then incr tests_passed
  else begin
    Printf.eprintf "FAIL: %s\n%!" name;
    exit 1
  end

let oid value = Core.origin_id_of_string value
let pid value = Core.program_id_of_string value
let cid value = Core.capability_id_of_string value
let digest value = Core.capability_contract_digest_of_string value
let version value = Core.core_version_of_string value

let empty_program () = {
  Core.program_id = pid "success-path-program";
  core_version = version "0.1.0";
  input_facts = [];
  entry_guards = [];
  entry_origin = None;
  success_continuations = [];
  origin_sites = [];
  branches = [];
  roles = [];
  item_templates = [];
  capability_contracts = [];
}

let action origin_id capability contract_digest = Core.Action_origin {
  action_origin_id = origin_id;
  capability_id = cid capability;
  contract_digest = digest contract_digest;
  inputs = [];
  declared_facts = [];
  execution_constraints = [];
}

let chain_program ?(tag = "base") ?(reverse = false) ?(distinct = false) size =
  let origins = List.init size (fun index ->
    oid ("success-path-" ^ tag ^ "-origin-" ^ string_of_int index)) in
  let sites = List.mapi (fun index origin_id ->
    if distinct then
      action origin_id ("success-path-cap-" ^ string_of_int index)
        ("success-path-digest-" ^ string_of_int index)
    else action origin_id "success-path-cap" "success-path-digest"
  ) origins in
  let contracts = List.mapi (fun index _ -> {
    Core.capability_id = cid (if distinct then
      "success-path-cap-" ^ string_of_int index else "success-path-cap");
    contract_digest = digest (if distinct then
      "success-path-digest-" ^ string_of_int index else "success-path-digest");
    schema_description = "success path test contract";
  }) (if distinct then origins else [List.hd origins]) in
  let continuations = List.mapi (fun index from_origin ->
    let target = if index + 1 = size then Core.Program_complete
      else Core.Origin_target (List.nth origins (index + 1)) in
    { Core.from_origin; target }
  ) origins in
  {
    (empty_program ()) with
    program_id = pid ("success-path-" ^ tag ^ "-program-" ^ string_of_int size);
    entry_origin = Some (List.hd origins);
    success_continuations = if reverse then List.rev continuations else continuations;
    origin_sites = if reverse then List.rev sites else sites;
    capability_contracts = contracts;
  }

let anchor_action_program () =
  let anchor_id = oid "success-path-anchor" in
  let action_id = oid "success-path-action" in
  {
    (empty_program ()) with
    entry_origin = Some anchor_id;
    origin_sites = [
      Core.Anchor_origin {
        anchor_origin_id = anchor_id;
        event_name = "success.path.event";
        declared_facts = [];
      };
      action action_id "success.path.action" "success.path.action.digest";
    ];
    success_continuations = [
      { Core.from_origin = anchor_id; target = Core.Origin_target action_id };
      { Core.from_origin = action_id; target = Core.Program_complete };
    ];
    capability_contracts = [{
      Core.capability_id = cid "success.path.action";
      contract_digest = digest "success.path.action.digest";
      schema_description = "anchor action test contract";
    }];
  }

let together_path_program () =
  let first = oid "success-path-together-first" in
  let second = oid "success-path-together-second" in
  let together = oid "success-path-together" in
  {
    (empty_program ()) with
    entry_origin = Some first;
    origin_sites = [
      Core.Anchor_origin {
        anchor_origin_id = first;
        event_name = "success.path.together.anchor";
        declared_facts = [];
      };
      action second "success.path.together.action" "success.path.together.digest";
      Core.Together_origin {
        together_origin_id = together;
        group_id = Core.group_id_of_string "success-path-group";
        member_origin_ids = [first; second];
        objective = Core.All_members_succeed;
      };
    ];
    success_continuations = [
      { Core.from_origin = first; target = Core.Origin_target second };
      { Core.from_origin = second; target = Core.Origin_target together };
      { Core.from_origin = together; target = Core.Program_complete };
    ];
    capability_contracts = [{
      Core.capability_id = cid "success.path.together.action";
      contract_digest = digest "success.path.together.digest";
      schema_description = "Together path action contract";
    }];
  }

let payload_for_labels program labels =
  let ids = List.filter_map (fun site ->
    Format.origin_id_of_site site
  ) program.Core.origin_sites in
  let origin_labels = List.fold_left2 (fun map id label ->
    Format.OriginMap.add id label map
  ) Format.OriginMap.empty ids labels in
  let assignment = {
    Format.origin_labels;
    fact_labels = Format.FactMap.empty;
    branch_labels = Format.BranchMap.empty;
    batch_labels = Format.BatchMap.empty;
    template_labels = Format.TemplateMap.empty;
    role_labels = Format.ScopedRoleMap.empty;
  } in
  Format.encode_program assignment program

let remove_one value values =
  let rec loop prefix = function
    | [] -> []
    | head :: tail when head = value -> List.rev_append prefix tail
    | head :: tail -> loop (head :: prefix) tail
  in
  loop [] values

let minimum_encoded_label count =
  let labels = List.init count (fun index -> index + 1) in
  match labels with
  | [] -> failwith "empty label domain"
  | first :: rest ->
      List.fold_left (fun best label ->
        if Format.compare_bytes_lex_unsigned (Format.encode_int label)
             (Format.encode_int best) < 0 then label else best
      ) first rest

let oracle_payload program =
  let count = List.length (List.filter_map Format.origin_id_of_site
    program.Core.origin_sites) in
  let entry = minimum_encoded_label count in
  let residual = remove_one entry (List.init count (fun index -> index + 1)) in
  let best = ref None in
  let rec consider prefix remaining =
    match remaining with
    | [] ->
        let labels = entry :: List.rev prefix in
        let payload = payload_for_labels program labels in
        begin match !best with
        | None -> best := Some payload
        | Some current when Format.compare_bytes_lex_unsigned payload current < 0 ->
            best := Some payload
        | Some _ -> ()
        end
    | _ ->
        List.iter (fun label ->
          consider (label :: prefix) (remove_one label remaining)
        ) remaining
  in
  consider [] residual;
  match !best with
  | Some payload -> payload
  | None -> failwith "empty oracle"

let digest_of_payload payload =
  let preimage = Bytes.cat Format.domain_v2 (Bytes.of_string payload) in
  Format.digest_string_v2 (Format.sha256_hex preimage)

let result_for ?(choice_order = Path.Encoded_ascending) program =
  match Path.canonicalise ~choice_order program with
  | Ok result -> result
  | Error _ ->
      Printf.eprintf "path canonicaliser rejected valid fixture\n%!";
      exit 1

let test_feasibility () =
  Printf.printf "testing exact partial-path feasibility\n%!";
  let origin a = { Path.source = fst a; target = Path.Origin_target (snd a) } in
  check "empty one-node state is completable"
    (Path.feasible_partial ~path_size:1 ~entry_label:1 ~processed_slots:0 []);
  check "one-node complete state is completable"
    (Path.feasible_partial ~path_size:1 ~entry_label:1 ~processed_slots:1
       [{ Path.source = 1; target = Path.Program_complete }]);
  check "duplicate predecessor is rejected"
    (not (Path.feasible_partial ~path_size:3 ~entry_label:1 ~processed_slots:2
      [origin (2, 3); origin (1, 3)]));
  check "predecessor into entry is rejected"
    (not (Path.feasible_partial ~path_size:3 ~entry_label:1 ~processed_slots:1
      [origin (1, 1)]));
  check "premature cycle is rejected"
    (not (Path.feasible_partial ~path_size:3 ~entry_label:1 ~processed_slots:2
      [origin (1, 2); origin (2, 1)]));
  check "multiple terminals are rejected"
    (not (Path.feasible_partial ~path_size:3 ~entry_label:1 ~processed_slots:2
      [{ Path.source = 1; target = Path.Program_complete };
        { Path.source = 2; target = Path.Program_complete }]));
  check "closed entry component with another component is rejected"
    (not (Path.feasible_partial ~path_size:3 ~entry_label:1 ~processed_slots:2
      [{ Path.source = 1; target = Path.Program_complete };
        origin (2, 3)]));
  check "joinable partial paths are accepted"
    (Path.feasible_partial ~path_size:4 ~entry_label:1 ~processed_slots:2
      [origin (1, 2); origin (2, 3)])

let test_feasibility_against_small_table_oracle () =
  let target_values size =
    Path.Program_complete ::
    List.init size (fun index -> Path.Origin_target (index + 1))
  in
  let complete_legal size entry choices =
    let next = Array.make (size + 1) None in
    let predecessor = Array.make (size + 1) false in
    let valid = ref true in
    List.iteri (fun index target ->
      let source = index + 1 in
      match target with
      | Path.Program_complete ->
          if next.(source) <> None then valid := false
          else next.(source) <- Some Path.Program_complete
      | Path.Origin_target label ->
          if label < 1 || label > size || label = entry ||
             predecessor.(label) || label = source || next.(source) <> None then
            valid := false
          else begin
            predecessor.(label) <- true;
            next.(source) <- Some (Path.Origin_target label)
          end
    ) choices;
    for source = 1 to size do
      if next.(source) = None then valid := false
    done;
    if not !valid then false else begin
      let visited = Array.make (size + 1) false in
      let rec follow label remaining =
        if remaining = 0 then true
        else if label < 1 || label > size || visited.(label) then false
        else begin
          visited.(label) <- true;
          match next.(label) with
          | Some (Path.Origin_target target) -> follow target (remaining - 1)
          | Some Path.Program_complete -> remaining = 1
          | None -> false
        end
      in
      follow entry size
    end
  in
  let exists_completion size entry prefix =
    let targets = target_values size in
    let rec extend choices source =
      if source > size then complete_legal size entry (List.rev choices)
      else List.exists (fun target ->
        extend (target :: choices) (source + 1)
      ) targets
    in
    extend (List.rev prefix) (List.length prefix + 1)
  in
  let rec all_prefixes size entry processed prefix =
    if processed > size then true
    else begin
      let expected = exists_completion size entry prefix in
      let actual = Path.feasible_partial ~path_size:size ~entry_label:entry
          ~processed_slots:processed
          (List.mapi (fun index target ->
             { Path.source = index + 1; target }) prefix) in
      if expected <> actual then false
      else if processed = size then true
      else List.for_all (fun target ->
        all_prefixes size entry (processed + 1) (prefix @ [target])
      ) (target_values size)
    end
  in
  check "partial feasibility agrees with independent n=4 table oracle"
    (all_prefixes 4 2 0 [])

let test_chain_differential () =
  Printf.printf "testing exact chains 1..11\n%!";
  List.iter (fun size ->
    let program = chain_program size in
    let expected = oracle_payload program in
    let result = result_for program in
    check ("chain " ^ string_of_int size ^ " payload")
      (result.Path.payload = expected);
    check ("chain " ^ string_of_int size ^ " digest")
      (digest_of_payload result.Path.payload = digest_of_payload expected);
    check ("chain " ^ string_of_int size ^ " zero permutations")
      (result.Path.stats.complete_permutations_enumerated = 0)
  ) (List.init 11 (fun index -> index + 1))

let test_chain11_labels () =
  let result = result_for (chain_program 11) in
  let labels = List.map snd result.Path.labels in
  check "chain-11 exact label sequence"
    (labels = [10; 9; 8; 7; 6; 5; 4; 3; 2; 1; 11])

let test_metamorphic_and_choices () =
  Printf.printf "testing path metamorphics and choice traversal\n%!";
  let base = chain_program 9 in
  let reversed = chain_program ~reverse:true 9 in
  let renamed = chain_program ~tag:"renamed" 9 in
  let distinct = chain_program ~distinct:true 9 in
  let base_result = result_for base in
  let reversed_result = result_for reversed in
  let repeated_result = result_for base in
  let same_stats left right =
    left.Path.stats.path_size = right.Path.stats.path_size &&
    left.Path.stats.successor_slots_processed =
      right.Path.stats.successor_slots_processed &&
    left.Path.stats.candidate_targets_considered =
      right.Path.stats.candidate_targets_considered &&
    left.Path.stats.feasibility_checks = right.Path.stats.feasibility_checks &&
    left.Path.stats.rejected_infeasible_choices =
      right.Path.stats.rejected_infeasible_choices &&
    left.Path.stats.committed_choices = right.Path.stats.committed_choices &&
    left.Path.stats.complete_permutations_enumerated =
      right.Path.stats.complete_permutations_enumerated &&
    left.Path.stats.max_partial_components = right.Path.stats.max_partial_components
  in
  check "storage reversal preserves payload"
    (base_result.Path.payload = reversed_result.Path.payload);
  check "storage reversal preserves labels"
    (base_result.Path.labels = reversed_result.Path.labels);
  check "raw-ID renaming preserves payload"
    (base_result.Path.payload = (result_for renamed).Path.payload);
  check "raw-ID renaming preserves labels"
    (List.map snd base_result.Path.labels =
       List.map snd (result_for renamed).Path.labels);
  check "distinct bodies preserve continuation-selected labels"
    (List.map snd base_result.Path.labels =
       List.map snd (result_for distinct).Path.labels);
  check "distinct bodies match frozen oracle"
    ((result_for distinct).Path.payload = oracle_payload distinct);
  check "repeat traversal preserves deterministic statistics"
    (same_stats base_result repeated_result);
  List.iter (fun choice_order ->
    let result = result_for ~choice_order base in
    check "choice traversal preserves payload"
      (result.Path.payload = base_result.Path.payload);
    check "choice traversal preserves digest"
      (digest_of_payload result.Path.payload =
       digest_of_payload base_result.Path.payload)
  ) [Path.Encoded_ascending; Path.Numeric_ascending; Path.Numeric_descending]

let test_shape_rejection () =
  Printf.printf "testing supported-shape rejection\n%!";
  let disconnected = {
    (chain_program 2) with
    success_continuations = [
      { Core.from_origin = oid "success-path-base-origin-0";
        target = Core.Program_complete };
      { Core.from_origin = oid "success-path-base-origin-1";
        target = Core.Program_complete };
    ];
  } in
  begin match Path.canonicalise disconnected with
  | Error (Path.Unsupported_success_path _) -> check "disconnected rejected" true
  | _ -> check "disconnected rejected" false
  end;
  begin match Path.canonicalise (together_path_program ()) with
  | Ok result ->
      check "Together path accepted" true;
      check "Together path matches frozen oracle"
        (result.Path.payload = oracle_payload (together_path_program ()))
  | Error _ -> check "Together path accepted" false
  end;
  begin match Path.canonicalise (anchor_action_program ()) with
  | Ok result ->
      check "Anchor/Action path accepted" true;
      check "Anchor/Action path matches frozen oracle"
        (result.Path.payload = oracle_payload (anchor_action_program ()))
  | Error _ -> check "Anchor/Action path accepted" false
  end

let test_boundaries () =
  Printf.printf "testing structural decimal boundaries\n%!";
  List.iter (fun size ->
    let result = result_for (chain_program size) in
    Printf.printf
      "success-path chain=%d path_size=%d successor_slots_processed=%d candidate_targets_considered=%d feasibility_checks=%d rejected_infeasible_choices=%d committed_choices=%d complete_permutations_enumerated=%d max_partial_components=%d\n%!"
      size result.Path.stats.path_size
      result.Path.stats.successor_slots_processed
      result.Path.stats.candidate_targets_considered
      result.Path.stats.feasibility_checks
      result.Path.stats.rejected_infeasible_choices
      result.Path.stats.committed_choices
      result.Path.stats.complete_permutations_enumerated
      result.Path.stats.max_partial_components;
    check ("boundary chain " ^ string_of_int size ^ " zero permutations")
      (result.Path.stats.complete_permutations_enumerated = 0)
  ) [9; 10; 11; 12; 99; 100; 999; 1000]

let () =
  test_feasibility ();
  test_feasibility_against_small_table_oracle ();
  test_chain_differential ();
  test_chain11_labels ();
  test_metamorphic_and_choices ();
  test_shape_rejection ();
  test_boundaries ();
  Printf.printf "rocket-v3-success-path: %d/%d checks passed\n%!"
    !tests_passed !tests_run
