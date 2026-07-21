let () =
  try
    while true do
      let line = input_line stdin in
      Tethers_evaluator.process_line line |> Yojson.Safe.to_string |> print_endline;
      flush stdout
    done
  with End_of_file -> ()
