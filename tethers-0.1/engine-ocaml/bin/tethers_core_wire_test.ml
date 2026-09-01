(* Tethers_core_wire_test -- tests for the Core wire adapter.

   T1: OCaml wire Matched
   T2: OCaml wire Not_matched
   T3: OCaml wire request error (missing core_environment) *)

(* ================================================================== *)
(*  Test helpers                                                       *)
(* ================================================================== *)

let assert_bool msg cond = if cond then () else failwith msg

let json_string json = Yojson.Safe.to_string json

let json_member name json =
  match json with
  | `Assoc fields -> List.assoc_opt name fields
  | _ -> None

let json_string_member name json =
  match json_member name json with
  | Some (`String s) -> s
  | _ -> failwith (name ^ " is not a string in " ^ json_string json)

(* ================================================================== *)
(*  Valid CORE-8B extended request fixture                             *)
(* ================================================================== *)

let valid_request_json =
  `Assoc
    [
      ("protocol_version", `String "0.1");
      ("language_version", `String "0.1");
      ("evaluation_id", `String "eval_wire_001");
      ( "tether",
        `Assoc
          [
            ("id", `String "core-rehearsal");
            ("version", `String "1");
            ( "source",
              `String
                "tether \"core rehearsal\"\n\nanchor\n    fixture.start\n\n\
                 when\n\ndo\n    notify\n        message: anchor.message\n" );
          ] );
      ( "event",
        `Assoc
          [
            ("id", `String "evt_wire_001");
            ("name", `String "fixture.start");
            ("data", `Assoc [ ("message", `String "Hello Core") ]);
          ] );
      ("facts", `Assoc []);
      ( "capabilities",
        `List
          [
            `Assoc
              [
                ("name", `String "fixture.ping");
                ("version", `String "1.0.0");
                ("inputs", `Assoc [ ("message", `String "string") ]);
                ("effects", `List [ `String "fixture.test" ]);
                ("reversibility", `String "compensatable");
              ];
          ] );
      ( "core_environment",
        `Assoc
          [
            ("program_id", `String "program.core9b");
            ("core_version", `String "1");
            ( "capabilities",
              `List
                [
                  `Assoc
                    [
                      ("source_name", `String "notify");
                      ("capability_id", `String "cap.semantic.notify");
                      ("contract_digest", `String "CORE-CONTRACT-9B");
                      ("runtime_name", `String "fixture.ping");
                    ];
                ] );
            ("input_facts", `List []);
          ] );
    ]

let wrong_event_request_json =
  `Assoc
    [
      ("protocol_version", `String "0.1");
      ("language_version", `String "0.1");
      ("evaluation_id", `String "eval_wire_002");
      ( "tether",
        `Assoc
          [
            ("id", `String "core-rehearsal");
            ("version", `String "1");
            ( "source",
              `String
                "tether \"core rehearsal\"\n\nanchor\n    fixture.start\n\n\
                 when\n\ndo\n    notify\n        message: anchor.message\n" );
          ] );
      ( "event",
        `Assoc
          [
            ("id", `String "evt_wire_002");
            ("name", `String "fixture.other");
            ("data", `Assoc [ ("message", `String "Hello Core") ]);
          ] );
      ("facts", `Assoc []);
      ( "capabilities",
        `List
          [
            `Assoc
              [
                ("name", `String "fixture.ping");
                ("version", `String "1.0.0");
                ("inputs", `Assoc [ ("message", `String "string") ]);
                ("effects", `List [ `String "fixture.test" ]);
                ("reversibility", `String "compensatable");
              ];
          ] );
      ( "core_environment",
        `Assoc
          [
            ("program_id", `String "program.core9b");
            ("core_version", `String "1");
            ( "capabilities",
              `List
                [
                  `Assoc
                    [
                      ("source_name", `String "notify");
                      ("capability_id", `String "cap.semantic.notify");
                      ("contract_digest", `String "CORE-CONTRACT-9B");
                      ("runtime_name", `String "fixture.ping");
                    ];
                ] );
            ("input_facts", `List []);
          ] );
    ]

(* ================================================================== *)
(*  T1: OCaml wire Matched                                             *)
(* ================================================================== *)

let test_t1_wire_matched () =
  let response = Tethers_core_wire.evaluate_request_json valid_request_json in
  let status = json_string_member "status" response in
  assert_bool "T1: status must be matched" (status = "matched");
  let pid = json_string_member "protocol_version" response in
  assert_bool "T1: protocol_version 0.1" (pid = "0.1");
  let eid = json_string_member "evaluation_id" response in
  assert_bool "T1: evaluation_id preserved" (eid = "eval_wire_001");
  let evt_id = json_string_member "event_id" response in
  assert_bool "T1: event_id preserved" (evt_id = "evt_wire_001");
  let tid = json_string_member "tether_id" response in
  assert_bool "T1: tether_id preserved" (tid = "core-rehearsal");
  let tv = json_string_member "tether_version" response in
  assert_bool "T1: tether_version preserved" (tv = "1");
  (* plan must exist *)
  (match json_member "plan" response with
   | Some (`Assoc _) -> ()
   | _ -> failwith "T1: plan must be an object");
  let plan = match json_member "plan" response with Some p -> p | _ -> assert false in
  (* plan.id = evaluation_id/plan *)
  let plan_id = json_string_member "id" plan in
  assert_bool "T1: plan.id correct" (plan_id = "eval_wire_001/plan");
  (* ProgramDigest V2 is top-level and versioned; bare V1 sha256: is legacy. *)
  let pd = json_string_member "program_digest" response in
  let prefix = "tethers:v2:sha256:" in
  assert_bool "T1: ProgramDigest uses V2 prefix"
    (String.length pd = String.length prefix + 64 &&
     String.sub pd 0 (String.length prefix) = prefix);
  assert_bool "T1: ProgramDigest V2 suffix is lowercase hex"
    (let hex = String.sub pd (String.length prefix) 64 in
     String.for_all
       (function '0' .. '9' | 'a' .. 'f' -> true | _ -> false)
       hex);
  (* program_digest must NOT be inside plan *)
  (match json_member "program_digest" plan with
   | Some _ -> failwith "T1: program_digest must NOT be inside plan"
   | None -> ());
  (* actions array has one entry *)
  (match json_member "actions" plan with
   | Some (`List [action]) ->
       let cap = json_string_member "capability" action in
       assert_bool "T1: action capability is fixture.ping" (cap = "fixture.ping");
       let idem = json_string_member "idempotency_key" action in
       assert_bool "T1: idempotency_key correct" (idem = "eval_wire_001/action_1")
   | _ -> failwith "T1: expected exactly one action");
  (* trail is empty array *)
  (match json_member "trail" response with
   | Some (`List []) -> ()
   | _ -> failwith "T1: trail must be empty array");
  Printf.printf "T1 PASS: wire matched\n"

(* ================================================================== *)
(*  T2: OCaml wire Not_matched                                         *)
(* ================================================================== *)

let test_t2_wire_not_matched () =
  let response =
    Tethers_core_wire.evaluate_request_json wrong_event_request_json
  in
  let status = json_string_member "status" response in
  assert_bool "T2: status must be not_matched" (status = "not_matched");
  let pid = json_string_member "protocol_version" response in
  assert_bool "T2: protocol_version 0.1" (pid = "0.1");
  let eid = json_string_member "evaluation_id" response in
  assert_bool "T2: evaluation_id preserved" (eid = "eval_wire_002");
  (* plan must be null *)
  (match json_member "plan" response with
   | Some `Null -> ()
   | _ -> failwith "T2: plan must be null for not_matched");
  Printf.printf "T2 PASS: wire not_matched\n"

(* ================================================================== *)
(*  T3: OCaml wire request error (missing core_environment)            *)
(* ================================================================== *)

let test_t3_wire_error () =
  let req_no_core =
    `Assoc
      [
        ("protocol_version", `String "0.1");
        ("language_version", `String "0.1");
        ("evaluation_id", `String "eval_wire_err");
        ( "tether",
          `Assoc
            [
              ("id", `String "test");
              ("version", `String "1");
              ("source", `String "tether \"t\"\n\nanchor\n    e\n\nwhen\n\ndo\n    c\n        k: \"v\"\n");
            ] );
        ( "event",
          `Assoc
            [
              ("id", `String "evt_err");
              ("name", `String "e");
            ] );
        ("facts", `Assoc []);
        ( "capabilities",
          `List
            [
              `Assoc
                [
                  ("name", `String "c");
                  ("version", `String "1.0.0");
                  ("inputs", `Assoc [ ("k", `String "string") ]);
                  ("effects", `List []);
                  ("reversibility", `String "compensatable");
                ];
            ] );
      ]
  in
  let response = Tethers_core_wire.evaluate_request_json req_no_core in
  let status = json_string_member "status" response in
  assert_bool "T3: status must be error" (status = "error");
  let pid = json_string_member "protocol_version" response in
  assert_bool "T3: protocol_version 0.1" (pid = "0.1");
  let code = json_string_member "code" (match json_member "error" response with Some e -> e | _ -> assert false) in
  assert_bool "T3: error code is missing_core_environment"
    (code = "missing_core_environment");
  Printf.printf "T3 PASS: wire error\n"

(* ================================================================== *)
(*  Run all tests                                                      *)
(* ================================================================== *)

let () =
  test_t1_wire_matched ();
  test_t2_wire_not_matched ();
  test_t3_wire_error ();
  Printf.printf "All core_wire tests passed.\n"
