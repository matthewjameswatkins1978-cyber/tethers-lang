open Rocket_path_kernel

let rec nat_of_int n =
  if n = 0 then O else S (nat_of_int (n - 1))

let rec int_of_nat = function
  | O -> 0
  | S n -> 1 + int_of_nat n

let rec int_list = function
  | Nil -> []
  | Cons (head, tail) -> int_of_nat head :: int_list tail

let expected_chain_11 = [10; 9; 8; 7; 6; 5; 4; 3; 2; 1; 11]

let () =
  let path size =
    int_list (PathCanon.canonical_assignment (nat_of_int size))
  in
  if path 11 <> expected_chain_11 then exit 1;
  List.iter (fun size ->
    let result = path size in
    if List.length result <> size then exit 1
  ) [1; 2; 3; 4; 5; 6; 7; 8; 9; 10; 11; 12];
  let large = path 1000 in
  if List.length large <> 1000 then exit 1;
  print_endline "extracted-kernel-small-ok"
