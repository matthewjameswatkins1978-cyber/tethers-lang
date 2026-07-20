open Tether_parser

type capability = {
  name : string;
  version : string;
  inputs : (string * string) list;
  effects : string list;
}

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