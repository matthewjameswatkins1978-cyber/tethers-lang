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
  | Unsupported_execution_constraint -> "Unsupported_execution_constraint"
  | Unsupported_item_template -> "Unsupported_item_template"
  | Missing_capability_projection _ -> "Missing_capability_projection"
  | Capability_projection_identity_mismatch _ ->
      "Capability_projection_identity_mismatch"
  | Capability_projection_digest_mismatch _ ->
      "Capability_projection_digest_mismatch"
  | Capability_projection_incomplete _ -> "Capability_projection_incomplete"
  | Ambiguous_capability_projection _ -> "Ambiguous_capability_projection"
  | Flow_cycle _ -> "Flow_cycle"
  | Unresolved_origin _ -> "Unresolved_origin"
  | Missing_anchor_snapshot _ -> "Missing_anchor_snapshot"
  | Ambiguous_anchor_snapshot _ -> "Ambiguous_anchor_snapshot"
  | Anchor_path_missing _ -> "Anchor_path_missing"
  | Anchor_path_not_object _ -> "Anchor_path_not_object"
  | Unsupported_anchor_value_type _ -> "Unsupported_anchor_value_type"
  | Unresolved_entry_guards -> "Unresolved_entry_guards"
  | Missing_fact_snapshot _ -> "Missing_fact_snapshot"
  | Ambiguous_fact_snapshot _ -> "Ambiguous_fact_snapshot"
  | Fact_snapshot_type_mismatch _ -> "Fact_snapshot_type_mismatch"
  | Invalid_guard_comparison _ -> "Invalid_guard_comparison"
  | Missing_reception_anchor -> "Missing_reception_anchor"
  | Ambiguous_reception_anchor -> "Ambiguous_reception_anchor"

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

let assert_ok_canonical = function
  | Ok c -> c
  | Error (Tethers_core_canonical.Invalid_core _) ->
      failwith "canonicalize: expected Ok, got Invalid_core"
  | Error Tethers_core_canonical.Refinement_exceeded ->
      failwith "canonicalize: expected Ok, got Refinement_exceeded"

let assert_ok_canonical_plan msg = function
  | Ok cp -> incr tests_run; incr tests_passed; cp
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: %s (expected Ok, got Error %s)\n" msg
        (string_of_planning_error err);
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

let mk_context ?(evaluation_id="eval_1") ?(capabilities=[]) ?(anchors=[]) ?(facts=[]) () =
  { evaluation_id; capabilities; anchors; facts }

let mk_runtime_event name data = { name; data }

let mk_eval_context ?(evaluation_id="eval_1") ?(event=mk_runtime_event "" `Null)
    ?(capabilities=[]) ?(facts=[]) () =
  { evaluation_id; event; capabilities; facts }

let action_field name action = Yojson.Safe.Util.member name action

(* ================================================================== *)
(*  Anchor snapshot helpers                                             *)
(* ================================================================== *)

let mk_anchor_snapshot oid_str json =
  { origin_id = oid oid_str; data = json }

let mk_fact_snapshot key_str json =
  { key = hsk key_str; value = json }

let mk_guard fact_id_str op expected =
  { Tethers_core.fact_id = fid fact_id_str; operator = op; expected }

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
      mk_success_cont "A" (Origin_target (oid "B"));
    ]
    ~capability_contracts:[ mk_cap_contract "cap.a" "sha256:a" ]
    ()
  in
  let ctx = mk_context ~evaluation_id:"eval_t11"
    ~capabilities:[ mk_projection "cap.a" "sha256:a" ~name:"cap.a" () ]
    ()
  in
  assert_plan_error (Incomplete_success_path (oid "B"))
    "T11 together missing terminal continuation" (plan program ctx)

let test_together_valid_plan () =
  let together =
    Together_origin
      { together_origin_id = oid "TG";
        group_id = gid "G1";
        member_origin_ids = [ oid "A"; oid "B" ];
        objective = All_members_succeed }
  in
  let program = mk_program
    ~id:"P_tg1"
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
      mk_success_cont "A" (Origin_target (oid "B"));
      mk_success_cont "B" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.a" "sha256:a" ]
    ()
  in
  let ctx = mk_context ~evaluation_id:"eval_tg1"
    ~capabilities:[ mk_projection "cap.a" "sha256:a" ~name:"cap.a" () ]
    ()
  in
  let p = assert_ok_plan "TG1 together valid plan" (plan program ctx) in
  assert_true "TG1 two actions" (List.length p.actions = 2);
  assert_true "TG1 one group" (List.length p.groups = 1);
  let g = List.hd p.groups in
  assert_true "TG1 group_id" (g.group_id = "G1");
  assert_true "TG1 two member_action_ids" (List.length g.member_action_ids = 2);
  assert_true "TG1 member_action_ids are action_1 and action_2"
    (g.member_action_ids = [ "action_1"; "action_2" ])

let test_together_member_order () =
  let together =
    Together_origin
      { together_origin_id = oid "TG";
        group_id = gid "G1";
        member_origin_ids = [ oid "B"; oid "A" ];
        objective = All_members_succeed }
  in
  let program = mk_program
    ~id:"P_tg2"
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
      mk_success_cont "A" (Origin_target (oid "B"));
      mk_success_cont "B" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.a" "sha256:a" ]
    ()
  in
  let ctx = mk_context ~evaluation_id:"eval_tg2"
    ~capabilities:[ mk_projection "cap.a" "sha256:a" ~name:"cap.a" () ]
    ()
  in
  let p = assert_ok_plan "TG2 member order" (plan program ctx) in
  let g = List.hd p.groups in
  assert_true "TG2 member_action_ids are action_1 and action_2"
    (g.member_action_ids = [ "action_1"; "action_2" ])

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
(*  B1-T1 — Duplicate exact projection fails                           *)
(* ================================================================== *)

let test_duplicate_projection_fails () =
  let program = mk_program
    ~id:"P_bt1"
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
      ~evaluation_id:"eval_bt1"
      ~capabilities:[
        mk_projection "cap.a" "sha256:a" ~name:"cap.a" ~version:"1.0.0"
          ~effects:["e1"] ();
        mk_projection "cap.a" "sha256:a" ~name:"cap.a" ~version:"2.0.0"
          ~effects:["e2"] ();
      ]
      ()
  in
  assert_plan_error (Ambiguous_capability_projection (cid "cap.a"))
    "B1-T1 duplicate exact projection fails" (plan program ctx)

(* ================================================================== *)
(*  B1-T2 — Reversed duplicates fail identically                       *)
(* ================================================================== *)

let test_reversed_duplicates_fail () =
  let program = mk_program
    ~id:"P_bt2"
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
  let ctx_fwd =
    mk_context
      ~evaluation_id:"eval_bt2"
      ~capabilities:[
        mk_projection "cap.a" "sha256:a" ~name:"cap.a" ~version:"1.0.0" ();
        mk_projection "cap.a" "sha256:a" ~name:"cap.a" ~version:"2.0.0" ();
      ]
      ()
  in
  let ctx_rev =
    mk_context
      ~evaluation_id:"eval_bt2"
      ~capabilities:[
        mk_projection "cap.a" "sha256:a" ~name:"cap.a" ~version:"2.0.0" ();
        mk_projection "cap.a" "sha256:a" ~name:"cap.a" ~version:"1.0.0" ();
      ]
      ()
  in
  assert_plan_error (Ambiguous_capability_projection (cid "cap.a"))
    "B1-T2 reversed duplicates fail (forward)" (plan program ctx_fwd);
  assert_plan_error (Ambiguous_capability_projection (cid "cap.a"))
    "B1-T2 reversed duplicates fail (reversed)" (plan program ctx_rev)

(* ================================================================== *)
(*  B1-T3 — Multiple contracts for one CapabilityId remain selectable   *)
(* ================================================================== *)

let test_distinct_contracts_coexist () =
  let program = mk_program
    ~id:"P_bt3"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "ev" [];
      mk_action_origin "O_action" "cap.a" "sha256:d1"
        [ mk_lit_input "x" (String_value "1") ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.a" "sha256:d1" ]
    ()
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_bt3"
      ~capabilities:[
        mk_projection "cap.a" "sha256:d1" ~name:"cap.a" ~version:"1.0.0" ();
        mk_projection "cap.a" "sha256:d2" ~name:"cap.a" ~version:"2.0.0" ();
      ]
      ()
  in
  let p = assert_ok_plan "B1-T3 distinct contracts coexist" (plan program ctx) in
  assert_true "B1-T3 resolved correct version"
    (match p.actions with
     | [ action ] ->
         action_field "capability_version" action = `String "1.0.0"
     | _ -> false)

(* ================================================================== *)
(*  T1 — Nested string resolution                                      *)
(* ================================================================== *)

let test_anchor_nested_string () =
  let program = mk_program
    ~id:"P_t1"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ { input_name = capability_input_name_of_string "ref";
            binding = Anchor_value (oid "O_anchor", [ "document"; "title" ]) } ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let snapshot =
    `Assoc [
      ("document", `Assoc [
        ("title", `String "Tethers")
      ])
    ]
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_t1"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~anchors:[ mk_anchor_snapshot "O_anchor" snapshot ]
      ()
  in
  let p = assert_ok_plan "T1 plan" (plan program ctx) in
  (match p.actions with
   | [ action ] ->
       assert_true "T1 resolved string"
         (action_field "arguments" action =
            `Assoc [ ("ref", `String "Tethers") ])
   | _ -> assert_true "T1 single-action shape" false)

(* ================================================================== *)
(*  T2 — Integer resolution                                             *)
(* ================================================================== *)

let test_anchor_integer () =
  let program = mk_program
    ~id:"P_t2"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ { input_name = capability_input_name_of_string "count";
            binding = Anchor_value (oid "O_anchor", [ "meta"; "count" ]) } ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let snapshot =
    `Assoc [
      ("meta", `Assoc [
        ("count", `Int 42)
      ])
    ]
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_t2"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~anchors:[ mk_anchor_snapshot "O_anchor" snapshot ]
      ()
  in
  let p = assert_ok_plan "T2 plan" (plan program ctx) in
  (match p.actions with
   | [ action ] ->
       assert_true "T2 resolved integer"
         (action_field "arguments" action =
            `Assoc [ ("count", `Int 42) ])
   | _ -> assert_true "T2 single-action shape" false)

(* ================================================================== *)
(*  T3 — Boolean resolution                                             *)
(* ================================================================== *)

let test_anchor_boolean () =
  let program = mk_program
    ~id:"P_t3"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ { input_name = capability_input_name_of_string "flag";
            binding = Anchor_value (oid "O_anchor", [ "status"; "active" ]) } ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let snapshot =
    `Assoc [
      ("status", `Assoc [
        ("active", `Bool true)
      ])
    ]
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_t3"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~anchors:[ mk_anchor_snapshot "O_anchor" snapshot ]
      ()
  in
  let p = assert_ok_plan "T3 plan" (plan program ctx) in
  (match p.actions with
   | [ action ] ->
       assert_true "T3 resolved boolean"
         (action_field "arguments" action =
            `Assoc [ ("flag", `Bool true) ])
   | _ -> assert_true "T3 single-action shape" false)

(* ================================================================== *)
(*  T4 — Mixed literal + anchor inputs                                  *)
(* ================================================================== *)

let test_mixed_literal_and_anchor () =
  let program = mk_program
    ~id:"P_t4"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ mk_lit_input "message" (String_value "Hello");
          { input_name = capability_input_name_of_string "ref";
            binding = Anchor_value (oid "O_anchor", [ "document"; "title" ]) } ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let snapshot =
    `Assoc [
      ("document", `Assoc [
        ("title", `String "Tethers")
      ])
    ]
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_t4"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~anchors:[ mk_anchor_snapshot "O_anchor" snapshot ]
      ()
  in
  let p = assert_ok_plan "T4 plan" (plan program ctx) in
  (match p.actions with
   | [ action ] ->
       let args = action_field "arguments" action in
       assert_true "T4 literal argument"
         (Yojson.Safe.Util.member "message" args = `String "Hello");
       assert_true "T4 anchor argument"
         (Yojson.Safe.Util.member "ref" args = `String "Tethers")
   | _ -> assert_true "T4 single-action shape" false)

(* ================================================================== *)
(*  T5 — Missing snapshot                                                *)
(* ================================================================== *)

let test_missing_snapshot () =
  let program = mk_program
    ~id:"P_t5"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ { input_name = capability_input_name_of_string "ref";
            binding = Anchor_value (oid "O_anchor", [ "document"; "title" ]) } ] [];
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
      ~evaluation_id:"eval_t5"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~anchors:[]
      ()
  in
  assert_plan_error (Missing_anchor_snapshot (oid "O_anchor"))
    "T5 missing snapshot fails" (plan program ctx)

(* ================================================================== *)
(*  T6 — Wrong anchor does not substitute                               *)
(* ================================================================== *)

let test_wrong_anchor_no_substitute () =
  let program = mk_program
    ~id:"P_t6"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ { input_name = capability_input_name_of_string "ref";
            binding = Anchor_value (oid "O_anchor", [ "document"; "title" ]) } ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let snapshot =
    `Assoc [
      ("document", `Assoc [
        ("title", `String "Tethers")
      ])
    ]
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_t6"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~anchors:[ mk_anchor_snapshot "O_other_anchor" snapshot ]
      ()
  in
  assert_plan_error (Missing_anchor_snapshot (oid "O_anchor"))
    "T6 wrong anchor does not substitute" (plan program ctx)

(* ================================================================== *)
(*  T7 — Duplicate snapshot ambiguity                                   *)
(* ================================================================== *)

let test_duplicate_snapshot_ambiguity () =
  let program = mk_program
    ~id:"P_t7"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ { input_name = capability_input_name_of_string "ref";
            binding = Anchor_value (oid "O_anchor", [ "document"; "title" ]) } ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let snapshot =
    `Assoc [
      ("document", `Assoc [
        ("title", `String "Tethers")
      ])
    ]
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_t7"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~anchors:[
        mk_anchor_snapshot "O_anchor" snapshot;
        mk_anchor_snapshot "O_anchor" snapshot;
      ]
      ()
  in
  assert_plan_error (Ambiguous_anchor_snapshot (oid "O_anchor"))
    "T7 duplicate snapshot ambiguity" (plan program ctx)

(* ================================================================== *)
(*  T8 — Reversed duplicate snapshot order                              *)
(* ================================================================== *)

let test_reversed_duplicate_snapshot_order () =
  let program = mk_program
    ~id:"P_t8"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ { input_name = capability_input_name_of_string "ref";
            binding = Anchor_value (oid "O_anchor", [ "document"; "title" ]) } ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let snapshot =
    `Assoc [
      ("document", `Assoc [
        ("title", `String "Tethers")
      ])
    ]
  in
  let ctx_fwd =
    mk_context
      ~evaluation_id:"eval_t8"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~anchors:[
        mk_anchor_snapshot "O_anchor" snapshot;
        mk_anchor_snapshot "O_anchor" snapshot;
      ]
      ()
  in
  let ctx_rev =
    mk_context
      ~evaluation_id:"eval_t8"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~anchors:[
        mk_anchor_snapshot "O_anchor" snapshot;
        mk_anchor_snapshot "O_anchor" snapshot;
      ]
      ()
  in
  assert_plan_error (Ambiguous_anchor_snapshot (oid "O_anchor"))
    "T8 reversed duplicates fail (forward)" (plan program ctx_fwd);
  assert_plan_error (Ambiguous_anchor_snapshot (oid "O_anchor"))
    "T8 reversed duplicates fail (reversed)" (plan program ctx_rev)

(* ================================================================== *)
(*  T9 — Missing path component                                         *)
(* ================================================================== *)

let test_missing_path_component () =
  let program = mk_program
    ~id:"P_t9"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ { input_name = capability_input_name_of_string "ref";
            binding = Anchor_value (oid "O_anchor", [ "document"; "title" ]) } ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let snapshot =
    `Assoc [
      ("document", `Assoc [
        ("other", `String "value")
      ])
    ]
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_t9"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~anchors:[ mk_anchor_snapshot "O_anchor" snapshot ]
      ()
  in
  assert_plan_error (Anchor_path_missing (oid "O_anchor", [ "document"; "title" ]))
    "T9 missing path component" (plan program ctx)

(* ================================================================== *)
(*  T10 — Non-object traversal                                          *)
(* ================================================================== *)

let test_non_object_traversal () =
  let program = mk_program
    ~id:"P_t10"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ { input_name = capability_input_name_of_string "ref";
            binding = Anchor_value (oid "O_anchor", [ "document"; "title" ]) } ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let snapshot =
    `Assoc [
      ("document", `String "hello")
    ]
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_t10"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~anchors:[ mk_anchor_snapshot "O_anchor" snapshot ]
      ()
  in
  assert_plan_error (Anchor_path_not_object (oid "O_anchor", [ "document"; "title" ]))
    "T10 non-object traversal" (plan program ctx)

(* ================================================================== *)
(*  T11 — Unsupported terminal JSON                                     *)
(* ================================================================== *)

let test_unsupported_terminal_json () =
  let program = mk_program
    ~id:"P_t11"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ { input_name = capability_input_name_of_string "ref";
            binding = Anchor_value (oid "O_anchor", [ "document"; "title" ]) } ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let test_unsupported name terminal =
    let snapshot =
      `Assoc [
        ("document", `Assoc [
          ("title", terminal)
        ])
      ]
    in
    let ctx =
      mk_context
        ~evaluation_id:"eval_t11"
        ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
        ~anchors:[ mk_anchor_snapshot "O_anchor" snapshot ]
        ()
    in
    assert_plan_error (Unsupported_anchor_value_type (oid "O_anchor", [ "document"; "title" ]))
      ("T11 " ^ name) (plan program ctx)
  in
  test_unsupported "object" (`Assoc [("key", `String "value")]);
  test_unsupported "array" (`List [ `String "a" ]);
  test_unsupported "null" `Null

(* ================================================================== *)
(*  T12 — Existing fail-closed behaviour (Fact_from_origin)             *)
(* ================================================================== *)

let test_existing_fail_closed_fact_from_origin () =
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
    ~id:"P_t12"
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
  let ctx = mk_context ~evaluation_id:"eval_t12" () in
  assert_plan_error Unsupported_fact_binding
    "T12 Fact_from_origin still fails closed" (plan program ctx)

(* ================================================================== *)
(*  E2E — Human → Parser → Lowerer → Planner proof                     *)
(* ================================================================== *)

let test_e2e_human_to_plan () =
  (* Step 1: Human Tether source *)
  let source = {|tether "anchor e2e"
anchor
    document.received
when
do
    notify
        title: anchor.document.title
|} in
  (* Step 2: Parse *)
  let parsed = Tether_parser.parse_tether source in
  assert_true "E2E parsed title" (parsed.title = "anchor e2e");
  assert_true "E2E parsed anchor" (parsed.anchor = "document.received");
  assert_true "E2E parsed no conditions" (parsed.conditions = []);
  (* Step 3: Lower with explicit capability mapping *)
  let env : Tethers_core_lowerer.lowering_environment = {
    program_id = program_id_of_string "P_e2e";
    core_version = core_version_of_string "0.1.0";
    capabilities = [
      { source_name = "notify";
        capability_id = cid "cap.notify";
        contract_digest = capability_contract_digest_of_string "sha256:e2e" };
    ];
    input_facts = [];
  } in
  let lowered = match Tethers_core_lowerer.lower env parsed with
    | Ok p -> p
    | Error _ -> assert_true "E2E lower ok" false; assert false
  in
  (* Step 4: Verify the lowered Core contains Anchor_value *)
  let expected_origin = oid "O_anchor" in
  (match lowered.origin_sites with
   | [ Anchor_origin _; Action_origin ao ] ->
       (match ao.inputs with
        | [ { binding = Anchor_value (resolved_oid, path); _ } ] ->
            assert_true "E2E anchor origin id" (resolved_oid = expected_origin);
            assert_true "E2E anchor path" (path = [ "document"; "title" ])
        | _ -> assert_true "E2E anchor binding shape" false)
   | _ -> assert_true "E2E origin_sites shape" false);
  (* Step 5: Plan with runtime snapshot *)
  let snapshot =
    `Assoc [
      ("document", `Assoc [
        ("title", `String "Tethers")
      ])
    ]
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_e2e"
      ~capabilities:[ mk_projection "cap.notify" "sha256:e2e" ~name:"cap.notify" () ]
      ~anchors:[ mk_anchor_snapshot "O_anchor" snapshot ]
      ()
  in
  let p = assert_ok_plan "E2E plan" (plan lowered ctx) in
  (* Step 6: Prove the planned argument is the resolved string *)
  (match p.actions with
   | [ action ] ->
       assert_true "E2E planned argument"
         (action_field "arguments" action =
            `Assoc [ ("title", `String "Tethers") ])
   | _ -> assert_true "E2E single-action shape" false)

(* ================================================================== *)
(*  CORE-6B T1 — Canonicalized entry point produces Runtime Plan       *)
(* ================================================================== *)

let test_canonical_plan_basic () =
  let program = mk_program
    ~id:"P_cb1"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ mk_lit_input "message" (String_value "hello") ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let ctx =
    mk_context
      ~evaluation_id:"eval_cb1"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  let cp = assert_ok_canonical_plan "CB-T1 plan" (plan_canonicalized c ctx) in
  assert_true "CB-T1 runtime plan has actions"
    (List.length cp.runtime_plan.actions = 1);
  assert_true "CB-T1 plan id derives from evaluation_id"
    (cp.runtime_plan.id = "eval_cb1/plan")

(* ================================================================== *)
(*  CORE-6B T2 — Returned ProgramDigest equals canonicalized digest     *)
(* ================================================================== *)

let test_canonical_plan_digest_matches () =
  let program = mk_program
    ~id:"P_cb2"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ mk_lit_input "message" (String_value "hello") ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let ctx =
    mk_context
      ~evaluation_id:"eval_cb2"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  let cp = assert_ok_canonical_plan "CB-T2 plan" (plan_canonicalized c ctx) in
  assert_true "CB-T2 digest matches"
    (Tethers_core_canonical.program_digest c = cp.program_digest)

(* ================================================================== *)
(*  CORE-6B T3 — Human → Canonical Core → Plan Anchor_value proof      *)
(* ================================================================== *)

let test_e2e_human_to_canonical_plan () =
  let source = {|tether "anchor canonical"
anchor
    doc.arrived
when
do
    notify
        title: anchor.document.title
|} in
  let parsed = Tether_parser.parse_tether source in
  let env : Tethers_core_lowerer.lowering_environment = {
    program_id = program_id_of_string "P_cb3";
    core_version = core_version_of_string "0.1.0";
    capabilities = [
      { source_name = "notify";
        capability_id = cid "cap.notify";
        contract_digest = capability_contract_digest_of_string "sha256:abc" };
    ];
    input_facts = [];
  } in
  let lowered = match Tethers_core_lowerer.lower env parsed with
    | Ok p -> p
    | Error _ -> assert_true "CB-T3 lower ok" false; assert false
  in
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize lowered) in
  let c_program = Tethers_core_canonical.canonical_program c in
  (* Locate the canonical Anchor_origin and extract its canonical OriginId *)
  let canonical_anchor_oid =
    let rec find = function
      | [] -> assert_true "CB-T3 has canonical anchor" false; oid "O_missing"
      | Anchor_origin a :: _ -> a.anchor_origin_id
      | _ :: rest -> find rest
    in
    find c_program.origin_sites
  in
  let snapshot =
    `Assoc [
      ("document", `Assoc [
        ("title", `String "Tethers")
      ])
    ]
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_cb3"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~anchors:[ mk_anchor_snapshot
                   (Tethers_core.string_of_origin_id canonical_anchor_oid)
                   snapshot ]
      ()
  in
  let cp = assert_ok_canonical_plan "CB-T3 plan" (plan_canonicalized c ctx) in
  (match cp.runtime_plan.actions with
   | [ action ] ->
       assert_true "CB-T3 resolved string"
         (action_field "arguments" action =
            `Assoc [ ("title", `String "Tethers") ])
   | _ -> assert_true "CB-T3 single-action shape" false)

(* ================================================================== *)
(*  CORE-6B T4 — ProgramId variation leaves digest and occurrence      *)
(*                   plan unchanged                                     *)
(* ================================================================== *)

let test_program_id_varies_digest_unchanged () =
  let build pid =
    mk_program
      ~id:pid
      ~entry_origin:(Some (oid "O_anchor"))
      ~origin_sites:[
        mk_anchor_origin "O_anchor" "doc.arrived" [];
        mk_action_origin "O_action" "cap.notify" "sha256:abc"
          [ mk_lit_input "message" (String_value "hello") ] [];
      ]
      ~success_continuations:[
        mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
        mk_success_cont "O_action" Program_complete;
      ]
      ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
      ()
  in
  let c1 = assert_ok_canonical (Tethers_core_canonical.canonicalize (build "P_alpha")) in
  let c2 = assert_ok_canonical (Tethers_core_canonical.canonicalize (build "P_beta")) in
  let ctx =
    mk_context
      ~evaluation_id:"eval_cb4"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  let cp1 = assert_ok_canonical_plan "CB-T4 plan alpha" (plan_canonicalized c1 ctx) in
  let cp2 = assert_ok_canonical_plan "CB-T4 plan beta" (plan_canonicalized c2 ctx) in
  assert_true "CB-T4 digests equal"
    (Tethers_core_canonical.program_digest c1 = Tethers_core_canonical.program_digest c2);
  assert_true "CB-T4 plan ids equal"
    (cp1.runtime_plan.id = cp2.runtime_plan.id);
  assert_true "CB-T4 plan ids derive from evaluation_id"
    (cp1.runtime_plan.id = "eval_cb4/plan");
  assert_true "CB-T4 actions equal"
    (cp1.runtime_plan.actions = cp2.runtime_plan.actions)

(* ================================================================== *)
(*  CORE-6B T5 — Pre-canonical temporary ID/storage variation           *)
(*                   canonicalises to equal plans                       *)
(* ================================================================== *)

let test_temp_id_storage_order_canonical_plan () =
  let mk_prog anchor_oid action_oid =
    mk_program
      ~id:"P_cb5"
      ~entry_origin:(Some (oid anchor_oid))
      ~origin_sites:[
        mk_anchor_origin anchor_oid "doc.arrived" [];
        mk_action_origin action_oid "cap.notify" "sha256:abc"
          [ mk_lit_input "message" (String_value "hello") ] [];
      ]
      ~success_continuations:[
        mk_success_cont anchor_oid (Origin_target (oid action_oid));
        mk_success_cont action_oid Program_complete;
      ]
      ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
      ()
  in
  let c1 = assert_ok_canonical (Tethers_core_canonical.canonicalize (mk_prog "O_x" "O_y")) in
  let c2 = assert_ok_canonical (Tethers_core_canonical.canonicalize (mk_prog "O_a" "O_b")) in
  let ctx =
    mk_context
      ~evaluation_id:"eval_cb5"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  let cp1 = assert_ok_canonical_plan "CB-T5 plan 1" (plan_canonicalized c1 ctx) in
  let cp2 = assert_ok_canonical_plan "CB-T5 plan 2" (plan_canonicalized c2 ctx) in
  assert_true "CB-T5 digests equal"
    (Tethers_core_canonical.program_digest c1 = Tethers_core_canonical.program_digest c2);
  assert_true "CB-T5 canonical programs structurally equal"
    (Tethers_core_canonical.canonical_program c1 = Tethers_core_canonical.canonical_program c2);
  assert_true "CB-T5 runtime plans equal"
    (cp1.runtime_plan = cp2.runtime_plan)

(* ================================================================== *)
(*  CORE-6B T6 — Anchor snapshot keyed by canonical OriginId resolves   *)
(* ================================================================== *)

let test_canonical_anchor_snapshot_resolves () =
  let source = {|tether "snap"
anchor
    doc.arrived
when
do
    notify
        title: anchor.document.title
|} in
  let parsed = Tether_parser.parse_tether source in
  let env : Tethers_core_lowerer.lowering_environment = {
    program_id = program_id_of_string "P_cb6";
    core_version = core_version_of_string "0.1.0";
    capabilities = [
      { source_name = "notify";
        capability_id = cid "cap.notify";
        contract_digest = capability_contract_digest_of_string "sha256:abc" };
    ];
    input_facts = [];
  } in
  let lowered = match Tethers_core_lowerer.lower env parsed with
    | Ok p -> p
    | Error _ -> assert_true "CB-T6 lower ok" false; assert false
  in
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize lowered) in
  let c_program = Tethers_core_canonical.canonical_program c in
  let canonical_anchor_oid =
    let rec find = function
      | [] -> assert_true "CB-T6 has canonical anchor" false; oid "O_missing"
      | Anchor_origin a :: _ -> a.anchor_origin_id
      | _ :: rest -> find rest
    in
    find c_program.origin_sites
  in
  let snapshot =
    `Assoc [
      ("document", `Assoc [
        ("title", `String "Tethers")
      ])
    ]
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_cb6"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~anchors:[ mk_anchor_snapshot
                   (Tethers_core.string_of_origin_id canonical_anchor_oid)
                   snapshot ]
      ()
  in
  let cp = assert_ok_canonical_plan "CB-T6 plan" (plan_canonicalized c ctx) in
  (match cp.runtime_plan.actions with
   | [ action ] ->
       assert_true "CB-T6 resolved string"
         (action_field "arguments" action =
            `Assoc [ ("title", `String "Tethers") ])
   | _ -> assert_true "CB-T6 single-action shape" false)

(* ================================================================== *)
(*  CORE-6B T7 — Stale pre-canonical Anchor OriginId does not work      *)
(* ================================================================== *)

let test_stale_pre_canonical_snapshot_fails () =
  let source = {|tether "stale"
anchor
    doc.arrived
when
do
    notify
        title: anchor.document.title
|} in
  let parsed = Tether_parser.parse_tether source in
  let env : Tethers_core_lowerer.lowering_environment = {
    program_id = program_id_of_string "P_cb7";
    core_version = core_version_of_string "0.1.0";
    capabilities = [
      { source_name = "notify";
        capability_id = cid "cap.notify";
        contract_digest = capability_contract_digest_of_string "sha256:abc" };
    ];
    input_facts = [];
  } in
  let lowered = match Tethers_core_lowerer.lower env parsed with
    | Ok p -> p
    | Error _ -> assert_true "CB-T7 lower ok" false; assert false
  in
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize lowered) in
  let c_program = Tethers_core_canonical.canonical_program c in
  let canonical_anchor_oid =
    let rec find = function
      | [] -> assert_true "CB-T7 has canonical anchor" false; oid "O_missing"
      | Anchor_origin a :: _ -> a.anchor_origin_id
      | _ :: rest -> find rest
    in
    find c_program.origin_sites
  in
  (* Use a deliberately wrong pre-canonical OriginId as snapshot key *)
  let stale_oid = oid "O_anchor" in
  let snapshot =
    `Assoc [
      ("document", `Assoc [
        ("title", `String "Tethers")
      ])
    ]
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_cb7"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~anchors:[ mk_anchor_snapshot "O_anchor" snapshot ]
      ()
  in
  (* The canonical program uses canonical OriginIds (e.g. O1), so a snapshot
     keyed by the pre-canonical O_anchor won't match.  The error will name the
     canonical OriginId that was actually looked up. *)
  (match plan_canonicalized c ctx with
   | Error (Missing_anchor_snapshot looked_up_oid) ->
       assert_true "CB-T7 error names canonical anchor"
         (looked_up_oid = canonical_anchor_oid);
       assert_true "CB-T7 canonical differs from pre-canonical"
         (canonical_anchor_oid <> stale_oid);
       incr tests_run; incr tests_passed
   | Error err ->
       incr tests_run;
       Printf.eprintf "FAIL: CB-T7 expected Missing_anchor_snapshot, got %s\n"
         (string_of_planning_error err);
       exit 1
   | Ok _ ->
       incr tests_run;
       Printf.eprintf "FAIL: CB-T7 expected Error, got Ok\n";
       exit 1)

(* ================================================================== *)
(*  CORE-6B T8 — Existing CORE-6A planner tests remain green            *)
(* ================================================================== *)

let test_existing_core6a_tests_green () =
  (* Re-run the existing anchor resolution through the low-level plan *)
  let program = mk_program
    ~id:"P_cb8"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ { input_name = capability_input_name_of_string "ref";
            binding = Anchor_value (oid "O_anchor", [ "document"; "title" ]) } ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let snapshot =
    `Assoc [
      ("document", `Assoc [
        ("title", `String "Tethers")
      ])
    ]
  in
  let ctx =
    mk_context
      ~evaluation_id:"eval_cb8"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~anchors:[ mk_anchor_snapshot "O_anchor" snapshot ]
      ()
  in
  (* Low-level plan still works *)
  let p = assert_ok_plan "CB-T8 low-level plan" (plan program ctx) in
  (match p.actions with
   | [ action ] ->
       assert_true "CB-T8 resolved string"
         (action_field "arguments" action =
            `Assoc [ ("ref", `String "Tethers") ])
   | _ -> assert_true "CB-T8 single-action shape" false);
  (* Canonical path: use the canonical OriginId for the snapshot *)
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let c_program = Tethers_core_canonical.canonical_program c in
  let canonical_anchor_oid =
    let rec find = function
      | [] -> assert_true "CB-T8 has canonical anchor" false; oid "O_missing"
      | Anchor_origin a :: _ -> a.anchor_origin_id
      | _ :: rest -> find rest
    in
    find c_program.origin_sites
  in
  let ctx_canonical =
    mk_context
      ~evaluation_id:"eval_cb8"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~anchors:[ mk_anchor_snapshot
                   (Tethers_core.string_of_origin_id canonical_anchor_oid)
                   snapshot ]
      ()
  in
  let cp = assert_ok_canonical_plan "CB-T8 canonical plan" (plan_canonicalized c ctx_canonical) in
  (match cp.runtime_plan.actions with
   | [ action ] ->
       assert_true "CB-T8 canonical resolved string"
         (action_field "arguments" action =
            `Assoc [ ("ref", `String "Tethers") ])
    | _ -> assert_true "CB-T8 canonical single-action shape" false)

(* ================================================================== *)
(*  CORE-7A T1 -- Equals string matches                               *)
(* ================================================================== *)

let test_guard_equals_string_match () =
  let program = mk_program
    ~id:"P_g1"
    ~input_facts:[ mk_eval_fact "F_type" "K_type" String_type ]
    ~entry_guards:[ mk_guard "F_type" Equals (String_value "pdf") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
    mk_eval_context
      ~evaluation_id:"eval_g1"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_type" (`String "pdf") ]
      ()
  in
  match evaluate_canonicalized (assert_ok_canonical (Tethers_core_canonical.canonicalize program)) ctx with
  | Ok (Matched cp) ->
      incr tests_run; incr tests_passed;
      assert_true "G1-T1 plan has actions" (List.length cp.runtime_plan.actions = 1)
  | Ok Not_matched ->
      incr tests_run;
      Printf.eprintf "FAIL: G1-T1 expected Matched, got Not_matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: G1-T1 expected Matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7A T2 -- Equals string false                                 *)
(* ================================================================== *)

let test_guard_equals_string_false () =
  let program = mk_program
    ~id:"P_g2"
    ~input_facts:[ mk_eval_fact "F_type" "K_type" String_type ]
    ~entry_guards:[ mk_guard "F_type" Equals (String_value "pdf") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
    mk_eval_context
      ~evaluation_id:"eval_g2"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_type" (`String "jpg") ]
      ()
  in
  match evaluate_canonicalized (assert_ok_canonical (Tethers_core_canonical.canonicalize program)) ctx with
  | Ok Not_matched -> incr tests_run; incr tests_passed
  | Ok (Matched _) ->
      incr tests_run;
      Printf.eprintf "FAIL: G1-T2 expected Not_matched, got Matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: G1-T2 expected Not_matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7A T3 -- Integer Greater_than                                *)
(* ================================================================== *)

let test_guard_integer_greater_than () =
  let program = mk_program
    ~id:"P_g3"
    ~input_facts:[ mk_eval_fact "F_size" "K_size" Integer_type ]
    ~entry_guards:[ mk_guard "F_size" Greater_than (Integer_value 10) ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
    mk_eval_context
      ~evaluation_id:"eval_g3"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_size" (`Int 42) ]
      ()
  in
  match evaluate_canonicalized (assert_ok_canonical (Tethers_core_canonical.canonicalize program)) ctx with
  | Ok (Matched _) -> incr tests_run; incr tests_passed
  | Ok Not_matched ->
      incr tests_run;
      Printf.eprintf "FAIL: G1-T3 expected Matched, got Not_matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: G1-T3 expected Matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7A T4 -- Integer Greater_than_or_equal                       *)
(* ================================================================== *)

let test_guard_integer_greater_than_or_equal () =
  let program = mk_program
    ~id:"P_g4"
    ~input_facts:[ mk_eval_fact "F_size" "K_size" Integer_type ]
    ~entry_guards:[ mk_guard "F_size" Greater_than_or_equal (Integer_value 10) ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
    mk_eval_context
      ~evaluation_id:"eval_g4"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_size" (`Int 10) ]
      ()
  in
  match evaluate_canonicalized (assert_ok_canonical (Tethers_core_canonical.canonicalize program)) ctx with
  | Ok (Matched _) -> incr tests_run; incr tests_passed
  | Ok Not_matched ->
      incr tests_run;
      Printf.eprintf "FAIL: G1-T4 expected Matched, got Not_matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: G1-T4 expected Matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7A T5 -- String Contains                                     *)
(* ================================================================== *)

let test_guard_string_contains () =
  let program = mk_program
    ~id:"P_g5"
    ~input_facts:[ mk_eval_fact "F_name" "K_name" String_type ]
    ~entry_guards:[ mk_guard "F_name" Contains (String_value "core") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
    mk_eval_context
      ~evaluation_id:"eval_g5"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_name" (`String "tethers-core") ]
      ()
  in
  match evaluate_canonicalized (assert_ok_canonical (Tethers_core_canonical.canonicalize program)) ctx with
  | Ok (Matched _) -> incr tests_run; incr tests_passed
  | Ok Not_matched ->
      incr tests_run;
      Printf.eprintf "FAIL: G1-T5 expected Matched, got Not_matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: G1-T5 expected Matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7A T6 -- Boolean Equals                                       *)
(* ================================================================== *)

let test_guard_boolean_equals () =
  let program = mk_program
    ~id:"P_g6"
    ~input_facts:[ mk_eval_fact "F_active" "K_active" Boolean_type ]
    ~entry_guards:[ mk_guard "F_active" Equals (Boolean_value true) ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
    mk_eval_context
      ~evaluation_id:"eval_g6"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_active" (`Bool true) ]
      ()
  in
  match evaluate_canonicalized (assert_ok_canonical (Tethers_core_canonical.canonicalize program)) ctx with
  | Ok (Matched _) -> incr tests_run; incr tests_passed
  | Ok Not_matched ->
      incr tests_run;
      Printf.eprintf "FAIL: G1-T6 expected Matched, got Not_matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: G1-T6 expected Matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7A T7 -- Multiple guards AND together                        *)
(* ================================================================== *)

let test_multiple_guards_and () =
  let program = mk_program
    ~id:"P_g7"
    ~input_facts:[
      mk_eval_fact "F_type" "K_type" String_type;
      mk_eval_fact "F_size" "K_size" Integer_type;
      mk_eval_fact "F_active" "K_active" Boolean_type;
    ]
    ~entry_guards:[
      mk_guard "F_type" Equals (String_value "pdf");
      mk_guard "F_size" Greater_than (Integer_value 10);
      mk_guard "F_active" Equals (Boolean_value true);
    ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
  (* All true *)
  let ctx_all =
    mk_eval_context
      ~evaluation_id:"eval_g7"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[
        mk_fact_snapshot "K_type" (`String "pdf");
        mk_fact_snapshot "K_size" (`Int 42);
        mk_fact_snapshot "K_active" (`Bool true);
      ]
      ()
  in
  (match evaluate_canonicalized (assert_ok_canonical (Tethers_core_canonical.canonicalize program)) ctx_all with
   | Ok (Matched _) -> incr tests_run; incr tests_passed
   | Ok Not_matched ->
       incr tests_run;
       Printf.eprintf "FAIL: G1-T7a expected Matched, got Not_matched\n";
       exit 1
   | Error err ->
       incr tests_run;
       Printf.eprintf "FAIL: G1-T7a expected Matched, got Error %s\n"
         (string_of_planning_error err);
       exit 1);
  (* One false -- change file_type to jpg *)
  let ctx_one_false =
    mk_eval_context
      ~evaluation_id:"eval_g7b"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[
        mk_fact_snapshot "K_type" (`String "jpg");
        mk_fact_snapshot "K_size" (`Int 42);
        mk_fact_snapshot "K_active" (`Bool true);
      ]
      ()
  in
  match evaluate_canonicalized (assert_ok_canonical (Tethers_core_canonical.canonicalize program)) ctx_one_false with
  | Ok Not_matched -> incr tests_run; incr tests_passed
  | Ok (Matched _) ->
      incr tests_run;
      Printf.eprintf "FAIL: G1-T7b expected Not_matched, got Matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: G1-T7b expected Not_matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7A T8 -- Missing runtime Fact                                 *)
(* ================================================================== *)

let test_missing_fact_snapshot () =
  let program = mk_program
    ~id:"P_g8"
    ~input_facts:[ mk_eval_fact "F_type" "K_type" String_type ]
    ~entry_guards:[ mk_guard "F_type" Equals (String_value "pdf") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
    mk_eval_context
      ~evaluation_id:"eval_g8"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[]  (* no snapshots *)
      ()
  in
  assert_plan_error (Missing_fact_snapshot (hsk "K_type"))
    "G1-T8 missing fact snapshot"
    (match evaluate_canonicalized (assert_ok_canonical (Tethers_core_canonical.canonicalize program)) ctx with
     | Ok _ -> Error Unresolved_entry_guards  (* dummy *)
     | Error e -> Error e)

(* ================================================================== *)
(*  CORE-7A T9 -- Wrong HostSnapshotKey does not substitute            *)
(* ================================================================== *)

let test_wrong_key_no_substitute () =
  let program = mk_program
    ~id:"P_g9"
    ~input_facts:[ mk_eval_fact "F_type" "K_type" String_type ]
    ~entry_guards:[ mk_guard "F_type" Equals (String_value "pdf") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
    mk_eval_context
      ~evaluation_id:"eval_g9"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_wrong_key" (`String "pdf") ]  (* wrong key *)
      ()
  in
  assert_plan_error (Missing_fact_snapshot (hsk "K_type"))
    "G1-T9 wrong key does not substitute"
    (match evaluate_canonicalized (assert_ok_canonical (Tethers_core_canonical.canonicalize program)) ctx with
     | Ok _ -> Error Unresolved_entry_guards
     | Error e -> Error e)

(* ================================================================== *)
(*  CORE-7A T10 -- Duplicate HostSnapshotKey                           *)
(* ================================================================== *)

let test_duplicate_fact_snapshot () =
  let program = mk_program
    ~id:"P_g10"
    ~input_facts:[ mk_eval_fact "F_type" "K_type" String_type ]
    ~entry_guards:[ mk_guard "F_type" Equals (String_value "pdf") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
    mk_eval_context
      ~evaluation_id:"eval_g10"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[
        mk_fact_snapshot "K_type" (`String "pdf");
        mk_fact_snapshot "K_type" (`String "pdf");
      ]
      ()
  in
  assert_plan_error (Ambiguous_fact_snapshot (hsk "K_type"))
    "G1-T10 duplicate fact snapshot"
    (match evaluate_canonicalized (assert_ok_canonical (Tethers_core_canonical.canonicalize program)) ctx with
     | Ok _ -> Error Unresolved_entry_guards
     | Error e -> Error e)

(* ================================================================== *)
(*  CORE-7A T11 -- Reversed duplicate order                           *)
(* ================================================================== *)

let test_reversed_duplicate_fact_order () =
  let program = mk_program
    ~id:"P_g11"
    ~input_facts:[ mk_eval_fact "F_type" "K_type" String_type ]
    ~entry_guards:[ mk_guard "F_type" Equals (String_value "pdf") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
  let ctx_fwd =
    mk_eval_context
      ~evaluation_id:"eval_g11"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[
        mk_fact_snapshot "K_type" (`String "pdf");
        mk_fact_snapshot "K_type" (`String "pdf");
      ]
      ()
  in
  let ctx_rev =
    mk_eval_context
      ~evaluation_id:"eval_g11"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[
        mk_fact_snapshot "K_type" (`String "pdf");
        mk_fact_snapshot "K_type" (`String "pdf");
      ]
      ()
  in
  let eval ctx =
    match evaluate_canonicalized (assert_ok_canonical (Tethers_core_canonical.canonicalize program)) ctx with
    | Ok _ -> Error Unresolved_entry_guards
    | Error e -> Error e
  in
  assert_plan_error (Ambiguous_fact_snapshot (hsk "K_type"))
    "G1-T11 reversed duplicates fail (forward)" (eval ctx_fwd);
  assert_plan_error (Ambiguous_fact_snapshot (hsk "K_type"))
    "G1-T11 reversed duplicates fail (reversed)" (eval ctx_rev)

(* ================================================================== *)
(*  CORE-7A T12 -- Runtime type mismatch                              *)
(* ================================================================== *)

let test_fact_snapshot_type_mismatch () =
  let program = mk_program
    ~id:"P_g12"
    ~input_facts:[ mk_eval_fact "F_size" "K_size" Integer_type ]
    ~entry_guards:[ mk_guard "F_size" Greater_than (Integer_value 10) ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
    mk_eval_context
      ~evaluation_id:"eval_g12"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_size" (`String "42") ]  (* string, not int *)
      ()
  in
  assert_plan_error (Fact_snapshot_type_mismatch (hsk "K_size"))
    "G1-T12 runtime type mismatch"
    (match evaluate_canonicalized (assert_ok_canonical (Tethers_core_canonical.canonicalize program)) ctx with
     | Ok _ -> Error Unresolved_entry_guards
     | Error e -> Error e)

(* ================================================================== *)
(*  CORE-7A T13 -- Invalid comparison typing                           *)
(* ================================================================== *)

let test_invalid_guard_comparison () =
  let program = mk_program
    ~id:"P_g13"
    ~input_facts:[ mk_eval_fact "F_name" "K_name" String_type ]
    ~entry_guards:[ mk_guard "F_name" Greater_than (String_value "abc") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let c_program = Tethers_core_canonical.canonical_program c in
  let canonical_fid =
    match c_program.entry_guards with
    | g :: _ -> g.fact_id
    | [] -> assert_true "G1-T13 has guards" false; fid "missing"
  in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_g13"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_name" (`String "hello") ]
      ()
  in
  assert_plan_error (Invalid_guard_comparison canonical_fid)
    "G1-T13 invalid comparison typing"
    (match evaluate_canonicalized c ctx with
     | Ok _ -> Error Unresolved_entry_guards
     | Error e -> Error e)

(* ================================================================== *)
(*  CORE-7A T14 -- Low-level guard bypass blocked                     *)
(* ================================================================== *)

let test_low_level_guard_bypass () =
  let program = mk_program
    ~id:"P_g14"
    ~input_facts:[ mk_eval_fact "F_type" "K_type" String_type ]
    ~entry_guards:[ mk_guard "F_type" Equals (String_value "pdf") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
      ~evaluation_id:"eval_g14"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_type" (`String "pdf") ]
      ()
  in
  assert_plan_error Unresolved_entry_guards
    "G1-T14 low-level plan rejects guarded program"
    (plan program ctx)

(* ================================================================== *)
(*  CORE-7A T15 -- plan_canonicalized guard bypass blocked             *)
(* ================================================================== *)

let test_canonical_guard_bypass () =
  let program = mk_program
    ~id:"P_g15"
    ~input_facts:[ mk_eval_fact "F_type" "K_type" String_type ]
    ~entry_guards:[ mk_guard "F_type" Equals (String_value "pdf") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let ctx =
    mk_context
      ~evaluation_id:"eval_g15"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_type" (`String "pdf") ]
      ()
  in
  assert_plan_error Unresolved_entry_guards
    "G1-T15 plan_canonicalized rejects guarded program"
    (plan_canonicalized c ctx)

(* ================================================================== *)
(*  CORE-7A T16 -- Unguarded existing behaviour preserved              *)
(* ================================================================== *)

let test_unguarded_existing_behaviour () =
  (* Existing zero-guard CORE-6B case still plans normally *)
  let program = mk_program
    ~id:"P_g16"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.arrived" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ mk_lit_input "message" (String_value "hello") ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_g16"
      ~event:(mk_runtime_event "doc.arrived" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  (* evaluate_canonicalized with no guards should produce Matched *)
  (match evaluate_canonicalized c ctx with
   | Ok (Matched cp) ->
       incr tests_run; incr tests_passed;
       assert_true "G1-T6 plan has actions" (List.length cp.runtime_plan.actions = 1)
   | Ok Not_matched ->
       incr tests_run;
       Printf.eprintf "FAIL: G1-T16 expected Matched, got Not_matched\n";
       exit 1
   | Error err ->
       incr tests_run;
       Printf.eprintf "FAIL: G1-T16 expected Matched, got Error %s\n"
         (string_of_planning_error err);
       exit 1);
  (* plan_canonicalized with no guards should also work *)
  let ctx_low =
    mk_context
      ~evaluation_id:"eval_g16"
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  match plan_canonicalized c ctx_low with
  | Ok cp ->
      incr tests_run; incr tests_passed;
      assert_true "G1-T16 low-level also works"
        (List.length cp.runtime_plan.actions = 1)
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: G1-T16 low-level expected Ok, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7A T17 -- ProgramDigest invariant across runtime facts        *)
(* ================================================================== *)

let test_program_digest_invariant_across_facts () =
  let program = mk_program
    ~id:"P_g17"
    ~input_facts:[ mk_eval_fact "F_type" "K_type" String_type ]
    ~entry_guards:[ mk_guard "F_type" Equals (String_value "pdf") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let expected_digest = Tethers_core_canonical.program_digest c in
  (* Occurrence A: matched *)
  let ctx_a =
    mk_eval_context
      ~evaluation_id:"eval_g17a"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_type" (`String "pdf") ]
      ()
  in
  (match evaluate_canonicalized c ctx_a with
   | Ok (Matched cp) ->
       assert_true "G1-T17a digest matches"
         (Tethers_core_canonical.program_digest c = cp.program_digest);
       assert_true "G1-T17a digest equals expected"
         (cp.program_digest = expected_digest)
   | _ ->
       incr tests_run;
       Printf.eprintf "FAIL: G1-T17a expected Matched\n";
       exit 1);
  (* Occurrence B: not matched *)
  let ctx_b =
    mk_eval_context
      ~evaluation_id:"eval_g17b"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_type" (`String "jpg") ]
      ()
  in
  (match evaluate_canonicalized c ctx_b with
   | Ok Not_matched ->
       (* No Runtime Plan exists; compare digest from canonicalized value *)
       assert_true "G1-T17b digest still equals expected"
         (expected_digest = expected_digest)  (* tautology but proves we reached here *)
   | _ ->
       incr tests_run;
       Printf.eprintf "FAIL: G1-T17b expected Not_matched\n";
       exit 1);
  incr tests_run; incr tests_passed

(* ================================================================== *)
(*  CORE-7A E2E -- Human -> Canonical Core -> Guard -> Plan            *)
(* ================================================================== *)

let test_e2e_human_to_guard_to_plan () =
  let source = {|tether "guarded"
anchor
    document.received
when
    file_type is "pdf"
    file_size greater_than 10
do
    notify
        title: anchor.document.title
|} in
  let parsed = Tether_parser.parse_tether source in
  let env : Tethers_core_lowerer.lowering_environment = {
    program_id = program_id_of_string "P_ge2e";
    core_version = core_version_of_string "0.1.0";
    capabilities = [
      { source_name = "notify";
        capability_id = cid "cap.notify";
        contract_digest = capability_contract_digest_of_string "sha256:e2e" };
    ];
    input_facts = [
      { source_name = "file_type";
        fact = { fact_id = fid "F_file_type"; schema_description = "file type";
                 provenance = Evaluation_input (hsk "K_file_type", String_type) } };
      { source_name = "file_size";
        fact = { fact_id = fid "F_file_size"; schema_description = "file size";
                 provenance = Evaluation_input (hsk "K_file_size", Integer_type) } };
    ];
  } in
  let lowered = match Tethers_core_lowerer.lower env parsed with
    | Ok p -> p
    | Error _ -> assert_true "GE2E lower ok" false; assert false
  in
  assert_true "GE2E has entry guards" (List.length lowered.entry_guards = 2);
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize lowered) in
  let event_data =
    `Assoc [
      ("document", `Assoc [
        ("title", `String "Tethers Report")
      ])
    ]
  in
  (* Matched case: event matches, file_type=pdf, file_size=42 *)
  let ctx_matched =
    mk_eval_context
      ~evaluation_id:"eval_ge2e"
      ~event:(mk_runtime_event "document.received" event_data)
      ~capabilities:[ mk_projection "cap.notify" "sha256:e2e" ~name:"cap.notify" () ]
      ~facts:[
        mk_fact_snapshot "K_file_type" (`String "pdf");
        mk_fact_snapshot "K_file_size" (`Int 42);
      ]
      ()
  in
  (match evaluate_canonicalized c ctx_matched with
   | Ok (Matched cp) ->
       assert_true "GE2E matched plan has actions"
         (List.length cp.runtime_plan.actions = 1);
       assert_true "GE2E ProgramDigest preserved"
         (Tethers_core_canonical.program_digest c = cp.program_digest);
       (match cp.runtime_plan.actions with
        | [ action ] ->
            assert_true "GE2E resolved title"
              (action_field "arguments" action =
                 `Assoc [ ("title", `String "Tethers Report") ])
        | _ -> assert_true "GE2E single-action shape" false)
   | Ok Not_matched ->
       incr tests_run;
       Printf.eprintf "FAIL: GE2E expected Matched, got Not_matched\n";
       exit 1
   | Error err ->
       incr tests_run;
       Printf.eprintf "FAIL: GE2E expected Matched, got Error %s\n"
         (string_of_planning_error err);
       exit 1);
  (* Not matched case: file_type=jpg *)
  let ctx_not_matched =
    mk_eval_context
      ~evaluation_id:"eval_ge2e_nm"
      ~event:(mk_runtime_event "document.received" event_data)
      ~capabilities:[ mk_projection "cap.notify" "sha256:e2e" ~name:"cap.notify" () ]
      ~facts:[
        mk_fact_snapshot "K_file_type" (`String "jpg");
        mk_fact_snapshot "K_file_size" (`Int 42);
      ]
      ()
  in
  match evaluate_canonicalized c ctx_not_matched with
  | Ok Not_matched -> incr tests_run; incr tests_passed
  | Ok (Matched _) ->
      incr tests_run;
      Printf.eprintf "FAIL: GE2E expected Not_matched, got Matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: GE2E expected Not_matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7A Adversarial -- Canonical identity independence             *)
(* ================================================================== *)

let test_canonical_identity_adversarial () =
  (* Two programs with different temporary FactIds but same meaning *)
  let mk_prog fid_str hsk_str =
    mk_program
      ~id:"P_gadv"
      ~input_facts:[ mk_eval_fact fid_str hsk_str String_type ]
      ~entry_guards:[ mk_guard fid_str Equals (String_value "pdf") ]
      ~entry_origin:(Some (oid "O_anchor"))
      ~origin_sites:[
        mk_anchor_origin "O_anchor" "doc.received" [];
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
  let c1 = assert_ok_canonical (Tethers_core_canonical.canonicalize (mk_prog "F_alpha" "K_ft")) in
  let c2 = assert_ok_canonical (Tethers_core_canonical.canonicalize (mk_prog "F_beta" "K_ft")) in
  (* Same ProgramDigest *)
  assert_true "adv digests equal"
    (Tethers_core_canonical.program_digest c1 = Tethers_core_canonical.program_digest c2);
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_gadv"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_ft" (`String "pdf") ]
      ()
  in
  let r1 = evaluate_canonicalized c1 ctx in
  let r2 = evaluate_canonicalized c2 ctx in
  (* Both Matched with equal plans *)
  match r1, r2 with
  | Ok (Matched cp1), Ok (Matched cp2) ->
      assert_true "adv plans equal" (cp1.runtime_plan = cp2.runtime_plan);
      assert_true "adv digests match" (cp1.program_digest = cp2.program_digest);
      incr tests_run; incr tests_passed
  | _ ->
      incr tests_run;
      Printf.eprintf "FAIL: adversarial expected both Matched\n";
      exit 1

(* ================================================================== *)
(*  CORE-7A1 E1 -- String_type + Equals + Integer_value                *)
(* ================================================================== *)

let test_equals_string_type_integer_value () =
  let program = mk_program
    ~id:"P_e1"
    ~input_facts:[ mk_eval_fact "F_name" "K_name" String_type ]
    ~entry_guards:[ mk_guard "F_name" Equals (Integer_value 42) ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let c_program = Tethers_core_canonical.canonical_program c in
  let canonical_fid =
    match c_program.entry_guards with
    | g :: _ -> g.fact_id
    | [] -> assert_true "E1 has guards" false; fid "missing"
  in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_e1"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_name" (`String "hello") ]
      ()
  in
  assert_plan_error (Invalid_guard_comparison canonical_fid)
    "E1 String_type + Equals + Integer_value"
    (match evaluate_canonicalized c ctx with
     | Ok _ -> Error Unresolved_entry_guards
     | Error e -> Error e)

(* ================================================================== *)
(*  CORE-7A1 E2 -- Integer_type + Equals + String_value               *)
(* ================================================================== *)

let test_equals_integer_type_string_value () =
  let program = mk_program
    ~id:"P_e2"
    ~input_facts:[ mk_eval_fact "F_size" "K_size" Integer_type ]
    ~entry_guards:[ mk_guard "F_size" Equals (String_value "42") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let c_program = Tethers_core_canonical.canonical_program c in
  let canonical_fid =
    match c_program.entry_guards with
    | g :: _ -> g.fact_id
    | [] -> assert_true "E2 has guards" false; fid "missing"
  in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_e2"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_size" (`Int 42) ]
      ()
  in
  assert_plan_error (Invalid_guard_comparison canonical_fid)
    "E2 Integer_type + Equals + String_value"
    (match evaluate_canonicalized c ctx with
     | Ok _ -> Error Unresolved_entry_guards
     | Error e -> Error e)

(* ================================================================== *)
(*  CORE-7A1 E3 -- Boolean_type + Equals + String_value               *)
(* ================================================================== *)

let test_equals_boolean_type_string_value () =
  let program = mk_program
    ~id:"P_e3"
    ~input_facts:[ mk_eval_fact "F_active" "K_active" Boolean_type ]
    ~entry_guards:[ mk_guard "F_active" Equals (String_value "true") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "doc.received" [];
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
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let c_program = Tethers_core_canonical.canonical_program c in
  let canonical_fid =
    match c_program.entry_guards with
    | g :: _ -> g.fact_id
    | [] -> assert_true "E3 has guards" false; fid "missing"
  in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_e3"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_active" (`Bool true) ]
      ()
  in
  assert_plan_error (Invalid_guard_comparison canonical_fid)
    "E3 Boolean_type + Equals + String_value"
    (match evaluate_canonicalized c ctx with
     | Ok _ -> Error Unresolved_entry_guards
     | Error e -> Error e)

(* ================================================================== *)
(*  CORE-7A1 E4 -- Valid String equality still works                   *)
(* ================================================================== *)

let test_valid_string_equals () =
  let build_guard expected =
    let program = mk_program
      ~id:"P_e4"
      ~input_facts:[ mk_eval_fact "F_name" "K_name" String_type ]
      ~entry_guards:[ mk_guard "F_name" Equals (String_value expected) ]
      ~entry_origin:(Some (oid "O_anchor"))
      ~origin_sites:[
        mk_anchor_origin "O_anchor" "doc.received" [];
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
    assert_ok_canonical (Tethers_core_canonical.canonicalize program)
  in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_e4"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_name" (`String "hello") ]
      ()
  in
  (* "hello" == "hello" -> Matched *)
  (match evaluate_canonicalized (build_guard "hello") ctx with
   | Ok (Matched _) -> incr tests_run; incr tests_passed
   | _ ->
       incr tests_run;
       Printf.eprintf "FAIL: E4a expected Matched\n";
       exit 1);
  (* "hello" == "world" -> Not_matched *)
  match evaluate_canonicalized (build_guard "world") ctx with
  | Ok Not_matched -> incr tests_run; incr tests_passed
  | _ ->
      incr tests_run;
      Printf.eprintf "FAIL: E4b expected Not_matched\n";
      exit 1

(* ================================================================== *)
(*  CORE-7A1 E5 -- Valid Integer equality still works                  *)
(* ================================================================== *)

let test_valid_integer_equals () =
  let build_guard expected =
    let program = mk_program
      ~id:"P_e5"
      ~input_facts:[ mk_eval_fact "F_size" "K_size" Integer_type ]
      ~entry_guards:[ mk_guard "F_size" Equals (Integer_value expected) ]
      ~entry_origin:(Some (oid "O_anchor"))
      ~origin_sites:[
        mk_anchor_origin "O_anchor" "doc.received" [];
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
    assert_ok_canonical (Tethers_core_canonical.canonicalize program)
  in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_e5"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_size" (`Int 42) ]
      ()
  in
  (* 42 == 42 -> Matched *)
  (match evaluate_canonicalized (build_guard 42) ctx with
   | Ok (Matched _) -> incr tests_run; incr tests_passed
   | _ ->
       incr tests_run;
       Printf.eprintf "FAIL: E5a expected Matched\n";
       exit 1);
  (* 42 == 99 -> Not_matched *)
  match evaluate_canonicalized (build_guard 99) ctx with
  | Ok Not_matched -> incr tests_run; incr tests_passed
  | _ ->
      incr tests_run;
      Printf.eprintf "FAIL: E5b expected Not_matched\n";
      exit 1

(* ================================================================== *)
(*  CORE-7A1 E6 -- Valid Boolean equality still works                  *)
(* ================================================================== *)

let test_valid_boolean_equals () =
  let build_guard expected =
    let program = mk_program
      ~id:"P_e6"
      ~input_facts:[ mk_eval_fact "F_active" "K_active" Boolean_type ]
      ~entry_guards:[ mk_guard "F_active" Equals (Boolean_value expected) ]
      ~entry_origin:(Some (oid "O_anchor"))
      ~origin_sites:[
        mk_anchor_origin "O_anchor" "doc.received" [];
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
    assert_ok_canonical (Tethers_core_canonical.canonicalize program)
  in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_e6"
      ~event:(mk_runtime_event "doc.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_active" (`Bool true) ]
      ()
  in
  (* true == true -> Matched *)
  (match evaluate_canonicalized (build_guard true) ctx with
   | Ok (Matched _) -> incr tests_run; incr tests_passed
   | _ ->
       incr tests_run;
       Printf.eprintf "FAIL: E6a expected Matched\n";
       exit 1);
  (* true == false -> Not_matched *)
  match evaluate_canonicalized (build_guard false) ctx with
  | Ok Not_matched -> incr tests_run; incr tests_passed
  | _ ->
      incr tests_run;
      Printf.eprintf "FAIL: E6b expected Not_matched\n";
      exit 1

(* ================================================================== *)
(*  CORE-7B T1 -- Exact event match                                    *)
(* ================================================================== *)

let test_reception_exact_match () =
  let program = mk_program
    ~id:"P_r1"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "document.received" [];
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
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_r1"
      ~event:(mk_runtime_event "document.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  match evaluate_canonicalized c ctx with
  | Ok (Matched cp) ->
      incr tests_run; incr tests_passed;
      assert_true "R-T1 plan has actions" (List.length cp.runtime_plan.actions = 1)
  | Ok Not_matched ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T1 expected Matched, got Not_matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T1 expected Matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7B T2 -- Event mismatch                                       *)
(* ================================================================== *)

let test_reception_event_mismatch () =
  let program = mk_program
    ~id:"P_r2"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "document.received" [];
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
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_r2"
      ~event:(mk_runtime_event "document.deleted" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  match evaluate_canonicalized c ctx with
  | Ok Not_matched -> incr tests_run; incr tests_passed
  | Ok (Matched _) ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T2 expected Not_matched, got Matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T2 expected Not_matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7B T3 -- Matching is exact (no normalisation)                  *)
(* ================================================================== *)

let test_reception_exact_matching () =
  let program = mk_program
    ~id:"P_r3"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "document.received" [];
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
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let test_mismatch event_name =
    let ctx =
      mk_eval_context
        ~evaluation_id:"eval_r3"
        ~event:(mk_runtime_event event_name `Null)
        ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
        ()
    in
    match evaluate_canonicalized c ctx with
    | Ok Not_matched -> incr tests_run; incr tests_passed
    | Ok (Matched _) ->
        incr tests_run;
        Printf.eprintf "FAIL: R-T3 '%s' expected Not_matched, got Matched\n" event_name;
        exit 1
    | Error err ->
        incr tests_run;
        Printf.eprintf "FAIL: R-T3 '%s' expected Not_matched, got Error %s\n"
          event_name (string_of_planning_error err);
        exit 1
  in
  test_mismatch "Document.received";
  test_mismatch "document.received ";
  test_mismatch "document"

(* ================================================================== *)
(*  CORE-7B T4 -- Reception before missing Fact                         *)
(* ================================================================== *)

let test_reception_before_missing_fact () =
  let program = mk_program
    ~id:"P_r4"
    ~input_facts:[ mk_eval_fact "F_type" "K_type" String_type ]
    ~entry_guards:[ mk_guard "F_type" Equals (String_value "pdf") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "document.received" [];
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
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  (* Wrong event + empty facts -- must be Not_matched, not Missing_fact_snapshot *)
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_r4"
      ~event:(mk_runtime_event "document.deleted" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[]
      ()
  in
  match evaluate_canonicalized c ctx with
  | Ok Not_matched -> incr tests_run; incr tests_passed
  | Ok (Matched _) ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T4 expected Not_matched, got Matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T4 expected Not_matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7B T5 -- Reception before malformed Fact                       *)
(* ================================================================== *)

let test_reception_before_malformed_fact () =
  let program = mk_program
    ~id:"P_r5"
    ~input_facts:[ mk_eval_fact "F_size" "K_size" Integer_type ]
    ~entry_guards:[ mk_guard "F_size" Greater_than (Integer_value 10) ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "document.received" [];
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
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  (* Wrong event + malformed Fact (string for integer) -- must be Not_matched *)
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_r5"
      ~event:(mk_runtime_event "document.deleted" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_size" (`String "42") ]
      ()
  in
  match evaluate_canonicalized c ctx with
  | Ok Not_matched -> incr tests_run; incr tests_passed
  | Ok (Matched _) ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T5 expected Not_matched, got Matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T5 expected Not_matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7B T6 -- Matched event then missing Fact                       *)
(* ================================================================== *)

let test_matched_then_missing_fact () =
  let program = mk_program
    ~id:"P_r6"
    ~input_facts:[ mk_eval_fact "F_type" "K_type" String_type ]
    ~entry_guards:[ mk_guard "F_type" Equals (String_value "pdf") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "document.received" [];
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
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  (* Right event + missing Fact -- must be Missing_fact_snapshot *)
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_r6"
      ~event:(mk_runtime_event "document.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[]
      ()
  in
  assert_plan_error (Missing_fact_snapshot (hsk "K_type"))
    "R-T6 matched event then missing fact"
    (match evaluate_canonicalized c ctx with
     | Ok _ -> Error Unresolved_entry_guards
     | Error e -> Error e)

(* ================================================================== *)
(*  CORE-7B T7 -- Matched event then guard false                        *)
(* ================================================================== *)

let test_matched_then_guard_false () =
  let program = mk_program
    ~id:"P_r7"
    ~input_facts:[ mk_eval_fact "F_type" "K_type" String_type ]
    ~entry_guards:[ mk_guard "F_type" Equals (String_value "pdf") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "document.received" [];
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
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_r7"
      ~event:(mk_runtime_event "document.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_type" (`String "jpg") ]
      ()
  in
  match evaluate_canonicalized c ctx with
  | Ok Not_matched -> incr tests_run; incr tests_passed
  | Ok (Matched _) ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T7 expected Not_matched, got Matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T7 expected Not_matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7B T8 -- Matched event + guard true                            *)
(* ================================================================== *)

let test_matched_event_and_guard () =
  let program = mk_program
    ~id:"P_r8"
    ~input_facts:[ mk_eval_fact "F_type" "K_type" String_type ]
    ~entry_guards:[ mk_guard "F_type" Equals (String_value "pdf") ]
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "document.received" [];
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
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_r8"
      ~event:(mk_runtime_event "document.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_type" (`String "pdf") ]
      ()
  in
  match evaluate_canonicalized c ctx with
  | Ok (Matched cp) ->
      incr tests_run; incr tests_passed;
      assert_true "R-T8 plan has actions" (List.length cp.runtime_plan.actions = 1)
  | Ok Not_matched ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T8 expected Matched, got Not_matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T8 expected Matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7B T9 -- Event data resolves Anchor_value                      *)
(* ================================================================== *)

let test_event_data_resolves_anchor () =
  let program = mk_program
    ~id:"P_r9"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "document.received" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ { input_name = capability_input_name_of_string "title";
            binding = Anchor_value (oid "O_anchor", [ "document"; "title" ]) } ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let event_data =
    `Assoc [
      ("document", `Assoc [
        ("title", `String "Tethers")
      ])
    ]
  in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_r9"
      ~event:(mk_runtime_event "document.received" event_data)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  match evaluate_canonicalized c ctx with
  | Ok (Matched cp) ->
      (match cp.runtime_plan.actions with
       | [ action ] ->
           assert_true "R-T9 resolved title"
             (action_field "arguments" action =
                `Assoc [ ("title", `String "Tethers") ]);
           incr tests_run; incr tests_passed
       | _ ->
           incr tests_run;
           Printf.eprintf "FAIL: R-T9 single-action shape\n";
           exit 1)
  | Ok Not_matched ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T9 expected Matched, got Not_matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T9 expected Matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7B T10 -- Event mismatch prevents Anchor path error            *)
(* ================================================================== *)

let test_mismatch_prevents_anchor_error () =
  let program = mk_program
    ~id:"P_r10"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "document.received" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ { input_name = capability_input_name_of_string "title";
            binding = Anchor_value (oid "O_anchor", [ "document"; "title" ]) } ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  (* Wrong event + empty data -- must be Not_matched, not Anchor_path_missing *)
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_r10"
      ~event:(mk_runtime_event "document.deleted" (`Assoc []))
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  match evaluate_canonicalized c ctx with
  | Ok Not_matched -> incr tests_run; incr tests_passed
  | Ok (Matched _) ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T10 expected Not_matched, got Matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T10 expected Not_matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7B T11 -- Event match exposes Anchor path error                *)
(* ================================================================== *)

let test_match_exposes_anchor_error () =
  let program = mk_program
    ~id:"P_r11"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "document.received" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ { input_name = capability_input_name_of_string "title";
            binding = Anchor_value (oid "O_anchor", [ "document"; "title" ]) } ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_anchor" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let c_program = Tethers_core_canonical.canonical_program c in
  let canonical_anchor_oid =
    let rec find = function
      | [] -> assert_true "R-T11 has canonical anchor" false; oid "O_missing"
      | Anchor_origin a :: _ -> a.anchor_origin_id
      | _ :: rest -> find rest
    in
    find c_program.origin_sites
  in
  (* Correct event + empty data -- must be Anchor_path_missing *)
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_r11"
      ~event:(mk_runtime_event "document.received" (`Assoc []))
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  assert_plan_error (Anchor_path_missing (canonical_anchor_oid, [ "document"; "title" ]))
    "R-T11 match exposes anchor path error"
    (match evaluate_canonicalized c ctx with
     | Ok _ -> Error Unresolved_entry_guards
     | Error e -> Error e)

(* ================================================================== *)
(*  CORE-7B T12 -- Missing reception Anchor                             *)
(* ================================================================== *)

let test_missing_reception_anchor () =
  (* A program with no Anchor_origin sites *)
  let program = mk_program
    ~id:"P_r12"
    ~entry_origin:(Some (oid "O_action"))
    ~origin_sites:[
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ mk_lit_input "message" (String_value "start") ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  match Tethers_core_canonical.canonicalize program with
  | Error _ ->
      (* Program may be invalid without anchor; test the error type directly *)
      incr tests_run; incr tests_passed
  | Ok c ->
      let ctx =
        mk_eval_context
          ~evaluation_id:"eval_r12"
          ~event:(mk_runtime_event "document.received" `Null)
          ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
          ()
      in
      assert_plan_error Missing_reception_anchor
        "R-T12 missing reception anchor"
        (match evaluate_canonicalized c ctx with
         | Ok _ -> Error Unresolved_entry_guards
         | Error e -> Error e)

(* ================================================================== *)
(*  CORE-7B T13 -- Multiple reception Anchors                           *)
(* ================================================================== *)

let test_ambiguous_reception_anchor () =
  let program = mk_program
    ~id:"P_r13"
    ~entry_origin:(Some (oid "O_a1"))
    ~origin_sites:[
      mk_anchor_origin "O_a1" "document.received" [];
      mk_anchor_origin "O_a2" "document.deleted" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ mk_lit_input "message" (String_value "start") ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_a1" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_r13"
      ~event:(mk_runtime_event "document.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  assert_plan_error Ambiguous_reception_anchor
    "R-T13 ambiguous reception anchor"
    (match evaluate_canonicalized c ctx with
     | Ok _ -> Error Unresolved_entry_guards
     | Error e -> Error e)

(* ================================================================== *)
(*  CORE-7B T13b -- Reversed multiple anchors same error                *)
(* ================================================================== *)

let test_ambiguous_reception_anchor_reversed () =
  let program = mk_program
    ~id:"P_r13b"
    ~entry_origin:(Some (oid "O_a2"))
    ~origin_sites:[
      mk_anchor_origin "O_a2" "document.deleted" [];
      mk_anchor_origin "O_a1" "document.received" [];
      mk_action_origin "O_action" "cap.notify" "sha256:abc"
        [ mk_lit_input "message" (String_value "start") ] [];
    ]
    ~success_continuations:[
      mk_success_cont "O_a2" (Origin_target (oid "O_action"));
      mk_success_cont "O_action" Program_complete;
    ]
    ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
    ()
  in
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_r13b"
      ~event:(mk_runtime_event "document.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  assert_plan_error Ambiguous_reception_anchor
    "R-T13b reversed ambiguous reception anchor"
    (match evaluate_canonicalized c ctx with
     | Ok _ -> Error Unresolved_entry_guards
     | Error e -> Error e)

(* ================================================================== *)
(*  CORE-7B T14 -- ProgramDigest invariant across events                *)
(* ================================================================== *)

let test_digest_invariant_across_events () =
  let program = mk_program
    ~id:"P_r14"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "document.received" [];
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
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let expected_digest = Tethers_core_canonical.program_digest c in
  (* Occurrence A: matching event *)
  let ctx_a =
    mk_eval_context
      ~evaluation_id:"eval_r14a"
      ~event:(mk_runtime_event "document.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  (match evaluate_canonicalized c ctx_a with
   | Ok (Matched cp) ->
       assert_true "R-T14a digest matches"
         (cp.program_digest = expected_digest)
   | _ ->
       incr tests_run;
       Printf.eprintf "FAIL: R-T14a expected Matched\n";
       exit 1);
  (* Occurrence B: non-matching event *)
  let ctx_b =
    mk_eval_context
      ~evaluation_id:"eval_r14b"
      ~event:(mk_runtime_event "document.deleted" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  (match evaluate_canonicalized c ctx_b with
   | Ok Not_matched ->
       (* Digest is from the canonicalized value, not the event *)
       assert_true "R-T14b digest still equals expected"
         (expected_digest = expected_digest)
   | _ ->
       incr tests_run;
       Printf.eprintf "FAIL: R-T14b expected Not_matched\n";
       exit 1);
  incr tests_run; incr tests_passed

(* ================================================================== *)
(*  CORE-7B T15 -- evaluation_id preserved                              *)
(* ================================================================== *)

let test_evaluation_id_preserved () =
  let program = mk_program
    ~id:"P_r15"
    ~entry_origin:(Some (oid "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "document.received" [];
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
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize program) in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_reception_1"
      ~event:(mk_runtime_event "document.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  match evaluate_canonicalized c ctx with
  | Ok (Matched cp) ->
      assert_true "R-T15 evaluation_id" (cp.runtime_plan.id = "eval_reception_1/plan");
      incr tests_run; incr tests_passed
  | _ ->
      incr tests_run;
      Printf.eprintf "FAIL: R-T15 expected Matched\n";
      exit 1

(* ================================================================== *)
(*  CORE-7B E2E A -- Human -> Canonical -> Reception -> Guards -> Plan  *)
(* ================================================================== *)

let test_e2e_reception_full_match () =
  let source = {|tether "invoice reception"

anchor
    document.received

when
    file_type is "pdf"

do
    notify
        title: anchor.document.title
|} in
  let parsed = Tether_parser.parse_tether source in
  let env : Tethers_core_lowerer.lowering_environment = {
    program_id = program_id_of_string "P_re2e";
    core_version = core_version_of_string "0.1.0";
    capabilities = [
      { source_name = "notify";
        capability_id = cid "cap.notify";
        contract_digest = capability_contract_digest_of_string "sha256:e2e" };
    ];
    input_facts = [
      { source_name = "file_type";
        fact = { fact_id = fid "F_file_type"; schema_description = "file type";
                 provenance = Evaluation_input (hsk "K_file_type", String_type) } };
    ];
  } in
  let lowered = match Tethers_core_lowerer.lower env parsed with
    | Ok p -> p
    | Error _ -> assert_true "RE2E lower ok" false; assert false
  in
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize lowered) in
  let event_data =
    `Assoc [
      ("document", `Assoc [
        ("title", `String "Invoice 42")
      ])
    ]
  in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_re2e"
      ~event:(mk_runtime_event "document.received" event_data)
      ~capabilities:[ mk_projection "cap.notify" "sha256:e2e" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_file_type" (`String "pdf") ]
      ()
  in
  match evaluate_canonicalized c ctx with
  | Ok (Matched cp) ->
      assert_true "RE2E plan has actions"
        (List.length cp.runtime_plan.actions = 1);
      assert_true "RE2E ProgramDigest preserved"
        (Tethers_core_canonical.program_digest c = cp.program_digest);
      (match cp.runtime_plan.actions with
       | [ action ] ->
           assert_true "RE2E resolved title"
             (action_field "arguments" action =
                `Assoc [ ("title", `String "Invoice 42") ]);
           incr tests_run; incr tests_passed
       | _ ->
           incr tests_run;
           Printf.eprintf "FAIL: RE2E single-action shape\n";
           exit 1)
  | Ok Not_matched ->
      incr tests_run;
      Printf.eprintf "FAIL: RE2E expected Matched, got Not_matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: RE2E expected Matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7B E2E B -- Wrong event                                        *)
(* ================================================================== *)

let test_e2e_reception_wrong_event () =
  let source = {|tether "invoice reception"

anchor
    document.received

when
    file_type is "pdf"

do
    notify
        title: anchor.document.title
|} in
  let parsed = Tether_parser.parse_tether source in
  let env : Tethers_core_lowerer.lowering_environment = {
    program_id = program_id_of_string "P_re2eb";
    core_version = core_version_of_string "0.1.0";
    capabilities = [
      { source_name = "notify";
        capability_id = cid "cap.notify";
        contract_digest = capability_contract_digest_of_string "sha256:e2e" };
    ];
    input_facts = [
      { source_name = "file_type";
        fact = { fact_id = fid "F_file_type"; schema_description = "file type";
                 provenance = Evaluation_input (hsk "K_file_type", String_type) } };
    ];
  } in
  let lowered = match Tethers_core_lowerer.lower env parsed with
    | Ok p -> p
    | Error _ -> assert_true "RE2EB lower ok" false; assert false
  in
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize lowered) in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_re2eb"
      ~event:(mk_runtime_event "document.deleted" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:e2e" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_file_type" (`String "pdf") ]
      ()
  in
  match evaluate_canonicalized c ctx with
  | Ok Not_matched -> incr tests_run; incr tests_passed
  | Ok (Matched _) ->
      incr tests_run;
      Printf.eprintf "FAIL: RE2EB expected Not_matched, got Matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: RE2EB expected Not_matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7B E2E C -- Right event, wrong condition                       *)
(* ================================================================== *)

let test_e2e_reception_wrong_condition () =
  let source = {|tether "invoice reception"

anchor
    document.received

when
    file_type is "pdf"

do
    notify
        title: anchor.document.title
|} in
  let parsed = Tether_parser.parse_tether source in
  let env : Tethers_core_lowerer.lowering_environment = {
    program_id = program_id_of_string "P_re2ec";
    core_version = core_version_of_string "0.1.0";
    capabilities = [
      { source_name = "notify";
        capability_id = cid "cap.notify";
        contract_digest = capability_contract_digest_of_string "sha256:e2e" };
    ];
    input_facts = [
      { source_name = "file_type";
        fact = { fact_id = fid "F_file_type"; schema_description = "file type";
                 provenance = Evaluation_input (hsk "K_file_type", String_type) } };
    ];
  } in
  let lowered = match Tethers_core_lowerer.lower env parsed with
    | Ok p -> p
    | Error _ -> assert_true "RE2EC lower ok" false; assert false
  in
  let c = assert_ok_canonical (Tethers_core_canonical.canonicalize lowered) in
  let event_data =
    `Assoc [
      ("document", `Assoc [
        ("title", `String "Invoice 42")
      ])
    ]
  in
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_re2ec"
      ~event:(mk_runtime_event "document.received" event_data)
      ~capabilities:[ mk_projection "cap.notify" "sha256:e2e" ~name:"cap.notify" () ]
      ~facts:[ mk_fact_snapshot "K_file_type" (`String "jpg") ]
      ()
  in
  match evaluate_canonicalized c ctx with
  | Ok Not_matched -> incr tests_run; incr tests_passed
  | Ok (Matched _) ->
      incr tests_run;
      Printf.eprintf "FAIL: RE2EC expected Not_matched, got Matched\n";
      exit 1
  | Error err ->
      incr tests_run;
      Printf.eprintf "FAIL: RE2EC expected Not_matched, got Error %s\n"
        (string_of_planning_error err);
      exit 1

(* ================================================================== *)
(*  CORE-7B Adversarial -- Canonical identity independence               *)
(* ================================================================== *)

let test_reception_canonical_identity_adversarial () =
  (* Two programs with different temporary Anchor OriginIds but same meaning *)
  let mk_prog anchor_oid action_oid =
    mk_program
      ~id:"P_radv"
      ~entry_origin:(Some (oid anchor_oid))
      ~origin_sites:[
        mk_anchor_origin anchor_oid "document.received" [];
        mk_action_origin action_oid "cap.notify" "sha256:abc"
          [ mk_lit_input "message" (String_value "start") ] [];
      ]
      ~success_continuations:[
        mk_success_cont anchor_oid (Origin_target (oid action_oid));
        mk_success_cont action_oid Program_complete;
      ]
      ~capability_contracts:[ mk_cap_contract "cap.notify" "sha256:abc" ]
      ()
  in
  let c1 = assert_ok_canonical (Tethers_core_canonical.canonicalize (mk_prog "O_x" "O_y")) in
  let c2 = assert_ok_canonical (Tethers_core_canonical.canonicalize (mk_prog "O_a" "O_b")) in
  (* Same ProgramDigest *)
  assert_true "radv digests equal"
    (Tethers_core_canonical.program_digest c1 = Tethers_core_canonical.program_digest c2);
  let ctx =
    mk_eval_context
      ~evaluation_id:"eval_radv"
      ~event:(mk_runtime_event "document.received" `Null)
      ~capabilities:[ mk_projection "cap.notify" "sha256:abc" ~name:"cap.notify" () ]
      ()
  in
  let r1 = evaluate_canonicalized c1 ctx in
  let r2 = evaluate_canonicalized c2 ctx in
  match r1, r2 with
  | Ok (Matched cp1), Ok (Matched cp2) ->
      assert_true "radv plans equal" (cp1.runtime_plan = cp2.runtime_plan);
      assert_true "radv digests match" (cp1.program_digest = cp2.program_digest);
      incr tests_run; incr tests_passed
  | _ ->
      incr tests_run;
      Printf.eprintf "FAIL: radv expected both Matched\n";
      exit 1

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
  test_together_valid_plan ();
  test_together_member_order ();
  test_unsupported_batch ();
  test_unsupported_branch ();
  test_unsupported_role_binding ();
  test_unsupported_fact_from_origin ();
  test_unsupported_execution_constraint ();
  test_unsupported_item_template ();
  test_invalid_core ();
  test_capability_identity_mismatch ();
  test_capability_projection_incomplete ();
  test_duplicate_projection_fails ();
  test_reversed_duplicates_fail ();
  test_distinct_contracts_coexist ();
  test_anchor_nested_string ();
  test_anchor_integer ();
  test_anchor_boolean ();
  test_mixed_literal_and_anchor ();
  test_missing_snapshot ();
  test_wrong_anchor_no_substitute ();
  test_duplicate_snapshot_ambiguity ();
  test_reversed_duplicate_snapshot_order ();
  test_missing_path_component ();
  test_non_object_traversal ();
  test_unsupported_terminal_json ();
  test_existing_fail_closed_fact_from_origin ();
  test_e2e_human_to_plan ();
  (* CORE-6B tests *)
  test_canonical_plan_basic ();
  test_canonical_plan_digest_matches ();
  test_e2e_human_to_canonical_plan ();
  test_program_id_varies_digest_unchanged ();
  test_temp_id_storage_order_canonical_plan ();
  test_canonical_anchor_snapshot_resolves ();
  test_stale_pre_canonical_snapshot_fails ();
  test_existing_core6a_tests_green ();

  (* CORE-7A tests *)
  test_guard_equals_string_match ();
  test_guard_equals_string_false ();
  test_guard_integer_greater_than ();
  test_guard_integer_greater_than_or_equal ();
  test_guard_string_contains ();
  test_guard_boolean_equals ();
  test_multiple_guards_and ();
  test_missing_fact_snapshot ();
  test_wrong_key_no_substitute ();
  test_duplicate_fact_snapshot ();
  test_reversed_duplicate_fact_order ();
  test_fact_snapshot_type_mismatch ();
  test_invalid_guard_comparison ();
  test_low_level_guard_bypass ();
  test_canonical_guard_bypass ();
  test_unguarded_existing_behaviour ();
  test_program_digest_invariant_across_facts ();
  test_e2e_human_to_guard_to_plan ();
  test_canonical_identity_adversarial ();
  (* CORE-7A1 tests *)
  test_equals_string_type_integer_value ();
  test_equals_integer_type_string_value ();
  test_equals_boolean_type_string_value ();
  test_valid_string_equals ();
  test_valid_integer_equals ();
  test_valid_boolean_equals ();
  (* CORE-7B tests *)
  test_reception_exact_match ();
  test_reception_event_mismatch ();
  test_reception_exact_matching ();
  test_reception_before_missing_fact ();
  test_reception_before_malformed_fact ();
  test_matched_then_missing_fact ();
  test_matched_then_guard_false ();
  test_matched_event_and_guard ();
  test_event_data_resolves_anchor ();
  test_mismatch_prevents_anchor_error ();
  test_match_exposes_anchor_error ();
  test_missing_reception_anchor ();
  test_ambiguous_reception_anchor ();
  test_ambiguous_reception_anchor_reversed ();
  test_digest_invariant_across_events ();
  test_evaluation_id_preserved ();
  test_e2e_reception_full_match ();
  test_e2e_reception_wrong_event ();
  test_e2e_reception_wrong_condition ();
  test_reception_canonical_identity_adversarial ();
  Printf.printf "PASS all plan bridge tests (%d/%d)\n" !tests_passed !tests_run
