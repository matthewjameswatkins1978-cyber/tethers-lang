open Tethers_core
open Tethers_core_evaluation_adapter

(* ================================================================== *)
(*  Types                                                              *)
(* ================================================================== *)

type request_context = {
  protocol_version : string;
  language_version : string;
  evaluation_id : string;
  event_id : string;
  tether_id : string;
  tether_version : string;
}

type evaluated_request = {
  context : request_context;
  evaluation : Tethers_core_plan.canonical_evaluation;
}

type request_error =
  | Invalid_request of string * string
  | Missing_core_environment
  | Invalid_core_environment of string
  | Missing_runtime_capability_binding of string
  | Ambiguous_runtime_capability_binding of string
  | Invalid_scalar_type of string
  | Adapter_error of adapter_error

type parsed_request = {
  protocol_version : string;
  language_version : string;
  evaluation_id : string;
  event_id : string;
  tether_id : string;
  tether_version : string;
  source : string;
  event_name : string;
  event_data : Yojson.Safe.t;
  facts_json : Yojson.Safe.t;
  top_level_caps : Tethers_protocol.capability list;
}

(* ================================================================== *)
(*  JSON helpers                                                       *)
(* ================================================================== *)

let ( >>= ) = Result.bind

let json_string field json =
  match Yojson.Safe.Util.member field json with
  | `String s -> Ok s
  | _ -> Error (Invalid_request (field, "expected string"))

let json_list field json =
  match Yojson.Safe.Util.member field json with
  | `List l -> Ok l
  | _ -> Error (Invalid_request (field, "expected array"))

let json_member field json = Yojson.Safe.Util.member field json

(* String extraction for core_environment fields.  Returns
   Invalid_core_environment on wrong type instead of raising. *)
let core_env_string field json =
  match Yojson.Safe.Util.member field json with
  | `String s -> Ok s
  | _ -> Error (Invalid_core_environment
                  (field ^ ": expected string"))

(* ================================================================== *)
(*  Scalar type mapping                                                *)
(* ================================================================== *)

let parse_scalar_type s =
  match s with
  | "string" -> Ok String_type
  | "integer" -> Ok Integer_type
  | "boolean" -> Ok Boolean_type
  | _ -> Error (Invalid_scalar_type s)

(* ================================================================== *)
(*  Core capability binding resolution                                 *)
(* ================================================================== *)

(* Resolve a core_environment capability binding's runtime_name against
   the already-parsed top-level runtime capabilities.
   All field extraction uses Result; no exceptions escape. *)
let resolve_one_capability top_level_caps binding =
  core_env_string "source_name" binding >>= fun source_name ->
  core_env_string "capability_id" binding >>= fun capability_id_str ->
  core_env_string "contract_digest" binding >>= fun contract_digest_str ->
  core_env_string "runtime_name" binding >>= fun runtime_name ->
  let matches =
    List.filter (fun c -> c.Tethers_protocol.name = runtime_name) top_level_caps
  in
  match matches with
  | [] -> Error (Missing_runtime_capability_binding runtime_name)
  | [ runtime ] ->
      Ok { source_name;
           capability_id = capability_id_of_string capability_id_str;
           contract_digest =
             capability_contract_digest_of_string contract_digest_str;
           runtime }
  | _ -> Error (Ambiguous_runtime_capability_binding runtime_name)

let resolve_capabilities top_level_caps binding_jsons =
  let rec resolve acc = function
    | [] -> Ok (List.rev acc)
    | b :: rest ->
        (match resolve_one_capability top_level_caps b with
         | Ok cap -> resolve (cap :: acc) rest
         | Error e -> Error e)
  in
  resolve [] binding_jsons

(* ================================================================== *)
(*  Core input fact declaration                                        *)
(* ================================================================== *)

(* Parse one input_fact declaration from core_environment.
   All field extraction uses Result; no exceptions escape. *)
let parse_one_fact fact_json =
  core_env_string "source_name" fact_json >>= fun source_name ->
  core_env_string "fact_id" fact_json >>= fun fact_id_str ->
  core_env_string "host_snapshot_key" fact_json >>= fun host_snapshot_key_str ->
  core_env_string "scalar_type" fact_json >>= fun scalar_type_str ->
  let schema_description = match Yojson.Safe.Util.member "schema_description" fact_json with
    | `String s -> s | _ -> "" in
  parse_scalar_type scalar_type_str >>= fun stype ->
  Ok { source_name;
        fact = { fact_id = fact_id_of_string fact_id_str;
                 schema_description;
                 provenance =
                   Evaluation_input (host_snapshot_key_of_string host_snapshot_key_str,
                                     stype) } }

let parse_facts fact_jsons =
  let rec parse acc = function
    | [] -> Ok (List.rev acc)
    | f :: rest ->
        (match parse_one_fact f with
         | Ok binding -> parse (binding :: acc) rest
         | Error e -> Error e)
  in
  parse [] fact_jsons

(* ================================================================== *)
(*  Core environment parsing                                            *)
(* ================================================================== *)

let parse_core_env top_level_caps env_json =
  let open Yojson.Safe.Util in
  json_string "program_id" env_json >>= fun program_id_str ->
  json_string "core_version" env_json >>= fun core_version_str ->
  json_list "capabilities" env_json >>= fun cap_jsons ->
  resolve_capabilities top_level_caps cap_jsons >>= fun capabilities ->
  (match member "input_facts" env_json with
   | `List fact_jsons -> parse_facts fact_jsons
   | `Null -> Ok []
   | _ -> Error (Invalid_core_environment "input_facts: expected array or null"))
  >>= fun input_facts ->
  Ok { program_id = program_id_of_string program_id_str;
       core_version = core_version_of_string core_version_str;
       capabilities;
       input_facts }

(* ================================================================== *)
(*  Request parsing                                                    *)
(* ================================================================== *)

let parse_request request =
  json_string "protocol_version" request >>= fun protocol_version ->
  json_string "language_version" request >>= fun language_version ->
  if protocol_version <> "0.1" then
    Error (Invalid_request
             ("protocol_version", "unsupported: " ^ protocol_version))
  else if language_version <> "0.1" then
    Error (Invalid_request
             ("language_version", "unsupported: " ^ language_version))
  else
    json_string "evaluation_id" request >>= fun evaluation_id ->
    let tether_json = json_member "tether" request in
    json_string "id" tether_json >>= fun tether_id ->
    json_string "version" tether_json >>= fun tether_version ->
    json_string "source" tether_json >>= fun source ->
    let event_json = json_member "event" request in
    json_string "id" event_json >>= fun event_id ->
    json_string "name" event_json >>= fun event_name ->
    let event_data = json_member "data" event_json in
    (* Require facts to be an object.  Missing or non-object is invalid. *)
    (match Yojson.Safe.Util.member "facts" request with
     | `Assoc _ as obj -> Ok obj
     | `Null -> Error (Invalid_request ("facts", "required object"))
     | _ -> Error (Invalid_request ("facts", "expected object")))
    >>= fun facts_json ->
    json_list "capabilities" request >>= fun cap_jsons ->
    (try
       let top_level_caps =
         List.map Tethers_protocol.parse_capability cap_jsons
       in
       Tethers_protocol.check_unique_capabilities top_level_caps;
       Ok top_level_caps
     with
     | Tethers_error.Tethers_error (code, msg) ->
         Error (Invalid_request (code, msg)))
    >>= fun top_level_caps ->
    Ok { protocol_version; language_version;
         evaluation_id; event_id;
         tether_id; tether_version; source;
         event_name; event_data;
         facts_json; top_level_caps }

(* ================================================================== *)
(*  Main evaluate_request                                              *)
(* ================================================================== *)

let evaluate_request request =
  match parse_request request with
  | Error e -> Error e
  | Ok req ->
      let core_env_json = json_member "core_environment" request in
      (match core_env_json with
       | `Null -> Error Missing_core_environment
       | `Assoc _ ->
           (match parse_core_env req.top_level_caps core_env_json with
           | Error (Invalid_request (f, m)) ->
               Error (Invalid_request (f, m))
           | Error (Missing_runtime_capability_binding n) ->
               Error (Missing_runtime_capability_binding n)
           | Error (Ambiguous_runtime_capability_binding n) ->
               Error (Ambiguous_runtime_capability_binding n)
           | Error (Invalid_scalar_type s) ->
               Error (Invalid_scalar_type s)
           | Error (Invalid_core_environment msg) ->
               Error (Invalid_core_environment msg)
           | Error _ ->
               Error (Invalid_core_environment "parse failure")
           | Ok env ->
               (* Pass occurrence facts through unchanged.
                  CORE owns semantic Fact type validation. *)
               let facts =
                 match req.facts_json with
                 | `Assoc pairs -> pairs
                 | _ -> []
               in
               let input =
                 { evaluation_id = req.evaluation_id;
                   source = req.source;
                   event_name = req.event_name;
                   event_data = req.event_data;
                   facts }
               in
               let context =
                 { protocol_version = req.protocol_version;
                   language_version = req.language_version;
                   evaluation_id = req.evaluation_id;
                   event_id = req.event_id;
                   tether_id = req.tether_id;
                   tether_version = req.tether_version }
               in
               match evaluate env input with
               | Ok evaluation ->
                   Ok { context; evaluation }
                | Error adapter_err ->
                    Error (Adapter_error adapter_err))
       | _ -> Error (Invalid_core_environment "expected object"))
