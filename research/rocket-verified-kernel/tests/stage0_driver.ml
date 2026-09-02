let rec of_int n =
  if n = 0 then Stage0_kernel.O else Stage0_kernel.S (of_int (n - 1))

let rec to_int = function
  | Stage0_kernel.O -> 0
  | Stage0_kernel.S n -> 1 + to_int n

let () =
  if to_int (Stage0_kernel.Stage0.successor (of_int 41)) = 42 then
    print_endline "stage0-ok"
  else
    exit 1
