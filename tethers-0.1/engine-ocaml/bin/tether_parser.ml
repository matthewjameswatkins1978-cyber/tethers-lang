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
