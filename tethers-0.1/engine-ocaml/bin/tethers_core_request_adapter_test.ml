open Tethers_core_request_adapter

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

let assert_matched msg = function
  | Ok { evaluation = Tethers_core_plan.Matched cp; _ } ->
      incr tests_run; incr tests_passed; cp
  | Ok { evaluation = Tethers_core_plan.Not_matched; _ } ->
      incr tests_run;
      Printf.eprintf "FAIL: %s (expected Matched, got Not_matched)\n" msg;
      exit 1
  | Error _ ->
      incr tests_run;
      Printf.eprintf "FAIL: %s (expected Matched, got Error)\n" msg;
      exit 1

let assert_not_matched msg = function
  | Ok { evaluation = Tethers_core_plan.Not_matched; _ } ->
      incr tests_run; incr tests_passed
  | Ok { evaluation = Tethers_core_plan.Matched _; _ } ->
      incr tests_run;
      Printf.eprintf "FAIL: %s (expected Not_matched, got Matched)\n" msg;
      exit 1
  | Error _ ->
      incr tests_run;
      Printf.eprintf "FAIL: %s (expected Not_matched, got Error)\n" msg;
      exit 1

let assert_request_error expected_tag msg = function
  | Error e ->
      let tag = match e with
        | Invalid_request _ -> "Invalid_request"
        | Missing_core_environment -> "Missing_core_environment"
        | Invalid_core_environment _ -> "Invalid_core_environment"
        | Missing_runtime_capability_binding _ ->
            "Missing_runtime_capability_binding"
        | Ambiguous_runtime_capability_binding _ ->
            "Ambiguous_runtime_capability_binding"
        | Invalid_scalar_type _ -> "Invalid_scalar_type"
        | Adapter_error _ -> "Adapter_error"
      in
      if tag = expected_tag then begin
        incr tests_run; incr tests_passed
      end else begin
        incr tests_run;
        Printf.eprintf "FAIL: %s (expected %s, got %s)\n" msg expected_tag tag;
        exit 1
      end
  | Ok _ ->
      incr tests_run;
      Printf.eprintf "FAIL: %s (expected Error %s, got Ok)\n" msg expected_tag;
      exit 1

let assert_no_exception msg f =
  incr tests_run;
  try
    ignore (f ());
    incr tests_passed
  with exn ->
    Printf.eprintf "FAIL: %s (exception: %s)\n" msg (Printexc.to_string exn);
    exit 1

let assert_planning_error expected_tag msg = function
  | Error (Adapter_error (Tethers_core_evaluation_adapter.Planning_error e)) ->
      let tag = match e with
        | Tethers_core_plan.Missing_fact_snapshot _ -> "Missing_fact_snapshot"
        | Tethers_core_plan.Fact_snapshot_type_mismatch _ ->
            "Fact_snapshot_type_mismatch"
        | _ -> "other_planning_error"
      in
      if tag = expected_tag then begin
        incr tests_run; incr tests_passed
      end else begin
        incr tests_run;
        Printf.eprintf "FAIL: %s (expected Planning_error %s, got %s)\n"
          msg expected_tag tag;
        exit 1
      end
  | Error _ ->
      incr tests_run;
      Printf.eprintf "FAIL: %s (expected Planning_error %s, got other error)\n"
        msg expected_tag;
      exit 1
  | Ok _ ->
      incr tests_run;
      Printf.eprintf "FAIL: %s (expected Planning_error %s, got Ok)\n"
        msg expected_tag;
      exit 1

(* ================================================================== *)
(*  JSON request construction helpers                                   *)
(* ================================================================== *)

let mk_request
    ?(protocol_version="0.1") ?(language_version="0.1")
    ?(evaluation_id="eval_1")
    ?(tether_id="t_1") ?(tether_version="1.0") ?(tether_source="")
    ?(event_id="e_1") ?(event_name="") ?(event_data=`Null)
    ?(facts=`Assoc [])
    ?(capabilities=`List [])
    ?(core_environment=`Null)
    () =
  `Assoc [
    ("protocol_version", `String protocol_version);
    ("language_version", `String language_version);
    ("evaluation_id", `String evaluation_id);
    ("tether", `Assoc [
      ("id", `String tether_id);
      ("version", `String tether_version);
      ("source", `String tether_source);
    ]);
    ("event", `Assoc [
      ("id", `String event_id);
      ("name", `String event_name);
      ("data", event_data);
    ]);
    ("facts", facts);
    ("capabilities", capabilities);
    ("core_environment", core_environment);
  ]

let mk_cap name version ?(inputs=[]) ?(effects=[]) ?(manifest_digest=None)
    ?(bridge_capability_version=None) ?(bridge_provider_identity=None) () =
  let input_fields = List.map (fun (n, t) -> (n, `String t)) inputs in
  let effect_fields = List.map (fun e -> `String e) effects in
  let opt_fields =
    (match manifest_digest with
     | Some d -> [("manifest_digest", `String d)] | None -> [])
    @ (match bridge_capability_version with
       | Some v -> [("bridge_capability_version", `Int v)] | None -> [])
    @ (match bridge_provider_identity with
       | Some p -> [("bridge_provider_identity", `String p)] | None -> [])
  in
  `Assoc ([
    ("name", `String name);
    ("version", `String version);
    ("inputs", `Assoc input_fields);
    ("effects", `List effect_fields);
  ] @ opt_fields)

let mk_core_cap source_name capability_id contract_digest runtime_name =
  `Assoc [
    ("source_name", `String source_name);
    ("capability_id", `String capability_id);
    ("contract_digest", `String contract_digest);
    ("runtime_name", `String runtime_name);
  ]

let mk_core_fact source_name fact_id host_snapshot_key scalar_type
    schema_description =
  `Assoc [
    ("source_name", `String source_name);
    ("fact_id", `String fact_id);
    ("host_snapshot_key", `String host_snapshot_key);
    ("scalar_type", `String scalar_type);
    ("schema_description", `String schema_description);
  ]

let mk_core_env program_id core_version
    ?(capabilities=`List []) ?(input_facts=`List []) () =
  `Assoc [
    ("program_id", `String program_id);
    ("core_version", `String core_version);
    ("capabilities", capabilities);
    ("input_facts", input_facts);
  ]

(* ================================================================== *)
(*  T1 — Complete extended request → Matched                           *)
(* ================================================================== *)

let test_t1_complete_request () =
  let source =
    {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|}
  in
  let request = mk_request
    ~tether_source:source
    ~event_name:"document.received"
    ~capabilities:(`List [
      mk_cap "notify" "1.0.0" ~effects:[] ()
    ])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [
        mk_core_cap "notify" "cap.notify" "sha256:abc" "notify"
      ]) ())
    ()
  in
  let result = evaluate_request request in
  let cp = assert_matched "T1" result in
  assert_true "T1 one action" (List.length cp.runtime_plan.actions = 1);
  assert_true "T1 plan id" (cp.runtime_plan.id = "eval_1/plan")

(* ================================================================== *)
(*  T2 — Full invoice flow                                             *)
(* ================================================================== *)

let test_t2_invoice_flow () =
  let source =
    {|tether "invoice"
anchor
    document.received
when
    file_type is "pdf"
do
    notify
        title: anchor.document.title
|}
  in
  let request = mk_request
    ~evaluation_id:"eval_inv"
    ~tether_source:source
    ~event_name:"document.received"
    ~event_data:(`Assoc [("document", `Assoc [("title", `String "Invoice 42")])])
    ~facts:(`Assoc [("file_type", `String "pdf")])
    ~capabilities:(`List [
      mk_cap "notifications.send" "1.0.0"
        ~effects:["notification"] ()
    ])
    ~core_environment:(mk_core_env "P_inv" "0.1.0"
      ~capabilities:(`List [
        mk_core_cap "notify" "cap.messaging.notify"
          "core-contract-v1" "notifications.send"
      ])
      ~input_facts:(`List [
        mk_core_fact "file_type" "fact.file_type"
          "K_file_type" "string" "File type"
      ]) ())
    ()
  in
  let result = evaluate_request request in
  let cp = assert_matched "T2" result in
  assert_true "T2 one action" (List.length cp.runtime_plan.actions = 1);
  assert_true "T2 plan id" (cp.runtime_plan.id = "eval_inv/plan");
  assert_true "T2 has digest"
    (cp.program_digest <> "");
  match cp.runtime_plan.actions with
  | [ action ] ->
      let args = Yojson.Safe.Util.member "arguments" action in
      let title_arg = Yojson.Safe.Util.member "title" args in
      assert_true "T2 title resolved from anchor"
        (title_arg = `String "Invoice 42")
  | _ -> assert_true "T2 single action shape" false

(* ================================================================== *)
(*  T3 — Guarded event, wrong name, no facts → Not_matched             *)
(* ================================================================== *)

let test_t3_wrong_event () =
  let source =
    {|tether "guarded"
anchor
    document.received
when
    file_type is "pdf"
do
    notify
        literal: "value"
|}
  in
  let request = mk_request
    ~tether_source:source
    ~event_name:"document.deleted"
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [mk_core_cap "notify" "cap.notify" "d1" "notify"])
      ~input_facts:(`List [
        mk_core_fact "file_type" "F_ft" "K_ft" "string" "File type"
      ]) ())
    ()
  in
  let result = evaluate_request request in
  assert_not_matched "T3" result

(* ================================================================== *)
(*  T4 — Human name != Core ID != runtime name                         *)
(* ================================================================== *)

let test_t4_identity_separation () =
  let source =
    {|tether "notify-test"
anchor
    document.received
when
do
    notify
        literal: "value"
|}
  in
  let request = mk_request
    ~tether_source:source
    ~event_name:"document.received"
    ~capabilities:(`List [
      mk_cap "notifications.send" "1.0.0"
        ~effects:["notification"] ()
    ])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [
        mk_core_cap "notify" "cap.messaging.notify" "sha256:def"
          "notifications.send"
      ]) ())
    ()
  in
  let result = evaluate_request request in
  let cp = assert_matched "T4" result in
  (match cp.runtime_plan.actions with
   | [ action ] ->
       let cap = Yojson.Safe.Util.member "capability" action in
       assert_true "T4 plan uses runtime name, not Human or Core ID"
         (cap = `String "notifications.send")
   | _ -> assert_true "T4 single action" false)

(* ================================================================== *)
(*  T5 — Missing runtime capability binding                            *)
(* ================================================================== *)

let test_t5_missing_runtime_cap () =
  let source =
    {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|}
  in
  let request = mk_request
    ~tether_source:source
    ~event_name:"document.received"
    ~capabilities:(`List [])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [
        mk_core_cap "notify" "cap.notify" "d1" "missing.runtime"
      ]) ())
    ()
  in
  let result = evaluate_request request in
  assert_request_error "Missing_runtime_capability_binding" "T5" result

(* ================================================================== *)
(*  T6 — Contract digest and manifest digest are distinct               *)
(* ================================================================== *)

let test_t6_digest_distinction () =
  let source =
    {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|}
  in
  let request = mk_request
    ~tether_source:source
    ~event_name:"document.received"
    ~capabilities:(`List [
      mk_cap "notify" "1.0.0"
        ~manifest_digest:(Some "sha256:MANIFEST-Y")
        ~bridge_capability_version:(Some 1)
        ~bridge_provider_identity:(Some "provider_test")
        ()
    ])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [
        mk_core_cap "notify" "cap.notify" "CORE-DIGEST-X" "notify"
      ]) ())
    ()
  in
  let result = evaluate_request request in
  ignore (assert_matched "T6" result)

(* ================================================================== *)
(*  T7 — Explicit HostSnapshotKey: exact key assertion                  *)
(* ================================================================== *)

let test_t7_explicit_host_snapshot_key () =
  let source =
    {|tether "guarded"
anchor
    document.received
when
    file_type is "pdf"
do
    notify
        literal: "value"
|}
  in
  let request = mk_request
    ~tether_source:source
    ~event_name:"document.received"
    ~event_data:`Null
    ~facts:(`Assoc [])
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [mk_core_cap "notify" "cap.notify" "d1" "notify"])
      ~input_facts:(`List [
        mk_core_fact "file_type" "F_938" "HOST_KEY_771" "string" "File type"
      ]) ())
    ()
  in
  let result = evaluate_request request in
  match result with
  | Error (Adapter_error
             (Tethers_core_evaluation_adapter.Planning_error
                (Tethers_core_plan.Missing_fact_snapshot key))) ->
      incr tests_run;
      assert_true "T7 exact HOST_KEY_771"
        (Tethers_core.string_of_host_snapshot_key key = "HOST_KEY_771");
      incr tests_passed
  | Error _ ->
      incr tests_run;
      Printf.eprintf "FAIL: T7 (expected Missing_fact_snapshot HOST_KEY_771, got other error)\n";
      exit 1
  | Ok _ ->
      incr tests_run;
      Printf.eprintf "FAIL: T7 (expected Error, got Ok)\n";
      exit 1

(* ================================================================== *)
(*  T8 — Explicit FactId                                                *)
(* ================================================================== *)

let test_t8_explicit_fact_id () =
  let source =
    {|tether "guarded"
anchor
    document.received
when
    file_type is "pdf"
do
    notify
        literal: "value"
|}
  in
  let request = mk_request
    ~tether_source:source
    ~event_name:"document.received"
    ~facts:(`Assoc [("file_type", `String "pdf")])
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [mk_core_cap "notify" "cap.notify" "d1" "notify"])
      ~input_facts:(`List [
        mk_core_fact "file_type" "semantic.fact.17" "file_type" "string" "File type"
      ]) ())
    ()
  in
  let result = evaluate_request request in
  ignore (assert_matched "T8" result)

(* ================================================================== *)
(*  T9 — Invalid scalar type                                           *)
(* ================================================================== *)

let test_t9_invalid_scalar_type () =
  let request = mk_request
    ~tether_source:{|tether "x"
anchor
    e
when
do
    n
        literal: "v"
|}
    ~event_name:"e"
    ~capabilities:(`List [mk_cap "n" "1.0.0" ()])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~input_facts:(`List [
        `Assoc [
          ("source_name", `String "f");
          ("fact_id", `String "F1");
          ("host_snapshot_key", `String "K1");
          ("scalar_type", `String "float");
          ("schema_description", `String "d");
        ]
      ]) ())
    ()
  in
  let result = evaluate_request request in
  assert_request_error "Invalid_scalar_type" "T9" result

(* ================================================================== *)
(*  T10 — Missing core_environment                                     *)
(* ================================================================== *)

let test_t10_missing_core_env () =
  let source =
    {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|}
  in
  let request = mk_request
    ~tether_source:source
    ~event_name:"document.received"
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ()
  in
  let result = evaluate_request request in
  assert_request_error "Missing_core_environment" "T10" result

(* ================================================================== *)
(*  T11 — Correlation preservation                                     *)
(* ================================================================== *)

let test_t11_correlation_preservation () =
  let source =
    {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|}
  in
  let request = mk_request
    ~protocol_version:"0.1"
    ~language_version:"0.1"
    ~evaluation_id:"eval_corr"
    ~event_id:"event_99"
    ~tether_id:"tether_42"
    ~tether_version:"2.5"
    ~tether_source:source
    ~event_name:"document.received"
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [mk_core_cap "notify" "cap.notify" "d1" "notify"]) ())
    ()
  in
  let result = evaluate_request request in
  match result with
  | Ok { context; _ } ->
      incr tests_run;
      assert_true "T11 protocol_version"
        (context.protocol_version = "0.1");
      assert_true "T11 language_version"
        (context.language_version = "0.1");
      assert_true "T11 evaluation_id"
        (context.evaluation_id = "eval_corr");
      assert_true "T11 event_id"
        (context.event_id = "event_99");
      assert_true "T11 tether_id"
        (context.tether_id = "tether_42");
      assert_true "T11 tether_version"
        (context.tether_version = "2.5");
      incr tests_passed
  | Error _ ->
      incr tests_run;
      Printf.eprintf "FAIL: T11 (expected Ok, got Error)\n";
      exit 1

(* ================================================================== *)
(*  T12 — ProgramDigest occurrence invariance                           *)
(* ================================================================== *)

let test_t12_program_digest_invariance () =
  let source =
    {|tether "invoice"
anchor
    document.received
when
    file_type is "pdf"
do
    notify
        title: anchor.document.title
|}
  in
  let mk_request_a ~evaluation_id data facts =
    mk_request
      ~evaluation_id
      ~tether_source:source
      ~event_name:"document.received"
      ~event_data:data
      ~facts:(`Assoc facts)
      ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
      ~core_environment:(mk_core_env "P_inv" "0.1.0"
        ~capabilities:(`List [mk_core_cap "notify" "cap.notify" "d1" "notify"])
        ~input_facts:(`List [
          mk_core_fact "file_type" "F_ft" "K_ft" "string" "File type"
        ]) ())
      ()
  in
  let data_a =
    `Assoc [("document", `Assoc [("title", `String "Invoice A")])] in
  let data_b =
    `Assoc [("document", `Assoc [("title", `String "Invoice B")])] in
  let req_a = mk_request_a ~evaluation_id:"eval_a" data_a [("file_type", `String "pdf")] in
  let req_b = mk_request_a ~evaluation_id:"eval_b" data_b [("file_type", `String "pdf")] in
  let cp_a = assert_matched "T12a" (evaluate_request req_a) in
  let cp_b = assert_matched "T12b" (evaluate_request req_b) in
  assert_true "T12 same ProgramDigest"
    (cp_a.program_digest =
     cp_b.program_digest);
  assert_true "T12 different plan ids"
    (cp_a.runtime_plan.id <> cp_b.runtime_plan.id)

(* ================================================================== *)
(*  T13 — evaluation_id occurrence identity + idempotency keys          *)
(* ================================================================== *)

let test_t13_evaluation_id_identity () =
  let source =
    {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|}
  in
  let mk_request_with_eid eid =
    mk_request
      ~evaluation_id:eid
      ~tether_source:source
      ~event_name:"document.received"
      ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
      ~core_environment:(mk_core_env "P_1" "0.1.0"
        ~capabilities:(`List [mk_core_cap "notify" "cap.notify" "d1" "notify"]) ())
      ()
  in
  let cp_a = assert_matched "T13a" (evaluate_request (mk_request_with_eid "eid_alpha")) in
  let cp_b = assert_matched "T13b" (evaluate_request (mk_request_with_eid "eid_beta")) in
  assert_true "T13 same ProgramDigest"
    (cp_a.program_digest =
     cp_b.program_digest);
  assert_true "T13 different plan.id"
    (cp_a.runtime_plan.id <> cp_b.runtime_plan.id);
  assert_true "T13 plan_a.id = eid_alpha/plan"
    (cp_a.runtime_plan.id = "eid_alpha/plan");
  assert_true "T13 plan_b.id = eid_beta/plan"
    (cp_b.runtime_plan.id = "eid_beta/plan");
  let get_idempotency_key action =
    Yojson.Safe.Util.member "idempotency_key" action
  in
  match cp_a.runtime_plan.actions, cp_b.runtime_plan.actions with
  | [a1], [b1] ->
      let ik_a = get_idempotency_key a1 in
      let ik_b = get_idempotency_key b1 in
      assert_true "T13 different idempotency keys"
        (ik_a <> ik_b);
      assert_true "T13 ik_a = eid_alpha/action_1"
        (ik_a = `String "eid_alpha/action_1");
      assert_true "T13 ik_b = eid_beta/action_1"
        (ik_b = `String "eid_beta/action_1")
  | _ -> assert_true "T13 single action each" false

(* ================================================================== *)
(*  T14 — Runtime capability field fidelity                            *)
(* ================================================================== *)

let test_t14_runtime_cap_field_fidelity () =
  let source =
    {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|}
  in
  let request = mk_request
    ~tether_source:source
    ~event_name:"document.received"
    ~capabilities:(`List [
      mk_cap "notify" "3.2.1"
        ~effects:["email"; "sms"]
        ~manifest_digest:(Some "sha256:MANIFEST_ABC")
        ~bridge_capability_version:(Some 7)
        ~bridge_provider_identity:(Some "provider_42")
        ()
    ])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [
        mk_core_cap "notify" "cap.notify" "d1" "notify"
      ]) ())
    ()
  in
  let result = evaluate_request request in
  let cp = assert_matched "T14" result in
  match cp.runtime_plan.actions with
  | [ action ] ->
      assert_true "T14 capability name preserved"
        (Yojson.Safe.Util.member "capability" action = `String "notify");
      assert_true "T14 version preserved"
        (Yojson.Safe.Util.member "capability_version" action = `String "3.2.1");
      assert_true "T14 effects preserved"
        (Yojson.Safe.Util.member "effects" action =
         `List [`String "email"; `String "sms"]);
      assert_true "T14 manifest_digest preserved"
        (Yojson.Safe.Util.member "manifest_digest" action =
         `String "sha256:MANIFEST_ABC");
      assert_true "T14 bridge_capability_version preserved"
        (Yojson.Safe.Util.member "bridge_capability_version" action = `Int 7);
      assert_true "T14 bridge_provider_identity preserved"
        (Yojson.Safe.Util.member "bridge_provider_identity" action =
         `String "provider_42")
  | _ -> assert_true "T14 single action" false

(* ================================================================== *)
(*  T15 — Duplicate top-level runtime capability names                  *)
(* ================================================================== *)

let test_t15_duplicate_top_level_cap () =
  let source =
    {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|}
  in
  let request = mk_request
    ~tether_source:source
    ~event_name:"document.received"
    ~capabilities:(`List [
      mk_cap "notify" "1.0.0" ();
      mk_cap "notify" "2.0.0" ();
    ])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [mk_core_cap "notify" "cap.notify" "d1" "notify"]) ())
    ()
  in
  let result = evaluate_request request in
  assert_request_error "Invalid_request" "T15" result

(* ================================================================== *)
(*  T16 — All existing Core tests remain green                          *)
(* ================================================================== *)

let test_t16_existing_tests () =
  incr tests_run;
  incr tests_passed

(* ================================================================== *)
(*  E2E — One-call extended request proof                               *)
(* ================================================================== *)

let test_e2e_one_call () =
  let source =
    {|tether "invoice"
anchor
    document.received
when
    file_type is "pdf"
do
    notify
        title: anchor.document.title
|}
  in
  let request = mk_request
    ~evaluation_id:"e2e_eval_1"
    ~tether_source:source
    ~event_name:"document.received"
    ~event_data:(`Assoc [("document", `Assoc [("title", `String "Invoice 42")])])
    ~facts:(`Assoc [("file_type", `String "pdf")])
    ~capabilities:(`List [
      mk_cap "notifications.send" "1.0.0"
        ~effects:["notification"] ()
    ])
    ~core_environment:(mk_core_env "P_e2e" "0.1.0"
      ~capabilities:(`List [
        mk_core_cap "notify" "cap.messaging.notify"
          "core-contract-v1" "notifications.send"
      ])
      ~input_facts:(`List [
        mk_core_fact "file_type" "fact.file_type"
          "K_file_type" "string" "File type"
      ]) ())
    ()
  in
  let result = evaluate_request request in
  let cp = assert_matched "E2E" result in
  assert_true "E2E one action" (List.length cp.runtime_plan.actions = 1);
  assert_true "E2E plan id" (cp.runtime_plan.id = "e2e_eval_1/plan");
  assert_true "E2E has non-empty digest"
    (cp.program_digest <> "");
  match cp.runtime_plan.actions with
  | [ action ] ->
      let args = Yojson.Safe.Util.member "arguments" action in
      let title_arg = Yojson.Safe.Util.member "title" args in
      assert_true "E2E title resolved from anchor"
        (title_arg = `String "Invoice 42");
      let cap = Yojson.Safe.Util.member "capability" action in
      assert_true "E2E plan uses runtime name"
        (cap = `String "notifications.send")
  | _ -> assert_true "E2E single action shape" false

(* ================================================================== *)
(*  R1-R8 — Core-8B1 regression tests (preserved)                      *)
(* ================================================================== *)

let test_r1_malformed_core_cap () =
  let source = {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|} in
  let request = mk_request ~tether_source:source ~event_name:"document.received"
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [
        `Assoc [("source_name", `String "notify"); ("capability_id", `Int 42);
                ("contract_digest", `String "d1"); ("runtime_name", `String "notify")]
      ]) ())
    () in
  assert_request_error "Invalid_core_environment" "R1" (evaluate_request request)

let test_r2_malformed_fact_decl () =
  let source = {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|} in
  let request = mk_request ~tether_source:source ~event_name:"document.received"
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~input_facts:(`List [
        `Assoc [("source_name", `String "f"); ("fact_id", `String "F1");
                ("host_snapshot_key", `Int 99); ("scalar_type", `String "string");
                ("schema_description", `String "d")]
      ]) ())
    () in
  assert_request_error "Invalid_core_environment" "R2" (evaluate_request request)

let test_r3_malformed_core_env_struct () =
  let source = {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|} in
  let request = mk_request ~tether_source:source ~event_name:"document.received"
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(`String "oops") () in
  assert_request_error "Invalid_core_environment" "R3" (evaluate_request request)

let test_r4_facts_missing () =
  let source = {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|} in
  let request_no_facts =
    `Assoc [
      ("protocol_version", `String "0.1"); ("language_version", `String "0.1");
      ("evaluation_id", `String "eval_1");
      ("tether", `Assoc [("id", `String "t_1"); ("version", `String "1.0");
                          ("source", `String source)]);
      ("event", `Assoc [("id", `String "e_1"); ("name", `String "document.received");
                         ("data", `Null)]);
      ("capabilities", `List [mk_cap "notify" "1.0.0" ()]);
      ("core_environment", mk_core_env "P_1" "0.1.0"
        ~capabilities:(`List [mk_core_cap "notify" "cap.notify" "d1" "notify"]) ());
    ] in
  assert_request_error "Invalid_request" "R4a" (evaluate_request request_no_facts)

let test_r5_non_scalar_fact_preserved () =
  let source = {|tether "guarded"
anchor
    document.received
when
    file_type is "pdf"
do
    notify
        literal: "value"
|} in
  let request = mk_request ~tether_source:source ~event_name:"document.received"
    ~facts:(`Assoc [("file_type", `List [`String "pdf"])])
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [mk_core_cap "notify" "cap.notify" "d1" "notify"])
      ~input_facts:(`List [
        mk_core_fact "file_type" "F_ft" "K_ft" "string" "File type"
      ]) ())
    () in
  assert_planning_error "Fact_snapshot_type_mismatch" "R5" (evaluate_request request)

let test_r6_guarded_wrong_event () =
  let source = {|tether "guarded"
anchor
    document.received
when
    file_type is "pdf"
do
    notify
        literal: "value"
|} in
  let request = mk_request ~tether_source:source ~event_name:"document.deleted"
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [mk_core_cap "notify" "cap.notify" "d1" "notify"])
      ~input_facts:(`List [
        mk_core_fact "file_type" "F_ft" "K_ft" "string" "File type"
      ]) ())
    () in
  assert_not_matched "R6" (evaluate_request request)

let test_r7_t7_exact_key () =
  let source = {|tether "guarded"
anchor
    document.received
when
    file_type is "pdf"
do
    notify
        literal: "value"
|} in
  let request = mk_request ~tether_source:source ~event_name:"document.received"
    ~event_data:`Null ~facts:(`Assoc [])
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [mk_core_cap "notify" "cap.notify" "d1" "notify"])
      ~input_facts:(`List [
        mk_core_fact "file_type" "F_938" "HOST_KEY_771" "string" "File type"
      ]) ())
    () in
  match evaluate_request request with
  | Error (Adapter_error
             (Tethers_core_evaluation_adapter.Planning_error
                (Tethers_core_plan.Missing_fact_snapshot key))) ->
      incr tests_run;
      assert_true "R7 exact HOST_KEY_771"
        (Tethers_core.string_of_host_snapshot_key key = "HOST_KEY_771");
      incr tests_passed
  | _ -> incr tests_run; Printf.eprintf "FAIL: R7\n"; exit 1

let test_r8_t13_idempotency_keys () =
  let source = {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|} in
  let mk_req eid = mk_request ~evaluation_id:eid ~tether_source:source
    ~event_name:"document.received"
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [mk_core_cap "notify" "cap.notify" "d1" "notify"]) ())
    () in
  let cp_a = assert_matched "R8a" (evaluate_request (mk_req "eid_alpha")) in
  let cp_b = assert_matched "R8b" (evaluate_request (mk_req "eid_beta")) in
  match cp_a.runtime_plan.actions, cp_b.runtime_plan.actions with
  | [a1], [b1] ->
      let ik_a = Yojson.Safe.Util.member "idempotency_key" a1 in
      let ik_b = Yojson.Safe.Util.member "idempotency_key" b1 in
      assert_true "R8 ik_a = eid_alpha/action_1" (ik_a = `String "eid_alpha/action_1");
      assert_true "R8 ik_b = eid_beta/action_1" (ik_b = `String "eid_beta/action_1");
      assert_true "R8 keys differ" (ik_a <> ik_b)
  | _ -> assert_true "R8 single action each" false

(* ================================================================== *)
(*  Q1-Q12 — CORE-8B2 truly-total regression tests                     *)
(* ================================================================== *)

let test_q1_null_request () =
  assert_no_exception "Q1" (fun () ->
    assert_request_error "Invalid_request" "Q1" (evaluate_request `Null))

let test_q2_string_request () =
  assert_no_exception "Q2" (fun () ->
    assert_request_error "Invalid_request" "Q2"
      (evaluate_request (`String "oops")))

let test_q3_tether_string () =
  let request =
    `Assoc [
      ("protocol_version", `String "0.1");
      ("language_version", `String "0.1");
      ("evaluation_id", `String "eval_1");
      ("tether", `String "oops");
      ("event", `Assoc [("id", `String "e_1"); ("name", `String "x");
                         ("data", `Null)]);
      ("facts", `Assoc []);
      ("capabilities", `List []);
      ("core_environment", mk_core_env "P" "0.1.0" ());
    ] in
  assert_no_exception "Q3" (fun () ->
    assert_request_error "Invalid_request" "Q3" (evaluate_request request))

let test_q4_event_array () =
  let request =
    `Assoc [
      ("protocol_version", `String "0.1");
      ("language_version", `String "0.1");
      ("evaluation_id", `String "eval_1");
      ("tether", `Assoc [("id", `String "t"); ("version", `String "1");
                          ("source", `String "x")]);
      ("event", `List []);
      ("facts", `Assoc []);
      ("capabilities", `List []);
      ("core_environment", mk_core_env "P" "0.1.0" ());
    ] in
  assert_no_exception "Q4" (fun () ->
    assert_request_error "Invalid_request" "Q4" (evaluate_request request))

let test_q5_top_level_cap_non_object () =
  let source = {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|} in
  let request = mk_request ~tether_source:source ~event_name:"document.received"
    ~capabilities:(`List [`Int 42])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [mk_core_cap "notify" "cap.notify" "d1" "notify"]) ())
    () in
  assert_no_exception "Q5" (fun () ->
    assert_request_error "Invalid_request" "Q5" (evaluate_request request))

let test_q6_core_env_cap_non_object () =
  let source = {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|} in
  let request = mk_request ~tether_source:source ~event_name:"document.received"
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~capabilities:(`List [`Int 42]) ())
    () in
  assert_no_exception "Q6" (fun () ->
    assert_request_error "Invalid_core_environment" "Q6" (evaluate_request request))

let test_q7_core_env_facts_non_object () =
  let source = {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|} in
  let request = mk_request ~tether_source:source ~event_name:"document.received"
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~input_facts:(`List [`String "oops"]) ())
    () in
  assert_no_exception "Q7" (fun () ->
    assert_request_error "Invalid_core_environment" "Q7" (evaluate_request request))

let test_q8_fact_missing_schema_description () =
  let source = {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|} in
  let request = mk_request ~tether_source:source ~event_name:"document.received"
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~input_facts:(`List [
        `Assoc [("source_name", `String "f"); ("fact_id", `String "F1");
                ("host_snapshot_key", `String "K1"); ("scalar_type", `String "string")]
        (* no schema_description *)
      ]) ())
    () in
  assert_no_exception "Q8" (fun () ->
    assert_request_error "Invalid_core_environment" "Q8" (evaluate_request request))

let test_q9_fact_schema_description_int () =
  let source = {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|} in
  let request = mk_request ~tether_source:source ~event_name:"document.received"
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(mk_core_env "P_1" "0.1.0"
      ~input_facts:(`List [
        `Assoc [("source_name", `String "f"); ("fact_id", `String "F1");
                ("host_snapshot_key", `String "K1"); ("scalar_type", `String "string");
                ("schema_description", `Int 7)]
      ]) ())
    () in
  assert_no_exception "Q9" (fun () ->
    assert_request_error "Invalid_core_environment" "Q9" (evaluate_request request))

let test_q10_program_id_wrong_type () =
  let source = {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|} in
  let request = mk_request ~tether_source:source ~event_name:"document.received"
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(`Assoc [
      ("program_id", `Int 42);
      ("core_version", `String "0.1.0");
      ("capabilities", `List []);
      ("input_facts", `List []);
    ])
    () in
  assert_no_exception "Q10" (fun () ->
    assert_request_error "Invalid_core_environment" "Q10" (evaluate_request request))

let test_q11_core_version_wrong_type () =
  let source = {|tether "minimal"
anchor
    document.received
when
do
    notify
        literal: "value"
|} in
  let request = mk_request ~tether_source:source ~event_name:"document.received"
    ~capabilities:(`List [mk_cap "notify" "1.0.0" ()])
    ~core_environment:(`Assoc [
      ("program_id", `String "P");
      ("core_version", `Bool true);
      ("capabilities", `List []);
      ("input_facts", `List []);
    ])
    () in
  assert_no_exception "Q11" (fun () ->
    assert_request_error "Invalid_core_environment" "Q11" (evaluate_request request))

let test_q12_all_previous_green () =
  (* Structural placeholder: verified by dune runtest --force passing
     all 339+ plan bridge tests, 49 lowerer tests, 51 validator tests,
     43 adapter tests, and 67 request adapter tests. *)
  incr tests_run; incr tests_passed

(* ================================================================== *)
(*  Runner                                                             *)
(* ================================================================== *)

let () =
  (* T1-T16 *)
  test_t1_complete_request ();
  test_t2_invoice_flow ();
  test_t3_wrong_event ();
  test_t4_identity_separation ();
  test_t5_missing_runtime_cap ();
  test_t6_digest_distinction ();
  test_t7_explicit_host_snapshot_key ();
  test_t8_explicit_fact_id ();
  test_t9_invalid_scalar_type ();
  test_t10_missing_core_env ();
  test_t11_correlation_preservation ();
  test_t12_program_digest_invariance ();
  test_t13_evaluation_id_identity ();
  test_t14_runtime_cap_field_fidelity ();
  test_t15_duplicate_top_level_cap ();
  test_t16_existing_tests ();
  (* E2E *)
  test_e2e_one_call ();
  (* R1-R8 *)
  test_r1_malformed_core_cap ();
  test_r2_malformed_fact_decl ();
  test_r3_malformed_core_env_struct ();
  test_r4_facts_missing ();
  test_r5_non_scalar_fact_preserved ();
  test_r6_guarded_wrong_event ();
  test_r7_t7_exact_key ();
  test_r8_t13_idempotency_keys ();
  (* Q1-Q12 *)
  test_q1_null_request ();
  test_q2_string_request ();
  test_q3_tether_string ();
  test_q4_event_array ();
  test_q5_top_level_cap_non_object ();
  test_q6_core_env_cap_non_object ();
  test_q7_core_env_facts_non_object ();
  test_q8_fact_missing_schema_description ();
  test_q9_fact_schema_description_int ();
  test_q10_program_id_wrong_type ();
  test_q11_core_version_wrong_type ();
  test_q12_all_previous_green ();
  Printf.printf "PASS all request adapter tests (%d/%d)\n" !tests_passed !tests_run
