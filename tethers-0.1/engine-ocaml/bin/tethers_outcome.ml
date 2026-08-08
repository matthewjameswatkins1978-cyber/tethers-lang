type error_details = {
  code : string;
  message : string;
}

type planned_action = Yojson.Safe.t

type trail_entry = Yojson.Safe.t

type plan = {
  id : string;
  required_effects : string list;
  actions : planned_action list;
}

type evaluation_context = {
  evaluation_id : string;
  event_id : string;
  tether_id : string;
  tether_version : string;
}

type status_payload =
  | Matched of plan
  | Not_matched
  | Evaluation_error of error_details

type contextual_result = {
  context : evaluation_context;
  payload : status_payload;
  trail : trail_entry list;
}

type response =
  | Contextual of contextual_result
  | Request_error of error_details

let json_of_response = function
  | Contextual
      {
        context = { evaluation_id; event_id; tether_id; tether_version };
        payload;
        trail;
      } ->
      let fields =
        [
          ("protocol_version", `String "0.1");
          ("evaluation_id", `String evaluation_id);
          ("event_id", `String event_id);
          ("tether_id", `String tether_id);
          ("tether_version", `String tether_version);
        ]
      in
      let status, plan, error_field, trail_field =
        match payload with
        | Matched { id; required_effects; actions } ->
            ( "matched",
              `Assoc
                [
                  ("id", `String id);
                  ( "required_effects",
                    `List (List.map (fun e -> `String e) required_effects) );
                  ("actions", `List actions);
                ],
              [],
              [ ("trail", `List trail) ] )
        | Not_matched ->
            ("not_matched", `Null, [], [ ("trail", `List trail) ])
        | Evaluation_error { code; message } ->
            ( "error",
              `Null,
              [
                ( "error",
                  `Assoc
                    [ ("code", `String code); ("message", `String message) ] );
              ],
              [ ("trail", `List trail) ] )
      in
      `Assoc (fields @ [ ("status", `String status); ("plan", plan) ] @ error_field @ trail_field)
  | Request_error { code; message } ->
      `Assoc
        [
          ("protocol_version", `String "0.1");
          ("status", `String "error");
          ( "error",
            `Assoc [ ("code", `String code); ("message", `String message) ] );
        ]

let error_response code message = Request_error { code; message }
