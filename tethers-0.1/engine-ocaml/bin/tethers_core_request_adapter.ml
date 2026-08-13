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
(*  Object-safe JSON helpers                                           *)
(*                                                                    *)
(*  Every helper proves `json` is `Assoc` before extracting fields.   *)
(*  No call to Yojson.Safe.Util.member on a non-object value.         *)
(* ================================================================== *)

let ( >>= ) = Result.bind

(* Ensure a value is a JSON object; return its association list. *)
let expect_object context json =
  match json with
  | `Assoc fields -> Ok fields
  | _ -> Error (Invalid_request (context, "expected object"))

let expect_object_core context json =
  match json with
  | `Assoc fields -> Ok fields
  | _ -> Error (Invalid_core_environment (context ^ ": expected object"))

(* Extract a string field from an association list. *)
let field_string context fields name =
  match List.assoc_opt name fields with
  | Some (`String s) -> Ok s
  | Some _ -> Error (Invalid_request (context, name ^ ": expected string"))
  | None -> Error (Invalid_request (context, name ^ ": required"))

let field_string_core context fields name =
  match List.assoc_opt name fields with
  | Some (`String s) -> Ok s
  | Some _ ->
      Error (Invalid_core_environment (context ^ "." ^ name ^ ": expected string"))
  | None ->
      Error (Invalid_core_environment (context ^ "." ^ name ^ ": required"))

(* Extract a list field from an association list. *)
let field_list context fields name =
  match List.assoc_opt name fields with
  | Some (`List l) -> Ok l
  | Some _ -> Error (Invalid_request (context, name ^ ": expected array"))
  | None -> Error (Invalid_request (context, name ^ ": required"))

let field_list_core context fields name =
  match List.assoc_opt name fields with
  | Some (`List l) -> Ok l
  | Some _ ->
      Error (Invalid_core_environment (context ^ "." ^ name ^ ": expected array"))
  | None ->
      Error (Invalid_core_environment (context ^ "." ^ name ^ ": required"))

(* Extract a value that may be null; returns None for null. *)
let field_maybe_null _context fields name =
  match List.assoc_opt name fields with
  | None | Some `Null -> Ok None
  | Some v -> Ok (Some v)

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
   Binding must be an object with four required string fields. *)
let resolve_one_capability top_level_caps binding =
  expect_object_core "core_environment.capabilities" binding
  >>= fun fields ->
  field_string_core "core_environment.capabilities" fields "source_name"
  >>= fun source_name ->
  field_string_core "core_environment.capabilities" fields "capability_id"
  >>= fun capability_id_str ->
  field_string_core "core_environment.capabilities" fields "contract_digest"
  >>= fun contract_digest_str ->
  field_string_core "core_environment.capabilities" fields "runtime_name"
  >>= fun runtime_name ->
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
   Declaration must be an object with all required fields.
   schema_description is required (not optional). *)
let parse_one_fact fact_json =
  expect_object_core "core_environment.input_facts" fact_json
  >>= fun fields ->
  field_string_core "core_environment.input_facts" fields "source_name"
  >>= fun source_name ->
  field_string_core "core_environment.input_facts" fields "fact_id"
  >>= fun fact_id_str ->
  field_string_core "core_environment.input_facts" fields "host_snapshot_key"
  >>= fun host_snapshot_key_str ->
  field_string_core "core_environment.input_facts" fields "scalar_type"
  >>= fun scalar_type_str ->
  field_string_core "core_environment.input_facts" fields "schema_description"
  >>= fun schema_description ->
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
  expect_object_core "core_environment" env_json >>= fun fields ->
  field_string_core "core_environment" fields "program_id"
  >>= fun program_id_str ->
  field_string_core "core_environment" fields "core_version"
  >>= fun core_version_str ->
  field_list_core "core_environment" fields "capabilities"
  >>= fun cap_jsons ->
  resolve_capabilities top_level_caps cap_jsons >>= fun capabilities ->
  (field_maybe_null "core_environment" fields "input_facts" >>= function
   | None | Some `Null -> Ok []
   | Some (`List fact_jsons) -> parse_facts fact_jsons
   | Some _ ->
       Error (Invalid_core_environment "core_environment.input_facts: expected array"))
  >>= fun input_facts ->
  Ok { program_id = program_id_of_string program_id_str;
       core_version = core_version_of_string core_version_str;
       capabilities;
       input_facts }

(* ================================================================== *)
(*  Request parsing                                                    *)
(* ================================================================== *)

let parse_request request =
  expect_object "request" request >>= fun req_fields ->
  field_string "request" req_fields "protocol_version"
  >>= fun protocol_version ->
  field_string "request" req_fields "language_version"
  >>= fun language_version ->
  if protocol_version <> "0.1" then
    Error (Invalid_request
             ("protocol_version", "unsupported: " ^ protocol_version))
  else if language_version <> "0.1" then
    Error (Invalid_request
             ("language_version", "unsupported: " ^ language_version))
  else
    field_string "request" req_fields "evaluation_id"
    >>= fun evaluation_id ->
    (* tether must be an object *)
    (match List.assoc_opt "tether" req_fields with
     | Some tether_json ->
         expect_object "request.tether" tether_json >>= fun tether_fields ->
         field_string "request.tether" tether_fields "id" >>= fun tether_id ->
         field_string "request.tether" tether_fields "version"
         >>= fun tether_version ->
         field_string "request.tether" tether_fields "source" >>= fun source ->
         Ok (tether_id, tether_version, source)
     | None ->
         Error (Invalid_request ("tether", "required")))
    >>= fun (tether_id, tether_version, source) ->
    (* event must be an object *)
    (match List.assoc_opt "event" req_fields with
     | Some event_json ->
         expect_object "request.event" event_json >>= fun event_fields ->
         field_string "request.event" event_fields "id" >>= fun event_id ->
         field_string "request.event" event_fields "name" >>= fun event_name ->
         let event_data =
           match List.assoc_opt "data" event_fields with
           | Some v -> v | None -> `Null
         in
         Ok (event_id, event_name, event_data)
     | None ->
         Error (Invalid_request ("event", "required")))
    >>= fun (event_id, event_name, event_data) ->
    (* Require facts to be an object.  Missing or non-object is invalid. *)
    (match List.assoc_opt "facts" req_fields with
     | Some (`Assoc _ as obj) -> Ok obj
     | None | Some `Null ->
         Error (Invalid_request ("facts", "required object"))
     | Some _ ->
         Error (Invalid_request ("facts", "expected object")))
    >>= fun facts_json ->
    (* capabilities must be an array; each item must be an object *)
    field_list "request" req_fields "capabilities" >>= fun cap_jsons ->
    (try
       let top_level_caps =
         List.map (fun cap_json ->
           match cap_json with
           | `Assoc _ -> Tethers_protocol.parse_capability cap_json
           | _ -> raise (Tethers_error.Tethers_error
                           ("invalid_request",
                            "capabilities: each item must be an object"))
         ) cap_jsons
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
      (* core_environment: must be an object, not Null, not string, etc. *)
      (match List.assoc_opt "core_environment"
               (match request with `Assoc f -> f | _ -> []) with
       | None | Some `Null -> Error Missing_core_environment
       | Some (`Assoc _) as core_env_opt ->
           let core_env_json = match core_env_opt with Some v -> v | _ -> `Null in
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
       | Some _ -> Error (Invalid_core_environment "expected object"))
