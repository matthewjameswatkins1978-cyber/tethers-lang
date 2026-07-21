open Tether_parser

type server_state =
  | Uninitialized
  | Initializing
  | Initialized

let server_state = ref Uninitialized

let supported_protocol_versions = ["2025-06-18"; "2025-11-25"]

let json_member_opt name fields = List.assoc_opt name fields

let make_response id result =
  `Assoc
    [
      ("jsonrpc", `String "2.0");
      ("id", id);
      ("result", result);
    ]

let make_error id code message data_opt =
  let error_fields = [ ("code", `Int code); ("message", `String message) ] in
  let error_fields =
    match data_opt with
    | None -> error_fields
    | Some data -> error_fields @ [ ("data", data) ]
  in
  `Assoc
    [
      ("jsonrpc", `String "2.0");
      ("id", id);
      ("error", `Assoc error_fields);
    ]

let handle_initialize id fields =
  let params =
    match json_member_opt "params" fields with
    | Some (`Assoc p) -> p
    | _ -> []
  in
  let protocol_version =
    match json_member_opt "protocolVersion" params with
    | Some (`String v) -> v
    | _ -> ""
  in
  if not (List.mem protocol_version supported_protocol_versions) then
    let data =
      `Assoc
        [
          ("requested", `String protocol_version);
          ( "supported",
            `List (List.map (fun v -> `String v) supported_protocol_versions) );
        ]
    in
    Some (make_error id (-32602) "Unsupported MCP protocol version" (Some data))
  else begin
    server_state := Initializing;
    let result =
      `Assoc
        [
          ("protocolVersion", `String protocol_version);
          ("capabilities", `Assoc [ ("tools", `Assoc []) ]);
          ( "serverInfo",
            `Assoc
              [ ("name", `String "tethers"); ("version", `String "0.1.0") ] );
        ]
    in
    Some (make_response id result)
  end

let handle_initialized () =
  if !server_state = Initializing then server_state := Initialized

let handle_ping id =
  if !server_state <> Initialized then
    Some (make_error id (-32002) "Server not initialized" None)
  else Some (make_response id (`Assoc []))

let handle_tools_list id =
  if !server_state <> Initialized then
    Some (make_error id (-32002) "Server not initialized" None)
  else
    let tools =
      `List
        [
          `Assoc
            [
              ("name", `String "tethers.evaluate");
              ( "description",
                `String
                  "Evaluate one complete Tethers 0.1 request and return the \
                   Tethers response envelope without executing Actions." );
              ( "inputSchema",
                `Assoc
                  [
                    ("type", `String "object");
                    ( "properties",
                      `Assoc
                        [
                          ( "request",
                            `Assoc
                              [
                                ("type", `String "object");
                                ( "description",
                                  `String
                                    "Complete Tethers 0.1 request envelope" );
                              ] );
                        ] );
                    ("required", `List [ `String "request" ]);
                    ("additionalProperties", `Bool false);
                  ] );
            ];
          `Assoc
            [
              ("name", `String "tethers.validate");
              ( "description",
                `String
                  "Validate Tethers 0.1 source syntax and structure without \
                   requiring event data or capabilities. Returns parse success \
                   or structured diagnostics." );
              ( "inputSchema",
                `Assoc
                  [
                    ("type", `String "object");
                    ( "properties",
                      `Assoc
                        [
                          ( "source",
                            `Assoc
                              [
                                ("type", `String "string");
                                ( "description",
                                  `String
                                    "Complete Tethers 0.1 source text" );
                              ] );
                        ] );
                    ("required", `List [ `String "source" ]);
                    ("additionalProperties", `Bool false);
                  ] );
            ];
        ]
    in
    Some (make_response id (`Assoc [ ("tools", tools) ]))

let handle_tools_call id fields =
  if !server_state <> Initialized then
    Some (make_error id (-32002) "Server not initialized" None)
  else
    let params =
      match json_member_opt "params" fields with
      | Some (`Assoc p) -> p
      | _ -> []
    in
    let tool_name =
      match json_member_opt "name" params with
      | Some (`String n) -> n
      | _ -> ""
    in
    if tool_name = "tethers.evaluate" then
      match json_member_opt "arguments" params with
      | Some (`Assoc args) -> (
          match json_member_opt "request" args with
          | Some request ->
              let tethers_response =
                try Tethers_evaluator.evaluate_request request with
                | Tethers_error (code, message) ->
                    Tethers_evaluator.error_response code message
                | Yojson.Json_error message ->
                    Tethers_evaluator.error_response "invalid_json" message
                | exn ->
                    Tethers_evaluator.error_response "internal_error"
                      (Printexc.to_string exn)
              in
              let compact_json = Yojson.Safe.to_string tethers_response in
              let result =
                `Assoc
                  [
                    ("structuredContent", tethers_response);
                    ( "content",
                      `List
                        [
                          `Assoc
                            [
                              ("type", `String "text");
                              ("text", `String compact_json);
                            ];
                        ] );
                    ("isError", `Bool false);
                  ]
              in
              Some (make_response id result)
          | None ->
              Some
                (make_error id (-32602)
                   "Invalid arguments for tethers.evaluate: expected object \
                    field request"
                   None))
      | _ ->
          Some
            (make_error id (-32602)
               "Invalid arguments for tethers.evaluate: expected object field \
                request"
               None)
    else if tool_name = "tethers.validate" then
      match json_member_opt "arguments" params with
      | Some (`Assoc args) -> (
          match json_member_opt "source" args with
          | Some (`String source) -> (
              let validate_response =
                try
                  let parsed = parse_tether source in
                  `Assoc
                    [
                      ("valid", `Bool true);
                      ("title", `String parsed.title);
                      ("anchor", `String parsed.anchor);
                      ( "condition_count",
                        `Int (List.length parsed.conditions) );
                      ("action_count", `Int (List.length parsed.actions));
                    ]
                with
                | Tethers_error (code, message) ->
                    `Assoc
                      [
                        ("valid", `Bool false);
                        ( "error",
                          `Assoc
                            [
                              ("code", `String code);
                              ("message", `String message);
                            ] );
                      ]
                | exn ->
                    `Assoc
                      [
                        ("valid", `Bool false);
                        ( "error",
                          `Assoc
                            [
                              ("code", `String "internal_error");
                              ("message", `String (Printexc.to_string exn));
                            ] );
                      ]
              in
              let compact_json = Yojson.Safe.to_string validate_response in
              let result =
                `Assoc
                  [
                    ("structuredContent", validate_response);
                    ( "content",
                      `List
                        [
                          `Assoc
                            [
                              ("type", `String "text");
                              ("text", `String compact_json);
                            ];
                        ] );
                    ("isError", `Bool false);
                  ]
              in
              Some (make_response id result))
          | _ ->
              Some
                (make_error id (-32602)
                   "Invalid arguments for tethers.validate: expected object \
                    field source"
                   None))
      | _ ->
          Some
            (make_error id (-32602)
               "Invalid arguments for tethers.validate: expected object field \
                source"
               None)
    else
      Some (make_error id (-32602) ("Unknown tool: " ^ tool_name) None)

let handle_message msg =
  try
    match msg with
    | `Assoc fields ->
        let id_opt = json_member_opt "id" fields in
        let method_opt = json_member_opt "method" fields in
        (match (method_opt, id_opt) with
        | Some (`String "initialize"), Some id -> handle_initialize id fields
        | Some (`String "notifications/initialized"), None ->
            handle_initialized ();
            None
        | Some (`String "ping"), Some id -> handle_ping id
        | Some (`String "tools/list"), Some id -> handle_tools_list id
        | Some (`String "tools/call"), Some id -> handle_tools_call id fields
        | Some _, Some id ->
            Some (make_error id (-32601) "Method not found" None)
        | Some _, None ->
            (* Unknown notification — silently ignore *)
            None
        | None, Some id ->
            Some (make_error id (-32600) "Invalid Request" None)
        | None, None -> None)
    | _ -> None
  with
  | exn ->
      let id =
        match msg with
        | `Assoc fields -> (
            match json_member_opt "id" fields with Some id -> id | None -> `Null)
        | _ -> `Null
      in
      Some
        (make_error id (-32603)
           ("Internal error: " ^ Printexc.to_string exn)
           None)