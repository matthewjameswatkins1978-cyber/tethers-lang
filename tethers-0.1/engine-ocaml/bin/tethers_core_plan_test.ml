open Tethers_core
open Tethers_core_plan

(* ================================================================== *)
(*  Test harness                                                        *)
(* ================================================================== *)

let tests_run = ref 0
let tests_passed = ref 0

let assert_true msg condition =
  incr tests_run;
  if condition then incr tests_passed
  else begin
    Printf.eprintf "FAIL: %s\n" msg;
    exit 1
  end

let string_of_planning_error = function
  | Invalid_core _ -> "Invalid_core"
  | Missing_entry_origin -> "Missing_entry_origin"
  | Unsupported_together -> "Unsupported_together"
  | Unsupported_batch -> "Unsupported_batch"
  | Unsupported_branch -> "Unsupported_branch"
  | Unsupported_role_binding -> "Unsupported_role_binding"
  | Unsupported_role_proxy -> "Unsupported_role_proxy"
  | Unsupported_fact_binding -> "Unsupported_fact_binding"
  | Unsupported_anchor_value -> "Unsupported_anchor_value"
  | Unsupported_execution_constraint -> "Unsupported_execution_constraint"
  | Unsupported_item_template -> "Unsupported_item_template"
  | Flow_cycle _ -> "Flow_cycle"
  | Unresolved_origin _ -> "Unresolved_origin"

let assert_ok_plan msg = function
  | Ok plan -> incr tests_run; incr tests_passed; plan
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: %s (expected Ok, got Error %s)\n" msg
        (string_of_planning_error err);
      exit 1

let assert_plan_error expected msg = function
  | Error err when err = expected -> incr tests_run; incr tests_passed
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: %s (expected %s, got %s)\n" msg
        (string_of_planning_error expected) (string_of_planning_error err);
      exit 1
  | Ok _ ->
      incr tests_run;
      Printf.eprintf "FAIL: %s (expected Error %s, got Ok)\n" msg
        (string_of_planning_error expected);
      exit 1

(* ================================================================== *)
(*  Core program construction helpers                                  *)
(* ================================================================== *)

let oid s = origin_id_of_string s
let fid s = fact_id_of_string s
let rid s = role_id_of_string s
let cid s = capability_id_of_string s
let gid s = group_id_of_string s
let btid s = batch_id_of_string s
let tid s = item_template_id_of_string s
let bid s = branch_id_of_string s
let hsk s = host_snapshot_key_of_string s

let mk_eval_fact fid_str key stype =
  { fact_id = fid fid_str;
    schema_description = "desc_" ^ fid_str;
    provenance = Evaluation_input (hsk key, stype) }

let mk_origin_fact fid_str oid_str =
  { fact_id = fid fid_str;
    schema_description = "desc_" ^ fid_str;
    provenance = Origin_provenance (oid oid_str) }

let mk_anchor_origin oid_str event_name facts =
  Anchor_origin { anchor_origin_id = oid oid_str; event_name; declared_facts = facts }

let mk_action_origin oid_str cap_id_str digest inputs facts =
  Action_origin { action_origin_id = oid oid_str;
                  capability_id = cid cap_id_str;
                  contract_digest = capability_contract_digest_of_string digest;
                  inputs;
                  declared_facts = facts;
                  execution_constraints = [] }

let mk_lit_input name value =
  { input_name = capability_input_name_of_string name;
    binding = Literal_value value }

let mk_cap_contract cap_id_str digest =
  { capability_id = cid cap_id_str;
    contract_digest = capability_contract_digest_of_string digest;
    schema_description = "cap desc" }

let mk_success_cont from_oid_str target =
  { from_origin = oid from_oid_str; target }

let mk_program ?(id="P_test") ?(core_version=core_version_of_string "0.1.0")
    ?(input_facts=[]) ?(entry_guards=[]) ?(entry_origin=None)
    ?(success_continuations=[]) ?(origin_sites=[]) ?(branches=[])
    ?(roles=[]) ?(item_templates=[]) ?(capability_contracts=[]) () =
  { program_id = program_id_of_string id;
    core_version;
    input_facts;
    entry_guards;
    entry_origin;
    success_continuations;
    origin_sites;
    branches;
    roles;
    item_templates;
    capability_contracts }

let action_field name action = Yojson.Safe.Util.member name action

(* ================================================================== *)
(*  T1 — Minimal Action                                                *)
(* ================================================================== *)

let test_minimal_action () =
  let program = mk_program
    ~id:"P_min"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "file.received" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ mk_lit_input "message" (String_value "start") ] []
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let p = assert_ok_plan "T1 plan" (plan program) in
  assert_true "T1 plan.id uses program_id" (p.id = "P_min");
  assert_true "T1 one action" (List.length p.actions = 1);
  assert_true "T1 empty required_effects" (p.required_effects = []);
  assert_true "T1 empty groups" (p.groups = []);
  (match p.actions with
   | [ action ] ->
       assert_true "T1 capability identity"
         (action_field "capability" action = `String "cap.notify");
       assert_true "T1 capability contract digest"
         (action_field "capability_contract_digest" action = `String "sha256:abc");
       assert_true "T1 literal inputs"
         (action_field "arguments" action = `Assoc [ ("message", `String "start") ]);
       assert_true "T1 sequential action_id"
         (action_field "action_id" action = `String "action_1")
   | _ -> assert_true "T1 single-action shape" false)

(* ================================================================== *)
(*  T2 — Sequential Two Actions                                        *)
(* ================================================================== *)

let test_sequential_two_actions () =
  let program = mk_program
    ~id:"P_two"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "file.received" [];
      mk_action_origin "O_a" "cap.notify" "sha256:abc"
        [ mk_lit_input "message" (String_value "start") ] [];
      mk_action_origin "O_b" "cap.save" "sha256:def"
        [ mk_lit_input "file" (String_value "report.pdf") ] []
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_a"));
      mk_success_cont "O_a" (Origin_target (oid "O_b"));
      mk_success_cont "O_b" Program_complete;
    ]
    ~capability_contracts:[
      mk_cap_contract "cap.notify" "sha256:abc";
      mk_cap_contract "cap.save" "sha256:def";
    ]
    ()
  in
  let p = assert_ok_plan "T2 plan" (plan program) in
  assert_true "T2 two actions" (List.length p.actions = 2);
  (match p.actions with
   | [ a; b ] ->
       assert_true "T2 A first in control-flow order"
         (action_field "capability" a = `String "cap.notify");
       assert_true "T2 B second in control-flow order"
         (action_field "capability" b = `String "cap.save");
       assert_true "T2 action_ids sequential"
         (action_field "action_id" a = `String "action_1"
          && action_field "action_id" b = `String "action_2")
   | _ -> assert_true "T2 two-action shape" false)

(* ================================================================== *)
(*  T3 — Storage Order Independence                                    *)
(* ================================================================== *)

let test_storage_order_independence () =
  let build reversed =
    let action_a =
      mk_action_origin "O_a" "cap.notify" "sha256:abc"
        [ mk_lit_input "message" (String_value "start") ] []
    in
    let action_b =
      mk_action_origin "O_b" "cap.save" "sha256:def"
        [ mk_lit_input "file" (String_value "report.pdf") ] []
    in
    let sites =
      if reversed then [ action_b; mk_anchor_origin "O_anchor" "file.received" []; action_a ]
      else [ mk_anchor_origin "O_anchor" "file.received" []; action_a; action_b ]
    in
    mk_program
      ~id:"P_store"
      ~entry_origin:(Some (oid "O_anchor"))
      ~origin_sites:sites
      ~success_continuations:[
        mk_success_cont "O_anchor" (Origin_target (oid "O_a"));
        mk_success_cont "O_a" (Origin_target (oid "O_b"));
        mk_success_cont "O_b" Program_complete;
      ]
      ~capability_contracts:[
        mk_cap_contract "cap.notify" "sha256:abc";
        mk_cap_contract "cap.save" "sha256:def";
      ]
      ()
  in
  let p1 = assert_ok_plan "T3 plan forward" (plan (build false)) in
  let p2 = assert_ok_plan "T3 plan reversed" (plan (build true)) in
  assert_true "T3 storage order does not change plan" (p1 = p2);
  assert_true "T3 control-flow order preserved"
    (match p1.actions with
     | [ a; b ] ->
         action_field "capability" a = `String "cap.notify"
         && action_field "capability" b = `String "cap.save"
     | _ -> false)

(* ================================================================== *)
(*  T4 — Capability Digest Preservation                                *)
(* ================================================================== *)

let test_capability_digest_preservation () =
  let program = mk_program
    ~id:"P_digest"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_send" "cap.send" "sha256:exact-digest-0123456789abcdef"
        [ mk_lit_input "to" (String_value "lucy@example.com") ] []
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_send"));
      mk_success_cont "O_send" Program_complete;
    ]
    ~capability_contracts:[
      mk_cap_contract "cap.send" "sha256:exact-digest-0123456789abcdef"
    ]
    ()
  in
  let p = assert_ok_plan "T4 plan" (plan program) in
  (match p.actions with
   | [ action ] ->
       assert_true "T4 CapabilityId preserved exactly"
         (action_field "capability" action = `String "cap.send");
       assert_true "T4 CapabilityContractDigest preserved exactly"
         (action_field "capability_contract_digest" action
          = `String "sha256:exact-digest-0123456789abcdef")
   | _ -> assert_true "T4 single-action shape" false)

(* ================================================================== *)
(*  T5 — Unsupported Together Fails Closed                             *)
(* ================================================================== *)

let test_unsupported_together () =
  let together =
    Together_origin
      { together_origin_id = oid "TG";
        group_id = gid "G1";
        member_origin_ids = [ oid "A"; oid "B" ];
        objective = All_members_succeed }
  in
  let program = mk_program
    ~id:"P_together"
    ~entry_origin:(Some (oid "ent"))
    ~origin_sites:[
      mk_anchor_origin "ent" "ev" [];
      mk_action_origin "A" "cap.a" "sha256:a"
        [ mk_lit_input "x" (String_value "a1") ] [];
      mk_action_origin "B" "cap.a" "sha256:a"
        [ mk_lit_input "x" (String_value "a2") ] [];
      together;
    ]
    ~success_continuations:[
      mk_success_cont "ent" (Origin_target (oid "A"));
      mk_success_cont "A" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.a" "sha256:a" ]
    ()
  in
  assert_plan_error Unsupported_together "T5 together fails closed"
    (plan program)

(* ================================================================== *)
(*  T6 — Unsupported Batch Fails Closed                                *)
(* ================================================================== *)

let test_unsupported_batch () =
  let batch =
    Batch_site
      { batch_id = btid "BAT1";
        collection_provenance = batch_collection_provenance_of_string "prov";
        item_template_id = tid "IT1";
        traversal_policy = batch_traversal_policy_of_string "pol";
        composite_objective = batch_objective_of_string "obj";
        aggregate_facts = [] }
  in
  let program = mk_program
    ~id:"P_batch"
    ~entry_origin:(Some (oid "ent"))
    ~origin_sites:[ mk_anchor_origin "ent" "ev" []; batch ]
    ~item_templates:[
      { item_template_id = tid "IT1";
        origin_sites = [];
        branches = [];
        roles = [
          { role_id = rid "R1";
            scope = Item_template_scope (tid "IT1");
            fact_contract = Role_fact_contract [];
            eligible_fulfillment = role_fulfillment_of_string "f" };
        ];
        objective = Required_role (rid "R1") };
    ]
    ~capability_contracts:[]
    ()
  in
  assert_plan_error Unsupported_batch "T6 batch fails closed"
    (plan program)

(* ================================================================== *)
(*  T7 — Unsupported Role Binding Fails Closed                         *)
(* ================================================================== *)

let test_unsupported_role_binding () =
  let role =
    { role_id = rid "R1";
      scope = Program_scope;
      fact_contract = Role_fact_contract [ fid "F_target" ];
      eligible_fulfillment = role_fulfillment_of_string "f" }
  in
  let program = mk_program
    ~id:"P_role"
    ~input_facts:[ mk_eval_fact "F_target" "hk1" String_type ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "event.ft" [];
      mk_action_origin "O_consumer" "cap.x" "sha256:d1"
        [ { input_name = capability_input_name_of_string "y";
            binding = Fact_through_role (fid "F_target", rid "R1") } ] []
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_consumer"));
      mk_success_cont "O_consumer" Program_complete;
    ]
    ~roles:[ role ]
    ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
    ()
  in
  assert_plan_error Unsupported_role_binding "T7 role binding fails closed"
    (plan program)

(* ================================================================== *)
(*  T8 — Invalid Core Cannot Plan                                      *)
(* ================================================================== *)

let test_invalid_core () =
  let program = mk_program
    ~id:"P_invalid"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "ev" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc" [] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[]  (* no contract for cap.notify -> invalid *)
    ()
  in
  match plan program with
  | Error (Invalid_core _) -> incr tests_run; incr tests_passed
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: T8 (expected Invalid_core, got %s)\n"
        (string_of_planning_error err);
      exit 1
  | Ok _ ->
      incr tests_run;
      Printf.eprintf "FAIL: T8 (expected Error, got Ok)\n";
      exit 1

(* ================================================================== *)
(*  T9 — Anchor_value binding fails closed                             *)
(* ================================================================== *)

let test_anchor_value_binding () =
  let program = mk_program
    ~id:"P_anchor_value"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "file.received" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ { input_name = capability_input_name_of_string "ref";
            binding = Anchor_value (oid "O_anchor", [ "document"; "name" ]) } ] []
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  assert_plan_error Unsupported_anchor_value
    "T9 Anchor_value fails closed" (plan program)

(* ================================================================== *)
(*  T10 — Fact_from_origin binding fails closed                        *)
(* ================================================================== *)

let test_fact_from_origin_binding () =
  let action1 =
    Action_origin
      { action_origin_id = oid "O1";
        capability_id = cid "cap.notify";
        contract_digest = capability_contract_digest_of_string "sha256:abc";
        inputs = [];
        declared_facts = [ mk_origin_fact "F_a" "O1" ];
        execution_constraints = [] }
  in
  let action2 =
    mk_action_origin "O2" "cap.save" "sha256:def"
      [ { input_name = capability_input_name_of_string "v";
          binding = Fact_from_origin (fid "F_a", oid "O1") } ] []
  in
  let program = mk_program
    ~id:"P_fact_from_origin"
    ~entry_origin:(Some (oid "O1"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "ev" [];
      action1;
      action2;
    ]
    ~success_continuations:[
      mk_success_cont "O1" (Origin_target (oid "O2"));
      mk_success_cont "O2" Program_complete;
    ]
    ~capability_contracts:[
      mk_cap_contract "cap.notify" "sha256:abc";
      mk_cap_contract "cap.save" "sha256:def";
    ]
    ()
  in
  assert_plan_error Unsupported_fact_binding
    "T10 Fact_from_origin fails closed" (plan program)

(* ================================================================== *)
(*  T11 — Branch semantics fails closed                                *)
(* ================================================================== *)

let test_branch_semantics () =
  let program = mk_program
    ~id:"P_branch"
    ~entry_origin:(Some (oid "oa"))
    ~origin_sites:[
      mk_anchor_origin "oa" "ev" [];
      mk_action_origin "ob" "cap.x" "sha256:d1"
        [ mk_lit_input "x" (String_value "v") ] []
    ]
    ~branches:[
      { branch_id = bid "B1";
        branch_subject = oid "oa";
        outcome_branches =
          [ (Success, Continue_to (oid "ob")); (Failure, Stop) ] };
    ]
    ~success_continuations:[
      mk_success_cont "oa" (Origin_target (oid "ob"));
      mk_success_cont "ob" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
    ()
  in
  assert_plan_error Unsupported_branch "T11 branch fails closed"
    (plan program)

(* ================================================================== *)
(*  T12 — Execution constraint (Deadline) fails closed                 *)
(* ================================================================== *)

let test_execution_constraint () =
  let action =
    Action_origin
      { action_origin_id = oid "O_action";
        capability_id = cid "cap.notify";
        contract_digest = capability_contract_digest_of_string "sha256:abc";
        inputs = [ mk_lit_input "message" (String_value "start") ];
        declared_facts = [];
        execution_constraints = [ Deadline "PT5M" ] }
  in
  let program = mk_program
    ~id:"P_deadline"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[ mk_anchor_origin "O_anchor" "ev" []; action ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  assert_plan_error Unsupported_execution_constraint
    "T12 Deadline fails closed" (plan program)

(* ================================================================== *)
(*  T13 — Item templates fail closed                                   *)
(* ================================================================== *)

let test_item_template () =
  let program = mk_program
    ~id:"P_item_template"
    ~entry_origin:(Some (oid "ent"))
    ~origin_sites:[ mk_anchor_origin "ent" "ev" [] ]
    ~item_templates:[
      { item_template_id = tid "IT1";
        origin_sites = [];
        branches = [];
        roles = [
          { role_id = rid "R1";
            scope = Item_template_scope (tid "IT1");
            fact_contract = Role_fact_contract [];
            eligible_fulfillment = role_fulfillment_of_string "f" };
        ];
        objective = Required_role (rid "R1") };
    ]
    ~capability_contracts:[]
    ()
  in
  assert_plan_error Unsupported_item_template
    "T13 item template fails closed" (plan program)

(* ================================================================== *)
(*  T14 — Missing entry_origin                                         *)
(* ================================================================== *)

let test_missing_entry () =
  let program = mk_program
    ~id:"P_no_entry"
    ~origin_sites:[ mk_anchor_origin "O_anchor" "ev" [] ]
    ()
  in
  assert_plan_error Missing_entry_origin "T14 missing entry fails closed"
    (plan program)

(* ================================================================== *)
(*  RUN ALL TESTS                                                       *)
(* ================================================================== *)

let () =
  test_minimal_action ();
  test_sequential_two_actions ();
  test_storage_order_independence ();
  test_capability_digest_preservation ();
  test_unsupported_together ();
  test_unsupported_batch ();
  test_unsupported_role_binding ();
  test_invalid_core ();
  test_anchor_value_binding ();
  test_fact_from_origin_binding ();
  test_branch_semantics ();
  test_execution_constraint ();
  test_item_template ();
  test_missing_entry ();
  Printf.printf "PASS all plan bridge tests (%d/%d)\n" !tests_passed !tests_run
