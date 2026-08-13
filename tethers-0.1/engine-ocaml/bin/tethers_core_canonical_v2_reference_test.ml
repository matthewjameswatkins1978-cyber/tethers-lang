(* ==================================================================
   CANONICAL FORMAT V2 — REFERENCE ORACLE TESTS (authoritative)

   Every failed expectation MUST make the dune test fail.
   No execution path may print FAIL and return exit code 0.
   ================================================================== *)

open Tethers_core

(* ================================================================== *)
(*  Assertion helpers — failwith on failure                              *)
(* ================================================================== *)

let check condition message =
  if not condition then failwith message

let check_equal_int expected actual label =
  check (expected = actual)
    (Printf.sprintf "FAIL: %s expected=%d actual=%d" label expected actual)

let check_equal_string expected actual label =
  check (expected = actual)
    (Printf.sprintf "FAIL: %s expected=%S actual=%S" label expected actual)

let check_ok result label =
  match result with
  | Ok v -> v
  | Error _ -> failwith ("FAIL: expected Ok but got Error for " ^ label)

let check_error result label =
  match result with
  | Error e -> e
  | Ok _ -> failwith ("FAIL: expected Error but got Ok for " ^ label)

(* ================================================================== *)
(*  Constructor helpers                                                *)
(* ================================================================== *)

let pid s = program_id_of_string s
let oid s = origin_id_of_string s
let fid s = fact_id_of_string s
let rid s = role_id_of_string s
let cid s = capability_id_of_string s
let gid s = group_id_of_string s
let tid s = item_template_id_of_string s
let cv s = core_version_of_string s
let hsk s = host_snapshot_key_of_string s
let ccd s = capability_contract_digest_of_string s
let rf s = role_fulfillment_of_string s

(* ================================================================== *)
(*  Basic domain and format tests                                      *)
(* ================================================================== *)

let test_domain_v2 () =
  let expected = "TETHERS_CORE_CANON_V2\x00" in
  let actual = Bytes.to_string Tethers_core_canonical_v2_reference.domain_v2 in
  check_equal_string expected actual "DOMAIN_V2"

let test_digest_string_format () =
  let hex = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" in
  let expected = "tethers:v2:sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" in
  let actual = Tethers_core_canonical_v2_reference.digest_string_v2 hex in
  check_equal_string expected actual "digest_string_v2"

(* ================================================================== *)
(*  Empty program                                                       *)
(* ================================================================== *)

let test_empty_program () =
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [];
    entry_guards = [];
    entry_origin = None;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [];
    item_templates = [];
    capability_contracts = [];
  } in
  let result = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p) "empty_program" in
  check_equal_int 1 result.candidate_count "empty_program candidate_count"

(* ================================================================== *)
(*  Single anchor + action                                              *)
(* ================================================================== *)

let test_single_anchor_action () =
  let anchor_id = oid "anchor1" in
  let action_id = oid "action1" in
  let fact_id = fid "fact1" in
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [{
      fact_id = fact_id;
      schema_description = "test fact";
      provenance = Evaluation_input (hsk "hk1", String_type);
    }];
    entry_guards = [];
    entry_origin = Some anchor_id;
    success_continuations = [];
    origin_sites = [
      Anchor_origin {
        anchor_origin_id = anchor_id;
        event_name = "test_event";
        declared_facts = [];
      };
      Action_origin {
        action_origin_id = action_id;
        capability_id = cid "cap.test";
        contract_digest = ccd "sha256:abc123";
        inputs = [];
        declared_facts = [];
        execution_constraints = [];
      };
    ];
    branches = [];
    roles = [];
    item_templates = [];
    capability_contracts = [{
      capability_id = cid "cap.test";
      contract_digest = ccd "sha256:abc123";
      schema_description = "test cap";
    }];
  } in
  let result = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p) "single_anchor_action" in
  check (result.candidate_count >= 1) "single_anchor_action candidate_count >= 1"

(* ================================================================== *)
(*  Neutrality: program_id                                              *)
(* ================================================================== *)

let test_neutrality_program_id () =
  let p1 = {
    program_id = pid "id1";
    core_version = cv "0.1.0";
    input_facts = [];
    entry_guards = [];
    entry_origin = None;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [];
    item_templates = [];
    capability_contracts = [];
  } in
  let p2 = { p1 with program_id = pid "id2" } in
  let r1 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p1) "neutrality_program_id_1" in
  let r2 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p2) "neutrality_program_id_2" in
  check_equal_string r1.payload r2.payload "program_id neutrality payload";
  check_equal_string r1.digest_string r2.digest_string "program_id neutrality digest"

(* ================================================================== *)
(*  Neutrality: schema_description                                      *)
(* ================================================================== *)

let test_neutrality_schema_description () =
  let fact1 = {
    fact_id = fid "f1";
    schema_description = "desc1";
    provenance = Evaluation_input (hsk "hk1", String_type);
  } in
  let fact2 = {
    fact_id = fid "f1";
    schema_description = "desc2";
    provenance = Evaluation_input (hsk "hk1", String_type);
  } in
  let make_p fact = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [fact];
    entry_guards = [];
    entry_origin = None;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [];
    item_templates = [];
    capability_contracts = [];
  } in
  let r1 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle (make_p fact1)) "neutrality_schema_1" in
  let r2 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle (make_p fact2)) "neutrality_schema_2" in
  check_equal_string r1.payload r2.payload "schema_description neutrality payload";
  check_equal_string r1.digest_string r2.digest_string "schema_description neutrality digest"

(* ================================================================== *)
(*  Raw-ID rename invariance                                            *)
(* ================================================================== *)

let test_raw_id_rename () =
  let p1 = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [{
      fact_id = fid "banana_thing_947";
      schema_description = "";
      provenance = Evaluation_input (hsk "hk1", String_type);
    }];
    entry_guards = [];
    entry_origin = Some (oid "banana_thing_947");
    success_continuations = [];
    origin_sites = [
      Anchor_origin {
        anchor_origin_id = oid "banana_thing_947";
        event_name = "ev";
        declared_facts = [];
      };
    ];
    branches = [];
    roles = [];
    item_templates = [];
    capability_contracts = [];
  } in
  let p2 = {
    p1 with
    input_facts = [{
      fact_id = fid "O_anchor";
      schema_description = "";
      provenance = Evaluation_input (hsk "hk1", String_type);
    }];
    entry_origin = Some (oid "O_anchor");
    origin_sites = [
      Anchor_origin {
        anchor_origin_id = oid "O_anchor";
        event_name = "ev";
        declared_facts = [];
      };
    ];
  } in
  let r1 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p1) "raw_id_rename_1" in
  let r2 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p2) "raw_id_rename_2" in
  check_equal_string r1.payload r2.payload "raw-ID rename invariance payload";
  check_equal_string r1.digest_string r2.digest_string "raw-ID rename invariance digest"

(* ================================================================== *)
(*  Multiplicity 1 vs 2                                                *)
(* ================================================================== *)

let test_multiplicity_1_vs_2 () =
  let make_p n =
    let origins = List.init n (fun i ->
      Action_origin {
        action_origin_id = oid ("action" ^ string_of_int i);
        capability_id = cid "cap.x";
        contract_digest = ccd "sha256:abc";
        inputs = [];
        declared_facts = [];
        execution_constraints = [];
      }
    ) in
    {
      program_id = pid "test";
      core_version = cv "0.1.0";
      input_facts = [];
      entry_guards = [];
      entry_origin = Some (oid "anchor");
      success_continuations = [];
      origin_sites =
        Anchor_origin {
          anchor_origin_id = oid "anchor";
          event_name = "ev";
          declared_facts = [];
        } :: origins;
      branches = [];
      roles = [];
      item_templates = [];
      capability_contracts = [{
        capability_id = cid "cap.x";
        contract_digest = ccd "sha256:abc";
        schema_description = "";
      }];
    }
  in
  let r1 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle (make_p 1)) "mult_1" in
  let r2 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle (make_p 2)) "mult_2" in
  check (r1.digest_string <> r2.digest_string) "multiplicity 1 vs 2 different digest"

(* ================================================================== *)
(*  Together member order invariant                                     *)
(* ================================================================== *)

let test_together_member_order () =
  let a1_id = oid "a1" in
  let a2_id = oid "a2" in
  let make_p order =
    let origins = match order with
    | `AB -> [
        Anchor_origin { anchor_origin_id = oid "anchor"; event_name = "ev"; declared_facts = [] };
        Action_origin {
          action_origin_id = a1_id;
          capability_id = cid "cap.x";
          contract_digest = ccd "sha256:abc";
          inputs = [];
          declared_facts = [];
          execution_constraints = [];
        };
        Action_origin {
          action_origin_id = a2_id;
          capability_id = cid "cap.x";
          contract_digest = ccd "sha256:abc";
          inputs = [];
          declared_facts = [];
          execution_constraints = [];
        };
        Together_origin {
          together_origin_id = oid "tog";
          group_id = gid "g1";
          member_origin_ids = [a1_id; a2_id];
          objective = All_members_succeed;
        };
      ]
    | `BA -> [
        Anchor_origin { anchor_origin_id = oid "anchor"; event_name = "ev"; declared_facts = [] };
        Action_origin {
          action_origin_id = a1_id;
          capability_id = cid "cap.x";
          contract_digest = ccd "sha256:abc";
          inputs = [];
          declared_facts = [];
          execution_constraints = [];
        };
        Action_origin {
          action_origin_id = a2_id;
          capability_id = cid "cap.x";
          contract_digest = ccd "sha256:abc";
          inputs = [];
          declared_facts = [];
          execution_constraints = [];
        };
        Together_origin {
          together_origin_id = oid "tog";
          group_id = gid "g1";
          member_origin_ids = [a2_id; a1_id];
          objective = All_members_succeed;
        };
      ]
    in
    {
      program_id = pid "test";
      core_version = cv "0.1.0";
      input_facts = [];
      entry_guards = [];
      entry_origin = Some (oid "anchor");
      success_continuations = [];
      origin_sites = origins;
      branches = [];
      roles = [];
      item_templates = [];
      capability_contracts = [{
        capability_id = cid "cap.x";
        contract_digest = ccd "sha256:abc";
        schema_description = "";
      }];
    }
  in
  let r1 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle (make_p `AB)) "together_ab" in
  let r2 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle (make_p `BA)) "together_ba" in
  check_equal_string r1.payload r2.payload "Together member order invariant payload";
  check_equal_string r1.digest_string r2.digest_string "Together member order invariant digest"

(* ================================================================== *)
(*  Role scope validation                                               *)
(* ================================================================== *)

let test_role_scope_validation () =
  let tid1 = tid "IT1" in
  let tid2 = tid "IT2" in
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [];
    entry_guards = [];
    entry_origin = None;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [];
    item_templates = [{
      item_template_id = tid1;
      origin_sites = [];
      branches = [];
      roles = [{
        role_id = rid "R1";
        scope = Item_template_scope tid1;
        fact_contract = Role_fact_contract [];
        eligible_fulfillment = rf "ok";
      }];
      objective = Required_role (rid "R1");
    }; {
      item_template_id = tid2;
      origin_sites = [];
      branches = [];
      roles = [{
        role_id = rid "R2";
        scope = Program_scope;
        fact_contract = Role_fact_contract [];
        eligible_fulfillment = rf "ok";
      }];
      objective = Required_role (rid "R2");
    }];
    capability_contracts = [];
  } in
  let errs = check_error (Tethers_core_validator.validate p) "role_scope_validation" in
  let has_mismatch = List.exists (fun e ->
    match e with
    | Tethers_core_validator.Role_scope_storage_mismatch _ -> true
    | _ -> false
  ) errs in
  check has_mismatch "role scope storage mismatch detected"

(* ================================================================== *)
(*  Role_fact_contract duplicate                                        *)
(* ================================================================== *)

let test_role_fact_contract_duplicate () =
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [{
      fact_id = fid "f1";
      schema_description = "";
      provenance = Evaluation_input (hsk "hk1", String_type);
    }];
    entry_guards = [];
    entry_origin = None;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [{
      role_id = rid "R1";
      scope = Program_scope;
      fact_contract = Role_fact_contract [fid "f1"; fid "f1"];
      eligible_fulfillment = rf "ok";
    }];
    item_templates = [];
    capability_contracts = [];
  } in
  let errs = check_error (Tethers_core_validator.validate p) "role_fact_contract_dup" in
  let has_dup = List.exists (fun e ->
    match e with
    | Tethers_core_validator.Role_fact_contract_duplicate_fact _ -> true
    | _ -> false
  ) errs in
  check has_dup "role_fact_contract duplicate detected"

(* ================================================================== *)
(*  Integer encoding boundaries                                         *)
(* ================================================================== *)

let test_integer_boundaries () =
  let test_encode_int n expected =
    let result = Tethers_core_canonical_v2_reference.encode_int n in
    check_equal_string expected result (Printf.sprintf "encode_int %d" n)
  in
  test_encode_int 0 "0;";
  test_encode_int 1 "1;";
  test_encode_int (-1) "-1;";
  test_encode_int 42 "42;";
  test_encode_int (-42) "-42;"

(* ================================================================== *)
(*  String encoding                                                     *)
(* ================================================================== *)

let test_string_encoding () =
  let test_encode_string s expected =
    let result = Tethers_core_canonical_v2_reference.encode_string s in
    check_equal_string expected result (Printf.sprintf "encode_string %S" s)
  in
  test_encode_string "" "0:";
  test_encode_string "hello" "5:hello";
  test_encode_string "a" "1:a"

(* ================================================================== *)
(*  Test A: Cross-family raw-ID collision                               *)
(*  OriginId "X", FactId "X", BranchId "X" are unrelated.             *)
(*  Renaming one family must not affect canonical bytes.                *)
(* ================================================================== *)

let test_cross_family_raw_id_collision () =
  let make_p ~anchor_name ~fact_name ~branch_name =
    let anchor_id = oid anchor_name in
    let fact_id_v = fid fact_name in
    let branch_id_v = branch_id_of_string branch_name in
    {
      program_id = pid "test";
      core_version = cv "0.1.0";
      input_facts = [{
        fact_id = fact_id_v;
        schema_description = "";
        provenance = Evaluation_input (hsk "hk1", String_type);
      }];
      entry_guards = [];
      entry_origin = Some anchor_id;
      success_continuations = [];
      origin_sites = [
        Anchor_origin {
          anchor_origin_id = anchor_id;
          event_name = "ev";
          declared_facts = [];
        };
      ];
      branches = [{
        branch_id = branch_id_v;
        branch_subject = anchor_id;
        outcome_branches = [(Success, Stop)];
      }];
      roles = [];
      item_templates = [];
      capability_contracts = [];
    }
  in
  (* All three families share raw string "X" *)
  let p1 = make_p ~anchor_name:"X" ~fact_name:"X" ~branch_name:"X" in
  (* Rename only the anchor *)
  let p2 = make_p ~anchor_name:"Y" ~fact_name:"X" ~branch_name:"X" in
  let r1 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p1) "cross_family_1" in
  let r2 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p2) "cross_family_2" in
  check_equal_string r1.payload r2.payload "cross-family raw-ID rename invariance payload";
  check_equal_string r1.digest_string r2.digest_string "cross-family raw-ID rename invariance digest"

(* ================================================================== *)
(*  Test B: Scoped same-raw role IDs across templates                   *)
(*  Template A has RoleId "R", Template B has RoleId "R".              *)
(*  Both valid. Prove separate role labels and correct resolution.     *)
(* ================================================================== *)

let test_scoped_same_raw_role_ids () =
  let tid_a = tid "TA" in
  let tid_b = tid "TB" in
  let r_id = rid "R" in
  let anchor_a = oid "anchorA" in
  let anchor_b = oid "anchorB" in
  let fact_a = fid "factA" in
  let fact_b = fid "factB" in
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [];
    entry_guards = [];
    entry_origin = Some anchor_a;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [];
    item_templates = [{
      item_template_id = tid_a;
      origin_sites = [
        Anchor_origin {
          anchor_origin_id = anchor_a;
          event_name = "evA";
          declared_facts = [{
            fact_id = fact_a;
            schema_description = "";
            provenance = Evaluation_input (hsk "hkA", String_type);
          }];
        };
      ];
      branches = [];
      roles = [{
        role_id = r_id;
        scope = Item_template_scope tid_a;
        fact_contract = Role_fact_contract [fact_a];
        eligible_fulfillment = rf "ok";
      }];
      objective = Required_role r_id;
    }; {
      item_template_id = tid_b;
      origin_sites = [
        Anchor_origin {
          anchor_origin_id = anchor_b;
          event_name = "evB";
          declared_facts = [{
            fact_id = fact_b;
            schema_description = "";
            provenance = Evaluation_input (hsk "hkB", String_type);
          }];
        };
      ];
      branches = [];
      roles = [{
        role_id = r_id;
        scope = Item_template_scope tid_b;
        fact_contract = Role_fact_contract [fact_b];
        eligible_fulfillment = rf "ok";
      }];
      objective = Required_role r_id;
    }];
    capability_contracts = [];
  } in
  let result = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p) "scoped_same_raw_role" in
  check (result.candidate_count >= 1) "scoped same-raw role has candidates"

(* ================================================================== *)
(*  Test C: Role-block enumeration count                                *)
(*  2 templates, 2 roles in each, 0 program roles.                     *)
(*  Template labelling factor: 2!                                       *)
(*  Role factor per template: 2! x 2!                                  *)
(*  Total: 2 x 2 x 2 = 8                                              *)
(* ================================================================== *)

let test_role_block_count () =
  let tid_a = tid "TA" in
  let tid_b = tid "TB" in
  let anchor = oid "anchor" in
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [];
    entry_guards = [];
    entry_origin = Some anchor;
    success_continuations = [];
    origin_sites = [
      Anchor_origin {
        anchor_origin_id = anchor;
        event_name = "ev";
        declared_facts = [];
      };
    ];
    branches = [];
    roles = [];
    item_templates = [{
      item_template_id = tid_a;
      origin_sites = [];
      branches = [];
      roles = [
        { role_id = rid "RA1"; scope = Item_template_scope tid_a;
          fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok" };
        { role_id = rid "RA2"; scope = Item_template_scope tid_a;
          fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok" };
      ];
      objective = Required_role (rid "RA1");
    }; {
      item_template_id = tid_b;
      origin_sites = [];
      branches = [];
      roles = [
        { role_id = rid "RB1"; scope = Item_template_scope tid_b;
          fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok" };
        { role_id = rid "RB2"; scope = Item_template_scope tid_b;
          fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok" };
      ];
      objective = Required_role (rid "RB1");
    }];
    capability_contracts = [];
  } in
  let result = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p) "role_block_count" in
  (* 2! template perms * 2! role perms for TA * 2! role perms for TB = 2*2*2 = 8 *)
  check_equal_int 8 result.candidate_count "role block count (2 templates x 2 roles each)"

(* ================================================================== *)
(*  Test D: Real template-local storage-order permutation invariance    *)
(*  Permutate all D/B collections inside an item_template.             *)
(*  Oracle payload and digest MUST remain byte-identical.              *)
(* ================================================================== *)

let test_nested_storage_order () =
  let anchor1 = oid "anc1" and anchor2 = oid "anc2" in
  let action1 = oid "act1" and action2 = oid "act2" in
  let br1 = branch_id_of_string "br1" and br2 = branch_id_of_string "br2" in
  let fact_a1 = fid "fa1" and fact_a2 = fid "fa2" in
  let r1 = rid "R1" and r2 = rid "R2" in
  let itid = tid "IT1" in

  let make_p ~origin_order ~branch_order ~role_order =
    (* Origin sites inside the template *)
    let anchor_origin1 = Anchor_origin {
      anchor_origin_id = anchor1; event_name = "ev1";
      declared_facts = [{
        fact_id = fact_a1; schema_description = "";
        provenance = Evaluation_input (hsk "hk_a1", String_type);
      }];
    } in
    let anchor_origin2 = Anchor_origin {
      anchor_origin_id = anchor2; event_name = "ev2";
      declared_facts = [{
        fact_id = fact_a2; schema_description = "";
        provenance = Evaluation_input (hsk "hk_a2", Integer_type);
      }];
    } in
    let action_origin1 = Action_origin {
      action_origin_id = action1; capability_id = cid "cap.x";
      contract_digest = ccd "sha256:abc";
      inputs = [];
      declared_facts = [];
      execution_constraints = [];
    } in
    let action_origin2 = Action_origin {
      action_origin_id = action2; capability_id = cid "cap.x";
      contract_digest = ccd "sha256:abc";
      inputs = [];
      declared_facts = [];
      execution_constraints = [];
    } in
    let template_origins = match origin_order with
      | `ABCD -> [anchor_origin1; action_origin1; anchor_origin2; action_origin2]
      | `DCBA -> [action_origin2; anchor_origin2; action_origin1; anchor_origin1]
      | `ABCD_B -> [anchor_origin1; action_origin1; anchor_origin2; action_origin2]
      | `DCBA_B -> [action_origin2; anchor_origin2; action_origin1; anchor_origin1]
    in
    (* Branches *)
    let make_branch bid succ =
      { branch_id = bid; branch_subject = anchor1;
        outcome_branches = [(Success, succ)]; }
    in
    let template_branches = match branch_order with
      | `ABCD -> [make_branch br1 Stop; make_branch br2 Stop]
      | `DCBA -> [make_branch br2 Stop; make_branch br1 Stop]
    in
    (* Roles with Role_fact_contract *)
    let fc_r1 = Role_fact_contract [] in
    let template_roles = match role_order with
      | `ABCD -> [
          { role_id = r1; scope = Item_template_scope itid;
            fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok" };
          { role_id = r2; scope = Item_template_scope itid;
            fact_contract = fc_r1; eligible_fulfillment = rf "ok" };
        ]
      | `DCBA -> [
          { role_id = r2; scope = Item_template_scope itid;
            fact_contract = fc_r1; eligible_fulfillment = rf "ok" };
          { role_id = r1; scope = Item_template_scope itid;
            fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok" };
        ]
    in
    let program_input_facts = [] in
    {
      program_id = pid "test";
      core_version = cv "0.1.0";
      input_facts = program_input_facts;
      entry_guards = [];
      entry_origin = Some anchor1;
      success_continuations = [];
      origin_sites = [];
      branches = [];
      roles = [];
      item_templates = [{
        item_template_id = itid;
        origin_sites = template_origins;
        branches = template_branches;
        roles = template_roles;
        objective = Required_role r1;
      }];
      capability_contracts = [{
        capability_id = cid "cap.x";
        contract_digest = ccd "sha256:abc";
        schema_description = "";
      }];
    }
  in
  (* P1: canonical storage order *)
  let p1 = make_p ~origin_order:`ABCD ~branch_order:`ABCD ~role_order:`ABCD in
  (* P2: reversed/permutated storage order *)
  let p2 = make_p ~origin_order:`DCBA ~branch_order:`DCBA ~role_order:`DCBA in
  let r1 = match Tethers_core_canonical_v2_reference.slow_oracle p1 with
    | Ok r -> r
    | Error (Tethers_core_canonical_v2_reference.Invalid_core errs) ->
        failwith (Printf.sprintf "FAIL: template_perm_1 validation: %d errors: %s"
          (List.length errs)
          (String.concat "; " (List.map (fun _ -> "err") errs)))
    | Error Tethers_core_canonical_v2_reference.Oracle_too_large ->
        failwith "FAIL: template_perm_1 oracle_too_large"
  in
  let r2 = match Tethers_core_canonical_v2_reference.slow_oracle p2 with
    | Ok r -> r
    | Error _ -> failwith "FAIL: template_perm_2 error"
  in
  check_equal_int r1.candidate_count r2.candidate_count "template storage perm candidate count";
  check_equal_string r1.payload r2.payload "template storage order permutation invariance payload";
  check_equal_string r1.digest_string r2.digest_string "template storage order permutation invariance digest"

(* ================================================================== *)
(*  Test D2: Extended template-local permutation with Batch_site        *)
(* ================================================================== *)

let test_nested_storage_order_with_batch () =
  let anchor1 = oid "anc1" and anchor2 = oid "anc2" in
  let br1 = branch_id_of_string "br1" and br2 = branch_id_of_string "br2" in
  let fact_a1 = fid "fa1" and fact_a2 = fid "fa2" in
  let r1 = rid "R1" and r2 = rid "R2" in
  let itid = tid "IT1" in
  let itid_batch = tid "IT_batch" in
  let batch_id = batch_id_of_string "batch1" in
  let batch_fact = fid "bf1" in

  let make_p ~origin_order ~branch_order ~role_order =
    let anchor_origin1 = Anchor_origin {
      anchor_origin_id = anchor1; event_name = "ev1";
      declared_facts = [{
        fact_id = fact_a1; schema_description = "";
        provenance = Evaluation_input (hsk "hk_a1", String_type);
      }];
    } in
    let anchor_origin2 = Anchor_origin {
      anchor_origin_id = anchor2; event_name = "ev2";
      declared_facts = [{
        fact_id = fact_a2; schema_description = "";
        provenance = Evaluation_input (hsk "hk_a2", Integer_type);
      }];
    } in
    let batch_site = Batch_site {
      batch_id = batch_id;
      collection_provenance = batch_collection_provenance_of_string "prov";
      item_template_id = itid_batch;
      traversal_policy = batch_traversal_policy_of_string "seq";
      composite_objective = batch_objective_of_string "all";
      aggregate_facts = [{
        fact_id = batch_fact; schema_description = "";
        provenance = Evaluation_input (hsk "hk_bf", Boolean_type);
      }];
    } in
    let template_origins = match origin_order with
      | `ABCD -> [anchor_origin1; anchor_origin2; batch_site]
      | `CBA -> [batch_site; anchor_origin2; anchor_origin1]
    in
    let make_branch bid = {
      branch_id = bid; branch_subject = anchor1;
      outcome_branches = [(Success, Stop)];
    } in
    let template_branches = match branch_order with
      | `ABCD -> [make_branch br1; make_branch br2]
      | `CBA -> [make_branch br2; make_branch br1]
    in
    let template_roles = match role_order with
      | `ABCD -> [
          { role_id = r1; scope = Item_template_scope itid;
            fact_contract = Role_fact_contract [fact_a1]; eligible_fulfillment = rf "ok" };
          { role_id = r2; scope = Item_template_scope itid;
            fact_contract = Role_fact_contract [fact_a2]; eligible_fulfillment = rf "ok" };
        ]
      | `CBA -> [
          { role_id = r2; scope = Item_template_scope itid;
            fact_contract = Role_fact_contract [fact_a2]; eligible_fulfillment = rf "ok" };
          { role_id = r1; scope = Item_template_scope itid;
            fact_contract = Role_fact_contract [fact_a1]; eligible_fulfillment = rf "ok" };
        ]
    in
    {
      program_id = pid "test";
      core_version = cv "0.1.0";
      input_facts = [];
      entry_guards = [];
      entry_origin = Some anchor1;
      success_continuations = [];
      origin_sites = [];
      branches = [];
      roles = [];
      item_templates = [{
        item_template_id = itid;
        origin_sites = template_origins;
        branches = template_branches;
        roles = template_roles;
        objective = Required_role r1;
      }; {
        item_template_id = itid_batch;
        origin_sites = [];
        branches = [];
        roles = [{
          role_id = rid "R_batch";
          scope = Item_template_scope itid_batch;
          fact_contract = Role_fact_contract [];
          eligible_fulfillment = rf "ok";
        }];
        objective = Required_role (rid "R_batch");
      }];
      capability_contracts = [];
    }
  in
  let p1 = make_p ~origin_order:`ABCD ~branch_order:`ABCD ~role_order:`ABCD in
  let p2 = make_p ~origin_order:`CBA ~branch_order:`CBA ~role_order:`CBA in
  let r1 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p1) "template_perm_batch_1" in
  let r2 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p2) "template_perm_batch_2" in
  check_equal_int r1.candidate_count r2.candidate_count "template perm with batch candidate count";
  check_equal_string r1.payload r2.payload "template perm with batch payload";
  check_equal_string r1.digest_string r2.digest_string "template perm with batch digest"

(* ================================================================== *)
(*  Test E: Real 24-permutation Persistent Branch Test (C-B3T)         *)
(*  4 equivalent Origins + 4 equivalent Branches                       *)
(*  For each of 24 raw-ID storage permutations:                        *)
(*    run slow_oracle, assert candidate_count = 576                    *)
(*  Collect payloads and digests.                                      *)
(*  Assert: 24 tested permutations, 1 unique payload, 1 unique digest  *)
(* ================================================================== *)

let test_persistent_branch_gold () =
  let make_p origin_names branch_names =
    let origins = List.map (fun name ->
      Anchor_origin {
        anchor_origin_id = oid name;
        event_name = "ev";
        declared_facts = [];
      }
    ) origin_names in
    let branches = List.map2 (fun bname oname ->
      {
        branch_id = branch_id_of_string bname;
        branch_subject = oid oname;
        outcome_branches = [(Success, Stop)];
      }
    ) branch_names origin_names in
    {
      program_id = pid "test";
      core_version = cv "0.1.0";
      input_facts = [];
      entry_guards = [];
      entry_origin = Some (oid (List.hd origin_names));
      success_continuations = [];
      origin_sites = origins;
      branches = branches;
      roles = [];
      item_templates = [];
      capability_contracts = [];
    }
  in
  (* 24 permutations of 4 elements *)
  let perms = Tethers_core_canonical_v2_reference.perm [0;1;2;3] in
  let names = ["a0";"a1";"a2";"a3"] in
  let branch_names = ["b0";"b1";"b2";"b3"] in
  let map_names perm = List.map (fun i -> List.nth names i) perm in
  let map_branches perm = List.map (fun i -> List.nth branch_names i) perm in
  let run_one (perm : int list) : Tethers_core_canonical_v2_reference.oracle_result =
    let onames = map_names perm in
    let bnames = map_branches perm in
    let p = make_p onames bnames in
    check_ok (Tethers_core_canonical_v2_reference.slow_oracle p)
      (Printf.sprintf "persistent_branch_perm_%s" (String.concat "," (List.map string_of_int perm)))
  in
  let results = List.map run_one perms in
  (* Assert every run has candidate_count = 576 *)
  List.iteri (fun i (r : Tethers_core_canonical_v2_reference.oracle_result) ->
    check_equal_int 576 r.candidate_count
      (Printf.sprintf "persistent branch perm %d candidate_count" i)
  ) results;
  (* Collect unique payloads and digests *)
  let payloads = List.map (fun (r : Tethers_core_canonical_v2_reference.oracle_result) -> r.payload) results in
  let digests = List.map (fun (r : Tethers_core_canonical_v2_reference.oracle_result) -> r.digest_string) results in
  let unique_payloads = List.sort_uniq String.compare payloads in
  let unique_digests = List.sort_uniq String.compare digests in
  let num_tested = List.length perms in
  let num_unique_payloads = List.length unique_payloads in
  let num_unique_digests = List.length unique_digests in
  (* Assert: tested 24 input permutations *)
  check_equal_int 24 num_tested "persistent branch: tested input permutations";
  (* Assert: exactly 1 unique payload *)
  check_equal_int 1 num_unique_payloads "persistent branch: unique payload count";
  (* Assert: exactly 1 unique digest *)
  check_equal_int 1 num_unique_digests "persistent branch: unique digest count"

(* ================================================================== *)
(*  Test F: group_id neutrality                                         *)
(* ================================================================== *)

let test_group_id_neutrality () =
  let a1_id = oid "a1" in
  let a2_id = oid "a2" in
  let make_p group_str =
    {
      program_id = pid "test";
      core_version = cv "0.1.0";
      input_facts = [];
      entry_guards = [];
      entry_origin = Some (oid "anchor");
      success_continuations = [];
      origin_sites = [
        Anchor_origin { anchor_origin_id = oid "anchor"; event_name = "ev"; declared_facts = [] };
        Action_origin {
          action_origin_id = a1_id; capability_id = cid "cap.x";
          contract_digest = ccd "sha256:abc"; inputs = [];
          declared_facts = []; execution_constraints = [];
        };
        Action_origin {
          action_origin_id = a2_id; capability_id = cid "cap.x";
          contract_digest = ccd "sha256:abc"; inputs = [];
          declared_facts = []; execution_constraints = [];
        };
        Together_origin {
          together_origin_id = oid "tog";
          group_id = gid group_str;
          member_origin_ids = [a1_id; a2_id];
          objective = All_members_succeed;
        };
      ];
      branches = [];
      roles = [];
      item_templates = [];
      capability_contracts = [{
        capability_id = cid "cap.x";
        contract_digest = ccd "sha256:abc";
        schema_description = "";
      }];
    }
  in
  let r1 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle (make_p "g1")) "group_id_1" in
  let r2 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle (make_p "g2")) "group_id_2" in
  check_equal_string r1.payload r2.payload "group_id neutrality payload";
  check_equal_string r1.digest_string r2.digest_string "group_id neutrality digest"

(* ================================================================== *)
(*  Test G: String bytes (empty, embedded NUL)                         *)
(* ================================================================== *)

let test_string_bytes () =
  let test_encode_string s expected label =
    let result = Tethers_core_canonical_v2_reference.encode_string s in
    check_equal_string expected result label
  in
  test_encode_string "" "0:" "string empty";
  test_encode_string "\x00" "1:\x00" "string embedded NUL";
  test_encode_string "abc\x00def" "7:abc\x00def" "string NUL in middle"

(* ================================================================== *)
(*  Test H: Integer exact boundaries                                   *)
(* ================================================================== *)

let test_integer_exact () =
  let test n expected label =
    let result = Tethers_core_canonical_v2_reference.encode_int n in
    check_equal_string expected result label
  in
  (* -2^62 *)
  test (-4611686018427387904) "-4611686018427387904;" "int -2^62";
  test (-1) "-1;" "int -1";
  test 0 "0;" "int 0";
  test 1 "1;" "int 1";
  (* 2^62 - 1 *)
  test 4611686018427387903 "4611686018427387903;" "int 2^62-1"

(* ================================================================== *)
(*  Test I: Validator suite (authoritative, not print-only)            *)
(* ================================================================== *)

let test_validator_program_role_program_scope () =
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [];
    entry_guards = [];
    entry_origin = None;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [{
      role_id = rid "R1";
      scope = Program_scope;
      fact_contract = Role_fact_contract [];
      eligible_fulfillment = rf "ok";
    }];
    item_templates = [];
    capability_contracts = [];
  } in
  let result = Tethers_core_validator.validate p in
  match result with
  | Ok () -> ()
  | Error errs -> failwith (Printf.sprintf "FAIL: program role Program_scope should be valid, got %d errors" (List.length errs))

let test_validator_program_role_template_scope_invalid () =
  let tid = tid "IT1" in
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [];
    entry_guards = [];
    entry_origin = None;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [{
      role_id = rid "R1";
      scope = Item_template_scope tid;
      fact_contract = Role_fact_contract [];
      eligible_fulfillment = rf "ok";
    }];
    item_templates = [];
    capability_contracts = [];
  } in
  let errs = check_error (Tethers_core_validator.validate p) "validator program role template scope" in
  let has = List.exists (fun e -> match e with Tethers_core_validator.Role_scope_storage_mismatch _ -> true | _ -> false) errs in
  check has "program role with template scope must fail"

let test_validator_template_role_program_scope_invalid () =
  let tid = tid "IT1" in
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [];
    entry_guards = [];
    entry_origin = None;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [];
    item_templates = [{
      item_template_id = tid;
      origin_sites = [];
      branches = [];
      roles = [{
        role_id = rid "R1";
        scope = Program_scope;
        fact_contract = Role_fact_contract [];
        eligible_fulfillment = rf "ok";
      }];
      objective = Required_role (rid "R1");
    }];
    capability_contracts = [];
  } in
  let errs = check_error (Tethers_core_validator.validate p) "validator template role program scope" in
  let has = List.exists (fun e -> match e with Tethers_core_validator.Role_scope_storage_mismatch _ -> true | _ -> false) errs in
  check has "template role with Program_scope must fail"

let test_validator_template_role_own_scope_valid () =
  let tid = tid "IT1" in
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [];
    entry_guards = [];
    entry_origin = None;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [];
    item_templates = [{
      item_template_id = tid;
      origin_sites = [];
      branches = [];
      roles = [{
        role_id = rid "R1";
        scope = Item_template_scope tid;
        fact_contract = Role_fact_contract [];
        eligible_fulfillment = rf "ok";
      }];
      objective = Required_role (rid "R1");
    }];
    capability_contracts = [];
  } in
  let result = Tethers_core_validator.validate p in
  match result with
  | Ok () -> ()
  | Error errs -> failwith (Printf.sprintf "FAIL: template role own scope should be valid, got %d errors" (List.length errs))

let test_validator_template_role_wrong_scope_invalid () =
  let tid_a = tid "TA" in
  let tid_b = tid "TB" in
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [];
    entry_guards = [];
    entry_origin = None;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [];
    item_templates = [{
      item_template_id = tid_a;
      origin_sites = [];
      branches = [];
      roles = [{
        role_id = rid "R1";
        scope = Item_template_scope tid_b;
        fact_contract = Role_fact_contract [];
        eligible_fulfillment = rf "ok";
      }];
      objective = Required_role (rid "R1");
    }];
    capability_contracts = [];
  } in
  let errs = check_error (Tethers_core_validator.validate p) "validator template role wrong scope" in
  let has = List.exists (fun e -> match e with Tethers_core_validator.Role_scope_template_mismatch _ -> true | _ -> false) errs in
  check has "template role with wrong scope must fail"

let test_validator_duplicate_role_fact_contract () =
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [{
      fact_id = fid "f1";
      schema_description = "";
      provenance = Evaluation_input (hsk "hk1", String_type);
    }];
    entry_guards = [];
    entry_origin = None;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [{
      role_id = rid "R1";
      scope = Program_scope;
      fact_contract = Role_fact_contract [fid "f1"; fid "f1"];
      eligible_fulfillment = rf "ok";
    }];
    item_templates = [];
    capability_contracts = [];
  } in
  let errs = check_error (Tethers_core_validator.validate p) "validator dup role fact contract" in
  let has = List.exists (fun e -> match e with Tethers_core_validator.Role_fact_contract_duplicate_fact _ -> true | _ -> false) errs in
  check has "duplicate Role_fact_contract must fail"

let test_validator_role_proxy_program_scope () =
  let anchor_id = oid "anchor" in
  let fact_id_v = fid "f1" in
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [];
    entry_guards = [];
    entry_origin = Some anchor_id;
    success_continuations = [];
    origin_sites = [
      Anchor_origin {
        anchor_origin_id = anchor_id;
        event_name = "ev";
        declared_facts = [];
      };
      Action_origin {
        action_origin_id = oid "act1";
        capability_id = cid "cap.x";
        contract_digest = ccd "sha256:abc";
        inputs = [];
        declared_facts = [{
          fact_id = fact_id_v;
          schema_description = "";
          provenance = Role_proxy (rid "R_prog");
        }];
        execution_constraints = [];
      };
    ];
    branches = [];
    roles = [{
      role_id = rid "R_prog";
      scope = Program_scope;
      fact_contract = Role_fact_contract [];
      eligible_fulfillment = rf "ok";
    }];
    item_templates = [];
    capability_contracts = [{
      capability_id = cid "cap.x";
      contract_digest = ccd "sha256:abc";
      schema_description = "";
    }];
  } in
  let result = Tethers_core_validator.validate p in
  match result with
  | Ok () -> ()
  | Error errs ->
      let msgs = List.map (fun e ->
        match e with
        | Tethers_core_validator.Role_proxy_scope_mismatch (fid, rid) ->
            Printf.sprintf "Role_proxy_scope_mismatch(%s,%s)" (Tethers_core.string_of_fact_id fid) (Tethers_core.string_of_role_id rid)
        | Tethers_core_validator.Unknown_entry_origin oid ->
            Printf.sprintf "Unknown_entry_origin(%s)" (Tethers_core.string_of_origin_id oid)
        | _ -> "other_error"
      ) errs in
      failwith (Printf.sprintf "FAIL: program-site Role_proxy should resolve, got errors: %s" (String.concat "; " msgs))

let test_validator_role_proxy_template_scope () =
  let tid = tid "IT1" in
  let anchor_id = oid "anchor" in
  let fact_id_v = fid "f1" in
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [];
    entry_guards = [];
    entry_origin = Some anchor_id;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [];
    item_templates = [{
      item_template_id = tid;
      origin_sites = [
        Anchor_origin {
          anchor_origin_id = anchor_id;
          event_name = "ev";
          declared_facts = [{
            fact_id = fact_id_v;
            schema_description = "";
            provenance = Role_proxy (rid "R_tmpl");
          }];
        };
      ];
      branches = [];
      roles = [{
        role_id = rid "R_tmpl";
        scope = Item_template_scope tid;
        fact_contract = Role_fact_contract [];
        eligible_fulfillment = rf "ok";
      }];
      objective = Required_role (rid "R_tmpl");
    }];
    capability_contracts = [];
  } in
  let result = Tethers_core_validator.validate p in
  match result with
  | Ok () -> ()
  | Error errs -> failwith (Printf.sprintf "FAIL: template Role_proxy should resolve, got %d errors" (List.length errs))

let test_validator_same_raw_role_wrong_template () =
  let tid_a = tid "TA" in
  let tid_b = tid "TB" in
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [];
    entry_guards = [];
    entry_origin = None;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [];
    item_templates = [{
      item_template_id = tid_a;
      origin_sites = [];
      branches = [];
      roles = [{
        role_id = rid "R";
        scope = Item_template_scope tid_b;
        fact_contract = Role_fact_contract [];
        eligible_fulfillment = rf "ok";
      }];
      objective = Required_role (rid "R");
    }];
    capability_contracts = [];
  } in
  let errs = check_error (Tethers_core_validator.validate p) "validator same raw role wrong template" in
  let has = List.exists (fun e -> match e with Tethers_core_validator.Role_scope_template_mismatch _ -> true | _ -> false) errs in
  check has "same raw role ID in wrong template must fail"

(* ================================================================== *)
(*  Test J: Role_fact_contract adversarial label ordering               *)
(*  Same role contract contains two facts whose raw lexical order is   *)
(*  opposite their candidate canonical label order.                    *)
(*  Consistent raw-ID renaming must not change payload/digest.         *)
(* ================================================================== *)

let test_role_fact_contract_adversarial_order () =
  (* P1: raw IDs "fA" and "fB" — lexical order matches label order *)
  let p1 = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [
      { fact_id = fid "fA"; schema_description = "";
        provenance = Evaluation_input (hsk "hkA", String_type) };
      { fact_id = fid "fB"; schema_description = "";
        provenance = Evaluation_input (hsk "hkB", String_type) };
    ];
    entry_guards = [];
    entry_origin = None;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [{
      role_id = rid "R1";
      scope = Program_scope;
      fact_contract = Role_fact_contract [fid "fA"; fid "fB"];
      eligible_fulfillment = rf "ok";
    }];
    item_templates = [];
    capability_contracts = [];
  } in
  (* P2: rename so raw lexical order is OPPOSITE label order *)
  let p2 = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [
      { fact_id = fid "fB"; schema_description = "";
        provenance = Evaluation_input (hsk "hkB", String_type) };
      { fact_id = fid "fA"; schema_description = "";
        provenance = Evaluation_input (hsk "hkA", String_type) };
    ];
    entry_guards = [];
    entry_origin = None;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [{
      role_id = rid "R1";
      scope = Program_scope;
      fact_contract = Role_fact_contract [fid "fB"; fid "fA"];
      eligible_fulfillment = rf "ok";
    }];
    item_templates = [];
    capability_contracts = [];
  } in
  let r1 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p1) "adversarial_order_1" in
  let r2 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p2) "adversarial_order_2" in
  check_equal_int r1.candidate_count r2.candidate_count "adversarial role_fact_contract candidate count";
  check_equal_string r1.payload r2.payload "adversarial role_fact_contract payload";
  check_equal_string r1.digest_string r2.digest_string "adversarial role_fact_contract digest"

(* ================================================================== *)
(*  Test K: Action input secondary binding sort                        *)
(*  Two inputs with same name but different bindings.                  *)
(*  Storage order reversed must not change payload/digest.             *)
(* ================================================================== *)

let test_action_input_binding_sort () =
  let anchor_id = oid "anchor" in
  let make_p ~input_order =
    let inputs = match input_order with
      | `AB -> [
          { input_name = capability_input_name_of_string "x";
            binding = Literal_value (String_value "alpha") };
          { input_name = capability_input_name_of_string "x";
            binding = Literal_value (String_value "beta") };
        ]
      | `BA -> [
          { input_name = capability_input_name_of_string "x";
            binding = Literal_value (String_value "beta") };
          { input_name = capability_input_name_of_string "x";
            binding = Literal_value (String_value "alpha") };
        ]
    in
    {
      program_id = pid "test";
      core_version = cv "0.1.0";
      input_facts = [];
      entry_guards = [];
      entry_origin = Some anchor_id;
      success_continuations = [];
      origin_sites = [
        Anchor_origin {
          anchor_origin_id = anchor_id; event_name = "ev";
          declared_facts = [];
        };
        Action_origin {
          action_origin_id = oid "act1";
          capability_id = cid "cap.x";
          contract_digest = ccd "sha256:abc";
          inputs = inputs;
          declared_facts = [];
          execution_constraints = [];
        };
      ];
      branches = [];
      roles = [];
      item_templates = [];
      capability_contracts = [{
        capability_id = cid "cap.x";
        contract_digest = ccd "sha256:abc";
        schema_description = "";
      }];
    }
  in
  let r1 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle (make_p ~input_order:`AB)) "input_sort_ab" in
  let r2 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle (make_p ~input_order:`BA)) "input_sort_ba" in
  check_equal_int r1.candidate_count r2.candidate_count "action input binding sort candidate count";
  check_equal_string r1.payload r2.payload "action input binding sort payload";
  check_equal_string r1.digest_string r2.digest_string "action input binding sort digest"

(* ================================================================== *)
(*  Main test runner                                                   *)
(* ================================================================== *)

let () =
  Printf.printf "=== V2 Reference Oracle Tests ===\n\n";
  test_domain_v2 ();
  Printf.printf "PASS: DOMAIN_V2\n";
  test_digest_string_format ();
  Printf.printf "PASS: digest_string_v2 format\n";
  test_empty_program ();
  Printf.printf "PASS: empty program\n";
  test_single_anchor_action ();
  Printf.printf "PASS: single anchor+action\n";
  test_neutrality_program_id ();
  Printf.printf "PASS: program_id neutrality\n";
  test_neutrality_schema_description ();
  Printf.printf "PASS: schema_description neutrality\n";
  test_raw_id_rename ();
  Printf.printf "PASS: raw-ID rename invariance\n";
  test_multiplicity_1_vs_2 ();
  Printf.printf "PASS: multiplicity 1 vs 2\n";
  test_together_member_order ();
  Printf.printf "PASS: Together member order\n";
  test_role_scope_validation ();
  Printf.printf "PASS: role scope validation\n";
  test_role_fact_contract_duplicate ();
  Printf.printf "PASS: role_fact_contract duplicate\n";
  test_integer_boundaries ();
  Printf.printf "PASS: integer boundaries\n";
  test_string_encoding ();
  Printf.printf "PASS: string encoding\n";
  test_cross_family_raw_id_collision ();
  Printf.printf "PASS: cross-family raw-ID collision\n";
  test_scoped_same_raw_role_ids ();
  Printf.printf "PASS: scoped same-raw role IDs\n";
  test_role_block_count ();
  Printf.printf "PASS: role-block enumeration count\n";
  test_nested_storage_order ();
  Printf.printf "PASS: nested storage order\n";
  test_nested_storage_order_with_batch ();
  Printf.printf "PASS: nested storage order with batch\n";
  test_persistent_branch_gold ();
  Printf.printf "PASS: persistent branch gold (24 permutations, 576 candidates each)\n";
  test_group_id_neutrality ();
  Printf.printf "PASS: group_id neutrality\n";
  test_string_bytes ();
  Printf.printf "PASS: string bytes\n";
  test_integer_exact ();
  Printf.printf "PASS: integer exact boundaries\n";
  test_validator_program_role_program_scope ();
  Printf.printf "PASS: validator program role Program_scope valid\n";
  test_validator_program_role_template_scope_invalid ();
  Printf.printf "PASS: validator program role template scope invalid\n";
  test_validator_template_role_program_scope_invalid ();
  Printf.printf "PASS: validator template role Program_scope invalid\n";
  test_validator_template_role_own_scope_valid ();
  Printf.printf "PASS: validator template role own scope valid\n";
  test_validator_template_role_wrong_scope_invalid ();
  Printf.printf "PASS: validator template role wrong scope invalid\n";
  test_validator_duplicate_role_fact_contract ();
  Printf.printf "PASS: validator duplicate Role_fact_contract invalid\n";
  test_validator_role_proxy_program_scope ();
  Printf.printf "PASS: validator program-site Role_proxy resolves\n";
  test_validator_role_proxy_template_scope ();
  Printf.printf "PASS: validator template Role_proxy resolves\n";
  test_validator_same_raw_role_wrong_template ();
  Printf.printf "PASS: validator same raw role wrong template fails\n";
  test_role_fact_contract_adversarial_order ();
  Printf.printf "PASS: role_fact_contract adversarial label ordering\n";
  test_action_input_binding_sort ();
  Printf.printf "PASS: action input secondary binding sort\n";
  Printf.printf "\n=== All Tests Complete ===\n"
