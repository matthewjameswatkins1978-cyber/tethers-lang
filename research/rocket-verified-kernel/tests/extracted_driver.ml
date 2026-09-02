open Rocket_path_kernel

let expected_chain_11 = [10; 9; 8; 7; 6; 5; 4; 3; 2; 1; 11]

let () =
  let path size =
    PathCanon.canonical_assignment size
  in
  if path 11 <> expected_chain_11 then exit 1;
  List.iter (fun size ->
    let result = path size in
    if List.length result <> size then exit 1
  ) [1; 2; 3; 4; 5; 6; 7; 8; 9; 10; 11; 12];
  let large = path 1000 in
  if List.length large <> 1000 then exit 1;
  print_endline "extracted-kernel-small-ok"
