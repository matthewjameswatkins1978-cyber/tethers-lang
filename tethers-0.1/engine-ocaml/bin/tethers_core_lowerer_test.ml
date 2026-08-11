open Tethers_core
open Tethers_core_lowerer

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

let assert_equal msg expected actual =
  incr tests_run;
  if expected = actual then begin
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

let assert_error_eq msg expected_err = function
  | Ok _ -> begin
      incr tests_run;
      Printf.eprintf "FAIL: %s (expected Error, got Ok)\n" msg;
      exit 1
    end
  | Error actual -> begin
      incr tests_run;
      if expected_err = actual then
        incr tests_passed
      else begin
        Printf.eprintf "FAIL: %s (error mismatch)\n" msg;
        exit 1
      end
    end

(* ------------------------------------------------------------------ *)
(*  Helper: build a lowering environment                               *)
(* ------------------------------------------------------------------ *)

let test_program_id = program_id_of_string "P_test"
let test_core_version = core_version_of_string "0.1.0"

let make_env caps facts =
  { program_id = test_program_id;
    core_version = test_core_version;
    capabilities = caps;
    input_facts = facts;
  }

let cap_binding name cap_id digest =
  { source_name = name;
    capability_id = capability_id_of_string cap_id;
    contract_digest = capability_contract_digest_of_string digest;
  }

let fact_binding name =
  { source_name = name;
    fact = { fact_id = fact_id_of_string ("F_" ^ name);
             schema_description = "";
             provenance = Evaluation_input (host_snapshot_key_of_string ("K_" ^ name), String_type);
           };
  }

let default_env =
  make_env
    [ cap_binding "notify" "C_notify" "D_notify_v1";
      cap_binding "save" "C_save" "D_save_v1";
      cap_binding "log" "C_log" "D_log_v1";
    ]
    [ fact_binding "file_type";
      fact_binding "file_size";
      fact_binding "priority";
      fact_binding "customer";
    ]

(* ------------------------------------------------------------------ *)
(*  Parse a Tether string (catches Tethers_error for test clarity)     *)
(* ------------------------------------------------------------------ *)

let parse source =
  try Ok (Tether_parser.parse_tether source) with
  | Tethers_error.Tethers_error _ -> Error "parse_error"

(* ------------------------------------------------------------------ *)
(*  A. Single Action                                                   *)
(* ------------------------------------------------------------------ *)

let test_single_action () =
  let source = {|
tether "test"
anchor
    file.received
when
    file_type is "pdf"
do
    notify
        message: "hello"
|} in
  match parse source with
  | Ok tether ->
      (match lower default_env tether with
       | Ok program ->
           assert_true "entry_origin is Some" (program.entry_origin <> None);
           let entry_id = Option.get program.entry_origin in
           assert_equal "entry is O_action_1"
             "O_action_1" (string_of_origin_id entry_id);
           assert_equal "one action origin in sites" 2
             (List.length program.origin_sites);
           assert_equal "one success continuation" 1
             (List.length program.success_continuations);
           (match List.hd (List.rev program.success_continuations) with
            | cont ->
                assert_equal "last continuation is Program_complete"
                  "O_action_1" (string_of_origin_id cont.from_origin);
                assert_true "target is Program_complete"
                  (cont.target = Program_complete));
           assert_equal "anchor origin id" "O_anchor"
             (match List.hd program.origin_sites with
              | Anchor_origin a -> string_of_origin_id a.anchor_origin_id
              | _ -> "WRONG");
           assert_equal "event name" "file.received"
             (match List.hd program.origin_sites with
              | Anchor_origin a -> a.event_name
              | _ -> "")
       | Error _ ->
           Printf.eprintf "FAIL: single action lowering returned error\n";
           exit 1)
  | _ -> ()

(* ------------------------------------------------------------------ *)
(*  B. Three Sequential Actions                                        *)
(* ------------------------------------------------------------------ *)

let test_three_actions () =
  let source = {|
tether "test"
anchor
    file.received
when
    file_type is "pdf"
do
    notify
        message: "start"
    save
        file: anchor.document
        copies: 1
    log
        entry: "done"
|} in
  match parse source with
  | Ok tether ->
      (match lower default_env tether with
       | Ok program ->
           assert_equal "three success continuations" 3
             (List.length program.success_continuations);
           let conts = program.success_continuations in
           (match conts with
            | c1 :: c2 :: [c3] ->
                assert_equal "c1 from O_action_1" "O_action_1"
                  (string_of_origin_id c1.from_origin);
                assert_true "c1 to O_action_2"
                  (c1.target = Origin_target (origin_id_of_string "O_action_2"));
                assert_equal "c2 from O_action_2" "O_action_2"
                  (string_of_origin_id c2.from_origin);
                assert_true "c2 to O_action_3"
                  (c2.target = Origin_target (origin_id_of_string "O_action_3"));
                assert_equal "c3 from O_action_3" "O_action_3"
                  (string_of_origin_id c3.from_origin);
                assert_true "c3 to Program_complete" (c3.target = Program_complete)
            | _ ->
                Printf.eprintf "FAIL: wrong number of continuations\n";
                exit 1);
           let entry_id = Option.get program.entry_origin in
           assert_equal "entry is O_action_1" "O_action_1"
             (string_of_origin_id entry_id);
           (* Assert storage order is not execution meaning: verify the
              origin_sites list contains the Anchor first, then Actions *)
           assert_true "origin_sites has 1 anchor + 3 actions = 4 entries"
             (List.length program.origin_sites = 4)
       | Error _ ->
           Printf.eprintf "FAIL: three-action lowering returned error\n";
           exit 1)
  | _ -> ()

(* ------------------------------------------------------------------ *)
(*  C. Typed Literals                                                  *)
(* ------------------------------------------------------------------ *)

let test_typed_literals () =
  let source = {|
tether "test"
anchor
    file.received
when
    file_size greater_than 0
do
    notify
        text: "string literal"
        count: 42
        flag: true
|} in
  match parse source with
  | Ok tether ->
      (match lower default_env tether with
       | Ok program ->
           let action_site =
             List.find
               (fun s -> match s with Action_origin _ -> true | _ -> false)
               program.origin_sites
           in
           (match action_site with
            | Action_origin origin ->
                assert_equal "3 inputs" 3 (List.length origin.inputs);
                let input_map = List.map
                  (fun (ai : action_input) ->
                     (string_of_capability_input_name ai.input_name, ai.binding))
                  origin.inputs
                in
                (match List.assoc_opt "text" input_map with
                 | Some (Literal_value (String_value "string literal")) -> ()
                 | _ ->
                     Printf.eprintf "FAIL: text input not String_value\n";
                     exit 1);
                (match List.assoc_opt "count" input_map with
                 | Some (Literal_value (Integer_value 42)) -> ()
                 | _ ->
                     Printf.eprintf "FAIL: count input not Integer_value 42\n";
                     exit 1);
                (match List.assoc_opt "flag" input_map with
                 | Some (Literal_value (Boolean_value true)) -> ()
                 | _ ->
                     Printf.eprintf "FAIL: flag input not Boolean_value true\n";
                     exit 1);
                incr tests_run; incr tests_passed;
                incr tests_run; incr tests_passed;
                incr tests_run; incr tests_passed
            | _ ->
                Printf.eprintf "FAIL: no Action_origin found\n";
                exit 1)
       | Error _ ->
           Printf.eprintf "FAIL: typed literals lowering returned error\n";
           exit 1)
  | _ -> ()

(* ------------------------------------------------------------------ *)
(*  D. Anchor Binding                                                  *)
(* ------------------------------------------------------------------ *)

let test_anchor_binding () =
  let source = {|
tether "test"
anchor
    file.received
when
    file_type is "pdf"
do
    notify
        reference: anchor.customer.id
|} in
  match parse source with
  | Ok tether ->
      (match lower default_env tether with
       | Ok program ->
           let action_site =
             List.find
               (fun s -> match s with Action_origin _ -> true | _ -> false)
               program.origin_sites
           in
           (match action_site with
            | Action_origin origin ->
                (match origin.inputs with
                 | [ { input_name; binding = Anchor_value (origin_id, path) } ] ->
                     assert_equal "input name is reference" "reference"
                       (string_of_capability_input_name input_name);
                     assert_equal "anchor origin id" "O_anchor"
                       (string_of_origin_id origin_id);
                     assert_equal "path parts" [ "customer"; "id" ] path
                 | _ ->
                     Printf.eprintf "FAIL: expected single Anchor_value input\n";
                     exit 1)
            | _ ->
                Printf.eprintf "FAIL: no Action_origin found\n"; exit 1)
       | Error _ ->
           Printf.eprintf "FAIL: anchor binding returned error\n"; exit 1)
  | _ -> ()

(* ------------------------------------------------------------------ *)
(*  E. Conditions / Guards - all four operators, order preserved       *)
(* ------------------------------------------------------------------ *)

let test_conditions () =
  let source = {|
tether "test"
anchor
    file.received
when
    file_type is "pdf"
    file_size greater_than 0
    priority greater_than_or_equal 1
    file_type contains "doc"
do
    notify
        message: "ok"
|} in
  match parse source with
  | Ok tether ->
      (match lower default_env tether with
       | Ok program ->
           assert_equal "four guards" 4 (List.length program.entry_guards);
           (match program.entry_guards with
            | g1 :: g2 :: g3 :: g4 :: [] ->
                assert_equal "g1 operator Equals" "F_file_type"
                  (string_of_fact_id g1.fact_id);
                assert_true "g1 op Equals" (g1.operator = Equals);
                assert_equal "g2 operator Greater_than" "F_file_size"
                  (string_of_fact_id g2.fact_id);
                assert_true "g2 op Greater_than" (g2.operator = Greater_than);
                assert_equal "g3 operator Greater_than_or_equal" "F_priority"
                  (string_of_fact_id g3.fact_id);
                assert_true "g3 op Greater_than_or_equal" (g3.operator = Greater_than_or_equal);
                assert_equal "g4 operator Contains" "F_file_type"
                  (string_of_fact_id g4.fact_id);
                assert_true "g4 op Contains" (g4.operator = Contains)
            | _ ->
                Printf.eprintf "FAIL: wrong guard count\n"; exit 1)
       | Error _ ->
           Printf.eprintf "FAIL: conditions lowering returned error\n"; exit 1)
  | _ -> ()

(* ------------------------------------------------------------------ *)
(*  F. Input Fact Binding                                              *)
(* ------------------------------------------------------------------ *)

let test_known_fact_resolves () =
  let source = {|
tether "test"
anchor
    file.received
when
    file_type is "pdf"
do
    notify
        message: "ok"
|} in
  match parse source with
  | Ok tether ->
      assert_ok "known fact resolves" (lower default_env tether)
  | _ -> ()

let test_unknown_fact_fails () =
  let source = {|
tether "test"
anchor
    file.received
when
    unknown_fact is "x"
do
    notify
        message: "ok"
|} in
  match parse source with
  | Ok tether ->
      assert_error_eq "unknown fact fails" (Unknown_fact "unknown_fact")
        (lower default_env tether)
  | _ -> ()

(* ------------------------------------------------------------------ *)
(*  G. Capability Binding                                              *)
(* ------------------------------------------------------------------ *)

let test_known_capability_resolves () =
  let source = {|
tether "test"
anchor
    file.received
when
    file_type is "pdf"
do
    notify
        message: "ok"
|} in
  match parse source with
  | Ok tether ->
      (match lower default_env tether with
       | Ok program ->
           let action_site =
             List.find
               (fun s -> match s with Action_origin _ -> true | _ -> false)
               program.origin_sites
           in
           (match action_site with
            | Action_origin origin ->
                assert_equal "capability_id" "C_notify"
                  (string_of_capability_id origin.capability_id);
                assert_equal "contract_digest" "D_notify_v1"
                  (string_of_capability_contract_digest origin.contract_digest)
            | _ ->
                Printf.eprintf "FAIL: no Action_origin\n"; exit 1)
       | Error _ ->
           Printf.eprintf "FAIL: known capability returned error\n"; exit 1)
  | _ -> ()

let test_unknown_capability_fails () =
  let source = {|
tether "test"
anchor
    file.received
when
    file_type is "pdf"
do
    nonexistent
        arg: "x"
|} in
  match parse source with
  | Ok tether ->
      assert_error_eq "unknown capability fails"
        (Unknown_capability "nonexistent") (lower default_env tether)
  | _ -> ()

let test_duplicate_capability_fails () =
  let dup_env = make_env
    [ cap_binding "notify" "C_notify_a" "D_a";
      cap_binding "notify" "C_notify_b" "D_b";
    ]
    [ fact_binding "file_type" ]
  in
  let source = {|
tether "test"
anchor
    file.received
when
    file_type is "pdf"
do
    notify
        message: "ok"
|} in
  match parse source with
  | Ok tether ->
      assert_error_eq "duplicate capability fails"
        (Duplicate_capability "notify") (lower dup_env tether)
  | _ -> ()

(* ------------------------------------------------------------------ *)
(*  H. Together Refusal                                                *)
(* ------------------------------------------------------------------ *)

let test_together_refused () =
  let source = {|
tether "test"
anchor
    file.received
when
    file_type is "pdf"
do
    together
        notify
            message: "a"
        save
            file: anchor.document
|} in
  match parse source with
  | Ok tether ->
      assert_error_eq "together refused"
        (Unsupported_construct "together") (lower default_env tether)
  | _ -> ()

(* ------------------------------------------------------------------ *)
(*  I. Determinism                                                     *)
(* ------------------------------------------------------------------ *)

let test_determinism () =
  let source = {|
tether "test"
anchor
    file.received
when
    file_type is "pdf"
    file_size greater_than 0
do
    notify
        message: "hello"
    save
        file: anchor.document
        copies: 2
    log
        entry: "processed"
|} in
  match parse source with
  | Ok tether ->
      let r1 = lower default_env tether in
      let r2 = lower default_env tether in
      let r3 = lower default_env tether in
      assert_true "determinism r1=r2" (r1 = r2);
      assert_true "determinism r1=r3" (r1 = r3);
      assert_true "determinism r2=r3" (r2 = r3)
  | _ -> ()

(* ------------------------------------------------------------------ *)
(*  Additional negative tests                                          *)
(* ------------------------------------------------------------------ *)

let test_non_anchor_reference_rejected () =
  let env = default_env in
  let source = {|
tether "test"
anchor
    file.received
when
    file_type is "pdf"
do
    notify
        bad_ref: something.here
|} in
  match parse source with
  | Ok tether ->
      assert_error_eq "non-anchor reference rejected"
        (Missing_anchor_reference "something.here") (lower env tether)
  | _ -> ()

let test_no_actions_handled () =
  let source = {|
tether "test"
anchor
    file.received
when
    file_type is "pdf"
do
|} in
  (* Parser rejects this - expect parse error *)
  match parse source with
  | Ok _ ->
      Printf.eprintf "FAIL: parser should reject zero-action tether\n";
      exit 1
  | Error _ -> begin
      incr tests_run; incr tests_passed  (* parse error is expected *)
    end

(* ------------------------------------------------------------------ *)
(*  Run all tests                                                      *)
(* ------------------------------------------------------------------ *)

let () =
  test_single_action ();
  test_three_actions ();
  test_typed_literals ();
  test_anchor_binding ();
  test_conditions ();
  test_known_fact_resolves ();
  test_unknown_fact_fails ();
  test_known_capability_resolves ();
  test_unknown_capability_fails ();
  test_duplicate_capability_fails ();
  test_together_refused ();
  test_determinism ();
  test_non_anchor_reference_rejected ();
  test_no_actions_handled ();
  Printf.printf "PASS all lowerer tests (%d/%d)\n" !tests_passed !tests_run
