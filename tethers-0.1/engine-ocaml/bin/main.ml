open Tethers_protocol
open Tether_parser

let rec nested_member path json =
  match path with
  | [] -> json
  | part :: rest ->
      (match json with
       | `Assoc fields ->
           (match List.assoc_opt part fields with
            | Some value -> nested_member rest value
            | None -> fail "missing_reference" ("Missing anchor value: " ^ String.concat "." path))
       | _ -> fail "missing_reference" ("Cannot traverse anchor reference: " ^ String.concat "." path))

let resolve_value event_data = function
  | Reference path ->
      let relative = drop_prefix "anchor." path in
      nested_member (String.split_on_char '.' relative) event_data |> value_of_json
  | literal -> literal

let matches_type expected value =
  match (expected, value) with
  | "string", String_value _ -> true
  | "integer", Int_value _ -> true
  | "boolean", Bool_value _ -> true
  | _ -> false

let evaluate_condition facts condition =
  let actual =
    match List.assoc_opt condition.fact facts with
    | Some json -> value_of_json json
    | None -> fail "missing_fact" ("Missing Fact: " ^ condition.fact)
  in
  match (condition.operator, actual, condition.expected) with
  | Is, String_value left, String_value right -> left = right
  | Is, Int_value left, Int_value right -> left = right
  | Is, Bool_value left, Bool_value right -> left = right
  | Contains, String_value left, String_value right ->
      let left_len = String.length left and right_len = String.length right in
      let rec search index =
        if right_len = 0 then true
        else if index + right_len > left_len then false
        else if String.sub left index right_len = right then true
        else search (index + 1)
      in
      search 0
  | Greater_than, Int_value left, Int_value right -> left > right
  | Greater_than_or_equal, Int_value left, Int_value right -> left >= right
  | _ -> fail "type_error" ("Invalid operands in condition: " ^ condition.source)

let trail_entry sequence phase kind outcome message =
  `Assoc [
    ("sequence", `Int sequence);
    ("phase", `String phase);
    ("kind", `String kind);
    ("outcome", `String outcome);
    ("message", `String message)
  ]

type condition_result =
  | Conditions_matched of int * Yojson.Safe.t list
  | Condition_not_matched of int * Yojson.Safe.t list
  | Condition_error of string * string * Yojson.Safe.t list

type action_planning_result =
  | Actions_planned of Yojson.Safe.t list * string list * Yojson.Safe.t list
  | Action_planning_error of string * string * Yojson.Safe.t list

let unique values =
  List.fold_left (fun acc value -> if List.mem value acc then acc else acc @ [value]) [] values

let evaluate request =
  let protocol_version = json_string "protocol_version" request in
  let language_version = json_string "language_version" request in
  if protocol_version <> "0.1" then fail "incompatible_protocol" ("Unsupported protocol: " ^ protocol_version);
  if language_version <> "0.1" then fail "incompatible_language" ("Unsupported language: " ^ language_version);
  let evaluation_id = json_string "evaluation_id" request in
  let tether_json = Yojson.Safe.Util.member "tether" request in
  let tether_id = json_string "id" tether_json in
  let tether_version = json_string "version" tether_json in
  let parsed = parse_tether (json_string "source" tether_json) in
  let _title = parsed.title in
  let event = Yojson.Safe.Util.member "event" request in
  let event_id = json_string "id" event in
  let event_name = json_string "name" event in
  let event_data = Yojson.Safe.Util.member "data" event in
  let facts = json_assoc "facts" request in
  let capabilities = json_list "capabilities" request |> List.map parse_capability in
  let () = check_unique_capabilities capabilities in
  let base = [
    trail_entry 1 "reception" "event_received" "accepted" ("Received " ^ event_name);
    trail_entry 2 "evaluation" "anchor_checked"
      (if parsed.anchor = event_name then "matched" else "not_matched")
      ("Anchor " ^ parsed.anchor ^ (if parsed.anchor = event_name then " matched" else " did not match"))
  ] in
  let response status plan trail =
    `Assoc [
      ("protocol_version", `String "0.1");
      ("evaluation_id", `String evaluation_id);
      ("event_id", `String event_id);
      ("tether_id", `String tether_id);
      ("tether_version", `String tether_version);
      ("status", `String status);
      ("plan", plan);
      ("trail", `List trail)
    ]
  in
  let contextual_error_response code message trail =
    `Assoc [
      ("protocol_version", `String "0.1");
      ("evaluation_id", `String evaluation_id);
      ("event_id", `String event_id);
      ("tether_id", `String tether_id);
      ("tether_version", `String tether_version);
      ("status", `String "error");
      ("plan", `Null);
      ("error", `Assoc [("code", `String code); ("message", `String message)]);
      ("trail", `List trail)
    ]
  in
  if parsed.anchor <> event_name then response "not_matched" `Null base
  else
    let rec check_conditions sequence trail = function
      | [] -> Conditions_matched (sequence, trail)
      | condition :: rest ->
          (try
             let matched = evaluate_condition facts condition in
             let entry = trail_entry sequence "evaluation" "condition_checked"
                 (if matched then "matched" else "not_matched") condition.source in
             let trail = trail @ [entry] in
             if matched then check_conditions (sequence + 1) trail rest
             else Condition_not_matched (sequence + 1, trail)
           with
           | Tethers_error (("missing_fact" | "type_error") as code, message) ->
               let entry = trail_entry sequence "evaluation" "condition_failed" "error" message in
               Condition_error (code, message, trail @ [entry])
          )
    in
    match check_conditions 3 base parsed.conditions with
    | Condition_error (code, message, trail) -> contextual_error_response code message trail
    | Condition_not_matched (_, trail) -> response "not_matched" `Null trail
    | Conditions_matched (next_sequence, condition_trail) ->
        let plan_result =
          try
            let rec plan_actions index sequence trail planned all_effects = function
              | [] -> (List.rev planned, unique all_effects, trail)
              | action :: rest ->
                  let schema =
                    match List.find_opt (fun item -> item.name = action.capability) capabilities with
                    | Some value -> value
                    | None -> fail "unknown_capability" ("Unknown Capability: " ^ action.capability)
                  in
                  List.iter (fun (input_name, input_type) ->
                    match List.assoc_opt input_name action.arguments with
                    | None -> fail "missing_argument" ("Missing argument " ^ input_name ^ " for " ^ action.capability)
                    | Some raw ->
                        let resolved = resolve_value event_data raw in
                        if not (matches_type input_type resolved) then
                          fail "type_error" ("Argument " ^ input_name ^ " for " ^ action.capability ^ " must be " ^ input_type)
                  ) schema.inputs;
                  List.iter (fun (argument_name, _) ->
                    if not (List.mem_assoc argument_name schema.inputs) then
                      fail "unknown_argument" ("Unknown argument " ^ argument_name ^ " for " ^ action.capability)
                  ) action.arguments;
                  let arguments =
                    action.arguments
                    |> List.map (fun (name, raw) -> (name, resolve_value event_data raw |> json_of_value))
                  in
                  let action_id = "action_" ^ string_of_int index in
                  let planned_action = `Assoc [
                    ("action_id", `String action_id);
                    ("idempotency_key", `String (evaluation_id ^ "/" ^ action_id));
                    ("capability", `String schema.name);
                    ("capability_version", `String schema.version);
                    ("arguments", `Assoc arguments);
                    ("effects", `List (List.map (fun item -> `String item) schema.effects))
                  ] in
                  let entry = trail_entry sequence "evaluation" "action_planned" "accepted"
                      ("Planned " ^ schema.name) in
                  plan_actions (index + 1) (sequence + 1) (trail @ [entry])
                    (planned_action :: planned) (all_effects @ schema.effects) rest
            in
            let actions, required_effects, trail =
              plan_actions 1 next_sequence condition_trail [] [] parsed.actions
            in
            Actions_planned (actions, required_effects, trail)
          with
          | Tethers_error (("unknown_capability" | "missing_argument" | "unknown_argument" | "type_error" | "missing_reference") as code, message) ->
              let entry = trail_entry next_sequence "evaluation" "action_planning_failed" "error" message in
              Action_planning_error (code, message, condition_trail @ [entry])
        in
        (match plan_result with
         | Action_planning_error (code, message, trail) -> contextual_error_response code message trail
         | Actions_planned (actions, required_effects, trail) ->
             let plan = `Assoc [
               ("id", `String (evaluation_id ^ "/plan"));
               ("required_effects", `List (List.map (fun item -> `String item) required_effects));
               ("actions", `List actions)
             ] in
             response "matched" plan trail
        )

let error_response code message =
  `Assoc [
    ("protocol_version", `String "0.1");
    ("status", `String "error");
    ("error", `Assoc [("code", `String code); ("message", `String message)])
  ]

let process_line line =
  try Yojson.Safe.from_string line |> evaluate
  with
  | Tethers_error (code, message) -> error_response code message
  | Yojson.Json_error message -> error_response "invalid_json" message
  | exn -> error_response "internal_error" (Printexc.to_string exn)

let () =
  try
    while true do
      let line = input_line stdin in
      process_line line |> Yojson.Safe.to_string |> print_endline;
      flush stdout
    done
  with End_of_file -> ()
