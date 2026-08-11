open Tethers_core
open Tethers_core_validator

(* ------------------------------------------------------------------ *)
(*  Test harness                                                       *)
(* ------------------------------------------------------------------ *)

let tests_run = ref 0
let tests_passed = ref 0

let assert_true msg condition =
  incr tests_run;
  if condition then begin
    incr tests_passed
  end else begin
    Printf.eprintf "FAIL: %s\n" msg;
    exit 1
  end

let assert_ok msg = function
  | Ok _ -> begin incr tests_run; incr tests_passed end
  | Error _ -> begin
      incr tests_run;
      Printf.eprintf "FAIL: %s (expected Ok, got Error)\n" msg;
      exit 1
    end

let assert_error msg = function
  | Ok _ -> begin
      incr tests_run;
      Printf.eprintf "FAIL: %s (expected Error, got Ok)\n" msg;
      exit 1
    end
  | Error _ -> begin incr tests_run; incr tests_passed end

let assert_has_error expected msg result =
  incr tests_run;
  match result with
  | Ok _ -> begin
      Printf.eprintf "FAIL: %s (expected Error, got Ok)\n" msg;
      exit 1
    end
  | Error errs ->
      if List.mem expected errs then incr tests_passed
      else begin
        Printf.eprintf "FAIL: %s (expected error not found)\n" msg;
        exit 1
      end

(* ------------------------------------------------------------------ *)
(*  Helper constructors                                                 *)
(* ------------------------------------------------------------------ *)

let oid s = origin_id_of_string s
let fid s = fact_id_of_string s
let rid s = role_id_of_string s
let cid s = capability_id_of_string s
let bid s = branch_id_of_string s
let gid s = group_id_of_string s
let btid s = batch_id_of_string s
let tid s = item_template_id_of_string s
let pid s = program_id_of_string s
let cv s = core_version_of_string s
let hsk s = host_snapshot_key_of_string s
let ccd s = capability_contract_digest_of_string s
let cin s = capability_input_name_of_string s

let input_fact name =
  { fact_id = fid ("F_" ^ name);
    schema_description = "";
    provenance = Evaluation_input (hsk ("K_" ^ name), String_type);
  }

let anchor_origin_record =
  { anchor_origin_id = oid "O_anchor";
    event_name = "file.received";
    declared_facts = [];
  }

let action_origin_record id_name cap_id digest =
  { action_origin_id = oid id_name;
    capability_id = cid cap_id;
    contract_digest = ccd digest;
    inputs = [];
    declared_facts = [];
    execution_constraints = [];
  }

let cap_contract cap_id digest =
  { capability_id = cid cap_id;
    contract_digest = ccd digest;
    schema_description = "";
  }

let basic_program () =
  let action = action_origin_record "O1" "C_notify" "D_notify_v1" in
  let entry_origin = Some (oid "O1") in
  {
    program_id = pid "P_test";
    core_version = cv "0.1.0";
    input_facts = [ input_fact "file_type" ];
    entry_guards =
      [ { fact_id = fid "F_file_type"; operator = Equals; expected = String_value "pdf" } ];
    entry_origin;
    success_continuations =
      [ { from_origin = oid "O1"; target = Program_complete } ];
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action ];
    branches = [];
    roles = [];
    item_templates = [];
    capability_contracts = [ cap_contract "C_notify" "D_notify_v1" ];
  }

let action_with_inputs id_name cap_id digest inputs =
  { action_origin_id = oid id_name;
    capability_id = cid cap_id;
    contract_digest = ccd digest;
    inputs;
    declared_facts = [];
    execution_constraints = [];
  }

(* ------------------------------------------------------------------ *)
(*  Test 1: Valid lowered CORE-2 Program                               *)
(* ------------------------------------------------------------------ *)

let test_valid_program () =
  let program = basic_program () in
  assert_ok "valid basic program" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 2: Duplicate Origin                                           *)
(* ------------------------------------------------------------------ *)

let test_duplicate_origin () =
  let action1 = action_origin_record "O_same" "C_notify" "D_notify_v1" in
  let action2 = action_origin_record "O_same" "C_save" "D_save_v1" in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record;
                     Action_origin action1; Action_origin action2 ];
    entry_origin = Some (oid "O_same");
    success_continuations = [ { from_origin = oid "O_same"; target = Program_complete } ];
  } in
  assert_has_error (Duplicate_origin_id (oid "O_same"))
    "duplicate origin rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 3: Missing entry target                                       *)
(* ------------------------------------------------------------------ *)

let test_missing_entry_origin_for_actions () =
  let action = action_origin_record "O1" "C_notify" "D_notify_v1" in
  let program = { (basic_program ()) with
    entry_origin = None;
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action ];
  } in
  assert_has_error Missing_entry_origin_for_actions
    "missing entry origin rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 4: Duplicate success continuation                              *)
(* ------------------------------------------------------------------ *)

let test_duplicate_success_continuation () =
  let action = action_origin_record "O1" "C_notify" "D_notify_v1" in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action ];
    success_continuations = [
      { from_origin = oid "O1"; target = Program_complete };
      { from_origin = oid "O1"; target = Program_complete };
    ];
  } in
  assert_has_error (Duplicate_success_continuation (oid "O1"))
    "duplicate success continuation rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 5: Success-flow self-cycle                                     *)
(* ------------------------------------------------------------------ *)

let test_self_cycle () =
  let action1 = action_origin_record "O_a" "C_notify" "D_notify_v1" in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action1 ];
    entry_origin = Some (oid "O_a");
    success_continuations = [ { from_origin = oid "O_a"; target = Origin_target (oid "O_a") } ];
  } in
  assert_has_error (Success_cycle [oid "O_a"; oid "O_a"])
    "self-cycle rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 6: Multi-node success cycle                                    *)
(* ------------------------------------------------------------------ *)

let test_multi_node_cycle () =
  let action1 = action_origin_record "O_a" "C_notify" "D_notify_v1" in
  let action2 = action_origin_record "O_b" "C_save" "D_save_v1" in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record;
                     Action_origin action1; Action_origin action2 ];
    entry_origin = Some (oid "O_a");
    success_continuations = [
      { from_origin = oid "O_a"; target = Origin_target (oid "O_b") };
      { from_origin = oid "O_b"; target = Origin_target (oid "O_a") };
    ];
  } in
  assert_error "multi-node cycle rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 7: Missing Capability contract                                 *)
(* ------------------------------------------------------------------ *)

let test_missing_capability_contract () =
  let action = action_origin_record "O1" "C_notify" "D_notify_v1" in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action ];
    capability_contracts = [];
  } in
  assert_has_error (Missing_capability_contract (cid "C_notify"))
    "missing capability contract rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 8: Contract digest mismatch                                    *)
(* ------------------------------------------------------------------ *)

let test_contract_digest_mismatch () =
  let action = action_origin_record "O1" "C_notify" "D_notify_v1" in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action ];
    capability_contracts = [ cap_contract "C_notify" "D_notify_v2" ];
  } in
  assert_has_error (Capability_contract_digest_mismatch (cid "C_notify"))
    "contract digest mismatch rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 9: Duplicate input Fact ID                                     *)
(* ------------------------------------------------------------------ *)

let test_duplicate_input_fact () =
  let f1 = input_fact "same_name" in
  let f2 = input_fact "same_name" in
  let program = { (basic_program ()) with
    input_facts = [ f1; f2 ];
  } in
  assert_has_error (Duplicate_fact_id (fid "F_same_name"))
    "duplicate input fact rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 10: Guard unknown Fact                                         *)
(* ------------------------------------------------------------------ *)

let test_guard_unknown_fact () =
  let program = { (basic_program ()) with
    entry_guards = [ { fact_id = fid "F_unknown"; operator = Equals; expected = String_value "x" } ];
  } in
  assert_has_error (Input_fact_not_declared (fid "F_unknown"))
    "guard unknown fact rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 11: Bad Anchor Origin reference                                *)
(* ------------------------------------------------------------------ *)

let test_bad_anchor_origin () =
  let action = action_with_inputs "O1" "C_notify" "D_notify_v1"
    [ { input_name = cin "ref";
        binding = Anchor_value (oid "O_nonexistent", ["a"; "b"]) } ]
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action ];
  } in
  assert_has_error (Missing_origin (oid "O_nonexistent"))
    "bad anchor origin reference rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 12: Anchor path empty                                          *)
(* ------------------------------------------------------------------ *)

let test_anchor_path_empty () =
  let action = action_with_inputs "O1" "C_notify" "D_notify_v1"
    [ { input_name = cin "ref";
        binding = Anchor_value (oid "O_anchor", []) } ]
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action ];
  } in
  assert_has_error Anchor_path_empty
    "anchor path empty rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 13: Fact provenance missing Origin                             *)
(* ------------------------------------------------------------------ *)

let test_fact_origin_missing () =
  let declared_fact =
    { fact_id = fid "F_result";
      schema_description = "";
      provenance = Origin_provenance (oid "O_nonexistent");
    }
  in
  let action = { (action_origin_record "O1" "C_notify" "D_notify_v1") with
    declared_facts = [ declared_fact ];
  } in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action ];
  } in
  assert_has_error (Fact_origin_provenance_missing_origin (fid "F_result"))
    "fact provenance missing origin rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 14: Fact_from_origin provenance mismatch                       *)
(* ------------------------------------------------------------------ *)

let test_fact_from_origin_provenance_mismatch () =
  let declared_fact =
    { fact_id = fid "F_result";
      schema_description = "";
      provenance = Origin_provenance (oid "O2");
    }
  in
  let action1 = { (action_origin_record "O1" "C_notify" "D_notify_v1") with
    inputs = [ { input_name = cin "v";
                 binding = Fact_from_origin (fid "F_result", oid "O1") } ];
    declared_facts = [];
  } in
  let action2 = { (action_origin_record "O2" "C_save" "D_save_v1") with
    declared_facts = [ declared_fact ];
  } in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record;
                     Action_origin action1; Action_origin action2 ];
    entry_origin = Some (oid "O1");
    success_continuations = [
      { from_origin = oid "O1"; target = Origin_target (oid "O2") };
      { from_origin = oid "O2"; target = Program_complete };
    ];
    capability_contracts = [ cap_contract "C_notify" "D_notify_v1";
                             cap_contract "C_save" "D_save_v1" ];
  } in
  assert_has_error (Fact_from_origin_provenance_mismatch (fid "F_result", oid "O1"))
    "fact from origin provenance mismatch rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 15: Fact_through_role missing role                             *)
(* ------------------------------------------------------------------ *)

let test_role_missing () =
  let action = action_with_inputs "O1" "C_notify" "D_notify_v1"
    [ { input_name = cin "v";
        binding = Fact_through_role (fid "F_file_type", rid "R_nonexistent") } ]
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action ];
  } in
  assert_has_error (Missing_role (rid "R_nonexistent"))
    "missing role rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 16: Role Fact Contract mismatch                                *)
(* ------------------------------------------------------------------ *)

let test_role_fact_contract_mismatch () =
  let role =
    { role_id = rid "R1";
      scope = Program_scope;
      fact_contract = Role_fact_contract [ fid "F_x" ];
      eligible_fulfillment = role_fulfillment_of_string "opaque";
    }
  in
  let action = action_with_inputs "O1" "C_notify" "D_notify_v1"
    [ { input_name = cin "v";
        binding = Fact_through_role (fid "F_y", rid "R1") } ]
  in
  (* F_y must exist so the contract mismatch, not Missing_fact, fires *)
  let declared_f =
    { fact_id = fid "F_y";
      schema_description = "";
      provenance = Origin_provenance (oid "O_notify");
    }
  in
  let notify_action = { (action_origin_record "O_notify" "C_notify" "D_notify_v1") with
    declared_facts = [ declared_f ];
  } in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record;
                     Action_origin notify_action; Action_origin action ];
    entry_origin = Some (oid "O_notify");
    success_continuations = [
      { from_origin = oid "O_notify"; target = Origin_target (oid "O1") };
      { from_origin = oid "O1"; target = Program_complete };
    ];
    roles = [ role ];
  } in
  assert_has_error (Fact_role_contract_not_exposed (fid "F_y", rid "R1"))
    "role fact contract mismatch rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 17: Branch duplicate Outcome                                   *)
(* ------------------------------------------------------------------ *)

let test_branch_duplicate_outcome () =
  let action1 = action_origin_record "O1" "C_notify" "D_notify_v1" in
  let action2 = action_origin_record "O2" "C_save" "D_save_v1" in
  let branch =
    { branch_id = bid "B1";
      branch_subject = oid "O1";
      outcome_branches = [ (Success, Continue_to (oid "O2"));
                           (Success, Stop) ];
    }
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record;
                     Action_origin action1; Action_origin action2 ];
    entry_origin = Some (oid "O1");
    success_continuations = [
      { from_origin = oid "O1"; target = Program_complete };
      { from_origin = oid "O2"; target = Program_complete };
    ];
    branches = [ branch ];
    capability_contracts = [ cap_contract "C_notify" "D_notify_v1";
                             cap_contract "C_save" "D_save_v1" ];
  } in
  assert_has_error (Branch_duplicate_outcome (bid "B1"))
    "branch duplicate outcome rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 18: Branch missing target                                      *)
(* ------------------------------------------------------------------ *)

let test_branch_missing_target () =
  let action1 = action_origin_record "O1" "C_notify" "D_notify_v1" in
  let branch =
    { branch_id = bid "B2";
      branch_subject = oid "O1";
      outcome_branches = [ (Failure, Continue_to (oid "O_nonexistent")) ];
    }
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action1 ];
    branches = [ branch ];
  } in
  assert_has_error (Missing_branch_target (oid "O_nonexistent"))
    "branch missing target rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 19: Together one member                                        *)
(* ------------------------------------------------------------------ *)

let test_together_one_member () =
  let action1 = action_origin_record "O1" "C_notify" "D_notify_v1" in
  let together =
    { together_origin_id = oid "T1";
      group_id = gid "G1";
      member_origin_ids = [ oid "O1" ];
      objective = All_members_succeed;
    }
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record;
                     Action_origin action1;
                     Together_origin together ];
    entry_origin = Some (oid "T1");
    success_continuations = [ { from_origin = oid "T1"; target = Program_complete } ];
  } in
  assert_has_error (Together_single_member (gid "G1"))
    "together one member rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 20: Together duplicate member                                  *)
(* ------------------------------------------------------------------ *)

let test_together_duplicate_member () =
  let action1 = action_origin_record "O1" "C_notify" "D_notify_v1" in
  let action2 = action_origin_record "O2" "C_save" "D_save_v1" in
  let together =
    { together_origin_id = oid "T1";
      group_id = gid "G2";
      member_origin_ids = [ oid "O1"; oid "O1" ];
      objective = All_members_succeed;
    }
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record;
                     Action_origin action1; Action_origin action2;
                     Together_origin together ];
    entry_origin = Some (oid "T1");
    success_continuations = [ { from_origin = oid "T1"; target = Program_complete } ];
    capability_contracts = [ cap_contract "C_notify" "D_notify_v1";
                             cap_contract "C_save" "D_save_v1" ];
  } in
  assert_has_error (Together_duplicate_member (gid "G2"))
    "together duplicate member rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 21: Together unknown member                                    *)
(* ------------------------------------------------------------------ *)

let test_together_unknown_member () =
  let together =
    { together_origin_id = oid "T1";
      group_id = gid "G3";
      member_origin_ids = [ oid "O_ghost" ];
      objective = All_members_succeed;
    }
  in
  let action1 = action_origin_record "O1" "C_notify" "D_notify_v1" in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record;
                     Action_origin action1;
                     Together_origin together ];
    entry_origin = Some (oid "T1");
    success_continuations = [ { from_origin = oid "T1"; target = Program_complete } ];
  } in
  assert_has_error (Together_unknown_member (gid "G3", oid "O_ghost"))
    "together unknown member rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 22: Item objective missing Role                                *)
(* ------------------------------------------------------------------ *)

let test_item_objective_missing_role () =
  let item_template =
    { item_template_id = tid "IT1";
      origin_sites = [];
      branches = [];
      roles = [];
      objective = Required_role (rid "R_nonexistent");
    }
  in
  let program = { (basic_program ()) with
    item_templates = [ item_template ];
  } in
  assert_has_error (Item_objective_missing_role (tid "IT1", rid "R_nonexistent"))
    "item objective missing role rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 23: Batch missing Item Template                                *)
(* ------------------------------------------------------------------ *)

let test_batch_missing_item_template () =
  let batch =
    { batch_id = btid "BAT1";
      collection_provenance = batch_collection_provenance_of_string "opaque";
      item_template_id = tid "IT_nonexistent";
      traversal_policy = batch_traversal_policy_of_string "opaque";
      composite_objective = batch_objective_of_string "opaque";
      aggregate_facts = [];
    }
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Batch_site batch ];
    entry_origin = None;
  } in
  assert_has_error (Batch_missing_item_template (btid "BAT1"))
    "batch missing item template rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  Test 24: Determinism                                                *)
(* ------------------------------------------------------------------ *)

let test_determinism () =
  let action = action_origin_record "O1" "C_notify" "D_notify_v1" in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action ];
    entry_origin = Some (oid "O1");
    success_continuations = [ { from_origin = oid "O1"; target = Program_complete } ];
  } in
  let r1 = validate program in
  let r2 = validate program in
  let r3 = validate program in
  assert_true "determinism r1=r2" (r1 = r2);
  assert_true "determinism r1=r3" (r1 = r3);
  assert_true "determinism r2=r3" (r2 = r3)

(* ------------------------------------------------------------------ *)
(*  Integration test: parse → lower → validate → OK                    *)
(* ------------------------------------------------------------------ *)

let test_integration_lower_validate () =
  let parse source =
    try Ok (Tether_parser.parse_tether source) with
    | Tethers_error.Tethers_error _ -> Error "parse_error"
  in
  let env =
    let open Tethers_core_lowerer in
    { program_id = pid "P_test";
      core_version = cv "0.1.0";
      capabilities =
        [ { source_name = "notify";
            capability_id = cid "C_notify";
            contract_digest = ccd "D_notify_v1" };
          { source_name = "save";
            capability_id = cid "C_save";
            contract_digest = ccd "D_save_v1" };
        ];
      input_facts =
        [ { source_name = "file_type";
            fact = { fact_id = fid "F_file_type";
                     schema_description = "";
                     provenance = Evaluation_input (hsk "K_file_type", String_type);
                   };
          };
          { source_name = "customer";
            fact = { fact_id = fid "F_customer";
                     schema_description = "";
                     provenance = Evaluation_input (hsk "K_customer", String_type);
                   };
          };
        ];
    }
  in
  let source = {|
tether "integration test"
anchor
    file.received
when
    file_type is "pdf"
do
    notify
        message: "start"
    save
        file: anchor.document
|} in
  match parse source with
  | Ok tether ->
      (match Tethers_core_lowerer.lower env tether with
       | Ok program ->
           assert_ok "integration: lowered program validates"
             (validate program)
       | Error _ ->
           Printf.eprintf "FAIL: integration lowering error\n"; exit 1)
  | Error _ ->
      Printf.eprintf "FAIL: integration parse error\n"; exit 1

(* ------------------------------------------------------------------ *)
(*  Additional edge cases                                               *)
(* ------------------------------------------------------------------ *)

let test_input_fact_wrong_provenance () =
  let bad_fact =
    { fact_id = fid "F_bad";
      schema_description = "";
      provenance = Origin_provenance (oid "O_anchor");
    }
  in
  let program = { (basic_program ()) with
    input_facts = [ bad_fact; input_fact "file_type" ];
    entry_guards = [ { fact_id = fid "F_file_type"; operator = Equals;
                       expected = String_value "x" } ];
  } in
  assert_has_error (Input_fact_wrong_provenance (fid "F_bad"))
    "input fact wrong provenance rejected" (validate program)

let test_unknown_entry_origin () =
  let program = { (basic_program ()) with
    entry_origin = Some (oid "O_ghost");
    origin_sites = [ Anchor_origin anchor_origin_record ];
  } in
  assert_has_error (Unknown_entry_origin (oid "O_ghost"))
    "unknown entry origin rejected" (validate program)

let test_anchor_path_empty_component () =
  let action = action_with_inputs "O1" "C_notify" "D_notify_v1"
    [ { input_name = cin "ref";
        binding = Anchor_value (oid "O_anchor", ["a"; ""; "b"]) } ]
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action ];
  } in
  assert_has_error (Anchor_path_empty_component (oid "O_anchor", ["a"; ""; "b"]))
    "anchor path empty component rejected" (validate program)

let test_anchor_origin_not_anchor () =
  let action1 = action_origin_record "O1" "C_notify" "D_notify_v1" in
  let action2 = action_with_inputs "O2" "C_save" "D_save_v1"
    [ { input_name = cin "ref";
        binding = Anchor_value (oid "O1", ["a"]) } ]
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record;
                     Action_origin action1; Action_origin action2 ];
    entry_origin = Some (oid "O1");
    success_continuations = [
      { from_origin = oid "O1"; target = Origin_target (oid "O2") };
      { from_origin = oid "O2"; target = Program_complete };
    ];
    capability_contracts = [ cap_contract "C_notify" "D_notify_v1";
                             cap_contract "C_save" "D_save_v1" ];
  } in
  assert_has_error (Anchor_origin_not_anchor (oid "O1"))
    "anchor origin not anchor rejected" (validate program)

let test_together_self_member () =
  let action = action_origin_record "O1" "C_notify" "D_notify_v1" in
  let together =
    { together_origin_id = oid "T1";
      group_id = gid "G_self";
      member_origin_ids = [ oid "T1"; oid "O1" ];
      objective = All_members_succeed;
    }
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record;
                     Action_origin action;
                     Together_origin together ];
    entry_origin = Some (oid "T1");
    success_continuations = [ { from_origin = oid "T1"; target = Program_complete } ];
  } in
  assert_has_error (Together_self_member (gid "G_self"))
    "together self member rejected" (validate program)

let test_branch_subject_missing () =
  let branch =
    { branch_id = bid "B_nosubject";
      branch_subject = oid "O_nonexistent";
      outcome_branches = [ (Success, Stop) ];
    }
  in
  let program = { (basic_program ()) with
    branches = [ branch ];
  } in
  assert_has_error (Branch_subject_missing (bid "B_nosubject"))
    "branch subject missing rejected" (validate program)

let test_role_scope_missing_item_template () =
  let role =
    { role_id = rid "R_tmpl_scoped";
      scope = Item_template_scope (tid "IT_nonexistent");
      fact_contract = Role_fact_contract [];
      eligible_fulfillment = role_fulfillment_of_string "opaque";
    }
  in
  let program = { (basic_program ()) with
    roles = [ role ];
  } in
  assert_has_error (Role_scope_missing_item_template (rid "R_tmpl_scoped"))
    "role scope missing item template rejected" (validate program)

let test_role_fact_contract_invalid_fact () =
  let role =
    { role_id = rid "R_badfact";
      scope = Program_scope;
      fact_contract = Role_fact_contract [ fid "F_nonexistent" ];
      eligible_fulfillment = role_fulfillment_of_string "opaque";
    }
  in
  let program = { (basic_program ()) with
    roles = [ role ];
  } in
  assert_has_error (Role_fact_contract_invalid_fact (rid "R_badfact", fid "F_nonexistent"))
    "role fact contract invalid fact rejected" (validate program)

let test_deadline_empty () =
  let action = { (action_origin_record "O1" "C_notify" "D_notify_v1") with
    execution_constraints = [ Deadline "" ];
  } in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action ];
  } in
  assert_has_error (Deadline_empty (oid "O1"))
    "deadline empty rejected" (validate program)

let test_duplicate_capability_contract () =
  let program = { (basic_program ()) with
    capability_contracts = [ cap_contract "C_notify" "D_notify_v1";
                             cap_contract "C_notify" "D_notify_v1" ];
  } in
  assert_has_error (Duplicate_capability_contract (cid "C_notify"))
    "duplicate capability contract rejected" (validate program)

let test_duplicate_group_id () =
  let action1 = action_origin_record "O1" "C_notify" "D_notify_v1" in
  let action2 = action_origin_record "O2" "C_save" "D_save_v1" in
  let action3 = action_origin_record "O3" "C_log" "D_log_v1" in
  let together1 =
    { together_origin_id = oid "T1";
      group_id = gid "G_dup";
      member_origin_ids = [ oid "O1"; oid "O2" ];
      objective = All_members_succeed;
    }
  in
  let together2 =
    { together_origin_id = oid "T2";
      group_id = gid "G_dup";
      member_origin_ids = [ oid "O2"; oid "O3" ];
      objective = All_members_succeed;
    }
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record;
                     Action_origin action1; Action_origin action2;
                     Action_origin action3;
                     Together_origin together1;
                     Together_origin together2 ];
    entry_origin = Some (oid "T1");
    success_continuations = [ { from_origin = oid "T1"; target = Program_complete };
                              { from_origin = oid "T2"; target = Program_complete } ];
    capability_contracts = [ cap_contract "C_notify" "D_notify_v1";
                             cap_contract "C_save" "D_save_v1";
                             cap_contract "C_log" "D_log_v1" ];
  } in
  assert_has_error (Duplicate_group_id (gid "G_dup"))
    "duplicate group id rejected" (validate program)

let test_duplicate_batch_id () =
  let batch1 =
    { batch_id = btid "BAT_dup";
      collection_provenance = batch_collection_provenance_of_string "opaque";
      item_template_id = tid "IT1";
      traversal_policy = batch_traversal_policy_of_string "opaque";
      composite_objective = batch_objective_of_string "opaque";
      aggregate_facts = [];
    }
  in
  let batch2 =
    { batch_id = btid "BAT_dup";
      collection_provenance = batch_collection_provenance_of_string "opaque2";
      item_template_id = tid "IT1";
      traversal_policy = batch_traversal_policy_of_string "opaque2";
      composite_objective = batch_objective_of_string "opaque2";
      aggregate_facts = [];
    }
  in
  let item =
    { item_template_id = tid "IT1";
      origin_sites = [];
      branches = [];
      roles = [];
      objective = Required_role (rid "R_nonexistent");
    }
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record;
                     Batch_site batch1; Batch_site batch2 ];
    item_templates = [ item ];
    entry_origin = None;
  } in
  assert_has_error (Duplicate_batch_id (btid "BAT_dup"))
    "duplicate batch id rejected" (validate program)

let test_duplicate_item_template_id () =
  let item1 =
    { item_template_id = tid "IT_dup";
      origin_sites = [];
      branches = [];
      roles = [];
      objective = Required_role (rid "R_nonexistent");
    }
  in
  let item2 =
    { item_template_id = tid "IT_dup";
      origin_sites = [];
      branches = [];
      roles = [];
      objective = Required_role (rid "R_nonexistent");
    }
  in
  let program = { (basic_program ()) with
    item_templates = [ item1; item2 ];
  } in
  assert_has_error (Duplicate_item_template_id (tid "IT_dup"))
    "duplicate item template id rejected" (validate program)

let test_duplicate_role_id () =
  let role1 =
    { role_id = rid "R_dup";
      scope = Program_scope;
      fact_contract = Role_fact_contract [];
      eligible_fulfillment = role_fulfillment_of_string "opaque";
    }
  in
  let role2 =
    { role_id = rid "R_dup";
      scope = Program_scope;
      fact_contract = Role_fact_contract [];
      eligible_fulfillment = role_fulfillment_of_string "opaque";
    }
  in
  let program = { (basic_program ()) with
    roles = [ role1; role2 ];
  } in
  assert_has_error (Duplicate_role_id (rid "R_dup"))
    "duplicate role id rejected" (validate program)

let test_duplicate_branch_id () =
  let action = action_origin_record "O1" "C_notify" "D_notify_v1" in
  let branch1 =
    { branch_id = bid "B_dup";
      branch_subject = oid "O1";
      outcome_branches = [ (Success, Stop) ];
    }
  in
  let branch2 =
    { branch_id = bid "B_dup";
      branch_subject = oid "O1";
      outcome_branches = [ (Failure, Stop) ];
    }
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action ];
    branches = [ branch1; branch2 ];
  } in
  assert_has_error (Duplicate_branch_id (bid "B_dup"))
    "duplicate branch id rejected" (validate program)

(* ------------------------------------------------------------------ *)
(*  CORE-3A tests                                                       *)
(* ------------------------------------------------------------------ *)

(* 3A-1: Ordinary Origin Fact does not self-cycle *)
let test_fact_origin_no_self_cycle () =
  let declared_fact =
    { fact_id = fid "F_result";
      schema_description = "";
      provenance = Origin_provenance (oid "O1");
    }
  in
  let action = { (action_origin_record "O1" "C_notify" "D_notify_v1") with
    declared_facts = [ declared_fact ];
    inputs = [];
  } in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action ];
  } in
  assert_ok "ordinary origin fact no self-cycle" (validate program)

(* 3A-2: Real Fact dependency *)
let test_real_fact_dependency () =
  let declared_f1 =
    { fact_id = fid "F_a";
      schema_description = "";
      provenance = Origin_provenance (oid "O1");
    }
  in
  let declared_f2 =
    { fact_id = fid "F_b";
      schema_description = "";
      provenance = Origin_provenance (oid "O2");
    }
  in
  let action1 = { (action_origin_record "O1" "C_notify" "D_notify_v1") with
    declared_facts = [ declared_f1 ];
    inputs = [];
  } in
  let action2 = { (action_origin_record "O2" "C_save" "D_save_v1") with
    declared_facts = [ declared_f2 ];
    inputs = [ { input_name = cin "v";
                 binding = Fact_from_origin (fid "F_a", oid "O1") } ];
  } in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record;
                     Action_origin action1; Action_origin action2 ];
    entry_origin = Some (oid "O1");
    success_continuations = [
      { from_origin = oid "O1"; target = Origin_target (oid "O2") };
      { from_origin = oid "O2"; target = Program_complete };
    ];
    capability_contracts = [ cap_contract "C_notify" "D_notify_v1";
                             cap_contract "C_save" "D_save_v1" ];
  } in
  assert_ok "real fact dependency validates" (validate program)

(* 3A-3: Real Fact dependency cycle *)
let test_fact_dependency_cycle () =
  let declared_f1 =
    { fact_id = fid "F_cycle_a";
      schema_description = "";
      provenance = Origin_provenance (oid "O1");
    }
  in
  let declared_f2 =
    { fact_id = fid "F_cycle_b";
      schema_description = "";
      provenance = Origin_provenance (oid "O2");
    }
  in
  let action1 = { (action_origin_record "O1" "C_notify" "D_notify_v1") with
    declared_facts = [ declared_f1 ];
    inputs = [ { input_name = cin "v";
                 binding = Fact_from_origin (fid "F_cycle_b", oid "O2") } ];
  } in
  let action2 = { (action_origin_record "O2" "C_save" "D_save_v1") with
    declared_facts = [ declared_f2 ];
    inputs = [ { input_name = cin "v";
                 binding = Fact_from_origin (fid "F_cycle_a", oid "O1") } ];
  } in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record;
                     Action_origin action1; Action_origin action2 ];
    entry_origin = Some (oid "O1");
    success_continuations = [
      { from_origin = oid "O1"; target = Origin_target (oid "O2") };
      { from_origin = oid "O2"; target = Program_complete };
    ];
    capability_contracts = [ cap_contract "C_notify" "D_notify_v1";
                             cap_contract "C_save" "D_save_v1" ];
  } in
  assert_error "fact dependency cycle rejected" (validate program)

(* 3A-4: Global program/template Origin collision *)
let test_global_origin_collision () =
  let action_prog = action_origin_record "O_collide" "C_notify" "D_notify_v1" in
  let action_item = action_origin_record "O_collide" "C_save" "D_save_v1" in
  let item =
    { item_template_id = tid "IT_collide";
      origin_sites = [ Action_origin action_item ];
      branches = [];
      roles = [];
      objective = Required_role (rid "R_nonexistent");
    }
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action_prog ];
    item_templates = [ item ];
    capability_contracts = [ cap_contract "C_notify" "D_notify_v1" ];
  } in
  assert_has_error (Duplicate_origin_id (oid "O_collide"))
    "global origin collision rejected" (validate program)

(* 3A-5: Missing Fact_from_origin Fact *)
let test_missing_fact_from_origin () =
  let declared_f =
    { fact_id = fid "F_exists";
      schema_description = "";
      provenance = Origin_provenance (oid "O2");
    }
  in
  let action1 = { (action_origin_record "O1" "C_notify" "D_notify_v1") with
    inputs = [ { input_name = cin "v";
                 binding = Fact_from_origin (fid "F_missing", oid "O1") } ];
  } in
  let action2 = { (action_origin_record "O2" "C_save" "D_save_v1") with
    declared_facts = [ declared_f ];
  } in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record;
                     Action_origin action1; Action_origin action2 ];
    entry_origin = Some (oid "O1");
    success_continuations = [
      { from_origin = oid "O1"; target = Origin_target (oid "O2") };
      { from_origin = oid "O2"; target = Program_complete };
    ];
    capability_contracts = [ cap_contract "C_notify" "D_notify_v1";
                             cap_contract "C_save" "D_save_v1" ];
  } in
  assert_has_error (Missing_fact (fid "F_missing"))
    "missing fact from origin rejected" (validate program)

(* 3A-6: Missing Fact_through_role Fact *)
let test_missing_fact_through_role () =
  let role =
    { role_id = rid "R_ex";
      scope = Program_scope;
      fact_contract = Role_fact_contract [ fid "F_exists" ];
      eligible_fulfillment = role_fulfillment_of_string "opaque";
    }
  in
  let action = action_with_inputs "O1" "C_notify" "D_notify_v1"
    [ { input_name = cin "v";
        binding = Fact_through_role (fid "F_missing_st", rid "R_ex") } ]
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action ];
    roles = [ role ];
  } in
  assert_has_error (Missing_fact (fid "F_missing_st"))
    "missing fact through role rejected" (validate program)

(* 3A-7: Program cannot steal Item Role *)
let test_program_cannot_use_item_role () =
  let item_role =
    { role_id = rid "R_item";
      scope = Item_template_scope (tid "IT_scope");
      fact_contract = Role_fact_contract [ fid "F_file_type" ];
      eligible_fulfillment = role_fulfillment_of_string "opaque";
    }
  in
  let item =
    { item_template_id = tid "IT_scope";
      origin_sites = [];
      branches = [];
      roles = [ item_role ];
      objective = Required_role (rid "R_item");
    }
  in
  let action = action_with_inputs "O1" "C_notify" "D_notify_v1"
    [ { input_name = cin "v";
        binding = Fact_through_role (fid "F_file_type", rid "R_item") } ]
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record; Action_origin action ];
    item_templates = [ item ];
  } in
  assert_has_error (Missing_role (rid "R_item"))
    "program cannot use item role rejected" (validate program)

(* 3A-8: Template isolation - cross-template Role reference *)
let test_cross_template_role_isolation () =
  let role_t2 =
    { role_id = rid "R_t2";
      scope = Item_template_scope (tid "IT_B");
      fact_contract = Role_fact_contract [ fid "F_file_type" ];
      eligible_fulfillment = role_fulfillment_of_string "opaque";
    }
  in
  let item2 =
    { item_template_id = tid "IT_B";
      origin_sites = [];
      branches = [];
      roles = [ role_t2 ];
      objective = Required_role (rid "R_t2");
    }
  in
  let action = action_with_inputs "O_item_a" "C_notify" "D_notify_v1"
    [ { input_name = cin "v";
        binding = Fact_through_role (fid "F_file_type", rid "R_t2") } ]
  in
  let dummy_role =
    { role_id = rid "R_dummy_a";
      scope = Item_template_scope (tid "IT_A");
      fact_contract = Role_fact_contract [];
      eligible_fulfillment = role_fulfillment_of_string "opaque";
    }
  in
  let item1 =
    { item_template_id = tid "IT_A";
      origin_sites = [ Action_origin action ];
      branches = [];
      roles = [ dummy_role ];
      objective = Required_role (rid "R_dummy_a");
    }
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record ];
    item_templates = [ item1; item2 ];
    entry_origin = None;
    success_continuations = [];
  } in
  assert_has_error (Missing_role (rid "R_t2"))
    "cross-template role isolation rejected" (validate program)

(* 3A-9: Correct same-template Role *)
let test_same_template_role_valid () =
  let role_t =
    { role_id = rid "R_own";
      scope = Item_template_scope (tid "IT_own");
      fact_contract = Role_fact_contract [];
      eligible_fulfillment = role_fulfillment_of_string "opaque";
    }
  in
  let action = action_origin_record "O_item" "C_notify" "D_notify_v1" in
  let item =
    { item_template_id = tid "IT_own";
      origin_sites = [ Action_origin action ];
      branches = [];
      roles = [ role_t ];
      objective = Required_role (rid "R_own");
    }
  in
  let program = { (basic_program ()) with
    origin_sites = [ Anchor_origin anchor_origin_record ];
    item_templates = [ item ];
    entry_origin = None;
    success_continuations = [];
  } in
  assert_ok "same template role valid" (validate program)

(* ------------------------------------------------------------------ *)
(*  Run all tests                                                      *)
(* ------------------------------------------------------------------ *)

let () =
  test_valid_program ();
  test_duplicate_origin ();
  test_missing_entry_origin_for_actions ();
  test_duplicate_success_continuation ();
  test_self_cycle ();
  test_multi_node_cycle ();
  test_missing_capability_contract ();
  test_contract_digest_mismatch ();
  test_duplicate_input_fact ();
  test_guard_unknown_fact ();
  test_bad_anchor_origin ();
  test_anchor_path_empty ();
  test_fact_origin_missing ();
  test_fact_from_origin_provenance_mismatch ();
  test_role_missing ();
  test_role_fact_contract_mismatch ();
  test_branch_duplicate_outcome ();
  test_branch_missing_target ();
  test_together_one_member ();
  test_together_duplicate_member ();
  test_together_unknown_member ();
  test_item_objective_missing_role ();
  test_batch_missing_item_template ();
  test_determinism ();
  test_integration_lower_validate ();
  test_input_fact_wrong_provenance ();
  test_unknown_entry_origin ();
  test_anchor_path_empty_component ();
  test_anchor_origin_not_anchor ();
  test_together_self_member ();
  test_branch_subject_missing ();
  test_role_scope_missing_item_template ();
  test_role_fact_contract_invalid_fact ();
  test_deadline_empty ();
  test_duplicate_capability_contract ();
  test_duplicate_group_id ();
  test_duplicate_batch_id ();
  test_duplicate_item_template_id ();
  test_duplicate_role_id ();
  test_duplicate_branch_id ();
  test_fact_origin_no_self_cycle ();
  test_real_fact_dependency ();
  test_fact_dependency_cycle ();
  test_global_origin_collision ();
  test_missing_fact_from_origin ();
  test_missing_fact_through_role ();
  test_program_cannot_use_item_role ();
  test_cross_template_role_isolation ();
  test_same_template_role_valid ();
  Printf.printf "PASS all validator tests (%d/%d)\n" !tests_passed !tests_run
