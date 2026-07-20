exception Tethers_error of string * string

type value =
  | String_value of string
  | Int_value of int
  | Bool_value of bool
  | Reference of string

type operator = Is | Contains | Greater_than | Greater_than_or_equal

type condition = {
  fact : string;
  operator : operator;
  expected : value;
  source : string;
}

type action = {
  capability : string;
  arguments : (string * value) list;
}

type tether = {
  title : string;
  anchor : string;
  conditions : condition list;
  actions : action list;
}

type capability = {
  name : string;
  version : string;
  inputs : (string * string) list;
  effects : string list;
}

let fail code message = raise (Tethers_error (code, message))

let trim = String.trim

let starts_with prefix value =
  let plen = String.length prefix in
  String.length value >= plen && String.sub value 0 plen = prefix

let drop_prefix prefix value =
  String.sub value (String.length prefix) (String.length value - String.length prefix)

let indentation line =
  let rec count i =
    if i < String.length line && line.[i] = ' ' then count (i + 1) else i
  in
  count 0

let non_blank_lines source =
  source
  |> String.split_on_char '\n'
  |> List.filter (fun line -> trim line <> "")

let unquote value =
  let value = trim value in
  let len = String.length value in
  if len >= 2 && value.[0] = '"' && value.[len - 1] = '"' then
    String.sub value 1 (len - 2)
  else fail "parse_error" ("Expected quoted string, got: " ^ value)

let parse_value raw =
  let value = trim raw in
  let len = String.length value in
  if len >= 2 && value.[0] = '"' && value.[len - 1] = '"' then
    String_value (String.sub value 1 (len - 2))
  else if value = "true" then Bool_value true
  else if value = "false" then Bool_value false
  else
    match int_of_string_opt value with
    | Some number -> Int_value number
    | None ->
        if starts_with "anchor." value then Reference value
        else fail "parse_error" ("Unsupported value: " ^ value)

let take_word value =
  let value = trim value in
  match String.index_opt value ' ' with
  | None -> (value, "")
  | Some index ->
      (String.sub value 0 index,
       String.sub value (index + 1) (String.length value - index - 1) |> trim)

let operator_of_string = function
  | "is" -> Is
  | "contains" -> Contains
  | "greater_than" -> Greater_than
  | "greater_than_or_equal" -> Greater_than_or_equal
  | value -> fail "parse_error" ("Unknown condition operator: " ^ value)

let parse_condition line =
  let source = trim line in
  let body = if starts_with "and " source then drop_prefix "and " source else source in
  let fact, rest = take_word body in
  let operator_text, expected = take_word rest in
  if fact = "" || operator_text = "" || expected = "" then
    fail "parse_error" ("Malformed condition: " ^ source);
  { fact; operator = operator_of_string operator_text; expected = parse_value expected; source = body }

let parse_argument line =
  let body = trim line in
  match String.index_opt body ':' with
  | None -> fail "parse_error" ("Malformed action argument: " ^ body)
  | Some index ->
      let name = String.sub body 0 index |> trim in
      let raw = String.sub body (index + 1) (String.length body - index - 1) |> trim in
      if name = "" || raw = "" then fail "parse_error" ("Malformed action argument: " ^ body);
      (name, parse_value raw)

let parse_actions lines =
  let finish current result =
    match current with
    | None -> result
    | Some (name, arguments) -> { capability = name; arguments = List.rev arguments } :: result
  in
  let rec loop remaining current result =
    match remaining with
    | [] -> List.rev (finish current result)
    | line :: rest ->
        let indent = indentation line in
        if indent >= 8 then
          (match current with
           | None -> fail "parse_error" "Action argument appeared before an Action"
           | Some (name, arguments) ->
               loop rest (Some (name, parse_argument line :: arguments)) result)
        else
          let result = finish current result in
          loop rest (Some (trim line, [])) result
  in
  loop lines None []

let parse_tether source =
  match non_blank_lines source with
  | first :: "anchor" :: anchor :: "when" :: rest ->
      if not (starts_with "tether " first) then fail "parse_error" "Tether must begin with tether \"name\"";
      let title = drop_prefix "tether " first |> unquote in
      let rec split_conditions acc = function
        | [] -> fail "parse_error" "Missing do section"
        | "do" :: action_lines -> (List.rev acc, action_lines)
        | line :: more -> split_conditions (parse_condition line :: acc) more
      in
      let conditions, action_lines = split_conditions [] rest in
      let actions = parse_actions action_lines in
      if actions = [] then fail "parse_error" "A Tether must contain at least one Action";
      { title; anchor = trim anchor; conditions; actions }
  | _ -> fail "parse_error" "Expected tether, anchor, when, and do sections"

let json_assoc name json =
  match Yojson.Safe.Util.member name json with
  | `Assoc fields -> fields
  | _ -> fail "invalid_request" ("Expected object field: " ^ name)

let json_string name json =
  match Yojson.Safe.Util.member name json with
  | `String value -> value
  | _ -> fail "invalid_request" ("Expected string field: " ^ name)

let json_list name json =
  match Yojson.Safe.Util.member name json with
  | `List values -> values
  | _ -> fail "invalid_request" ("Expected array field: " ^ name)

let value_of_json = function
  | `String value -> String_value value
  | `Int value -> Int_value value
  | `Bool value -> Bool_value value
  | _ -> fail "type_error" "Version 0.1 supports string, integer, and boolean values"

let json_of_value = function
  | String_value value -> `String value
  | Int_value value -> `Int value
  | Bool_value value -> `Bool value
  | Reference value -> fail "internal_error" ("Unresolved reference: " ^ value)

let parse_capability json =
  let inputs =
    json_assoc "inputs" json
    |> List.map (fun (name, kind) ->
           match kind with
           | `String value -> (name, value)
           | _ -> fail "invalid_capability" ("Input type for " ^ name ^ " must be a string"))
  in
  let effects =
    json_list "effects" json
    |> List.map (function
         | `String value -> value
         | _ -> fail "invalid_capability" "Effects must be strings")
  in
  { name = json_string "name" json; version = json_string "version" json; inputs; effects }

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
  if parsed.anchor <> event_name then response "not_matched" `Null base
  else
    let rec check_conditions sequence trail = function
      | [] -> (true, sequence, trail)
      | condition :: rest ->
          let matched = evaluate_condition facts condition in
          let entry = trail_entry sequence "evaluation" "condition_checked"
              (if matched then "matched" else "not_matched") condition.source in
          let trail = trail @ [entry] in
          if matched then check_conditions (sequence + 1) trail rest
          else (false, sequence + 1, trail)
    in
    let all_matched, next_sequence, condition_trail = check_conditions 3 base parsed.conditions in
    if not all_matched then response "not_matched" `Null condition_trail
    else
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
      let plan = `Assoc [
        ("id", `String (evaluation_id ^ "/plan"));
        ("required_effects", `List (List.map (fun item -> `String item) required_effects));
        ("actions", `List actions)
      ] in
      response "matched" plan trail

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
