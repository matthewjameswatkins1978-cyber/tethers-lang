type value =
  | String_value of string
  | Int_value of int
  | Bool_value of bool
  | Reference of string

type operator =
  | Is
  | Contains
  | Greater_than
  | Greater_than_or_equal

type condition = {
  fact : string;
  operator : operator;
  expected : value;
  source : string;
}

type action = {
  capability : string;
  arguments : (string * value) list;
}

type tether = {
  title : string;
  anchor : string;
  conditions : condition list;
  actions : action list;
}

val drop_prefix : string -> string -> string

val parse_tether : string -> tether
