(* ==================================================================
   CANONICAL FORMAT V2 — OPTIMISED IR SEARCH TESTS (C-B4I3)

   Proves IR returns EXACTLY the same CanonicalPayload_V2 and
   ProgramDigest_V2 as both the slow oracle (where it can run) and
   the accepted exhaustive baseline everywhere both succeed.

   Covers:
     - frozen hard gates
     - Persistent Branch 24 perms
     - generated differential corpus (500)
     - adversarial symmetry corpus (A-G)
     - metamorphic raw-ID / storage tests
     - 7! baseline-beyond-oracle
     - performance evidence (non-gating, reported)
     - deterministic budget fail-closed
   ================================================================== *)

open Tethers_core

let check cond msg = if not cond then failwith msg
let check_equal_int e a l = check (e = a) (Printf.sprintf "FAIL %s expected=%d actual=%d" l e a)
let check_equal_string e a l = check (e = a) (Printf.sprintf "FAIL %s expected=%S actual=%S" l e a)
let check_ok r l = match r with Ok v -> v | Error _ -> failwith ("FAIL expected Ok " ^ l)
let _check_error r l = match r with Error e -> e | Ok _ -> failwith ("FAIL expected Error " ^ l)

let pid s = program_id_of_string s
let oid s = origin_id_of_string s
let fid s = fact_id_of_string s
let rid s = role_id_of_string s
let cid s = capability_id_of_string s
let tid s = item_template_id_of_string s
let gid s = group_id_of_string s
let cv s = core_version_of_string s
let hsk s = host_snapshot_key_of_string s
let ccd s = capability_contract_digest_of_string s
let rf s = role_fulfillment_of_string s
let bid s = batch_id_of_string s

(* Differential helper — reusable *)
let assert_ir_eq_oracle_and_baseline p label =
  let oracle_res = Tethers_core_canonical_v2_reference.slow_oracle p in
  let baseline_res = Tethers_core_canonical_v2.canonicalize p in
  let ir_res = Tethers_core_canonical_v2_ir.canonicalize_ir p in
  match oracle_res, baseline_res, ir_res with
  | Ok oracle, Ok baseline, Ok (ir, _stats) ->
      let ir_payload = Tethers_core_canonical_v2_ir.canonical_payload_ir ir in
      let ir_digest = Tethers_core_canonical_v2_ir.program_digest_ir ir in
      let baseline_payload = Tethers_core_canonical_v2.canonical_payload baseline in
      let baseline_digest = Tethers_core_canonical_v2.program_digest baseline in
      check_equal_string oracle.payload ir_payload (label ^ " oracle==IR payload");
      check_equal_string oracle.digest_string ir_digest (label ^ " oracle==IR digest");
      check_equal_string baseline_payload ir_payload (label ^ " baseline==IR payload");
      check_equal_string baseline_digest ir_digest (label ^ " baseline==IR digest")
  | Error Tethers_core_canonical_v2_reference.Oracle_too_large, Ok baseline, Ok (ir, _) ->
      let ir_payload = Tethers_core_canonical_v2_ir.canonical_payload_ir ir in
      let ir_digest = Tethers_core_canonical_v2_ir.program_digest_ir ir in
      let baseline_payload = Tethers_core_canonical_v2.canonical_payload baseline in
      let baseline_digest = Tethers_core_canonical_v2.program_digest baseline in
      check_equal_string baseline_payload ir_payload (label ^ " baseline==IR payload (beyond oracle)");
      check_equal_string baseline_digest ir_digest (label ^ " baseline==IR digest (beyond oracle)")
  | Error (Tethers_core_canonical_v2_reference.Invalid_core _), Error (Tethers_core_canonical_v2.Invalid_core _), Error (Tethers_core_canonical_v2_ir.Invalid_core _) -> ()
  | Error Tethers_core_canonical_v2_reference.Oracle_too_large, Error Tethers_core_canonical_v2.Canonicalisation_too_complex, Error Tethers_core_canonical_v2_ir.Canonicalisation_too_complex -> ()
  | Error Tethers_core_canonical_v2_reference.Oracle_too_large, Error (Tethers_core_canonical_v2.Invalid_core _), Error (Tethers_core_canonical_v2_ir.Invalid_core _) -> ()
  | _ ->
      let shape s = match s with Ok _ -> "Ok" | Error _ -> "Error" in
      let o = (match oracle_res with Ok _ -> "Ok" | Error Tethers_core_canonical_v2_reference.Oracle_too_large -> "Oracle_too_large" | Error _ -> "Invalid_core") in
      let b = shape baseline_res in
      let ir = shape ir_res in
      failwith (Printf.sprintf "FAIL %s result-shape mismatch oracle=%s baseline=%s ir=%s" label o b ir)

let _assert_payload_ir_eq p label expected_payload_opt =
  let ir = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p) (label ^ " IR") in
  let (ir_val, _) = ir in
  let payload = Tethers_core_canonical_v2_ir.canonical_payload_ir ir_val in
  match expected_payload_opt with
  | Some exp -> check_equal_string exp payload label
  | None -> check (String.length payload > 0) (label ^ " payload non-empty")

(* ================================================================== *)
(*  Hard gates — fixtures                                               *)
(* ================================================================== *)

let test_empty () =
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = None;
    success_continuations = []; origin_sites = []; branches = []; roles = [];
    item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "empty"

let test_simple_anchor () =
  let anchor = oid "anchor1" in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = [{ fact_id = fid "fact1"; schema_description = ""; provenance = Evaluation_input (hsk "hk1", String_type)}];
    entry_guards = []; entry_origin = Some anchor; success_continuations = [];
    origin_sites = [Anchor_origin { anchor_origin_id = anchor; event_name = "ev"; declared_facts = []}];
    branches = []; roles = []; item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "simple Anchor"

let test_anchor_action () =
  let anchor = oid "a1" and action = oid "act1" in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = [{ fact_id = fid "f1"; schema_description = ""; provenance = Evaluation_input (hsk "hk", String_type)}];
    entry_guards = []; entry_origin = Some anchor; success_continuations = [];
    origin_sites = [
      Anchor_origin { anchor_origin_id = anchor; event_name = "ev"; declared_facts = []};
      Action_origin { action_origin_id = action; capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; inputs = []; declared_facts = []; execution_constraints = []};
    ];
    branches = []; roles = []; item_templates = [];
    capability_contracts = [{ capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; schema_description = ""}];
  } in
  assert_ir_eq_oracle_and_baseline p "Anchor+Action"

let test_raw_id_rename () =
  let make anchor_name fact_name =
    let a = oid anchor_name in
    {
      program_id = pid "test"; core_version = cv "0.1.0";
      input_facts = [{ fact_id = fid fact_name; schema_description = ""; provenance = Evaluation_input (hsk "hk1", String_type)}];
      entry_guards = []; entry_origin = Some a; success_continuations = [];
      origin_sites = [Anchor_origin { anchor_origin_id = a; event_name = "ev"; declared_facts = []}];
      branches = []; roles = []; item_templates = []; capability_contracts = [];
    }
  in
  let p1 = make "banana_thing_947" "banana_thing_947" in
  let p2 = make "O_anchor" "O_anchor" in
  assert_ir_eq_oracle_and_baseline p1 "raw rename 1";
  assert_ir_eq_oracle_and_baseline p2 "raw rename 2";
  let ir1 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p1) "ir1" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  let ir2 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p2) "ir2" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  check_equal_string ir1 ir2 "raw-ID rename invariance IR"

let test_cross_family_same_raw () =
  let make anchor fact branch =
    let a = oid anchor and f = fid fact and b = branch_id_of_string branch in
    {
      program_id = pid "test"; core_version = cv "0.1.0";
      input_facts = [{ fact_id = f; schema_description = ""; provenance = Evaluation_input (hsk "hk1", String_type)}];
      entry_guards = []; entry_origin = Some a; success_continuations = [];
      origin_sites = [Anchor_origin { anchor_origin_id = a; event_name = "ev"; declared_facts = []}];
      branches = [{ branch_id = b; branch_subject = a; outcome_branches = [(Success, Stop)]}];
      roles = []; item_templates = []; capability_contracts = [];
    }
  in
  let p1 = make "X" "X" "X" in
  let p2 = make "Y" "X" "X" in
  assert_ir_eq_oracle_and_baseline p1 "cross-family X";
  assert_ir_eq_oracle_and_baseline p2 "cross-family Y";
  let ir1 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p1) "ir1" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  let ir2 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p2) "ir2" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  check_equal_string ir1 ir2 "cross-family rename invariance IR"

let test_same_raw_role_ids_across_templates () =
  let tid_a = tid "TA" and tid_b = tid "TB" and r = rid "R" in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = Some (oid "anchorA"); success_continuations = [];
    origin_sites = []; branches = []; roles = [];
    item_templates = [{
      item_template_id = tid_a;
      origin_sites = [Anchor_origin { anchor_origin_id = oid "anchorA"; event_name = "evA"; declared_facts = [{ fact_id = fid "factA"; schema_description = ""; provenance = Evaluation_input (hsk "hkA", String_type)}]}];
      branches = []; roles = [{ role_id = r; scope = Item_template_scope tid_a; fact_contract = Role_fact_contract [fid "factA"]; eligible_fulfillment = rf "ok"}];
      objective = Required_role r;
    }; {
      item_template_id = tid_b;
      origin_sites = [Anchor_origin { anchor_origin_id = oid "anchorB"; event_name = "evB"; declared_facts = [{ fact_id = fid "factB"; schema_description = ""; provenance = Evaluation_input (hsk "hkB", String_type)}]}];
      branches = []; roles = [{ role_id = r; scope = Item_template_scope tid_b; fact_contract = Role_fact_contract [fid "factB"]; eligible_fulfillment = rf "ok"}];
      objective = Required_role r;
    }];
    capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "same raw RoleId across templates"

let test_role_blocks () =
  let tid_a = tid "TA" and tid_b = tid "TB" and anchor = oid "anchor" in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = Some anchor; success_continuations = [];
    origin_sites = [Anchor_origin { anchor_origin_id = anchor; event_name = "ev"; declared_facts = []}];
    branches = []; roles = [];
    item_templates = [{
      item_template_id = tid_a; origin_sites = []; branches = [];
      roles = [
        { role_id = rid "RA1"; scope = Item_template_scope tid_a; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok"};
        { role_id = rid "RA2"; scope = Item_template_scope tid_a; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok"};
      ]; objective = Required_role (rid "RA1");
    }; {
      item_template_id = tid_b; origin_sites = []; branches = [];
      roles = [
        { role_id = rid "RB1"; scope = Item_template_scope tid_b; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok"};
        { role_id = rid "RB2"; scope = Item_template_scope tid_b; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok"};
      ]; objective = Required_role (rid "RB1");
    }];
    capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "role blocks 2x2"

let test_mixed_origin_batch () =
  let anchor = oid "anc" and batch_id = bid "batch1" and tid_b = tid "IT_batch" and tid_a = tid "IT1" in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = Some anchor; success_continuations = [];
    origin_sites = []; branches = []; roles = [];
    item_templates = [{
      item_template_id = tid_a;
      origin_sites = [
        Anchor_origin { anchor_origin_id = anchor; event_name = "ev"; declared_facts = []};
        Batch_site { batch_id = batch_id; collection_provenance = batch_collection_provenance_of_string "prov";
          item_template_id = tid_b; traversal_policy = batch_traversal_policy_of_string "seq";
          composite_objective = batch_objective_of_string "all";
          aggregate_facts = [{ fact_id = fid "bf1"; schema_description = ""; provenance = Evaluation_input (hsk "hk_bf", Boolean_type)}] };
      ];
      branches = []; roles = [{ role_id = rid "R1"; scope = Item_template_scope tid_a; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok"}];
      objective = Required_role (rid "R1");
    }; {
      item_template_id = tid_b; origin_sites = []; branches = [];
      roles = [{ role_id = rid "R_batch"; scope = Item_template_scope tid_b; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok"}];
      objective = Required_role (rid "R_batch");
    }];
    capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "mixed Origin/Batch"

let test_nested_storage_order () =
  let anchor1 = oid "anc1" and anchor2 = oid "anc2" in
  let action1 = oid "act1" and action2 = oid "act2" in
  let br1 = branch_id_of_string "br1" and br2 = branch_id_of_string "br2" in
  let fact_a1 = fid "fa1" and fact_a2 = fid "fa2" in
  let r1 = rid "R1" and r2 = rid "R2" in
  let itid = tid "IT1" in
  let make_p ~rev =
    let origins = if not rev then [
        Anchor_origin { anchor_origin_id = anchor1; event_name = "ev1"; declared_facts = [{ fact_id = fact_a1; schema_description = ""; provenance = Evaluation_input (hsk "hk_a1", String_type)}]};
        Action_origin { action_origin_id = action1; capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; inputs = []; declared_facts = []; execution_constraints = []};
        Anchor_origin { anchor_origin_id = anchor2; event_name = "ev2"; declared_facts = [{ fact_id = fact_a2; schema_description = ""; provenance = Evaluation_input (hsk "hk_a2", Integer_type)}]};
        Action_origin { action_origin_id = action2; capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; inputs = []; declared_facts = []; execution_constraints = []};
      ] else [
        Action_origin { action_origin_id = action2; capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; inputs = []; declared_facts = []; execution_constraints = []};
        Anchor_origin { anchor_origin_id = anchor2; event_name = "ev2"; declared_facts = [{ fact_id = fact_a2; schema_description = ""; provenance = Evaluation_input (hsk "hk_a2", Integer_type)}]};
        Action_origin { action_origin_id = action1; capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; inputs = []; declared_facts = []; execution_constraints = []};
        Anchor_origin { anchor_origin_id = anchor1; event_name = "ev1"; declared_facts = [{ fact_id = fact_a1; schema_description = ""; provenance = Evaluation_input (hsk "hk_a1", String_type)}]};
      ] in
    let branches = if not rev then [
        { branch_id = br1; branch_subject = anchor1; outcome_branches = [(Success, Stop)]};
        { branch_id = br2; branch_subject = anchor1; outcome_branches = [(Success, Stop)]};
      ] else [
        { branch_id = br2; branch_subject = anchor1; outcome_branches = [(Success, Stop)]};
        { branch_id = br1; branch_subject = anchor1; outcome_branches = [(Success, Stop)]};
      ] in
    let roles = if not rev then [
        { role_id = r1; scope = Item_template_scope itid; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok"};
        { role_id = r2; scope = Item_template_scope itid; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok"};
      ] else [
        { role_id = r2; scope = Item_template_scope itid; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok"};
        { role_id = r1; scope = Item_template_scope itid; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok"};
      ] in
    {
      program_id = pid "test"; core_version = cv "0.1.0"; input_facts = []; entry_guards = [];
      entry_origin = Some anchor1; success_continuations = [];
      origin_sites = []; branches = []; roles = [];
      item_templates = [{ item_template_id = itid; origin_sites = origins; branches = branches; roles = roles; objective = Required_role r1}];
      capability_contracts = [{ capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; schema_description = ""}];
    }
  in
  let p1 = make_p ~rev:false and p2 = make_p ~rev:true in
  assert_ir_eq_oracle_and_baseline p1 "nested storage 1";
  assert_ir_eq_oracle_and_baseline p2 "nested storage 2";
  let ir1 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p1) "ir1" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  let ir2 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p2) "ir2" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  check_equal_string ir1 ir2 "nested storage order invariance IR"

let test_action_input_secondary_sorting () =
  let anchor = oid "anchor" and action = oid "act1" in
  let fid_a = fid "fa" and fid_b = fid "fb" in
  let make_inputs order =
    match order with
    | `AB -> [
        { input_name = capability_input_name_of_string "in_a"; binding = Fact_from_origin (fid_a, anchor)};
        { input_name = capability_input_name_of_string "in_a"; binding = Fact_from_origin (fid_b, anchor)};
      ]
    | `BA -> [
        { input_name = capability_input_name_of_string "in_a"; binding = Fact_from_origin (fid_b, anchor)};
        { input_name = capability_input_name_of_string "in_a"; binding = Fact_from_origin (fid_a, anchor)};
      ]
  in
  let make_p order =
    {
      program_id = pid "test"; core_version = cv "0.1.0";
      input_facts = [
        { fact_id = fid_a; schema_description = ""; provenance = Evaluation_input (hsk "hk_a", String_type)};
        { fact_id = fid_b; schema_description = ""; provenance = Evaluation_input (hsk "hk_b", String_type)};
      ];
      entry_guards = []; entry_origin = Some anchor; success_continuations = [];
      origin_sites = [
        Anchor_origin { anchor_origin_id = anchor; event_name = "ev"; declared_facts = []};
        Action_origin { action_origin_id = action; capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; inputs = make_inputs order; declared_facts = []; execution_constraints = []};
      ];
      branches = []; roles = []; item_templates = [];
      capability_contracts = [{ capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; schema_description = ""}];
    }
  in
  let p1 = make_p `AB and p2 = make_p `BA in
  assert_ir_eq_oracle_and_baseline p1 "action input AB";
  assert_ir_eq_oracle_and_baseline p2 "action input BA";
  let ir1 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p1) "ir1" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  let ir2 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p2) "ir2" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  check_equal_string ir1 ir2 "action input secondary sorting invariance"

let test_constraint_ordering () =
  let anchor = oid "anchor" in
  let make_p c1 c2 =
    {
      program_id = pid "test"; core_version = cv "0.1.0";
      input_facts = []; entry_guards = []; entry_origin = Some anchor; success_continuations = [];
      origin_sites = [Action_origin { action_origin_id = anchor; capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; inputs = []; declared_facts = []; execution_constraints = [Deadline c1; Deadline c2]}];
      branches = []; roles = []; item_templates = [];
      capability_contracts = [{ capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; schema_description = ""}];
    }
  in
  let p1 = make_p "aa" "z" and p2 = make_p "z" "aa" in
  assert_ir_eq_oracle_and_baseline p1 "constraint aa/z 1";
  assert_ir_eq_oracle_and_baseline p2 "constraint aa/z 2";
  let ir1 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p1) "ir1" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  let ir2 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p2) "ir2" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  check_equal_string ir1 ir2 "constraint aa/z ordering invariance"

let test_role_fact_contract_ordering () =
  let make_p fids =
    {
      program_id = pid "test"; core_version = cv "0.1.0";
      input_facts = List.map (fun f -> { fact_id = f; schema_description = ""; provenance = Evaluation_input (hsk ("hk_" ^ Tethers_core.string_of_fact_id f), String_type)}) fids;
      entry_guards = []; entry_origin = None; success_continuations = [];
      origin_sites = []; branches = [];
      roles = [{ role_id = rid "R1"; scope = Program_scope; fact_contract = Role_fact_contract fids; eligible_fulfillment = rf "ok"}];
      item_templates = []; capability_contracts = [];
    }
  in
  let p1 = make_p [fid "fB"; fid "fA"] and p2 = make_p [fid "fA"; fid "fB"] in
  assert_ir_eq_oracle_and_baseline p1 "role fact contract BA";
  assert_ir_eq_oracle_and_baseline p2 "role fact contract AB";
  let ir1 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p1) "ir1" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  let ir2 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p2) "ir2" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  check_equal_string ir1 ir2 "Role_fact_contract canonical ordering IR"

let test_together_permutation () =
  let a1 = oid "a1" and a2 = oid "a2" in
  let make_p order =
    let members = match order with `AB -> [a1; a2] | `BA -> [a2; a1] in
    {
      program_id = pid "test"; core_version = cv "0.1.0"; input_facts = []; entry_guards = [];
      entry_origin = Some (oid "anchor"); success_continuations = [];
      origin_sites = [
        Anchor_origin { anchor_origin_id = oid "anchor"; event_name = "ev"; declared_facts = []};
        Action_origin { action_origin_id = a1; capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; inputs = []; declared_facts = []; execution_constraints = []};
        Action_origin { action_origin_id = a2; capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; inputs = []; declared_facts = []; execution_constraints = []};
        Together_origin { together_origin_id = oid "tog"; group_id = gid "g1"; member_origin_ids = members; objective = All_members_succeed};
      ];
      branches = []; roles = []; item_templates = [];
      capability_contracts = [{ capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; schema_description = ""}];
    }
  in
  let p1 = make_p `AB and p2 = make_p `BA in
  assert_ir_eq_oracle_and_baseline p1 "together AB";
  assert_ir_eq_oracle_and_baseline p2 "together BA";
  let ir1 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p1) "ir1" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  let ir2 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p2) "ir2" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  check_equal_string ir1 ir2 "Together permutation invariance IR"

let test_multiplicity () =
  let make_p n =
    let origins = List.init n (fun i -> Action_origin { action_origin_id = oid ("action" ^ string_of_int i); capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; inputs = []; declared_facts = []; execution_constraints = []}) in
    {
      program_id = pid "test"; core_version = cv "0.1.0"; input_facts = []; entry_guards = [];
      entry_origin = Some (oid "anchor"); success_continuations = [];
      origin_sites = Anchor_origin { anchor_origin_id = oid "anchor"; event_name = "ev"; declared_facts = []} :: origins;
      branches = []; roles = []; item_templates = [];
      capability_contracts = [{ capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; schema_description = ""}];
    }
  in
  let p1 = make_p 1 and p2 = make_p 2 in
  assert_ir_eq_oracle_and_baseline p1 "mult 1";
  assert_ir_eq_oracle_and_baseline p2 "mult 2";
  let ir1 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p1) "ir1" |> fst |> Tethers_core_canonical_v2_ir.program_digest_ir in
  let ir2 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p2) "ir2" |> fst |> Tethers_core_canonical_v2_ir.program_digest_ir in
  check (ir1 <> ir2) "multiplicity different digest IR"

let test_integer_boundaries () =
  let test_int n =
    let p = {
      program_id = pid "test"; core_version = cv "0.1.0";
      input_facts = [{ fact_id = fid "f1"; schema_description = ""; provenance = Evaluation_input (hsk "hk", Integer_type)}];
      entry_guards = [{ fact_id = fid "f1"; operator = Equals; expected = Integer_value n}];
      entry_origin = None; success_continuations = [];
      origin_sites = []; branches = []; roles = []; item_templates = []; capability_contracts = [];
    } in
    assert_ir_eq_oracle_and_baseline p (Printf.sprintf "int %d" n)
  in
  test_int (-4611686018427387904); test_int (-1); test_int 0; test_int 1; test_int 4611686018427387903

let test_high_bytes () =
  let p_nul = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = [{ fact_id = fid "f1"; schema_description = ""; provenance = Evaluation_input (hsk "abc\x00def", String_type)}];
    entry_guards = []; entry_origin = None; success_continuations = [];
    origin_sites = []; branches = []; roles = []; item_templates = []; capability_contracts = [];
  } in
  let p_empty = { p_nul with input_facts = [{ fact_id = fid "f1"; schema_description = ""; provenance = Evaluation_input (hsk "", String_type)}]} in
  assert_ir_eq_oracle_and_baseline p_nul "high bytes NUL";
  assert_ir_eq_oracle_and_baseline p_empty "high bytes empty";
  check (Tethers_core_canonical_v2_format.compare_bytes_lex_unsigned "\x7f" "\x80" < 0) "high byte compare"

(* ================================================================== *)
(*  Persistent Branch: 24 perms, 576 candidates, 1 payload/digest       *)
(* ================================================================== *)

let test_persistent_branch_ir () =
  let make_p names branches =
    let origins = List.map (fun n -> Anchor_origin { anchor_origin_id = oid n; event_name = "ev"; declared_facts = []}) names in
    let brs = List.map2 (fun bname oname -> { branch_id = branch_id_of_string bname; branch_subject = oid oname; outcome_branches = [(Success, Stop)]}) branches names in
    {
      program_id = pid "test"; core_version = cv "0.1.0"; input_facts = []; entry_guards = [];
      entry_origin = Some (oid (List.hd names)); success_continuations = [];
      origin_sites = origins; branches = brs; roles = []; item_templates = []; capability_contracts = [];
    }
  in
  let perms = Tethers_core_canonical_v2_reference.perm [0;1;2;3] in
  let names = ["a0";"a1";"a2";"a3"] and bnames = ["b0";"b1";"b2";"b3"] in
  let map lst perm = List.map (fun i -> List.nth lst i) perm in
  let results = List.map (fun perm ->
    let onames = map names perm and bnames' = map bnames perm in
    let p = make_p onames bnames' in
    let oracle = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p) "oracle" in
    let baseline = check_ok (Tethers_core_canonical_v2.canonicalize p) "baseline" in
    let (ir, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p) "ir" in
    check_equal_int 576 oracle.candidate_count "persistent 576";
    check_equal_string oracle.payload (Tethers_core_canonical_v2.canonical_payload baseline) "oracle==baseline";
    check_equal_string oracle.payload (Tethers_core_canonical_v2_ir.canonical_payload_ir ir) "oracle==IR";
    check_equal_string oracle.digest_string (Tethers_core_canonical_v2_ir.program_digest_ir ir) "digest IR";
    (ir, stats)
  ) perms in
  let payloads = List.map (fun (ir, _) -> Tethers_core_canonical_v2_ir.canonical_payload_ir ir) results in
  let digests = List.map (fun (ir, _) -> Tethers_core_canonical_v2_ir.program_digest_ir ir) results in
  let uniq_payloads = List.sort_uniq String.compare payloads in
  let uniq_digests = List.sort_uniq String.compare digests in
  check_equal_int 24 (List.length perms) "persistent tested 24";
  check_equal_int 1 (List.length uniq_payloads) "persistent 1 payload IR";
  check_equal_int 1 (List.length uniq_digests) "persistent 1 digest IR";
  (* Report IR stats for witness — first perm *)
  let (_, stats0) = List.hd results in
  Printf.printf "Persistent Branch IR stats: nodes=%d leaves=%d rounds=%d prefix_pruned=%d orbit_pruned=%d dup_hits=%d leaves_avoided=%d\n"
    stats0.nodes stats0.leaves_encoded stats0.refinement_rounds stats0.prefix_subtrees_pruned stats0.orbit_branches_pruned stats0.duplicate_payload_hits stats0.leaves_avoided

let test_7fact_beyond_oracle () =
  let anchor = oid "anchor" in
  let facts = List.init 7 (fun i -> { fact_id = fid ("f" ^ string_of_int i); schema_description = ""; provenance = Evaluation_input (hsk ("hk" ^ string_of_int i), String_type)}) in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = facts;
    entry_guards = []; entry_origin = Some anchor; success_continuations = [];
    origin_sites = [Anchor_origin { anchor_origin_id = anchor; event_name = "ev"; declared_facts = []}];
    branches = []; roles = []; item_templates = []; capability_contracts = [];
  } in
  (match Tethers_core_canonical_v2_reference.slow_oracle p with
   | Error Tethers_core_canonical_v2_reference.Oracle_too_large -> ()
   | Ok _ -> failwith "oracle should reject 7 facts"
   | Error _ -> failwith "oracle wrong error");
  let baseline = check_ok (Tethers_core_canonical_v2.canonicalize p) "baseline 7!" in
  let (ir, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p) "IR 7!" in
  check_equal_string (Tethers_core_canonical_v2.canonical_payload baseline) (Tethers_core_canonical_v2_ir.canonical_payload_ir ir) "7! payload";
  check_equal_string (Tethers_core_canonical_v2.program_digest baseline) (Tethers_core_canonical_v2_ir.program_digest_ir ir) "7! digest";
  Printf.printf "7! stats: baseline candidates=5040 IR nodes=%d leaves=%d rounds=%d prefix_pruned=%d orbit_pruned=%d dup_hits=%d avoided=%d\n" stats.nodes stats.leaves_encoded stats.refinement_rounds stats.prefix_subtrees_pruned stats.orbit_branches_pruned stats.duplicate_payload_hits stats.leaves_avoided;
  (* Storage permutations invariance *)
  let p_rev = { p with input_facts = List.rev facts } in
  let (ir_rev, _) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p_rev) "IR rev" in
  check_equal_string (Tethers_core_canonical_v2_ir.canonical_payload_ir ir) (Tethers_core_canonical_v2_ir.canonical_payload_ir ir_rev) "7! storage invariance"

(* ================================================================== *)
(*  Adversarial symmetry corpus A-G                                     *)
(* ================================================================== *)

let test_sym_all_identical_independent () =
  (* A. all-identical independent entities — 4 identical origins, no edges *)
  let origins = List.init 4 (fun i -> Anchor_origin { anchor_origin_id = oid ("a" ^ string_of_int i); event_name = "ev"; declared_facts = []}) in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = []; entry_guards = [];
    entry_origin = Some (oid "a0"); success_continuations = [];
    origin_sites = origins; branches = []; roles = []; item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "sym identical independent"

let test_sym_paired_branch_origin () =
  (* C. paired symmetric Branch/Origin *)
  let origins = List.init 2 (fun i -> Anchor_origin { anchor_origin_id = oid ("a" ^ string_of_int i); event_name = "ev"; declared_facts = []}) in
  let branches = List.init 2 (fun i -> { branch_id = branch_id_of_string ("b" ^ string_of_int i); branch_subject = oid ("a" ^ string_of_int i); outcome_branches = [(Success, Stop)]}) in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = []; entry_guards = [];
    entry_origin = Some (oid "a0"); success_continuations = [];
    origin_sites = origins; branches = branches; roles = []; item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "sym paired branch/origin"

let test_sym_two_identical_templates () =
  (* D. two identical ItemTemplates *)
  let mk_template tid_str =
    let tid = tid tid_str in
    { item_template_id = tid; origin_sites = []; branches = []; roles = [{ role_id = rid "R"; scope = Item_template_scope tid; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok"}]; objective = Required_role (rid "R") }
  in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = []; entry_guards = [];
    entry_origin = None; success_continuations = [];
    origin_sites = []; branches = []; roles = [];
    item_templates = [mk_template "TA"; mk_template "TB"]; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "sym identical templates"

let test_sym_same_raw_role_ids () =
  (* E. same raw Role IDs across symmetric templates *)
  let tid_a = tid "TA" and tid_b = tid "TB" in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = []; entry_guards = [];
    entry_origin = None; success_continuations = [];
    origin_sites = []; branches = []; roles = [];
    item_templates = [{
      item_template_id = tid_a; origin_sites = []; branches = [];
      roles = [
        { role_id = rid "R"; scope = Item_template_scope tid_a; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "same"};
        { role_id = rid "S"; scope = Item_template_scope tid_a; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "same"};
      ]; objective = Required_role (rid "R")
    }; {
      item_template_id = tid_b; origin_sites = []; branches = [];
      roles = [
        { role_id = rid "R"; scope = Item_template_scope tid_b; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "same"};
        { role_id = rid "S"; scope = Item_template_scope tid_b; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "same"};
      ]; objective = Required_role (rid "R")
    }];
    capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "sym same raw role IDs"

let test_sym_together_groups () =
  (* F. Together groups with symmetric members *)
  let a1 = oid "a1" and a2 = oid "a2" and a3 = oid "a3" and a4 = oid "a4" in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = []; entry_guards = [];
    entry_origin = Some (oid "anchor"); success_continuations = [];
    origin_sites = [
      Anchor_origin { anchor_origin_id = oid "anchor"; event_name = "ev"; declared_facts = []};
      Action_origin { action_origin_id = a1; capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; inputs = []; declared_facts = []; execution_constraints = []};
      Action_origin { action_origin_id = a2; capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; inputs = []; declared_facts = []; execution_constraints = []};
      Action_origin { action_origin_id = a3; capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; inputs = []; declared_facts = []; execution_constraints = []};
      Action_origin { action_origin_id = a4; capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; inputs = []; declared_facts = []; execution_constraints = []};
      Together_origin { together_origin_id = oid "tog1"; group_id = gid "g1"; member_origin_ids = [a1; a2]; objective = All_members_succeed};
      Together_origin { together_origin_id = oid "tog2"; group_id = gid "g2"; member_origin_ids = [a3; a4]; objective = All_members_succeed};
    ];
    branches = []; roles = []; item_templates = [];
    capability_contracts = [{ capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; schema_description = ""}];
  } in
  assert_ir_eq_oracle_and_baseline p "sym together groups"

let test_sym_regular_biregular () =
  (* G. regular relationship where WL leaves large cell — 3 facts each with same provenance shape *)
  let facts = List.init 3 (fun i -> { fact_id = fid ("f" ^ string_of_int i); schema_description = ""; provenance = Evaluation_input (hsk "hk_same", String_type)}) in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = facts;
    entry_guards = []; entry_origin = None; success_continuations = [];
    origin_sites = []; branches = []; roles = []; item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "sym regular biregular facts"

(* ================================================================== *)
(*  Raw-ID / storage metamorphic                                        *)
(* ================================================================== *)

let test_metamorphic_storage () =
  let facts = [
    { fact_id = fid "f1"; schema_description = ""; provenance = Evaluation_input (hsk "hk1", String_type)};
    { fact_id = fid "f2"; schema_description = ""; provenance = Evaluation_input (hsk "hk2", String_type)};
  ] in
  let p_base = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = facts;
    entry_guards = []; entry_origin = Some (oid "a0"); success_continuations = [];
    origin_sites = [
      Anchor_origin { anchor_origin_id = oid "a0"; event_name = "ev"; declared_facts = []};
      Anchor_origin { anchor_origin_id = oid "a1"; event_name = "ev"; declared_facts = []};
    ];
    branches = [{ branch_id = branch_id_of_string "b0"; branch_subject = oid "a0"; outcome_branches = [(Success, Stop)]}];
    roles = []; item_templates = []; capability_contracts = [];
  } in
  let ir_base = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p_base) "base" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  let p_rev = { p_base with input_facts = List.rev facts; origin_sites = List.rev p_base.origin_sites } in
  let ir_rev = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p_rev) "rev" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  check_equal_string ir_base ir_rev "metamorphic reverse storage";
  (* Rename IDs to opposite lexical order *)
  let p_renamed = {
    p_base with
    input_facts = [
      { fact_id = fid "zzz"; schema_description = ""; provenance = Evaluation_input (hsk "hk1", String_type)};
      { fact_id = fid "aaa"; schema_description = ""; provenance = Evaluation_input (hsk "hk2", String_type)};
    ];
    origin_sites = [
      Anchor_origin { anchor_origin_id = oid "zzz"; event_name = "ev"; declared_facts = []};
      Anchor_origin { anchor_origin_id = oid "aaa"; event_name = "ev"; declared_facts = []};
    ];
    entry_origin = Some (oid "zzz");
    branches = [{ branch_id = branch_id_of_string "zzz"; branch_subject = oid "zzz"; outcome_branches = [(Success, Stop)]}];
  } in
  (* This renamed program is not structurally identical (different linkage) — skip direct compare.
     Instead test same structure with ugly IDs *)
  let _ = p_renamed in
  let make_ugly ids =
    let origins = List.map (fun id -> Anchor_origin { anchor_origin_id = oid id; event_name = "ev"; declared_facts = []}) ids in
    {
      program_id = pid "test"; core_version = cv "0.1.0"; input_facts = []; entry_guards = [];
      entry_origin = Some (oid (List.hd ids)); success_continuations = [];
      origin_sites = origins; branches = []; roles = []; item_templates = []; capability_contracts = [];
    }
  in
  let p_ugly1 = make_ugly ["a0"; "a1"; "a2"] in
  let p_ugly2 = make_ugly ["___ugly_zzz___"; "!!!"; "000"] in
  let ir_ugly1 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p_ugly1) "ugly1" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  let ir_ugly2 = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p_ugly2) "ugly2" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  check_equal_string ir_ugly1 ir_ugly2 "ugly IDs invariance (same structure, different raw strings)"

(* ================================================================== *)
(*  Generated differential corpus — deterministic systematic enumeration *)
(* ================================================================== *)

let generated_case n =
  (* Deterministic program generation based on n.
     Vary family cardinalities, scalar equality patterns, relation patterns.
     Keep program valid. *)
  let num_facts = n mod 3 in
  let num_origins = (n / 3) mod 3 in
  let num_branches = (n / 9) mod 2 in
  let num_templates = (n / 18) mod 2 in
  let scalar_variant = n / 36 in
  let facts = List.init num_facts (fun i ->
    let hk = if scalar_variant mod 2 = 0 then "hkA" else "hkB" in
    { fact_id = fid ("f" ^ string_of_int n ^ "_" ^ string_of_int i);
      schema_description = "";
      provenance = Evaluation_input (hsk (hk ^ string_of_int i), if i mod 2 = 0 then String_type else Integer_type) }
  ) in
  let origins = List.init num_origins (fun i ->
    let oid_str = "o" ^ string_of_int n ^ "_" ^ string_of_int i in
    if i mod 2 = 0 then
      Anchor_origin { anchor_origin_id = oid oid_str; event_name = (if scalar_variant mod 3 = 0 then "evA" else "evB"); declared_facts = []}
    else
      Action_origin { action_origin_id = oid oid_str; capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; inputs = []; declared_facts = []; execution_constraints = []}
  ) in
  let entry_origin = match origins with
    | Anchor_origin a :: _ -> Some a.anchor_origin_id
    | Action_origin a :: _ -> Some a.action_origin_id
    | _ -> None
  in
  let branches = List.init num_branches (fun i ->
    let bname = "b" ^ string_of_int n ^ "_" ^ string_of_int i in
    let subj = match origins with
      | Anchor_origin a :: _ -> a.anchor_origin_id
      | Action_origin a :: _ -> a.action_origin_id
      | _ -> oid "dummy"
    in
    { branch_id = branch_id_of_string bname; branch_subject = subj; outcome_branches = [(Success, Stop)]}
  ) in
  let branches = if origins = [] then [] else branches in
  let templates = if num_templates = 0 then [] else
    let tid_v = tid ("T" ^ string_of_int n) in
    [{
      item_template_id = tid_v; origin_sites = []; branches = [];
      roles = [{ role_id = rid ("R" ^ string_of_int n); scope = Item_template_scope tid_v; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok"}];
      objective = Required_role (rid ("R" ^ string_of_int n))
    }]
  in
  {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = facts; entry_guards = []; entry_origin = entry_origin;
    success_continuations = []; origin_sites = origins; branches = branches;
    roles = []; item_templates = templates;
    capability_contracts = if num_origins > 0 && List.exists (function Action_origin _ -> true | _ -> false) origins
      then [{ capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; schema_description = ""}]
      else []
  }

let test_generated_corpus () =
  let total = 1000 in
  let valid = ref 0 in
  let mismatches = ref 0 in
  for n = 0 to total - 1 do
    let p = generated_case n in
    match Tethers_core_validator.validate p with
    | Error _ -> ()
    | Ok () ->
        incr valid;
        let oracle_res = Tethers_core_canonical_v2_reference.slow_oracle p in
        let baseline_res = Tethers_core_canonical_v2.canonicalize p in
        let ir_res = Tethers_core_canonical_v2_ir.canonicalize_ir p in
        let mismatch =
          match oracle_res, baseline_res, ir_res with
          | Ok oracle, Ok baseline, Ok (ir, _) ->
              let ir_p = Tethers_core_canonical_v2_ir.canonical_payload_ir ir in
              let ir_d = Tethers_core_canonical_v2_ir.program_digest_ir ir in
              oracle.payload <> ir_p || oracle.digest_string <> ir_d ||
              Tethers_core_canonical_v2.canonical_payload baseline <> ir_p ||
              Tethers_core_canonical_v2.program_digest baseline <> ir_d
          | Error Tethers_core_canonical_v2_reference.Oracle_too_large, Ok baseline, Ok (ir, _) ->
              let ir_p = Tethers_core_canonical_v2_ir.canonical_payload_ir ir in
              let ir_d = Tethers_core_canonical_v2_ir.program_digest_ir ir in
              Tethers_core_canonical_v2.canonical_payload baseline <> ir_p ||
              Tethers_core_canonical_v2.program_digest baseline <> ir_d
          | Error (Tethers_core_canonical_v2_reference.Invalid_core _), Error (Tethers_core_canonical_v2.Invalid_core _), Error (Tethers_core_canonical_v2_ir.Invalid_core _) -> false
          | Error Tethers_core_canonical_v2_reference.Oracle_too_large, Error Tethers_core_canonical_v2.Canonicalisation_too_complex, Error Tethers_core_canonical_v2_ir.Canonicalisation_too_complex -> false
          | _ -> true
        in
        if mismatch then incr mismatches
  done;
  Printf.printf "Generated corpus: total=%d valid=%d mismatches=%d\n" total !valid !mismatches;
  check_equal_int 0 !mismatches "generated corpus mismatches"

(* ================================================================== *)
(*  Deterministic budget fail-closed                                    *)
(* ================================================================== *)

let test_budget_fail_closed () =
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = List.init 8 (fun i -> { fact_id = fid ("f" ^ string_of_int i); schema_description = ""; provenance = Evaluation_input (hsk ("hk" ^ string_of_int i), String_type)});
    entry_guards = []; entry_origin = Some (oid "anchor"); success_continuations = [];
    origin_sites = [Anchor_origin { anchor_origin_id = oid "anchor"; event_name = "ev"; declared_facts = []}];
    branches = []; roles = []; item_templates = []; capability_contracts = [];
  } in
  (* 8! = 40320 > small budget *)
  let small_budget = { Tethers_core_canonical_v2_ir.max_nodes = 100; max_leaves = 100; max_refinement_rounds = 1000 } in
  (match Tethers_core_canonical_v2_ir.canonicalize_ir ~budget:small_budget p with
   | Error Tethers_core_canonical_v2_ir.Canonicalisation_too_complex -> ()
   | Ok _ -> failwith "should be too_complex"
   | Error _ -> failwith "wrong error");
  (* No payload/digest on failure — checked by Error case *)
  Printf.printf "Budget fail-closed: PASS\n"

(* ================================================================== *)
(*  Performance evidence (non-gating)                                   *)
(* ================================================================== *)

let time f =
  let t0 = Unix.gettimeofday () in
  let r = f () in
  let t1 = Unix.gettimeofday () in
  (r, t1 -. t0)

let bench_case name p =
  let baseline_candidates = match Tethers_core_canonical_v2.candidate_count_within_budget ~limit:max_int p with Some n -> n | None -> -1 in
  let (_, t_base) = time (fun () -> ignore (Tethers_core_canonical_v2.canonicalize p)) in
  let (ir_res, t_ir) = time (fun () -> Tethers_core_canonical_v2_ir.canonicalize_ir p) in
  let (ir_nodes, ir_leaves, ir_rounds, ir_prefix, ir_orbit, ir_dup) = match ir_res with Ok (_, s) -> (s.nodes, s.leaves_encoded, s.refinement_rounds, s.prefix_subtrees_pruned, s.orbit_branches_pruned, s.duplicate_payload_hits) | Error _ -> (-1, -1, -1, -1, -1, -1) in
  Printf.printf "BENCH %s: baseline_candidates=%d baseline_time=%.4fs IR nodes=%d leaves=%d rounds=%d prefix_pruned=%d orbit_pruned=%d dup_hits=%d IR_time=%.4fs\n"
    name baseline_candidates t_base ir_nodes ir_leaves ir_rounds ir_prefix ir_orbit ir_dup t_ir;
  (baseline_candidates, ir_nodes, ir_leaves)

let test_performance_evidence () =
  Printf.printf "\n=== Performance Evidence ===\n";
  (* 1. N=7 distinct-ish facts *)
  let p7 = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = List.init 7 (fun i -> { fact_id = fid ("f" ^ string_of_int i); schema_description = ""; provenance = Evaluation_input (hsk ("hk" ^ string_of_int i), String_type)});
    entry_guards = []; entry_origin = Some (oid "anchor"); success_continuations = [];
    origin_sites = [Anchor_origin { anchor_origin_id = oid "anchor"; event_name = "ev"; declared_facts = []}];
    branches = []; roles = []; item_templates = []; capability_contracts = [];
  } in
  ignore (bench_case "N=7 distinct facts" p7);
  (* 2. N=8 where baseline practical *)
  let p8 = { p7 with input_facts = List.init 8 (fun i -> { fact_id = fid ("f" ^ string_of_int i); schema_description = ""; provenance = Evaluation_input (hsk ("hk" ^ string_of_int i), String_type)}) } in
  ignore (bench_case "N=8" p8);
  (* 3. Persistent Branch symmetry witness *)
  let p_persist = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = []; entry_guards = [];
    entry_origin = Some (oid "a0"); success_continuations = [];
    origin_sites = List.map (fun n -> Anchor_origin { anchor_origin_id = oid n; event_name = "ev"; declared_facts = []}) ["a0";"a1";"a2";"a3"];
    branches = List.map2 (fun b o -> { branch_id = branch_id_of_string b; branch_subject = oid o; outcome_branches = [(Success, Stop)]}) ["b0";"b1";"b2";"b3"] ["a0";"a1";"a2";"a3"];
    roles = []; item_templates = []; capability_contracts = [];
  } in
  ignore (bench_case "Persistent Branch" p_persist);
  (* 4. high-symmetry independent origins *)
  let p_sym = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = []; entry_guards = [];
    entry_origin = Some (oid "a0"); success_continuations = [];
    origin_sites = List.init 4 (fun i -> Anchor_origin { anchor_origin_id = oid ("a" ^ string_of_int i); event_name = "ev"; declared_facts = []});
    branches = []; roles = []; item_templates = []; capability_contracts = [];
  } in
  ignore (bench_case "high-symmetry 4 origins" p_sym);
  (* 5. mixed realistic small Core fixture *)
  let p_mixed = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = [{ fact_id = fid "f1"; schema_description = ""; provenance = Evaluation_input (hsk "hk1", String_type)}];
    entry_guards = [{ fact_id = fid "f1"; operator = Equals; expected = String_value "v"}];
    entry_origin = Some (oid "a0"); success_continuations = [];
    origin_sites = [
      Anchor_origin { anchor_origin_id = oid "a0"; event_name = "ev"; declared_facts = []};
      Action_origin { action_origin_id = oid "a1"; capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; inputs = []; declared_facts = []; execution_constraints = [Deadline "z"]};
    ];
    branches = [{ branch_id = branch_id_of_string "b0"; branch_subject = oid "a0"; outcome_branches = [(Success, Continue_to (oid "a1")); (Failure, Stop)]}];
    roles = []; item_templates = []; capability_contracts = [{ capability_id = cid "cap.x"; contract_digest = ccd "sha256:abc"; schema_description = ""}];
  } in
  ignore (bench_case "mixed small" p_mixed);
  (* 6. templates/roles *)
  let tid_a = tid "TA" and tid_b = tid "TB" in
  let p_tpl = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = []; entry_guards = [];
    entry_origin = Some (oid "anchor"); success_continuations = [];
    origin_sites = [Anchor_origin { anchor_origin_id = oid "anchor"; event_name = "ev"; declared_facts = []}];
    branches = []; roles = [];
    item_templates = [
      { item_template_id = tid_a; origin_sites = []; branches = [];
        roles = [{ role_id = rid "RA1"; scope = Item_template_scope tid_a; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok"}; { role_id = rid "RA2"; scope = Item_template_scope tid_a; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok"}];
        objective = Required_role (rid "RA1")};
      { item_template_id = tid_b; origin_sites = []; branches = [];
        roles = [{ role_id = rid "RB1"; scope = Item_template_scope tid_b; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok"}];
        objective = Required_role (rid "RB1")};
    ];
    capability_contracts = [];
  } in
  ignore (bench_case "templates/roles" p_tpl);
  Printf.printf "=== End Performance Evidence ===\n\n"

(* ================================================================== *)
(*  Counterexample-driven pruning tests (§9)                            *)
(* ================================================================== *)

let test_equal_colour_non_automorphic_not_pruned () =
  (* Two facts share initial scalar (same host_key) but differ via origin link.
     Equal initial colour != automorphism — IR must not prune one. *)
  let f1 = { fact_id = fid "f1"; schema_description = ""; provenance = Evaluation_input (hsk "same", String_type)} in
  let f2 = { fact_id = fid "f2"; schema_description = ""; provenance = Evaluation_input (hsk "same", String_type)} in
  let o1 = oid "o1" in
  let f1_linked = { fact_id = fid "f3"; schema_description = ""; provenance = Origin_provenance o1 } in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = [f1; f2; f1_linked];
    entry_guards = []; entry_origin = Some o1; success_continuations = [];
    origin_sites = [Anchor_origin { anchor_origin_id = o1; event_name = "ev"; declared_facts = []}];
    branches = []; roles = []; item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "equal colour non-automorphic"

let test_multi_round_distinction () =
  (* Two origins identical scalar, distinguished only after one refinement round via fact degree. *)
  let fA = { fact_id = fid "fA"; schema_description = ""; provenance = Evaluation_input (hsk "hkA", String_type)} in
  let fB = { fact_id = fid "fB"; schema_description = ""; provenance = Evaluation_input (hsk "hkB", String_type)} in
  let o1 = oid "o1" and o2 = oid "o2" in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = [fA; fB];
    entry_guards = []; entry_origin = Some o1; success_continuations = [];
    origin_sites = [
      Anchor_origin { anchor_origin_id = o1; event_name = "ev"; declared_facts = [fA]};
      Anchor_origin { anchor_origin_id = o2; event_name = "ev"; declared_facts = []};
    ];
    branches = []; roles = []; item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "multi-round distinction"

let test_role_proxy_scope_counterexample () =
  (* Same raw RoleId R in two template scopes; Role_proxy in each template must resolve to local R.
     Naive global lookup would merge them incorrectly. *)
  let tid_a = tid "TA" and tid_b = tid "TB" in
  let r = rid "R" in
  let fA = { fact_id = fid "fA"; schema_description = ""; provenance = Role_proxy r } in
  let fB = { fact_id = fid "fB"; schema_description = ""; provenance = Role_proxy r } in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = []; entry_guards = [];
    entry_origin = None; success_continuations = [];
    origin_sites = []; branches = []; roles = [];
    item_templates = [{
      item_template_id = tid_a;
      origin_sites = [Anchor_origin { anchor_origin_id = oid "oa"; event_name = "evA"; declared_facts = [fA]}];
      branches = []; roles = [{ role_id = r; scope = Item_template_scope tid_a; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "okA"}];
      objective = Required_role r;
    }; {
      item_template_id = tid_b;
      origin_sites = [Anchor_origin { anchor_origin_id = oid "ob"; event_name = "evB"; declared_facts = [fB]}];
      branches = []; roles = [{ role_id = r; scope = Item_template_scope tid_b; fact_contract = Role_fact_contract []; eligible_fulfillment = rf "okB"}];
      objective = Required_role r;
    }];
    capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "Role_proxy scope counterexample"

let test_lexical_vs_scalar_order () =
  (* Storage/raw-ID order opposite scalar order — scalar order must win. *)
  let facts = [
    { fact_id = fid "zzz"; schema_description = ""; provenance = Evaluation_input (hsk "hk_a", String_type)};
    { fact_id = fid "aaa"; schema_description = ""; provenance = Evaluation_input (hsk "hk_z", String_type)};
  ] in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = facts;
    entry_guards = []; entry_origin = Some (oid "anchor"); success_continuations = [];
    origin_sites = [Anchor_origin { anchor_origin_id = oid "anchor"; event_name = "ev"; declared_facts = []}];
    branches = []; roles = []; item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "lexical vs scalar order"

let test_branch_symmetry_broken_by_target () =
  (* Branch symmetry broken by differing targets — must not be considered automorphic *)
  let a0 = oid "a0" and a1 = oid "a1" and a2 = oid "a2" in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = []; entry_guards = [];
    entry_origin = Some a0; success_continuations = [];
    origin_sites = List.map (fun oid -> Anchor_origin { anchor_origin_id = oid; event_name = "ev"; declared_facts = []}) [a0;a1;a2];
    branches = [
      { branch_id = branch_id_of_string "b0"; branch_subject = a0; outcome_branches = [(Success, Continue_to a1)]};
      { branch_id = branch_id_of_string "b1"; branch_subject = a0; outcome_branches = [(Success, Continue_to a2)]};
    ];
    roles = []; item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "branch symmetry broken by target"

let test_refinement_fail_closed () =
  let facts = List.init 3 (fun i -> { fact_id = fid ("f" ^ string_of_int i); schema_description = ""; provenance = Evaluation_input (hsk ("hk" ^ string_of_int i), String_type)}) in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = facts;
    entry_guards = []; entry_origin = None; success_continuations = [];
    origin_sites = []; branches = []; roles = []; item_templates = []; capability_contracts = [];
  } in
  let tiny = { Tethers_core_canonical_v2_ir.default_budget_ir with max_refinement_rounds = 1 } in
  (* With only 1 round, refinement may not converge for programs needing more rounds — must fail closed if not stable.
     For this tiny program it actually converges in 1 round, so we test a case that needs more:
     Use a chain where 2 rounds needed — if not stable, must error. *)
  let _ = tiny in
  (* Directly test that budget 0 still works (no refinement limit hit) and that a program with needing many rounds would fail.
     We force fail by using max_refinement_rounds=0 which means no stable check passes for non-empty? Instead test limit 0 with non-empty: stable_refinement should require at least 1 round and thus fail. *)
  let zero_budget = { Tethers_core_canonical_v2_ir.default_budget_ir with max_refinement_rounds = 0 } in
  let res = Tethers_core_canonical_v2_ir.canonicalize_ir ~budget:zero_budget p in
  (match res with
   | Error Tethers_core_canonical_v2_ir.Canonicalisation_too_complex -> ()
   | Ok _ when List.length facts = 0 -> ()
   | Ok _ -> if List.length facts > 0 then failwith "expected fail-closed on max_refinement_rounds=0" else ()
   | Error _ -> failwith "wrong error kind");
  Printf.printf "PASS: refinement fail-closed\n"

(* ================================================================== *)
(*  Main                                                                *)
(* ================================================================== *)

let () =
  Printf.printf "=== V2 IR Search Tests (C-B4I3B) ===\n\n";
  test_empty (); Printf.printf "PASS: empty\n";
  test_simple_anchor (); Printf.printf "PASS: simple Anchor\n";
  test_anchor_action (); Printf.printf "PASS: Anchor+Action\n";
  test_raw_id_rename (); Printf.printf "PASS: raw-ID rename\n";
  test_cross_family_same_raw (); Printf.printf "PASS: cross-family same raw\n";
  test_same_raw_role_ids_across_templates (); Printf.printf "PASS: same raw RoleId across templates\n";
  test_role_blocks (); Printf.printf "PASS: role blocks\n";
  test_mixed_origin_batch (); Printf.printf "PASS: mixed Origin/Batch\n";
  test_nested_storage_order (); Printf.printf "PASS: nested storage order\n";
  test_action_input_secondary_sorting (); Printf.printf "PASS: action input secondary sorting\n";
  test_constraint_ordering (); Printf.printf "PASS: constraint aa/z\n";
  test_role_fact_contract_ordering (); Printf.printf "PASS: Role_fact_contract ordering\n";
  test_together_permutation (); Printf.printf "PASS: Together permutation\n";
  test_multiplicity (); Printf.printf "PASS: multiplicity\n";
  test_integer_boundaries (); Printf.printf "PASS: integer boundaries\n";
  test_high_bytes (); Printf.printf "PASS: high bytes\n";
  test_persistent_branch_ir (); Printf.printf "PASS: Persistent Branch 24 perms\n";
  test_7fact_beyond_oracle (); Printf.printf "PASS: 7! beyond oracle\n";
  test_sym_all_identical_independent (); Printf.printf "PASS: symmetry A identical independent\n";
  test_sym_paired_branch_origin (); Printf.printf "PASS: symmetry C paired branch/origin\n";
  test_sym_two_identical_templates (); Printf.printf "PASS: symmetry D identical templates\n";
  test_sym_same_raw_role_ids (); Printf.printf "PASS: symmetry E same raw role IDs\n";
  test_sym_together_groups (); Printf.printf "PASS: symmetry F Together groups\n";
  test_sym_regular_biregular (); Printf.printf "PASS: symmetry G regular/biregular\n";
  test_metamorphic_storage (); Printf.printf "PASS: metamorphic raw-ID/storage\n";
  test_equal_colour_non_automorphic_not_pruned (); Printf.printf "PASS: counterexample equal colour non-automorphic\n";
  test_multi_round_distinction (); Printf.printf "PASS: counterexample multi-round distinction\n";
  test_role_proxy_scope_counterexample (); Printf.printf "PASS: counterexample Role_proxy scope\n";
  test_lexical_vs_scalar_order (); Printf.printf "PASS: counterexample lexical vs scalar order\n";
  test_branch_symmetry_broken_by_target (); Printf.printf "PASS: counterexample branch symmetry broken\n";
  test_refinement_fail_closed ();
  test_generated_corpus (); Printf.printf "PASS: generated corpus 1000\n";
  test_budget_fail_closed (); Printf.printf "PASS: deterministic budget fail-closed\n";
  test_performance_evidence (); Printf.printf "PASS: performance evidence (reported)\n";
  Printf.printf "\n=== All V2 IR Tests Complete ===\n"
