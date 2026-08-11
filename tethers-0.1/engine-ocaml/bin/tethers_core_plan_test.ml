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
  | Incomplete_success_path _ -> "Incomplete_success_path"
  | Unsupported_together -> "Unsupported_together"
  | Unsupported_batch -> "Unsupported_batch"
  | Unsupported_branch -> "Unsupported_branch"
  | Unsupported_role_binding -> "Unsupported_role_binding"
  | Unsupported_role_proxy -> "Unsupported_role_proxy"
  | Unsupported_fact_binding -> "Unsupported_fact_binding"
  | Unsupported_anchor_value -> "Unsupported_anchor_value"
  | Unsupported_execution_constraint -> "Unsupported_execution_constraint"
  | Unsupported_item_template -> "Unsupported_item_template"
  | Missing_capability_projection _ -> "Missing_capability_projection"
  | Capability_projection_identity_mismatch _ ->
      "Capability_projection_identity_mismatch"
  | Capability_projection_digest_mismatch _ ->
      "Capability_projection_digest_mismatch"
  | Capability_projection_incomplete _ -> "Capability_projection_incomplete"
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

(* ------------------------------------------------------------------ *)
(*  Runtime planning context helpers                                   *)
(* ------------------------------------------------------------------ *)

let mk_projection cap_id_str digest ?(name="") ?(version="1.0.0")
    ?(effects=[]) ?(manifest_digest=None) ?(bridge_capability_version=None)
    ?(bridge_provider_identity=None) () =
  let open Tethers_protocol in
  { capability_id = cid cap_id_str;
    contract_digest = capability_contract_digest_of_string digest;
    runtime =
      { name;
        version;
        inputs = [];
        effects;
        manifest_digest;
        bridge_capability_version;
        bridge_provider_identity } }

let mk_context ?(evaluation_id="eval_1") ?(capabilities=[]) () =
  { evaluation_id; capabilities }

let action_field name action = Yojson.Safe.Util.member name action

(* ================================================================== *)
(*  T1 — Explicit completion: A → Program_complete                     *)
(* ================================================================== *)

let test_explicit_completion () =
  let program = mk_program
    ~id:"P_t1"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "file.received" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ mk_lit_input "message" (String_value "start") ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_t1"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  let p = assert_ok_plan "T1 plan" (plan program ctx) in
  assert_true "T1 one action" (List.length p.actions = 1);
  assert_true "T1 occurrence id" (p.id = "eval_t1/plan");
  (match p.actions with
   | [ action ] ->
       assert_true "T1 capability identity"
         (action_field "capability" action = `String "cap.notify");
       assert_true "T1 action_id"
         (action_field "action_id" action = `String "action_1")
   | _ -> assert_true "T1 single-action shape" false)

(* ================================================================== *)
(*  T2 — Missing terminal continuation fails closed                    *)
(* ================================================================== *)

let test_missing_terminal_continuation () =
  let program = mk_program
    ~id:"P_t2"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "file.received" [];
      mk_action_origin "O_b" "cap.save" "sha256:def"
        [ mk_lit_input "file" (String_value "report.pdf") ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_b"));
    ]
    ~capability_contracts:[ mk_cap_contract "cap.save" "sha256:def" ]
    ()
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_t2"
      ~capabilities:[ mk_projection "cap.save" "sha256:def" ~name:"cap.save" () ]
      ()
  in
  assert_plan_error (Incomplete_success_path (oid "O_b"))
    "T2 missing terminal continuation fails" (plan program ctx)

(* ================================================================== *)
(*  T3 — Runtime occurrence identity, not ProgramId                    *)
(* ================================================================== *)

let test_runtime_occurrence_identity () =
  let program = mk_program
    ~id:"MY_PROGRAM"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "file.received" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ mk_lit_input "message" (String_value "start") ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_123"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  let p = assert_ok_plan "T3 plan" (plan program ctx) in
  assert_true "T3 occurrence id = eval_123/plan" (p.id = "eval_123/plan");
  assert_true "T3 occurrence id is not program_id" (p.id <> "MY_PROGRAM")

(* ================================================================== *)
(*  T4 — ProgramId does not become occurrence identity                 *)
(* ================================================================== *)

let test_program_id_not_occurrence () =
  let build id =
    mk_program
      ~id
      ~entry_origin:(Some (oid "O_anchor"))
      ~origin_sites:[
        mk_anchor_origin "O_anchor" "file.received" [];
        mk_action_origin "O_action" "cap.notify" "sha256:abc"
          [ mk_lit_input "message" (String_value "start") ] [];
      ]
      ~success_continuations:[
        mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
        mk_success_cont "O_action" Program_complete;
      ]
      ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
      ()
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_occ"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  let p1 = assert_ok_plan "T4 plan alpha" (plan (build "P_alpha") ctx) in
  let p2 = assert_ok_plan "T4 plan beta" (plan (build "P_beta") ctx) in
  assert_true "T4 occurrence ids equal"
    (p1.id = p2.id && p1.id = "eval_occ/plan")

(* ================================================================== *)
(*  T5 — Existing Runtime Plan Action contract                         *)
(* ================================================================== *)

let test_existing_action_shape () =
  let program = mk_program
    ~id:"P_t5"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_send" "cap.send" "sha256:exact-digest"
        [ mk_lit_input "to" (String_value "lucy@example.com") ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_send"));
      mk_success_cont "O_send" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.send" "sha256:exact-digest" ]
    ()
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_t5"
      ~capabilities:[
        mk_projection "cap.send" "sha256:exact-digest"
          ~name:"cap.send" ~version:"1.0.0"
          ~effects:[ "email.send" ]
          ~manifest_digest:(Some "sha256:manifest-0001")
          ~bridge_capability_version:(Some 1)
          ~bridge_provider_identity:(Some "mail-provider-1") ();
      ]
      ()
  in
  let p = assert_ok_plan "T5 plan" (plan program ctx) in
  (match p.actions with
   | [ action ] ->
       assert_true "T5 action_id"
         (action_field "action_id" action = `String "action_1");
       assert_true "T5 idempotency_key"
         (action_field "idempotency_key" action = `String "eval_t5/action_1");
       assert_true "T5 capability"
         (action_field "capability" action = `String "cap.send");
       assert_true "T5 capability_version"
         (action_field "capability_version" action = `String "1.0.0");
       assert_true "T5 arguments"
         (action_field "arguments" action = `Assoc [ ("to", `String "lucy@example.com") ]);
       assert_true "T5 effects"
         (action_field "effects" action = `List [ `String "email.send" ]);
       assert_true "T5 manifest_digest"
         (action_field "manifest_digest" action = `String "sha256:manifest-0001");
       assert_true "T5 bridge_capability_version"
         (action_field "bridge_capability_version" action = `Int 1);
       assert_true "T5 bridge_provider_identity"
         (action_field "bridge_provider_identity" action = `String "mail-provider-1")
   | _ -> assert_true "T5 single-action shape" false)

(* ================================================================== *)
(*  T6 — Capability projection missing                                 *)
(* ================================================================== *)

let test_capability_projection_missing () =
  let program = mk_program
    ~id:"P_t6"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "ev" [];
      mk_action_origin "O_action" "cap.a" "sha256:a"
        [ mk_lit_input "x" (String_value "1") ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.a" "sha256:a" ]
    ()
  in
  let ctx = mk_context ~evaluation_id:"eval_t6" ~capabilities:[] () in
  assert_plan_error (Missing_capability_projection (cid "cap.a"))
    "T6 projection missing" (plan program ctx)

(* ================================================================== *)
(*  T7 — Capability digest mismatch                                    *)
(* ================================================================== *)

let test_capability_digest_mismatch () =
  let program = mk_program
    ~id:"P_t7"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "ev" [];
      mk_action_origin "O_action" "cap.a" "sha256:a"
        [ mk_lit_input "x" (String_value "1") ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.a" "sha256:a" ]
    ()
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_t7"
      ~capabilities:[ mk_projection "cap.a" "sha256:WRONG" ~name:"cap.a" () ]
      ()
  in
  assert_plan_error (Capability_projection_digest_mismatch (cid "cap.a"))
    "T7 digest mismatch" (plan program ctx)

(* ================================================================== *)
(*  T8 — Effects aggregation with deterministic uniqueness             *)
(* ================================================================== *)

let test_effects_aggregation () =
  let program = mk_program
    ~id:"P_t8"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "ev" [];
      mk_action_origin "O_a" "cap.notify" "sha256:abc"
        [ mk_lit_input "message" (String_value "start") ] [];
      mk_action_origin "O_b" "cap.save" "sha256:def"
        [ mk_lit_input "file" (String_value "report.pdf") ] [];
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
  let ctx =
    mk_context
      ~evaluation_id:"eval_t8"
      ~capabilities:[
        mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify"
          ~effects:[ "filesystem.read"; "mail.send" ] ();
        mk_projection "cap.save" "sha256:def" ~name:"cap.save"
          ~effects:[ "mail.send"; "filesystem.write" ] ();
      ]
      ()
  in
  let p = assert_ok_plan "T8 plan" (plan program ctx) in
  assert_true "T8 unique first-occurrence effects"
    (p.required_effects = [ "filesystem.read"; "mail.send"; "filesystem.write" ])

(* ================================================================== *)
(*  T9 — Idempotency keys                                              *)
(* ================================================================== *)

let test_idempotency_keys () =
  let program = mk_program
    ~id:"P_t9"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "ev" [];
      mk_action_origin "O_a" "cap.a" "sha256:a"
        [ mk_lit_input "x" (String_value "1") ] [];
      mk_action_origin "O_b" "cap.b" "sha256:b"
        [ mk_lit_input "y" (String_value "2") ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_a"));
      mk_success_cont "O_a" (Origin_target (oid "O_b"));
      mk_success_cont "O_b" Program_complete;
    ]
    ~capability_contracts:[
      mk_cap_contract "cap.a" "sha256:a";
      mk_cap_contract "cap.b" "sha256:b";
    ]
    ()
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_X"
      ~capabilities:[
        mk_projection "cap.a" "sha256:a" ~name:"cap.a" ();
        mk_projection "cap.b" "sha256:b" ~name:"cap.b" ();
      ]
      ()
  in
  let p = assert_ok_plan "T9 plan" (plan program ctx) in
  (match p.actions with
   | [ a; b ] ->
       assert_true "T9 action_1 key"
         (action_field "idempotency_key" a = `String "eval_X/action_1");
       assert_true "T9 action_2 key"
         (action_field "idempotency_key" b = `String "eval_X/action_2")
   | _ -> assert_true "T9 two-action shape" false)

(* ================================================================== *)
(*  T10 — Storage-order independence preserved                         *)
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
  let ctx =
    mk_context
      ~evaluation_id:"eval_t10"
      ~capabilities:[
        mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" ();
        mk_projection "cap.save" "sha256:def" ~name:"cap.save" ();
      ]
      ()
  in
  let p1 = assert_ok_plan "T10 plan forward" (plan (build false) ctx) in
  let p2 = assert_ok_plan "T10 plan reversed" (plan (build true) ctx) in
  assert_true "T10 storage order does not change plan" (p1 = p2)

(* ================================================================== *)
(*  T11 — Existing CORE-5A fail-closed behaviour preserved             *)
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
  let ctx = mk_context ~evaluation_id:"eval_t11" () in
  assert_plan_error Unsupported_together "T11 together fails closed"
    (plan program ctx)

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
  let ctx = mk_context ~evaluation_id:"eval_t11" () in
  assert_plan_error Unsupported_batch "T11 batch fails closed"
    (plan program ctx)

let test_unsupported_branch () =
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
  let ctx = mk_context ~evaluation_id:"eval_t11" () in
  assert_plan_error Unsupported_branch "T11 branch fails closed"
    (plan program ctx)

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
  let ctx = mk_context ~evaluation_id:"eval_t11" () in
  assert_plan_error Unsupported_role_binding "T11 role binding fails closed"
    (plan program ctx)

let test_unsupported_anchor_value () =
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
  let ctx = mk_context ~evaluation_id:"eval_t11" () in
  assert_plan_error Unsupported_anchor_value
    "T11 Anchor_value fails closed" (plan program ctx)

let test_unsupported_fact_from_origin () =
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
  let ctx = mk_context ~evaluation_id:"eval_t11" () in
  assert_plan_error Unsupported_fact_binding
    "T11 Fact_from_origin fails closed" (plan program ctx)

let test_unsupported_execution_constraint () =
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
  let ctx = mk_context ~evaluation_id:"eval_t11" () in
  assert_plan_error Unsupported_execution_constraint
    "T11 Deadline fails closed" (plan program ctx)

let test_unsupported_item_template () =
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
  let ctx = mk_context ~evaluation_id:"eval_t11" () in
  assert_plan_error Unsupported_item_template
    "T11 item template fails closed" (plan program ctx)

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
  let ctx = mk_context ~evaluation_id:"eval_t11" () in
  match plan program ctx with
  | Error (Invalid_core _) -> incr tests_run; incr tests_passed
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: T11 invalid core (expected Invalid_core, got %s)\n"
        (string_of_planning_error err);
      exit 1
  | Ok _ ->
      incr tests_run;
      Printf.eprintf "FAIL: T11 invalid core (expected Error, got Ok)\n";
      exit 1

(* ================================================================== *)
(*  T12 — Capability identity mismatch fails closed                    *)
(* ================================================================== *)

let test_capability_identity_mismatch () =
  let program = mk_program
    ~id:"P_t12"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "ev" [];
      mk_action_origin "O_action" "cap.a" "sha256:req"
        [ mk_lit_input "x" (String_value "1") ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.a" "sha256:req" ]
    ()
  in
  (* The digest is approved, but only under a different capability identity. *)
  let ctx =
    mk_context
      ~evaluation_id:"eval_t12"
      ~capabilities:[ mk_projection "cap.other" "sha256:req" ~name:"cap.other" () ]
      ()
  in
  assert_plan_error (Capability_projection_identity_mismatch (cid "cap.a"))
    "T12 identity mismatch fails closed" (plan program ctx)

(* ================================================================== *)
(*  T13 — Incomplete projection metadata fails closed                  *)
(* ================================================================== *)

let test_capability_projection_incomplete () =
  let program = mk_program
    ~id:"P_t13"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "ev" [];
      mk_action_origin "O_action" "cap.a" "sha256:a"
        [ mk_lit_input "x" (String_value "1") ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.a" "sha256:a" ]
    ()
  in
  (* Projection present and digest matches, but runtime name is empty. *)
  let ctx =
    mk_context
      ~evaluation_id:"eval_t13"
      ~capabilities:[ mk_projection "cap.a" "sha256:a" ~name:"" () ]
      ()
  in
  assert_plan_error (Capability_projection_incomplete (cid "cap.a"))
    "T13 incomplete projection fails closed" (plan program ctx)

(* ================================================================== *)
(*  RUN ALL TESTS                                                       *)
(* ================================================================== *)

let () =
  test_explicit_completion ();
  test_missing_terminal_continuation ();
  test_runtime_occurrence_identity ();
  test_program_id_not_occurrence ();
  test_existing_action_shape ();
  test_capability_projection_missing ();
  test_capability_digest_mismatch ();
  test_effects_aggregation ();
  test_idempotency_keys ();
  test_storage_order_independence ();
  test_unsupported_together ();
  test_unsupported_batch ();
  test_unsupported_branch ();
  test_unsupported_role_binding ();
  test_unsupported_anchor_value ();
  test_unsupported_fact_from_origin ();
  test_unsupported_execution_constraint ();
  test_unsupported_item_template ();
  test_invalid_core ();
  test_capability_identity_mismatch ();
  test_capability_projection_incomplete ();
  Printf.printf "PASS all plan bridge tests (%d/%d)\n" !tests_passed !tests_run
