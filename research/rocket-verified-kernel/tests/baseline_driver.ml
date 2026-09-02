open Rocket_path_kernel

let rec nat_of_int n =
  if n = 0 then O else S (nat_of_int (n - 1))

let rec int_of_nat = function
  | O -> 0
  | S n -> 1 + int_of_nat n

let rec int_list = function
  | Nil -> []
  | Cons (head, tail) -> int_of_nat head :: int_list tail

let () =
  let size =
    match Array.to_list Sys.argv with
    | _ :: value :: _ -> int_of_string value
    | _ -> invalid_arg "baseline_driver SIZE"
  in
  let start = Unix.gettimeofday () in
  let result =
    int_list (PathCanon.canonical_assignment (nat_of_int size))
  in
  let elapsed = Unix.gettimeofday () -. start in
  Printf.printf "baseline size=%d result_length=%d elapsed_seconds=%.6f\n%!"
    size (List.length result) elapsed
