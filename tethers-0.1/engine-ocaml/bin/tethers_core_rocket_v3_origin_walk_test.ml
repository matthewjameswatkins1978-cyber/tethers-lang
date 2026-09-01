module Core = Tethers_core
module Format = Tethers_core_canonical_v2_format
module Walk = Tethers_core_rocket_v3_origin_walk

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
  Core.program_id = pid "origin-walk-empty";
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

let action origin_id = Core.Action_origin {
  action_origin_id = origin_id;
  capability_id = cid "origin.walk.capability";
  contract_digest = digest "origin.walk.contract";
  inputs = [];
  declared_facts = [];
  execution_constraints = [];
}

let action_variant origin_id capability contract_digest = Core.Action_origin {
  action_origin_id = origin_id;
  capability_id = cid capability;
  contract_digest = digest contract_digest;
  inputs = [];
  declared_facts = [];
  execution_constraints = [];
}

let origin_walk_contract = {
  Core.capability_id = cid "origin.walk.capability";
  contract_digest = digest "origin.walk.contract";
  schema_description = "origin walk test contract";
}

let chain_program ?(tag = "base") ?(reverse = false) size =
  let origins = List.init size (fun index ->
    oid ("origin-walk-" ^ tag ^ "-" ^ string_of_int index)) in
  let sites = List.map action origins in
  let continuations = List.mapi (fun index from_origin ->
    let target = if index + 1 = size then Core.Program_complete
      else Core.Origin_target (List.nth origins (index + 1)) in
    { Core.from_origin; target }
  ) origins in
  {
    (empty_program ()) with
    program_id = pid ("origin-walk-chain-" ^ tag);
    entry_origin = Some (List.hd origins);
    success_continuations = if reverse then List.rev continuations else continuations;
    origin_sites = if reverse then List.rev sites else sites;
    capability_contracts = [origin_walk_contract];
  }

let anchor_chain_program () =
  let anchor_id = oid "origin-walk-anchor" in
  let action_id = oid "origin-walk-anchor-action" in
  {
    (empty_program ()) with
    program_id = pid "origin-walk-anchor-chain";
    entry_origin = Some anchor_id;
    origin_sites = [
      Core.Anchor_origin {
        anchor_origin_id = anchor_id;
        event_name = "origin.walk.event";
        declared_facts = [];
      };
      action action_id;
    ];
    success_continuations = [
      { Core.from_origin = anchor_id; target = Core.Origin_target action_id };
      { Core.from_origin = action_id; target = Core.Program_complete };
    ];
    capability_contracts = [origin_walk_contract];
  }

let together_program () =
  let anchor_id = oid "origin-walk-together-anchor" in
  let action_id = oid "origin-walk-together-action" in
  let together_id = oid "origin-walk-together-group" in
  {
    (empty_program ()) with
    program_id = pid "origin-walk-together";
    entry_origin = Some anchor_id;
    origin_sites = [
      Core.Anchor_origin {
        anchor_origin_id = anchor_id;
        event_name = "origin.walk.event";
        declared_facts = [];
      };
      action action_id;
      Core.Together_origin {
        together_origin_id = together_id;
        group_id = Core.group_id_of_string "origin-walk-group";
        member_origin_ids = [anchor_id; action_id];
        objective = Core.All_members_succeed;
      };
    ];
    success_continuations = [
      { Core.from_origin = anchor_id; target = Core.Origin_target action_id };
      { Core.from_origin = action_id; target = Core.Program_complete };
    ];
    capability_contracts = [origin_walk_contract];
  }

let symmetric_program () =
  let first = oid "origin-walk-symmetric-a" in
  let second = oid "origin-walk-symmetric-b" in
  let third = oid "origin-walk-symmetric-c" in
  {
    (empty_program ()) with
    program_id = pid "origin-walk-symmetric";
    origin_sites = [
      Core.Anchor_origin {
        anchor_origin_id = first;
        event_name = "origin.walk.symmetric";
        declared_facts = [];
      };
      Core.Anchor_origin {
        anchor_origin_id = second;
        event_name = "origin.walk.symmetric";
        declared_facts = [];
      };
      Core.Anchor_origin {
        anchor_origin_id = third;
        event_name = "origin.walk.symmetric";
        declared_facts = [];
      };
   ];
  }

let disconnected_anchor_program () =
  let first = oid "origin-walk-disconnected-a" in
  let second = oid "origin-walk-disconnected-b" in
  let third = oid "origin-walk-disconnected-c" in
  {
    (empty_program ()) with
    program_id = pid "origin-walk-disconnected-anchors";
    origin_sites = [
      Core.Anchor_origin {
        anchor_origin_id = first;
        event_name = "z-event";
        declared_facts = [];
      };
      Core.Anchor_origin {
        anchor_origin_id = second;
        event_name = "a-event";
        declared_facts = [];
      };
      Core.Anchor_origin {
        anchor_origin_id = third;
        event_name = "m-event";
        declared_facts = [];
      };
    ];
  }

let disconnected_action_program () =
  let first = oid "origin-walk-disconnected-action-a" in
  let second = oid "origin-walk-disconnected-action-b" in
  {
    (empty_program ()) with
    program_id = pid "origin-walk-disconnected-actions";
    entry_origin = Some first;
    origin_sites = [
      action_variant first "origin.walk.first" "origin.walk.first.contract";
      action_variant second "origin.walk.second" "origin.walk.second.contract";
    ];
    success_continuations = [
      { Core.from_origin = first; target = Core.Program_complete };
    ];
    capability_contracts = [
      { Core.capability_id = cid "origin.walk.first";
        contract_digest = digest "origin.walk.first.contract";
        schema_description = "first disconnected action" };
      { Core.capability_id = cid "origin.walk.second";
        contract_digest = digest "origin.walk.second.contract";
        schema_description = "second disconnected action" };
    ];
  }

let rec insert_all value = function
  | [] -> [[value]]
  | head :: tail ->
      (value :: head :: tail) ::
      (List.map (fun suffix -> head :: suffix) (insert_all value tail))

let rec permutations = function
  | [] -> [[]]
  | head :: tail ->
      List.concat (List.map (insert_all head) (permutations tail))

let origin_entries program =
  List.filter_map (fun site ->
    match Format.origin_id_of_site site with
    | Some origin_id -> Some (origin_id, site)
    | None -> None
  ) program.Core.origin_sites

let assignment_for_labels program labels =
  let ids = List.map fst (origin_entries program) in
  let origin_labels = List.fold_left2 (fun map origin_id label ->
    Format.OriginMap.add origin_id label map
  ) Format.OriginMap.empty ids labels in
  {
    Format.origin_labels;
    fact_labels = Format.FactMap.empty;
    branch_labels = Format.BranchMap.empty;
    batch_labels = Format.BatchMap.empty;
    template_labels = Format.TemplateMap.empty;
    role_labels = Format.ScopedRoleMap.empty;
  }

let payload_for_labels program labels =
  Format.encode_program (assignment_for_labels program labels) program

let first_difference left right =
  let limit = min (String.length left) (String.length right) in
  let rec find index =
    if index = limit then
      if String.length left = String.length right then None else Some index
    else if left.[index] <> right.[index] then Some index
    else find (index + 1)
  in
  find 0

let remove_one value values =
  let rec loop prefix = function
    | [] -> []
    | head :: tail when head = value -> List.rev_append prefix tail
    | head :: tail -> loop (head :: prefix) tail
  in
  loop [] values

(* This is deliberately an independent exhaustive reference for the
   decimal-boundary chains.  The entry label is fixed by the frozen first
   field law; every residual legal Origin assignment is still emitted
   through encode_program and compared byte-for-byte. *)
let fixed_entry_exhaustive_oracle program fixed_entry_label =
  let origin_count = List.length (origin_entries program) in
  let all_labels = List.init origin_count (fun index -> index + 1) in
  let residual_labels = remove_one fixed_entry_label all_labels in
  let best = ref None in
  let candidate_count = ref 0 in
  let consider permutation =
    incr candidate_count;
    let labels = 10 :: permutation in
    let payload = payload_for_labels program labels in
    match !best with
    | None -> best := Some (payload, labels)
    | Some (current, _) ->
        if Format.compare_bytes_lex_unsigned payload current < 0 then
          best := Some (payload, labels)
  in
  let rec enumerate prefix remaining =
    match remaining with
    | [] -> consider (List.rev prefix)
    | _ ->
        List.iter (fun label ->
          enumerate (label :: prefix) (remove_one label remaining)
        ) remaining
  in
  enumerate [] residual_labels;
  match !best with
  | Some result -> result, !candidate_count
  | None -> failwith "fixed-entry oracle produced no candidates"

let chain11_exhaustive_oracle program =
  fixed_entry_exhaustive_oracle program 10

let chain11_former_labels =
  10 :: 11 :: List.init 9 (fun index -> index + 1)

(* Independent test-only oracle.  It deliberately knows nothing about the
   walker state machine or its branch ordering.  It enumerates only the
   Origin family and delegates every byte to the frozen format module. *)
let oracle_payload program =
  let entries = origin_entries program in
  let ids = List.map fst entries in
  let labels = List.init (List.length ids) (fun index -> index + 1) in
  let assignment permutation =
    let origin_labels = List.fold_left2 (fun map origin_id label ->
      Format.OriginMap.add origin_id label map
    ) Format.OriginMap.empty ids permutation in
    {
      Format.origin_labels;
      fact_labels = Format.FactMap.empty;
      branch_labels = Format.BranchMap.empty;
      batch_labels = Format.BatchMap.empty;
      template_labels = Format.TemplateMap.empty;
      role_labels = Format.ScopedRoleMap.empty;
    }
  in
  match permutations labels with
  | [] -> failwith "oracle requires a non-empty Origin domain"
  | first :: rest ->
      List.fold_left (fun best permutation ->
        let candidate = Format.encode_program (assignment permutation) program in
        if Format.compare_bytes_lex_unsigned candidate best < 0
        then candidate
        else best
      ) (Format.encode_program (assignment first) program) rest

let digest_of_payload payload =
  let preimage = Bytes.cat Format.domain_v2 (Bytes.of_string payload) in
  Format.digest_string_v2 (Format.sha256_hex preimage)

let stats_equal left right =
  left.Walk.emitted_bytes = right.Walk.emitted_bytes &&
  left.forced_assignments = right.forced_assignments &&
  left.decision_points = right.decision_points &&
  left.branches_explored = right.branches_explored &&
  left.prefix_prunes = right.prefix_prunes &&
  left.completed_candidates = right.completed_candidates &&
  left.max_depth = right.max_depth

let walk_ok name ?(check_oracle = true) program =
  let oracle = if check_oracle then Some (oracle_payload program) else None in
  let results : Walk.result list = List.map (fun order ->
    match Walk.walk ~branch_order:order program with
    | Error _ ->
        Printf.eprintf "FAIL: %s rejected by Origin walker\n%!" name;
        exit 1
    | Ok result -> result
  ) [Walk.Numeric_ascending; Walk.Numeric_descending; Walk.Semantic_first] in
  let first = List.hd results in
  List.iteri (fun index result ->
    check (name ^ " branch policy " ^ string_of_int index)
      (result.Walk.payload = first.Walk.payload)
  ) results;
  begin match oracle with
  | Some expected ->
      if first.Walk.payload <> expected then
        Printf.eprintf "DIFF %s\\n  walk=%S\\n  oracle=%S\\n%!"
          name first.Walk.payload expected;
      check (name ^ " exact frozen payload") (first.Walk.payload = expected);
      check (name ^ " exact frozen digest")
        (digest_of_payload first.Walk.payload = digest_of_payload expected)
  | None -> ()
  end;
  first

let test_chain11_reproduction () =
  Printf.printf "reproducing R3-3B chain-11 counterexample\n%!";
  let program = chain_program 11 in
  let former = payload_for_labels program chain11_former_labels in
  let (expected, expected_labels), candidate_count =
    chain11_exhaustive_oracle program in
  let current = match Walk.walk program with
    | Ok result -> result
    | Error _ ->
        Printf.eprintf "FAIL: R3-3B chain-11 diagnostic rejected fixture\n%!";
        exit 1
  in
  let former_difference = first_difference former expected in
  Printf.printf
    "R3-3B1 chain-11 former_labels=[%s] exact_labels=[%s] candidates=%d former_vs_exact=%s repaired_vs_exact=%s\n%!"
    (String.concat "," (List.map string_of_int chain11_former_labels))
    (String.concat "," (List.map string_of_int expected_labels))
    candidate_count
    (match former_difference with Some index -> string_of_int index | None -> "none")
    (match first_difference current.Walk.payload expected with
     | Some index -> string_of_int index | None -> "none");
  check "chain-11 former result differs from repaired result"
    (current.Walk.payload <> former);
  check "chain-11 former/exact first difference is byte 23"
    (former_difference = Some 23);
  check "chain-11 former/exact bytes are 0x32 versus 0x31"
    (Char.code former.[23] = 0x32 && Char.code expected.[23] = 0x31);
  check "chain-11 repaired result matches exhaustive minimum"
    (current.Walk.payload = expected)

let test_chain10_exact () =
  Printf.printf "checking exact repaired chain-10 boundary\n%!";
  let program = chain_program 10 in
  let (expected, _labels), candidate_count =
    fixed_entry_exhaustive_oracle program 10 in
  check "chain-10 residual oracle enumerates 9! candidates"
    (candidate_count = 362880);
  List.iter (fun order ->
    match Walk.walk ~branch_order:order program with
    | Error _ -> check "chain-10 repaired walk succeeds" false
    | Ok result ->
        check "chain-10 repaired payload parity" (result.Walk.payload = expected);
        check "chain-10 repaired digest parity"
          (digest_of_payload result.Walk.payload = digest_of_payload expected)
  ) [Walk.Numeric_ascending; Walk.Numeric_descending; Walk.Semantic_first]

let test_small_chains () =
  List.iter (fun size ->
    Printf.printf "testing small chain %d\n%!" size;
    ignore (walk_ok ("chain-" ^ string_of_int size) (chain_program size))
  ) [1; 2; 3; 4; 5; 6; 7]

let test_origin_shapes_and_decisions () =
  Printf.printf "testing anchor/together/symmetric Origin fixtures\n%!";
  ignore (walk_ok "anchor-chain" (anchor_chain_program ()));
  ignore (walk_ok "together-origin" (together_program ()));
  let symmetric = walk_ok "symmetric-origin-domain" (symmetric_program ()) in
  check "symmetric Origin fixture has a legal exact result"
    (String.length symmetric.Walk.payload > 0);
  ignore (walk_ok "disconnected-anchor-owner-slot" (disconnected_anchor_program ()));
  ignore (walk_ok "disconnected-distinct-actions" (disconnected_action_program ()));
  begin match Walk.initial_decision (chain_program 3) with
  | Ok (Walk.NeedLabel origin_id) ->
      check "initial decision identifies entry Origin"
        (origin_id = oid "origin-walk-base-0")
  | Ok (Walk.NeedOwnerOfNumericSlot _) ->
      check "initial decision identifies entry Origin" false
  | Error _ -> check "initial decision identifies entry Origin" false
  end;
  begin match Walk.initial_decision (disconnected_anchor_program ()) with
  | Ok (Walk.NeedOwnerOfNumericSlot 1) ->
      check "initial decision exposes numeric owner slot" true
  | Ok (Walk.NeedOwnerOfNumericSlot _)
  | Ok (Walk.NeedLabel _)
  | Error _ -> check "initial decision exposes numeric owner slot" false
  end;
  begin match Walk.walk (empty_program ()) with
  | Error Walk.Empty_origin_domain -> check "empty Origin domain fails closed" true
  | Error _
  | Ok _ -> check "empty Origin domain fails closed" false
  end

let test_metamorphic_chains () =
  Printf.printf "testing metamorphic chains\n%!";
  let base = walk_ok "chain-6-base" (chain_program 6) in
  let permuted = walk_ok "chain-6-storage-reversed" (chain_program ~reverse:true 6) in
  let renamed = walk_ok "chain-6-raw-id-renamed"
      (chain_program ~tag:"renamed" 6) in
  check "storage permutation preserves payload"
    (base.Walk.payload = permuted.Walk.payload);
  check "storage permutation preserves statistics"
    (stats_equal base.Walk.stats permuted.Walk.stats);
  check "raw-ID renaming preserves payload"
    (base.Walk.payload = renamed.Walk.payload);
  check "raw-ID renaming preserves statistics"
    (stats_equal base.Walk.stats renamed.Walk.stats)

let test_scaling_chains () =
  List.iter (fun size ->
    Printf.printf "testing scaling chain %d\n%!" size;
    let result = walk_ok ("scaling-chain-" ^ string_of_int size)
        ~check_oracle:false (chain_program size) in
    Printf.printf
      "origin-walk chain=%d emitted_bytes=%d forced_assignments=%d decision_points=%d branches_explored=%d prefix_prunes=%d completed_candidates=%d max_depth=%d\n%!"
      size result.Walk.stats.emitted_bytes result.Walk.stats.forced_assignments
      result.Walk.stats.decision_points result.Walk.stats.branches_explored
      result.Walk.stats.prefix_prunes result.Walk.stats.completed_candidates
      result.Walk.stats.max_depth
  ) [10; 12]

let test_integer_boundaries () =
  Printf.printf "testing integer boundaries\n%!";
  let compare left right =
    Format.compare_bytes_lex_unsigned (Format.encode_int left)
      (Format.encode_int right)
  in
  check "encoded integer boundary 8/9" (compare 8 9 < 0);
  check "encoded integer boundary 9/10" (compare 9 10 > 0);
  check "encoded integer boundary 10/11" (compare 10 11 < 0);
  check "encoded integer boundary 11/12" (compare 11 12 < 0);
  check "encoded integer boundary 12/2" (compare 12 2 < 0)

let () =
  test_chain10_exact ();
  test_chain11_reproduction ();
  test_small_chains ();
  test_origin_shapes_and_decisions ();
  test_metamorphic_chains ();
  test_integer_boundaries ();
  test_scaling_chains ();
  Printf.printf "rocket-v3-origin-walk: %d/%d checks passed\n%!"
    !tests_passed !tests_run
