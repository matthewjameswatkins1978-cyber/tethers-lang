let process_line line =
  try
    let response = Yojson.Safe.from_string line |> Tethers_evaluator.evaluate_request in
    Tethers_outcome.json_of_response response
  with
  | Tethers_error.Tethers_error (code, message) ->
      Tethers_outcome.json_of_response (Tethers_outcome.error_response code message)
  | Yojson.Json_error message ->
      Tethers_outcome.json_of_response (Tethers_outcome.error_response "invalid_json" message)
  | exn ->
      Tethers_outcome.json_of_response (Tethers_outcome.error_response "internal_error" (Printexc.to_string exn))

let () =
  try
    while true do
      let line = input_line stdin in
      process_line line |> Yojson.Safe.to_string |> print_endline;
      flush stdout
    done
  with End_of_file -> ()
