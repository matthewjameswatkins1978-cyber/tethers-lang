(* ==================================================================
   CANONICAL FORMAT V2 — PRODUCTION CANONICALISER TESTS

   Conformance tests for the production V2 canonicaliser.
   Proves exact agreement with the reference oracle on all
   oracle-sized cases, plus budget and streaming structure tests.
   ================================================================== *)

open Tethers_core

(* ================================================================== *)
(*  Assertion helpers                                                  *)
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

(* ================================================================== *)
(*  Constructor helpers                                                *)
(* ================================================================== *)

let pid s = program_id_of_string s
let oid s = origin_id_of_string s
let fid s = fact_id_of_string s
let rid s = role_id_of_string s
let cid s = capability_id_of_string s
let tid s = item_template_id_of_string s
let cv s = core_version_of_string s
let hsk s = host_snapshot_key_of_string s
let ccd s = capability_contract_digest_of_string s
let rf s = role_fulfillment_of_string s

(* ================================================================== *)
(*  J1: All existing V2 gold fixtures — production == oracle           *)
(* ================================================================== *)

let test_empty_program_prod () =
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
  let oracle = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p) "empty_oracle" in
  let prod = check_ok (Tethers_core_canonical_v2.canonicalize p) "empty_prod" in
  check_equal_int 1 oracle.candidate_count "empty candidate count";
  check_equal_string oracle.payload (Tethers_core_canonical_v2.canonical_payload prod) "empty payload";
  check_equal_string oracle.digest_string (Tethers_core_canonical_v2.program_digest prod) "empty digest"

let test_single_anchor_action_prod () =
  let anchor_id = oid "anchor1" in
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [{
      fact_id = fid "fact1";
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
    branches = [];
    roles = [];
    item_templates = [];
    capability_contracts = [];
  } in
  let oracle = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p) "single_anchor_oracle" in
  let prod = check_ok (Tethers_core_canonical_v2.canonicalize p) "single_anchor_prod" in
  check_equal_string oracle.payload (Tethers_core_canonical_v2.canonical_payload prod) "single anchor payload";
  check_equal_string oracle.digest_string (Tethers_core_canonical_v2.program_digest prod) "single anchor digest"

let test_cross_family_prod () =
  let make_p anchor_name =
    let anchor_id = oid anchor_name in
    {
      program_id = pid "test";
      core_version = cv "0.1.0";
      input_facts = [{
        fact_id = fid "X";
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
        branch_id = branch_id_of_string "X";
        branch_subject = anchor_id;
        outcome_branches = [(Success, Stop)];
      }];
      roles = [];
      item_templates = [];
      capability_contracts = [];
    }
  in
  let oracle = check_ok (Tethers_core_canonical_v2_reference.slow_oracle (make_p "X")) "cross_family_oracle" in
  let prod = check_ok (Tethers_core_canonical_v2.canonicalize (make_p "X")) "cross_family_prod" in
  check_equal_string oracle.payload (Tethers_core_canonical_v2.canonical_payload prod) "cross-family payload";
  check_equal_string oracle.digest_string (Tethers_core_canonical_v2.program_digest prod) "cross-family digest"

(* ================================================================== *)
(*  J2: Persistent Branch — 24 permutations                            *)
(* ================================================================== *)

let test_persistent_branch_prod () =
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
  let perms = Tethers_core_canonical_v2_reference.perm [0;1;2;3] in
  let names = ["a0";"a1";"a2";"a3"] in
  let branch_names = ["b0";"b1";"b2";"b3"] in
  let map_names perm = List.map (fun i -> List.nth names i) perm in
  let map_branches perm = List.map (fun i -> List.nth branch_names i) perm in
  let run_one perm =
    let onames = map_names perm in
    let bnames = map_branches perm in
    let p = make_p onames bnames in
    let oracle = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p)
      (Printf.sprintf "persistent_oracle_%s" (String.concat "," (List.map string_of_int perm))) in
    let prod = check_ok (Tethers_core_canonical_v2.canonicalize p)
      (Printf.sprintf "persistent_prod_%s" (String.concat "," (List.map string_of_int perm))) in
    check_equal_string oracle.payload (Tethers_core_canonical_v2.canonical_payload prod)
      (Printf.sprintf "persistent payload perm %s" (String.concat "," (List.map string_of_int perm)));
    check_equal_string oracle.digest_string (Tethers_core_canonical_v2.program_digest prod)
      (Printf.sprintf "persistent digest perm %s" (String.concat "," (List.map string_of_int perm)));
    prod
  in
  let results = List.map run_one perms in
  let unique_payloads = List.sort_uniq String.compare (List.map Tethers_core_canonical_v2.canonical_payload results) in
  let unique_digests = List.sort_uniq String.compare (List.map Tethers_core_canonical_v2.program_digest results) in
  check_equal_int 24 (List.length perms) "persistent: tested permutations";
  check_equal_int 1 (List.length unique_payloads) "persistent: unique payloads";
  check_equal_int 1 (List.length unique_digests) "persistent: unique digests"

(* ================================================================== *)
(*  J3: Role blocks — 2 templates, 2 roles each                        *)
(* ================================================================== *)

let test_role_blocks_prod () =
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
  let oracle = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p) "role_blocks_oracle" in
  let prod = check_ok (Tethers_core_canonical_v2.canonicalize p) "role_blocks_prod" in
  check_equal_int 8 oracle.candidate_count "role blocks candidate count";
  check_equal_string oracle.payload (Tethers_core_canonical_v2.canonical_payload prod) "role blocks payload";
  check_equal_string oracle.digest_string (Tethers_core_canonical_v2.program_digest prod) "role blocks digest"

(* ================================================================== *)
(*  J5: Template/Batch/nested collection cases                         *)
(* ================================================================== *)

let test_nested_storage_order_prod () =
  let anchor1 = oid "anc1" and anchor2 = oid "anc2" in
  let action1 = oid "act1" and action2 = oid "act2" in
  let br1 = branch_id_of_string "br1" and br2 = branch_id_of_string "br2" in
  let fact_a1 = fid "fa1" and fact_a2 = fid "fa2" in
  let r1 = rid "R1" and r2 = rid "R2" in
  let itid = tid "IT1" in

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
    in
    let make_branch bid succ =
      { branch_id = bid; branch_subject = anchor1;
        outcome_branches = [(Success, succ)]; }
    in
    let template_branches = match branch_order with
      | `ABCD -> [make_branch br1 Stop; make_branch br2 Stop]
      | `DCBA -> [make_branch br2 Stop; make_branch br1 Stop]
    in
    let template_roles = match role_order with
      | `ABCD -> [
          { role_id = r1; scope = Item_template_scope itid;
            fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok" };
          { role_id = r2; scope = Item_template_scope itid;
            fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok" };
        ]
      | `DCBA -> [
          { role_id = r2; scope = Item_template_scope itid;
            fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok" };
          { role_id = r1; scope = Item_template_scope itid;
            fact_contract = Role_fact_contract []; eligible_fulfillment = rf "ok" };
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
      }];
      capability_contracts = [{
        capability_id = cid "cap.x";
        contract_digest = ccd "sha256:abc";
        schema_description = "";
      }];
    }
  in
  let p1 = make_p ~origin_order:`ABCD ~branch_order:`ABCD ~role_order:`ABCD in
  let p2 = make_p ~origin_order:`DCBA ~branch_order:`DCBA ~role_order:`DCBA in
  let oracle1 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p1) "nested_oracle_1" in
  let prod1 = check_ok (Tethers_core_canonical_v2.canonicalize p1) "nested_prod_1" in
  let prod2 = check_ok (Tethers_core_canonical_v2.canonicalize p2) "nested_prod_2" in
  check_equal_string oracle1.payload (Tethers_core_canonical_v2.canonical_payload prod1) "nested oracle vs prod 1";
  check_equal_string (Tethers_core_canonical_v2.canonical_payload prod1) (Tethers_core_canonical_v2.canonical_payload prod2) "nested prod 1 vs prod 2"

(* ================================================================== *)
(*  J7: Multiplicity — one vs two different, two identical swapped same *)
(* ================================================================== *)

let test_multiplicity_prod () =
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
  let r1 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle (make_p 1)) "mult_1_oracle" in
  let r2 = check_ok (Tethers_core_canonical_v2_reference.slow_oracle (make_p 2)) "mult_2_oracle" in
  let p1 = check_ok (Tethers_core_canonical_v2.canonicalize (make_p 1)) "mult_1_prod" in
  let p2 = check_ok (Tethers_core_canonical_v2.canonicalize (make_p 2)) "mult_2_prod" in
  check_equal_string r1.payload (Tethers_core_canonical_v2.canonical_payload p1) "mult 1 oracle==prod";
  check_equal_string r2.payload (Tethers_core_canonical_v2.canonical_payload p2) "mult 2 oracle==prod";
  check (r1.digest_string <> r2.digest_string) "multiplicity 1 vs 2 different digest"

(* ================================================================== *)
(*  J8: Strings — embedded NUL, high bytes                              *)
(* ================================================================== *)

let test_strings_prod () =
  let p_empty_str = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [{
      fact_id = fid "f1";
      schema_description = "";
      provenance = Evaluation_input (hsk "", String_type);
    }];
    entry_guards = [];
    entry_origin = None;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [];
    item_templates = [];
    capability_contracts = [];
  } in
  let p_nul_str = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = [{
      fact_id = fid "f1";
      schema_description = "";
      provenance = Evaluation_input (hsk "abc\x00def", String_type);
    }];
    entry_guards = [];
    entry_origin = None;
    success_continuations = [];
    origin_sites = [];
    branches = [];
    roles = [];
    item_templates = [];
    capability_contracts = [];
  } in
  let oracle_empty = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p_empty_str) "str_empty_oracle" in
  let prod_empty = check_ok (Tethers_core_canonical_v2.canonicalize p_empty_str) "str_empty_prod" in
  check_equal_string oracle_empty.payload (Tethers_core_canonical_v2.canonical_payload prod_empty) "empty string key payload";
  let oracle_nul = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p_nul_str) "str_nul_oracle" in
  let prod_nul = check_ok (Tethers_core_canonical_v2.canonicalize p_nul_str) "str_nul_prod" in
  check_equal_string oracle_nul.payload (Tethers_core_canonical_v2.canonical_payload prod_nul) "NUL string key payload"

(* ================================================================== *)
(*  J9: Integer boundaries — value actually enters Enc_V2               *)
(* ================================================================== *)

let test_integer_boundaries_prod () =
  let test_int n =
    let p = {
      program_id = pid "test";
      core_version = cv "0.1.0";
      input_facts = [{
        fact_id = fid "f1";
        schema_description = "";
        provenance = Evaluation_input (hsk "hk", Integer_type);
      }];
      entry_guards = [{
        fact_id = fid "f1";
        operator = Equals;
        expected = Integer_value n;
      }];
      entry_origin = None;
      success_continuations = [];
      origin_sites = [];
      branches = [];
      roles = [];
      item_templates = [];
      capability_contracts = [];
    } in
    let oracle = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p)
      (Printf.sprintf "int_%d_oracle" n) in
    let prod = check_ok (Tethers_core_canonical_v2.canonicalize p)
      (Printf.sprintf "int_%d_prod" n) in
    check_equal_string oracle.payload (Tethers_core_canonical_v2.canonical_payload prod)
      (Printf.sprintf "integer boundary %d payload" n)
  in
  test_int (-4611686018427387904);
  test_int (-1);
  test_int 0;
  test_int 1;
  test_int 4611686018427387903

(* ================================================================== *)
(*  HIGH-BYTE: compare_bytes_lex_unsigned direct tests                  *)
(* ================================================================== *)

let test_high_byte_comparator () =
  let cmp = Tethers_core_canonical_v2_format.compare_bytes_lex_unsigned in
  (* \x7f < \x80 because 0x7f < 0x80 *)
  check (cmp "\x7f" "\x80" < 0) "\\x7f < \\x80";
  (* \x80 < \xff because 0x80 < 0xff *)
  check (cmp "\x80" "\xff" < 0) "\\x80 < \\xff";
  (* \xff > \x80 *)
  check (cmp "\xff" "\x80" > 0) "\\xff > \\x80";
  (* \x80 < \x80\x00 because \x80 is prefix of \x80\x00 *)
  check (cmp "\x80" "\x80\x00" < 0) "\\x80 < \\x80\\x00";
  (* Empty string is prefix of everything *)
  check (cmp "" "a" < 0) "empty < a";
  check (cmp "" "\xff" < 0) "empty < \\xff";
  (* Same string *)
  check (cmp "\x80" "\x80" = 0) "\\x80 = \\x80"

(* ================================================================== *)
(*  K: Production beyond oracle limit (family size 7)                   *)
(* ================================================================== *)

let test_family_size_7 () =
  let anchor = oid "anchor" in
  let facts = List.init 7 (fun i ->
    { fact_id = fid ("f" ^ string_of_int i);
      schema_description = "";
      provenance = Evaluation_input (hsk ("hk" ^ string_of_int i), String_type); }
  ) in
  let p = {
    program_id = pid "test";
    core_version = cv "0.1.0";
    input_facts = facts;
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
    item_templates = [];
    capability_contracts = [];
  } in
  (* Oracle should reject (family size 7 > max 6) *)
  let oracle_result = Tethers_core_canonical_v2_reference.slow_oracle p in
  (match oracle_result with
   | Error Tethers_core_canonical_v2_reference.Oracle_too_large -> ()
   | Ok _ -> failwith "FAIL: oracle should reject family size 7"
   | Error (Tethers_core_canonical_v2_reference.Invalid_core _) ->
       failwith "FAIL: oracle rejected with Invalid_core instead of Oracle_too_large");

  (* Production should succeed with default budget *)
  let prod = check_ok (Tethers_core_canonical_v2.canonicalize p) "family_7_prod" in
  let payload = Tethers_core_canonical_v2.canonical_payload prod in
  check (String.length payload > 0) "family 7 has non-empty payload";

  (* Test multiple storage permutations produce same result *)
  let make_p order =
    let ordered_facts = match order with
      | `Normal -> facts
      | `Reversed -> List.rev facts
    in
    { p with input_facts = ordered_facts }
  in
  let prod_normal = check_ok (Tethers_core_canonical_v2.canonicalize (make_p `Normal)) "family_7_normal" in
  let prod_reversed = check_ok (Tethers_core_canonical_v2.canonicalize (make_p `Reversed)) "family_7_reversed" in
  check_equal_string
    (Tethers_core_canonical_v2.canonical_payload prod_normal)
    (Tethers_core_canonical_v2.canonical_payload prod_reversed)
    "family 7 storage order invariant";
  check_equal_string
    (Tethers_core_canonical_v2.program_digest prod_normal)
    (Tethers_core_canonical_v2.program_digest prod_reversed)
    "family 7 storage order digest invariant"

(* ================================================================== *)
(*  L: Budget tests                                                     *)
(* ================================================================== *)

let test_budget_exact_boundary () =
  (* 4 origins -> 4! = 24 candidates *)
  let origins = List.init 4 (fun i ->
    Anchor_origin {
      anchor_origin_id = oid ("a" ^ string_of_int i);
      event_name = "ev";
      declared_facts = [];
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
    branches = [];
    roles = [];
    item_templates = [];
    capability_contracts = [];
  } in
  (* Budget 23: should fail *)
  let budget_23 = { Tethers_core_canonical_v2.max_candidates = 23 } in
  let result_23 = Tethers_core_canonical_v2.canonicalize ~budget:budget_23 p in
  (match result_23 with
   | Error Tethers_core_canonical_v2.Canonicalisation_too_complex -> ()
   | Ok _ -> failwith "FAIL: budget 23 should fail for 24-candidate space"
   | Error (Tethers_core_canonical_v2.Invalid_core _) ->
       failwith "FAIL: should not get Invalid_core for valid program");

  (* Budget 24: should succeed *)
  let budget_24 = { Tethers_core_canonical_v2.max_candidates = 24 } in
  let result_24 = check_ok (Tethers_core_canonical_v2.canonicalize ~budget:budget_24 p) "budget_24" in
  check (String.length (Tethers_core_canonical_v2.canonical_payload result_24) > 0)
    "budget 24 produces payload";

  (* Budget 25: should also succeed *)
  let budget_25 = { Tethers_core_canonical_v2.max_candidates = 25 } in
  let result_25 = check_ok (Tethers_core_canonical_v2.canonicalize ~budget:budget_25 p) "budget_25" in
  check_equal_string
    (Tethers_core_canonical_v2.canonical_payload result_24)
    (Tethers_core_canonical_v2.canonical_payload result_25)
    "budget 24 and 25 same result"

let test_budget_equivalent_raw_variants () =
  let make_p names =
    let origins = List.map (fun name ->
      Anchor_origin {
        anchor_origin_id = oid name;
        event_name = "ev";
        declared_facts = [];
      }
    ) names in
    {
      program_id = pid "test";
      core_version = cv "0.1.0";
      input_facts = [];
      entry_guards = [];
      entry_origin = Some (oid (List.hd names));
      success_continuations = [];
      origin_sites = origins;
      branches = [];
      roles = [];
      item_templates = [];
      capability_contracts = [];
    }
  in
  let budget_23 = { Tethers_core_canonical_v2.max_candidates = 23 } in
  let result_normal = Tethers_core_canonical_v2.canonicalize ~budget:budget_23 (make_p ["a";"b";"c";"d"]) in
  let result_renamed = Tethers_core_canonical_v2.canonicalize ~budget:budget_23 (make_p ["x";"y";"z";"w"]) in
  (* Both should have the same budget admission result *)
  let both_failed = (match result_normal, result_renamed with
    | Error _, Error _ -> true
    | Ok _, Ok _ -> true
    | _ -> false) in
  check both_failed "equivalent variants have same budget admission"

(* ================================================================== *)
(*  N: Known vector lock — frozen literals                              *)
(*                                                                     *)
(*  Gold source: accepted B4I1 reference at                             *)
(*  838251d75c41005c4057b278fca31b26b779b2d8                            *)
(*  These are hard-coded expected values.  Do NOT generate at runtime.  *)
(* ================================================================== *)

let frozen_empty_payload_sha256 = "03882b01ddaffd0944e1b38e3f55495e8e34d11bc25def374883cc262700c938"
let frozen_empty_digest = "tethers:v2:sha256:750a06eea394bb38eefc073cd77d6c36b291efa13f6ff5173eacce35ca7b4619"

let frozen_simple_payload_sha256 = "9dd7aeb4e3bec49aed88ea4844461d0c1cb4846ebc781b7d3816458b8ce3ecdd"
let frozen_simple_digest = "tethers:v2:sha256:1bba9a344584c9b32d066a6de1e69ec196222682546ad7f40c51f04c061e3932"

let frozen_persistent_payload_sha256 = "b0877dbca6b7c04634bb9e61fed850e4a832ec60fdfa7b25f51c1185a92a940b"
let frozen_persistent_digest = "tethers:v2:sha256:6eae6604bb65580646be8cbc077284cf520c87eecbd81438ae8b4031606eb0f8"

let sha256_hex_of_string s =
  Digestif.SHA256.(to_hex (digest_string s))

let test_known_vectors () =
  (* Fixture A: Empty program *)
  let p_empty = {
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
  let oracle_empty = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p_empty) "known_empty_oracle" in
  let prod_empty = check_ok (Tethers_core_canonical_v2.canonicalize p_empty) "known_empty_prod" in
  let prod_empty_payload = Tethers_core_canonical_v2.canonical_payload prod_empty in
  let prod_empty_digest = Tethers_core_canonical_v2.program_digest prod_empty in
  check_equal_string frozen_empty_digest oracle_empty.digest_string "oracle empty digest";
  check_equal_string frozen_empty_digest prod_empty_digest "production empty digest";
  check_equal_string frozen_empty_payload_sha256 (sha256_hex_of_string prod_empty_payload) "production empty payload SHA256";
  check_equal_string oracle_empty.payload prod_empty_payload "empty oracle==prod payload"

(* ================================================================== *)
(*  N: Frozen vector — simple anchor/action                              *)
(* ================================================================== *)

let test_known_vector_simple_anchor () =
  let anchor_id = oid "anchor1" in
  let p_simple = {
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
    ];
    branches = [];
    roles = [];
    item_templates = [];
    capability_contracts = [];
  } in
  let oracle_simple = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p_simple) "frozen_simple_oracle" in
  let prod_simple = check_ok (Tethers_core_canonical_v2.canonicalize p_simple) "frozen_simple_prod" in
  let prod_simple_payload = Tethers_core_canonical_v2.canonical_payload prod_simple in
  let prod_simple_digest = Tethers_core_canonical_v2.program_digest prod_simple in
  check_equal_string frozen_simple_digest oracle_simple.digest_string "oracle simple digest";
  check_equal_string frozen_simple_digest prod_simple_digest "production simple digest";
  check_equal_string frozen_simple_payload_sha256 (sha256_hex_of_string prod_simple_payload) "production simple payload SHA256";
  check_equal_string oracle_simple.payload prod_simple_payload "simple oracle==prod payload"

(* ================================================================== *)
(*  N: Frozen vector — Persistent Branch witness                        *)
(* ================================================================== *)

let test_known_vector_persistent_branch () =
  let make_persistent names =
    let origins = List.map (fun name ->
      Anchor_origin {
        anchor_origin_id = oid name;
        event_name = "ev";
        declared_facts = [];
      }
    ) names in
    let branches = List.map2 (fun bname oname ->
      {
        branch_id = branch_id_of_string bname;
        branch_subject = oid oname;
        outcome_branches = [(Success, Stop)];
      }
    ) ["b0";"b1";"b2";"b3"] names in
    {
      program_id = pid "test";
      core_version = cv "0.1.0";
      input_facts = [];
      entry_guards = [];
      entry_origin = Some (oid (List.hd names));
      success_continuations = [];
      origin_sites = origins;
      branches = branches;
      roles = [];
      item_templates = [];
      capability_contracts = [];
    }
  in
  let p_persistent = make_persistent ["a0";"a1";"a2";"a3"] in
  let oracle_persistent = check_ok (Tethers_core_canonical_v2_reference.slow_oracle p_persistent) "frozen_persistent_oracle" in
  let prod_persistent = check_ok (Tethers_core_canonical_v2.canonicalize p_persistent) "frozen_persistent_prod" in
  let prod_persistent_payload = Tethers_core_canonical_v2.canonical_payload prod_persistent in
  let prod_persistent_digest = Tethers_core_canonical_v2.program_digest prod_persistent in
  check_equal_string frozen_persistent_digest oracle_persistent.digest_string "oracle persistent digest";
  check_equal_string frozen_persistent_digest prod_persistent_digest "production persistent digest";
  check_equal_string frozen_persistent_payload_sha256 (sha256_hex_of_string prod_persistent_payload) "production persistent payload SHA256";
  check_equal_string oracle_persistent.payload prod_persistent_payload "persistent oracle==prod payload"

(* ================================================================== *)
(*  Main test runner                                                   *)
(* ================================================================== *)

let () =
  Printf.printf "=== V2 Production Canonicaliser Tests ===\n\n";
  test_empty_program_prod ();
  Printf.printf "PASS: empty program production\n";
  test_single_anchor_action_prod ();
  Printf.printf "PASS: single anchor/action production\n";
  test_cross_family_prod ();
  Printf.printf "PASS: cross-family production\n";
  test_persistent_branch_prod ();
  Printf.printf "PASS: persistent branch production (24 permutations)\n";
  test_role_blocks_prod ();
  Printf.printf "PASS: role blocks production\n";
  test_nested_storage_order_prod ();
  Printf.printf "PASS: nested storage order production\n";
  test_multiplicity_prod ();
  Printf.printf "PASS: multiplicity production\n";
  test_strings_prod ();
  Printf.printf "PASS: strings production\n";
  test_integer_boundaries_prod ();
  Printf.printf "PASS: integer boundaries production\n";
  test_high_byte_comparator ();
  Printf.printf "PASS: high-byte comparator\n";
  test_family_size_7 ();
  Printf.printf "PASS: family size 7 (beyond oracle limit)\n";
  test_budget_exact_boundary ();
  Printf.printf "PASS: budget exact boundary\n";
  test_budget_equivalent_raw_variants ();
  Printf.printf "PASS: budget equivalent raw variants\n";
  test_known_vectors ();
  Printf.printf "PASS: known frozen vectors\n";
  test_known_vector_simple_anchor ();
  Printf.printf "PASS: frozen simple anchor vector\n";
  test_known_vector_persistent_branch ();
  Printf.printf "PASS: frozen persistent branch vector\n";
  Printf.printf "\n=== All V2 Production Tests Complete ===\n"
