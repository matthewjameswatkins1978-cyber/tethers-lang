let process_line line =
  try
    let request = Yojson.Safe.from_string line in
    Tethers_core_wire.evaluate_request_json request
  with
  | Yojson.Json_error message ->
      `Assoc [("status", `String "error"); ("error", `Assoc [("code", `String "invalid_json"); ("message", `String message)])]
  | exn ->
      `Assoc [("status", `String "error"); ("error", `Assoc [("code", `String "internal_error"); ("message", `String (Printexc.to_string exn))])]

let () =
  try
    while true do
      let line = input_line stdin in
      process_line line |> Yojson.Safe.to_string |> print_endline;
      flush stdout
    done
  with End_of_file -> ()
