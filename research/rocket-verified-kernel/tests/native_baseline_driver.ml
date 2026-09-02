open Rocket_path_kernel_native

let () =
  let size =
    match Array.to_list Sys.argv with
    | _ :: value :: _ -> int_of_string value
    | _ -> invalid_arg "native_baseline_driver SIZE"
  in
  let start = Unix.gettimeofday () in
  let result = PathCanon.canonical_assignment size in
  let elapsed = Unix.gettimeofday () -. start in
  Printf.printf "native-baseline size=%d result_length=%d elapsed_seconds=%.6f\n%!"
    size (List.length result) elapsed
