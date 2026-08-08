exception Tethers_error of string * string

let fail code message =
  raise (Tethers_error (code, message))
