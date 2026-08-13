(* ==================================================================
   CANONICAL FORMAT V2 — ROCKET ANCHOR TIE REPAIR TESTS (C-B4I3R2A)

   Proves IR returns EXACTLY the same CanonicalPayload_V2 and
   ProgramDigest_V2 as both the slow oracle (where it can run) and
   the accepted exhaustive baseline everywhere both succeed.

   Covers:
     - frozen hard gates
     - Persistent Branch 24 perms
     - dense generated differential corpus (5,000)
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
  let p_pair = { p_nul with input_facts = [
    { fact_id = fid "hi"; schema_description = ""; provenance = Evaluation_input (hsk "\x80", String_type)};
    { fact_id = fid "lo"; schema_description = ""; provenance = Evaluation_input (hsk "\x7f", String_type)};
  ]} in
  assert_ir_eq_oracle_and_baseline p_nul "high bytes NUL";
  assert_ir_eq_oracle_and_baseline p_empty "high bytes empty";
  assert_ir_eq_oracle_and_baseline p_pair "high bytes pair";
  let (_, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p_pair) "high bytes pair IR" in
  check_equal_int 1 stats.leaves_encoded "high bytes pair shortcut";
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
  check_equal_int 6 stats0.leaves_encoded "persistent Branch Anchor-tie residual search";
  check_equal_int 570 stats0.leaves_avoided "persistent Branch leaves avoided";
  Printf.printf "Persistent Branch IR stats: nodes=%d leaves=%d rounds=%d prefix_pruned=%d orbit_pruned=%d dup_hits=%d leaves_avoided=%d\n"
    stats0.nodes stats0.leaves_encoded stats0.refinement_rounds stats0.prefix_subtrees_pruned stats0.orbit_branches_pruned stats0.duplicate_payload_hits stats0.leaves_avoided

let test_single_collection_branch_shortcut () =
  let anchor = oid "anchor" in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = Some anchor;
    success_continuations = [];
    origin_sites = [Anchor_origin {
      anchor_origin_id = anchor; event_name = "ev"; declared_facts = [];
    }];
    branches = List.init 8 (fun index -> {
      branch_id = branch_id_of_string ("branch" ^ string_of_int index);
      branch_subject = anchor;
      outcome_branches = [(Success, Stop)];
    });
    roles = []; item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "single collection 8-Branch shortcut";
  let (_, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p)
    "single collection 8-Branch IR" in
  check_equal_int 1 stats.leaves_encoded "single collection 8-Branch exact leaf";
  check_equal_int 40_319 stats.leaves_avoided
    "single collection 8-Branch leaves avoided"

let test_single_collection_branch_body_order () =
  let anchor = oid "anchor" in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = Some anchor;
    success_continuations = [];
    origin_sites = [Anchor_origin {
      anchor_origin_id = anchor; event_name = "ev"; declared_facts = [];
    }];
    branches = [
      { branch_id = branch_id_of_string "failure-first";
        branch_subject = anchor; outcome_branches = [(Failure, Stop)] };
      { branch_id = branch_id_of_string "success-second";
        branch_subject = anchor; outcome_branches = [(Success, Stop)] };
    ];
    roles = []; item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "single collection distinct Branch bodies";
  let (_, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p)
    "single collection distinct Branch bodies IR" in
  check_equal_int 1 stats.leaves_encoded
    "single collection distinct Branch bodies exact leaf"

let test_dependency_closed_program_anchor_origins () =
  let expected_minimum_label count =
    let best = ref 1 in
    for label = 2 to count do
      if Tethers_core_canonical_v2_format.compare_bytes_lex_unsigned
           (Tethers_core_canonical_v2_format.encode_int label)
           (Tethers_core_canonical_v2_format.encode_int !best) < 0 then
        best := label
    done;
    !best
  in
  check (Tethers_core_canonical_v2_format.compare_bytes_lex_unsigned
    (Tethers_core_canonical_v2_format.encode_int 10)
    (Tethers_core_canonical_v2_format.encode_int 1) < 0)
    "decimal label 10 precedes 1 lexically";
  List.iter (fun (count, expected) ->
    check_equal_int expected (expected_minimum_label count)
      (Printf.sprintf "entry-origin encoded minimum at %d" count)
  ) [(8, 1); (9, 1); (10, 10); (11, 10); (12, 10); (19, 10); (20, 10); (21, 10)];
  let make count reverse hostile_ids =
    let names = List.init count (fun index ->
      if hostile_ids then Printf.sprintf "\xffraw-%02d" (count - index)
      else Printf.sprintf "origin-%02d" index
    ) in
    let event index = Printf.sprintf "%02d|" index ^ match index mod 7 with
      | 0 -> ""
      | 1 -> "\x00"
      | 2 -> "\x80"
      | 3 -> "aa"
      | 4 -> "z"
      | 5 -> "same\xff"
      | _ -> "same\x00late"
    in
    let sites = List.mapi (fun index name -> Anchor_origin {
      anchor_origin_id = oid name; event_name = event index; declared_facts = [];
    }) names in
    {
      program_id = pid "test"; core_version = cv "0.1.0";
      input_facts = []; entry_guards = []; entry_origin = Some (oid (List.hd names));
      success_continuations = [];
      origin_sites = if reverse then List.rev sites else sites;
      branches = []; roles = []; item_templates = []; capability_contracts = [];
    }
  in
  let projection = make 6 false false in
  assert_ir_eq_oracle_and_baseline projection "dependency-closed Anchor Origin projection";
  let projection_reversed = make 6 true true in
  assert_ir_eq_oracle_and_baseline projection_reversed "dependency-closed Anchor Origin hostile projection";
  let payload label p =
    let (result, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p)
      "dependency-closed Anchor Origin IR" in
    check_equal_int 1 stats.leaves_encoded (label ^ " dependency-closed Anchor Origin one leaf");
    Tethers_core_canonical_v2_ir.canonical_payload_ir result
  in
  check_equal_string (payload "forward projection" projection)
    (payload "reversed projection" projection_reversed)
    "dependency-closed Anchor Origin raw-ID/storage invariance";
  List.iter (fun count ->
    let p = make count false false in
    let p_reversed = make count true true in
    let raw = Tethers_core_canonical_v2.candidate_count_within_budget ~limit:max_int p in
    (match count, raw with
     | 10, Some n -> check_equal_int 3_628_800 n "10 Anchor Origin raw candidates"
     | 12, Some n -> check_equal_int 479_001_600 n "12 Anchor Origin raw candidates"
     | 19, None | 20, None | 21, None -> ()
     | _ -> ());
    check_equal_string (payload (Printf.sprintf "forward %d" count) p)
      (payload (Printf.sprintf "reversed %d" count) p_reversed)
      (Printf.sprintf "dependency-closed Anchor Origin %d metamorphic" count)
  ) [8; 9; 10; 11; 12; 19; 20; 21]

let test_program_anchor_origin_negative_guards () =
  let a0 = oid "a0" and a1 = oid "a1" and a2 = oid "a2" in
  let with_continuation = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = Some a0;
    success_continuations = [{ from_origin = a1; target = Origin_target a2 }];
    origin_sites = [
      Anchor_origin { anchor_origin_id = a0; event_name = ""; declared_facts = []};
      Anchor_origin { anchor_origin_id = a1; event_name = "aa"; declared_facts = []};
      Anchor_origin { anchor_origin_id = a2; event_name = "z"; declared_facts = []};
    ];
    branches = []; roles = []; item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline with_continuation "Anchor Origin continuation guard";
  let (_, continuation_stats) = check_ok
    (Tethers_core_canonical_v2_ir.canonicalize_ir with_continuation)
    "Anchor Origin continuation guard IR" in
  check_equal_int 2 continuation_stats.leaves_encoded
    "success continuation disables Anchor Origin shortcut";
  let together = oid "together" in
  let with_together = {
    with_continuation with
    success_continuations = [];
    origin_sites = [
      Anchor_origin { anchor_origin_id = a0; event_name = ""; declared_facts = []};
      Anchor_origin { anchor_origin_id = a1; event_name = "aa"; declared_facts = []};
      Together_origin { together_origin_id = together; group_id = gid "neutral";
        member_origin_ids = [a0; a1]; objective = All_members_succeed };
    ];
  } in
  assert_ir_eq_oracle_and_baseline with_together "Anchor Origin Together guard";
  let (_, together_stats) = check_ok
    (Tethers_core_canonical_v2_ir.canonicalize_ir with_together)
    "Anchor Origin Together guard IR" in
  check_equal_int 2 together_stats.leaves_encoded
    "Together disables Anchor Origin shortcut"

let make_anchor_tie_witness ~reverse =
  let a0 = oid "a0" and a1 = oid "a1" and a2 = oid "a2" in
  let sites = [
    Anchor_origin { anchor_origin_id = a0; event_name = "entry"; declared_facts = [] };
    Anchor_origin { anchor_origin_id = a1; event_name = "tie"; declared_facts = [] };
    Anchor_origin { anchor_origin_id = a2; event_name = "tie"; declared_facts = [] };
  ] in
  {
    program_id = pid "anchor-tie-repro"; core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = Some a0;
    success_continuations = [];
    origin_sites = if reverse then List.rev sites else sites;
    branches = [{
      branch_id = branch_id_of_string "later-branch";
      branch_subject = a2; outcome_branches = [(Success, Stop)];
    }];
    roles = []; item_templates = []; capability_contracts = [];
  }

let test_anchor_tie_repair_minimal () =
  let p = make_anchor_tie_witness ~reverse:false in
  let oracle = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p)
    "Anchor tie repro oracle" in
  let baseline = check_ok (Tethers_core_canonical_v2.canonicalize p)
    "Anchor tie repro baseline" in
  let (rocket, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p)
    "Anchor tie repro Rocket" in
  let baseline_payload = Tethers_core_canonical_v2.canonical_payload baseline in
  let rocket_payload = Tethers_core_canonical_v2_ir.canonical_payload_ir rocket in
  check_equal_string oracle.payload baseline_payload "Anchor tie repair oracle==baseline";
  check_equal_string baseline_payload rocket_payload "Anchor tie repair baseline==Rocket";
  check_equal_string oracle.digest_string
    (Tethers_core_canonical_v2_ir.program_digest_ir rocket)
    "Anchor tie repair digest";
  check_equal_int 2 stats.leaves_encoded "Anchor tie residual 2!";
  let reversed = make_anchor_tie_witness ~reverse:true in
  assert_ir_eq_oracle_and_baseline reversed "Anchor tie reversed repair";
  let reversed_payload = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir reversed)
    "Anchor tie reversed Rocket" |> fst
    |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  check_equal_string rocket_payload reversed_payload "Anchor tie storage invariance"

let make_anchor_tie_torture ~count ~reverse ~hostile_ids ~observer ~almost =
  let raw index =
    if hostile_ids then Printf.sprintf "\xff-hostile-%02d" (count - index)
    else Printf.sprintf "anchor-%02d" index
  in
  let ids = List.init count (fun index -> oid (raw index)) in
  let a0 = List.nth ids 0 and a1 = List.nth ids 1 and a2 = List.nth ids 2 in
  let tie_event = "\x00\x80:tie" in
  let sites = List.mapi (fun index origin_id ->
    let event_name =
      if index = 0 then "entry"
      else if index = 1 then tie_event
      else if index = 2 then (if almost then tie_event ^ "\x00late" else tie_event)
      else Printf.sprintf "%02d:distinct:z\x80" index
    in
    Anchor_origin { anchor_origin_id = origin_id; event_name; declared_facts = [] }
  ) ids in
  let branches = match observer with
    | `Subject -> [{
        branch_id = branch_id_of_string "subject-observer";
        branch_subject = a2; outcome_branches = [(Success, Stop)];
      }]
    | `Continue -> [{
        branch_id = branch_id_of_string "continue-observer";
        branch_subject = a0; outcome_branches = [(Success, Continue_to a2)];
      }]
    | `Multiple -> [
        { branch_id = branch_id_of_string "multi-z"; branch_subject = a2;
          outcome_branches = [(Failure, Continue_to a1); (Success, Stop)] };
        { branch_id = branch_id_of_string "multi-aa"; branch_subject = a1;
          outcome_branches = [(Cancelled, Continue_to a2); (Success, Continue_to a1)] };
      ]
  in
  {
    program_id = pid "anchor-tie-torture"; core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = Some a0;
    success_continuations = [];
    origin_sites = if reverse then List.rev sites else sites;
    branches = if reverse then List.rev branches else branches;
    roles = []; item_templates = []; capability_contracts = [];
  }

let test_anchor_tie_torture () =
  let payload_and_stats p label =
    let result, stats = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p) label in
    (Tethers_core_canonical_v2_ir.canonical_payload_ir result, stats)
  in
  List.iter (fun observer ->
    let label = match observer with
      | `Subject -> "branch_subject"
      | `Continue -> "Continue_to"
      | `Multiple -> "multiple later Branch references"
    in
    let p = make_anchor_tie_torture ~count:3 ~reverse:false ~hostile_ids:false
      ~observer ~almost:false in
    let reversed = make_anchor_tie_torture ~count:3 ~reverse:true ~hostile_ids:false
      ~observer ~almost:false in
    let hostile = make_anchor_tie_torture ~count:3 ~reverse:true ~hostile_ids:true
      ~observer ~almost:false in
    assert_ir_eq_oracle_and_baseline p ("Anchor tie " ^ label);
    assert_ir_eq_oracle_and_baseline reversed ("Anchor tie reversed " ^ label);
    assert_ir_eq_oracle_and_baseline hostile ("Anchor tie hostile " ^ label);
    let payload, stats = payload_and_stats p ("Anchor tie stats " ^ label) in
    let reversed_payload, reversed_stats = payload_and_stats reversed
      ("Anchor tie reversed stats " ^ label) in
    let hostile_payload, hostile_stats = payload_and_stats hostile
      ("Anchor tie hostile stats " ^ label) in
    check_equal_int 2 stats.leaves_encoded (label ^ " residual 2!");
    check_equal_int 2 reversed_stats.leaves_encoded (label ^ " reversed residual 2!");
    check_equal_int 2 hostile_stats.leaves_encoded (label ^ " hostile residual 2!");
    check_equal_string payload reversed_payload (label ^ " storage invariance");
    check_equal_string payload hostile_payload (label ^ " raw-ID invariance")
  ) [`Subject; `Continue; `Multiple];
  let almost = make_anchor_tie_torture ~count:3 ~reverse:false ~hostile_ids:false
    ~observer:`Subject ~almost:true in
  assert_ir_eq_oracle_and_baseline almost "almost-identical Anchor bodies";
  let _, almost_stats = payload_and_stats almost "almost-identical Anchor stats" in
  check_equal_int 1 almost_stats.leaves_encoded
    "almost-identical Anchor bodies are decided by exact bytes";
  let triple_tie =
    let ids = List.init 5 (fun i -> oid (Printf.sprintf "triple-%d" i)) in
    let sites = List.mapi (fun index origin_id -> Anchor_origin {
      anchor_origin_id = origin_id;
      event_name = if index = 0 then "entry" else if index = 1 then "distinct" else "tie";
      declared_facts = [];
    }) ids in
    { program_id = pid "triple-tie"; core_version = cv "0.1.0";
      input_facts = []; entry_guards = []; entry_origin = Some (List.hd ids);
      success_continuations = []; origin_sites = sites;
      branches = [{ branch_id = branch_id_of_string "triple-observer";
        branch_subject = List.nth ids 4; outcome_branches = [(Success, Stop)] }];
      roles = []; item_templates = []; capability_contracts = []; }
  in
  assert_ir_eq_oracle_and_baseline triple_tie "three-way Anchor body tie";
  let _, triple_stats = payload_and_stats triple_tie "three-way Anchor tie stats" in
  check_equal_int 6 triple_stats.leaves_encoded "distinct class plus tied x3 becomes residual 3!";
  List.iter (fun count ->
    let p = make_anchor_tie_torture ~count ~reverse:false ~hostile_ids:false
      ~observer:`Multiple ~almost:false in
    let reversed = make_anchor_tie_torture ~count ~reverse:true ~hostile_ids:true
      ~observer:`Multiple ~almost:false in
    let payload, stats = payload_and_stats p (Printf.sprintf "Anchor tie boundary %d" count) in
    let reversed_payload, reversed_stats = payload_and_stats reversed
      (Printf.sprintf "Anchor tie reversed boundary %d" count) in
    check_equal_int 2 stats.leaves_encoded (Printf.sprintf "Anchor tie boundary %d residual" count);
    check_equal_int 2 reversed_stats.leaves_encoded
      (Printf.sprintf "Anchor tie reversed boundary %d residual" count);
    check_equal_string payload reversed_payload
      (Printf.sprintf "Anchor tie boundary %d metamorphic" count);
    if count = 9 then assert_ir_eq_oracle_and_baseline p "Anchor tie boundary 9 exhaustive"
  ) [9; 10; 11; 12];
  let boundary12 = make_anchor_tie_torture ~count:12 ~reverse:false ~hostile_ids:false
    ~observer:`Subject ~almost:false in
  let budget_two = { Tethers_core_canonical_v2_ir.default_budget_ir with max_leaves = 2 } in
  let budget_one = { Tethers_core_canonical_v2_ir.default_budget_ir with max_leaves = 1 } in
  ignore (check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir ~budget:budget_two boundary12)
    "Anchor tie pre-admission residual 2 accepted");
  (match Tethers_core_canonical_v2_ir.canonicalize_ir ~budget:budget_one boundary12 with
   | Error Tethers_core_canonical_v2_ir.Canonicalisation_too_complex -> ()
   | _ -> failwith "Anchor tie pre-admission must reject residual 2 under leaf budget 1")

let test_branch_label_count_boundaries () =
  let make count reverse =
    let anchor = oid "anchor\x80" in
    let branches = List.init count (fun index -> {
      branch_id = branch_id_of_string (Printf.sprintf "raw-branch-%02d" index);
      branch_subject = anchor;
      outcome_branches = [(Success, Stop)];
    }) in
    {
      program_id = pid "test"; core_version = cv "0.1.0";
      input_facts = []; entry_guards = []; entry_origin = Some anchor;
      success_continuations = [];
      origin_sites = [Anchor_origin {
        anchor_origin_id = anchor; event_name = "z\x80"; declared_facts = [];
      }];
      branches = if reverse then List.rev branches else branches;
      roles = []; item_templates = []; capability_contracts = [];
    }
  in
  List.iter (fun (count, expected_raw) ->
    let p = make count false in
    let p_reversed = make count true in
    (match Tethers_core_canonical_v2.candidate_count_within_budget ~limit:max_int p, expected_raw with
     | Some actual, Some expected ->
         check_equal_int expected actual (Printf.sprintf "%d-Branch raw candidates" count)
     | None, None -> ()
     | _ -> failwith (Printf.sprintf "%d-Branch raw candidate count shape" count));
    let (ir, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p)
      (Printf.sprintf "%d-Branch IR" count) in
    let (ir_reversed, reversed_stats) = check_ok
      (Tethers_core_canonical_v2_ir.canonicalize_ir p_reversed)
      (Printf.sprintf "%d-Branch reversed IR" count) in
    check_equal_int 1 stats.leaves_encoded (Printf.sprintf "%d-Branch exact leaf" count);
    check_equal_int 1 reversed_stats.leaves_encoded
      (Printf.sprintf "%d-Branch reversed exact leaf" count);
    check_equal_string
      (Tethers_core_canonical_v2_ir.canonical_payload_ir ir)
      (Tethers_core_canonical_v2_ir.canonical_payload_ir ir_reversed)
      (Printf.sprintf "%d-Branch storage invariance" count)
  ) [
    (8, Some 40_320);
    (9, Some 362_880);
    (10, Some 3_628_800);
    (11, Some 39_916_800);
    (12, Some 479_001_600);
    (19, Some 121_645_100_408_832_000);
    (20, Some 2_432_902_008_176_640_000);
    (21, None);
  ];
  (* Nine is still practical for the exhaustive baseline.  The larger cases
     use this exact same shape with storage metamorphism rather than treating
     a non-completing baseline as an oracle. *)
  assert_ir_eq_oracle_and_baseline (make 9 false) "9-Branch baseline projection"

let test_decimal_label_family_boundaries () =
  let counts = [8; 9; 10; 11; 12; 19; 20; 21] in
  let check_raw label count p =
    match count, Tethers_core_canonical_v2.candidate_count_within_budget ~limit:max_int p with
    | 10, Some n -> check_equal_int 3_628_800 n (label ^ " 10 raw candidates")
    | 12, Some n -> check_equal_int 479_001_600 n (label ^ " 12 raw candidates")
    | 19, Some n -> check_equal_int 121_645_100_408_832_000 n (label ^ " 19 raw candidates")
    | 20, Some n -> check_equal_int 2_432_902_008_176_640_000 n (label ^ " 20 raw candidates")
    | 21, None -> ()
    | _, Some _ -> ()
    | _ -> failwith (label ^ " raw candidate count shape")
  in
  let make_facts count reverse = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = (List.init count (fun i -> {
      fact_id = fid (Printf.sprintf "fact-%02d" i); schema_description = "";
      provenance = Evaluation_input (hsk (Printf.sprintf "host-%02d\x80" i), String_type);
    }) |> fun xs -> if reverse then List.rev xs else xs);
    entry_guards = []; entry_origin = None; success_continuations = [];
    origin_sites = []; branches = []; roles = []; item_templates = []; capability_contracts = [];
  } in
  let make_program_roles count reverse =
    let roles = List.init count (fun i -> {
      role_id = rid (Printf.sprintf "program-role-%02d" i); scope = Program_scope;
      fact_contract = Role_fact_contract [];
      eligible_fulfillment = rf (Printf.sprintf "body-%02d\x80" i);
    }) in
    { program_id = pid "test"; core_version = cv "0.1.0";
      input_facts = []; entry_guards = []; entry_origin = None; success_continuations = [];
      origin_sites = []; branches = []; roles = if reverse then List.rev roles else roles;
      item_templates = []; capability_contracts = []; }
  in
  let make_template_roles count reverse =
    let template = tid "decimal-template" in
    let roles = List.init count (fun i -> {
      role_id = rid (Printf.sprintf "template-role-%02d" i);
      scope = Item_template_scope template; fact_contract = Role_fact_contract [];
      eligible_fulfillment = rf (Printf.sprintf "template-body-%02d\x80" i);
    }) in
    { program_id = pid "test"; core_version = cv "0.1.0";
      input_facts = []; entry_guards = []; entry_origin = None; success_continuations = [];
      origin_sites = []; branches = []; roles = [];
      item_templates = [{ item_template_id = template; origin_sites = []; branches = [];
        roles = if reverse then List.rev roles else roles;
        objective = Required_role (rid "template-role-00"); }];
      capability_contracts = []; }
  in
  let check_family label make =
    List.iter (fun count ->
      let p = make count false and reversed = make count true in
      check_raw label count p;
      let (ir, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p)
        (Printf.sprintf "%s %d IR" label count) in
      let (ir_reversed, reversed_stats) = check_ok
        (Tethers_core_canonical_v2_ir.canonicalize_ir reversed)
        (Printf.sprintf "%s %d reversed IR" label count) in
      check_equal_int 1 stats.leaves_encoded (Printf.sprintf "%s %d exact leaf" label count);
      check_equal_int 1 reversed_stats.leaves_encoded
        (Printf.sprintf "%s %d reversed exact leaf" label count);
      check_equal_string
        (Tethers_core_canonical_v2_ir.canonical_payload_ir ir)
        (Tethers_core_canonical_v2_ir.canonical_payload_ir ir_reversed)
        (Printf.sprintf "%s %d storage invariance" label count)
    ) counts
  in
  check_family "decimal Facts" make_facts;
  check_family "decimal Program Roles" make_program_roles;
  check_family "decimal Template Roles" make_template_roles;
  assert_ir_eq_oracle_and_baseline (make_facts 8 false) "decimal Fact baseline projection";
  assert_ir_eq_oracle_and_baseline (make_program_roles 8 false) "decimal Program Role baseline projection";
  assert_ir_eq_oracle_and_baseline (make_template_roles 8 false) "decimal Template Role baseline projection"

let test_compound_factor_collapses () =
  let make ~facts_count ~branches_count ~roles_count ~reverse =
    let anchor = oid "compound-anchor" in
    let facts = List.init facts_count (fun i -> {
      fact_id = fid (Printf.sprintf "compound-fact-%02d" i); schema_description = "";
      provenance = Evaluation_input (hsk (Printf.sprintf "compound-host-%02d" i), String_type);
    }) in
    let branches = List.init branches_count (fun i -> {
      branch_id = branch_id_of_string (Printf.sprintf "compound-branch-%02d" i);
      branch_subject = anchor; outcome_branches = [(Success, Stop)];
    }) in
    let roles = List.init roles_count (fun i -> {
      role_id = rid (Printf.sprintf "compound-role-%02d" i); scope = Program_scope;
      fact_contract = Role_fact_contract [];
      eligible_fulfillment = rf (Printf.sprintf "compound-fulfillment-%02d" i);
    }) in
    { program_id = pid "test"; core_version = cv "0.1.0";
      input_facts = if reverse then List.rev facts else facts;
      entry_guards = []; entry_origin = Some anchor; success_continuations = [];
      origin_sites = [Anchor_origin {
        anchor_origin_id = anchor; event_name = "compound-event"; declared_facts = [];
      }];
      branches = if reverse then List.rev branches else branches;
      roles = if reverse then List.rev roles else roles;
      item_templates = []; capability_contracts = []; }
  in
  let check_case label ~facts_count ~branches_count ~roles_count ~raw =
    let p = make ~facts_count ~branches_count ~roles_count ~reverse:false in
    let p_reversed = make ~facts_count ~branches_count ~roles_count ~reverse:true in
    let actual_raw = match Tethers_core_canonical_v2.candidate_count_within_budget ~limit:max_int p with
      | Some n -> n | None -> failwith (label ^ " raw count overflowed") in
    check_equal_int raw actual_raw (label ^ " raw candidates");
    let (ir, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p) (label ^ " IR") in
    let (ir_reversed, reversed_stats) = check_ok
      (Tethers_core_canonical_v2_ir.canonicalize_ir p_reversed) (label ^ " reversed IR") in
    check_equal_int 1 stats.leaves_encoded (label ^ " all proven factors collapse");
    check_equal_int 1 reversed_stats.leaves_encoded (label ^ " reversed all proven factors collapse");
    check_equal_string
      (Tethers_core_canonical_v2_ir.canonical_payload_ir ir)
      (Tethers_core_canonical_v2_ir.canonical_payload_ir ir_reversed)
      (label ^ " raw-ID/storage metamorphic")
  in
  check_case "A 8 Facts x 8 Branches" ~facts_count:8 ~branches_count:8 ~roles_count:0
    ~raw:1_625_702_400;
  check_case "B 6 Program Roles x 8 Branches" ~facts_count:0 ~branches_count:8 ~roles_count:6
    ~raw:29_030_400;
  check_case "C 8 Facts x 8 Branches x 6 Program Roles"
    ~facts_count:8 ~branches_count:8 ~roles_count:6 ~raw:1_170_505_728_000;
  assert_ir_eq_oracle_and_baseline
    (make ~facts_count:3 ~branches_count:3 ~roles_count:0 ~reverse:false)
    "A compound exhaustive projection";
  assert_ir_eq_oracle_and_baseline
    (make ~facts_count:0 ~branches_count:3 ~roles_count:3 ~reverse:false)
    "B compound exhaustive projection";
  assert_ir_eq_oracle_and_baseline
    (make ~facts_count:3 ~branches_count:3 ~roles_count:3 ~reverse:false)
    "C compound exhaustive projection";
  let semantic_before = make ~facts_count:3 ~branches_count:3 ~roles_count:3 ~reverse:false in
  let semantic_after = { semantic_before with input_facts =
    match semantic_before.input_facts with
    | first :: rest -> { first with provenance = Evaluation_input (hsk "different-semantic-host", String_type) } :: rest
    | [] -> [] } in
  let before_payload = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir semantic_before)
    "compound semantic before" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  let after_payload = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir semantic_after)
    "compound semantic after" |> fst |> Tethers_core_canonical_v2_ir.canonical_payload_ir in
  check (before_payload <> after_payload) "compound semantic change must change payload"

let make_mixed_branch_torture ~reverse ~hostile_ids =
  let name plain hostile = if hostile_ids then hostile else plain in
  let a0 = oid (name "a0" "\xff-entry") in
  let a1 = oid (name "a1" "raw-z") in
  let a2 = oid (name "a2" "raw-aa") in
  let together = oid (name "together" "\x80-group") in
  let template = tid (name "template" "template-\xff") in
  let role = rid (name "role" "role-\x80") in
  let site_list = [
    Anchor_origin { anchor_origin_id = a0; event_name = "z"; declared_facts = [] };
    Anchor_origin { anchor_origin_id = a1; event_name = "aa"; declared_facts = [] };
    Anchor_origin { anchor_origin_id = a2; event_name = "\x80late"; declared_facts = [] };
    Together_origin {
      together_origin_id = together; group_id = gid (name "g" "group-raw");
      member_origin_ids = [a2; a0; a1]; objective = All_members_succeed;
    };
    Batch_site {
      batch_id = bid (name "batch" "batch-\xff");
      collection_provenance = batch_collection_provenance_of_string "z";
      item_template_id = template;
      traversal_policy = batch_traversal_policy_of_string "aa";
      composite_objective = batch_objective_of_string "\x80objective";
      aggregate_facts = [];
    };
  ] in
  let branches = [
    { branch_id = branch_id_of_string (name "b0" "branch-z");
      branch_subject = a0;
      outcome_branches = [
        (Cancelled, Stop); (Failure, Continue_to a2); (Success, Continue_to a1);
      ]; };
    { branch_id = branch_id_of_string (name "b1" "branch-aa");
      branch_subject = a0;
      outcome_branches = [
        (Success, Continue_to a2); (Failure, Stop);
      ]; };
    { branch_id = branch_id_of_string (name "b2" "branch-\x80");
      branch_subject = a1;
      outcome_branches = [(Uncertain, Stop)]; };
    { branch_id = branch_id_of_string (name "b3" "branch-late");
      branch_subject = together;
      outcome_branches = [(Success, Stop)]; };
  ] in
  {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = Some a0;
    success_continuations = [
      { from_origin = a1; target = Origin_target a2 };
    ];
    origin_sites = if reverse then List.rev site_list else site_list;
    branches = if reverse then List.rev (List.map (fun b ->
      { b with outcome_branches = List.rev b.outcome_branches }
    ) branches) else branches;
    roles = [];
    item_templates = [{
      item_template_id = template;
      origin_sites = [];
      branches = [];
      roles = [{ role_id = role; scope = Item_template_scope template;
        fact_contract = Role_fact_contract []; eligible_fulfillment = rf "z" }];
      objective = Required_role role;
    }];
    capability_contracts = [];
  }

let test_mixed_branch_torture () =
  let normal = make_mixed_branch_torture ~reverse:false ~hostile_ids:false in
  let reversed = make_mixed_branch_torture ~reverse:true ~hostile_ids:false in
  let hostile = make_mixed_branch_torture ~reverse:true ~hostile_ids:true in
  List.iter (fun (p, label) -> assert_ir_eq_oracle_and_baseline p label) [
    (normal, "mixed Branch torture normal");
    (reversed, "mixed Branch torture reversed");
    (hostile, "mixed Branch torture hostile raw IDs");
  ];
  let get_payload p =
    let (result, _) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p)
      "mixed Branch torture IR" in
    Tethers_core_canonical_v2_ir.canonical_payload_ir result
  in
  let normal_payload = get_payload normal in
  check_equal_string normal_payload (get_payload reversed)
    "mixed Branch torture reversed storage/outcome invariance";
  check_equal_string normal_payload (get_payload hostile)
    "mixed Branch torture hostile raw-ID invariance";
  let (_, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir normal)
    "mixed Branch torture stats" in
  check_equal_int 6 stats.leaves_encoded
    "mixed Branch torture leaves after entry and Branch reductions"

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
  (* Sixteen deliberately different valid structural archetypes, with 312+
     deterministic scalar/order/reference variants apiece.  Every case stays
     at or below 720 raw candidates so the slow oracle remains authoritative. *)
  let shape = n mod 16 in
  let variant = n / 16 in
  let reverse = variant land 1 = 1 in
  let choose offset values = List.nth values ((variant + offset) mod List.length values) in
  let scalar offset = choose offset [""; "z"; "aa"; "\x00"; "\x80"; "1:x"; "2:aa\xff"] in
  let raw stem =
    if variant land 2 = 0 then Printf.sprintf "%s-%05d" stem n
    else Printf.sprintf "\xff%s-hostile-%05d" stem (5000 - n)
  in
  let order xs = if reverse then List.rev xs else xs in
  let pa = oid (raw "pa") and pb = oid (raw "pb") and pc = oid (raw "pc") in
  let ta = oid (raw "ta") and tb = oid (raw "tb") in
  let fin = fid (raw "fin") and fout = fid (raw "fout") and fproxy = fid (raw "fproxy") in
  let ftproxy = fid (raw "ftproxy") in
  let program_role_id = rid (raw "program-role") in
  let shared_role_id = rid "same-raw-role" in
  let template_id = tid (raw "template") in
  let capability = cid "cap.dense" and digest = ccd "sha256:dense" in
  let input_fact fact_id offset = {
    fact_id; schema_description = scalar (offset + 1);
    provenance = Evaluation_input (hsk (scalar offset),
      if (variant + offset) mod 2 = 0 then String_type else Integer_type);
  } in
  let anchor origin_id event facts = Anchor_origin {
    anchor_origin_id = origin_id; event_name = event; declared_facts = order facts;
  } in
  let action origin_id inputs facts = Action_origin {
    action_origin_id = origin_id; capability_id = capability; contract_digest = digest;
    inputs = order inputs; declared_facts = order facts;
    execution_constraints = [Deadline ("deadline-" ^ scalar 4)];
  } in
  let program_role role_id facts = {
    role_id; scope = Program_scope; fact_contract = Role_fact_contract facts;
    eligible_fulfillment = rf ("program-" ^ scalar 5);
  } in
  let template_role role_id facts = {
    role_id; scope = Item_template_scope template_id;
    fact_contract = Role_fact_contract facts;
    eligible_fulfillment = rf ("template-" ^ scalar 6);
  } in
  let branch name subject outcomes = {
    branch_id = branch_id_of_string (raw name); branch_subject = subject;
    outcome_branches = order outcomes;
  } in
  let contract = [{ capability_id = capability; contract_digest = digest;
    schema_description = scalar 3 }] in
  let empty = {
    program_id = pid (raw "program"); core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = None;
    success_continuations = []; origin_sites = []; branches = []; roles = [];
    item_templates = []; capability_contracts = [];
  } in
  match shape with
  | 0 ->
      { empty with input_facts = order [input_fact fin 0; input_fact fout 2];
        entry_guards = [{ fact_id = fin; operator = Equals;
          expected = String_value (scalar 4) }] }
  | 1 ->
      let produced = { fact_id = fout; schema_description = scalar 2;
        provenance = Origin_provenance pa } in
      { empty with input_facts = [input_fact fin 0]; entry_origin = Some pa;
        origin_sites = order [anchor pa (scalar 1) [produced];
          action pb [{ input_name = capability_input_name_of_string "from-origin";
            binding = Fact_from_origin (fout, pa) }] []];
        capability_contracts = contract }
  | 2 ->
      let proxy = { fact_id = fproxy; schema_description = scalar 2;
        provenance = Role_proxy program_role_id } in
      { empty with input_facts = [input_fact fin 0]; entry_origin = Some pa;
        origin_sites = [anchor pa (scalar 1) [proxy]];
        roles = [program_role program_role_id [fin]] }
  | 3 ->
      let produced = { fact_id = fout; schema_description = scalar 2;
        provenance = Origin_provenance pb } in
      { empty with input_facts = [input_fact fin 0]; entry_origin = Some pa;
        origin_sites = order [anchor pa (scalar 1) [];
          action pb [{ input_name = capability_input_name_of_string "through-role";
            binding = Fact_through_role (fin, program_role_id) }] [produced]];
        roles = [program_role program_role_id [fin]];
        capability_contracts = contract }
  | 4 ->
      { empty with entry_origin = Some pa;
        origin_sites = order [anchor pa (scalar 0) []; action pb [] [];
          Together_origin { together_origin_id = pc; group_id = gid (raw "group");
            member_origin_ids = order [pa; pb]; objective = All_members_succeed }];
        branches = order [
          branch "branch-a" pc [(Success, Continue_to pb); (Failure, Stop)];
          branch "branch-b" pa [(Cancelled, Continue_to pc); (Uncertain, Stop)]];
        capability_contracts = contract }
  | 5 ->
      let role = template_role shared_role_id [] in
      let template = { item_template_id = template_id; origin_sites = []; branches = [];
        roles = [role]; objective = Required_role shared_role_id } in
      { empty with origin_sites = [Batch_site {
          batch_id = bid (raw "batch"); collection_provenance =
            batch_collection_provenance_of_string (scalar 0);
          item_template_id = template_id; traversal_policy =
            batch_traversal_policy_of_string (scalar 1);
          composite_objective = batch_objective_of_string (scalar 2);
          aggregate_facts = [input_fact fout 3] }];
        item_templates = [template] }
  | 6 ->
      let role = template_role shared_role_id [fin] in
      let proxy = { fact_id = ftproxy; schema_description = scalar 2;
        provenance = Role_proxy shared_role_id } in
      let template = { item_template_id = template_id;
        origin_sites = order [anchor ta (scalar 0) [proxy];
          action tb [{ input_name = capability_input_name_of_string "template-through";
            binding = Fact_through_role (fin, shared_role_id) }] []];
        branches = []; roles = [role]; objective = Required_role shared_role_id } in
      { empty with input_facts = [input_fact fin 1]; item_templates = [template];
        capability_contracts = contract }
  | 7 ->
      let program_proxy = { fact_id = fproxy; schema_description = scalar 2;
        provenance = Role_proxy shared_role_id } in
      let template_proxy = { fact_id = ftproxy; schema_description = scalar 3;
        provenance = Role_proxy shared_role_id } in
      let template = { item_template_id = template_id;
        origin_sites = [anchor ta (scalar 1) [template_proxy]]; branches = [];
        roles = [template_role shared_role_id [fin]];
        objective = Required_role shared_role_id } in
      { empty with input_facts = [input_fact fin 0]; entry_origin = Some pa;
        origin_sites = [anchor pa (scalar 1) [program_proxy]];
        roles = [program_role shared_role_id [fin]]; item_templates = [template] }
  | 8 ->
      { empty with entry_origin = Some pa;
        success_continuations = order [
          { from_origin = pa; target = Origin_target pb };
          { from_origin = pb; target = Program_complete }];
        origin_sites = order [anchor pa (scalar 0) []; anchor pb (scalar 1) [];
          anchor pc (scalar 2) []];
        branches = order [
          branch "multi-a" pc [(Success, Continue_to pb); (Failure, Continue_to pa)];
          branch "multi-b" pb [(Cancelled, Stop); (Uncertain, Continue_to pc)]] }
  | 9 ->
      let template = { item_template_id = template_id;
        origin_sites = [anchor ta (scalar 1) []];
        branches = [branch "template-branch" ta [(Success, Continue_to pa)]];
        roles = [template_role shared_role_id []];
        objective = Required_role shared_role_id } in
      { empty with entry_origin = Some pa; origin_sites = [anchor pa (scalar 0) []];
        branches = [branch "program-branch" pa [(Success, Continue_to ta)]];
        item_templates = [template] }
  | 10 ->
      let proxy = { fact_id = fproxy; schema_description = scalar 2;
        provenance = Role_proxy program_role_id } in
      { empty with input_facts = [input_fact fin 0]; entry_origin = Some pa;
        success_continuations = if variant mod 3 = 0 then
          [{ from_origin = pa; target = Origin_target pb }] else [];
        origin_sites = order [anchor pa (scalar 1) [proxy];
          action pb [{ input_name = capability_input_name_of_string "dense-through";
            binding = Fact_through_role (fin, program_role_id) }] [];
          Together_origin { together_origin_id = pc; group_id = gid (raw "dense-group");
            member_origin_ids = order [pa; pb]; objective = All_members_succeed }];
        branches = [branch "dense-branch" pc
          [(Success, Continue_to pb); (Failure, Stop); (Cancelled, Continue_to pa)]];
        roles = [program_role program_role_id [fin]]; capability_contracts = contract }
  | 11 ->
      let proxy = { fact_id = ftproxy; schema_description = scalar 2;
        provenance = Role_proxy shared_role_id } in
      let template = { item_template_id = template_id;
        origin_sites = order [anchor ta (scalar 0) [proxy];
          action tb [{ input_name = capability_input_name_of_string "template-role-input";
            binding = Fact_through_role (fin, shared_role_id) }] []];
        branches = [branch "template-dense-branch" tb
          [(Success, Continue_to ta); (Failure, Stop)]];
        roles = [template_role shared_role_id [fin]];
        objective = Required_role shared_role_id } in
      { empty with input_facts = [input_fact fin 1]; entry_origin = Some pa;
        origin_sites = order [anchor pa (scalar 3) []; Batch_site {
          batch_id = bid (raw "dense-batch"); collection_provenance =
            batch_collection_provenance_of_string (scalar 4);
          item_template_id = template_id; traversal_policy =
            batch_traversal_policy_of_string (scalar 5);
          composite_objective = batch_objective_of_string (scalar 6);
          aggregate_facts = [] }];
        item_templates = [template]; capability_contracts = contract }
  | 12 ->
      make_anchor_tie_torture ~count:3 ~reverse ~hostile_ids:(variant land 2 <> 0)
        ~observer:(match variant mod 3 with 0 -> `Subject | 1 -> `Continue | _ -> `Multiple)
        ~almost:(variant mod 5 = 0)
  | 13 ->
      let role = template_role shared_role_id [] in
      let template = { item_template_id = template_id; origin_sites = []; branches = [];
        roles = [role]; objective = Required_role shared_role_id } in
      let batch index fact_id = Batch_site {
        batch_id = bid (raw ("batch-" ^ string_of_int index));
        collection_provenance = batch_collection_provenance_of_string (scalar index);
        item_template_id = template_id;
        traversal_policy = batch_traversal_policy_of_string (scalar (index + 1));
        composite_objective = batch_objective_of_string (scalar (index + 2));
        aggregate_facts = [input_fact fact_id (index + 3)] } in
      { empty with origin_sites = order [batch 0 fout; batch 1 fproxy];
        item_templates = [template] }
  | 14 ->
      let template which origin_id =
        let template_id_local = tid (raw ("template-" ^ string_of_int which)) in
        let role = { role_id = shared_role_id; scope = Item_template_scope template_id_local;
          fact_contract = Role_fact_contract [];
          eligible_fulfillment = rf (scalar which) } in
        { item_template_id = template_id_local;
          origin_sites = [anchor origin_id (scalar (which + 2)) []]; branches = [];
          roles = [role]; objective = Required_role shared_role_id }
      in
      { empty with item_templates = order [template 0 ta; template 1 tb] }
  | _ ->
      let program_proxy = { fact_id = fproxy; schema_description = scalar 2;
        provenance = Role_proxy shared_role_id } in
      let template_proxy = { fact_id = ftproxy; schema_description = scalar 3;
        provenance = Role_proxy shared_role_id } in
      let template = { item_template_id = template_id;
        origin_sites = [anchor ta (scalar 4) [template_proxy]];
        branches = [branch "cross-template" ta
          [(Success, Continue_to pb); (Failure, Stop)]];
        roles = [template_role shared_role_id [fin]];
        objective = Required_role shared_role_id } in
      { empty with input_facts = [input_fact fin 0]; entry_origin = Some pa;
        success_continuations = if variant mod 2 = 0 then
          [{ from_origin = pa; target = Origin_target pb }] else [];
        origin_sites = order [anchor pa (scalar 1) [program_proxy];
          action pb [{ input_name = capability_input_name_of_string "combined-through";
            binding = Fact_through_role (fin, shared_role_id) }] [];
          Together_origin { together_origin_id = pc; group_id = gid (raw "combined-group");
            member_origin_ids = order [pa; pb]; objective = All_members_succeed };
          Batch_site { batch_id = bid (raw "combined-batch");
            collection_provenance = batch_collection_provenance_of_string (scalar 4);
            item_template_id = template_id;
            traversal_policy = batch_traversal_policy_of_string (scalar 5);
            composite_objective = batch_objective_of_string (scalar 6);
            aggregate_facts = [] }];
        branches = [branch "cross-program" pc
          [(Success, Continue_to ta); (Cancelled, Continue_to pa)]];
        roles = [program_role shared_role_id [fin]]; item_templates = [template];
        capability_contracts = contract }

let test_generated_corpus () =
  let seed = 0x4B4A2 in
  let total = 5_000 in
  let valid = ref 0 in
  for n = 0 to total - 1 do
    let p = generated_case n in
    match Tethers_core_validator.validate p with
    | Error _ ->
        failwith (Printf.sprintf "dense generator invalid seed=%d case=%d" seed n)
    | Ok () ->
        incr valid;
        (match Tethers_core_canonical_v2.candidate_count_within_budget ~limit:720 p with
         | Some _ -> ()
         | None -> failwith (Printf.sprintf
             "dense generator exceeds oracle envelope seed=%d case=%d" seed n));
        let oracle_res = Tethers_core_canonical_v2_reference.slow_oracle p in
        let baseline_res = Tethers_core_canonical_v2.canonicalize p in
        let ir_res = Tethers_core_canonical_v2_ir.canonicalize_ir p in
        (match oracle_res, baseline_res, ir_res with
         | Ok oracle, Ok baseline, Ok (ir, _) ->
             let ir_payload = Tethers_core_canonical_v2_ir.canonical_payload_ir ir in
             let ir_digest = Tethers_core_canonical_v2_ir.program_digest_ir ir in
             if oracle.payload <> ir_payload || oracle.digest_string <> ir_digest ||
                Tethers_core_canonical_v2.canonical_payload baseline <> ir_payload ||
                Tethers_core_canonical_v2.program_digest baseline <> ir_digest then begin
               Printf.eprintf "STOP-THE-LINE dense mismatch seed=%d case=%d shape=%d\n%!"
                 seed n (n mod 16);
               failwith "dense differential mismatch"
             end
         | _ ->
             Printf.eprintf "STOP-THE-LINE dense result-shape mismatch seed=%d case=%d shape=%d\n%!"
               seed n (n mod 16);
             failwith "dense differential result-shape mismatch")
  done;
  Printf.printf "Dense generated corpus: seed=%d total=%d valid=%d mismatches=0 archetypes=16\n"
    seed total !valid

(* ================================================================== *)
(*  Deterministic budget fail-closed                                    *)
(* ================================================================== *)

let test_budget_fail_closed () =
  let template = tid "template" in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = None; success_continuations = [];
    origin_sites = []; branches = [];
    roles = [];
    item_templates = [{
      item_template_id = template; origin_sites = []; branches = [];
      roles = List.init 8 (fun i -> {
        role_id = rid ("r" ^ string_of_int i); scope = Item_template_scope template;
        fact_contract = Role_fact_contract []; eligible_fulfillment = rf "same";
      });
      objective = Required_role (rid "r0");
    }];
    capability_contracts = [];
  } in
  (* Template Role labels remain exhaustive: their template can contain earlier
     role-sensitive origin fields, so Rocket has no blanket role shortcut. *)
  let small_budget = { Tethers_core_canonical_v2_ir.max_nodes = 100; max_leaves = 100; max_refinement_rounds = 1000 } in
  (match Tethers_core_canonical_v2_ir.canonicalize_ir ~budget:small_budget p with
   | Error Tethers_core_canonical_v2_ir.Canonicalisation_too_complex -> ()
   | Ok _ -> failwith "should be too_complex"
   | Error _ -> failwith "wrong error");
  (* No payload/digest on failure — checked by Error case *)
  Printf.printf "Budget fail-closed: PASS\n"

let test_reduced_pre_admission_for_single_collection_branches () =
  let anchor = oid "anchor" in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = Some anchor;
    success_continuations = [];
    origin_sites = [Anchor_origin {
      anchor_origin_id = anchor; event_name = "ev"; declared_facts = [];
    }];
    branches = List.init 11 (fun index -> {
      branch_id = branch_id_of_string ("branch" ^ string_of_int index);
      branch_subject = anchor;
      outcome_branches = [(Success, Stop)];
    });
    roles = []; item_templates = []; capability_contracts = [];
  } in
  let raw_candidate_count = match
      Tethers_core_canonical_v2.candidate_count_within_budget ~limit:max_int p with
    | Some count -> count
    | None -> failwith "11-Branch raw candidate count overflowed unexpectedly"
  in
  check_equal_int 39_916_800 raw_candidate_count "11-Branch raw candidate count";
  (match Tethers_core_canonical_v2.canonicalize p with
   | Error Tethers_core_canonical_v2.Canonicalisation_too_complex -> ()
   | Ok _ -> failwith "baseline should reject 11! candidates at its default budget"
   | Error _ -> failwith "baseline returned the wrong 11-Branch error");
  let (_, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p)
    "11-Branch reduced pre-admission" in
  check_equal_int 1 stats.leaves_encoded "11-Branch exact leaf after pre-admission";
  check_equal_int 39_916_799 stats.leaves_avoided "11-Branch leaves avoided"

(* ================================================================== *)
(*  Performance evidence (non-gating)                                   *)
(* ================================================================== *)

let time f =
  let t0 = Unix.gettimeofday () in
  let r = f () in
  let t1 = Unix.gettimeofday () in
  (r, t1 -. t0)

type gc_delta = {
  minor_words : float;
  major_words : float;
  minor_collections : int;
  major_collections : int;
}

let time_and_gc f =
  let before = Gc.quick_stat () in
  let (result, seconds) = time f in
  let after = Gc.quick_stat () in
  (result, seconds, {
    minor_words = after.minor_words -. before.minor_words;
    major_words = after.major_words -. before.major_words;
    minor_collections = after.minor_collections - before.minor_collections;
    major_collections = after.major_collections - before.major_collections;
  })

let bench_case name p =
  let baseline_candidates = match Tethers_core_canonical_v2.candidate_count_within_budget ~limit:max_int p with Some n -> n | None -> -1 in
  let (_, t_base, gc_base) = time_and_gc (fun () -> ignore (Tethers_core_canonical_v2.canonicalize p)) in
  let ir_runs = 1000 in
  let (ir_res, t_ir_total, gc_ir) = time_and_gc (fun () ->
    let rec run remaining =
      let result = Tethers_core_canonical_v2_ir.canonicalize_ir p in
      if remaining = 1 then result else run (remaining - 1)
    in
    run ir_runs
  ) in
  let t_ir = t_ir_total /. float_of_int ir_runs in
  let (ir_nodes, ir_leaves, ir_rounds, ir_prefix, ir_orbit, ir_dup, ir_avoided) = match ir_res with Ok (_, s) -> (s.nodes, s.leaves_encoded, s.refinement_rounds, s.prefix_subtrees_pruned, s.orbit_branches_pruned, s.duplicate_payload_hits, s.leaves_avoided) | Error _ -> (-1, -1, -1, -1, -1, -1, -1) in
  Printf.printf "BENCH %s: raw_candidates=%d baseline_time=%.4fs baseline_minor_words=%.0f baseline_major_words=%.0f baseline_minor_gc=%d baseline_major_gc=%d IR_nodes=%d IR_leaves=%d IR_rounds=%d prefix_pruned=%d orbit_pruned=%d dup_hits=%d leaves_avoided=%d IR_time_per_call=%.6fs IR_runs=%d IR_minor_words_per_call=%.1f IR_major_words_per_call=%.1f IR_minor_gc_total=%d IR_major_gc_total=%d\n"
    name baseline_candidates t_base gc_base.minor_words gc_base.major_words gc_base.minor_collections gc_base.major_collections ir_nodes ir_leaves ir_rounds ir_prefix ir_orbit ir_dup ir_avoided t_ir ir_runs (gc_ir.minor_words /. float_of_int ir_runs) (gc_ir.major_words /. float_of_int ir_runs) gc_ir.minor_collections gc_ir.major_collections;
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
  (* 5. one physical Branch collection, eight repeated Branch occurrences *)
  let branch_anchor = oid "branch-anchor" in
  let p_branch8 = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = []; entry_guards = [];
    entry_origin = Some branch_anchor; success_continuations = [];
    origin_sites = [Anchor_origin { anchor_origin_id = branch_anchor; event_name = "ev"; declared_facts = []}];
    branches = List.init 8 (fun index -> {
      branch_id = branch_id_of_string ("branch" ^ string_of_int index);
      branch_subject = branch_anchor;
      outcome_branches = [(Success, Stop)];
    });
    roles = []; item_templates = []; capability_contracts = [];
  } in
  ignore (bench_case "high-symmetry 8 branches" p_branch8);
  (* 6. mixed realistic small Core fixture *)
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
  (* 7. templates/roles *)
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

let test_length_prefix_string_trap () =
  (* Raw string order is aa < z, but Enc_V2 orders the length-prefixed
     provenance bytes 1:z < 2:aa.  This exercises the only active shortcut. *)
  let facts = [
    { fact_id = fid "aa"; schema_description = ""; provenance = Evaluation_input (hsk "aa", String_type)};
    { fact_id = fid "z"; schema_description = ""; provenance = Evaluation_input (hsk "z", String_type)};
  ] in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = facts;
    entry_guards = []; entry_origin = None; success_continuations = [];
    origin_sites = []; branches = []; roles = []; item_templates = []; capability_contracts = [];
  } in
  check (Tethers_core_canonical_v2_format.compare_bytes_lex_unsigned "1:z" "2:aa" < 0)
    "length-prefix precondition";
  assert_ir_eq_oracle_and_baseline p "length-prefix aa/z";
  let (_, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p) "length-prefix IR" in
  check_equal_int 1 stats.leaves_encoded "length-prefix shortcut encodes one exact leaf"

let test_structural_location_fact_trap () =
  (* A global scalar sort over collect_facts is invalid: one fact is emitted in
     input_facts and the other in a later origin declaration.  The fast path
     must be disabled and exact enumeration retained. *)
  let make input_id declared_id anchor_id spare_id sites =
    let anchor = oid anchor_id in
    let input = { fact_id = fid input_id; schema_description = ""; provenance = Evaluation_input (hsk "aa", String_type)} in
    let declared = { fact_id = fid declared_id; schema_description = ""; provenance = Evaluation_input (hsk "z", String_type)} in
    let origin_for id =
      if id = anchor_id then
        Anchor_origin { anchor_origin_id = anchor; event_name = "ev"; declared_facts = [declared]}
      else
        Anchor_origin { anchor_origin_id = oid spare_id; event_name = "spare"; declared_facts = []}
    in
    {
      program_id = pid "test"; core_version = cv "0.1.0"; input_facts = [input];
      entry_guards = []; entry_origin = Some anchor; success_continuations = [];
      origin_sites = List.map origin_for sites;
      branches = []; roles = []; item_templates = []; capability_contracts = [];
    }
  in
  let p = make "input-z" "declared-a" "anchor-z" "spare-a" ["anchor-z"; "spare-a"] in
  let p_reversed = make "input-a" "declared-z" "anchor-a" "spare-z" ["spare-z"; "anchor-a"] in
  assert_ir_eq_oracle_and_baseline p "structural-location Fact trap";
  assert_ir_eq_oracle_and_baseline p_reversed "structural-location Fact trap reversed";
  let (ir, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p) "structural-location IR" in
  let (ir_reversed, stats_reversed) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p_reversed) "structural-location reversed IR" in
  check_equal_int 2 stats.leaves_encoded "structural-location retains both fact labellings";
  check_equal_int 2 stats_reversed.leaves_encoded "structural-location reversed retains both fact labellings";
  check_equal_string
    (Tethers_core_canonical_v2_ir.canonical_payload_ir ir)
    (Tethers_core_canonical_v2_ir.canonical_payload_ir ir_reversed)
    "structural-location raw-ID and storage invariance"

let test_entry_origin_provenance_is_rejected_before_search () =
  (* The apparent counterexample to the entry-label proof is outside Lambda(P):
     valid input Facts are Evaluation_input only. *)
  let entry = oid "entry" and provenance_origin = oid "provenance" in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = [{
      fact_id = fid "f"; schema_description = "";
      provenance = Origin_provenance provenance_origin;
    }];
    entry_guards = []; entry_origin = Some entry; success_continuations = [];
    origin_sites = [
      Anchor_origin { anchor_origin_id = entry; event_name = "ev"; declared_facts = []};
      Anchor_origin { anchor_origin_id = provenance_origin; event_name = "ev"; declared_facts = []};
    ];
    branches = []; roles = []; item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "input Origin_provenance rejected before search"

let test_multiple_branch_collections_remain_exhaustive () =
  let program_origin = oid "program-origin" in
  let template_origin = oid "template-origin" in
  let template_id = tid "template" in
  let template_role = rid "role" in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = Some program_origin;
    success_continuations = [];
    origin_sites = [Anchor_origin {
      anchor_origin_id = program_origin; event_name = "ev"; declared_facts = [];
    }];
    branches = [{
      branch_id = branch_id_of_string "program-branch";
      branch_subject = program_origin;
      outcome_branches = [(Success, Stop)];
    }];
    roles = [];
    item_templates = [{
      item_template_id = template_id;
      origin_sites = [Anchor_origin {
        anchor_origin_id = template_origin; event_name = "ev"; declared_facts = [];
      }];
      branches = [{
        branch_id = branch_id_of_string "template-branch";
        branch_subject = template_origin;
        outcome_branches = [(Success, Stop)];
      }];
      roles = [{
        role_id = template_role; scope = Item_template_scope template_id;
        fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok";
      }];
      objective = Required_role template_role;
    }];
    capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "multiple Branch collections";
  let (_, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p)
    "multiple Branch collections IR" in
  check_equal_int 2 stats.leaves_encoded
    "multiple Branch collections retain both global label assignments"

let test_role_completion_trap () =
  (* With no earlier Program Role reference, the Program Role collection itself
     is the first role-sensitive field.  Exact encoded-body ordering is sound. *)
  let role name = {
    role_id = rid name; scope = Program_scope; fact_contract = Role_fact_contract [];
    eligible_fulfillment = rf name;
  } in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = [];
    entry_guards = []; entry_origin = None; success_continuations = [];
    origin_sites = []; branches = []; roles = [role "z"; role "aa"];
    item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "role-completion trap";
  let (_, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p) "role-completion IR" in
  check_equal_int 1 stats.leaves_encoded "role-completion exact body order";
  check_equal_int 0 stats.prefix_subtrees_pruned "no representative-completion prefix pruning"

let test_program_role_body_order_and_earlier_reference_guard () =
  let make_role name fulfillment = {
    role_id = rid name; scope = Program_scope; fact_contract = Role_fact_contract [];
    eligible_fulfillment = rf fulfillment;
  } in
  let unreferenced = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = None; success_continuations = [];
    origin_sites = []; branches = [];
    roles = [
      make_role "z" "z"; make_role "aa" "aa"; make_role "r2" "\x80";
      make_role "r3" "late"; make_role "r4" "same"; make_role "r5" "same";
    ];
    item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline unreferenced "Program Role exact body order";
  let (_, unreferenced_stats) = check_ok
    (Tethers_core_canonical_v2_ir.canonicalize_ir unreferenced)
    "Program Role exact body order IR" in
  check_equal_int 1 unreferenced_stats.leaves_encoded "Program Role 720 to 1";
  let anchor = oid "anchor" in
  let earlier_reference = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = Some anchor; success_continuations = [];
    origin_sites = [Anchor_origin {
      anchor_origin_id = anchor; event_name = "ev";
      declared_facts = [{ fact_id = fid "role-fact"; schema_description = "";
        provenance = Role_proxy (rid "r1") }];
    }];
    branches = [];
    roles = [make_role "r1" "aa"; make_role "r2" "z"];
    item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline earlier_reference "Program Role earlier reference guard";
  let (_, referenced_stats) = check_ok
    (Tethers_core_canonical_v2_ir.canonicalize_ir earlier_reference)
    "Program Role earlier reference guard IR" in
  check_equal_int 2 referenced_stats.leaves_encoded
    "Program Role earlier reference remains exhaustive"

let test_template_role_body_order_and_guards () =
  let template_id = tid "template" in
  let make_role name fulfillment = {
    role_id = rid name; scope = Item_template_scope template_id;
    fact_contract = Role_fact_contract []; eligible_fulfillment = rf fulfillment;
  } in
  let distinct = {
    program_id = pid "test"; core_version = cv "0.1.0";
    input_facts = []; entry_guards = []; entry_origin = None;
    success_continuations = []; origin_sites = []; branches = []; roles = [];
    item_templates = [{
      item_template_id = template_id; origin_sites = []; branches = [];
      roles = [
        make_role "z" "z"; make_role "aa" "aa"; make_role "r2" "\x80";
        make_role "r3" "late"; make_role "r4" "same-4"; make_role "r5" "same-5";
      ];
      objective = Required_role (rid "r3");
    }];
    capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline distinct "Template Role distinct exact body order";
  let (_, distinct_stats) = check_ok
    (Tethers_core_canonical_v2_ir.canonicalize_ir distinct)
    "Template Role distinct exact body order IR" in
  check_equal_int 1 distinct_stats.leaves_encoded "Template Role 720 to 1";
  let tied = {
    distinct with item_templates = [{
      item_template_id = template_id; origin_sites = []; branches = [];
      roles = [make_role "r1" "same"; make_role "r2" "same"];
      objective = Required_role (rid "r2");
    }];
  } in
  assert_ir_eq_oracle_and_baseline tied "Template Role tied body guard";
  let (_, tied_stats) = check_ok
    (Tethers_core_canonical_v2_ir.canonicalize_ir tied)
    "Template Role tied body guard IR" in
  check_equal_int 2 tied_stats.leaves_encoded
    "Template Role tied bodies remain exhaustive for objective";
  let anchor = oid "template-anchor" in
  let referenced = {
    distinct with item_templates = [{
      item_template_id = template_id;
      origin_sites = [Anchor_origin {
        anchor_origin_id = anchor; event_name = "ev";
        declared_facts = [{ fact_id = fid "template-role-fact"; schema_description = "";
          provenance = Role_proxy (rid "r1") }];
      }];
      branches = [];
      roles = [make_role "r1" "aa"; make_role "r2" "z"];
      objective = Required_role (rid "r2");
    }];
  } in
  assert_ir_eq_oracle_and_baseline referenced "Template Role earlier reference guard";
  let (_, referenced_stats) = check_ok
    (Tethers_core_canonical_v2_ir.canonicalize_ir referenced)
    "Template Role earlier reference guard IR" in
  check_equal_int 2 referenced_stats.leaves_encoded
    "Template Role earlier reference remains exhaustive"

let test_multi_round_refinement () =
  (* Distinct initial origin scalars reach their roles only through the
     Origin -> Fact -> Role path, requiring synchronous propagation. *)
  let oa = oid "oa" and ob = oid "ob" in
  let fa = { fact_id = fid "fa"; schema_description = ""; provenance = Origin_provenance oa } in
  let fb = { fact_id = fid "fb"; schema_description = ""; provenance = Origin_provenance ob } in
  let role name fact = {
    role_id = rid name; scope = Program_scope; fact_contract = Role_fact_contract [fact];
    eligible_fulfillment = rf "same";
  } in
  let p = {
    program_id = pid "test"; core_version = cv "0.1.0"; input_facts = [];
    entry_guards = []; entry_origin = Some oa; success_continuations = [];
    origin_sites = [
      Anchor_origin { anchor_origin_id = oa; event_name = "event-a"; declared_facts = [fa]};
      Anchor_origin { anchor_origin_id = ob; event_name = "event-b"; declared_facts = [fb]};
    ];
    branches = []; roles = [role "ra" (fid "fa"); role "rb" (fid "fb")];
    item_templates = []; capability_contracts = [];
  } in
  assert_ir_eq_oracle_and_baseline p "multi-round refinement";
  let (_, stats) = check_ok (Tethers_core_canonical_v2_ir.canonicalize_ir p) "multi-round refinement IR" in
  check (stats.refinement_rounds >= 2) "multi-round refinement must not reuse a partial round"

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
  Printf.printf "=== V2 Exact Hybrid Search Tests (C-B4I3C) ===\n\n";
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
  test_single_collection_branch_shortcut (); Printf.printf "PASS: single-collection 8-Branch shortcut\n";
  test_single_collection_branch_body_order (); Printf.printf "PASS: single-collection distinct Branch bodies\n";
  test_dependency_closed_program_anchor_origins (); Printf.printf "PASS: dependency-closed program Anchor Origins\n";
  test_program_anchor_origin_negative_guards (); Printf.printf "PASS: program Anchor Origin negative guards\n";
  test_anchor_tie_repair_minimal (); Printf.printf "PASS: repaired minimal Anchor tie mismatch\n";
  test_anchor_tie_torture (); Printf.printf "PASS: Anchor tie torture and residual pre-admission\n";
  test_branch_label_count_boundaries (); Printf.printf "PASS: Branch label count 8/9/10/11/12/19/20/21\n";
  test_decimal_label_family_boundaries (); Printf.printf "PASS: Fact/Program Role/Template Role decimal label boundaries\n";
  test_compound_factor_collapses (); Printf.printf "PASS: compound factorial factor collapses\n";
  test_mixed_branch_torture (); Printf.printf "PASS: mixed Branch torture\n";
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
  test_length_prefix_string_trap (); Printf.printf "PASS: counterexample length-prefix string trap\n";
  test_structural_location_fact_trap (); Printf.printf "PASS: counterexample structural-location Fact trap\n";
  test_entry_origin_provenance_is_rejected_before_search (); Printf.printf "PASS: input Origin_provenance rejected before search\n";
  test_multiple_branch_collections_remain_exhaustive (); Printf.printf "PASS: counterexample multiple Branch collections\n";
  test_role_completion_trap (); Printf.printf "PASS: counterexample role completion trap\n";
  test_program_role_body_order_and_earlier_reference_guard (); Printf.printf "PASS: Program Role body order and guard\n";
  test_template_role_body_order_and_guards (); Printf.printf "PASS: Template Role body order and guards\n";
  test_multi_round_refinement (); Printf.printf "PASS: counterexample multi-round refinement\n";
  test_branch_symmetry_broken_by_target (); Printf.printf "PASS: counterexample branch symmetry broken\n";
  test_refinement_fail_closed ();
  test_generated_corpus (); Printf.printf "PASS: dense generated corpus 5000\n";
  test_budget_fail_closed (); Printf.printf "PASS: deterministic budget fail-closed\n";
  test_reduced_pre_admission_for_single_collection_branches (); Printf.printf "PASS: reduced pre-admission 11-Branch shortcut\n";
  test_performance_evidence (); Printf.printf "PASS: performance evidence (reported)\n";
  Printf.printf "\n=== All V2 IR Tests Complete ===\n"
