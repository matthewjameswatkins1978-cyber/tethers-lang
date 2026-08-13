(* ==================================================================
   CANONICAL FORMAT V2 — REFERENCE ORACLE TESTS

   Tests for the frozen V2 reference encoder and slow oracle.
   ================================================================== *)

open Tethers_core

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
  match Tethers_core_canonical_v2_reference.slow_oracle p with
  | Error _ -> Printf.printf "FAIL: empty program returned error\n"
  | Ok result ->
      Printf.printf "PASS: empty program candidate_count=%d digest=%s\n"
        result.candidate_count result.digest_string

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
  match Tethers_core_canonical_v2_reference.slow_oracle p with
  | Error _ ->
      Printf.printf "FAIL: single anchor+action returned oracle error\n"
  | Ok result ->
      Printf.printf "PASS: single anchor+action candidates=%d digest=%s\n"
        result.candidate_count result.digest_string

let test_domain_v2 () =
  let expected = "TETHERS_CORE_CANON_V2\x00" in
  let actual = Bytes.to_string Tethers_core_canonical_v2_reference.domain_v2 in
  if expected = actual then
    Printf.printf "PASS: DOMAIN_V2 bytes correct\n"
  else
    Printf.printf "FAIL: DOMAIN_V2 bytes incorrect\n"

let test_digest_string_format () =
  let hex = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" in
  let expected = "tethers:v2:sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" in
  let actual = Tethers_core_canonical_v2_reference.digest_string_v2 hex in
  if expected = actual then
    Printf.printf "PASS: digest_string_v2 format correct\n"
  else
    Printf.printf "FAIL: digest_string_v2 format incorrect: %s\n" actual

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
  let p2 = {
    p1 with program_id = pid "id2";
  } in
  let r1 = Tethers_core_canonical_v2_reference.slow_oracle p1 in
  let r2 = Tethers_core_canonical_v2_reference.slow_oracle p2 in
  match r1, r2 with
  | Ok r1, Ok r2 ->
      if r1.payload = r2.payload && r1.digest_string = r2.digest_string then
        Printf.printf "PASS: program_id neutrality preserved\n"
      else
        Printf.printf "FAIL: program_id neutrality violated\n"
  | _ -> Printf.printf "FAIL: oracle error\n"

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
  let r1 = Tethers_core_canonical_v2_reference.slow_oracle (make_p fact1) in
  let r2 = Tethers_core_canonical_v2_reference.slow_oracle (make_p fact2) in
  match r1, r2 with
  | Ok r1, Ok r2 ->
      if r1.payload = r2.payload && r1.digest_string = r2.digest_string then
        Printf.printf "PASS: schema_description neutrality preserved\n"
      else
        Printf.printf "FAIL: schema_description neutrality violated\n"
  | _ -> Printf.printf "FAIL: oracle error\n"

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
  let r1 = Tethers_core_canonical_v2_reference.slow_oracle p1 in
  let r2 = Tethers_core_canonical_v2_reference.slow_oracle p2 in
  match r1, r2 with
  | Ok r1, Ok r2 ->
      if r1.payload = r2.payload && r1.digest_string = r2.digest_string then
        Printf.printf "PASS: raw-ID rename invariance\n"
      else
        Printf.printf "FAIL: raw-ID rename invariance violated\n"
  | _ -> Printf.printf "FAIL: oracle error\n"

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
  let r1 = Tethers_core_canonical_v2_reference.slow_oracle (make_p 1) in
  let r2 = Tethers_core_canonical_v2_reference.slow_oracle (make_p 2) in
  match r1, r2 with
  | Ok r1, Ok r2 ->
      if r1.digest_string <> r2.digest_string then
        Printf.printf "PASS: multiplicity 1 vs 2 different digest\n"
      else
        Printf.printf "FAIL: multiplicity 1 vs 2 same digest\n"
  | _ -> Printf.printf "FAIL: oracle error\n"

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
  let r1 = Tethers_core_canonical_v2_reference.slow_oracle (make_p `AB) in
  let r2 = Tethers_core_canonical_v2_reference.slow_oracle (make_p `BA) in
  match r1, r2 with
  | Ok r1, Ok r2 ->
      if r1.payload = r2.payload && r1.digest_string = r2.digest_string then
        Printf.printf "PASS: Together member order invariant\n"
      else
        Printf.printf "FAIL: Together member order invariant violated\n"
  | _ -> Printf.printf "FAIL: oracle error\n"

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
        scope = Program_scope; (* Wrong: should be Item_template_scope tid2 *)
        fact_contract = Role_fact_contract [];
        eligible_fulfillment = rf "ok";
      }];
      objective = Required_role (rid "R2");
    }];
    capability_contracts = [];
  } in
  match Tethers_core_validator.validate p with
  | Error errs ->
      let has_mismatch = List.exists (fun e ->
        match e with
        | Tethers_core_validator.Role_scope_storage_mismatch _ -> true
        | _ -> false
      ) errs in
      if has_mismatch then
        Printf.printf "PASS: role scope storage mismatch detected\n"
      else
        Printf.printf "FAIL: role scope storage mismatch not detected\n"
  | Ok () -> Printf.printf "FAIL: program with scope mismatch accepted\n"

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
      fact_contract = Role_fact_contract [fid "f1"; fid "f1"]; (* Duplicate! *)
      eligible_fulfillment = rf "ok";
    }];
    item_templates = [];
    capability_contracts = [];
  } in
  match Tethers_core_validator.validate p with
  | Error errs ->
      let has_dup = List.exists (fun e ->
        match e with
        | Tethers_core_validator.Role_fact_contract_duplicate_fact _ -> true
        | _ -> false
      ) errs in
      if has_dup then
        Printf.printf "PASS: role_fact_contract duplicate detected\n"
      else
        Printf.printf "FAIL: role_fact_contract duplicate not detected\n"
  | Ok () -> Printf.printf "FAIL: program with duplicate role_fact_contract accepted\n"

let test_integer_boundaries () =
  let test_encode_int n expected =
    let result = Tethers_core_canonical_v2_reference.encode_int n in
    if result = expected then
      Printf.printf "PASS: encode_int %d = %s\n" n expected
    else
      Printf.printf "FAIL: encode_int %d = %s, expected %s\n" n result expected
  in
  test_encode_int 0 "0;";
  test_encode_int 1 "1;";
  test_encode_int (-1) "-1;";
  test_encode_int 42 "42;";
  test_encode_int (-42) "-42;"

let test_string_encoding () =
  let test_encode_string s expected =
    let result = Tethers_core_canonical_v2_reference.encode_string s in
    if result = expected then
      Printf.printf "PASS: encode_string %S = %s\n" s expected
    else
      Printf.printf "FAIL: encode_string %S = %s, expected %s\n" s result expected
  in
  test_encode_string "" "0:";
  test_encode_string "hello" "5:hello";
  test_encode_string "a" "1:a"

let test_empty_string () =
  let s = "" in
  let encoded = Tethers_core_canonical_v2_reference.encode_string s in
  if encoded = "0:" then
    Printf.printf "PASS: empty string encoded as 0:\n"
  else
    Printf.printf "FAIL: empty string encoded as %s\n" encoded

let () =
  Printf.printf "=== V2 Reference Oracle Tests ===\n\n";
  test_domain_v2 ();
  test_digest_string_format ();
  test_empty_program ();
  test_single_anchor_action ();
  test_neutrality_program_id ();
  test_neutrality_schema_description ();
  test_raw_id_rename ();
  test_multiplicity_1_vs_2 ();
  test_together_member_order ();
  test_role_scope_validation ();
  test_role_fact_contract_duplicate ();
  test_integer_boundaries ();
  test_string_encoding ();
  test_empty_string ();
  Printf.printf "\n=== Tests Complete ===\n"
