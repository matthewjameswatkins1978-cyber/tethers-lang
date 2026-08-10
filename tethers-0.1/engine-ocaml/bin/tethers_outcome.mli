type error_details = {
  code : string;
  message : string;
}

type planned_action = Yojson.Safe.t
type trail_entry = Yojson.Safe.t

type group_plan = {
  group_id : string;
  member_action_ids : string list;
}

type plan = {
  id : string;
  required_effects : string list;
  actions : planned_action list;
  groups : group_plan list;
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

val error_response : string -> string -> response

val json_of_response : response -> Yojson.Safe.t
