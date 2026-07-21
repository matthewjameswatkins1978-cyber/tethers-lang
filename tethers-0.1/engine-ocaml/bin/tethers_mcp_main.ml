let () =
  let rec loop () =
    try
      let line = input_line stdin in
      if String.length (String.trim line) > 0 then
        let msg = Yojson.Safe.from_string line in
        (match Tethers_mcp_server.handle_message msg with
         | Some response ->
             let json = Yojson.Safe.to_string response in
             output_string stdout json;
             output_string stdout "\n";
             flush stdout
         | None -> ());
        loop ()
      else
        loop ()
    with
    | End_of_file -> ()
    | exn ->
        let msg =
          Printf.sprintf
            "internal error processing input: %s\n"
            (Printexc.to_string exn)
        in
        output_string stderr msg;
        flush stderr;
        loop ()
  in
  loop ()