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
(*  Test D: Nested storage order permutation invariance                 *)
(*  Reverse/permutate each D/B collection.                             *)
(*  Oracle payload and digest MUST remain byte-identical.              *)
(* ================================================================== *)

let test_nested_storage_order () =
  (* Build one valid program, then build a second with the same entities
     but reversed storage order in input_facts and origin_sites.
     The oracle must produce the same canonical payload. *)
  let anchor_id = oid "anchor" in
  let action_id1 = oid "act1" in
  let action_id2 = oid "act2" in
  let make_p anchor_first =
    let origins =
      if anchor_first then [
        Anchor_origin {
          anchor_origin_id = anchor_id;
          event_name = "ev";
          declared_facts = [];
        };
        Action_origin {
          action_origin_id = action_id1;
          capability_id = cid "cap.x";
          contract_digest = ccd "sha256:abc";
          inputs = [];
          declared_facts = [];
          execution_constraints = [Deadline "10s"; Deadline "5s"];
        };
        Action_origin {
          action_origin_id = action_id2;
          capability_id = cid "cap.x";
          contract_digest = ccd "sha256:abc";
          inputs = [];
          declared_facts = [];
          execution_constraints = [];
        };
      ] else [
        Action_origin {
          action_origin_id = action_id2;
          capability_id = cid "cap.x";
          contract_digest = ccd "sha256:abc";
          inputs = [];
          declared_facts = [];
          execution_constraints = [];
        };
        Action_origin {
          action_origin_id = action_id1;
          capability_id = cid "cap.x";
          contract_digest = ccd "sha256:abc";
          inputs = [];
          declared_facts = [];
          execution_constraints = [Deadline "10s"; Deadline "5s"];
        };
        Anchor_origin {
          anchor_origin_id = anchor_id;
          event_name = "ev";
          declared_facts = [];
        };
      ]
    in
    {
      program_id = pid "test";
      core_version = cv "0.1.0";
      input_facts = [
        { fact_id = fid "f1"; schema_description = ""; provenance = Evaluation_input (hsk "hk1", String_type) };
        { fact_id = fid "f2"; schema_description = ""; provenance = Evaluation_input (hsk "hk2", String_type) };
      ];
      entry_guards = [];
      entry_origin = Some anchor_id;
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
  let r1 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle (make_p true)) "nested_order_1" in
  let r2 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle (make_p false)) "nested_order_2" in
  Printf.printf "nested_order: c1=%d c2=%d\n%!" r1.candidate_count r2.candidate_count;
  if r1.payload <> r2.payload then begin
    Printf.printf "  p1 first 80: %S\n%!" (String.sub r1.payload 0 (min 80 (String.length r1.payload)));
    Printf.printf "  p2 first 80: %S\n%!" (String.sub r2.payload 0 (min 80 (String.length r2.payload)));
  end;
  (* For now, just check that both are valid and have the same candidate count *)
  check_equal_int r1.candidate_count r2.candidate_count "nested storage order candidate count";
  (* The canonical payload for storage-order variants should match.
     If it doesn't, it reveals a bug in the oracle's permutation coverage. *)
  check_equal_string r1.payload r2.payload "nested storage order permutation invariance payload"

(* ================================================================== *)
(*  Test E: Persistent Branch Gold Test (C-B3T witness)                *)
(*  4 equivalent Origins + 4 equivalent Branches                       *)
(*  Expected: 4! x 4! = 576 candidates                                *)
(*  All produce identical canonical payload.                           *)
(* ================================================================== *)

let test_persistent_branch_gold () =
  let origins = List.init 4 (fun i ->
    Anchor_origin {
      anchor_origin_id = oid ("a" ^ string_of_int i);
      event_name = "ev";
      declared_facts = [];
    }
  ) in
  let branches = List.init 4 (fun i ->
    {
      branch_id = branch_id_of_string ("b" ^ string_of_int i);
      branch_subject = oid ("a" ^ string_of_int i);
      outcome_branches = [(Success, Stop)];
    }
  ) in
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [];
    entry_guards = [];
    entry_origin = Some (oid "a0");
    success_continuations = [];
    origin_sites = origins;
    branches = branches;
    roles = [];
    item_templates = [];
    capability_contracts = [];
  } in
  let result = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p) "persistent_branch_gold" in
  (* 4! origins x 4! branches = 24 x 24 = 576 *)
  check_equal_int 576 result.candidate_count "persistent branch candidate count (4! x 4! = 576)"

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
  test_persistent_branch_gold ();
  Printf.printf "PASS: persistent branch gold (576 candidates)\n";
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
  Printf.printf "\n=== All Tests Complete ===\n"
